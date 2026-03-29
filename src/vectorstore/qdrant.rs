use anyhow::Result;
use log::{info, warn};
use serde_json::Value;

use crate::config::QdrantConfig;

/// Search infrastructure for vector storage and retrieval
pub struct QdrantStore {
    url: String,
    api_key: Option<String>,
    collection_name: String,
    vector_size: usize,
    client: reqwest::Client,
}

/// Search result from vector store
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: Value,
}

impl QdrantStore {
    /// Create a new Qdrant store connection
    pub fn new(config: &QdrantConfig) -> Self {
        Self {
            url: config.qdrant.url.clone(),
            api_key: if config.qdrant.api_key.is_empty() {
                None
            } else {
                Some(config.qdrant.api_key.clone())
            },
            collection_name: config.collection.name.clone(),
            vector_size: config.collection.vector_size,
            client: reqwest::Client::new(),
        }
    }

    /// Ensure collection exists with correct schema
    pub async fn ensure_collection(&self) -> Result<()> {
        // Check if collection exists
        let url = format!("{}/collections/{}", self.url, self.collection_name);
        
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            info!("Collection '{}' already exists", self.collection_name);
            return Ok(());
        }

        // Create collection
        info!("Creating collection '{}'", self.collection_name);
        let create_url = format!("{}/collections/{}", self.url, self.collection_name);
        
        let body = serde_json::json!({
            "vectors": {
                "size": self.vector_size,
                "distance": "Cosine"
            },
            "optimizers_config": {
                "memmap_threshold": 20000
            },
            "on_disk_payload": true
        });

        let response = self.client
            .put(&create_url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to create collection: {}", error));
        }

        info!("Collection '{}' created successfully", self.collection_name);
        Ok(())
    }

    /// Insert a vector with metadata
    pub async fn insert(&self, id: u64, vector: Vec<f32>, payload: Value) -> Result<()> {
        let url = format!(
            "{}/collections/{}/points",
            self.url, self.collection_name
        );

        let point = serde_json::json!({
            "points": [{
                "id": id,
                "vector": vector,
                "payload": payload
            }]
        });

        let response = self.client
            .put(&url)
            .json(&point)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to insert point: {}", error));
        }

        Ok(())
    }

    /// Insert multiple vectors in batch
    pub async fn insert_batch(
        &self,
        points: Vec<(u64, Vec<f32>, Value)>,
    ) -> Result<()> {
        // Split into batches of 100
        for batch in points.chunks(100) {
            let points_json: Vec<Value> = batch
                .iter()
                .map(|(id, vector, payload)| {
                    serde_json::json!({
                        "id": id,
                        "vector": vector,
                        "payload": payload
                    })
                })
                .collect();

            let url = format!(
                "{}/collections/{}/points",
                self.url, self.collection_name
            );

            let body = serde_json::json!({ "points": points_json });

            let response = self.client
                .put(&url)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let error = response.text().await?;
                warn!("Batch insert warning: {}", error);
            }
        }

        Ok(())
    }

    /// Search for similar vectors
    pub async fn search(&self, vector: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>> {
        let url = format!(
            "{}/collections/{}/points/search",
            self.url, self.collection_name
        );

        let body = serde_json::json!({
            "vector": vector,
            "limit": top_k,
            "with_payload": true
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Search failed: {}", error));
        }

        let result: Value = response.json().await?;
        
        let results = result["result"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|r| SearchResult {
                id: r["id"].as_u64().unwrap_or(0),
                score: r["score"].as_f64().unwrap_or(0.0) as f32,
                payload: r["payload"].clone(),
            })
            .collect();

        Ok(results)
    }

    /// Delete vectors by ID
    pub async fn delete(&self, id: u64) -> Result<()> {
        let url = format!(
            "{}/collections/{}/points/delete",
            self.url, self.collection_name
        );

        let body = serde_json::json!({
            "points": [id]
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Delete failed: {}", error));
        }

        Ok(())
    }

    /// Recreate collection (drop and recreate)
    pub async fn recreate_collection(&self) -> Result<()> {
        // Delete existing collection
        let delete_url = format!("{}/collections/{}", self.url, self.collection_name);
        
        let _ = self.client.delete(&delete_url).send().await?;
        
        // Create new collection
        self.ensure_collection().await?;
        
        info!("Collection '{}' recreated", self.collection_name);
        Ok(())
    }

    /// Get point count in collection
    pub async fn count(&self) -> Result<u64> {
        let url = format!(
            "{}/collections/{}",
            self.url, self.collection_name
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get collection info"));
        }

        let result: Value = response.json().await?;
        let count = result["result"]["points_count"]
            .as_u64()
            .unwrap_or(0);

        Ok(count)
    }

    /// Search with payload filter
    pub async fn search_with_filter(
        &self,
        vector: Vec<f32>,
        top_k: usize,
        filter: Value,
    ) -> Result<Vec<SearchResult>> {
        let url = format!(
            "{}/collections/{}/points/search",
            self.url, self.collection_name
        );

        let body = serde_json::json!({
            "vector": vector,
            "limit": top_k,
            "with_payload": true,
            "filter": filter
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Search with filter failed: {}", error));
        }

        let result: Value = response.json().await?;
        
        let results = result["result"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|r| SearchResult {
                id: r["id"].as_u64().unwrap_or(0),
                score: r["score"].as_f64().unwrap_or(0.0) as f32,
                payload: r["payload"].clone(),
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            id: 1,
            score: 0.95,
            payload: serde_json::json!({"content": "test"}),
        };
        assert_eq!(result.id, 1);
        assert!((result.score - 0.95).abs() < 0.01);
    }
}
