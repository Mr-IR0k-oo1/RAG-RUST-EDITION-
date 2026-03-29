# System Architecture

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Data Flow Diagrams](#2-data-flow-diagrams)
3. [Module Responsibilities](#3-module-responsibilities)
4. [Configuration System](#4-configuration-system)
5. [Data Storage Layout](#5-data-storage-layout)
6. [Dependencies and Technology Stack](#6-dependencies-and-technology-stack)

---

## 1. System Overview

This project is a production-grade RAG system built in Rust that:
1. Extracts text from books (PDF, EPUB, scanned documents)
2. Chunks and embeds the text into a vector database
3. Enables semantic search and retrieval
4. Generates fine-tuning datasets in multiple formats

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        RAG PDF Dataset                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Ingestion   │    │   Embedding   │    │  Vector DB   │      │
│  │   Pipeline    │───>│   Pipeline    │───>│  (Qdrant)    │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│         │                                        │              │
│         v                                        v              │
│  ┌──────────────┐                        ┌──────────────┐      │
│  │   Dataset     │                        │    RAG       │      │
│  │   Generator   │                        │  Retriever   │      │
│  └──────────────┘                        └──────────────┘      │
│         │                                        │              │
│         v                                        v              │
│  ┌──────────────┐                        ┌──────────────┐      │
│  │  Fine-Tuning  │                        │   Prompt     │      │
│  │   Datasets    │                        │   Builder    │      │
│  └──────────────┘                        └──────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Data Flow Diagrams

### 2.1 Ingestion Flow (Book -> Vector DB)

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  Book    │────>│ Loader  │────>│ Cleaner │────>│ Chunker │
│  Files   │     │         │     │         │     │         │
└─────────┘     └─────────┘     └─────────┘     └─────────┘
  PDF/EPUB        Extract         Normalize       Split into
  Scanned         text            unicode,        512-token
                  Handle          fix spaces,     chunks with
                  structure       remove          12.5%
                                  artifacts       overlap
                                                      │
                                                      v
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│ Qdrant   │<────│ Index   │<────│ Batch   │<────│ Embed   │
│ Vector   │     │ Insert  │     │ Process │     │ Chunks  │
│ Store    │     │         │     │         │     │         │
└─────────┘     └─────────┘     └─────────┘     └─────────┘
                  Store           Group by        Convert
                  vectors +       batch size      text to
                  metadata        (32)            384-dim
                                                  vectors
```

### 2.2 Query Flow (User Query -> Answer)

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  User    │────>│ Embed   │────>│ Vector  │────>│ Re-rank │
│  Query   │     │ Query   │     │ Search  │     │ Results │
└─────────┘     └─────────┘     └─────────┘     └─────────┘
                  384-dim         Top-20          Cross-
                  vector          candidates      encoder
                  (5ms)           (1ms)           (80ms)
                                                      │
                                                      v
┌─────────┐     ┌─────────┐     ┌─────────┐
│  Final   │<────│  LLM    │<────│ Prompt  │
│  Answer  │     │ Generate│     │ Build   │
└─────────┘     └─────────┘     └─────────┘
                  Grounded        Inject top-5
                  response        chunks into
                  with sources    context
```

### 2.3 Dataset Generation Flow (Chunks -> Fine-Tuning Data)

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  Text    │────>│ Q&A     │────>│ Format  │────>│ JSONL   │
│  Chunks  │     │ Generate│     │ Convert │     │ Output  │
└─────────┘     └─────────┘     └─────────┘     └─────────┘
                  Generate        Convert to:     Write to:
                  question-       - Instruction   - instruction.jsonl
                  answer pairs    - Q&A           - qa.jsonl
                  from chunks     - Chat          - chat.jsonl
```

---

## 3. Module Responsibilities

### 3.1 `src/ingestion/` - Document Processing

| Module | Responsibility |
|--------|---------------|
| `book_loader.rs` | Load PDF, EPUB, scanned docs. Extract text with structure preservation |
| `cleaner.rs` | Unicode normalization, whitespace cleanup, artifact removal |
| `chunker.rs` | Split text into overlapping chunks with sentence awareness |
| `tokenizer.rs` | Token counting and sentence detection |
| `ocr.rs` | OCR processing for scanned documents |

### 3.2 `src/embedding/` - Vector Generation

| Module | Responsibility |
|--------|---------------|
| `client.rs` | Local inference using Candle + sentence-transformers |
| `batcher.rs` | Batch processing with retry logic |
| `cache.rs` | Embedding cache to avoid re-computation |

### 3.3 `src/vectorstore/` - Vector Database

| Module | Responsibility |
|--------|---------------|
| `qdrant.rs` | Qdrant client: insert, search, delete, recreate |
| `schema.rs` | Collection schema and payload definitions |

### 3.4 `src/rag/` - Retrieval and Generation

| Module | Responsibility |
|--------|---------------|
| `retriever.rs` | Query embedding + vector search + re-ranking |
| `prompt.rs` | Context injection into LLM prompts |
| `reranker.rs` | Cross-encoder re-ranking of search results |

### 3.5 `src/dataset/` - Fine-Tuning Data

| Module | Responsibility |
|--------|---------------|
| `generator.rs` | Generate Q&A pairs from chunks |
| `formatter.rs` | Convert to instruction/QA/chat formats |
| `writer.rs` | Write JSONL output files |
| `validator.rs` | Validate dataset quality |

### 3.6 `src/config/` - Configuration

| Module | Responsibility |
|--------|---------------|
| `loader.rs` | Load TOML config files |

### 3.7 `src/types/` - Shared Types

| Module | Responsibility |
|--------|---------------|
| `metadata.rs` | Document and chunk metadata structures |

### 3.8 `src/utils/` - Utilities

| Module | Responsibility |
|--------|---------------|
| `errors.rs` | Unified error types |
| `fs.rs` | File system operations |
| `ids.rs` | UUID generation |

---

## 4. Configuration System

### 4.1 Configuration Files

```
configs/
├── app.toml          # Application settings
├── embedding.toml    # Embedding model configuration
├── qdrant.toml       # Vector store configuration
└── dataset.toml      # Dataset generation settings (NEW)
```

### 4.2 Configuration Hierarchy

```
Environment Variables (highest priority)
    |
    v
TOML Config Files
    |
    v
Default Values (lowest priority)
```

### 4.3 Configuration Schemas

**app.toml**:
```toml
[application]
name = "rag-pdf-dataset"
version = "0.2.0"
environment = "production"
log_level = "info"

[processing]
chunk_size = 512
chunk_overlap = 64          # 12.5% of chunk_size
min_chunk_size = 50
max_tokens = 1024
remove_stop_words = false
max_workers = 4
batch_size = 32

[paths]
raw_data_dir = "./data/raw"
processed_data_dir = "./data/processed"
indexes_dir = "./data/indexes"
config_dir = "./configs"

[output]
format = "jsonl"
compression = false
include_metadata = true
```

**embedding.toml**:
```toml
[embedding]
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
model_path = "./models/all-MiniLM-L6-v2"
use_gpu = false

[batch]
size = 32
max_retries = 3
retry_delay_ms = 1000

[cache]
enabled = true
max_size_mb = 1024
ttl_secs = 86400
```

**qdrant.toml**:
```toml
[qdrant]
url = "http://localhost:6333"
api_key = ""
timeout_secs = 30

[collection]
name = "book_documents"
vector_size = 384
distance_metric = "Cosine"
on_disk = true

[indexing]
hnsw_m = 16
hnsw_ef_construct = 200
hnsw_ef_search = 200
```

**dataset.toml** (NEW):
```toml
[generation]
num_qa_pairs_per_chunk = 3
min_answer_length = 20
max_answer_length = 500
question_types = ["factual", "inferential", "comparative"]

[formats]
instruction = true
qa = true
chat = true

[quality]
min_similarity_score = 0.7
validate_answers = true
deduplicate = true

[splits]
train = 0.8
validation = 0.1
test = 0.1
```

---

## 5. Data Storage Layout

```
data/
├── raw/
│   └── books/
│       ├── book1.pdf
│       ├── book2.epub
│       └── scanned_book/
│           ├── page_001.png
│           └── page_002.png
├── processed/
│   ├── text/
│   │   ├── book1.txt
│   │   └── book2.txt
│   ├── jsonl/
│   │   ├── instruction.jsonl    # Fine-tuning: instruction format
│   │   ├── qa.jsonl             # Fine-tuning: Q&A format
│   │   ├── chat.jsonl           # Fine-tuning: chat format
│   │   └── chunks.jsonl         # Raw chunks for indexing
│   └── metadata/
│       ├── book1.json           # Book metadata
│       └── book2.json
├── indexes/
│   └── embedding_cache/         # Cached embeddings
└── models/
    └── all-MiniLM-L6-v2/        # Local embedding model
```

---

## 6. Dependencies and Technology Stack

### 6.1 Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.35 | Async runtime |
| serde | 1.0 | Serialization |
| serde_json | 1.0 | JSON handling |
| toml | 0.8 | Config parsing |
| anyhow | 1.0 | Error handling |
| thiserror | 1.0 | Error types |
| log | 0.4 | Logging |
| env_logger | 0.11 | Log output |

### 6.2 Document Processing

| Crate | Version | Purpose |
|-------|---------|---------|
| pdfium-render | 0.8 | PDF text extraction |
| pdf_oxide | 0.3 | Fast PDF parsing |
| epub | 2.1 | EPUB book loading |
| ocrs | 0.8 | OCR for scanned docs |
| unicode-normalization | 0.1 | Unicode handling |
| regex | 1.10 | Text pattern matching |

### 6.3 ML/Embedding

| Crate | Version | Purpose |
|-------|---------|---------|
| candle-core | 0.6 | ML tensor operations |
| candle-nn | 0.6 | Neural network layers |
| candle-transformers | 0.6 | Transformer models |

### 6.4 Vector Database

| Crate | Version | Purpose |
|-------|---------|---------|
| qdrant-client | 1.7 | Qdrant vector DB client |

### 6.5 CLI and Utilities

| Crate | Version | Purpose |
|-------|---------|---------|
| clap | 4.x | Command line parsing |
| indicatif | 0.17 | Progress bars |
| uuid | 1.6 | ID generation |
| chrono | 0.4 | Timestamps |
| futures | 0.3 | Async utilities |
| reqwest | 0.11 | HTTP client |
