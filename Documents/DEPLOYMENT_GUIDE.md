# Deployment Guide

## Table of Contents

1. [System Requirements](#1-system-requirements)
2. [Installation](#2-installation)
3. [Qdrant Setup](#3-qdrant-setup)
4. [Model Downloads](#4-model-downloads)
5. [Configuration](#5-configuration)
6. [Docker Deployment](#6-docker-deployment)
7. [Performance Tuning](#7-performance-tuning)
8. [Monitoring](#8-monitoring)
9. [Troubleshooting](#9-troubleshooting)

---

## 1. System Requirements

### Minimum Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16+ GB |
| Storage | 50 GB | 200+ GB SSD |
| OS | Linux, macOS, Windows | Linux (Ubuntu 22.04+) |

### Software Requirements

| Software | Version |
|----------|---------|
| Rust | 1.70+ |
| Docker | 24+ (for Qdrant) |
| Qdrant | 1.7+ |

### For OCR (Scanned Documents)

| Component | Requirement |
|-----------|-------------|
| Additional RAM | +4 GB |
| Processing Time | ~2-5 seconds per page |

### GPU (Optional)

| GPU | Benefit |
|-----|---------|
| NVIDIA CUDA | 10x faster embedding inference |
| Apple Silicon | 5x faster via Metal |

---

## 2. Installation

### 2.1 Clone Repository

```bash
git clone https://github.com/yourusername/RAG-RUST-EDITION-.git
cd RAG-RUST-EDITION-
```

### 2.2 Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
```

### 2.3 Configure Environment

```bash
cp .env.example .env
# Edit .env with your settings
```

### 2.4 Create Data Directories

```bash
mkdir -p data/raw/books
mkdir -p data/processed/{text,jsonl,metadata}
mkdir -p data/indexes/embedding_cache
mkdir -p data/models
```

### 2.5 Build

```bash
cargo build --release
```

---

## 3. Qdrant Setup

### 3.1 Docker (Recommended)

```bash
docker run -d \
  --name qdrant \
  -p 6333:6333 \
  -p 6334:6334 \
  -v $(pwd)/qdrant_storage:/qdrant/storage \
  qdrant/qdrant:v1.7.4
```

### 3.2 Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  qdrant:
    image: qdrant/qdrant:v1.7.4
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant_data:/qdrant/storage
    environment:
      - QDRANT__SERVICE__GRPC_PORT=6334
    restart: unless-stopped

volumes:
  qdrant_data:
```

```bash
docker-compose up -d
```

### 3.3 Verify Qdrant

```bash
curl http://localhost:6333/health
# Should return: {"title":"qdrant - vector search engine","version":"1.7.4"}
```

### 3.4 Qdrant Configuration

Update `configs/qdrant.toml`:

```toml
[qdrant]
url = "http://localhost:6333"
api_key = ""  # Set if using Qdrant Cloud
timeout_secs = 30

[collection]
name = "book_documents"
vector_size = 384
distance_metric = "Cosine"
on_disk = true  # Store vectors on disk for large datasets

[indexing]
hnsw_m = 16
hnsw_ef_construct = 200
hnsw_ef_search = 200
```

---

## 4. Model Downloads

### 4.1 Download Embedding Model

```bash
# Using the built-in downloader
cargo run --bin download_model

# Or manually download all-MiniLM-L6-v2
mkdir -p data/models/all-MiniLM-L6-v2
cd data/models/all-MiniLM-L6-v2

# Download model files
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.safetensors
```

### 4.2 Download Reranker Model (Optional)

```bash
mkdir -p data/models/bge-reranker-v2
cd data/models/bge-reranker-v2

wget https://huggingface.co/BAAI/bge-reranker-v2-m3/resolve/main/config.json
wget https://huggingface.co/BAAI/bge-reranker-v2-m3/resolve/main/tokenizer.json
wget https://huggingface.co/BAAI/bge-reranker-v2-m3/resolve/main/model.safetensors
```

### 4.3 Update Configuration

Update `configs/embedding.toml`:

```toml
[embedding]
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
model_path = "./data/models/all-MiniLM-L6-v2"
use_gpu = false  # Set true if CUDA available
```

---

## 5. Configuration

### 5.1 Configuration Files

| File | Purpose |
|------|---------|
| `configs/app.toml` | General application settings |
| `configs/embedding.toml` | Embedding model configuration |
| `configs/qdrant.toml` | Vector database settings |
| `configs/dataset.toml` | Dataset generation settings |
| `.env` | Environment variables (API keys, secrets) |

### 5.2 Environment Variables

```bash
# .env
RUST_LOG=info
EMBEDDING_API_KEY=  # Only if using API-based embeddings
QDRANT_API_KEY=     # Only if using Qdrant Cloud
```

### 5.3 Production Configuration

For production deployment, adjust these settings:

**app.toml**:
```toml
[application]
environment = "production"
log_level = "warn"

[processing]
max_workers = 8      # Match CPU cores
batch_size = 64      # Larger batches for throughput
```

**qdrant.toml**:
```toml
[indexing]
hnsw_ef_search = 128  # Lower for faster search (trade-off: recall)
```

---

## 6. Docker Deployment

### 6.1 Full Stack Docker Compose

```yaml
version: '3.8'

services:
  rag-app:
    build:
      context: .
      dockerfile: Dockerfile
    depends_on:
      - qdrant
    volumes:
      - ./data:/app/data
      - ./configs:/app/configs
    environment:
      - RUST_LOG=info
      - QDRANT_URL=http://qdrant:6333
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant:v1.7.4
    ports:
      - "6333:6333"
    volumes:
      - qdrant_data:/qdrant/storage
    restart: unless-stopped

volumes:
  qdrant_data:
```

### 6.2 Dockerfile

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY config/ config/
COPY configs/ configs/
COPY scripts/ scripts/
COPY dataset/ dataset/
COPY embedding/ embedding/

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rag-pdf-dataset /app/
COPY --from=builder /app/target/release/ingest /app/
COPY --from=builder /app/target/release/rebuild_index /app/
COPY configs/ /app/configs/

CMD ["/app/rag-pdf-dataset"]
```

### 6.3 Build and Run

```bash
docker-compose build
docker-compose up -d
```

---

## 7. Performance Tuning

### 7.1 Embedding Performance

| Setting | Impact |
|---------|--------|
| `use_gpu = true` | 10x faster with NVIDIA GPU |
| `batch_size = 64` | Better throughput |
| `max_workers = 8` | Parallel processing |

### 7.2 Chunking Performance

| Setting | Impact |
|---------|--------|
| `chunk_size = 512` | Best accuracy (2026 data) |
| `overlap = 64` | 12.5% overlap recommended |
| `min_chunk_size = 50` | Filter noise |

### 7.3 Vector Search Performance

| Setting | Impact |
|---------|--------|
| `hnsw_ef_search = 64` | Fast but lower recall |
| `hnsw_ef_search = 200` | Balanced |
| `hnsw_ef_search = 512` | High recall, slower |

### 7.4 Memory Usage

| Component | Memory Usage |
|-----------|-------------|
| Embedding model | ~500 MB |
| Qdrant (1M vectors) | ~1.5 GB |
| Processing pipeline | ~2 GB |

### 7.5 Processing Speed Estimates

| Input | Processing Time |
|-------|----------------|
| 1 PDF (200 pages) | ~30 seconds |
| 1 EPUB (500 pages) | ~45 seconds |
| 1000 chunks to embed | ~5 seconds (CPU) |
| 1000 chunks to embed | ~0.5 seconds (GPU) |

---

## 8. Monitoring

### 8.1 Logging

Configure logging level in `.env`:

```bash
RUST_LOG=info      # Production
RUST_LOG=debug     # Development
RUST_LOG=trace     # Debugging
```

### 8.2 Qdrant Metrics

Access Qdrant dashboard:
```
http://localhost:6333/dashboard
```

API endpoints:
```bash
# Collection info
curl http://localhost:6333/collections/book_documents

# Cluster info
curl http://localhost:6333/cluster
```

### 8.3 Health Checks

```bash
# Application health
cargo run --bin health_check

# Qdrant health
curl http://localhost:6333/health
```

---

## 9. Troubleshooting

### 9.1 Common Issues

**Qdrant connection refused**:
```bash
# Check if Qdrant is running
docker ps | grep qdrant

# Check logs
docker logs qdrant

# Restart
docker restart qdrant
```

**Model download fails**:
```bash
# Check internet connection
curl -I https://huggingface.co

# Manual download
wget -c https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.safetensors
```

**Out of memory during embedding**:
```toml
# Reduce batch size in configs/embedding.toml
[batch]
size = 16  # Default: 32
```

**Slow PDF extraction**:
```bash
# Check PDF size
ls -lh data/raw/books/

# Large PDFs (>100MB) may take several minutes
```

### 9.2 Debug Mode

```bash
RUST_LOG=debug cargo run -- ingest --path data/raw/books/
```

### 9.3 Reset Everything

```bash
# Clear Qdrant data
docker stop qdrant
docker rm qdrant
rm -rf qdrant_storage/

# Clear processed data
rm -rf data/processed/*
rm -rf data/indexes/*

# Restart
docker run -d --name qdrant -p 6333:6333 qdrant/qdrant:v1.7.4
```

### 9.4 Support

- GitHub Issues: https://github.com/yourusername/RAG-RUST-EDITION-/issues
- Documentation: `/Documents/` directory
