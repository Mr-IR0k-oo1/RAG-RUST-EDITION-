use anyhow::Result;

use crate::embedding::EmbeddingClient;
use crate::vectorstore::QdrantStore;

/// Query-time retriever for RAG
pub struct Retriever {
    embedding_client: EmbeddingClient,
    vectorstore: QdrantStore,
}

/// Retrieved chunk with relevance score
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub score: f32,
    pub payload: serde_json::Value,
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

        // Retrieve more candidates for re-ranking (4x)
        let candidates = self.vectorstore.search(query_vector, top_k * 4).await?;

        // Convert to RetrievedChunk
        let mut chunks: Vec<RetrievedChunk> = candidates
            .into_iter()
            .map(|r| RetrievedChunk {
                score: r.score,
                payload: r.payload,
            })
            .collect();

        // Re-rank using simple score-based sorting
        // In production, use a cross-encoder re-ranker
        chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Return top-K
        chunks.truncate(top_k);

        Ok(chunks)
    }

    /// Retrieve with minimum score threshold
    pub async fn retrieve_with_scores(
        &self,
        query: &str,
        top_k: usize,
        min_score: f32,
    ) -> Result<Vec<RetrievedChunk>> {
        let chunks = self.retrieve(query, top_k * 2).await?;
        
        // Filter by minimum score
        let filtered: Vec<RetrievedChunk> = chunks
            .into_iter()
            .filter(|c| c.score >= min_score)
            .take(top_k)
            .collect();

        Ok(filtered)
    }

    /// Retrieve with metadata filter
    pub async fn retrieve_with_filter(
        &self,
        query: &str,
        top_k: usize,
        filter: serde_json::Value,
    ) -> Result<Vec<RetrievedChunk>> {
        let query_vector = self.embedding_client.embed(query).await?;

        let candidates = self.vectorstore
            .search_with_filter(query_vector, top_k, filter)
            .await?;

        let chunks: Vec<RetrievedChunk> = candidates
            .into_iter()
            .map(|r| RetrievedChunk {
                score: r.score,
                payload: r.payload,
            })
            .collect();

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieved_chunk_creation() {
        let chunk = RetrievedChunk {
            score: 0.95,
            payload: serde_json::json!({
                "content": "Machine learning is a subset of AI."
            }),
        };
        assert!((chunk.score - 0.95).abs() < 0.01);
        assert!(chunk.payload.get("content").is_some());
    }
}
