use anyhow::Result;
use log::info;

/// Explicit execution entry point for index rebuilding.
/// Wipe & reinsert
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting index rebuild");

    // TODO: Implement rebuild workflow:
    // 1. Load config from configs/
    // 2. Connect to vectorstore
    // 3. Drop existing collection (if exists)
    // 4. Recreate collection with schema
    // 5. Read from data/processed/jsonl/
    // 6. For each chunk:
    //    - Embed the content
    //    - Insert into vectorstore with metadata
    //    - Log progress

    info!("Index rebuild complete");
    Ok(())
}
