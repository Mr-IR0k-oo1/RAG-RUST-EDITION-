use rag_pdf_dataset::ingestion::{TextChunker, ChunkerConfig};

#[test]
fn test_chunking_respects_token_boundaries() {
    let config = ChunkerConfig {
        chunk_size: 10,
        overlap: 2,
        min_chunk_size: 3,
    };
    let chunker = TextChunker::new(config);

    let text = "word one word two word three word four word five word six word seven word eight word nine word ten word eleven word twelve";
    let chunks = chunker.chunk(text);

    assert!(!chunks.is_empty());
    // Each chunk should be max 10 tokens
    for chunk in &chunks {
        let token_count = chunk.split_whitespace().count();
        assert!(token_count <= 10, "Chunk exceeds max size: {} tokens", token_count);
    }
}

#[test]
fn test_chunking_respects_min_chunk_size() {
    let config = ChunkerConfig {
        chunk_size: 10,
        overlap: 2,
        min_chunk_size: 5,
    };
    let chunker = TextChunker::new(config);

    let text = "a b c d e f g h i j k l m n o p q r s t";
    let chunks = chunker.chunk(text);

    // All chunks should respect min size
    for chunk in &chunks {
        let token_count = chunk.split_whitespace().count();
        assert!(token_count >= 5, "Chunk below min size: {} tokens", token_count);
    }
}

#[test]
fn test_chunking_with_overlap() {
    let config = ChunkerConfig {
        chunk_size: 5,
        overlap: 2,
        min_chunk_size: 3,
    };
    let chunker = TextChunker::new(config);

    let text = "one two three four five six seven eight nine ten";
    let chunks = chunker.chunk(text);

    // With overlap, chunks should reuse tokens
    assert!(chunks.len() > 1, "Expected multiple chunks with overlap");
}
