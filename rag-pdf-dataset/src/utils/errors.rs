use thiserror::Error;

/// Unified error handling
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Vectorstore error: {0}")]
    Vectorstore(String),

    #[error("PDF processing error: {0}")]
    PdfProcessing(String),

    #[error("Ingestion error: {0}")]
    Ingestion(String),

    #[error("Dataset error: {0}")]
    Dataset(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
