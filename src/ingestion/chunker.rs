use crate::ingestion::tokenizer::Tokenizer;

/// Configuration for text chunking
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub chunk_size: usize,
    pub overlap: usize,
    pub min_chunk_size: usize,
}

/// Splits text into overlapping chunks for embedding
pub struct TextChunker {
    config: ChunkerConfig,
    tokenizer: Tokenizer,
}

impl TextChunker {
    pub fn new(config: ChunkerConfig) -> Self {
        Self {
            config,
            tokenizer: Tokenizer::new(),
        }
    }

    /// Split text into chunks based on token count
    pub fn chunk(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenizer.tokenize(text);
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.overlap;
        let min_size = self.config.min_chunk_size;

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < tokens.len() {
            let end = std::cmp::min(start + chunk_size, tokens.len());
            let chunk_tokens = &tokens[start..end];

            if chunk_tokens.len() >= min_size {
                let chunk_text = chunk_tokens.join(" ");
                chunks.push(chunk_text);
            }

            start += chunk_size - overlap;
        }

        chunks
    }

    /// Split text into chunks based on sentences (semantic chunks)
    pub fn chunk_by_sentences(&self, text: &str) -> Vec<String> {
        // TODO: Implement sentence-aware chunking
        vec![text.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking() {
        let config = ChunkerConfig {
            chunk_size: 10,
            overlap: 2,
            min_chunk_size: 3,
        };
        let chunker = TextChunker::new(config);
        let text = "one two three four five six seven eight nine ten eleven twelve";
        let chunks = chunker.chunk(text);
        assert!(!chunks.is_empty());
    }
}
