mod partial_reader;

mod download_partial_gguf;

mod estimate_vram_usage_gb {
    use gguf::GGUFFile;
    use hf_hub::api::tokio::Api;
    use reqwest::Client;
    use std::{fs::File, io::Write, path::PathBuf};

    use super::partial_reader

    /// Estimate VRAM usage from metadata only (same logic as before)
    fn estimate_vram_usage_gb(gguf_path: &PathBuf, context_len: usize) -> f64 {
        let gguf: GGUFFile = todo!(); //GGUFFile::open(gguf_path).expect("Failed to parse GGUF header");
        let embed_len = gguf
            .get("llama.embedding_length")
            .and_then(|v| v.to_u64())
            .unwrap_or(4096);
        let layers = gguf
            .get("llama.block_count")
            .and_then(|v| v.to_u64())
            .unwrap_or(32);
        let arch = gguf
            .get("general.architecture")
            .and_then(|v| v.to_string())
            .unwrap_or_default();

        let params_constant = match arch.as_str() {
            "mistral" => 12.0,
            "llama" => 12.0,
            "falcon" => 10.0,
            _ => 11.0,
        };

        let params = params_constant * (embed_len as f64).powi(2) * layers as f64;

        let quant = gguf
            .get("general.quantization_version")
            .and_then(|v| v.to_string())
            .unwrap_or_else(|| "Q4_K_M".to_string());

        let bytes_per_param = match quant.as_str() {
            q if q.contains("Q2") => 0.25,
            q if q.contains("Q3") => 0.375,
            q if q.contains("Q4") => 0.5,
            q if q.contains("Q5") => 0.625,
            q if q.contains("Q6") => 0.75,
            q if q.contains("Q8") => 1.0,
            q if q.contains("F16") => 2.0,
            q if q.contains("F32") => 4.0,
            _ => 0.5,
        };

        let context_overhead = 0.00005 * context_len as f64; // ~0.2 GB @ 4k
        ((params * bytes_per_param) / 1_073_741_824.0 / 1_024.0) + context_overhead
    }
}
