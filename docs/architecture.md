# Personal Data Portal — Architecture Document

## 1. System Overview

A self-hosted personal data portal that unifies data entry (via NocoDB), visualization (via Grafana), and navigation/orchestration (via a custom Rust application) into a single cohesive interface.

### Container Stack

```
┌────────────────────────────────────────────────────────┐
│                  Docker Compose                        │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │        Rust App Container (Port 8080)            │  │
│  │  ┌────────────────┐  ┌────────────────────────┐  │  │
│  │  │  Axum Backend  │  │  Yew SPA (WASM)        │  │  │
│  │  │  - API routes  │  │  - Served as static    │  │  │
│  │  │  - Auth        │  │    assets by Axum      │  │  │
│  │  │  - Proxy       │  │  - Drag-and-drop grid  │  │  │
│  │  │  - Orchestrator│  │  - Iframe management   │  │  │
│  │  └───────┬────────┘  └────────────────────────┘  │  │
│  └──────────┼───────────────────────────────────────┘  │
│             │                                          │
│      ┌──────┴──────┐                                   │
│      │ Internal    │                                   │
│      │ Docker Net  │                                   │
│      ├─────┬───────┤                                   │
│      ▼     ▼       ▼                                   │
│  ┌───────┐ ┌──────┐ ┌─────────────┐                    │
│  │Grafana│ │NocoDB│ │  PostgreSQL │                    │
│  │ :3000 │ │ :8090│ │  :5432      │                    │
│  └───┬───┘ └──┬───┘ └──────┬──────┘                    │
│      │        │            │                           │
│      └────────┴────────────┘                           │
│         All query Postgres                             │
└────────────────────────────────────────────────────────┘
```

### Key Principle

The Rust app is the **only publicly exposed service** (port 8080). Grafana and NocoDB are on the internal Docker network only. The Axum backend proxies and authenticates all requests to them. This gives you a single URL, single login, and full control over what's accessible.

---

## 2. Database Schema (Axum-owned tables in Postgres)

The portal's own metadata lives in a dedicated `portal` schema, separate from NocoDB's and Grafana's schemas.

```sql
-- Portal's own schema
CREATE SCHEMA portal;

-- Users & Auth
CREATE TABLE portal.users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username    VARCHAR(64) UNIQUE NOT NULL,
    email       VARCHAR(255),
    password_hash TEXT NOT NULL,            -- argon2id
    role        VARCHAR(16) NOT NULL DEFAULT 'viewer',  -- 'admin' | 'editor' | 'viewer'
    created_at  TIMESTAMPTZ DEFAULT now(),
    updated_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE portal.sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES portal.users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL,              -- sha256 of session token
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- Dashboard & Layout
CREATE TABLE portal.dashboards (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id    UUID REFERENCES portal.users(id),
    title       VARCHAR(255) NOT NULL,
    slug        VARCHAR(255) UNIQUE NOT NULL,   -- URL-friendly identifier
    icon        VARCHAR(64),                     -- icon name for sidebar
    sort_order  INT DEFAULT 0,
    is_shared   BOOLEAN DEFAULT false,           -- visible to all users?
    created_at  TIMESTAMPTZ DEFAULT now(),
    updated_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE portal.panels (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id    UUID REFERENCES portal.dashboards(id) ON DELETE CASCADE,
    title           VARCHAR(255),
    panel_type      VARCHAR(32) NOT NULL,        -- 'grafana_panel' | 'grafana_dashboard'
                                                  -- 'nocodb_form' | 'nocodb_grid'
                                                  -- 'nocodb_gallery' | 'markdown' | 'link'
    source_url      TEXT,                         -- iframe src (Grafana/NocoDB URL)
    config          JSONB DEFAULT '{}',           -- type-specific config
    -- Grid position (CSS Grid compatible)
    grid_x          INT NOT NULL DEFAULT 0,       -- column start
    grid_y          INT NOT NULL DEFAULT 0,       -- row start
    grid_w          INT NOT NULL DEFAULT 6,       -- column span (out of 12)
    grid_h          INT NOT NULL DEFAULT 4,       -- row span
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);

-- Dataset Templates (orchestration metadata)
CREATE TABLE portal.dataset_templates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    -- References to provisioned resources
    nocodb_table_id VARCHAR(255),                  -- NocoDB table ID
    nocodb_form_id  VARCHAR(255),                  -- NocoDB shared form view ID
    grafana_dashboard_uid VARCHAR(255),             -- Grafana dashboard UID
    -- Field definitions (source of truth for provisioning)
    fields          JSONB NOT NULL,                -- array of field definitions
    -- e.g. [{"name": "weight_kg", "type": "number", "unit": "kg"},
    --        {"name": "body_fat_pct", "type": "number", "unit": "%"},
    --        {"name": "notes", "type": "text"}]
    created_by      UUID REFERENCES portal.users(id),
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);
```

---

## 3. Axum Backend Architecture

### 3.1 Project Structure

```
backend/
├── Cargo.toml
├── src/
│   ├── main.rs                  -- Server bootstrap, router composition
│   ├── config.rs                -- Environment/config loading
│   ├── error.rs                 -- Unified error type (thiserror)
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs              -- SQLx PgPool setup
│   │   └── migrations/          -- SQLx migrations
│   │
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── middleware.rs         -- Axum middleware: extract session, enforce roles
│   │   ├── handlers.rs          -- Login, logout, register, me
│   │   ├── password.rs          -- Argon2id hashing
│   │   └── session.rs           -- Session creation, validation, cleanup
│   │
│   ├── api/
│   │   ├── mod.rs               -- Route composition
│   │   ├── dashboards.rs        -- CRUD for dashboards
│   │   ├── panels.rs            -- CRUD for panels (incl. grid position updates)
│   │   ├── templates.rs         -- Dataset template CRUD + provisioning
│   │   └── users.rs             -- User management (admin only)
│   │
│   ├── proxy/
│   │   ├── mod.rs
│   │   ├── grafana.rs           -- Reverse proxy to Grafana (path: /proxy/grafana/*)
│   │   └── nocodb.rs            -- Reverse proxy to NocoDB  (path: /proxy/nocodb/*)
│   │
│   ├── orchestrator/
│   │   ├── mod.rs
│   │   ├── nocodb_client.rs     -- NocoDB REST API client (create table, fields, views)
│   │   ├── grafana_client.rs    -- Grafana HTTP API client (create dashboard, panels)
│   │   └── provisioner.rs       -- End-to-end dataset provisioning pipeline
│   │
│   └── spa.rs                   -- Serve Yew WASM + static assets, SPA fallback
│
├── static/                      -- Built Yew app output (wasm, js, index.html)
└── tests/
    ├── api_tests.rs
    └── provisioner_tests.rs
```

### 3.2 Design Patterns

| Pattern | Where | Why |
|---------|-------|-----|
| **Repository pattern** | `db/` layer | Abstract SQL queries behind trait-based repos; makes testing with mocks straightforward |
| **Tower middleware** | `auth/middleware.rs` | Axum's native middleware stack for auth extraction, role enforcement, request logging |
| **Type-state builder** | `orchestrator/provisioner.rs` | The provisioning pipeline (create table → create form → create dashboard → register) uses the type-state pattern so each step's output feeds the next, and you can't skip steps at compile time |
| **Facade** | `orchestrator/mod.rs` | Single `Orchestrator` struct that hides `nocodb_client`, `grafana_client`, and `provisioner` behind a simple interface |
| **Strategy** | `proxy/` | Both proxy modules implement a common `ProxyTarget` trait, so the proxy handler is generic |
| **DTO ↔ Domain separation** | Throughout | API request/response types (serde) are separate from internal domain types; conversion via `From`/`Into` |

### 3.3 Key API Routes

```
POST   /api/auth/login              -- Returns session cookie
POST   /api/auth/logout
POST   /api/auth/register           -- Admin-only or first-user bootstrap
GET    /api/auth/me                 -- Current user info

GET    /api/dashboards              -- List dashboards (filtered by visibility + ownership)
POST   /api/dashboards              -- Create dashboard
GET    /api/dashboards/:slug        -- Get dashboard with all panels
PUT    /api/dashboards/:id          -- Update dashboard metadata
DELETE /api/dashboards/:id          -- Delete dashboard + cascade panels

GET    /api/dashboards/:id/panels   -- List panels
POST   /api/dashboards/:id/panels   -- Add panel
PUT    /api/panels/:id              -- Update panel (title, source, config)
PUT    /api/panels/:id/position     -- Update grid position (drag-and-drop)
PUT    /api/panels/batch-position   -- Batch update positions (after reflow)
DELETE /api/panels/:id

GET    /api/templates               -- List dataset templates
POST   /api/templates               -- Create template + provision resources
GET    /api/templates/:id           -- Get template details
DELETE /api/templates/:id           -- Delete template (+ cleanup resources?)

GET    /api/users                   -- Admin: list users
PUT    /api/users/:id/role          -- Admin: change role

-- Reverse proxies (transparent, session-authenticated)
ANY    /proxy/grafana/*             -- All Grafana requests
ANY    /proxy/nocodb/*              -- All NocoDB requests
```

### 3.4 Reverse Proxy Detail

The proxy is critical — it's what lets you have a single origin and single auth:

```rust
// Simplified proxy handler concept
async fn proxy_grafana(
    State(state): State<AppState>,
    session: AuthenticatedSession,   // extracted by middleware
    req: Request<Body>,
) -> Result<Response<Body>, AppError> {
    // Rewrite path: /proxy/grafana/d/abc123 → /d/abc123
    let target_path = req.uri().path().strip_prefix("/proxy/grafana").unwrap();
    
    // Forward to internal Grafana with service account auth
    let grafana_url = format!("http://grafana:3000{}", target_path);
    
    // Inject Grafana auth header (service account token or basic auth)
    // This means Grafana never needs to be exposed or have its own login
    let response = state.http_client
        .request(rewrite_request(req, &grafana_url, &state.grafana_token))
        .await?;
    
    Ok(response)
}
```

Grafana is configured with:
- `allow_embedding = true`
- Anonymous auth enabled (since the Axum proxy handles real auth)
- Or a service account token injected by the proxy

---

## 4. Yew Frontend Architecture

### 4.1 Project Structure

```
frontend/
├── Cargo.toml
├── index.html                    -- Trunk entry point
├── Trunk.toml
├── tailwind.config.js            -- Tailwind via Trunk pipeline
├── static/                       -- CSS, JS deps (gridstack)
│   └── main.css
│   └── ...
├── src/
│   ├── main.rs                   -- App entry, mount root component
│   ├── app.rs                    -- Root component: router + layout shell
│   │
│   ├── router.rs                 -- Client-side routing definitions
│   │   -- /login
│   │   -- /dashboards
│   │   -- /dashboards/:slug
│   │   -- /dashboards/:slug/edit
│   │   -- /templates
│   │   -- /templates/new
│   │   -- /settings
│   │
│   ├── api/
│   │   ├── mod.rs                -- Re-exports
│   │   ├── client.rs             -- HTTP client wrapper (gloo-net or reqwasm)
│   │   ├── auth.rs               -- Login/logout/me calls
│   │   ├── dashboards.rs         -- Dashboard + panel CRUD calls
│   │   └── templates.rs          -- Template CRUD calls
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs               -- User, Session, Role
│   │   ├── dashboard.rs          -- Dashboard, Panel, PanelType, GridPosition
│   │   └── template.rs           -- DatasetTemplate, FieldDefinition
│   │
│   ├── components/
│   │   ├── mod.rs
│   │   ├── layout/
│   │   │   ├── shell.rs          -- App shell: sidebar + topbar + content area
│   │   │   ├── sidebar.rs        -- Navigation sidebar with dashboard list
│   │   │   └── topbar.rs         -- User menu, settings, breadcrumbs
│   │   │
│   │   ├── grid/
│   │   │   ├── dashboard_grid.rs -- The drag-and-drop grid container
│   │   │   ├── grid_item.rs      -- Individual panel wrapper (resize handles, drag)
│   │   │   └── grid_engine.rs    -- Grid layout computation (collision, reflow)
│   │   │
│   │   ├── panels/
│   │   │   ├── panel_frame.rs    -- Generic panel chrome (title bar, menu, iframe)
│   │   │   ├── grafana_panel.rs  -- Grafana-specific iframe config
│   │   │   ├── nocodb_panel.rs   -- NocoDB-specific iframe config
│   │   │   ├── markdown_panel.rs -- Static markdown/HTML content panel
│   │   │   └── panel_picker.rs   -- Modal: choose panel type + configure source
│   │   │
│   │   ├── templates/
│   │   │   ├── template_list.rs  -- List all dataset templates
│   │   │   ├── template_wizard.rs-- Multi-step create: name → fields → preview → provision
│   │   │   └── field_editor.rs   -- Add/edit/reorder fields in a template
│   │   │
│   │   ├── auth/
│   │   │   └── login_page.rs
│   │   │
│   │   └── common/
│   │       ├── modal.rs
│   │       ├── dropdown.rs
│   │       ├── icon.rs
│   │       ├── toast.rs          -- Notification toasts
│   │       └── loading.rs
│   │
│   ├── hooks/
│   │   ├── use_auth.rs           -- Auth context hook (current user, login state)
│   │   ├── use_api.rs            -- Generic fetch hook with loading/error states
│   │   └── use_drag.rs           -- Drag-and-drop state management
│   │
│   └── context/
│       ├── auth_context.rs       -- App-wide auth state (ContextProvider)
│       └── theme_context.rs      -- Optional: light/dark theme toggle
```

### 4.2 Design Patterns

| Pattern | Where | Why |
|---------|-------|-----|
| **Context providers** | `context/` | App-wide state (auth, theme) without prop drilling through every component |
| **Custom hooks** | `hooks/` | Reusable stateful logic (API calls, drag state) separated from component rendering |
| **Component composition** | `panels/` | `panel_frame.rs` is the generic wrapper; specific panel types compose inside it |
| **Command pattern** | `grid/grid_engine.rs` | Grid mutations (move, resize, add, remove) are modeled as commands with undo support |
| **Observer** | Grid ↔ Panels | Grid emits position change events; panels react to re-render at new coordinates |

### 4.3 Drag-and-Drop Grid

**JS interop with an existing grid library**: Use `wasm-bindgen` to wrap a battle-tested JS grid library like `gridstack.js` or `react-grid-layout` (adapted). The Yew component manages the Rust state while delegating DOM manipulation to JS. This is the pragmatic choice — grid DnD is notoriously fiddly. It's framework-agnostic (vanilla JS), handles collision, reflow, resize, and serialization. Your Yew component would:
- Render placeholder `<div>` elements with `gs-*` attributes
- Initialize GridStack via `wasm-bindgen` on mount
- Listen for GridStack change events → update Rust state → persist via API

### 4.4 Iframe Management

Each embedded panel uses a sandboxed iframe:

```html
<iframe
    src="/proxy/grafana/d-solo/abc123/my-dashboard?panelId=2&theme=light"
    sandbox="allow-scripts allow-same-origin allow-forms"
    loading="lazy"
    style="width: 100%; height: 100%; border: none;"
></iframe>
```

Key points:
- The `/proxy/grafana/` and `/proxy/nocodb/` prefixes ensure all iframe requests go through the Axum proxy (same origin = no CORS issues, session cookie flows naturally)
- Grafana's `d-solo` endpoint renders a single panel without Grafana's own chrome
- NocoDB shared form/grid views are self-contained and iframe-friendly
- `loading="lazy"` prevents off-screen panels from loading until scrolled into view

---

## 5. Dataset Provisioning Flow

When a user creates a new dataset template through the wizard:

```
┌──────────────────────────────────────────────────────────┐
│                  Template Wizard (Yew)                   │
│  Step 1: Name & Description                              │
│  Step 2: Define Fields (name, type, unit)                │
│  Step 3: Preview & Confirm                               │
└──────────────────────┬───────────────────────────────────┘
                       │ POST /api/templates
                       ▼
┌──────────────────────────────────────────────────────────┐
│              Provisioner (Axum backend)                  │
│                                                          │
│  1. Save template to portal.dataset_templates            │
│                                                          │
│  2. NocoDB API calls:                                    │
│     POST /api/v2/meta/tables     → create table          │
│     POST /api/v2/meta/columns    → create fields         │
│     POST /api/v2/meta/forms      → create form view      │
│     → Store table_id, form_id back in template record    │
│                                                          │
│  3. Grafana API calls:                                   │
│     POST /api/dashboards/db      → create dashboard      │
│     (with auto-generated panels for each numeric field:  │
│      time-series line chart by default)                  │
│     → Store dashboard_uid back in template record        │
│                                                          │
│  4. Optionally: auto-create a portal dashboard page      │
│     with the Grafana dashboard + NocoDB form embedded    │
│                                                          │
│  5. Return success + links                               │
└──────────────────────────────────────────────────────────┘
```

---

## 6. Authentication Flow

```
Browser                     Axum                      Postgres
  │                          │                           │
  │  POST /api/auth/login    │                           │
  │  {username, password}    │                           │
  │ ─────────────────────►   │                           │
  │                          │  SELECT password_hash     │
  │                          │  FROM portal.users        │
  │                          │ ─────────────────────►    │
  │                          │  ◄─────────────────────   │
  │                          │                           │
  │                          │  Verify argon2id          │
  │                          │  Generate session token   │
  │                          │                           │
  │                          │  INSERT portal.sessions   │
  │                          │ ─────────────────────►    │
  │                          │                           │
  │  Set-Cookie: session=... │                           │
  │  (HttpOnly, SameSite,    │                           │
  │   Secure if HTTPS)       │                           │
  │ ◄─────────────────────   │                           │
  │                          │                           │
  │  GET /proxy/grafana/...  │                           │
  │  Cookie: session=...     │                           │
  │ ─────────────────────►   │                           │
  │                          │  Validate session         │
  │                          │  Proxy to Grafana:3000    │
  │                          │  (inject Grafana auth)    │
  │ ◄─────────────────────   │                           │
```

First user to register gets the `admin` role (bootstrap). Subsequent users are created by the admin.

---

## 7. Configuration

All configuration via environment variables (12-factor app style), with a `.env` file for docker-compose:

```env
# Database
DATABASE_URL=postgres://portal:secret@postgres:5432/portal_db

# Grafana
GRAFANA_INTERNAL_URL=http://grafana:3000
GRAFANA_SERVICE_ACCOUNT_TOKEN=glsa_xxxxxxxxxxxx

# NocoDB
NOCODB_INTERNAL_URL=http://nocodb:8090
NOCODB_API_TOKEN=xxxxxxxxxxxxx

# Auth
SESSION_SECRET=<random-64-bytes-hex>
SESSION_TTL_HOURS=168          # 1 week

# App
RUST_LOG=info,backend=debug
BIND_ADDRESS=0.0.0.0:8080
```

---

## 8. Docker Compose Outline

```yaml
services:
  postgres:
    image: postgres:16-alpine
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: portal_db
      POSTGRES_USER: portal
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U portal"]

  grafana:
    image: grafana/grafana-oss:latest
    depends_on:
      postgres: { condition: service_healthy }
    environment:
      GF_DATABASE_TYPE: postgres
      GF_DATABASE_HOST: postgres:5432
      GF_DATABASE_NAME: grafana
      GF_DATABASE_USER: portal
      GF_DATABASE_PASSWORD: ${POSTGRES_PASSWORD}
      GF_SECURITY_ALLOW_EMBEDDING: "true"
      GF_AUTH_ANONYMOUS_ENABLED: "true"
      GF_AUTH_ANONYMOUS_ORG_ROLE: Viewer
      GF_SERVER_SERVE_FROM_SUB_PATH: "true"
      GF_SERVER_ROOT_URL: "%(protocol)s://%(domain)s:%(http_port)s/proxy/grafana/"
    # NOT exposed to host — only accessible via internal network
    expose:
      - "3000"

  nocodb:
    image: nocodb/nocodb:latest
    depends_on:
      postgres: { condition: service_healthy }
    environment:
      NC_DB: "pg://postgres:5432?u=portal&p=${POSTGRES_PASSWORD}&d=nocodb"
    expose:
      - "8090"

  portal:
    build: .
    depends_on:
      postgres: { condition: service_healthy }
      grafana: { condition: service_started }
      nocodb: { condition: service_started }
    ports:
      - "8080:8080"     # The ONLY exposed port
    environment:
      DATABASE_URL: postgres://portal:${POSTGRES_PASSWORD}@postgres:5432/portal_db
      GRAFANA_INTERNAL_URL: http://grafana:3000
      NOCODB_INTERNAL_URL: http://nocodb:8080
      # ... other env vars
    volumes:
      - ./static:/app/static   # Built Yew assets (or baked into image)

volumes:
  pgdata:
```

---

## 9. Build & Development Workflow

### Development

```bash
# Terminal 1: Backend (with hot reload)
cd backend && cargo watch -x run

# Terminal 2: Frontend (Trunk serves with hot reload + proxy to backend)
cd frontend && trunk serve --proxy-backend=http://localhost:8080/api

# Terminal 3: Supporting services
docker compose up postgres grafana nocodb
```

### Production Build

```dockerfile
# Multi-stage Dockerfile
FROM rust:1.83 AS backend-builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

FROM rust:1.83 AS frontend-builder
RUN cargo install trunk && rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY frontend/ .
RUN trunk build --release

FROM debian:bookworm-slim
COPY --from=backend-builder /app/target/release/portal-backend /usr/local/bin/
COPY --from=frontend-builder /app/dist/ /app/static/
CMD ["portal-backend"]
```

---

## 10. Security Considerations

- **Single entry point**: Only the Axum container exposes a port. Grafana and NocoDB are unreachable from outside Docker's internal network.
- **Session cookies**: `HttpOnly`, `SameSite=Strict`, `Secure` (when behind HTTPS).
- **Password hashing**: Argon2id with recommended parameters.
- **Role-based access**: Middleware enforces roles on API routes. Viewers can see dashboards but not create templates or manage users.
- **Proxy isolation**: The proxy injects service account credentials for Grafana/NocoDB — end users never see or handle these tokens.
- **CSRF**: SameSite cookies provide baseline protection; add CSRF tokens for state-changing operations if the portal will be exposed beyond the home network.
- **Input sanitization**: All user inputs for dataset field names are validated (alphanumeric + underscore) before being sent to NocoDB/Grafana APIs.
