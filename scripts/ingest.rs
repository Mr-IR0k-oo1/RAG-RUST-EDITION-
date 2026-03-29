use anyhow::Result;
use log::info;

/// Ingestion entry point for processing books into the RAG system
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting ingestion pipeline");

    // Get path from args or use default
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("data/raw/books");

    println!("=== RAG Book Ingestion Pipeline ===");
    println!("Input directory: {}", path);

    // 1. Load configuration
    let app_config = rag_pdf_dataset::config::AppConfig::load("configs/app.toml")?;
    let embedding_config = rag_pdf_dataset::config::EmbeddingConfig::load("configs/embedding.toml")?;
    let qdrant_config = rag_pdf_dataset::config::QdrantConfig::load("configs/qdrant.toml")?;

    println!("Configuration loaded successfully");

    // 2. Ensure data directories exist
    rag_pdf_dataset::utils::FileUtils::ensure_dir(&app_config.paths.raw_data_dir)?;
    rag_pdf_dataset::utils::FileUtils::ensure_dir(&app_config.paths.processed_data_dir)?;
    rag_pdf_dataset::utils::FileUtils::ensure_dir(&app_config.paths.indexes_dir)?;

    // 3. Load books from directory
    println!("\n--- Loading Books ---");
    let books = rag_pdf_dataset::ingestion::BookLoader::load_directory(path).await?;
    
    if books.is_empty() {
        println!("No books found in {}", path);
        println!("Place PDF, EPUB, or scanned documents in the directory and try again.");
        return Ok(());
    }

    println!("Loaded {} books:", books.len());
    for book in &books {
        println!("  - {} ({} chars, {} chapters)", 
            book.title, 
            book.raw_text.len(),
            book.chapters.len()
        );
    }

    // 4. Clean and chunk text
    println!("\n--- Processing Text ---");
    let cleaner = rag_pdf_dataset::ingestion::TextCleaner::new();
    let chunker_config = rag_pdf_dataset::ingestion::ChunkerConfig {
        chunk_size: app_config.processing.chunk_size,
        overlap: app_config.processing.chunk_overlap,
        min_chunk_size: app_config.processing.min_chunk_size,
    };
    let chunker = rag_pdf_dataset::ingestion::TextChunker::new(chunker_config);

    let mut all_chunks = Vec::new();

    for book in &books {
        let cleaned = cleaner.full_clean(&book.raw_text);
        let chunks = chunker.chunk_with_metadata(&cleaned, &book.id);
        println!("  {}: {} chunks", book.title, chunks.len());
        all_chunks.extend(chunks);
    }

    println!("\nTotal chunks: {}", all_chunks.len());

    // 5. Write chunks to JSONL
    println!("\n--- Writing Processed Data ---");
    let jsonl_path = format!("{}/chunks.jsonl", app_config.paths.processed_data_dir);
    let mut writer = rag_pdf_dataset::dataset::DatasetWriter::with_path(&jsonl_path)?;
    writer.write_chunks(&all_chunks)?;
    writer.flush()?;
    println!("Wrote chunks to: {}", jsonl_path);

    // 6. Generate fine-tuning dataset
    println!("\n--- Generating Fine-Tuning Dataset ---");
    let dataset_config = rag_pdf_dataset::dataset::DatasetConfig::default();
    let generator = rag_pdf_dataset::dataset::DatasetGenerator::new(dataset_config);
    
    let dataset = generator.generate_from_chunks(&all_chunks)?;
    println!("Generated:");
    println!("  - {} instruction entries", dataset.instructions.len());
    println!("  - {} Q&A entries", dataset.qa_pairs.len());
    println!("  - {} chat entries", dataset.chat_entries.len());

    // 7. Write dataset splits
    let splits = generator.generate_splits(&dataset)?;
    rag_pdf_dataset::dataset::DatasetWriter::write_splits(&app_config.paths.processed_data_dir, &splits)?;
    println!("Dataset written to: {}", app_config.paths.processed_data_dir);

    // 8. Embed and index (optional - requires Qdrant running)
    println!("\n--- Embedding and Indexing ---");
    println!("Creating embeddings...");
    
    let embedding_client = rag_pdf_dataset::embedding::EmbeddingClient::new_local(
        &embedding_config, 
        &format!("{}/all-MiniLM-L6-v2", app_config.paths.indexes_dir)
    );

    // Embed first chunk as test
    if let Some(first_chunk) = all_chunks.first() {
        let embedding: Vec<f32> = embedding_client.embed(&first_chunk.content).await?;
        println!("Embedding dimension: {}", embedding.len());
        let preview: Vec<String> = embedding[..5.min(embedding.len())].iter().map(|v| format!("{:.4}", v)).collect();
        println!("Sample embedding (first 5 values): [{}]", preview.join(", "));
    }

    // 9. Store in Qdrant (if running)
    println!("\n--- Vector Store ---");
    let store = rag_pdf_dataset::vectorstore::QdrantStore::new(&qdrant_config);
    
    match store.ensure_collection().await {
        Ok(_) => {
            println!("Qdrant collection ready");
            
            // Index all chunks
            let batcher = rag_pdf_dataset::embedding::EmbeddingBatcher::new(embedding_client, &embedding_config);
            let chunk_texts: Vec<String> = all_chunks.iter().map(|c| c.content.clone()).collect();
            let embeddings: Vec<Vec<f32>> = batcher.embed_batched(&chunk_texts).await?;
            
            println!("Embedding {} chunks...", embeddings.len());
            
            // Insert into Qdrant
            let mut points: Vec<(u64, Vec<f32>, serde_json::Value)> = Vec::new();
            for (i, (chunk, embedding)) in all_chunks.iter().zip(embeddings.iter()).enumerate() {
                let payload = serde_json::json!({
                    "content": chunk.content,
                    "document_id": chunk.document_id,
                    "chunk_index": chunk.chunk_index,
                    "token_count": chunk.token_count,
                });
                points.push((i as u64, embedding.clone(), payload));
            }
            
            store.insert_batch(points).await?;
            let count = store.count().await?;
            println!("Indexed {} points in Qdrant", count);
        }
        Err(e) => {
            println!("Qdrant not available: {}", e);
            println!("Start Qdrant with: docker run -p 6333:6333 qdrant/qdrant");
        }
    }

    println!("\n=== Ingestion Complete ===");
    println!("Processed data: {}", app_config.paths.processed_data_dir);
    println!("Run 'cargo run --bin rebuild_index' to rebuild the vector index");

    info!("Ingestion complete");
    Ok(())
}
