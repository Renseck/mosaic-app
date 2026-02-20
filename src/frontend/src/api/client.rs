use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("network error: {0}")]
    Network(String),
    #[error("server error {status}: {message}")]
    Server { status: u16, message: String },
    #[error("deserialize error: {0}")]
    Deserialize(String),
}

impl From<gloo_net::Error> for ApiError {
    fn from(e: gloo_net::Error) -> Self {
        ApiError::Network(e.to_string())
    }
}

/* ============================================================================================== */
async fn check(response: Response) -> Result<Response, ApiError> {
    if response.ok() {
        Ok(response)
    } else {
        let status = response.status();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(ApiError::Server { status, message })
    }
}

/* ============================================================================================== */
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let resp = check(Request::get(path).send().await?).await?;
    resp.json::<T>().await.map_err(|e| ApiError::Deserialize(e.to_string()))
}

/* ============================================================================================== */
pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let resp = check(
        Request::post(path)
            .json(body)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await?,
    )
    .await?;
    resp.json::<T>().await.map_err(|e| ApiError::Deserialize(e.to_string()))
}

/* ============================================================================================== */
pub async fn post_empty(path: &str) -> Result<(), ApiError> {
    check(Request::post(path).send().await?).await?;
    Ok(())
}

/* ============================================================================================== */
pub async fn put_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let resp = check(
        Request::put(path)
            .json(body)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await?,
    )
    .await?;
    resp.json::<T>().await.map_err(|e| ApiError::Deserialize(e.to_string()))
}

/* ============================================================================================== */
pub async fn delete(path: &str) -> Result<(), ApiError> {
    check(Request::delete(path).send().await?).await?;
    Ok(())
}