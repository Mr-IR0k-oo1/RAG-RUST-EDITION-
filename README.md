# RAG PDF Dataset

A Rust-based Retrieval-Augmented Generation (RAG) system for processing, embedding, and retrieving information from PDF and EPUB documents.

## Current Status

> **Status:** Active Development (v0.2.0)  
> **Implementation Progress:** ~70% complete

This project is actively under development with significant recent updates. The following features are implemented or in progress:

| Feature | Status |
|---------|--------|
| PDF text extraction | Done |
| EPUB text extraction | Done |
| Text cleaning & normalization | Done |
| Text chunking (512 tokens, 12.5% overlap) | Done |
| Tokenization | Done |
| Embedding API client (HuggingFace) | Done |
| Local embedding (Candle-based) | In Progress |
| Qdrant vector store integration | Done |
| RAG retrieval & re-ranking | In Progress |
| Dataset generator (instruction/QA/chat formats) | Done |
| CLI infrastructure | Done |
| Comprehensive documentation | Done |
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

- **Multi-format document support**: PDF and EPUB ingestion with full text extraction
- **Advanced text processing**: Intelligent chunking (512 tokens with 12.5% overlap), cleaning, and tokenization
- **Embedding generation**: HuggingFace API integration with batch processing
- **Vector store**: Full Qdrant integration with semantic search capabilities
- **Dataset generation**: Multiple output formats (instruction-tuning, Q&A, chat)
- **Retrieval pipeline**: Advanced RAG retrieval with prompt building
- **CLI-first design**: Modular command-line interface with multiple binaries
- **Comprehensive documentation**: Architecture guides, API references, and deployment guides
- **Async-first architecture**: Built on tokio for high performance
- **Error handling**: Type-safe error handling with custom error types

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

Process PDF and EPUB files to extract text, chunk content, and generate embeddings:

```bash
cargo run --bin ingest -- path/to/documents/
```

### Download Embedding Model

Download or initialize the embedding model for local inference:

```bash
cargo run --bin download_model
```

### Rebuild Vector Index

Rebuild the Qdrant vector index from processed documents:

```bash
cargo run --bin rebuild_index
```

### Main CLI

The main binary provides the core functionality:

```bash
cargo run -- --help
```

## Configuration

Configuration files are located in `configs/`:

- `app.toml` - Application settings
- `embedding.toml` - Embedding model configuration
- `qdrant.toml` - Vector store configuration

## Documentation

Comprehensive documentation is available in the [Documents](./Documents/) folder:

- **[ARCHITECTURE.md](./Documents/ARCHITECTURE.md)** - System architecture and component design
- **[IMPLEMENTATION_PLAN.md](./Documents/IMPLEMENTATION_PLAN.md)** - Detailed implementation roadmap
- **[API_REFERENCE.md](./Documents/API_REFERENCE.md)** - Complete API documentation
- **[RAG_FUNDAMENTALS.md](./Documents/RAG_FUNDAMENTALS.md)** - RAG concepts and theory
- **[FINE_TUNING_DATASETS.md](./Documents/FINE_TUNING_DATASETS.md)** - Dataset formats and generation
- **[DEPLOYMENT_GUIDE.md](./Documents/DEPLOYMENT_GUIDE.md)** - Production deployment instructions

## Contributing

Contributions are welcome! This project is open to:

- Bug fixes and documentation improvements
- New feature implementations (OCR support, additional document formats)
- Local embedding model optimizations (Candle)
- RAG retrieval and re-ranking enhancements
- Performance optimizations and benchmarking
- Test coverage improvements
- Example use cases and tutorials

### How to Contribute

1. **Fork** the repository
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Areas Needing Contribution

- Local embedding model support improvements (Candle-based)
- RAG retrieval and re-ranking optimization
- OCR support for scanned documents
- Additional document format support
- Performance benchmarking and optimization
- Extended test coverage

## Testing

```bash
cargo test
```

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with Rust for performance and reliability.
