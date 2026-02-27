use axum::extract::State;

use crate::auth::middleware::AuthenticatedUser;
use crate::error::AppError;
use crate::proxy::{self, ProxyTarget};
use crate::AppState;

struct GrafanaProxy {
    base_url: String,
    token: String,
}

impl ProxyTarget for GrafanaProxy {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn prefix(&self) -> &str {
        ""  // Don't strip — Grafana serves from /proxy/grafana/ via SERVE_FROM_SUB_PATH
    }

    fn inject_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder.header(
            axum::http::header::AUTHORIZATION, 
            format!("Bearer {}", self.token),
        )
    }
}

/* ============================================================================================== */
/// ANY /proxy/grafana/{*path}
///
/// Requires a valid portal session. Strips the `/proxy/grafana` prefix,
/// forwards the request to the internal Grafana instance, and injects
/// the configured service account token.
pub async fn grafana_proxy(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    req: axum::extract::Request,
) ->Result<axum:: response::Response, AppError> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, 50 * 1024 * 1024)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("request body read error: {e}")))?;

    let target = GrafanaProxy {
        base_url: state.config.grafana_internal_url.clone(),
        token: state.config.grafana_service_account_token.clone(),
    };

    proxy::forward(
        &target,
        &state.http_client,
        parts.method,
        parts.uri,
        parts.headers,
        body_bytes,
    )
    .await
}