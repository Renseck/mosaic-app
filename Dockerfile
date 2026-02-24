# ── Stage 1: Build Yew frontend → WASM ──────────────────────────────────────
FROM rust:1.93-slim AS frontend-builder

RUN apt-get update && apt-get install -y \
    curl pkg-config libssl-dev nodejs npm \
    && rm -rf /var/lib/apt/lists/*

# Install Trunk + WASM target
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked

WORKDIR /frontend
COPY src/frontend/ .

# Install Tailwind CLI (used by Trunk build hook if configured)
RUN npm install -g tailwindcss 2>/dev/null || true

RUN trunk build --release

# ── Stage 2: Build Axum backend ──────────────────────────────────────────────
FROM rust:1.93-slim AS backend-builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl    

WORKDIR /backend
COPY src/backend/ .

# Requires .sqlx/ query cache; run `cargo sqlx prepare` locally and commit it.
ENV SQLX_OFFLINE=true
RUN cargo build --release --target x86_64-unknown-linux-musl

# ── Stage 3: Minimal runtime image ───────────────────────────────────────────
FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=backend-builder /backend/target/x86_64-unknown-linux-musl/release/mosaic-app /usr/local/bin/mosaic-app
# Axum's SPA handler serves these as static files
COPY --from=frontend-builder /frontend/dist /app/static

EXPOSE 8080
CMD ["mosaic-app"]
