FROM postgres:18-alpine

RUN apk add --no-cache bash

COPY backup.sh /backup.sh
COPY restore.sh /restore.sh
COPY crontab /etc/crontabs/root
RUN chmod +x /backup.sh /restore.sh

# Default entrypoint: run crond in foreground (scheduled backups)
# Override with /backup.sh for one-shot or /restore.sh for restore
CMD ["crond", "-f", "-l", "2"]