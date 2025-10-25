use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::{
    alias_or_index::AliasOrIndex, external::ExternalConfig, llama_cpp::LlamaCppModelConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type", content = "config")]
pub enum ModelTypeConfig {
    LlamaCpp(LlamaCppModelConfig),
    External(ExternalConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unloads: Option<Vec<AliasOrIndex>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loads: Option<Vec<AliasOrIndex>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "default")]
    pub is_default: Option<bool>,
    #[serde(flatten)]
    pub config: ModelTypeConfig,
}

impl ModelConfig {
    pub fn alias(&self) -> &str {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => &x.alias,
            ModelTypeConfig::External(x) => {
                let x = &x.unwrap().model;
                x.alias.as_deref().unwrap_or(&x.id)
            }
        }
    }

    pub fn id(&self) -> &str {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => &x.alias,
            ModelTypeConfig::External(x) => &x.unwrap().model.id,
        }
    }

    pub fn api_key(&self) -> Option<&str> {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => x.api_key.as_deref(),
            ModelTypeConfig::External(x) => Some(&x.unwrap().provider.api_key),
        }
    }

    pub fn url(&self) -> Result<Url, url::ParseError> {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => format!(
                "http://{host}:{port}",
                host = x.host.as_deref().unwrap_or("localhost"),
                port = x.port
            )
            .parse(),
            ModelTypeConfig::External(x) => Ok(x.unwrap().provider.base_url.clone()),
        }
    }
}
