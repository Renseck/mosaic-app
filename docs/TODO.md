# After original implementation is complete.

After the original plan has been completed, there are a few points that have been signaled during development which may need attention after the complete plan has been compelted, which had not been foreseen beforehand. These are:

## High Priority
- **Session expiry handling**: Detect 401 globally in frontend API client, redirect to login with toast
- **Confirmation dialogs**: Add confirm step to delete dashboard, delete template, deprovision actions
- **Backend integration tests**: Auth flow, dashboard CRUD, provisioner pipeline (mock external services)

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