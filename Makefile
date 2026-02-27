.PHONY: help \
        backend-build backend-build-release backend-run backend-watch \
        backend-test backend-test-verbose backend-check backend-fmt backend-lint \
        backend-lint-fix backend-doc backend-doc-private backend-bench \
        backend-audit backend-bloat \
        frontend-build frontend-build-release frontend-serve frontend-clean \
        docker-up docker-down docker-services docker-build docker-logs docker-logs-portal \
        migrate \
        build run test fmt lint check clean all

# ============================================= Paths ============================================ #
BACKEND  := src/backend
FRONTEND := src/frontend
COMPOSE  := src/docker-compose.yml

# =========================================== Detect OS ========================================== #
ifeq ($(OS),Windows_NT)
    SHELL := cmd.exe
    .SHELLFLAGS := -NoProfile -Command
    CARGO := cargo
else
    CARGO := cargo
endif

help: ## Display this help message
ifeq ($(OS),Windows_NT)
	@Get-Content Makefile | Select-String -Pattern '^[a-zA-Z_-]+:.*?## .*$$' | ForEach-Object { $$match = $$_.Line -match '^([a-zA-Z_-]+):.*?## (.*)$$'; $$name = $$matches[1]; $$desc = $$matches[2]; Write-Host ("{0,-20}" -f $$name) -NoNewline -ForegroundColor Cyan; Write-Host $$desc }
else
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'
endif

# ============================================ Backend =========================================== #
backend-build: ## Build backend (debug)
	cd $(BACKEND) && $(CARGO) build

backend-build-release: ## Build backend (release)
	cd $(BACKEND) && $(CARGO) build --release

backend-run: ## Run backend server
	cd $(BACKEND) && $(CARGO) run

backend-watch: ## Run backend with hot reload (requires cargo-watch)
	cd $(BACKEND) && $(CARGO) watch -x run

backend-test: ## Run backend tests
	cd $(BACKEND) && $(CARGO) test

backend-test-verbose: ## Run backend tests with output
	cd $(BACKEND) && $(CARGO) test -- --nocapture --test-threads=1

backend-check: ## Check backend without building
	cd $(BACKEND) && $(CARGO) check --all-targets --all-features

backend-fmt: ## Format backend code
	cd $(BACKEND) && $(CARGO) fmt --all

backend-lint: ## Run clippy on backend
	cd $(BACKEND) && $(CARGO) clippy --all-targets --all-features -- -D warnings

backend-lint-fix: ## Auto-fix clippy warnings in backend
	cd $(BACKEND) && $(CARGO) clippy --fix --allow-dirty --allow-staged

backend-doc: ## Open backend docs in browser
	cd $(BACKEND) && $(CARGO) doc --no-deps --open

backend-doc-private: ## Open backend docs (including private items)
	cd $(BACKEND) && $(CARGO) doc --no-deps --document-private-items --open

backend-bench: ## Run backend benchmarks
	cd $(BACKEND) && $(CARGO) bench

backend-audit: ## Audit backend dependencies for vulnerabilities
	cd $(BACKEND) && $(CARGO) audit

backend-bloat: ## Analyse backend binary size
	cd $(BACKEND) && $(CARGO) bloat --release

# =========================================== Frontend =========================================== #
frontend-build: ## Build frontend WASM (debug)
	cd $(FRONTEND) && trunk build

frontend-build-release: ## Build frontend WASM (release, minified)
	cd $(FRONTEND) && trunk build --release

frontend-serve: ## Start frontend dev server (proxies /api and /proxy to localhost:8080)
	cd $(FRONTEND) && trunk serve --proxy-backend=http://localhost:8080/api

frontend-clean: ## Remove frontend build output
	rm -rf $(FRONTEND)/dist

# ============================================ Docker ============================================ #
docker-up: ## Start the full stack (all services including portal)
	docker compose -f $(COMPOSE) up -d --build

docker-down: ## Stop and remove all containers
	docker compose -f $(COMPOSE) down

docker-services: ## Start supporting services only (postgres, grafana, nocodb)
	docker compose -f $(COMPOSE) up postgres grafana nocodb -d

docker-build: ## Build the portal Docker image
	docker compose -f $(COMPOSE) build portal

docker-rebuild: ## Rebuild all images from scratch and start
	docker compose -f ${COMPOSE} up -d --build --force

docker-logs: ## Follow logs from all services
	docker compose -f $(COMPOSE) logs -f

docker-logs-portal: ## Follow portal service logs only
	docker compose -f $(COMPOSE) logs -f portal

# =========================================== Database =========================================== #
migrate: ## Run pending database migrations
	sqlx migrate run --source $(BACKEND)/src/db/migrations

# ======================== Shortcuts (keep muscle memory from old targets) ======================= #
build:   backend-build         ## Alias → backend-build
run:     backend-run           ## Alias → backend-run
test:    backend-test          ## Alias → backend-test
check:   backend-check         ## Alias → backend-check
fmt:     backend-fmt           ## Alias → backend-fmt
lint:    backend-lint          ## Alias → backend-lint

clean: frontend-clean ## Clean all build artifacts
ifeq ($(OS),Windows_NT)
	@powershell -Command "Set-Location $(BACKEND); cargo clean"
else
	cd $(BACKEND) && $(CARGO) clean
endif

# =========================================== Compound =========================================== #
all: backend-fmt backend-lint backend-test backend-build ## Format, lint, test, and build backend

.DEFAULT_GOAL := help