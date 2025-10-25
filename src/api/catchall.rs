use anyhow::Context;
use axum::{
    body::Body,
    extract::{Request, State},
    http::Response,
    routing::{MethodRouter, any},
};
use http::{HeaderMap, Method, Uri, Version, header::CONTENT_TYPE, request::Parts};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::mem::take;

use crate::{
    api::{
        result::{ApiError, ApiResult},
        state::ApiState,
    },
    config::ModelConfig,
};

pub fn handler() -> MethodRouter<ApiState> {
    any(route_request)
}

#[axum::debug_handler]
async fn route_request(State(state): State<ApiState>, request: Request) -> ApiResult {
    if request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|x| x.to_str().ok())
        .is_some_and(|x| x.to_lowercase().contains("json"))
    {
        route_request_by_json_model_field_or_model_header(state, request).await
    } else {
        route_request_by_model_header(state, request).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonObjectBody {
    model: Option<String>,
    #[serde(flatten)]
    additional_properties: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonBody {
    Object(JsonObjectBody),
    Other(Value),
}

async fn route_request_by_json_model_field_or_model_header(
    state: ApiState,
    request: Request,
) -> ApiResult {
    let (
        Parts {
            method,
            uri,
            headers,
            version,
            ..
        },
        body,
    ) = request.into_parts();

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|err| ApiError::BadRequest(format!("Unable to collect body bytes: {err:?}")))?
        .to_vec();

    let json_body: JsonBody = serde_json::from_slice(&body_bytes)
        .map_err(|err| ApiError::BadRequest(format!("Unable to deserialize json body: {err:?}")))?;

    let alias_from_json_body = match &json_body {
        JsonBody::Object(object) => object.model.as_deref(),
        JsonBody::Other(_) => None,
    };

    let alias = alias_from_json_body
        .or_else(|| headers.get("model").and_then(|v| v.to_str().ok()))
        .ok_or(ApiError::BadRequest(
            "No model specified in body nor headers".into(),
        ))?;

    let model_config = state
        .models()
        .get_loaded_config(alias)
        .await
        .ok_or(ApiError::NotFound(format!("Model alias {alias} not found")))?;

    let json_body = if let JsonBody::Object(mut object) = json_body {
        object.model = Some(model_config.id().to_string());

        JsonBody::Object(object)
    } else {
        json_body
    };

    let body = reqwest::Body::wrap(serde_json::to_string(&json_body).unwrap());

    route_request_parts_and_body_by_model_config(model_config, method, uri, headers, version, body)
        .await
}

async fn route_request_by_model_header(state: ApiState, request: Request) -> ApiResult {
    let alias = request
        .headers()
        .get("model")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::BadRequest("No model specified in headers".into()))?;

    let model_config = state
        .models()
        .get_loaded_config(alias)
        .await
        .ok_or(ApiError::NotFound(format!("Model alias {alias} not found")))?;

    let (
        Parts {
            method,
            uri,
            headers,
            version,
            ..
        },
        body,
    ) = request.into_parts();

    let body = reqwest::Body::wrap_stream(body.into_data_stream());

    route_request_parts_and_body_by_model_config(model_config, method, uri, headers, version, body)
        .await
}

async fn route_request_parts_and_body_by_model_config(
    model_config: ModelConfig,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    version: Version,
    body: reqwest::Body,
) -> ApiResult {
    let path_and_query = uri
        .path_and_query()
        .context("Failed to extract path and query")?
        .as_str();

    let url = model_config
        .url()
        .and_then(|url| url.join(path_and_query))
        .context("Failed constructing the model url")?;

    let mut request = reqwest::Request::new(method, url);

    *request.headers_mut() = headers;
    *request.version_mut() = version;
    *request.body_mut() = Some(body);

    if let Some(api_key) = model_config.api_key() {
        request.headers_mut().insert(
            AUTHORIZATION,
            format!("Bearer {api_key}")
                .parse()
                .context("Failed constructing bearer header value")?,
        );
    };

    let mut response = reqwest::Client::new()
        .execute(request)
        .await
        .context("Error passing on request")?;

    let headers = take(response.headers_mut());
    let mut response = Response::new(Body::from_stream(response.bytes_stream()));

    *response.headers_mut() = headers;

    Ok(response)
}
