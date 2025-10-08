pub mod result;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Sse},
    routing::{get, post},
};
use reqwest::Method;
use reqwest_sse::EventSource;
use result::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::{
    config::{AliasOrIndex, Config, ModelConfig},
    models::{LogsAndTailReceiver, Models},
    reqwest_ext::{ReqwestRequestBuildExt, ReqwestResponseExt},
};

#[derive(Clone)]
struct ApiState {
    config_path: PathBuf,
    models: Models,
}

pub async fn serve(port: u16, config_path: PathBuf) -> Result<()> {
    let state = ApiState {
        config_path,
        models: Default::default(),
    };

    let api = Router::new()
        .route("/hurrdurr/models", get(list_hurr_durr_models_handler))
        .route(
            "/hurrdurr/models/{alias_or_index}",
            get(get_hurr_durr_models_alias_or_index_handler)
                .post(post_hurr_durr_models_alias_or_index_handler)
                .delete(delete_hurr_durr_models_alias_or_index_handler),
        )
        .route(
            "/hurrdurr/models/{alias_or_index}/logs",
            get(get_hurr_durr_models_alias_or_index_logs_handler),
        )
        .route("/v1/models", get(list_v1_models_handler))
        .route("/{*path}", post(post_handler))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    println!("hurrdurr listening on {port}");

    axum::serve(listener, api).await?;

    Ok(())
}

#[axum::debug_handler]
async fn list_hurr_durr_models_handler(State(state): State<ApiState>) -> Json<Vec<ModelConfig>> {
    Json(state.models.get_loaded_configs().await)
}

#[derive(Debug, Deserialize)]
struct AliasOrIndexPath {
    alias_or_index: String,
}

#[axum::debug_handler]
async fn get_hurr_durr_models_alias_or_index_handler(
    State(state): State<ApiState>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult<Json<Option<ModelConfig>>> {
    let alias = match AliasOrIndex::from(alias_or_index) {
        AliasOrIndex::Alias(alias) => alias,
        AliasOrIndex::Index(index) => {
            let config = Config::load(&state.config_path)?;

            config.get_model_config(index)?.alias().to_string()
        }
    };

    Ok(Json(state.models.unload(&alias).await?))
}

#[derive(Debug, Deserialize)]
struct TailAndJsonQuery {
    tail: Option<bool>,
    json: Option<bool>,
}

#[axum::debug_handler]
async fn post_hurr_durr_models_alias_or_index_handler(
    State(state): State<ApiState>,
    Query(TailAndJsonQuery { tail, json }): Query<TailAndJsonQuery>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult {
    let config = Config::load(&state.config_path)?;

    let (model_config, logs_and_tail_receiver) =
        state.models.load(&config, &alias_or_index).await?;

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
async fn delete_hurr_durr_models_alias_or_index_handler(
    State(state): State<ApiState>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult<Json<Option<ModelConfig>>> {
    let alias = match AliasOrIndex::from(alias_or_index) {
        AliasOrIndex::Alias(alias) => alias,
        AliasOrIndex::Index(index) => {
            let config = Config::load(&state.config_path)?;

            config.get_model_config(index)?.alias().to_string()
        }
    };

    Ok(Json(state.models.unload(&alias).await?))
}

#[axum::debug_handler]
async fn get_hurr_durr_models_alias_or_index_logs_handler(
    State(state): State<ApiState>,
    Query(TailAndJsonQuery { tail, json }): Query<TailAndJsonQuery>,
    Path(AliasOrIndexPath { alias_or_index }): Path<AliasOrIndexPath>,
) -> ApiResult {
    let alias = match AliasOrIndex::from(alias_or_index) {
        AliasOrIndex::Alias(alias) => alias,
        AliasOrIndex::Index(index) => {
            let config = Config::load(&state.config_path)?;

            config.get_model_config(index)?.alias().to_string()
        }
    };

    let LogsAndTailReceiver {
        logs,
        tail_receiver,
    } = state
        .models
        .get_logs_and_receiver(&alias)
        .await
        .ok_or(ApiError::Notfound(alias))?;

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

#[derive(Serialize, Deserialize)]
struct V1ModelsResponseItem {
    id: String,
    #[serde(flatten)]
    additional_properties: Map<String, Value>,
}

#[derive(Default, Serialize, Deserialize)]
struct V1ModelsResponse {
    data: Vec<V1ModelsResponseItem>,
    #[serde(flatten)]
    additional_properties: Map<String, Value>,
}

#[axum::debug_handler]
async fn list_v1_models_handler(
    State(state): State<ApiState>,
) -> ApiResult<Json<V1ModelsResponse>> {
    let mut result = V1ModelsResponse::default();

    for model_config in state.models.get_loaded_configs().await {
        let url = model_config.url().context("Failed constructing Url")?;

        result.data.push(
            reqwest::get(url.clone())
                .await
                .context("Failed sending request")?
                .map_http_error_response(Method::GET, url)
                .await?
                .json::<V1ModelsResponse>()
                .await
                .context("Failed parsing response")?
                .data
                .into_iter()
                .find({
                    let name = model_config.name();
                    move |model| model.id == name
                })
                .context("No matching model found")?,
        );
    }

    Ok(Json(result))
}

#[derive(Debug, Serialize, Deserialize)]
struct PostRequest {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(flatten)]
    additional_properties: Map<String, Value>,
}

#[axum::debug_handler]
async fn post_handler(
    State(state): State<ApiState>,
    Path(path): Path<String>,
    Json(mut body): Json<PostRequest>,
) -> ApiResult {
    let Some(model_config) = state.models.get_loaded_config(&body.model).await else {
        return Err(ApiError::Notfound(body.model));
    };

    body.model = model_config.name().into();

    let url = model_config
        .url()
        .and_then(|url| url.join(&path))
        .context("Failed constructing the model url")?
        .to_string();

    if body.stream.unwrap_or(false) {
        let mut es = reqwest::Client::new()
            .post(&url)
            .optional_bearer_auth(model_config.api_key())
            .json(&body)
            .send()
            .await
            .context("Failed sending request")?
            .events()
            .await
            .map_err(ApiError::ReqwestSse)?;

        while let Some(event) = es.next().await {
            println!("{event:#?}")
        }

        // let sse = Sse::new(es.map(|result| match result {
        //     Ok(event) => {
        //         let mut sse_event = axum::response::sse::Event::default()
        //             .data(event.data)
        //             .event(event.event_type);

        //         if let Some(id) = event.last_event_id {
        //             sse_event = sse_event.id(id);
        //         }

        //         if let Some(duration) = event.retry {
        //             sse_event = sse_event.retry(duration);
        //         }

        //         Ok(sse_event)
        //     }
        //     Err(err) => Err(ApiError::ReqwestSseEvent(err)),
        // }));

        // Ok(sse.into_response())

        Err(ApiError::Notfound("derp".into()))
    } else {
        Ok(Json(
            reqwest::Client::new()
                .post(&url)
                .optional_bearer_auth(model_config.api_key())
                .json(&body)
                .send()
                .await
                .context("Failed sending request")?
                .map_http_error_response(Method::POST, url)
                .await?
                .json::<Value>()
                .await
                .context("Failed parsing response")?,
        )
        .into_response())
    }
}
