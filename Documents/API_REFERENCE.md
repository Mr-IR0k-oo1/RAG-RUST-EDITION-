# API Reference

## Table of Contents

1. [Ingestion Module](#1-ingestion-module)
2. [Embedding Module](#2-embedding-module)
3. [Vector Store Module](#3-vector-store-module)
4. [RAG Module](#4-rag-module)
5. [Dataset Module](#5-dataset-module)
6. [Configuration Module](#6-configuration-module)
7. [Types Module](#7-types-module)
8. [Utilities Module](#8-utilities-module)

---

## 1. Ingestion Module

### `src/ingestion/book_loader.rs`

#### `BookLoader`

```rust
/// Multi-format book loader supporting PDF, EPUB, and scanned documents
pub struct BookLoader;
```

#### Methods

```rust
impl BookLoader {
    /// Load a single book file
    ///
    /// # Arguments
    /// * `path` - Path to the book file
    ///
    /// # Returns
    /// * `Result<BookContent>` - Extracted book content with chapters
    ///
    /// # Example
    /// ```rust
    /// let book = BookLoader::load("data/raw/books/textbook.pdf").await?;
    /// println!("Title: {}", book.title);
    /// println!("Chapters: {}", book.chapters.len());
    /// ```
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<BookContent>

    /// Load all books from a directory (recursive)
    ///
    /// # Arguments
    /// * `path` - Directory path containing book files
    ///
    /// # Returns
    /// * `Result<Vec<BookContent>>` - All extracted books
    pub async fn load_directory<P: AsRef<Path>>(path: P) -> Result<Vec<BookContent>>

    /// Detect book format from file extension and content
    ///
    /// # Arguments
    /// * `path` - Path to the file
    ///
    /// # Returns
    /// * `BookFormat` - Detected format (Pdf, Epub, Scanned)
    pub fn detect_format<P: AsRef<Path>>(path: P) -> BookFormat
}
```

#### Types

```rust
/// Supported book formats
pub enum BookFormat {
    Pdf,
    Epub,
    Scanned,
}

/// Content extracted from a book
pub struct BookContent {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: BookFormat,
    pub chapters: Vec<Chapter>,
    pub raw_text: String,
    pub metadata: BookMetadata,
}

/// A single chapter in a book
pub struct Chapter {
    pub index: usize,
    pub title: String,
    pub content: String,
    pub page_start: Option<usize>,
    pub page_end: Option<usize>,
}

/// Metadata about the book file
pub struct BookMetadata {
    pub filename: String,
    pub file_size: u64,
    pub page_count: Option<usize>,
    pub created_at: DateTime<Utc>,
}
```

---

### `src/ingestion/ocr.rs`

#### `OcrProcessor`

```rust
/// OCR processor for scanned documents
pub struct OcrProcessor;
```

#### Methods

```rust
impl OcrProcessor {
    /// Create a new OCR processor
    pub fn new() -> Result<Self>

    /// Process a single image and extract text
    ///
    /// # Arguments
    /// * `image_path` - Path to the image file
    ///
    /// # Returns
    /// * `Result<String>` - Extracted text
    pub async fn process_image<P: AsRef<Path>>(&self, image_path: P) -> Result<String>

    /// Process a PDF as images (for scanned PDFs)
    ///
    /// # Arguments
    /// * `pdf_path` - Path to the PDF file
    ///
    /// # Returns
    /// * `Result<String>` - Extracted text from all pages
    pub async fn process_pdf_as_images<P: AsRef<Path>>(&self, pdf_path: P) -> Result<String>
}
```

---

### `src/ingestion/cleaner.rs`

#### `TextCleaner`

```rust
/// Text cleaner for normalizing extracted text
pub struct TextCleaner;
```

#### Methods

```rust
impl TextCleaner {
    /// Create a new text cleaner
    pub fn new() -> Self

    /// Clean text: normalize unicode, whitespace, trim
    pub fn clean(&self, text: &str) -> String

    /// Remove special characters while preserving sentence structure
    pub fn remove_special_chars(&self, text: &str) -> String

    /// Remove URLs and email addresses
    pub fn remove_links(&self, text: &str) -> String

    /// Fix words split across line breaks (hyphenation)
    pub fn fix_hyphenation(&self, text: &str) -> String

    /// Remove repeating headers and footers
    pub fn remove_headers_footers(&self, text: &str) -> String
}
```

---

### `src/ingestion/chunker.rs`

#### `TextChunker`

```rust
/// Text chunker for splitting documents into segments
pub struct TextChunker {
    config: ChunkerConfig,
}
```

#### Methods

```rust
impl TextChunker {
    /// Create a new chunker with configuration
    pub fn new(config: ChunkerConfig) -> Self

    /// Split text into chunks using recursive character splitting
    pub fn chunk(&self, text: &str) -> Vec<String>

    /// Split text into chunks at sentence boundaries
    pub fn chunk_by_sentences(&self, text: &str) -> Vec<String>

    /// Split text and return chunks with metadata
    pub fn chunk_with_metadata(&self, text: &str, doc_id: &str) -> Vec<Chunk>

    /// Detect chapter boundaries in text
    pub fn detect_chapters(&self, text: &str) -> Vec<Chapter>
}
```

#### Types

```rust
/// Configuration for text chunking
pub struct ChunkerConfig {
    pub chunk_size: usize,      // Target chunk size in tokens (default: 512)
    pub overlap: usize,         // Overlap between chunks (default: 64)
    pub min_chunk_size: usize,  // Minimum chunk size (default: 50)
}
```

---

### `src/ingestion/tokenizer.rs`

#### `Tokenizer`

```rust
/// Simple tokenizer for token counting and sentence detection
pub struct Tokenizer;
```

#### Methods

```rust
impl Tokenizer {
    /// Create a new tokenizer
    pub fn new() -> Self

    /// Tokenize text into words
    pub fn tokenize(&self, text: &str) -> Vec<String>

    /// Count tokens in text
    pub fn count_tokens(&self, text: &str) -> usize

    /// Split text into sentences
    pub fn tokenize_sentences(&self, text: &str) -> Vec<String>
}
```

---

## 2. Embedding Module

### `src/embedding/client.rs`

#### `EmbeddingClient`

```rust
/// Embedding client for generating text embeddings
pub struct EmbeddingClient;
```

#### Methods

```rust
impl EmbeddingClient {
    /// Create a new embedding client with local model
    pub fn new_local(config: &EmbeddingConfig) -> Result<Self>

    /// Embed a single text
    ///
    /// # Arguments
    /// * `text` - Text to embed
    ///
    /// # Returns
    /// * `Result<Vec<f32>>` - 384-dimensional embedding vector
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>>

    /// Embed multiple texts
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>

    /// Get embedding dimension
    pub fn get_dimension(&self) -> usize

    /// Get model name
    pub fn get_model(&self) -> &str
}
```

---

### `src/embedding/batcher.rs`

#### `EmbeddingBatcher`

```rust
/// Rate-limit safe batching for embedding operations
pub struct EmbeddingBatcher;
```

#### Methods

```rust
impl EmbeddingBatcher {
    /// Create a new batcher
    pub fn new(client: EmbeddingClient, config: &EmbeddingConfig) -> Self

    /// Embed texts in batches with retry logic
    pub async fn embed_batched(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>
}
```

---

## 3. Vector Store Module

### `src/vectorstore/qdrant.rs`

#### `QdrantStore`

```rust
/// Qdrant vector store client
pub struct QdrantStore;
```

#### Methods

```rust
impl QdrantStore {
    /// Connect to Qdrant instance
    pub async fn connect(config: &QdrantConfig) -> Result<Self>

    /// Ensure collection exists with correct schema
    pub async fn ensure_collection(&self) -> Result<()>

    /// Insert a single vector with metadata
    ///
    /// # Arguments
    /// * `id` - Unique point ID
    /// * `vector` - Embedding vector
    /// * `payload` - Metadata as JSON
    pub async fn insert(&self, id: u64, vector: Vec<f32>, payload: Value) -> Result<()>

    /// Insert multiple vectors in batch
    pub async fn insert_batch(&self, points: Vec<PointStruct>) -> Result<()>

    /// Search for similar vectors
    ///
    /// # Arguments
    /// * `vector` - Query vector
    /// * `top_k` - Number of results to return
    ///
    /// # Returns
    /// * `Result<Vec<SearchResult>>` - Ranked results with scores
    pub async fn search(&self, vector: Vec<f32>, top_k: usize) -> Result<Vec<SearchResult>>

    /// Delete a vector by ID
    pub async fn delete(&self, id: u64) -> Result<()>

    /// Drop and recreate collection
    pub async fn recreate_collection(&self) -> Result<()>

    /// Get point count
    pub async fn count(&self) -> Result<u64>
}
```

#### Types

```rust
/// Search result from vector store
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: Value,
}
```

---

## 4. RAG Module

### `src/rag/retriever.rs`

#### `Retriever`

```rust
/// RAG retriever: embeds queries and searches vector store
pub struct Retriever;
```

#### Methods

```rust
impl Retriever {
    /// Create a new retriever
    pub fn new(embedding_client: EmbeddingClient, vectorstore: QdrantStore) -> Self

    /// Retrieve top-K relevant chunks for a query
    ///
    /// # Arguments
    /// * `query` - User query
    /// * `top_k` - Number of results
    ///
    /// # Returns
    /// * `Result<Vec<RetrievedChunk>>` - Retrieved chunks with scores
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RetrievedChunk>>

    /// Retrieve with minimum score threshold
    pub async fn retrieve_with_scores(
        &self,
        query: &str,
        top_k: usize,
        min_score: f32,
    ) -> Result<Vec<RetrievedChunk>>
}
```

#### Types

```rust
/// Retrieved chunk with relevance score
pub struct RetrievedChunk {
    pub score: f32,
    pub payload: Value,
}
```

---

### `src/rag/prompt.rs`

#### `PromptBuilder`

```rust
/// Prompt builder for RAG context injection
pub struct PromptBuilder;
```

#### Methods

```rust
impl PromptBuilder {
    /// Build a prompt with retrieved context
    pub fn build_prompt(query: &str, chunks: &[RetrievedChunk]) -> String

    /// Build RAG-specific prompt with instructions
    pub fn build_rag_prompt(query: &str, chunks: &[RetrievedChunk]) -> String
}
```

---

## 5. Dataset Module

### `src/dataset/generator.rs`

#### `DatasetGenerator`

```rust
/// Generates fine-tuning datasets from text chunks
pub struct DatasetGenerator;
```

#### Methods

```rust
impl DatasetGenerator {
    /// Create a new generator
    pub fn new(config: DatasetConfig) -> Result<Self>

    /// Generate dataset from chunks
    pub fn generate_from_chunks(&self, chunks: &[Chunk]) -> Result<Dataset>

    /// Split dataset into train/val/test
    pub fn generate_splits(&self, dataset: &Dataset) -> Result<DataSplits>
}
```

---

### `src/dataset/writer.rs`

#### `DatasetWriter`

```rust
/// Writes dataset to JSONL format
pub struct DatasetWriter;
```

#### Methods

```rust
impl DatasetWriter {
    /// Create a new writer
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self>

    /// Write a single chunk
    pub fn write_chunk(&mut self, chunk: &Chunk) -> Result<()>

    /// Write instruction format entries
    pub fn write_instruction(&mut self, entries: &[InstructionEntry]) -> Result<()>

    /// Write Q&A format entries
    pub fn write_qa(&mut self, entries: &[QAEntry]) -> Result<()>

    /// Write chat format entries
    pub fn write_chat(&mut self, entries: &[ChatEntry]) -> Result<()>

    /// Write all splits
    pub fn write_splits(&mut self, splits: &DataSplits) -> Result<()>

    /// Flush buffer to disk
    pub fn flush(&mut self) -> Result<()>
}
```

---

## 6. Configuration Module

### `src/config/loader.rs`

#### `AppConfig`

```rust
/// Application configuration
pub struct AppConfig {
    pub application: ApplicationSettings,
    pub processing: ProcessingSettings,
    pub paths: PathSettings,
    pub output: OutputSettings,
}

impl AppConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self>
}
```

#### `EmbeddingConfig`

```rust
/// Embedding configuration
pub struct EmbeddingConfig {
    pub embedding: EmbeddingSettings,
    pub batch: BatchSettings,
    pub cache: CacheSettings,
}

impl EmbeddingConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self>
}
```

#### `QdrantConfig`

```rust
/// Qdrant configuration
pub struct QdrantConfig {
    pub qdrant: QdrantSettings,
    pub collection: CollectionSettings,
    pub indexing: IndexingSettings,
}

impl QdrantConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self>
}
```

---

## 7. Types Module

### `src/types/metadata.rs`

```rust
/// Shared metadata structure
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_type: String,
    pub tags: Vec<String>,
}
```

---

## 8. Utilities Module

### `src/utils/errors.rs`

```rust
/// Unified error types
pub enum AppError {
    Io(std::io::Error),
    Config(String),
    Embedding(String),
    Vectorstore(String),
    PdfProcessing(String),
    Ingestion(String),
    Dataset(String),
    Unknown(String),
}
```

### `src/utils/fs.rs`

```rust
/// File system utilities
pub struct FileUtils;

impl FileUtils {
    pub fn find_pdfs<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>>
    pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()>
    pub fn filename<P: AsRef<Path>>(path: P) -> String
}
```

### `src/utils/ids.rs`

```rust
/// ID generation utilities
pub struct IdGenerator;

impl IdGenerator {
    pub fn document_id() -> String
    pub fn chunk_id() -> String
    pub fn batch_id() -> String
}
```
