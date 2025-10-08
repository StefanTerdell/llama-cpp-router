use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use std::fmt::Display;

// use crate::models::ModelsError;

pub type ApiResult<T = Response> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiErrorResponseBody {
    Text(String),
    Json(Value),
    None,
}

impl Display for ApiErrorResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiErrorResponseBody::Text(text) => f.write_fmt(format_args!("\"{text}\"")),
            ApiErrorResponseBody::Json(value) => {
                f.write_str(&serde_json::to_string_pretty(value).unwrap())
            }
            ApiErrorResponseBody::None => f.write_str("None"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    // #[error(transparent)]
    // Url(#[from] url::ParseError),
    // #[error(transparent)]
    // Json(#[from] serde_json::Error),
    #[error("Model alias {0} not found")]
    Notfound(String),
    // #[error(transparent)]
    // Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    ReqwestSse(reqwest_sse::error::EventSourceError),
    // #[error("{0}")]
    // ReqwestSseEvent(reqwest_sse::error::EventError),
    // #[error(transparent)]
    // Models(#[from] ModelsError),
    // #[error(transparent)]
    // Config(#[from] ConfigError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
    #[error("HTTP error response from {url}: Status Code: '{status_code}', Body: '{body}'")]
    HttpErrorResponse {
        method: reqwest::Method,
        url: String,
        status_code: reqwest::StatusCode,
        body: ApiErrorResponseBody,
    },
}

impl ApiError {
    pub async fn from_http_error_response(
        method: reqwest::Method,
        url: impl Display,
        response: reqwest::Response,
    ) -> Self {
        Self::HttpErrorResponse {
            method,
            url: url.to_string(),
            status_code: response.status(),
            body: if let Ok(text) = response.text().await {
                if let Ok(value) = serde_json::from_str(&text) {
                    ApiErrorResponseBody::Json(value)
                } else {
                    ApiErrorResponseBody::Text(text)
                }
            } else {
                ApiErrorResponseBody::None
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match &self {
            // ApiError::Url(_)
            // | ApiError::Json(_)
            // | ApiError::Reqwest(_)
            // | ApiError::ReqwestSse(_)
            // | ApiError::ReqwestSseEvent(_)
            // | ApiError::Models(_)
            // | ApiError::Config(_) => {
            //     (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            // }
            //
            ApiError::ReqwestSse(_) | ApiError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            ApiError::Notfound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            ApiError::HttpErrorResponse { status_code, .. } => {
                (*status_code, self.to_string()).into_response()
            }
        }
    }
}
