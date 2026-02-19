pub mod grafana;
pub mod nocodb;

use axum::Router;
use bytes::Bytes;

use crate::error::AppError;

/* ============================================================================================== */
/*                                        ProxyTarget trait                                       */
/* ============================================================================================== */

/// Implemented by Grafana and NocoDB proxies to describe where to forward
/// requests and how to authenticate with the upstream service.
pub trait ProxyTarget: Send + Sync {
    /// Internal Docker base URL (e.g. `http://grafana:3000`).
    fn base_url(&self) -> &str;

    /// Path prefix to strip from the incoming request URI (e.g. `/proxy/grafana`).
    fn prefix(&self) -> &str;

    /// Inject upstream service credentials into the outgoing reqwest request.
    fn inject_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder;
}

/* ============================================================================================== */
/*                                     Shared forwarding logic                                    */
/* ============================================================================================== */

/// Strips the target prefix, rewrites the URL, copies safe headers,
/// injects upstream auth, forwards the body, and returns the upstream response.
pub async fn forward<T: ProxyTarget>(
    target: &T,
    client: &reqwest::Client,
    method: axum::http::Method,
    uri: axum::http::Uri,
    headers: axum::http::HeaderMap,
    body_bytes: Bytes,
) -> Result<axum::response::Response, AppError> {
    // Build target URL: strip prefix, preserve path + query string.
    let path = uri.path().strip_prefix(target.prefix()).unwrap_or("/");
    let path = if path.is_empty() { "/" } else { path };
    let query = uri
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let target_url = format!("{}{path}{query}", target.base_url().trim_end_matches('/'));

    // Build outgoing request — forward safe headers only.
    let mut builder = client.request(method, &target_url);
    for (key, value) in &headers {
        let name = key.as_str();
        // Skip hop-by-hop headers, Host (reqwest sets it from the URL),
        // and Cookie (portal session must not be forwarded to upstream).
        if !is_hop_by_hop(name) && name != "host" && name != "cookie" {
            builder = builder.header(key, value);
        }
    }

    // Inject upstream service auth (overwrites any forwarded Authorization).
    builder = target.inject_auth(builder);

    // Always attach body (empty Bytes is fine for GET/HEAD/DELETE).
    builder = builder.body(body_bytes);

    let upstream = builder
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("proxy upstream error: {e}")))?;

    // Map upstream status + safe response headers.
    let status = axum::http::StatusCode::from_u16(upstream.status().as_u16())
        .unwrap_or(axum::http::StatusCode::BAD_GATEWAY);

    let mut resp_builder = axum::response::Response::builder().status(status);
    for (key, value) in upstream.headers() {
        let name = key.as_str();
        // Skip hop-by-hop and content-length (axum will set it from the buffer).
        if !is_hop_by_hop(name) && name != "content-length" {
            resp_builder = resp_builder.header(key, value);
        }
    }

    let resp_bytes = upstream
        .bytes()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("upstream response read error: {e}")))?;

    resp_builder
        .body(axum::body::Body::from(resp_bytes))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("response build error: {e}")))
}

/* ============================================================================================== */
/*                                        Route composition                                       */
/* ============================================================================================== */

pub fn router() -> Router<crate::AppState> {
    use axum::routing::any;
    Router::new()
        .route("/proxy/grafana/{*path}", any(grafana::grafana_proxy))
        .route("/proxy/nocodb/{*path}", any(nocodb::nocodb_proxy))
}

/* ============================================================================================== */
/*                                             Helpers                                            */
/* ============================================================================================== */

fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}