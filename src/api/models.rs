use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Sse},
    routing::get,
};
use serde::Deserialize;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::{
    api::{
        result::{ApiError, ApiResult},
        state::ApiState,
    },
    config::{AliasOrIndex, Config, ModelConfig},
    models::LogsAndTailReceiver,
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/", get(list_model_configs))
        .route(
            "/{alias_or_index}",
            get(get_model_config)
                .post(post_model_config)
                .delete(delete_model_config),
        )
        .route("/{alias_or_index}/logs", get(get_model_logs))
}

#[axum::debug_handler]
async fn list_model_configs(State(state): State<ApiState>) -> Json<Vec<ModelConfig>> {
    Json(state.models().get_loaded_configs().await)
}

#[derive(Debug, Deserialize)]
struct AliasOrIndexPath {
    alias_or_index: String,
}

#[axum::debug_handler]
async fn get_model_config(
    State(state): State<ApiState>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult<Json<Option<ModelConfig>>> {
    let alias = match AliasOrIndex::from(alias_or_index) {
        AliasOrIndex::Alias(alias) => alias,
        AliasOrIndex::Index(index) => {
            let config = Config::load(state.config_path())?;

            config.get_model_config(index)?.alias().to_string()
        }
    };

    Ok(Json(state.models().unload(&alias).await?))
}

#[derive(Debug, Deserialize)]
struct TailAndJsonQuery {
    tail: Option<bool>,
    json: Option<bool>,
}

#[axum::debug_handler]
async fn post_model_config(
    State(state): State<ApiState>,
    Query(TailAndJsonQuery { tail, json }): Query<TailAndJsonQuery>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult {
    let config = Config::load(state.config_path())?;

    let (model_config, logs_and_tail_receiver) =
        state.models().load(&config, &alias_or_index).await?;

    let (tail, json) = (tail.unwrap_or(false), json.unwrap_or(true));

    let response = if tail && let Some(logs_and_tail_receiver) = logs_and_tail_receiver {
        Sse::new(
            BroadcastStream::new(logs_and_tail_receiver.tail_receiver).map(move |result| {
                result.map(|log| {
                    axum::response::sse::Event::default().data(if json {
                        serde_json::to_string_pretty(&log).unwrap()
                    } else {
                        log.to_string()
                    })
                })
            }),
        )
        .into_response()
    } else {
        Json(model_config).into_response()
    };

    Ok(response)
}

#[axum::debug_handler]
async fn delete_model_config(
    State(state): State<ApiState>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult<Json<Option<ModelConfig>>> {
    let alias = match AliasOrIndex::from(alias_or_index) {
        AliasOrIndex::Alias(alias) => alias,
        AliasOrIndex::Index(index) => {
            let config = Config::load(state.config_path())?;

            config.get_model_config(index)?.alias().to_string()
        }
    };

    Ok(Json(state.models().unload(&alias).await?))
}

#[axum::debug_handler]
async fn get_model_logs(
    State(state): State<ApiState>,
    Query(TailAndJsonQuery { tail, json }): Query<TailAndJsonQuery>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult {
    let alias = match AliasOrIndex::from(alias_or_index) {
        AliasOrIndex::Alias(alias) => alias,
        AliasOrIndex::Index(index) => {
            let config = Config::load(state.config_path())?;

            config.get_model_config(index)?.alias().to_string()
        }
    };

    let LogsAndTailReceiver {
        logs,
        tail_receiver,
    } = state
        .models()
        .get_logs_and_receiver(&alias)
        .await
        .ok_or(ApiError::NotFound(alias))?;

    let (tail, json) = (tail.unwrap_or(false), json.unwrap_or(true));

    let response = if tail {
        Sse::new(BroadcastStream::new(tail_receiver).map(move |result| {
            result.map(|log| {
                axum::response::sse::Event::default().data(if json {
                    serde_json::to_string_pretty(&log).unwrap()
                } else {
                    log.to_string()
                })
            })
        }))
        .into_response()
    } else if json {
        Json(logs).into_response()
    } else {
        logs.into_iter()
            .map(|log| log.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .into_response()
    };

    Ok(response)
}
