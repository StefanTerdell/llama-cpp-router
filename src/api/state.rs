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
                .filter(|x| x.is_default.as_bool())
                .map(|x| x.alias());

            for alias in default_aliases {
                models.load(&config, alias).await?;
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
