use anyhow::Result;
use std::path::{Path, PathBuf};
use utils_rs::option::as_bool::AsBool;

use crate::{config::Config, models::Models};

#[derive(Clone)]
pub struct ApiState {
    config_path: PathBuf,
    models: Models,
}

impl ApiState {
    pub async fn init(config_path: PathBuf) -> Result<Self> {
        let models = Models::default();
        let config = Config::load(&config_path)?;

        if config.load_defaults_on_launch.as_bool() {
            let default_aliases = config
                .models
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_default.as_bool());

            for (index, model_config) in default_aliases {
                println!(
                    "Loading default model for alias '{}' (config.models[{index}]: {})",
                    model_config.alias(),
                    serde_json::to_string(&model_config.config).unwrap()
                );
                models.load(&config, index).await?;
            }
        }

        Ok(Self {
            config_path,
            models,
        })
    }

    pub fn models(&self) -> &Models {
        &self.models
    }
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}
