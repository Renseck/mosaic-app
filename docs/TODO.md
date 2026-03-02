# After original implementation is complete.

After the original plan has been completed, there are a few points that have been signaled during development which may need attention after the complete plan has been compelted, which had not been foreseen beforehand. These are:

## High Priority
- **Session expiry handling**: Detect 401 globally in frontend API client, redirect to login with toast
- **Confirmation dialogs**: Add confirm step to delete dashboard, delete template, deprovision actions
- **Backend integration tests**: Auth flow, dashboard CRUD, provisioner pipeline (mock external services)
- **E2E tests (Playwright)**: Full-stack browser tests against the Docker Compose stack
  - Setup: `tests/e2e/` at repo root with `playwright.config.ts` pointing at `localhost:8080`
  - Auth fixture: login once via API, save `storageState` (session cookie), reuse across specs
  - Test cases:
    1. Auth flow: register first user → logout → login → verify session
    2. Dashboard lifecycle: create → appears in sidebar + list → rename → navigate → delete → gone
    3. Template provisioning: wizard → NocoDB table + Grafana dashboard created → portal dashboard with panels → delete → cleanup
    4. Panel management: add panel → drag/resize → reload → verify position persisted
    5. Admin actions: create user → change role → reset password → login as that user
  - Run: `make test-e2e` → `docker compose up -d --build`, wait for health, `npx playwright test`, teardown
  - Keep it lean: 5-8 coarse-grained specs covering happy paths, not individual API calls

```bash
mosaic-app/
├── tests/
│   └── e2e/
│       ├── playwright.config.ts
│       ├── package.json          # playwright + @playwright/test
│       ├── fixtures/
│       │   └── auth.ts           # Login helper / storage state
│       └── specs/
│           ├── auth.spec.ts
│           ├── dashboard.spec.ts
│           ├── template.spec.ts
│           └── admin.spec.ts
```

## Medium Priority
- **Frontend error boundary**: Catch WASM panics at app root, show recovery UI
- **Single migration owner**: Remove schema SQL from Postgres Dockerfile, let SQLx own all migrations
- **SQLx offline query checking**: Add `cargo sqlx prepare` to CI/Makefile
- **Dashboard reordering**: Drag-to-reorder in sidebar/list, persist via sort_order
- **Orphan panel detection**: Warn or auto-clean panels whose source_url target no longer exists
- **Grafana panel browser**: Visual picker using Grafana API instead of manual URL entry

## Low Priority
- **ETag caching**: Add conditional GET to list endpoints when query volume warrants it
- **Dashboard duplication**: Clone dashboard + panels with new IDs
- **Toast improvements**: Persist critical toasts (password reset, provisioning results), add copy-to-clipboard
- **Pagination**: Add cursor-based pagination to list endpoints when dataset size warrants it