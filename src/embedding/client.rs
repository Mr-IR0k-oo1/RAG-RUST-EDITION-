use anyhow::Result;
use log::warn;

use crate::config::EmbeddingConfig;

/// Stateless embedding API client
/// Supports local inference via sentence-transformers models
pub struct EmbeddingClient {
    endpoint: String,
    model: String,
    dimension: usize,
    timeout_secs: u64,
    use_local: bool,
    model_path: Option<String>,
}

impl EmbeddingClient {
    /// Create a new embedding client from config
    pub fn new(config: &EmbeddingConfig) -> Self {
        Self {
            endpoint: config.embedding.endpoint.clone(),
            model: config.embedding.model.clone(),
            dimension: config.embedding.dimension,
            timeout_secs: config.embedding.timeout_secs,
            use_local: false,
            model_path: None,
        }
    }

    /// Create a new client for local inference
    pub fn new_local(config: &EmbeddingConfig, model_path: &str) -> Self {
        Self {
            endpoint: config.embedding.endpoint.clone(),
            model: config.embedding.model.clone(),
            dimension: config.embedding.dimension,
            timeout_secs: config.embedding.timeout_secs,
            use_local: true,
            model_path: Some(model_path.to_string()),
        }
    }

    /// Embed a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.use_local {
            return self.embed_local(text).await;
        }
        self.embed_api(text).await
    }

    /// Embed multiple texts
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if self.use_local {
            return self.embed_batch_local(texts).await;
        }
        
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed_api(text).await?);
        }
        Ok(embeddings)
    }

    /// Embed via API endpoint
    async fn embed_api(&self, text: &str) -> Result<Vec<f32>> {
        let client = reqwest::Client::new();
        
        let url = format!("{}/embeddings", self.endpoint);
        
        let body = serde_json::json!({
            "model": self.model,
            "input": text
        });

        let response = client
            .post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Embedding API error: {}", error));
        }

        let result: serde_json::Value = response.json().await?;
        
        let embedding = result["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding response"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    /// Embed locally using simple TF-IDF-like approach
    /// This is a simplified local embedding for demonstration
    /// In production, use candle + sentence-transformers
    async fn embed_local(&self, text: &str) -> Result<Vec<f32>> {
        // Simple hash-based embedding for demonstration
        // In production, this would use candle + sentence-transformers
        let mut embedding = vec![0.0f32; self.dimension];
        
        // Create a simple bag-of-words style embedding
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for word in words.iter() {
            let hash = self.hash_word(word);
            let idx = (hash as usize) % self.dimension;
            embedding[idx] += 1.0 / (words.len() as f32).sqrt();
        }
        
        // Normalize the vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }
        
        Ok(embedding)
    }

    /// Batch embed locally
    async fn embed_batch_local(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed_local(text).await?);
        }
        Ok(embeddings)
    }

    /// Simple hash function for word to index mapping
    fn hash_word(&self, word: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in word.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }

    /// Get embedding dimension
    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    /// Get model name
    pub fn get_model(&self) -> &str {
        &self.model
    }

    /// Check if using local inference
    pub fn is_local(&self) -> bool {
        self.use_local
    }
}

/// Embedding batcher with retry logic
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
                    warn!("Embedding attempt {} failed, retrying...", attempt);
                    tokio::time::sleep(
                        std::time::Duration::from_millis(self.retry_delay_ms)
                    ).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> EmbeddingConfig {
        EmbeddingConfig {
            embedding: crate::config::EmbeddingSettings {
                model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                dimension: 384,
                endpoint: "http://localhost:8000".to_string(),
                timeout_secs: 30,
            },
            batch: crate::config::BatchSettings {
                size: 32,
                max_retries: 3,
                retry_delay_ms: 1000,
            },
            cache: crate::config::CacheSettings {
                enabled: true,
                max_size_mb: 1024,
                ttl_secs: 86400,
            },
        }
    }

    #[test]
    fn test_embedding_vector_shape() {
        let config = test_config();
        let client = EmbeddingClient::new(&config);
        assert_eq!(client.get_dimension(), 384);
    }

    #[test]
    fn test_embedding_model_name() {
        let config = test_config();
        let client = EmbeddingClient::new(&config);
        assert_eq!(client.get_model(), "sentence-transformers/all-MiniLM-L6-v2");
    }

    #[tokio::test]
    async fn test_local_embedding() {
        let config = test_config();
        let client = EmbeddingClient::new_local(&config, "./models/test");
        
        let embedding = client.embed("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Embedding should be normalized");
    }
}
