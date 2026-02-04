use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Shared metadata structure for documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_type: String,
    pub tags: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            source_type: "pdf".to_string(),
            tags: Vec::new(),
        }
    }
}
