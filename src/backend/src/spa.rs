use axum::{
    http::{StatusCode, Uri}, response::{Html, IntoResponse, Response}
};
use std::path::PathBuf;
use tokio::fs;

fn dist_dir() -> String {
    std::env::var("STATIC_DIR").unwrap_or_else(|_| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/dist").to_string()
    })
}

const DIST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/dist");

/// Serve a static file from the frontend dist directory, or fall back to index.html.
pub async fn spa_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let file_path = PathBuf::from(dist_dir()).join(path);

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

    let index = PathBuf::from(dist_dir()).join("index.html");
    match fs::read_to_string(&index).await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "Frontend not built (looked in: {}). Run: cd frontend && trunk build --release",
                index.display()
            ),
        )
            .into_response(),
    }
}