mod config;
mod ingestion;
mod dataset;
mod embedding;
mod vectorstore;
mod rag;
mod utils;
mod types;

use anyhow::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting RAG PDF Dataset application");

    Ok(())
}
