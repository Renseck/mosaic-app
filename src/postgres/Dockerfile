FROM postgres:18-alpine
COPY init-db.sql /docker-entrypoint-initdb.d/01-init.sql
COPY backend/src/db/migrations/20260217000001_initial_schema.sql /docker-entrypoint-initdb.d/02-schema.sql
