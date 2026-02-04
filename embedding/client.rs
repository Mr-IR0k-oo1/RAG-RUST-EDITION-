use anyhow::Result;
use crate::config::EmbeddingConfig;

/// Stateless embedding API client
/// If this breaks, ingestion still works.
pub struct EmbeddingClient {
    endpoint: String,
    model: String,
    dimension: usize,
    timeout_secs: u64,
}

impl EmbeddingClient {
    pub fn new(config: &EmbeddingConfig) -> Self {
        Self {
            endpoint: config.embedding.endpoint.clone(),
            model: config.embedding.model.clone(),
            dimension: config.embedding.dimension,
            timeout_secs: config.embedding.timeout_secs,
        }
    }

    /// Embed a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // TODO: Call embedding API (HuggingFace, local inference, etc.)
        // Returns vector of dimension size
        Ok(vec![0.0; self.dimension])
    }

    /// Embed multiple texts (not batched, see EmbeddingBatcher)
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    pub fn get_model(&self) -> &str {
        &self.model
    }
}
