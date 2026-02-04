# RAG PDF Dataset

A Rust-based Retrieval-Augmented Generation (RAG) system for processing, embedding, and retrieving information from PDF documents.

## Project Structure

- **configs/** - Configuration files for application, embeddings, and vector store
- **data/** - Data storage (raw PDFs, processed text/JSONL, vector indexes)
- **src/** - Main source code
  - **config/** - Configuration loading and management
  - **ingestion/** - PDF loading, text cleaning, chunking, and tokenization
  - **dataset/** - Dataset schema and output formatting
  - **embedding/** - Embedding API client and batch processing
  - **vectorstore/** - Vector store integration (Qdrant)
  - **rag/** - RAG retrieval and prompt building
  - **utils/** - Utility functions and error handling
  - **types/** - Shared type definitions
- **scripts/** - Executable scripts for ingestion and index rebuilding
- **tests/** - Test suite

## Prerequisites

- Rust 1.70+
- Qdrant instance running on `localhost:6333`
- Embedding service (optional, can use Hugging Face API)

## Setup

1. Clone and enter the project:
   ```bash
   cd rag-pdf-dataset
   ```

2. Configure environment:
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

3. Create data directories:
   ```bash
   mkdir -p data/raw/pdfs data/processed/{text,jsonl} data/indexes
   ```

4. Build:
   ```bash
   cargo build --release
   ```

## Usage

### Ingest PDFs

```bash
cargo run --bin ingest -- path/to/pdfs/
```

### Rebuild Index

```bash
cargo run --bin rebuild_index
```

## Configuration

Configuration files in `configs/`:
- `app.toml` - Application settings
- `embedding.toml` - Embedding model configuration
- `qdrant.toml` - Vector store configuration

## Testing

```bash
cargo test
```

## License

MIT
