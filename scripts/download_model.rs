use anyhow::Result;
use log::info;

/// Download embedding model for local inference
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting model download");

    println!("=== Download Embedding Model ===");

    let model_dir = "data/models/all-MiniLM-L6-v2";
    
    // Create model directory
    std::fs::create_dir_all(model_dir)?;
    println!("Model directory: {}", model_dir);

    // Model files to download
    let base_url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main";
    let files = [
        "config.json",
        "tokenizer.json",
        "tokenizer_config.json",
        "special_tokens_map.json",
        "modules.json",
        "sentence_bert_config.json",
    ];

    println!("\nDownloading model files...");
    
    let client = reqwest::Client::new();

    for file in &files {
        let url = format!("{}/{}", base_url, file);
        let output_path = format!("{}/{}", model_dir, file);
        
        if std::path::Path::new(&output_path).exists() {
            println!("  {} (already exists)", file);
            continue;
        }

        print!("  {} ... ", file);
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let bytes = response.bytes().await?;
                    std::fs::write(&output_path, &bytes)?;
                    println!("OK ({} bytes)", bytes.len());
                } else {
                    println!("Failed ({})", response.status());
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // Download model weights (safetensors format)
    let model_file = "model.safetensors";
    let model_url = format!("{}/{}", base_url, model_file);
    let model_path = format!("{}/{}", model_dir, model_file);

    if !std::path::Path::new(&model_path).exists() {
        print!("\nDownloading model weights (this may take a while)...\n  {} ... ", model_file);
        
        match client.get(&model_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let bytes = response.bytes().await?;
                    std::fs::write(&model_path, &bytes)?;
                    println!("OK ({} bytes)", bytes.len());
                } else {
                    println!("Failed ({})", response.status());
                    println!("\nNote: Model weights may need to be downloaded manually from:");
                    println!("  {}", model_url);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    } else {
        println!("\n  {} (already exists)", model_file);
    }

    println!("\n=== Download Complete ===");
    println!("Model files are in: {}", model_dir);
    println!("\nYou can now run:");
    println!("  cargo run --bin ingest -- data/raw/books/");

    info!("Model download complete");
    Ok(())
}
