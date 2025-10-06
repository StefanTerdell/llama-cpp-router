use anyhow::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fs::canonicalize, path::Path};

use crate::models::Quantization;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum LlamaCppServerType {
    Chat,
    Embedding,
    UnpooledEmbedding,
    Reranking,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum LlamaCppServerQuant {
    Known(Quantization),
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaCppServer {
    pub server_type: LlamaCppServerType,
    pub repo: String,
    pub quant: LlamaCppServerQuant,
    pub context_size: Option<usize>,
    pub parallel: Option<u8>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub servers: Vec<LlamaCppServer>,
    pub port: u16,
}

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        let path = canonicalize(path)?;
        let file_content = std::fs::read_to_string(&path)?;
        let config = serde_json::from_str(&file_content)?;

        println!("Config loaded from {path:?}: {config:#?}");

        Ok(config)
    }
}
