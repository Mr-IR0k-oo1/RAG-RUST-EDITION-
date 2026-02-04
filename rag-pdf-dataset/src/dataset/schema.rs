use serde::{Deserialize, Serialize};
use crate::types::Metadata;

/// Represents a complete document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filename: String,
    pub source: String,
    pub content: String,
    pub metadata: Metadata,
}

/// Represents a chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub metadata: ChunkMetadata,
}

/// Metadata specific to a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub filename: String,
    pub source: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub token_count: usize,
    pub created_at: String,
}

impl Document {
    pub fn new(id: String, filename: String, source: String, content: String) -> Self {
        Self {
            id,
            filename,
            source,
            content,
            metadata: Metadata::default(),
        }
    }
}

impl Chunk {
    pub fn new(
        id: String,
        document_id: String,
        content: String,
        chunk_index: usize,
        total_chunks: usize,
        filename: String,
        source: String,
        token_count: usize,
    ) -> Self {
        let created_at = chrono::Utc::now().to_rfc3339();

        Self {
            id,
            document_id,
            content,
            chunk_index,
            total_chunks,
            metadata: ChunkMetadata {
                filename,
                source,
                chunk_index,
                total_chunks,
                start_char: 0,
                end_char: 0,
                token_count,
                created_at,
            },
        }
    }
}
