# Personal Data Portal

Self-hosted personal data portal (Rust) that unifies data entry (NocoDB), visualization (Grafana), and navigation into a single authenticated interface. Axum backend + Yew frontend served as WASM SPA.

## Tech Stack

- **Backend**: Rust (latest stable), Axum 0.8+, SQLx (Postgres), Tokio
- **Frontend**: Yew 0.21+, Trunk, wasm-bindgen, gloo, gridstack.js (via JS interop)
- **Database**: PostgreSQL 16 — shared instance, portal owns the `portal` schema
- **External services** (Docker internal network only): Grafana OSS, NocoDB
- **Containerization**: Docker Compose — only the Rust app exposes a port (8080)

## Project Structure

```
mosaic-app/
├── .github/
│   ├── CLAUDE.md
│   └── copilot-instructions.md
├── .github/
│   └── architecture.md
├── src/
│   ├── docker-compose.yml
│   ├── .env.example
│   ├── backend/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs              # Bootstrap: router composition, state init
│   │   │   ├── config.rs            # Env-based config (envy or dotenvy)
│   │   │   ├── error.rs             # Unified AppError (thiserror + IntoResponse)
│   │   │   ├── db/                  # Pool setup + SQLx migrations
│   │   │   ├── auth/                # Middleware, handlers, password (argon2), sessions
│   │   │   ├── api/                 # REST handlers: dashboards, panels, templates, users
│   │   │   ├── proxy/               # Reverse proxy to Grafana & NocoDB
│   │   │   ├── orchestrator/        # NocoDB + Grafana API clients, provisioner pipeline
│   │   │   └── spa.rs               # Serve static Yew assets + SPA fallback
│   │   └── tests/
│   ├── frontend/
│   │   ├── Cargo.toml
│   │   ├── Trunk.toml
│   │   ├── index.html
│   │   ├── tailwind.config.js
│   │   ├── src/
│   │   │   ├── main.rs              # Mount root component
│   │   │   ├── app.rs               # Root: router + layout shell
│   │   │   ├── router.rs            # Client-side routes
│   │   │   ├── api/                 # HTTP client wrappers (gloo-net)
│   │   │   ├── models/              # Shared types (user, dashboard, panel, template)
│   │   │   ├── components/
│   │   │   │   ├── layout/          # Shell, sidebar, topbar
│   │   │   │   ├── grid/            # Drag-and-drop grid (gridstack.js interop)
│   │   │   │   ├── panels/          # Panel frame, grafana/nocodb/markdown panels, panel picker
│   │   │   │   ├── templates/       # Template wizard, field editor
│   │   │   │   ├── auth/            # Login page
│   │   │   │   └── common/          # Modal, dropdown, icon, toast, loading
│   │   │   ├── hooks/               # use_auth, use_api, use_drag
│   │   │   └── context/             # Auth + theme context providers
│   │   └── static/                  # CSS, JS deps (gridstack)
└── Dockerfile                       # Multi-stage: build backend + frontend, slim runtime
```

## Commands

Feel free to make specific `Make` commands for these, or rewrite the existing ones.

- `cd backend && cargo run` — start backend (expects Postgres, Grafana, NocoDB running)
- `cd backend && cargo test` — run backend tests
- `cd backend && cargo watch -x run` — backend with hot reload
- `cd frontend && trunk serve --proxy-backend=http://localhost:8080/api` — frontend dev server
- `cd frontend && trunk build --release` — production WASM build (output in `frontend/dist/`)
- `docker compose up -d` — full stack (Postgres, Grafana, NocoDB, portal)
- `docker compose up postgres grafana nocodb` — supporting services only (for local dev)
- `sqlx migrate run --source backend/src/db/migrations` — run DB migrations

## Architecture Rules

### Single Entry Point

The Axum backend is the ONLY service exposed outside Docker. Grafana and NocoDB sit on the internal Docker network. All access goes through the Axum reverse proxy at `/proxy/grafana/*` and `/proxy/nocodb/*`. This means:
- No CORS configuration needed (same origin)
- Session cookies flow naturally to iframed content
- Grafana uses anonymous auth internally — real auth is handled by Axum middleware

### Database Ownership

Portal owns the `portal` schema in the shared Postgres instance. Never write to NocoDB's or Grafana's schemas directly — always use their REST APIs via the orchestrator clients.

### Auth Model

Multi-user with roles: `admin`, `editor`, `viewer`. First registered user auto-promotes to `admin`. Session-based auth using HttpOnly cookies. Passwords hashed with argon2id.

## Mandatory Patterns

### Backend: Unified Error Type

Every handler returns `Result<T, AppError>`. Use `thiserror` for variants, implement `IntoResponse` to map to HTTP status codes.

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("validation: {0}")]
    Validation(String),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Return JSON body: { "error": "..." }
        let body = serde_json::json!({ "error": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}
```

### Backend: Auth Middleware Extractor

Use a custom Axum extractor for auth. Handlers that need auth just add the extractor to their signature.

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub role: Role,
}

// Extract from session cookie → validate against portal.sessions → return user
impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract session token from cookie
        // 2. Hash token, look up in portal.sessions
        // 3. Join portal.users, check expiry
        // 4. Return AuthenticatedUser or AppError::Unauthorized
        todo!()
    }
}

pub struct RequireAdmin(pub AuthenticatedUser);
// Same pattern but rejects if role != admin
```

### Backend: Repository Pattern

Database access goes through repository traits. This enables testing with mocks.

```rust
#[async_trait::async_trait]
pub trait DashboardRepo: Send + Sync {
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Dashboard>, AppError>;
    async fn get_by_slug(&self, slug: &str) -> Result<Dashboard, AppError>;
    async fn create(&self, input: CreateDashboard) -> Result<Dashboard, AppError>;
    async fn update(&self, id: Uuid, input: UpdateDashboard) -> Result<Dashboard, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct PgDashboardRepo {
    pool: sqlx::PgPool,
}

// Inject via Axum state:
pub struct AppState {
    pub dashboards: Arc<dyn DashboardRepo>,
    pub panels: Arc<dyn PanelRepo>,
    pub templates: Arc<dyn TemplateRepo>,
    pub users: Arc<dyn UserRepo>,
    pub orchestrator: Arc<Orchestrator>,
    pub config: AppConfig,
}
```

### Backend: Reverse Proxy

The proxy rewrites paths and injects service credentials. Both Grafana and NocoDB proxy handlers implement a shared approach:

```rust
// Proxy trait for consistent handling
pub trait ProxyTarget {
    fn internal_base_url(&self) -> &str;
    fn strip_prefix(&self) -> &str;
    fn inject_auth(&self, req: &mut reqwest::Request);
}
```

### Backend: Orchestrator / Provisioner

Dataset provisioning uses a pipeline that creates resources across NocoDB and Grafana. Each step's output feeds the next. Keep API clients (`NococbClient`, `GrafanaClient`) as thin wrappers over `reqwest` — the orchestrator coordinates them.

```rust
pub struct Orchestrator {
    nocodb: NocodbClient,
    grafana: GrafanaClient,
    templates: Arc<dyn TemplateRepo>,
}

impl Orchestrator {
    /// Full provisioning pipeline for a new dataset template.
    pub async fn provision_dataset(&self, input: CreateTemplate) -> Result<DatasetTemplate, AppError> {
        // 1. Create NocoDB table with fields
        // 2. Create NocoDB form view
        // 3. Create Grafana dashboard with default panels per numeric field
        // 4. Persist template record with all external IDs
        // 5. Optionally create a portal dashboard page with embedded panels
        todo!()
    }
}
```

### Frontend: Grid via gridstack.js Interop

The drag-and-drop dashboard grid wraps gridstack.js through `wasm-bindgen`. The Yew component renders placeholder divs, initializes GridStack on mount, and listens for change events to sync back to Rust state.

Do NOT attempt to build collision detection or grid reflow in pure Rust/Yew — use gridstack.js.

```rust
// In frontend/src/components/grid/grid_engine.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type GridStack;

    #[wasm_bindgen(static_method_of = GridStack, js_name = init)]
    pub fn init(opts: &JsValue) -> GridStack;

    #[wasm_bindgen(method)]
    pub fn on(this: &GridStack, event: &str, callback: &Closure<dyn FnMut(JsValue, JsValue)>);

    #[wasm_bindgen(method, js_name = makeWidget)]
    pub fn make_widget(this: &GridStack, el: &web_sys::Element);

    #[wasm_bindgen(method, js_name = removeWidget)]
    pub fn remove_widget(this: &GridStack, el: &web_sys::Element);
}
```

### Frontend: Context Providers for Global State

Use Yew's `ContextProvider` for auth and theme state. Components consume via `use_context`.

```rust
// Auth context — wraps current user + login/logout actions
#[derive(Clone, PartialEq)]
pub struct AuthState {
    pub user: Option<User>,
    pub loading: bool,
}

pub enum AuthAction {
    SetUser(User),
    Logout,
    SetLoading(bool),
}

// Provide at app root, consume anywhere with:
// let auth = use_context::<AuthState>().expect("AuthContext not found");
```

### Frontend: Panel Embedding

All embedded content uses the proxy paths (same origin). Grafana panels use the `d-solo` endpoint for chrome-free rendering:

- Grafana panel: `/proxy/grafana/d-solo/{dashboard_uid}/{dashboard_slug}?orgId=1&panelId={id}&theme=light`
- Grafana dashboard: `/proxy/grafana/d/{uid}/{slug}?kiosk`
- NocoDB form: `/proxy/nocodb/#/nc/form/{form_id}`
- NocoDB grid: `/proxy/nocodb/#/nc/view/{view_id}`

## API Routes

```
POST   /api/auth/login
POST   /api/auth/logout
POST   /api/auth/register         # Admin-only (except first user bootstrap)
GET    /api/auth/me

GET    /api/dashboards
POST   /api/dashboards
GET    /api/dashboards/:slug
PUT    /api/dashboards/:id
DELETE /api/dashboards/:id

GET    /api/dashboards/:id/panels
POST   /api/dashboards/:id/panels
PUT    /api/panels/:id
PUT    /api/panels/:id/position
PUT    /api/panels/batch-position  # After drag-and-drop reflow
DELETE /api/panels/:id

GET    /api/templates
POST   /api/templates              # Triggers full provisioning pipeline
GET    /api/templates/:id
DELETE /api/templates/:id

GET    /api/users                  # Admin only
PUT    /api/users/:id/role         # Admin only

ANY    /proxy/grafana/*            # Reverse proxy (session-authenticated)
ANY    /proxy/nocodb/*             # Reverse proxy (session-authenticated)
```

## Database Schema

Portal tables live in the `portal` schema. Key tables: `users`, `sessions`, `dashboards`, `panels`, `dataset_templates`. Full schema is in `docs/architecture.md`. When creating migrations:
- Always use `CREATE SCHEMA IF NOT EXISTS portal;` at the top
- All tables are `portal.<table_name>`
- Use UUIDs as primary keys (`gen_random_uuid()`)
- Use `TIMESTAMPTZ` for all timestamps
- `panels.config` is `JSONB` for panel-type-specific configuration
- `dataset_templates.fields` is `JSONB` array of field definitions

## Do Not

- **Do not** expose Grafana or NocoDB ports in docker-compose — they are internal only
- **Do not** query NocoDB or Grafana databases directly — always use their REST APIs via the orchestrator
- **Do not** store plaintext passwords — always argon2id
- **Do not** build grid drag-and-drop in pure Rust — use gridstack.js interop
- **Do not** put business logic in API handlers — handlers are thin, logic lives in repos and orchestrator
- **Do not** use `unwrap()` or `expect()` in handler code — propagate errors via `AppError` and `?`
- **Do not** add `#[allow(unused)]` to suppress warnings long-term — fix or remove dead code

## Working Preferences

- **Code proposals only**: Always propose code in chat with file paths and line numbers. Do not directly edit files. The user reviews and applies changes manually.
- **Local dev setup**: When running outside Docker, `.env` uses `localhost` URLs (not Docker-internal hostnames like `postgres`, `grafana`, `nocodb`). Docker-internal hostnames are only used in docker-compose environment variables.
- **Migration naming**: Use timestamp format: `YYYYMMDDHHMMSS_description.sql` (e.g., `20260217000001_initial_schema.sql`).
- **Latest service versions**: Grafana, NocoDB, Postgres `18-alpine`.
- **Init-db pattern**: `src/init-db.sql` is mounted into Postgres as `/docker-entrypoint-initdb.d/01-init.sql` to create the `grafana` and `nocodb` databases at first startup.


## When Compacting

Always preserve: the full project structure listing, all code snippets in Mandatory Patterns, the API routes table, the Do Not list, and the names of any files modified in the current session.
