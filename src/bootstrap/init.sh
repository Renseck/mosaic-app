#!/usr/bin/env bash
set -euo pipefail

SHARED_DIR="/shared"
ENV_FILE="${SHARED_DIR}/bootstrap.env"
GRAFANA_URL="http://grafana:3000"
NOCODB_URL="http://nocodb:8080"

# If bootstrap.env exists, verify tokens are still valid
if [ -f "${ENV_FILE}" ]; then
    echo "🔍 bootstrap.env exists — validating tokens..."
    set -a; . "${ENV_FILE}"; set +a

    GRAFANA_OK=false
    NOCODB_OK=false

    # Check Grafana token
    if [ -n "${GRAFANA_SERVICE_ACCOUNT_TOKEN:-}" ]; then
        HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
            -H "Authorization: Bearer ${GRAFANA_SERVICE_ACCOUNT_TOKEN}" \
            "${GRAFANA_URL}/api/org")
        [ "$HTTP_CODE" = "200" ] && GRAFANA_OK=true
    fi

    # Check NocoDB token
    if [ -n "${NOCODB_API_TOKEN:-}" ]; then
        HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
            -H "xc-token: ${NOCODB_API_TOKEN}" \
            "${NOCODB_URL}/api/v1/meta/tables")
        [ "$HTTP_CODE" = "200" ] && NOCODB_OK=true
    fi

    if [ "$GRAFANA_OK" = true ] && [ "$NOCODB_OK" = true ]; then
        echo "✅ Tokens validated — skipping bootstrap"
        exit 0
    fi

    echo "⚠️  Stale tokens detected — re-bootstrapping..."
    rm -f "${ENV_FILE}"
fi

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
#                                                                              #
#  The idea is that we'll bounce between two service accounts, cycling between #
#  two names between launches, and using the latest one to clean the previous. #
# --------------------------------------------------------------------------- #

wait_for "Grafana" "${GRAFANA_URL}/api/health"

SA_NAMES=("portal-sa-a" "portal-sa-b")
CREATED_SA=""
GRAFANA_TOKEN=""


# Try to create one of the two service accounts
for SA_NAME in "${SA_NAMES[@]}"; do
    echo "🔧 Trying to create Grafana service account '${SA_NAME}'..."
    SA_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${GRAFANA_URL}/api/serviceaccounts" \
        -u "${GRAFANA_ADMIN_USER}:${GRAFANA_ADMIN_PASS}" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"${SA_NAME}\",\"role\":\"Admin\",\"isDisabled\":false}")

    if [ "$SA_RESPONSE" = "201" ]; then
        CREATED_SA="${SA_NAME}"
        break
    fi
    echo "   '${SA_NAME}' already exists, trying next..."
done

if [ -z "$CREATED_SA" ]; then
    echo "❌ Could not create either service account"
    exit 1
fi

# Get the ID of the just-created SA
SA_SEARCH=$(curl -sf "${GRAFANA_URL}/api/serviceaccounts/search?query=${CREATED_SA}" \
    -u "${GRAFANA_ADMIN_USER}:${GRAFANA_ADMIN_PASS}")
SA_ID=$(echo "$SA_SEARCH" | jq -r ".serviceAccounts[] | select(.name==\"${CREATED_SA}\") | .id")

# Create a token for it
echo "🔧 Creating token for '${CREATED_SA}'..."
TOKEN_RESPONSE=$(curl -sf -X POST "${GRAFANA_URL}/api/serviceaccounts/${SA_ID}/tokens" \
    -u "${GRAFANA_ADMIN_USER}:${GRAFANA_ADMIN_PASS}" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"portal-token-${CREATED_SA##*-}\"}")

GRAFANA_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.key')
echo "   Token created successfully"

# Now use the new token to clean up the OTHER service account
for SA_NAME in "${SA_NAMES[@]}"; do
    [ "$SA_NAME" = "$CREATED_SA" ] && continue

    echo "🧹 Cleaning up stale service account '${SA_NAME}'..."
    OLD_SEARCH=$(curl -sf "${GRAFANA_URL}/api/serviceaccounts/search?query=${SA_NAME}" \
        -H "Authorization: Bearer ${GRAFANA_TOKEN}")
    OLD_ID=$(echo "$OLD_SEARCH" | jq -r ".serviceAccounts[] | select(.name==\"${SA_NAME}\") | .id")

    if [ -n "$OLD_ID" ] && [ "$OLD_ID" != "null" ]; then
        curl -sf -X DELETE "${GRAFANA_URL}/api/serviceaccounts/${OLD_ID}" \
            -H "Authorization: Bearer ${GRAFANA_TOKEN}" || true
        echo "   Deleted '${SA_NAME}' (ID: ${OLD_ID})"
    fi
done

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