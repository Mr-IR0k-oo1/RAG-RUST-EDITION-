use anyhow::Result;
use tokio::time::{sleep, Duration};
use crate::embedding::EmbeddingClient;
use crate::config::EmbeddingConfig;

/// Rate-limit safe batching for embedding operations
pub struct EmbeddingBatcher {
    client: EmbeddingClient,
    batch_size: usize,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl EmbeddingBatcher {
    pub fn new(client: EmbeddingClient, config: &EmbeddingConfig) -> Self {
        Self {
            client,
            batch_size: config.batch.size,
            max_retries: config.batch.max_retries,
            retry_delay_ms: config.batch.retry_delay_ms,
        }
    }

    /// Embed texts in batches with retry logic
    pub async fn embed_batched(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings = Vec::new();

        for chunk in texts.chunks(self.batch_size) {
            let refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
            let embeddings = self.embed_with_retry(&refs).await?;
            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }

    async fn embed_with_retry(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut attempt = 0;

        loop {
            match self.client.embed_batch(texts).await {
                Ok(embeddings) => return Ok(embeddings),
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.max_retries {
                        return Err(e);
                    }
                    sleep(Duration::from_millis(self.retry_delay_ms)).await;
                }
            }
        }
    }
}
