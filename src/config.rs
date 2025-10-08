use anyhow::{Context, Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs::canonicalize, path::Path};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LlamaCppModelConfig {
    pub hf_repo: String,
    pub port: u16,
    pub alias: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(flatten)]
    pub additional_properties: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ExternalModelConfig {
    pub base_url: Url,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "config", rename_all = "kebab-case")]
pub enum ModelTypeConfig {
    LlamaCpp(LlamaCppModelConfig),
    External(ExternalModelConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unloads: Option<Vec<AliasOrIndex>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loads: Option<Vec<AliasOrIndex>>,
    #[serde(flatten)]
    pub config: ModelTypeConfig,
}

impl ModelConfig {
    pub fn alias(&self) -> &str {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => &x.alias,
            ModelTypeConfig::External(x) => x.alias.as_deref().unwrap_or(&x.name),
        }
    }

    pub fn name(&self) -> &str {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => &x.alias,
            ModelTypeConfig::External(x) => &x.name,
        }
    }

    pub fn api_key(&self) -> Option<&str> {
        match &self.config {
            ModelTypeConfig::LlamaCpp(x) => x.api_key.as_deref(),
            ModelTypeConfig::External(x) => Some(&x.api_key),
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
            ModelTypeConfig::External(x) => Ok(x.base_url.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub models: Vec<ModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AliasOrIndex {
    Alias(String),
    Index(usize),
}

impl From<&AliasOrIndex> for AliasOrIndex {
    fn from(value: &AliasOrIndex) -> Self {
        value.clone()
    }
}

impl AliasOrIndex {
    pub fn as_alias(&self) -> Option<&str> {
        match self {
            AliasOrIndex::Alias(alias) => Some(alias),
            AliasOrIndex::Index(_) => None,
        }
    }

    #[allow(unused)]
    pub fn as_index(&self) -> Option<usize> {
        match self {
            AliasOrIndex::Alias(_) => None,
            AliasOrIndex::Index(index) => Some(*index),
        }
    }
}

impl From<&str> for AliasOrIndex {
    fn from(str: &str) -> Self {
        str.to_string().into()
    }
}

impl From<&String> for AliasOrIndex {
    fn from(str: &String) -> Self {
        str.to_owned().into()
    }
}

impl From<String> for AliasOrIndex {
    fn from(value: String) -> Self {
        match value.parse() {
            Ok(index) => AliasOrIndex::Index(index),
            Err(_) => AliasOrIndex::Alias(value),
        }
    }
}

impl From<usize> for AliasOrIndex {
    fn from(index: usize) -> Self {
        AliasOrIndex::Index(index)
    }
}

// #[derive(Debug, thiserror::Error)]
// pub enum ConfigError {
//     #[error(transparent)]
//     Io(#[from] std::io::Error),
//     #[error(transparent)]
//     Json(#[from] serde_json::Error),
//     #[error("No model config found using {0:?}")]
//     ModelNotFound(AliasOrIndex),
//     #[error(
//         "Can't determine which model config to get as there are more than one model configs using the alias or name '{0}'. Use an index instead."
//     )]
//     Indeterminable(String),
// }

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        let path = canonicalize(path)?;
        let file_content =
            std::fs::read_to_string(&path).context("Failed to read config file content")?;
        let config = serde_json::from_str(&file_content)?;

        Ok(config)
    }

    pub fn get_model_config(
        &self,
        alias_or_index: impl Into<AliasOrIndex>,
    ) -> Result<&ModelConfig> {
        let alias_or_index = alias_or_index.into();

        let mut model_configs = self
            .models
            .iter()
            .enumerate()
            .filter(|(i, m)| match &alias_or_index {
                AliasOrIndex::Alias(alias) => m.alias() == alias,
                AliasOrIndex::Index(index) => i == index,
            })
            .collect::<Vec<_>>();

        if model_configs.is_empty() {
            bail!("Model with alias or index {alias_or_index:#?} not found");
            // return Err(ConfigError::ModelNotFound(alias_or_index));
        }

        let count = model_configs.len();

        if count != 1 {
            bail!(
                "{count} models with the same alias {} exists. Use an index instead.",
                alias_or_index.as_alias().expect("wtf")
            );
            // return Err(ConfigError::Indeterminable(
            //     alias_or_index
            //         .as_alias()
            //         .expect("what in the goddamn fuck")
            //         .to_string(),
            // ));
        }

        let (_, model_config) = model_configs.remove(0);

        Ok(model_config)
    }
}
