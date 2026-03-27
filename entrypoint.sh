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

# Wait for Postgres to be reachable (DNS + TCP)
echo "⏳ Waiting for Postgres..."
MAX_RETRIES=30
RETRY=0
while ! nc -z postgres 5432 2>/dev/null; do
    RETRY=$((RETRY + 1))
    if [ "$RETRY" -ge "$MAX_RETRIES" ]; then
        echo "❌ Postgres not reachable after ${MAX_RETRIES} attempts, starting anyway..."
        break
    fi
    sleep(1)
done
echo "✅ Postgres is reachable"

exec mosaic-app