# Implementation Plan - Phase by Phase

## Table of Contents

1. [Phase 1: Multi-Format Book Loader](#phase-1-multi-format-book-loader)
2. [Phase 2: Advanced Text Processing](#phase-2-advanced-text-processing)
3. [Phase 3: Local Embedding with Candle](#phase-3-local-embedding-with-candle)
4. [Phase 4: Real Qdrant Integration](#phase-4-real-qdrant-integration)
5. [Phase 5: RAG Retriever + Re-ranking](#phase-5-rag-retriever--re-ranking)
6. [Phase 6: Fine-Tuning Dataset Generator](#phase-6-fine-tuning-dataset-generator)
7. [Phase 7: Production Hardening](#phase-7-production-hardening)

---

## Phase 1: Multi-Format Book Loader

### Goal
Replace the stub `PdfLoader` with a production book loader supporting PDF, EPUB, and scanned documents (OCR).

### Files to Create/Modify

#### New: `src/ingestion/book_loader.rs`

```rust
// BookLoader struct with methods:
// - load(path) -> Result<BookContent>
// - load_directory(path) -> Result<Vec<BookContent>>
// - detect_format(path) -> BookFormat
// - load_pdf(path) -> Result<String>
// - load_epub(path) -> Result<String>
// - load_scanned(path) -> Result<String>  // via OCR
```

#### New: `src/ingestion/ocr.rs`

```rust
// OCR processor for scanned documents
// - process_image(image_path) -> Result<String>
// - process_pdf_as_images(pdf_path) -> Result<String>
// Uses ocrs crate
```

#### Modify: `src/ingestion/mod.rs`

Add new modules:
```rust
pub mod book_loader;
pub mod ocr;

pub use book_loader::{BookLoader, BookFormat, BookContent};
pub use ocr::OcrProcessor;
```

#### Modify: `Cargo.toml`

Add dependencies:
```toml
epub = "2.1"
ocrs = "0.8"
image = "0.24"  # For OCR image processing
```

### Data Structures

```rust
pub enum BookFormat {
    Pdf,
    Epub,
    Scanned,  // Directory of images or image-based PDF
}

pub struct BookContent {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: BookFormat,
    pub chapters: Vec<Chapter>,
    pub raw_text: String,
    pub metadata: BookMetadata,
}

pub struct Chapter {
    pub index: usize,
    pub title: String,
    pub content: String,
    pub page_start: Option<usize>,
    pub page_end: Option<usize>,
}

pub struct BookMetadata {
    pub filename: String,
    pub file_size: u64,
    pub page_count: Option<usize>,
    pub created_at: DateTime<Utc>,
}
```

### Implementation Steps

1. Implement PDF loading using `pdfium-render`
2. Implement EPUB loading using `epub` crate
3. Implement OCR pipeline using `ocrs` crate
4. Add format auto-detection based on file extension and content
5. Add chapter/section detection via heading patterns
6. Write tests for each format

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(BookLoader::detect_format("book.pdf"), BookFormat::Pdf);
        assert_eq!(BookLoader::detect_format("book.epub"), BookFormat::Epub);
    }

    #[tokio::test]
    async fn test_pdf_extraction() {
        let content = BookLoader::load("tests/fixtures/sample.pdf").await.unwrap();
        assert!(!content.raw_text.is_empty());
        assert!(!content.chapters.is_empty());
    }
}
```

---

## Phase 2: Advanced Text Processing

### Goal
Implement production-grade text cleaning and intelligent chunking with sentence awareness and chapter detection.

### Files to Create/Modify

#### Modify: `src/ingestion/cleaner.rs`

Implement all stub methods:
```rust
impl TextCleaner {
    pub fn clean(&self, text: &str) -> String {
        // Unicode normalization (NFKC)
        // Whitespace normalization
        // Fix hyphenation across line breaks
        // Remove page numbers, headers, footers
        // Normalize quotes and dashes
    }

    pub fn remove_special_chars(&self, text: &str) -> String {
        // Remove non-printable characters
        // Preserve sentence punctuation
    }

    pub fn remove_links(&self, text: &str) -> String {
        // Remove URLs
        // Remove email addresses
    }

    pub fn fix_hyphenation(&self, text: &str) -> String {
        // Join words split across lines: "exam-\nple" -> "example"
    }

    pub fn remove_headers_footers(&self, text: &str) -> String {
        // Detect and remove repeating page headers/footers
    }
}
```

#### Modify: `src/ingestion/chunker.rs`

Implement sentence-aware chunking:
```rust
impl TextChunker {
    pub fn chunk(&self, text: &str) -> Vec<String> {
        // Recursive character splitting at 512 tokens
        // Split priority: paragraphs > sentences > words
        // Maintain 12.5% overlap between chunks
    }

    pub fn chunk_by_sentences(&self, text: &str) -> Vec<String> {
        // Split at sentence boundaries
        // Accumulate sentences until reaching target size
        // Include overlap from previous chunk
    }

    pub fn chunk_with_metadata(&self, text: &str, doc_id: &str) -> Vec<Chunk> {
        // Return chunks with full metadata:
        // - chunk_id, document_id
        // - chunk_index, total_chunks
        // - start_char, end_char
        // - token_count
    }

    pub fn detect_chapters(&self, text: &str) -> Vec<Chapter> {
        // Detect chapter boundaries via patterns:
        // - "Chapter N" / "CHAPTER N"
        // - Roman numerals: "I.", "II.", "III."
        // - Numbered sections: "1.", "1.1", "1.1.1"
    }
}
```

#### Modify: `src/ingestion/tokenizer.rs`

Add proper tokenization:
```rust
impl Tokenizer {
    pub fn count_tokens(&self, text: &str) -> usize {
        // More accurate token counting
        // Handle special characters properly
    }

    pub fn split_sentences(&self, text: &str) -> Vec<String> {
        // Better sentence splitting using regex
        // Handle abbreviations (Dr., Mr., etc.)
        // Handle decimal numbers (3.14)
    }
}
```

### Chunking Algorithm (Pseudocode)

```
function recursive_split(text, max_tokens, overlap):
    if count_tokens(text) <= max_tokens:
        return [text]
    
    # Try splitting on paragraphs
    paragraphs = text.split("\n\n")
    if len(paragraphs) > 1:
        return accumulate_chunks(paragraphs, max_tokens, overlap)
    
    # Try splitting on sentences
    sentences = split_sentences(text)
    if len(sentences) > 1:
        return accumulate_chunks(sentences, max_tokens, overlap)
    
    # Fall back to word splitting
    words = text.split(" ")
    return accumulate_chunks(words, max_tokens, overlap)

function accumulate_chunks(segments, max_tokens, overlap):
    chunks = []
    current = []
    current_tokens = 0
    
    for segment in segments:
        segment_tokens = count_tokens(segment)
        
        if current_tokens + segment_tokens > max_tokens:
            # Save current chunk
            chunks.append(" ".join(current))
            
            # Start new chunk with overlap
            overlap_segments = get_overlap(current, overlap)
            current = overlap_segments + [segment]
            current_tokens = count_tokens(" ".join(current))
        else:
            current.append(segment)
            current_tokens += segment_tokens
    
    if current:
        chunks.append(" ".join(current))
    
    return chunks
```

---

## Phase 3: Local Embedding with Candle

### Goal
Replace the dummy embedding client with local inference using Candle framework and sentence-transformers models.

### Files to Create/Modify

#### New: `src/embedding/model.rs`

```rust
// Load and run sentence-transformers model
pub struct EmbeddingModel {
    model: SentenceTransformer,
    device: Device,
}

impl EmbeddingModel {
    pub fn load(model_path: &Path) -> Result<Self>;
    pub fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    pub fn embed_single(&self, text: &str) -> Result<Vec<f32>>;
    pub fn dimension(&self) -> usize;  // 384 for all-MiniLM-L6-v2
}
```

#### New: `src/embedding/tokenizer_impl.rs`

```rust
// Tokenizer for the embedding model
pub struct ModelTokenizer {
    tokenizer: Tokenizer,  // HuggingFace tokenizers
}

impl ModelTokenizer {
    pub fn load(model_path: &Path) -> Result<Self>;
    pub fn encode(&self, texts: &[String]) -> Result<EncodedInput>;
    pub fn encode_single(&self, text: &str) -> Result<EncodedInput>;
}
```

#### Modify: `src/embedding/client.rs`

```rust
impl EmbeddingClient {
    pub fn new_local(config: &EmbeddingConfig) -> Result<Self> {
        // Load model from model_path
        // Initialize tokenizer
        // Set device (CPU/GPU)
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize
        // Run through model
        // Mean pooling
        // L2 normalize
        // Return 384-dim vector
    }

    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Batch tokenization
        // Batch inference
        // Pooling and normalization
    }
}
```

### Model Download

Add a script to download the model:
```
scripts/download_model.rs
- Download all-MiniLM-L6-v2 from HuggingFace
- Save to ./models/all-MiniLM-L6-v2/
- Files: config.json, tokenizer.json, model.safetensors
```

### Embedding Pipeline

```
Text Input
    |
    v
[Tokenize] --> Token IDs + Attention Mask
    |
    v
[Model Forward Pass] --> Hidden States (batch_size, seq_len, 384)
    |
    v
[Mean Pooling] --> Sentence Embeddings (batch_size, 384)
    |
    v
[L2 Normalize] --> Unit Vectors (batch_size, 384)
    |
    v
Output Embeddings
```

### Dependencies to Add

```toml
candle-core = "0.6"
candle-nn = "0.6"
candle-transformers = "0.6"
tokenizers = "0.15"
hf-hub = "0.3"  # For downloading models
```

---

## Phase 4: Real Qdrant Integration

### Goal
Implement actual Qdrant client operations for vector storage and retrieval.

### Files to Modify

#### Modify: `src/vectorstore/qdrant.rs`

```rust
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{PointStruct, SearchPoints, VectorParams};

impl QdrantStore {
    pub async fn connect(config: &QdrantConfig) -> Result<Self> {
        let client = QdrantClient::from_url(&config.qdrant.url)
            .with_api_key(&config.qdrant.api_key)
            .build()?;
        
        Ok(Self { client, config })
    }

    pub async fn ensure_collection(&self) -> Result<()> {
        // Check if collection exists
        // If not, create with correct schema
        // Vector size: 384
        // Distance: Cosine
    }

    pub async fn insert(&self, id: u64, vector: Vec<f32>, payload: Value) -> Result<()> {
        let point = PointStruct::new(id, vector, payload);
        self.client
            .upsert_points(&self.collection_name, vec![point], None)
            .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, points: Vec<PointStruct>) -> Result<()> {
        // Batch insert for efficiency
        // Split into batches of 100
    }

    pub async fn search(&self, vector: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>> {
        let results = self.client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector,
                limit: top_k as u64,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;
        
        Ok(results.result.into_iter().map(|p| SearchResult {
            id: p.id.unwrap().as_num().unwrap(),
            score: p.score,
            payload: p.payload.into(),
        }).collect())
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        // Delete point by ID
    }

    pub async fn recreate_collection(&self) -> Result<()> {
        // Delete existing collection
        // Create new collection with schema
    }

    pub async fn count(&self) -> Result<u64> {
        // Return number of points in collection
    }
}
```

### Payload Schema

```rust
pub struct ChunkPayload {
    pub content: String,
    pub document_id: String,
    pub document_title: String,
    pub chapter_index: usize,
    pub chapter_title: String,
    pub chunk_index: usize,
    pub page_number: Option<usize>,
    pub token_count: usize,
}
```

### Dependencies to Add

```toml
qdrant-client = "1.7"
```

---

## Phase 5: RAG Retriever + Re-ranking

### Goal
Implement hybrid retrieval with re-ranking for production-quality search.

### Files to Create/Modify

#### New: `src/rag/reranker.rs`

```rust
pub struct Reranker {
    model: CrossEncoderModel,
}

impl Reranker {
    pub fn load(model_path: &Path) -> Result<Self>;
    
    pub fn rerank(&self, query: &str, candidates: &[SearchResult]) -> Result<Vec<ScoredChunk>> {
        // For each candidate, compute cross-encoder score
        // [CLS] query [SEP] document [SEP] -> score
        // Sort by score descending
        // Return top-K
    }
}
```

#### Modify: `src/rag/retriever.rs`

```rust
impl Retriever {
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RetrievedChunk>> {
        // Embed query
        let query_vector = self.embedding_client.embed(query).await?;
        
        // Search (retrieve more candidates for re-ranking)
        let candidates = self.vectorstore.search(query_vector, top_k * 4).await?;
        
        // Re-rank
        let reranked = self.reranker.rerank(query, &candidates)?;
        
        // Return top-K
        Ok(reranked.into_iter().take(top_k).collect())
    }

    pub async fn retrieve_with_scores(
        &self,
        query: &str,
        top_k: usize,
        min_score: f32,
    ) -> Result<Vec<RetrievedChunk>> {
        // Retrieve with minimum score threshold
    }
}
```

#### Modify: `src/rag/prompt.rs`

```rust
impl PromptBuilder {
    pub fn build_prompt(query: &str, chunks: &[RetrievedChunk]) -> String {
        // Enhanced prompt template
        // Include source attribution
        // Handle empty context gracefully
    }

    pub fn build_rag_prompt(query: &str, chunks: &[RetrievedChunk]) -> String {
        // RAG-specific prompt with instructions:
        // - Answer only from context
        // - Cite sources
        // - Say "I don't know" if context insufficient
    }
}
```

---

## Phase 6: Fine-Tuning Dataset Generator

### Goal
Generate fine-tuning datasets from book content in multiple formats.

### Files to Create

#### New: `src/dataset/generator.rs`

```rust
pub struct DatasetGenerator {
    config: DatasetConfig,
    qa_generator: QAGenerator,
}

impl DatasetGenerator {
    pub fn generate_from_chunks(&self, chunks: &[Chunk]) -> Result<Dataset> {
        // For each chunk:
        // 1. Generate Q&A pairs
        // 2. Create instruction format entries
        // 3. Create chat format entries
        // 4. Validate quality
        // 5. Deduplicate
    }

    pub fn generate_splits(&self, dataset: &Dataset) -> Result<DataSplits> {
        // Split into train/validation/test
        // Ensure no data leakage (same source in multiple splits)
    }
}
```

#### New: `src/dataset/qa_generator.rs`

```rust
pub struct QAGenerator;

impl QAGenerator {
    pub fn generate_factual_qa(&self, chunk: &Chunk) -> Result<Vec<QAPair>> {
        // Extract key facts from chunk
        // Generate "What is X?" style questions
        // Answer = relevant sentences from chunk
    }

    pub fn generate_inferential_qa(&self, chunk: &Chunk) -> Result<Vec<QAPair>> {
        // Generate "Why does X?" style questions
        // Requires reasoning over chunk content
    }

    pub fn generate_comparative_qa(&self, chunk: &Chunk) -> Result<Vec<QAPair>> {
        // Generate "How does X compare to Y?" style questions
        // Extract comparison points from chunk
    }
}
```

#### New: `src/dataset/formatter.rs`

```rust
pub struct DatasetFormatter;

impl DatasetFormatter {
    pub fn to_instruction(&self, qa: &QAPair) -> InstructionEntry {
        // {
        //   "instruction": "Explain the concept of...",
        //   "input": "Context: ...",
        //   "output": "The concept of ... refers to..."
        // }
    }

    pub fn to_qa(&self, qa: &QAPair) -> QAEntry {
        // {
        //   "question": "What is ...?",
        //   "answer": "...",
        //   "context": "...",
        //   "source": "book_title, chapter_N"
        // }
    }

    pub fn to_chat(&self, qa: &QAPair) -> ChatEntry {
        // {
        //   "messages": [
        //     {"role": "system", "content": "You are a knowledgeable assistant."},
        //     {"role": "user", "content": "What is ...?"},
        //     {"role": "assistant", "content": "..."}
        //   ]
        // }
    }
}
```

#### New: `src/dataset/validator.rs`

```rust
pub struct DatasetValidator;

impl DatasetValidator {
    pub fn validate(&self, dataset: &Dataset) -> Result<ValidationReport> {
        // Check:
        // - Answer matches source content
        // - No duplicate questions
        // - Answer length within bounds
        // - Question is actually a question
        // - No PII or sensitive content
    }
}
```

#### Modify: `src/dataset/writer.rs`

```rust
impl DatasetWriter {
    pub fn write_instruction(&mut self, entries: &[InstructionEntry]) -> Result<()>;
    pub fn write_qa(&mut self, entries: &[QAEntry]) -> Result<()>;
    pub fn write_chat(&mut self, entries: &[ChatEntry]) -> Result<()>;
    pub fn write_splits(&mut self, splits: &DataSplits) -> Result<()>;
}
```

### Dataset Formats

**Instruction Format**:
```json
{"instruction": "Summarize the main argument of Chapter 3", "input": "Chapter 3 discusses the evolution of neural networks from perceptrons to deep learning architectures...", "output": "Chapter 3 traces the development of neural networks, starting from simple perceptrons in the 1950s through to modern deep learning architectures like transformers."}
```

**Q&A Format**:
```json
{"question": "What year was the transformer architecture introduced?", "answer": "The transformer architecture was introduced in 2017 by Vaswani et al. in the paper 'Attention Is All You Need'.", "context": "In 2017, Vaswani et al. introduced the transformer architecture...", "source": "Deep Learning Fundamentals, Chapter 8"}
```

**Chat Format**:
```json
{"messages": [{"role": "system", "content": "You are a helpful assistant knowledgeable about machine learning."}, {"role": "user", "content": "Explain the attention mechanism in transformers."}, {"role": "assistant", "content": "The attention mechanism in transformers allows the model to weigh the importance of different parts of the input when producing each part of the output..."}]}
```

---

## Phase 7: Production Hardening

### Goal
Add CLI, logging, error recovery, and evaluation capabilities.

### Files to Create/Modify

#### New: `src/cli.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rag-pdf-dataset")]
#[command(about = "Production RAG system for book-to-dataset conversion")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest books from a directory
    Ingest {
        /// Path to directory containing books
        #[arg(short, long)]
        path: String,
        
        /// Force re-processing of already processed books
        #[arg(short, long)]
        force: bool,
    },
    
    /// Generate fine-tuning dataset
    Generate {
        /// Number of Q&A pairs per chunk
        #[arg(short, long, default_value = "3")]
        num_pairs: usize,
    },
    
    /// Rebuild vector index
    RebuildIndex,
    
    /// Query the RAG system
    Query {
        /// The query text
        #[arg(short, long)]
        text: String,
        
        /// Number of results to return
        #[arg(short, long, default_value = "5")]
        top_k: usize,
    },
    
    /// Download embedding model
    DownloadModel,
    
    /// Evaluate retrieval quality
    Evaluate {
        /// Path to evaluation dataset
        #[arg(short, long)]
        eval_set: String,
    },
}
```

#### Modify: `src/main.rs`

```rust
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    
    let cli = cli::Cli::parse();
    
    match cli.command {
        Commands::Ingest { path, force } => {
            // Load config
            // Initialize components
            // Process books
        }
        Commands::Generate { num_pairs } => {
            // Load processed chunks
            // Generate dataset
            // Write to JSONL
        }
        Commands::RebuildIndex => {
            // Connect to Qdrant
            // Recreate collection
            // Re-embed and insert all chunks
        }
        Commands::Query { text, top_k } => {
            // Embed query
            // Search
            // Re-rank
            // Build prompt
            // Display results
        }
        Commands::DownloadModel => {
            // Download sentence-transformers model
        }
        Commands::Evaluate { eval_set } => {
            // Load evaluation set
            // Run retrieval
            // Compute metrics
            // Display report
        }
    }
    
    Ok(())
}
```

#### New: `src/evaluation/metrics.rs`

```rust
pub struct RetrievalMetrics;

impl RetrievalMetrics {
    pub fn precision_at_k(retrieved: &[String], relevant: &[String], k: usize) -> f64;
    pub fn recall_at_k(retrieved: &[String], relevant: &[String], k: usize) -> f64;
    pub fn f1_at_k(retrieved: &[String], relevant: &[String], k: usize) -> f64;
    pub fn mrr(retrieved: &[String], relevant: &[String]) -> f64;
    pub fn ndcg_at_k(retrieved: &[String], relevant: &[String], k: usize) -> f64;
}
```

### Progress Reporting

Use `indicatif` for progress bars:
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(books.len() as u64);
pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
    .unwrap());

for book in books {
    // Process book
    pb.inc(1);
}
pb.finish_with_message("Done");
```

### Dependencies to Add

```toml
clap = { version = "4", features = ["derive"] }
indicatif = "0.17"
```

---

## Summary of All Changes

### New Files (14)
```
src/ingestion/book_loader.rs
src/ingestion/ocr.rs
src/embedding/model.rs
src/embedding/tokenizer_impl.rs
src/rag/reranker.rs
src/dataset/generator.rs
src/dataset/qa_generator.rs
src/dataset/formatter.rs
src/dataset/validator.rs
src/evaluation/metrics.rs
src/evaluation/mod.rs
src/cli.rs
scripts/download_model.rs
configs/dataset.toml
```

### Modified Files (12)
```
Cargo.toml                    # Add new dependencies
src/main.rs                   # Add CLI
src/ingestion/mod.rs          # Add new modules
src/ingestion/cleaner.rs      # Implement all methods
src/ingestion/chunker.rs      # Implement sentence-aware chunking
src/ingestion/tokenizer.rs    # Improve tokenization
src/embedding/client.rs       # Real embedding implementation
src/vectorstore/qdrant.rs     # Real Qdrant operations
src/rag/retriever.rs          # Add re-ranking
src/rag/prompt.rs             # Enhanced prompts
src/dataset/mod.rs            # Add new modules
src/dataset/writer.rs         # Multi-format output
```

### New Dependencies (10)
```
epub = "2.1"
ocrs = "0.8"
image = "0.24"
candle-core = "0.6"
candle-nn = "0.6"
candle-transformers = "0.6"
tokenizers = "0.15"
hf-hub = "0.3"
qdrant-client = "1.7"
clap = { version = "4", features = ["derive"] }
indicatif = "0.17"
```
