use hf_hub::api::tokio::Api;
use reqwest::Client;
use std::{fs::File, io::Write, path::PathBuf};

/// Download only the first N bytes of a file from Hugging Face
pub async fn download_partial_gguf(repo_id: &str, filename: &str, bytes: usize) -> PathBuf {
    let api = Api::new().unwrap();
    let file_url = api.model(repo_id.to_string()).url(filename);

    let client = Client::new();
    let response = client
        .get(file_url)
        .header("Range", format!("bytes=0-{bytes}"))
        .send()
        .await
        .expect("Failed to request partial content");

    if !response.status().is_success() && response.status() != 206 {
        panic!("Expected partial content (206), got {}", response.status());
    }

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("{filename}_partial.gguf"));
    let mut file = File::create(&temp_path).unwrap();
    let content = response.bytes().await.unwrap();
    file.write_all(&content).unwrap();

    temp_path
}
