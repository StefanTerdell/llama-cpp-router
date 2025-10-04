use anyhow::Result;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use reqwest::StatusCode;
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::Config;

struct ApiState {
    config: Config,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Anyhow: {0:#?}")]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type ApiResult<T, E = ApiError> = Result<Json<T>, E>;

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self:#?}")).into_response()
    }
}

pub async fn serve(config: Config) -> Result<()> {
    let port = config.port;
    let state = Arc::new(ApiState { config });

    let api = Router::new()
        .route("/v1/models", get(get_v1_models_handler))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    println!("Server listening on {port}");

    axum::serve(listener, api).await?;

    Ok(())
}

#[axum::debug_handler]
async fn get_v1_models_handler(State(state): State<Arc<ApiState>>) -> ApiResult<Vec<Value>> {
    let mut models: Vec<Value> = Vec::new();

    for server in &state.config.servers {
        let mut server_models = reqwest::get(format!("http://localhost:{}/v1/models", server.port))
            .await?
            .json::<Vec<Value>>()
            .await?;
        models.append(&mut server_models);
    }

    Ok(Json(models))
}
