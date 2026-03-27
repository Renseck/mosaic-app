#!/usr/bin/env sh
set -eu

BOOTSTRAP_ENV="/shared/bootstrap.env"

# Source bootstrap-generated tokens if available
if [ -f "$BOOTSTRAP_ENV" ]; then
    echo "📦 Loading bootstrap.env..."
    set -a
    . "$BOOTSTRAP_ENV"
    set +a
fi

# Wait for Postgres to be reachable (DNS resolution + TCP connect)
echo "⏳ Waiting for Postgres..."
MAX_RETRIES=30
RETRY=0
while true; do
    if nc -z -w2 postgres 5432 2>/dev/null; then
        echo "✅ Postgres is reachable"
        break
    fi
 
    RETRY=$((RETRY + 1))
    if [ "$RETRY" -ge "$MAX_RETRIES" ]; then
        echo "❌ Postgres not reachable after ${MAX_RETRIES} attempts"
        exit 1
    fi
 
    echo "   Attempt ${RETRY}/${MAX_RETRIES} — retrying in 2s..."
    sleep 2
done
 
exec mosaic-app