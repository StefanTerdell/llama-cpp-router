use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DetachConfig {
    pub working_dir: PathBuf,
    pub pid_file_path: PathBuf,
    pub out_file_path: PathBuf,
    pub err_file_path: PathBuf,
}

impl Default for DetachConfig {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::from("/tmp"),
            pid_file_path: PathBuf::from("/tmp/hrdr.pid"),
            out_file_path: PathBuf::from("/tmp/hrdr.out"),
            err_file_path: PathBuf::from("/tmp/hrdr.err"),
        }
    }
}
