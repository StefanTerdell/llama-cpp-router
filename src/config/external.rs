use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;
use utils_rs::secret::Secret;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExternalProviderConfig {
    pub base_url: Url,
    pub api_key: Secret<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExternalModelConfig {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExternalProviderAndModelConfig {
    #[serde(flatten)]
    pub provider: ExternalProviderConfig,
    #[serde(flatten)]
    pub model: ExternalModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExternalProviderNameAndModelConfig {
    pub provider: String,
    #[serde(flatten)]
    pub model: ExternalModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ExternalConfig {
    ProviderAndModel(ExternalProviderAndModelConfig),
    ProviderNameAndModel(ExternalProviderNameAndModelConfig),
}

impl ExternalConfig {
    pub fn model(&self) -> &ExternalModelConfig {
        match self {
            ExternalConfig::ProviderAndModel(x) => &x.model,
            ExternalConfig::ProviderNameAndModel(x) => &x.model,
        }
    }

    pub fn unwrap_provider(&self) -> &ExternalProviderConfig {
        match self {
            ExternalConfig::ProviderAndModel(x) => &x.provider,
            ExternalConfig::ProviderNameAndModel(x) => {
                panic!("Expected resolved ProviderAndModel, found {x:#?}")
            }
        }
    }
}
