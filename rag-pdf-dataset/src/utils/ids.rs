use uuid::Uuid;

/// ID generation utilities
pub struct IdGenerator;

impl IdGenerator {
    /// Generate a unique document ID
    pub fn document_id() -> String {
        format!("doc_{}", Uuid::new_v4())
    }

    /// Generate a unique chunk ID
    pub fn chunk_id() -> String {
        format!("chunk_{}", Uuid::new_v4())
    }

    /// Generate a unique batch ID
    pub fn batch_id() -> String {
        format!("batch_{}", Uuid::new_v4())
    }
}
