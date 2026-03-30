#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Mosaic — Database restore from backup archive
#
# Restores both portal_db and nocodb databases from a backup archive created
# by backup.sh. This script is designed to be run MANUALLY as part of a
# disaster recovery procedure — see RESTORE_RUNBOOK.md.
#
# Usage:
#   ./restore.sh <path-to-backup-archive>
#   ./restore.sh /backups/daily/mosaic_20260330_020000.tar.gz
#
# This script will:
#   1. Extract the archive
#   2. Drop and recreate both databases
#   3. Restore from the pg_dump files
#   4. Print next steps (re-provision templates)
#
# Expected environment variables:
#   PGHOST, PGUSER, PGPASSWORD  — Postgres connection (superuser or owner)
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <backup-archive.tar.gz>"
    echo ""
    echo "Available backups:"
    if [ -d "/backups" ]; then
        find /backups -name 'mosaic_*.tar.gz' -print | sort -r | head -20
    else
        echo "  (no /backups directory found)"
    fi
    exit 1
fi

ARCHIVE="$1"

if [ ! -f "${ARCHIVE}" ]; then
    echo "❌ File not found: ${ARCHIVE}"
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Mosaic Restore"
echo "  Archive: $(basename "${ARCHIVE}")"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "⚠️  WARNING: This will DROP and RECREATE the portal_db and nocodb databases."
echo "   All current data in these databases will be lost."
echo ""
read -p "Type 'yes' to continue: " CONFIRM
if [ "${CONFIRM}" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

# ── Wait for Postgres ────────────────────────────────────────────────────────
echo ""
echo "⏳ Waiting for Postgres..."
for i in $(seq 1 30); do
    if pg_isready -h "${PGHOST}" -U "${PGUSER}" -q 2>/dev/null; then
        echo "✅ Postgres is ready"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "❌ Postgres not ready after 30 attempts"
        exit 1
    fi
    sleep 2
done

# ── Extract archive ──────────────────────────────────────────────────────────
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT

echo "📦 Extracting archive..."
tar -xzf "${ARCHIVE}" -C "${WORK_DIR}"

if [ ! -f "${WORK_DIR}/portal_db.dump" ] || [ ! -f "${WORK_DIR}/nocodb.dump" ]; then
    echo "❌ Archive is missing expected dump files (portal_db.dump, nocodb.dump)"
    exit 1
fi

if [ -f "${WORK_DIR}/metadata.json" ]; then
    echo "   Backup metadata:"
    cat "${WORK_DIR}/metadata.json"
    echo ""
fi

# ── Terminate active connections ─────────────────────────────────────────────
echo "🔌 Terminating active connections..."
for DB in portal_db nocodb; do
    psql -h "${PGHOST}" -U "${PGUSER}" -d postgres -c \
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${DB}' AND pid <> pg_backend_pid();" \
        2>/dev/null || true
done

# ── Drop and recreate databases ──────────────────────────────────────────────
echo "🗑️  Dropping databases..."
for DB in portal_db nocodb; do
    psql -h "${PGHOST}" -U "${PGUSER}" -d postgres -c \
        "DROP DATABASE IF EXISTS ${DB};" 2>/dev/null
    echo "   Dropped ${DB}"
done

echo "🔨 Recreating databases..."
psql -h "${PGHOST}" -U "${PGUSER}" -d postgres -c \
    "CREATE DATABASE portal_db OWNER ${PGUSER};"
psql -h "${PGHOST}" -U "${PGUSER}" -d postgres -c \
    "CREATE DATABASE nocodb OWNER ${PGUSER};"
echo "   Created portal_db and nocodb"

# ── Restore dumps ────────────────────────────────────────────────────────────
echo "📥 Restoring portal_db..."
pg_restore -h "${PGHOST}" -U "${PGUSER}" -d portal_db \
    --no-owner --no-privileges \
    "${WORK_DIR}/portal_db.dump"
echo "   ✅ portal_db restored"

echo "📥 Restoring nocodb..."
pg_restore -h "${PGHOST}" -U "${PGUSER}" -d nocodb \
    --no-owner --no-privileges \
    "${WORK_DIR}/nocodb.dump"
echo "   ✅ nocodb restored"

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✅ Database restore complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "  1. Restart the full stack:  make docker-down && make docker-up"
echo "  2. Log in as admin"
echo "  3. Re-provision all templates (Grafana dashboards + NocoDB forms):"
echo ""
echo "     # Get a session cookie"
echo "     curl -c cookies.txt -X POST http://localhost:8080/api/auth/login \\"
echo "       -H 'Content-Type: application/json' \\"
echo "       -d '{\"username\":\"admin\",\"password\":\"YOUR_PASSWORD\"}'"
echo ""
echo "     # List templates"
echo "     curl -b cookies.txt http://localhost:8080/api/templates | jq '.[].id'"
echo ""
echo "     # Re-provision each"
echo "     curl -b cookies.txt -X POST http://localhost:8080/api/templates/<ID>/reprovision"
echo ""
echo "  Or use the Re-provision button in the Mosaic UI for each template."
