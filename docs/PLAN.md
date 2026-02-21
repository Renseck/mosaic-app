# Mosaic App — Phased Implementation Plan

## Current State

The project has a complete file structure scaffolded (all files exist but are empty). Both `Cargo.toml` files have package metadata but no dependencies. `docker-compose.yml`, `main.rs`, `config.rs`, `error.rs`, etc. are all empty. This plan starts from zero.

---

## Phase 1: Foundation [DONE]

**Goal**: Docker Compose runs Postgres + Grafana + NocoDB. The Axum backend starts, connects to Postgres, runs migrations to create the `portal` schema, and responds to a health check. Everything compiles and connects.

### Files to create/modify

| File | Action |
|------|--------|
| `src/docker-compose.yml` | Write full compose with postgres, grafana, nocodb services (no portal service yet — we run locally) |
| `src/.env` / `src/.env.example` | Populate with all config vars (DATABASE_URL, GRAFANA_*, NOCODB_*, SESSION_*, BIND_ADDRESS) |
| `src/backend/Cargo.toml` | Add dependencies: axum, tokio, sqlx (postgres, runtime-tokio, tls-rustls, migrate), serde, serde_json, dotenvy, tracing, tracing-subscriber, thiserror, anyhow, uuid, chrono |
| `src/backend/src/main.rs` | Bootstrap: load config, init tracing, create DB pool, run migrations, build router (health check only), start server |
| `src/backend/src/config.rs` | `AppConfig` struct with all env vars, loaded via `dotenvy` + manual parsing or `envy` |
| `src/backend/src/error.rs` | `AppError` enum with all variants + `IntoResponse` impl (exact code from CLAUDE.md) |
| `src/backend/src/db/mod.rs` | Re-export pool module |
| `src/backend/src/db/pool.rs` | `create_pool(database_url) -> PgPool` function |
| `src/backend/src/db/migrations/` | `001_initial_schema.sql` — creates `portal` schema + `users`, `sessions`, `dashboards`, `panels`, `dataset_templates` tables |
| `src/backend/src/api/mod.rs` | Empty router composition (just health check for now) |

### Acceptance criteria

1. `docker compose up postgres grafana nocodb` starts all three services, Postgres healthcheck passes
2. `cd src/backend && cargo run` compiles and starts the Axum server on port 8080
3. `curl http://localhost:8080/api/health` returns `200 OK` with `{"status": "ok"}`
4. Backend logs show successful DB connection and migration run
5. Connecting to Postgres and running `\dt portal.*` shows all 5 tables

### Open questions

- **Grafana database**: The architecture doc shows Grafana using the same Postgres instance (`grafana` database) vs a separate SQLite. Using shared Postgres is cleaner — but Grafana needs its own database (not just schema) since it manages its own migrations. We should create a separate `grafana` database in the Postgres init script, or let Grafana use its built-in SQLite. **Recommendation**: Use a Postgres init script that creates both `portal_db` and `grafana` databases under the same instance.
- **NocoDB port**: The `.env.example` says `8090` but the docker-compose outline in architecture.md uses `8080` for NocoDB internally. We need to pick one — suggest `8090` for NocoDB to avoid collisions with the portal's `8080`. Update NocoDB's config accordingly.
- **Edition 2024**: The Cargo.toml uses `edition = "2024"` and `rust-version = "1.93.1"`. This is very new. If any dependency doesn't support it yet, we may need to fall back to `edition = "2021"`. Keep an eye on this during Phase 1.

---

## Phase 2: Auth [DONE]

**Goal**: Users can register (first user becomes admin), log in, receive a session cookie, and access protected routes. The `AuthenticatedUser` and `RequireAdmin` extractors work.

### Files to create/modify

| File | Action |
|------|--------|
| `src/backend/Cargo.toml` | Add: argon2, rand, sha2, hex, cookie, tower-cookies (or axum-extra with cookie feature) |
| `src/backend/src/auth/mod.rs` | Re-export all auth submodules |
| `src/backend/src/auth/password.rs` | `hash_password(plain) -> String` and `verify_password(plain, hash) -> bool` using argon2id |
| `src/backend/src/auth/session.rs` | `create_session(pool, user_id) -> (token, cookie)` and `validate_session(pool, token_hash) -> Option<AuthenticatedUser>`. Token = random bytes, stored as SHA-256 hash |
| `src/backend/src/auth/handlers.rs` | `POST /register`, `POST /login`, `POST /logout`, `GET /me` handlers |
| `src/backend/src/auth/middleware.rs` | `AuthenticatedUser` extractor (FromRequestParts) + `RequireAdmin` extractor |
| `src/backend/src/main.rs` | Mount auth routes under `/api/auth/*` |

### Acceptance criteria

1. `POST /api/auth/register` with `{"username": "admin", "password": "test1234"}` succeeds and returns user with role `admin`
2. A second registration attempt is rejected with 403 (unless authenticated as admin)
3. `POST /api/auth/login` returns a `Set-Cookie` header with an HttpOnly session cookie
4. `GET /api/auth/me` with the session cookie returns the current user
5. `GET /api/auth/me` without a cookie returns 401
6. `POST /api/auth/logout` invalidates the session
7. Backend test: password hash round-trip works

### Open questions

- **Session storage**: Cookie name — suggest `portal_session`. Max-Age / TTL from `SESSION_TTL_HOURS` env var.
- **Admin-only registration enforcement**: After the first user, `POST /register` requires `RequireAdmin`. Should we allow self-registration with a config flag later? **Recommendation**: No — keep it admin-only for now, matching the architecture doc. Self-registration can be a future feature.

---

## Phase 3: Reverse Proxy [DONE]

**Goal**: Authenticated users can access Grafana and NocoDB through the Axum proxy. The proxy strips the prefix path, forwards requests to the internal service, and injects service-level auth.

### Files to create/modify

| File | Action |
|------|--------|
| `src/backend/Cargo.toml` | Add: reqwest (with stream feature), hyper, http-body-util |
| `src/backend/src/proxy/mod.rs` | `ProxyTarget` trait definition + shared proxy handler function |
| `src/backend/src/proxy/grafana.rs` | `GrafanaProxy` implementing `ProxyTarget` — strips `/proxy/grafana`, injects `Authorization: Bearer <service_account_token>` |
| `src/backend/src/proxy/nocodb.rs` | `NocodbProxy` implementing `ProxyTarget` — strips `/proxy/nocodb`, injects `xc-token` header |
| `src/backend/src/main.rs` | Mount proxy routes: `/proxy/grafana/*` and `/proxy/nocodb/*`, both behind `AuthenticatedUser` |

### Acceptance criteria

1. While logged in: `curl -b cookie.txt http://localhost:8080/proxy/grafana/api/health` returns Grafana's health response
2. While logged in: `curl -b cookie.txt http://localhost:8080/proxy/nocodb/api/v1/health` returns NocoDB's health response
3. Without a session cookie: proxy routes return 401
4. Grafana's own port (3000) is NOT accessible from the host machine
5. Response headers (Content-Type, etc.) are correctly forwarded
6. WebSocket upgrade works for Grafana live features (or is at least not broken — can defer WS to polish)

### Open questions

- **Request body forwarding**: Need to handle streaming bodies for large uploads (e.g., NocoDB file attachments). `reqwest`'s streaming body should handle this, but worth testing.
- **Grafana sub-path**: Grafana is configured with `GF_SERVER_ROOT_URL` pointing to `/proxy/grafana/`. This means Grafana generates links relative to that path. Need to verify that Grafana's internal link generation works correctly with this setup or if we need to rewrite response bodies. **Recommendation**: Test with the config as-is; Grafana's `serve_from_sub_path` + `root_url` settings should handle this.
- **NocoDB sub-path**: NocoDB may not have an equivalent sub-path setting. If its frontend hardcodes `/` paths, we may need to rewrite HTML responses. **Recommendation**: Test first — NocoDB's shared views (form/grid) are simpler and may work fine as iframed content.

---

## Phase 4: Dashboard & Panel CRUD [DONE]

**Goal**: Full REST API for dashboards and panels, including batch position updates. All routes are auth-protected. Repository pattern with Postgres implementations.

### Files to create/modify

| File | Action |
|------|--------|
| `src/backend/src/api/mod.rs` | Compose dashboard, panel, user routes |
| `src/backend/src/api/dashboards.rs` | `GET /`, `POST /`, `GET /:slug`, `PUT /:id`, `DELETE /:id` handlers — thin wrappers calling repo |
| `src/backend/src/api/panels.rs` | `GET /dashboards/:id/panels`, `POST /dashboards/:id/panels`, `PUT /panels/:id`, `PUT /panels/:id/position`, `PUT /panels/batch-position`, `DELETE /panels/:id` |
| `src/backend/src/api/users.rs` | `GET /users` (admin), `PUT /users/:id/role` (admin) |
| `src/backend/src/db/mod.rs` | Add repo trait definitions + Pg implementations |
| New: `src/backend/src/db/repos/` | `dashboard_repo.rs`, `panel_repo.rs`, `user_repo.rs`, `template_repo.rs` — traits + `Pg*Repo` impls |
| `src/backend/src/main.rs` | Build `AppState` with all repos, mount API routes |
| `src/backend/tests/api_tests.rs` | Integration tests for dashboard + panel CRUD |

### Acceptance criteria

1. Create a dashboard → returns 201 with slug auto-generated from title
2. List dashboards → returns only dashboards owned by the user or marked `is_shared`
3. Get dashboard by slug → returns dashboard with its panels
4. Update dashboard → 200, verify changes persisted
5. Delete dashboard → cascades to panels
6. Add panels to a dashboard with grid positions → verify positions stored
7. Batch position update → send array of `{id, grid_x, grid_y, grid_w, grid_h}`, all update atomically
8. Admin can list users and change roles; non-admin gets 403
9. Integration tests pass with a test database

### Open questions

- **Slug generation**: Auto-generate from title (e.g., `my-dashboard` from `My Dashboard`)? Or let the user specify? **Recommendation**: Auto-generate with a `slugify` function, but allow override in the `PUT` update.
- **Repo file location**: The architecture doc shows repos under `db/`, but having a `db/repos/` subdirectory is cleaner for multiple repo files. This is a minor structural deviation from the doc's flat `db/` layout. **Recommendation**: Use `db/repos/` — it's a refinement, not a contradiction.
- **Batch position endpoint**: The route `PUT /api/panels/batch-position` needs to be mounted before `/api/panels/:id` to avoid the `:id` segment matching `batch-position`. Axum's router handles this correctly since exact matches take priority over parameterized segments, but worth verifying.

---

## Phase 5: Yew SPA Shell [DONE]

**Goal**: The Yew frontend compiles to WASM, is served by Axum's SPA fallback, and renders a login page + authenticated app shell (sidebar + topbar + content area). Login works end-to-end through the browser.

### Files to create/modify

| File | Action |
|------|--------|
| `src/frontend/Cargo.toml` | Add dependencies: yew, yew-router, wasm-bindgen, gloo (gloo-net, gloo-timers, gloo-storage), web-sys, js-sys, serde, serde_json |
| `src/frontend/Trunk.toml` | Configure Trunk build: dist dir, proxy for dev, tailwind plugin |
| `src/frontend/index.html` | HTML shell: load WASM, include Tailwind CSS CDN (or built CSS) |
| `src/frontend/tailwind.config.js` | Tailwind config scoped to Yew component files |
| `src/frontend/static/main.css` | Base Tailwind directives (`@tailwind base; components; utilities;`) |
| `src/frontend/src/main.rs` | Mount `<App />` to DOM |
| `src/frontend/src/app.rs` | Root component: wraps router in `AuthContextProvider` + `ThemeContextProvider` |
| `src/frontend/src/router.rs` | Route enum: Login, DashboardList, DashboardView, TemplateList, TemplateNew, Settings. Switch component |
| `src/frontend/src/context/auth_context.rs` | `AuthState` + `AuthContextProvider` component. On mount: call `GET /api/auth/me` to check session |
| `src/frontend/src/context/theme_context.rs` | Minimal theme context (light/dark toggle — can be a stub) |
| `src/frontend/src/models/user.rs` | `User`, `Role` structs (serde Deserialize) |
| `src/frontend/src/models/mod.rs` | Re-exports |
| `src/frontend/src/api/client.rs` | Base HTTP client wrapper using `gloo-net`: `get()`, `post()`, `put()`, `delete()` with JSON |
| `src/frontend/src/api/auth.rs` | `login()`, `logout()`, `me()` API calls |
| `src/frontend/src/api/mod.rs` | Re-exports |
| `src/frontend/src/components/auth/login_page.rs` | Login form: username + password, calls login API, on success sets auth context + navigates to `/dashboards` |
| `src/frontend/src/components/layout/shell.rs` | App shell layout: sidebar left, topbar top, content area |
| `src/frontend/src/components/layout/sidebar.rs` | Navigation links (dashboards, templates, settings), user info at bottom |
| `src/frontend/src/components/layout/topbar.rs` | Breadcrumbs, user avatar/dropdown, logout button |
| `src/frontend/src/components/mod.rs` | Re-exports for all component modules |
| `src/frontend/src/hooks/use_auth.rs` | `use_auth()` hook — convenience wrapper for auth context |
| `src/backend/src/spa.rs` | Serve `frontend/dist/` as static files + SPA fallback (any non-API/proxy route returns `index.html`) |
| `src/backend/src/main.rs` | Mount SPA fallback as the catch-all route |

### Acceptance criteria

1. `cd src/frontend && trunk build --release` compiles WASM and produces `dist/` with `index.html` + `.wasm` + `.js`
2. Backend serves the SPA at `http://localhost:8080/` — browser loads the Yew app
3. Unauthenticated users see the login page
4. Login with valid credentials → redirected to dashboard list (empty page with shell)
5. Shell renders: sidebar with nav links, topbar with username + logout
6. Logout works: clears session, redirects to login
7. Direct navigation to `/dashboards` while unauthenticated redirects to `/login`
8. Page refresh while authenticated maintains session (cookie persists)

### Open questions

- **Tailwind in Yew**: Yew uses Rust macros for HTML (`html!{}`), not template files. Tailwind's JIT scanner needs to find class names. Options: (a) configure Tailwind to scan `.rs` files for class strings, (b) use a Trunk plugin for Tailwind, (c) use the Tailwind CDN (play script) for dev simplicity. **Recommendation**: Use Tailwind CLI as a Trunk hook (pre-build step) scanning `src/**/*.rs` for class names. The CDN play script is fine for initial development but won't tree-shake.
- **Dev workflow**: During development, do we run `trunk serve` with proxy to the backend? Or build frontend and let Axum serve it? **Recommendation**: Use `trunk serve --proxy-backend=http://localhost:8080/api` for hot reload during frontend dev. For integration testing, build frontend and serve via Axum.
- **SPA fallback routing**: The Axum SPA handler needs to serve `index.html` for any path that doesn't match `/api/*` or `/proxy/*`. This enables client-side routing. Use `tower-http`'s `ServeDir` with a fallback to `ServeFile("index.html")`.

---

## Phase 6: Dashboard Grid [DONE]

**Goal**: Users can view dashboards with a drag-and-drop grid of panels. Panels render Grafana charts and NocoDB views in iframes. Grid changes persist to the backend.

### Files to create/modify

| File | Action |
|------|--------|
| `src/frontend/static/` | Add `gridstack-all.js` and `gridstack.min.css` (vendored or CDN link in index.html) |
| `src/frontend/index.html` | Add gridstack JS/CSS includes |
| `src/frontend/src/components/grid/grid_engine.rs` | `wasm_bindgen` bindings for GridStack: `init()`, `on()`, `make_widget()`, `remove_widget()`, `update()` |
| `src/frontend/src/components/grid/dashboard_grid.rs` | Yew component: renders panel divs with `gs-*` attributes, initializes GridStack on mount, listens for `change` events → calls batch-position API |
| `src/frontend/src/components/grid/grid_item.rs` | Individual grid item wrapper: renders panel content inside gridstack-managed div |
| `src/frontend/src/components/panels/panel_frame.rs` | Generic panel chrome: title bar, drag handle, settings menu, content area |
| `src/frontend/src/components/panels/grafana_panel.rs` | Iframe pointing to `/proxy/grafana/d-solo/...` with correct query params |
| `src/frontend/src/components/panels/nocodb_panel.rs` | Iframe pointing to `/proxy/nocodb/#/nc/form/...` or `/nc/view/...` |
| `src/frontend/src/components/panels/markdown_panel.rs` | Render static markdown/HTML content |
| `src/frontend/src/components/panels/panel_picker.rs` | Modal: choose panel type (Grafana panel, Grafana dashboard, NocoDB form, NocoDB grid, Markdown), configure source URL, add to grid |
| `src/frontend/src/models/dashboard.rs` | `Dashboard`, `Panel`, `PanelType`, `GridPosition` structs |
| `src/frontend/src/api/dashboards.rs` | API client calls for dashboard + panel CRUD |
| `src/frontend/src/hooks/use_api.rs` | Generic fetch hook with loading/error/data states |
| New: page component for dashboard view | `DashboardPage` that loads dashboard by slug, passes panels to `DashboardGrid` |
| New: page component for dashboard list | `DashboardListPage` with create button |

### Acceptance criteria

1. Navigate to `/dashboards/my-dashboard` → loads dashboard with panels in a grid layout
2. Drag a panel to a new position → grid reflows, position is saved to backend via batch-position API
3. Resize a panel → new dimensions saved
4. Click "Add Panel" → panel picker modal opens → choose type → configure → panel appears in grid
5. Grafana panel iframe loads and displays chart content (requires a Grafana dashboard to exist)
6. NocoDB form iframe loads and displays form (requires a NocoDB form to exist)
7. Markdown panel renders HTML content inline
8. Delete a panel from the grid → removed from backend
9. Grid layout persists across page refreshes

### Open questions

- **GridStack version**: gridstack.js v10+ has a different API than v7. Need to pick a version and match the `wasm_bindgen` bindings. **Recommendation**: Use latest stable (v10+) — it's vanilla JS, framework-agnostic, and well-documented.
- **Panel picker UX**: How does the user specify a Grafana panel URL? Options: (a) paste the full Grafana panel URL, (b) browse Grafana dashboards/panels via the Grafana API, (c) select from templates created via the orchestrator. **Recommendation**: Start with (a) paste URL — it's simplest. The orchestrator (Phase 7) will later automate panel creation. Browsing Grafana dashboards can be a Phase 8 enhancement.
- **Edit mode**: Should the grid always be editable, or should there be a view/edit toggle? **Recommendation**: Add an "Edit" button in the topbar that toggles gridstack's `static` mode. Default to view mode (panels are locked). This prevents accidental drags while still being simple to implement.

---

## Phase 7: Orchestrator

**Goal**: The provisioner pipeline creates NocoDB tables + forms and Grafana dashboards from a template definition. The template wizard in the frontend drives this end-to-end.

### Files to create/modify

| File | Action |
|------|--------|
| `src/backend/Cargo.toml` | reqwest is already added (Phase 3) |
| `src/backend/src/orchestrator/mod.rs` | `Orchestrator` struct + `provision_dataset()` pipeline method |
| `src/backend/src/orchestrator/nocodb_client.rs` | `NocodbClient`: create_table, create_columns, create_form_view, delete_table. Thin reqwest wrappers over NocoDB v2 API |
| `src/backend/src/orchestrator/grafana_client.rs` | `GrafanaClient`: create_dashboard (with time-series panels per numeric field), delete_dashboard. Thin reqwest wrappers over Grafana HTTP API |
| `src/backend/src/orchestrator/provisioner.rs` | Type-state pipeline: `New → TableCreated → FormCreated → DashboardCreated → Registered`. Each step returns the next state |
| `src/backend/src/db/repos/template_repo.rs` | `TemplateRepo` trait + `PgTemplateRepo` — CRUD for `dataset_templates` |
| `src/backend/src/api/templates.rs` | `GET /templates`, `POST /templates` (triggers provisioning), `GET /templates/:id`, `DELETE /templates/:id` |
| `src/frontend/src/models/template.rs` | `DatasetTemplate`, `FieldDefinition`, `CreateTemplateRequest` |
| `src/frontend/src/api/templates.rs` | API client calls for template CRUD |
| `src/frontend/src/components/templates/template_list.rs` | List page showing all templates with status |
| `src/frontend/src/components/templates/template_wizard.rs` | Multi-step wizard: Name → Define Fields → Preview → Submit |
| `src/frontend/src/components/templates/field_editor.rs` | Add/edit/reorder/delete fields. Each field: name (validated alphanumeric+underscore), type (number/text/date/select), unit (optional) |
| `src/backend/tests/provisioner_tests.rs` | Integration tests against real NocoDB + Grafana (requires Docker services running) |

### Acceptance criteria

1. Create template via API with fields → NocoDB table created with matching columns
2. NocoDB form view created and accessible via proxy
3. Grafana dashboard created with a time-series panel per numeric field
4. Template record in `portal.dataset_templates` has all external IDs populated
5. Template wizard in frontend: fill in name, add fields, submit → see success message with links
6. After provisioning: can add the Grafana dashboard as a panel in a portal dashboard, and it renders live data
7. Can enter data via the NocoDB form (through proxy), and see it appear in the Grafana chart
8. Delete template → cleans up NocoDB table and Grafana dashboard (or at least marks as deleted)

### Open questions

- **NocoDB API version**: NocoDB v2 API changed significantly from v1. We need to target v2 (`/api/v2/meta/...`). **Recommendation**: Pin a specific NocoDB Docker image version and document it.
- **Grafana datasource**: The Grafana time-series panels need a Postgres datasource configured pointing at the NocoDB tables. This datasource needs to be provisioned too (or pre-configured). **Recommendation**: Pre-configure a Postgres datasource in Grafana via provisioning file (mounted volume) that points to the shared Postgres instance. The orchestrator then creates dashboards referencing this datasource. Add a Grafana provisioning config file to the docker-compose setup.
- **Cleanup on delete**: Deleting a template could leave orphan NocoDB tables and Grafana dashboards. Should we clean up? **Recommendation**: Yes, best-effort cleanup. Call NocoDB delete table + Grafana delete dashboard. Log failures but don't block the portal delete.
- **Field type mapping**: How do NocoDB column types map to Grafana panel types? Numeric fields → time-series chart. Text fields → table. Date fields → x-axis. **Recommendation**: Start simple — only auto-create panels for numeric fields. Other fields exist in NocoDB but don't get Grafana panels by default.
- **Auto-create portal dashboard**: Step 5 of the pipeline ("optionally create a portal dashboard page") — should this be automatic or manual? **Recommendation**: Automatic. After provisioning, create a portal dashboard with the Grafana dashboard + NocoDB form as panels. The user can then customize it.

---

## Phase 8: Polish

**Goal**: Production readiness. User management UI, theme toggle, responsive layout, error handling UX, loading states, and general cleanup.

### Files to create/modify

| File | Action |
|------|--------|
| `src/frontend/src/components/common/toast.rs` | Toast notification system (success/error/info) |
| `src/frontend/src/components/common/loading.rs` | Loading spinner / skeleton components |
| `src/frontend/src/components/common/modal.rs` | Reusable modal component (may already be partially done from panel picker) |
| `src/frontend/src/components/common/dropdown.rs` | Dropdown menu component |
| `src/frontend/src/components/common/icon.rs` | Icon component (inline SVGs or icon library) |
| `src/frontend/src/context/theme_context.rs` | Full light/dark theme implementation with `prefers-color-scheme` support |
| New: settings page component | User settings: theme preference, change password |
| New: admin user management page | List users, change roles, create new users (admin only) |
| `src/frontend/src/components/layout/sidebar.rs` | Responsive: collapsible on mobile, dashboard list dynamically loaded |
| `src/frontend/src/components/layout/shell.rs` | Responsive breakpoints, mobile-friendly layout |
| `Dockerfile` | Multi-stage build: backend + frontend → slim runtime image |
| `src/docker-compose.yml` | Add the `portal` service (built from Dockerfile) |
| `Makefile` | Dev commands: `make dev`, `make build`, `make test`, `make docker-up` |

### Acceptance criteria

1. Error toast appears when an API call fails (network error, 4xx, 5xx)
2. Loading spinners show while data is being fetched
3. Theme toggle switches between light and dark mode, preference persists (localStorage)
4. Admin can see user management page: list users, change roles
5. Admin can create new users from the UI
6. Layout is usable on mobile (sidebar collapses to hamburger menu)
7. `docker compose up` starts the full stack including the portal service
8. `Dockerfile` produces a working image under 100MB
9. No compiler warnings in either backend or frontend
10. All existing tests pass

### Open questions

- **Icon library**: Options: Heroicons (SVG, MIT), Lucide, or inline SVGs. **Recommendation**: Heroicons as inline SVG components — no JS dependency, works perfectly with Yew's `html!{}` macro.
- **Toast implementation**: Use a context provider + portal pattern? Or simpler CSS-only approach? **Recommendation**: Context provider (`ToastContext`) with a fixed-position container. Components dispatch toast actions, the provider renders them with auto-dismiss timers.
- **Change password**: Should this be in Phase 8 or deferred? **Recommendation**: Include it — it's a simple handler + form and important for a multi-user app.

---

## Phase ordering assessment

The proposed ordering is sound. Each phase builds on the previous one and produces something testable. A few notes:

1. **Phases 1-3 are correctly sequenced** — you need the DB before auth, and auth before the proxy (since proxy routes require authentication).

2. **Phase 4 before Phase 5 is good** — having a fully tested API before building the frontend means the frontend can be built against a stable contract. You can use curl/httpie to verify everything before touching Yew.

3. **Phase 5 before Phase 6 is correct** — the app shell, routing, and auth context need to exist before you can build dashboard views on top of them.

4. **Phase 7 (Orchestrator) after Phase 6 is correct** — you need the dashboard grid to be working before the orchestrator can auto-create dashboards with panels. Without the grid, there's no way to see the provisioned content.

5. **No phase needs splitting** — each phase is scoped to about 1-2 days of focused work. Phase 6 (Dashboard Grid) is the largest and could be split into "grid rendering" and "panel picker + editing", but it's more satisfying to deliver as one unit since a read-only grid without panel management isn't very useful.

---

## Dependency summary

```
Phase 1 (Foundation)
  └─► Phase 2 (Auth)
       └─► Phase 3 (Proxy)
       └─► Phase 4 (CRUD)
            └─► Phase 5 (SPA Shell)
                 └─► Phase 6 (Grid)
                      └─► Phase 7 (Orchestrator)
                           └─► Phase 8 (Polish)
```

Phases 3 and 4 can technically be built in parallel (both depend on Phase 2 but not on each other), but doing 3 first is simpler since the proxy code is smaller and self-contained.
