# Scheduled backups (automatic)

The `backup` container runs `crond` by default, executing `backup.sh` daily at 02:00 UTC.
Logs are at `/var/log/backup.log` inside the container:

```bash
docker logs mosaic-backup
```

## Manual backup (one-shot)

```bash
docker compose run --rm backup /backup.sh
```

## List backups

```bash
docker compose run --rm backup ls -lah /backups/daily/ /backups/weekly/
```

## Restore
```bash
# Stop application services first
docker compose stop portal portal-init nocodb grafana bootstrapper

# Run restore interactively
docker compose run --rm backup /restore.sh /backups/daily/mosaic_20260330_020000.tar.gz

# Restart everything
docker compose down && docker compose up -d
```

Then re-provision templates via the UI or curl (see RESTORE_RUNBOOK.md).

---

## Verify backups are working

After the first scheduled run (or after a manual run), check:

```bash
# Should show at least one archive
docker compose run --rm backup ls -lah /backups/daily/

# Check the archive contents
docker compose run --rm backup tar -tzf /backups/daily/mosaic_20260330_020000.tar.gz
# Expected: portal_db.dump, nocodb.dump, metadata.json
```