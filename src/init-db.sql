-- Create additional databases for Grafana and NocoDB
-- (portal_db is made by POSTGRES_DB env var)
CREATE DATABASE grafana;
CREATE DATABASE nocodb;
CREATE DATABASE nocodb_data OWNER portal;