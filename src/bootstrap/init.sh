#!/usr/bin/env bash
set -euo pipefail

SHARED_DIR="/shared"
ENV_FILE="${SHARED_DIR}/bootstrap.env"

# Skip if already bootstrapped
if [ -f "${ENV_FILE}" ]; then
    echo "✅ bootstrap.env already exists — skipping"
    exit 0
fi

GRAFANA_URL="http://grafana:3000"
NOCODB_URL="http://nocodb:8080"
GRAFANA_ADMIN_USER="admin"
GRAFANA_ADMIN_PASS="${GF_SECURITY_ADMIN_PASSWORD:-admin}"
NOCODB_OWNER_EMAIL="${NOCODB_OWNER_EMAIL:-admin@portal.local}"
NOCODB_OWNER_PASS="${NOCODB_OWNER_PASSWORD:-Portal1234!}"

# --------------------------------------------------------------------------- #
#  Helpers                                                                     #
# --------------------------------------------------------------------------- #

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

# --------------------------------------------------------------------------- #
#  Grafana bootstrap                                                           #
# --------------------------------------------------------------------------- #

wait_for "Grafana" "${GRAFANA_URL}/api/health"

# Create service account
echo "🔧 Creating Grafana service account..."
SA_RESPONSE=$(curl -sf -X POST "${GRAFANA_URL}/api/serviceaccounts" \
    -u "${GRAFANA_ADMIN_USER}:${GRAFANA_ADMIN_PASS}" \
    -H "Content-Type: application/json" \
    -d '{"name":"portal-sa","role":"Admin","isDisabled":false}')

SA_ID=$(echo "$SA_RESPONSE" | jq -r '.id')
echo "   Service account ID: ${SA_ID}"

# Create token for service account
echo "🔧 Creating Grafana service account token..."
TOKEN_RESPONSE=$(curl -sf -X POST "${GRAFANA_URL}/api/serviceaccounts/${SA_ID}/tokens" \
    -u "${GRAFANA_ADMIN_USER}:${GRAFANA_ADMIN_PASS}" \
    -H "Content-Type: application/json" \
    -d '{"name":"portal-token"}')

GRAFANA_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.key')
echo "   Token created successfully"

# --------------------------------------------------------------------------- #
#  NocoDB bootstrap                                                            #
# --------------------------------------------------------------------------- #

wait_for "NocoDB" "${NOCODB_URL}/api/v1/health"

# Sign up the owner (first user becomes super admin)
echo "🔧 Creating NocoDB owner account..."
SIGNUP_RESPONSE=$(curl -sf -X POST "${NOCODB_URL}/api/v1/auth/user/signup" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${NOCODB_OWNER_EMAIL}\",\"password\":\"${NOCODB_OWNER_PASS}\"}" \
    || true)

# Login to get auth token
echo "🔧 Logging in to NocoDB..."
LOGIN_RESPONSE=$(curl -sf -X POST "${NOCODB_URL}/api/v1/auth/user/signin" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"${NOCODB_OWNER_EMAIL}\",\"password\":\"${NOCODB_OWNER_PASS}\"}")

NC_AUTH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')

# Create an API token
echo "🔧 Creating NocoDB API token..."
API_TOKEN_RESPONSE=$(curl -sf -X POST "${NOCODB_URL}/api/v1/tokens" \
    -H "Content-Type: application/json" \
    -H "xc-auth: ${NC_AUTH_TOKEN}" \
    -d '{"description":"portal-api-token"}')

NOCODB_TOKEN=$(echo "$API_TOKEN_RESPONSE" | jq -r '.token')
echo "   API token created successfully"

# --------------------------------------------------------------------------- #
#  Write results                                                               #
# --------------------------------------------------------------------------- #

echo "📝 Writing bootstrap.env..."
cat > "${ENV_FILE}" <<EOF
GRAFANA_SERVICE_ACCOUNT_TOKEN=${GRAFANA_TOKEN}
GRAFANA_DATASOURCE_UID=portal-postgres-ds
NOCODB_API_TOKEN=${NOCODB_TOKEN}
EOF

echo "🎉 Bootstrap complete!"