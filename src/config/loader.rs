use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub application: ApplicationSettings,
    pub processing: ProcessingSettings,
    pub paths: PathSettings,
    pub output: OutputSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSettings {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingSettings {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub min_chunk_size: usize,
    pub max_tokens: usize,
    pub remove_stop_words: bool,
    pub max_workers: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSettings {
    pub raw_data_dir: String,
    pub processed_data_dir: String,
    pub indexes_dir: String,
    pub config_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub format: String,
    pub compression: bool,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub embedding: EmbeddingSettings,
    pub batch: BatchSettings,
    pub cache: CacheSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingSettings {
    pub model: String,
    pub dimension: usize,
    pub endpoint: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSettings {
    pub size: usize,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub enabled: bool,
    pub max_size_mb: usize,
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub qdrant: QdrantSettings,
    pub collection: CollectionSettings,
    pub indexing: IndexingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSettings {
    pub url: String,
    pub api_key: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSettings {
    pub name: String,
    pub vector_size: usize,
    pub distance_metric: String,
    pub on_disk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingSettings {
    pub hnsw_m: u32,
    pub hnsw_ef_construct: u32,
    pub hnsw_ef_search: u32,
}

impl AppConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl EmbeddingConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl QdrantConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
