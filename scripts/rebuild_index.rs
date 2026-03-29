use anyhow::Result;
use log::info;

/// Index rebuilding entry point
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting index rebuild");

    println!("=== RAG Index Rebuild ===");

    // 1. Load configuration
    let app_config = rag_pdf_dataset::config::AppConfig::load("configs/app.toml")?;
    let embedding_config = rag_pdf_dataset::config::EmbeddingConfig::load("configs/embedding.toml")?;
    let qdrant_config = rag_pdf_dataset::config::QdrantConfig::load("configs/qdrant.toml")?;

    println!("Configuration loaded");

    // 2. Connect to Qdrant
    println!("\n--- Connecting to Qdrant ---");
    let store = rag_pdf_dataset::vectorstore::QdrantStore::new(&qdrant_config);
    
    // 3. Recreate collection
    println!("Recreating collection...");
    store.recreate_collection().await?;
    println!("Collection ready");

    // 4. Read processed chunks
    println!("\n--- Loading Chunks ---");
    let chunks_path = format!("{}/chunks.jsonl", app_config.paths.processed_data_dir);
    
    if !std::path::Path::new(&chunks_path).exists() {
        println!("No chunks found at: {}", chunks_path);
        println!("Run 'cargo run --bin ingest' first to process books.");
        return Ok(());
    }

    // Read chunks from JSONL
    let content = std::fs::read_to_string(&chunks_path)?;
    let chunk_lines: Vec<&str> = content.lines().collect();
    println!("Loaded {} chunks", chunk_lines.len());

    // 5. Create embeddings
    println!("\n--- Creating Embeddings ---");
    let embedding_client = rag_pdf_dataset::embedding::EmbeddingClient::new_local(
        &embedding_config,
        &format!("{}/all-MiniLM-L6-v2", app_config.paths.indexes_dir)
    );
    
    let batcher = rag_pdf_dataset::embedding::EmbeddingBatcher::new(embedding_client, &embedding_config);
    
    // Extract content from chunks
    let mut chunk_contents: Vec<String> = Vec::new();
    let mut chunk_payloads: Vec<serde_json::Value> = Vec::new();
    
    for line in chunk_lines.iter() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                chunk_contents.push(content.to_string());
                chunk_payloads.push(json.clone());
            }
        }
    }

    println!("Embedding {} chunks...", chunk_contents.len());
    let embeddings: Vec<Vec<f32>> = batcher.embed_batched(&chunk_contents).await?;
    println!("Created {} embeddings", embeddings.len());

    // 6. Insert into Qdrant
    println!("\n--- Indexing in Qdrant ---");
    let mut points: Vec<(u64, Vec<f32>, serde_json::Value)> = Vec::new();
    
    for (i, (embedding, payload)) in embeddings.iter().zip(chunk_payloads.iter()).enumerate() {
        points.push((i as u64, embedding.clone(), payload.clone()));
    }

    store.insert_batch(points).await?;
    
    let count = store.count().await?;
    println!("Indexed {} points in Qdrant", count);

    println!("\n=== Index Rebuild Complete ===");
    println!("Collection: {}", qdrant_config.collection.name);
    println!("Total points: {}", count);

    info!("Index rebuild complete");
    Ok(())
}
