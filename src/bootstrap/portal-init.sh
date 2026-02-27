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

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${PORTAL_URL}/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"${PORTAL_ADMIN_USERNAME}\",\"password\":\"${PORTAL_ADMIN_PASSWORD}\"}")

if [ "$HTTP_CODE" = "201" ]; then
    echo "Portal admin user created successfully"
elif [ "$HTTP_CODE" = "409" ] || [ "$HTTP_CODE" = "403" ]; then
    echo "Portal admin user already exists — skipping"
else
    echo "Unexpected response: HTTP ${HTTP_CODE}"
    exit 1
fi

echo "✅ Portal admin user created succesfully"