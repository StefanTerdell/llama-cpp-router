mod alias_or_index;
mod llama_cpp;
mod external;
mod model;

pub use alias_or_index::*;
pub use llama_cpp::*;
pub use model::*;
pub use external::*;

use anyhow::{Context, Result, anyhow, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utils_rs::option::as_bool::AsBool;
use std::{collections::{HashMap, HashSet}, fs::canonicalize, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub models: Vec<ModelConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, ExternalProviderConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_defaults_on_launch : Option<bool>
}

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        let path = canonicalize(path)?;
        let file_content =
            std::fs::read_to_string(&path).context("Failed to read config file content")?;
        let Config { schema, models, providers, load_defaults_on_launch }: Config = serde_json::from_str(&file_content)?;

        let mut defaults = HashSet::new();

        Ok(Config {
            schema,
            load_defaults_on_launch,
            models: models
                .into_iter()
                .map(|model_config| {
                    if model_config.is_default.as_bool() && !defaults.insert(model_config.alias().to_string()) {
                        bail!("Multiple models with alias '{}' marked as default", model_config.alias())
                    }
                    
                    anyhow::Ok(ModelConfig {
                        config: match model_config.config {
                            ModelTypeConfig::External(ExternalConfig::ProviderNameAndModel(
                                ExternalProviderNameAndModelConfig { provider, model },
                            )) => {
                                  ModelTypeConfig::External(ExternalConfig::ProviderAndModel(
                                    ExternalProviderAndModelConfig {
                                        provider: providers
                                            .as_ref()
                                            .and_then(|providers| providers.get(&provider).cloned())
                                            .ok_or(anyhow!("External model config {model:?} references missing provider '{provider}'"))?,
                                        model,
                                    },
                                ))          
                            },
                            other => other,
                        },
                        ..model_config
                    })
                })
               .collect::<Result<Vec<_>, _>>()?,
            providers
        })
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
        }

        let count = model_configs.len();

        if count != 1 {
            bail!(
                "{count} models with the same alias '{}' exists. Use an index instead.",
                alias_or_index.as_alias().expect("wtf")
            );
        }

        let (_, model_config) = model_configs.remove(0);

        Ok(model_config)
    }
}
