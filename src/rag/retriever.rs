use anyhow::Result;
use crate::embedding::EmbeddingClient;
use crate::vectorstore::{QdrantStore, SearchResult};

/// Query-time only.
/// No ingestion logic here.
pub struct Retriever {
    embedding_client: EmbeddingClient,
    vectorstore: QdrantStore,
}

impl Retriever {
    pub fn new(embedding_client: EmbeddingClient, vectorstore: QdrantStore) -> Self {
        Self {
            embedding_client,
            vectorstore,
        }
    }

    /// Retrieve top-K relevant chunks for a query
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RetrievedChunk>> {
        // Embed the query
        let query_vector = self.embedding_client.embed(query).await?;

        // Search in vector store
        let results = self.vectorstore.search(query_vector, top_k).await?;

        // Convert to RetrievedChunk
        let chunks = results
            .into_iter()
            .map(|r| RetrievedChunk {
                score: r.score,
                payload: r.payload,
            })
            .collect();

        Ok(chunks)
    }
}

/// Retrieved chunk with relevance score
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub score: f32,
    pub payload: serde_json::Value,
}
