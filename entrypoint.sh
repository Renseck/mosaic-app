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

exec mosaic-app