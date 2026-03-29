# RAG Fundamentals - Complete Guide

## Table of Contents

1. [What is RAG](#1-what-is-rag)
2. [Why RAG vs Pure Fine-Tuning](#2-why-rag-vs-pure-fine-tuning)
3. [Core Components](#3-core-components)
4. [Embedding Models](#4-embedding-models)
5. [Chunking Strategies](#5-chunking-strategies)
6. [Vector Similarity Search](#6-vector-similarity-search)
7. [Re-ranking](#7-re-ranking)
8. [Agentic RAG](#8-agentic-rag)
9. [RAG Evaluation Metrics](#9-rag-evaluation-metrics)
10. [Production Considerations](#10-production-considerations)

---

## 1. What is RAG

Retrieval-Augmented Generation (RAG) is an architecture pattern that enhances Large Language Models (LLMs) by providing them with external knowledge at inference time. Instead of relying solely on the model's training data, RAG retrieves relevant documents from a knowledge base and injects them into the prompt context.

### The Problem RAG Solves

LLMs have three fundamental limitations:

1. **Knowledge cutoff**: Models only know information from their training data
2. **Hallucination**: Models generate plausible but factually incorrect information
3. **No source attribution**: Models cannot cite where information comes from

RAG addresses all three by grounding model responses in retrieved, verifiable source documents.

### How RAG Works

```
User Query
    |
    v
[Embed Query] --> [Vector Search] --> [Retrieve Top-K Chunks]
    |                                      |
    |                                      v
    +-------------------------------> [Build Prompt with Context]
                                            |
                                            v
                                    [Generate Response via LLM]
                                            |
                                            v
                                    [Return Answer + Sources]
```

### Simple vs Complex RAG

**Simple RAG (Naive RAG)**:
- Single retrieval pass
- Fixed top-K selection
- Direct context injection
- Works for straightforward Q&A

**Advanced RAG**:
- Query expansion/reformulation
- Hybrid search (dense + sparse)
- Re-ranking retrieved results
- Multi-step retrieval
- Contextual compression

**Agentic RAG (2026 Standard)**:
- LLM agent plans retrieval strategy
- Dynamic tool selection
- Result reflection and retry
- Multi-source synthesis

---

## 2. Why RAG vs Pure Fine-Tuning

### Comparison Table

| Aspect | RAG | Fine-Tuning |
|--------|-----|-------------|
| Knowledge update | Real-time (update database) | Requires retraining |
| Cost | Low (embedding + search) | High (GPU hours) |
| Traceability | Can cite sources | Cannot explain why |
| Domain adaptation | Add documents | Train on examples |
| Response quality | Grounded in facts | Can hallucinate |
| Latency | Higher (retrieval step) | Lower (direct generation) |
| Data privacy | Data stays in your infra | Data used in training |

### When to Use RAG

- Knowledge base changes frequently
- You need source attribution
- Data cannot leave your infrastructure
- You want to reduce hallucination
- Cost of retraining is prohibitive
- You need to combine multiple knowledge sources

### When to Use Fine-Tuning

- You need specific output style/format
- Task requires specialized reasoning
- Domain has unique vocabulary/patterns
- Latency is critical (no retrieval overhead)
- You have high-quality training data

### The Hybrid Approach (Recommended)

Use both together:
1. **RAG** for factual grounding and source retrieval
2. **Fine-tuning** for output style, format, and domain-specific reasoning

This project generates fine-tuning datasets FROM RAG-retrieved book content, giving you the best of both worlds.

---

## 3. Core Components

### 3.1 Document Loader

Responsible for ingesting raw documents from various sources.

**Supported formats in this project**:
- PDF (native text extraction)
- EPUB (e-book format)
- Scanned documents (OCR processing)

**Key considerations**:
- Preserve document structure (chapters, sections, headings)
- Extract metadata (title, author, page numbers)
- Handle encoding issues (UTF-8 normalization)
- Detect and handle tables, images, footnotes

### 3.2 Text Cleaner

Normalizes extracted text for consistent processing:

```
Raw Text --> Unicode Normalization --> Whitespace Cleanup -->
Remove Artifacts --> Fix Hyphenation --> Normalize Quotes/Dashes
```

### 3.3 Text Chunker

Splits documents into semantically meaningful segments:

```
Document --> [Chunk 1] [Chunk 2] [Chunk 3] ... [Chunk N]
             (with overlap between consecutive chunks)
```

**Why chunking matters**:
- Embedding models have token limits (256-8192 tokens)
- Smaller chunks = more precise retrieval
- Overlap prevents information loss at boundaries
- Chunk size affects both recall and precision

### 3.4 Embedding Model

Converts text chunks into dense vector representations:

```
"The cat sat on the mat" --> [0.12, -0.34, 0.56, ..., 0.78] (384 dimensions)
```

**Properties of good embeddings**:
- Semantically similar text produces similar vectors
- Dimensionality captures meaningful features
- Distance in vector space correlates with semantic similarity

### 3.5 Vector Store

Database optimized for similarity search over high-dimensional vectors:

```
Vector Store (Qdrant)
├── Collection: "pdf_documents"
│   ├── Point 1: {id, vector[384], payload{content, metadata}}
│   ├── Point 2: {id, vector[384], payload{content, metadata}}
│   └── ...
```

**Why not a regular database?**
- Brute-force search over millions of vectors is O(n)
- Vector stores use approximate nearest neighbor (ANN) algorithms
- HNSW index provides O(log n) search with ~95% recall

### 3.6 Retriever

Orchestrates the search process:

```
Query --> Embed Query --> Search Vector Store --> Return Top-K Results
```

### 3.7 Prompt Builder

Injects retrieved context into the LLM prompt:

```
System: You are a helpful assistant. Answer based on the provided context.

Context:
[Chunk 1 content]
[Chunk 2 content]
[Chunk 3 content]

Question: {user_query}

Answer the question using only the provided context. If the context doesn't
contain enough information, say so.
```

### 3.8 Generator (LLM)

Produces the final response grounded in retrieved context.

---

## 4. Embedding Models

### 4.1 How Embeddings Work

Embedding models are neural networks trained to map text into vector spaces where semantic similarity corresponds to vector proximity.

**Training objectives**:
- Contrastive learning: Pull similar text together, push dissimilar apart
- Masked language modeling: Predict masked tokens from context
- Sentence similarity: Match question-answer pairs

### 4.2 Model Comparison (2026)

| Model | Dimensions | Context | Cost | Quality |
|-------|-----------|---------|------|---------|
| all-MiniLM-L6-v2 | 384 | 256 tokens | Free (local) | Good |
| all-mpnet-base-v2 | 768 | 384 tokens | Free (local) | Better |
| text-embedding-3-small | 1536 | 8191 tokens | $0.02/M | Excellent |
| text-embedding-3-large | 3072 | 8191 tokens | $0.13/M | Best |
| voyage-3 | 1024 | 32K tokens | $0.06/M | Excellent |
| Cohere embed-v3 | 1024 | 512 tokens | $0.10/M | Excellent |

### 4.3 Local Embedding with Candle (This Project)

We use `candle` (Rust ML framework) with sentence-transformers models:

**Advantages**:
- Zero API costs
- No data leaves your infrastructure
- Low latency (~5ms per chunk on CPU)
- Works offline

**Model choice**: `all-MiniLM-L6-v2`
- 384 dimensions (good balance of quality/memory)
- 256 token limit per text (matches our chunk size)
- ~80MB model size
- Fast inference on CPU

### 4.4 Embedding Best Practices

1. **Normalize vectors**: Unit vectors enable cosine similarity via dot product
2. **Batch processing**: Process multiple texts together for GPU efficiency
3. **Caching**: Cache embeddings for unchanged content
4. **Dimension matching**: Query and document embeddings must use the same model

---

## 5. Chunking Strategies

### 5.1 Fixed-Size Chunking

Split text every N tokens with optional overlap.

```
[---Chunk 1---]
     [---Chunk 2---]
          [---Chunk 3---]
```

**Pros**: Simple, predictable chunk count
**Cons**: Breaks sentences, loses context at boundaries

### 5.2 Recursive Character Splitting (Recommended)

Split on hierarchical separators: paragraphs > sentences > words > characters.

```
1. Try to split on "\n\n" (paragraphs)
2. If chunk still too large, split on "\n" (lines)
3. If still too large, split on ". " (sentences)
4. If still too large, split on " " (words)
```

**2026 research finding**: 512 tokens with 10-15% overlap achieves highest answer accuracy consistently.

### 5.3 Sentence-Aware Chunking

Split at sentence boundaries, accumulate until reaching target size.

```
Sentence 1 + Sentence 2 + Sentence 3 = ~512 tokens --> Chunk
Sentence 4 + Sentence 5 + Sentence 6 = ~512 tokens --> Chunk
```

**Pros**: Preserves sentence integrity
**Cons**: Variable chunk sizes

### 5.4 Semantic Chunking

Use embeddings to detect topic boundaries, split where semantic shift occurs.

```
[Topic A sentences] | [Topic B sentences] | [Topic C sentences]
```

**Pros**: Chunks are topically coherent
**Cons**: 3-5x more fragments, higher embedding cost, complex implementation

### 5.5 Document-Structure Chunking (For Books)

Leverage document structure for intelligent splitting:

```
Book
├── Chapter 1: Introduction
│   ├── Section 1.1: Background
│   ├── Section 1.2: Motivation
│   └── Section 1.3: Overview
├── Chapter 2: Methods
│   ├── Section 2.1: Approach A
│   └── Section 2.2: Approach B
└── ...
```

**This project uses**: Recursive character splitting at 512 tokens with 12.5% overlap, enhanced with sentence-boundary awareness and chapter/section detection for books.

### 5.6 Chunk Size Impact

| Chunk Size | Retrieval Precision | Context Coverage | Token Cost |
|------------|---------------------|------------------|------------|
| 128 tokens | High | Low (more chunks needed) | High |
| 256 tokens | Good | Moderate | Moderate |
| 512 tokens | Best (2026 data) | Good | Balanced |
| 1024 tokens | Lower | High | Low |

---

## 6. Vector Similarity Search

### 6.1 Distance Metrics

**Cosine Similarity** (most common for text):
```
similarity = (A · B) / (||A|| * ||B||)
Range: [-1, 1] where 1 = identical direction
```

**Dot Product**:
```
similarity = A · B = Σ(a_i * b_i)
Works well with normalized vectors (equivalent to cosine)
```

**Euclidean Distance**:
```
distance = √(Σ(a_i - b_i)²)
Lower distance = more similar
```

### 6.2 Approximate Nearest Neighbor (ANN)

Exact nearest neighbor search is O(n*d) where n = vectors, d = dimensions. For millions of vectors, this is too slow.

**HNSW (Hierarchical Navigable Small World)**:
- Graph-based index
- O(log n) search time
- ~95% recall rate
- Used by Qdrant, Pinecone, Weaviate

**Parameters**:
- `m`: Number of connections per node (higher = better recall, more memory)
- `ef_construct`: Search width during index construction (higher = better quality)
- `ef_search`: Search width during queries (higher = better recall, slower search)

### 6.3 Qdrant Architecture

```
Qdrant Instance
├── Collection: "pdf_documents"
│   ├── Config: {vector_size: 384, distance: Cosine}
│   ├── Payload Index
│   │   ├── source: keyword
│   │   ├── chapter: keyword
│   │   ├── page: integer
│   │   └── token_count: integer
│   └── Vectors
│       ├── Point 0: [0.12, -0.34, ...]
│       ├── Point 1: [0.56, 0.78, ...]
│       └── ...
```

---

## 7. Re-ranking

### 7.1 What is Re-ranking

Re-ranking is a two-stage retrieval process:

1. **Stage 1 (Retriever)**: Fast approximate search returns 20-50 candidates
2. **Stage 2 (Re-ranker)**: Precise cross-encoder scores each candidate, returns top 3-5

### 7.2 Why Re-ranking Works

Bi-encoder (embedding model) computes vectors independently:
```
Query --> Vector Q
Doc A --> Vector A  (computed separately)
Doc B --> Vector B
```

Cross-encoder (re-ranker) processes query and document together:
```
[Query, Doc A] --> Score A  (joint attention)
[Query, Doc B] --> Score B
```

Cross-encoders capture fine-grained interactions that bi-encoders miss.

### 7.3 Performance Impact

- **Precision improvement**: 18-42% over retrieval without re-ranking
- **Latency cost**: 50-200ms per batch
- **Cost tradeoff**: Re-ranker cost < LLM cost savings from fewer, better chunks

### 7.4 Available Re-rankers (2026)

| Model | Type | Cost | Latency |
|-------|------|------|---------|
| Cohere Rerank 3 | API | $1/1K queries | ~100ms |
| BGE-Reranker-v2 | Open source (local) | Free | ~80ms |
| bge-reranker-large | Open source (local) | Free | ~150ms |
| Voyage Rerank-2 | API | $0.06/1K queries | ~90ms |

### 7.5 This Project's Approach

Use `BGE-Reranker-v2` via Candle for fully local re-ranking:
- No API costs
- No data leaves infrastructure
- ~80ms per batch of 20 candidates

---

## 8. Agentic RAG

### 8.1 Simple RAG Limitations

Simple RAG fails when:
- Query requires information from multiple documents
- Query needs comparison or synthesis
- First retrieval attempt returns irrelevant results
- Query is ambiguous and needs clarification

### 8.2 Agentic RAG Architecture

```
User Query
    |
    v
[Agent: Plan Retrieval Strategy]
    |
    +---> [Tool: Vector Search]
    +---> [Tool: Keyword Search]
    +---> [Tool: SQL Query]
    +---> [Tool: API Call]
    |
    v
[Agent: Evaluate Results]
    |
    +---> Results sufficient? --> [Generate Response]
    |
    +---> Results insufficient? --> [Reformulate Query] --> [Retry]
```

### 8.3 When to Use Agentic RAG

| Scenario | Simple RAG | Agentic RAG |
|----------|-----------|-------------|
| Single-document Q&A | Yes | Overkill |
| Multi-document synthesis | Struggles | Required |
| Queries requiring comparison | Struggles | Required |
| Latency-sensitive (<1s) | Yes | Too slow |
| Complex reasoning chains | No | Required |

### 8.4 Cost of Agentic RAG

- 1-3 extra LLM calls per query
- Higher latency (2-8 seconds vs <2 seconds)
- More token consumption

---

## 9. RAG Evaluation Metrics

### 9.1 Retrieval Metrics

**Retrieval Precision**: Fraction of retrieved chunks that are relevant
```
Precision = Relevant Retrieved / Total Retrieved
Target: >70% acceptable, >85% good, >95% excellent
```

**Retrieval Recall**: Fraction of all relevant chunks that were retrieved
```
Recall = Relevant Retrieved / Total Relevant in Database
```

**Retrieval F1**: Harmonic mean of precision and recall
```
F1 = 2 * (Precision * Recall) / (Precision + Recall)
```

**Mean Reciprocal Rank (MRR)**: Average of 1/rank of first relevant result
```
MRR = (1/N) * Σ(1/rank_i)
```

### 9.2 Generation Metrics

**Answer Grounding Rate**: Fraction of response supported by retrieved context
```
Grounding = Supported Claims / Total Claims
Target: >90%
```

**Hallucination Rate**: Rate of claims not in retrieved context
```
Hallucination = Unsupported Claims / Total Claims
Target: <5%
```

**Answer Relevance**: How well the answer addresses the query
- Measured by LLM-as-judge (GPT-4, Claude)

### 9.3 End-to-End Metrics

**Latency**: P50 and P95 response times
- Simple RAG: <2 seconds
- Agentic RAG: <8 seconds

**Token Efficiency**: Tokens used per query
- Lower is better (cost savings)
- Typical: 2000-4000 tokens per query

### 9.4 Evaluation Tools

| Tool | Type | Features |
|------|------|----------|
| Ragas | Open source | Comprehensive RAG metrics |
| LangSmith | Platform | Tracing + evaluation |
| Arize Phoenix | Platform | Production monitoring |
| UpTrain | Open source | LLM evaluation |

---

## 10. Production Considerations

### 10.1 Cost Architecture (100K queries/month)

| Component | Cost |
|-----------|------|
| Embedding (queries) | $2-5 |
| Vector DB (1M vectors) | $70/month |
| Re-ranking | $100/month |
| LLM generation | $200-2000/month |

**Key insight**: LLM generation dominates cost. Reducing context tokens directly reduces cost.

### 10.2 Scaling Considerations

**Horizontal scaling**:
- Vector DB: Shard by document collection
- Embedding: Stateless, scale horizontally
- Retrieval: Read replicas for search

**Vertical scaling**:
- Larger machines for embedding inference
- More memory for larger vector indexes

### 10.3 Monitoring

Production RAG requires monitoring:
- Retrieval precision (sample and label)
- Hallucination rate (LLM-as-judge sampling)
- Latency percentiles (P50, P95, P99)
- Token consumption (cost tracking)
- Error rates by component

### 10.4 Data Pipeline

```
Source Documents --> Ingestion Queue --> Processing Workers
                                            |
                                            v
                                    [Extract] --> [Clean] --> [Chunk]
                                            |
                                            v
                                    [Embed] --> [Store in Vector DB]
                                            |
                                            v
                                    [Update Index] --> [Ready for Queries]
```

### 10.5 Security

- API keys in environment variables, never in code
- Qdrant API key authentication
- Rate limiting on embedding API calls
- Input sanitization for user queries
- Access control on document collections
