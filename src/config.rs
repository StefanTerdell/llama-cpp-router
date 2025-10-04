use anyhow::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::canonicalize, path::Path};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum LlamaCppServerType {
    Chat,
    Embedding,
    UnpooledEmbedding,
    Reranking,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LlamaCppServerQuant {
    #[serde(rename = "Q4_K_M")]
    Q4KM,
    #[serde(rename = "Q6_K")]
    Q6K,
    #[serde(rename = "Q8_0")]
    Q80,
    #[serde(untagged)]
    Other(String),
}

impl Display for LlamaCppServerQuant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LlamaCppServerQuant::Q4KM => "Q4_K_M",
            LlamaCppServerQuant::Q6K => "Q6_K",
            LlamaCppServerQuant::Q80 => "Q8_0",
            LlamaCppServerQuant::Other(value) => value.as_str(),
        })
    }
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
