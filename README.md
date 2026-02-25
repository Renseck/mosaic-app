# Mosaic

A self-hosted personal data portal that unifies data entry, visualization, and navigation into a single authenticated interface.

Mosaic combines [NocoDB](https://nocodb.com/) for structured data entry, [Grafana](https://grafana.com/) for visualization, and a custom drag-and-drop dashboard grid into one cohesive application. An Axum backend serves as the single entry point — Grafana and NocoDB run on an internal Docker network, accessed exclusively through a reverse proxy. No CORS headaches, no exposed ports, just one URL.

## How It Works

- **Define a dataset template** — name your dataset and define its fields (numeric, text, date, etc.)
- **Mosaic provisions everything** — a NocoDB table and form for data entry, a Grafana dashboard with time-series panels for each numeric field, and a portal dashboard page that embeds it all
- **Enter data through NocoDB forms**, see it visualized in Grafana charts, and arrange everything on customizable dashboard grids with drag-and-drop

All of this happens behind a single authentication layer. Register users, assign roles (admin / editor / viewer), and share dashboards — all from the same interface.

## Tech Stack

| Layer       | Technology                          |
|-------------|-------------------------------------|
| Backend     | Rust, Axum, SQLx, Tokio             |
| Frontend    | Yew (Rust/WASM), Tailwind, gridstack.js |
| Database    | PostgreSQL 18                       |
| Viz         | Grafana OSS                         |
| Data entry  | NocoDB                              |
| Runtime     | Docker Compose                      |

## Getting Started

Launching Mosaic is a single command (or two, if you want to be pedantic):

```sh
cp src/.env.example src/.env   # review and adjust if needed
make docker-up
```

That's it. The stack bootstraps itself completely — Postgres, Grafana, NocoDB, and the portal application all start up, a bootstrapper container provisions service accounts and API tokens, and a default admin user is created automatically. Once healthy, the portal is available at [http://localhost:8080](http://localhost:8080).

Default credentials (configurable in `.env`):

| Service | Username | Password      |
|---------|----------|---------------|
| Portal  | admin    | Owner1234!    |

To tear everything down:

```sh
make docker-down
```

## Development

For local development, start the supporting services and run the backend and frontend separately:

```sh
make docker-services              # start Postgres, Grafana, NocoDB
make backend-watch                # backend with hot reload (requires cargo-watch)
make frontend-serve               # Yew dev server with proxy to backend
```

Run `make help` for the full list of available targets.

### Prerequisites (local dev)

- Rust (latest stable) with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) for building the Yew frontend
- Docker and Docker Compose

## Architecture

The Axum backend is the only service exposed outside Docker. Grafana and NocoDB are internal — all access is routed through the reverse proxy at `/proxy/grafana/*` and `/proxy/nocodb/*`. This means session cookies flow naturally to iframed content without any cross-origin configuration.

```
Browser
  |
  v
[Axum :8080] -- /api/*            -- REST handlers
  |           -- /proxy/grafana/*  -- Grafana (internal :3000)
  |           -- /proxy/nocodb/*   -- NocoDB  (internal :8080)
  |           -- /*                -- Yew SPA (WASM)
  |
  v
[PostgreSQL :5432]
```

For more detail, see [docs/PLAN.md](docs/PLAN.md).

## License

TBD