use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type ApiResult<T = Response> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Model alias {0} not found")]
    NotFound(String),
    #[error("Request rejected: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        eprintln!("Responding with error: {self:#?}");

        (
            match &self {
                ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
                ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            },
            self.to_string(),
        )
            .into_response()
    }
}
