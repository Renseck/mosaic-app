#!/usr/bin/env bash
set -euo pipefail

PORTAL_URL="http://portal:8080"
PORTAL_ADMIN_USERNAME="${PORTAL_ADMIN_USERNAME:-admin}"
PORTAL_ADMIN_PASSWORD="${PORTAL_ADMIN_PASSWORD:-Owner1234!}"

# --------------------------------------------------------------------------- #
#  Portal admin bootstrap                                                      #
# --------------------------------------------------------------------------- #

# Reuse the same wait_for helper
wait_for() {
    local name="$1" url="$2" max_attempts="${3:-30}"
    echo "⏳ Waiting for ${name}..."
    for i in $(seq 1 "$max_attempts"); do
        if curl -sf "${url}" > /dev/null 2>&1; then
            echo "✅ ${name} is ready"
            return 0
        fi
        sleep 2
    done
    echo "❌ ${name} did not become ready in time"
    exit 1
}

wait_for "Portal" "${PORTAL_URL}/api/health"

curl -sf -X POST "${PORTAL_URL}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"${PORTAL_ADMIN_USERNAME}\",\"password\":\"${PORTAL_ADMIN_PASSWORD}\"}"

echo "✅ Portal admin user created succesfully"