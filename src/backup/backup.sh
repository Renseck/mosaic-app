#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Mosaic — Automated PostgreSQL backup with rotation
#
# Dumps both portal_db and nocodb databases, compresses them into a single
# timestamped archive, and rotates old backups to prevent unbounded growth.
#
# Retention policy (configurable via env vars):
#   KEEP_DAILY=7    — keep the last 7 daily backups
#   KEEP_WEEKLY=4   — keep the last 4 weekly backups (Sundays)
#
# Expected environment variables:
#   PGHOST, PGUSER, PGPASSWORD   — Postgres connection details
#   BACKUP_DIR                   — where to store backups (default: /backups)
#   KEEP_DAILY, KEEP_WEEKLY      — retention counts
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
KEEP_DAILY="${KEEP_DAILY:-7}"
KEEP_WEEKLY="${KEEP_WEEKLY:-4}"
TIMESTAMP="$(date -u +%Y%m%d_%H%M%S)"
DAY_OF_WEEK="$(date -u +%u)"  # 1=Monday, 7=Sunday

DAILY_DIR="${BACKUP_DIR}/daily"
WEEKLY_DIR="${BACKUP_DIR}/weekly"

mkdir -p "${DAILY_DIR}" "${WEEKLY_DIR}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Mosaic Backup — ${TIMESTAMP}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ── Wait for Postgres ────────────────────────────────────────────────────────
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

# ── Dump databases ───────────────────────────────────────────────────────────
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT

echo "📦 Dumping portal_db..."
pg_dump -h "${PGHOST}" -U "${PGUSER}" -d portal_db \
    --format=custom --compress=6 \
    -f "${WORK_DIR}/portal_db.dump"

echo "📦 Dumping nocodb..."
pg_dump -h "${PGHOST}" -U "${PGUSER}" -d nocodb \
    --format=custom --compress=6 \
    -f "${WORK_DIR}/nocodb.dump"

# ── Package into a single archive ────────────────────────────────────────────
ARCHIVE_NAME="mosaic_${TIMESTAMP}.tar.gz"
ARCHIVE_PATH="${DAILY_DIR}/${ARCHIVE_NAME}"

# Include a metadata file for easier identification during restore
cat > "${WORK_DIR}/metadata.json" <<EOF
{
  "timestamp": "${TIMESTAMP}",
  "databases": ["portal_db", "nocodb"],
  "pg_version": "$(pg_dump --version | head -1)"
}
EOF

tar -czf "${ARCHIVE_PATH}" -C "${WORK_DIR}" \
    portal_db.dump nocodb.dump metadata.json

ARCHIVE_SIZE="$(du -h "${ARCHIVE_PATH}" | cut -f1)"
echo "✅ Backup created: ${ARCHIVE_NAME} (${ARCHIVE_SIZE})"

# ── Promote to weekly on Sundays ─────────────────────────────────────────────
if [ "${DAY_OF_WEEK}" -eq 7 ]; then
    cp "${ARCHIVE_PATH}" "${WEEKLY_DIR}/${ARCHIVE_NAME}"
    echo "📌 Promoted to weekly backup"
fi

# ── Rotate old backups ───────────────────────────────────────────────────────
echo "🔄 Rotating backups (keep ${KEEP_DAILY} daily, ${KEEP_WEEKLY} weekly)..."

# Sort by name (which sorts by timestamp), delete oldest beyond retention
rotate_dir() {
    local dir="$1" keep="$2"
    local count
    count="$(find "${dir}" -maxdepth 1 -name 'mosaic_*.tar.gz' | wc -l)"
    if [ "${count}" -gt "${keep}" ]; then
        local to_delete=$((count - keep))
        find "${dir}" -maxdepth 1 -name 'mosaic_*.tar.gz' -print0 \
            | sort -z \
            | head -z -n "${to_delete}" \
            | xargs -0 rm -f
        echo "   Removed ${to_delete} old backup(s) from ${dir}"
    fi
}

rotate_dir "${DAILY_DIR}" "${KEEP_DAILY}"
rotate_dir "${WEEKLY_DIR}" "${KEEP_WEEKLY}"

# ── Summary ──────────────────────────────────────────────────────────────────
DAILY_COUNT="$(find "${DAILY_DIR}" -name 'mosaic_*.tar.gz' | wc -l)"
WEEKLY_COUNT="$(find "${WEEKLY_DIR}" -name 'mosaic_*.tar.gz' | wc -l)"
TOTAL_SIZE="$(du -sh "${BACKUP_DIR}" | cut -f1)"

echo ""
echo "📊 Backup inventory: ${DAILY_COUNT} daily, ${WEEKLY_COUNT} weekly (${TOTAL_SIZE} total)"
echo "🎉 Backup complete!"
