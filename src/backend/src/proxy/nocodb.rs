use axum::extract::State;

use crate::auth::middleware::AuthenticatedUser;
use crate::error::AppError;
use crate::proxy::{self, ProxyTarget};
use crate::AppState;

struct NocodbProxy {
    base_url: String,
    token: String,
}

impl ProxyTarget for NocodbProxy {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn prefix(&self) -> &str {
        "/proxy/nocodb"
    }

    fn inject_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder.header(
            "xc-token", 
            &self.token,
        )
    }
}

/* ============================================================================================== */
/// ANY /proxy/nocodb/{*path}
///
/// Requires a valid portal session. Strips the `/proxy/nocodb` prefix,
/// forwards the request to the internal Nocodb instance, and injects
/// the configured API token.
pub async fn nocodb_proxy(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    req: axum::extract::Request,
) ->Result<axum:: response::Response, AppError> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, 50 * 1024 * 1024)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("request body read error: {e}")))?;

    let target = NocodbProxy {
        base_url: state.config.nocodb_internal_url.clone(),
        token: state.config.nocodb_api_token.clone(),
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