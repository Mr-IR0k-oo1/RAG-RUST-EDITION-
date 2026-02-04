use anyhow::Result;
use crate::config::QdrantConfig;

/// Search infrastructure only.
/// No PDF logic. Zero.
/// Future-proof: swap Qdrant → Tantivy → FAISS
pub struct QdrantStore {
    url: String,
    collection_name: String,
    vector_size: usize,
}

impl QdrantStore {
    pub fn new(config: &QdrantConfig) -> Self {
        Self {
            url: config.qdrant.url.clone(),
            collection_name: config.collection.name.clone(),
            vector_size: config.collection.vector_size,
        }
    }

    /// Insert a vector with metadata
    pub async fn insert(&self, id: u64, vector: Vec<f32>, payload: serde_json::Value) -> Result<()> {
        // TODO: Insert into Qdrant
        // - id: unique identifier
        // - vector: embedding vector
        // - payload: metadata (chunk_id, document_id, etc.)
        Ok(())
    }

    /// Search for similar vectors
    pub async fn search(&self, vector: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>> {
        // TODO: Query Qdrant for top-k similar vectors
        Ok(Vec::new())
    }

    /// Delete vectors by ID
    pub async fn delete(&self, id: u64) -> Result<()> {
        // TODO: Delete from Qdrant
        Ok(())
    }

    /// Recreate collection
    pub async fn recreate_collection(&self) -> Result<()> {
        // TODO: Drop and recreate collection with correct schema
        Ok(())
    }
}

/// Search result from vector store
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: serde_json::Value,
}
