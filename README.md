# RAG PDF Dataset

A Rust-based Retrieval-Augmented Generation (RAG) system for processing, embedding, and retrieving information from PDF and EPUB documents.

## Current Status

> **Status:** Early Development (v0.2.0)  
> **Implementation Progress:** ~40% complete

This project is actively under development. The following features are implemented or in progress:

| Feature | Status |
|---------|--------|
| PDF/EPUB text extraction | Partial |
| Text cleaning & normalization | Done |
| Text chunking (512 tokens, 12.5% overlap) | Done |
| Tokenization | Done |
| Embedding API client (HuggingFace) | Done |
| Local embedding (placeholder) | Partial |
| Qdrant vector store integration | In Progress |
| RAG retrieval & re-ranking | Not Started |
| Dataset generator (instruction/QA/chat formats) | Not Started |
| Scanned document OCR support | Not Started |

## Goal

The goal of this project is to build a production-ready RAG pipeline in pure Rust that can:

1. **Ingest documents** from multiple formats (PDF, EPUB, and scanned documents with OCR)
2. **Process text** with intelligent chunking and tokenization
3. **Generate embeddings** using both API-based and local models
4. **Store vectors** efficiently in Qdrant for semantic search
5. **Retrieve relevant context** with re-ranking capabilities
6. **Generate fine-tuning datasets** in multiple formats (instruction-tuning, Q&A, chat)

The project aims to provide a fast, memory-efficient alternative to Python-based RAG systems while maintaining flexibility for various use cases.

## Features

- Multi-format document support (PDF, EPUB)
- Configurable text chunking with overlap
- Batch embedding generation
- Qdrant vector store integration
- Multiple output formats for fine-tuning datasets
- CLI-first design
- Comprehensive logging and error handling

## Project Structure

```
rag-pdf-dataset/
├── configs/           # Configuration files (app, embedding, qdrant)
├── data/              # Data storage
│   ├── raw/pdfs/      # Raw PDF files
│   ├── processed/     # Processed text and JSONL
│   └── indexes/       # Vector indexes
├── src/
│   ├── config/        # Configuration loading
│   ├── ingestion/     # PDF/EPUB loading, text processing
│   ├── dataset/       # Dataset schemas and output
│   ├── embedding/     # Embedding client
│   ├── vectorstore/   # Qdrant integration
│   ├── rag/           # Retrieval and prompt building
│   ├── utils/         # Utilities and error handling
│   └── types/         # Shared type definitions
├── scripts/           # Executable scripts
├── tests/             # Test suite
└── Documents/          # Architecture and deployment docs
```

## Prerequisites

- **Rust 1.70+**
- **Qdrant** (for vector storage)
- **Embedding service** (HuggingFace API) or local model support (planned)

## Setup

```bash
# Clone the repository
git clone https://github.com/irok/rag-rust-edition.git
cd rag-rust-edition

# Copy environment template and configure
cp .env.example .env
# Edit .env with your settings

# Create data directories
mkdir -p data/raw/pdfs data/processed/{text,jsonl} data/indexes

# Build
cargo build --release
```

## Usage

### Ingest Documents

```bash
cargo run --bin ingest -- path/to/documents/
```

### Query RAG System

```bash
cargo run --bin query -- "Your question here"
```

### Generate Fine-tuning Dataset

```bash
cargo run --bin generate
```

### Rebuild Vector Index

```bash
cargo run --bin rebuild_index
```

### View Statistics

```bash
cargo run --bin stats
```

## Configuration

Configuration files are located in `configs/`:

- `app.toml` - Application settings
- `embedding.toml` - Embedding model configuration
- `qdrant.toml` - Vector store configuration

## Contributing

Contributions are welcome! This project is open to:

- Bug fixes and documentation improvements
- New feature implementations (OCR support, additional document formats)
- Performance optimizations
- Test coverage improvements
- Example use cases and tutorials

### How to Contribute

1. **Fork** the repository
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Areas Needing Contribution

- Qdrant integration completion
- Local embedding model support (candle-based)
- OCR support for scanned documents
- RAG retrieval and re-ranking implementation
- Dataset generator for fine-tuning
- Additional document format support

Please read the [Documents](./Documents/) folder for architecture details and implementation plans.

## Testing

```bash
cargo test
```

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with Rust for performance and reliability.
