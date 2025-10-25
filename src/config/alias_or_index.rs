use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
