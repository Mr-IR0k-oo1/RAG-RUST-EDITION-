use rag_pdf_dataset::config::EmbeddingConfig;
use rag_pdf_dataset::embedding::EmbeddingClient;

#[tokio::test]
async fn test_embedding_vector_shape() {
    let config = EmbeddingConfig {
        embedding: rag_pdf_dataset::config::EmbeddingSettings {
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            endpoint: "http://localhost:8000".to_string(),
            timeout_secs: 30,
        },
        batch: rag_pdf_dataset::config::BatchSettings {
            size: 32,
            max_retries: 3,
            retry_delay_ms: 1000,
        },
        cache: rag_pdf_dataset::config::CacheSettings {
            enabled: true,
            max_size_mb: 1024,
            ttl_secs: 86400,
        },
    };

    let client = EmbeddingClient::new(&config);

    // Verify dimension matches config
    assert_eq!(client.get_dimension(), 384);
}

#[tokio::test]
async fn test_embedding_model_name() {
    let config = EmbeddingConfig {
        embedding: rag_pdf_dataset::config::EmbeddingSettings {
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            endpoint: "http://localhost:8000".to_string(),
            timeout_secs: 30,
        },
        batch: rag_pdf_dataset::config::BatchSettings {
            size: 32,
            max_retries: 3,
            retry_delay_ms: 1000,
        },
        cache: rag_pdf_dataset::config::CacheSettings {
            enabled: true,
            max_size_mb: 1024,
            ttl_secs: 86400,
        },
    };

    let client = EmbeddingClient::new(&config);

    assert_eq!(
        client.get_model(),
        "sentence-transformers/all-MiniLM-L6-v2"
    );
}
