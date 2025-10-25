use anyhow::{Context, anyhow};
use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::api::{result::ApiResult, state::ApiState};

pub fn router() -> Router<ApiState> {
    Router::new().route("/models", get(list_v1_models))
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
async fn list_v1_models(State(state): State<ApiState>) -> ApiResult<Json<V1ModelsResponse>> {
    let mut result = V1ModelsResponse::default();

    for model_config in state.models().get_loaded_configs().await {
        let url = model_config
            .url()
            .and_then(|url| url.join("/v1/models"))
            .context("Failed constructing Url")?;

        let mut request = reqwest::Client::new().get(url);

        if let Some(key) = model_config.api_key() {
            request = request.bearer_auth(key)
        }

        result.data.push(
            request
                .send()
                .await
                .context("Failed sending request")?
                .error_for_status()
                .context("Error returned from provider")?
                .json::<V1ModelsResponse>()
                .await
                .context("Failed parsing response")?
                .data
                .into_iter()
                .find({
                    let id = model_config.id();
                    move |model| model.id == id
                })
                .ok_or(anyhow!(
                    "No model with id '{}' returned from provider of {model_config:#?}",
                    model_config.id()
                ))?,
        );
    }

    Ok(Json(result))
}
