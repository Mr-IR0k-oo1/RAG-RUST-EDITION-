use anyhow::Result;
use log::info;

/// Explicit execution entry point for ingestion.
/// Build dataset + index
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting ingestion pipeline");

    // TODO: Implement ingestion workflow:
    // 1. Load config from configs/
    // 2. Walk through data/raw/pdfs/
    // 3. For each PDF:
    //    - Extract text (pdf_loader)
    //    - Clean text (cleaner)
    //    - Chunk into segments (chunker)
    //    - Write to data/processed/jsonl/ (writer)
    //    - Embed chunks (embedding_batcher)
    //    - Insert into vectorstore (qdrant)

    info!("Ingestion complete");
    Ok(())
}
