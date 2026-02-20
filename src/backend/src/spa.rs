use axum::{
    http::{StatusCode, Uri}, response::{Html, IntoResponse, Response}
};
use std::path::PathBuf;
use tokio::fs;

const DIST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/dist");

/// Serve a static file from the frontend dist directory, or fall back to index.html.
pub async fn spa_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let file_path = PathBuf::from(DIST_DIR).join(path);

    // Try to serve the exact file first.
    if file_path.is_file() {
        match fs::read(&file_path).await {
            Ok(bytes) => {
                let mime = mime_guess::from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();
                return (
                    StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, mime)],
                    bytes,
                )
                    .into_response();
            }
            Err(_) => {}
        }
    }

    // Fall back to index.html for client-side routing.
    let index = PathBuf::from(DIST_DIR).join("index.html");
    match fs::read_to_string(&index).await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "Frontend not built (looked in: {}). Run: cd frontend && trunk build --release",
                index.display()
            )
            
        )
            .into_response(),
    }
}