# Mosaic — Restore Runbook

## Overview

This runbook covers disaster recovery for the Mosaic stack. It assumes you have a backup archive created by the automated backup container (`backup.sh`).

Backups are stored in the `mosaic-backups` Docker volume, mounted at `/backups` inside the backup container. Each archive contains `pg_dump` files for both `portal_db` (Mosaic portal data) and `nocodb` (NocoDB tables and row data).

**What backups preserve:**
- All portal data: users, sessions, dashboards, panels, dataset templates
- All NocoDB data: tables, rows, form views, column definitions
- Grafana's own database is NOT backed up (it uses a separate `grafana` database). Grafana dashboards are re-created by the re-provision step.

**What re-provisioning restores:**
- Grafana dashboards (created from template field definitions)
- NocoDB form share links (new UUIDs are generated and stored)
- Portal panel source URLs (automatically rewritten to point at new Grafana UIDs)

---

## Quick Reference

```bash
# List available backups
docker compose run --rm backup ls -lah /backups/daily/ /backups/weekly/

# Restore from a specific backup
docker compose run --rm backup /restore.sh /backups/daily/mosaic_20260330_020000.tar.gz

# Run a manual backup right now
docker compose run --rm backup
```

---

## Full Restore Procedure

### 1. Stop the application services

Keep Postgres running — the restore script needs it. Stop everything else to prevent NocoDB or the portal from writing while we restore.

```bash
cd src/
docker compose stop portal portal-init nocodb grafana bootstrapper
```

### 2. Pick a backup to restore

```bash
docker compose run --rm backup ls -lah /backups/daily/ /backups/weekly/
```

Choose the most recent backup before the incident. Weekly backups are Sunday snapshots; daily backups cover the last 7 days.

### 3. Run the restore

```bash
docker compose run --rm backup /restore.sh /backups/daily/mosaic_20260330_020000.tar.gz
```

The script will:
- Ask for confirmation (type `yes`)
- Terminate active connections to both databases
- Drop and recreate `portal_db` and `nocodb`
- Restore both databases from the dump files

### 4. Restart the full stack

```bash
docker compose down
docker compose up -d
```

Wait for all services to become healthy:

```bash
docker compose ps
```

The bootstrapper will run again and generate fresh Grafana service account tokens and NocoDB API tokens. The portal will pick these up from `/shared/bootstrap.env`.

### 5. Re-provision all templates

The database restore brings back your template definitions and NocoDB row data, but Grafana dashboards need to be re-created (they live in Grafana's own database which is separate).

**Option A: Via the UI**
1. Log in as admin
2. Go to Templates
3. Click "⟳ Re-provision" on each template

**Option B: Via curl**
```bash
# Log in
curl -c cookies.txt -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"YOUR_PASSWORD"}'

# List all template IDs
curl -b cookies.txt http://localhost:8080/api/templates | jq -r '.[].id'

# Re-provision each (replace <ID> with actual UUID)
curl -b cookies.txt -X POST http://localhost:8080/api/templates/<ID>/reprovision
```

**Option C: Re-provision all in one shot**
```bash
curl -b cookies.txt http://localhost:8080/api/templates \
  | jq -r '.[].id' \
  | while read id; do
      echo "Re-provisioning ${id}..."
      curl -b cookies.txt -X POST "http://localhost:8080/api/templates/${id}/reprovision"
      echo ""
    done
```

### 6. Verify

- [ ] Portal loads at `http://localhost:8080`
- [ ] Dashboards appear in the sidebar
- [ ] Grafana panels render data in dashboard views
- [ ] NocoDB entry forms load when clicked
- [ ] Historical data is visible in Grafana charts

---

## Backup Schedule

The backup container runs on a schedule defined in `docker-compose.yml`. Default: daily at 02:00 UTC.

**Retention:**
- 7 daily backups (~1 week of point-in-time recovery)
- 4 weekly backups (~1 month of weekly snapshots)

**Storage:** A typical Mosaic backup is 1–10 MB compressed. With default retention, expect ~50–100 MB total. The rotation logic in `backup.sh` enforces the limits automatically.

**To change the schedule**, edit the `backup` service's `command` in `docker-compose.yml`. The container uses `crond` with a crontab entry.

---

## Troubleshooting

### "Postgres not ready" during restore
Postgres may not be running. Start it first:
```bash
docker compose up -d postgres
# Wait a few seconds, then retry the restore
```

### "database is being accessed by other users"
The restore script terminates connections automatically, but if it still fails:
```bash
docker compose stop portal nocodb grafana
# Then retry the restore
```

### NocoDB shows empty tables after restore
NocoDB caches metadata. Restart it:
```bash
docker compose restart nocodb
```

### Grafana shows "dashboard not found" after re-provision
Check that the panel `source_url` was updated:
```sql
SELECT id, source_url FROM portal.panels WHERE source_url LIKE '/proxy/grafana/%';
```
If the URL still references an old UID, re-provision the template again.