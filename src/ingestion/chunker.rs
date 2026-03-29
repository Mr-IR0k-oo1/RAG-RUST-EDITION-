use crate::ingestion::tokenizer::Tokenizer;
use crate::types::metadata::Metadata;
use regex::Regex;

/// Configuration for text chunking
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub chunk_size: usize,
    pub overlap: usize,
    pub min_chunk_size: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            overlap: 64,
            min_chunk_size: 50,
        }
    }
}

/// A text chunk with metadata
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub token_count: usize,
    pub metadata: Metadata,
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

    /// Split text into chunks using recursive character splitting
    pub fn chunk(&self, text: &str) -> Vec<String> {
        self.recursive_split(text, self.config.chunk_size, self.config.overlap)
    }

    /// Split text into chunks at sentence boundaries
    pub fn chunk_by_sentences(&self, text: &str) -> Vec<String> {
        let sentences = self.tokenizer.tokenize_sentences(text);
        self.accumulate_chunks(&sentences, self.config.chunk_size, self.config.overlap)
    }

    /// Split text and return chunks with full metadata
    pub fn chunk_with_metadata(&self, text: &str, doc_id: &str) -> Vec<Chunk> {
        let text_chunks = self.chunk(text);
        let total_chunks = text_chunks.len();
        let mut current_pos = 0;
        let mut chunks = Vec::new();

        for (i, content) in text_chunks.iter().enumerate() {
            let start_char = current_pos;
            let end_char = current_pos + content.len();
            let token_count = self.tokenizer.count_tokens(content);

            chunks.push(Chunk {
                id: format!("chunk_{}_{}", doc_id, i),
                document_id: doc_id.to_string(),
                content: content.clone(),
                chunk_index: i,
                total_chunks,
                start_char,
                end_char,
                token_count,
                metadata: Metadata::default(),
            });

            current_pos = end_char;
        }

        chunks
    }

    /// Detect chapter boundaries in text
    pub fn detect_chapters(&self, text: &str) -> Vec<(usize, String)> {
        let patterns = [
            r"(?im)^chapter\s+(\d+|[ivxlcdm]+)[\s:.]",
            r"(?im)^(\d+)\.\s+[A-Z]",
            r"(?im)^(part|section)\s+(\d+|[ivxlcdm]+)[\s:.]",
        ];

        let combined = format!("({})", patterns.join("|"));
        let re = Regex::new(&combined).unwrap();

        let mut chapters = Vec::new();

        for cap in re.captures_iter(text) {
            let match_start = cap.get(0).unwrap().start();
            let title = cap.get(0).unwrap().as_str().to_string();
            chapters.push((match_start, title));
        }

        chapters
    }

    /// Recursive character splitting algorithm
    fn recursive_split(&self, text: &str, max_tokens: usize, overlap: usize) -> Vec<String> {
        let token_count = self.tokenizer.count_tokens(text);

        if token_count <= max_tokens {
            return if token_count >= self.config.min_chunk_size {
                vec![text.to_string()]
            } else {
                vec![]
            };
        }

        // Try splitting on paragraphs
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        if paragraphs.len() > 1 {
            return self.accumulate_chunks(
                &paragraphs.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                max_tokens,
                overlap,
            );
        }

        // Try splitting on sentences
        let sentences = self.tokenizer.tokenize_sentences(text);
        if sentences.len() > 1 {
            return self.accumulate_chunks(&sentences, max_tokens, overlap);
        }

        // Fall back to word splitting
        let words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        self.accumulate_chunks(&words, max_tokens, overlap)
    }

    /// Accumulate segments into chunks respecting token limits
    fn accumulate_chunks(
        &self,
        segments: &[String],
        max_tokens: usize,
        overlap: usize,
    ) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current: Vec<String> = Vec::new();
        let mut current_tokens = 0;

        for segment in segments {
            let segment_tokens = self.tokenizer.count_tokens(segment);

            if current_tokens + segment_tokens > max_tokens && !current.is_empty() {
                // Save current chunk
                let chunk_text = current.join(" ");
                if self.tokenizer.count_tokens(&chunk_text) >= self.config.min_chunk_size {
                    chunks.push(chunk_text);
                }

                // Start new chunk with overlap
                let overlap_segments = self.get_overlap_segments(&current, overlap);
                current = overlap_segments;
                current.push(segment.clone());
                current_tokens = self.tokenizer.count_tokens(&current.join(" "));
            } else {
                current.push(segment.clone());
                current_tokens += segment_tokens;
            }
        }

        // Add remaining segments
        if !current.is_empty() {
            let chunk_text = current.join(" ");
            if self.tokenizer.count_tokens(&chunk_text) >= self.config.min_chunk_size {
                chunks.push(chunk_text);
            }
        }

        chunks
    }

    /// Get overlap segments from the end of current chunk
    fn get_overlap_segments(&self, segments: &[String], overlap_tokens: usize) -> Vec<String> {
        let mut overlap = Vec::new();
        let mut tokens = 0;

        for segment in segments.iter().rev() {
            let segment_tokens = self.tokenizer.count_tokens(segment);
            if tokens + segment_tokens > overlap_tokens {
                break;
            }
            overlap.insert(0, segment.clone());
            tokens += segment_tokens;
        }

        overlap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for chunk in &chunks {
            let token_count = chunk.split_whitespace().count();
            assert!(
                token_count <= 10,
                "Chunk exceeds max size: {} tokens",
                token_count
            );
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

        for chunk in &chunks {
            let token_count = chunk.split_whitespace().count();
            assert!(
                token_count >= 5,
                "Chunk below min size: {} tokens",
                token_count
            );
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

        assert!(chunks.len() > 1, "Expected multiple chunks with overlap");
    }

    #[test]
    fn test_sentence_chunking() {
        let config = ChunkerConfig {
            chunk_size: 20,
            overlap: 5,
            min_chunk_size: 5,
        };
        let chunker = TextChunker::new(config);

        let text = "This is sentence one. This is sentence two. This is sentence three. This is sentence four. This is sentence five. This is sentence six.";
        let chunks = chunker.chunk_by_sentences(text);

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_with_metadata() {
        let config = ChunkerConfig {
            chunk_size: 10,
            overlap: 2,
            min_chunk_size: 3,
        };
        let chunker = TextChunker::new(config);

        let text = "one two three four five six seven eight nine ten eleven twelve";
        let chunks = chunker.chunk_with_metadata(text, "doc_123");

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].document_id, "doc_123");
        assert_eq!(chunks[0].chunk_index, 0);
    }

    #[test]
    fn test_chapter_detection() {
        let config = ChunkerConfig::default();
        let chunker = TextChunker::new(config);

        let text = "Chapter 1: Introduction\nSome text.\nChapter 2: Methods\nMore text.";
        let chapters = chunker.detect_chapters(text);

        assert!(chapters.len() >= 2);
    }
}
