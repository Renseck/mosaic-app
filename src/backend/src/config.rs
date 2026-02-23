use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url:                   String,
    pub grafana_internal_url:           String,
    pub grafana_service_account_token:  String,
    pub grafana_datasource_uid:         String,
    pub nocodb_internal_url:            String,
    pub nocodb_api_token:               String,
    pub nocodb_pg_host:                 String,
    pub nocodb_pg_port:                 u16,
    pub nocodb_pg_user:                 String,
    pub nocodb_pg_password:             String,
    pub nocodb_pg_database:             String,
    pub session_secret:                 String,
    pub session_ttl_hours:              u64,
    pub bind_address:                   String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url:                   env::var("DATABASE_URL")?,
            grafana_internal_url:           env::var("GRAFANA_INTERNAL_URL")?,
            grafana_service_account_token:  env::var("GRAFANA_SERVICE_ACCOUNT_TOKEN")?,
            grafana_datasource_uid:         env::var("GRAFANA_DATASOURCE_UID")
                                                .unwrap_or_else(|_| "nocodb-pg".to_string()),
            nocodb_internal_url:            env::var("NOCODB_INTERNAL_URL")?,
            nocodb_api_token:               env::var("NOCODB_API_TOKEN")?,
            nocodb_pg_host:                 env::var("NOCODB_PG_HOST")
                                                .unwrap_or_else(|_| "postgres".to_string()),
            nocodb_pg_port:                 env::var("NOCODB_PG_PORT")
                                                .unwrap_or_else(|_| "5432".to_string())
                                                .parse()
                                                .expect("NOCODB_PG_PORT must be a valid u16"),
            nocodb_pg_user:                 env::var("NOCODB_PG_USER")
                                                .unwrap_or_else(|_| "portal".to_string()),
            nocodb_pg_password:             env::var("NOCODB_PG_PASSWORD")?,
            nocodb_pg_database:             env::var("NOCODB_PG_DATABASE")
                                                .unwrap_or_else(|_| "nocodb_data".to_string()),
            session_secret:                 env::var("SESSION_SECRET")?,
            session_ttl_hours:              env::var("SESSION_TTL_HOURS")
                                                .unwrap_or_else(|_| "168".to_string())
                                                .parse()
                                                .expect("SESSION_TTL_HOURS must be a valid u64"),
            bind_address:                   env::var("BIND_ADDRESS")
                                                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        })
    }
}