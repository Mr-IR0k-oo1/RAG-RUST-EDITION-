# Fine-Tuning Datasets - Formats and Generation

## Table of Contents

1. [Overview](#1-overview)
2. [Dataset Formats](#2-dataset-formats)
3. [Q&A Generation Methodology](#3-qa-generation-methodology)
4. [Dataset Quality Control](#4-dataset-quality-control)
5. [Train/Validation/Test Splits](#5-trainvalidationtest-splits)
6. [JSONL Schema Specifications](#6-jsonl-schema-specifications)
7. [Usage with Popular Frameworks](#7-usage-with-popular-frameworks)

---

## 1. Overview

This system generates fine-tuning datasets from book content using a RAG pipeline. The process:

1. **Extract** text from books (PDF, EPUB, scanned)
2. **Chunk** text into semantically meaningful segments
3. **Generate** question-answer pairs from each chunk
4. **Format** into multiple training formats
5. **Validate** quality and deduplicate
6. **Split** into train/validation/test sets
7. **Output** as JSONL files

### Why Generate Datasets from Books

- Books are high-quality, curated knowledge sources
- Chapter structure provides natural topic segmentation
- Book content is more reliable than web-scraped data
- Domain coverage: textbooks, technical manuals, reference materials

---

## 2. Dataset Formats

### 2.1 Instruction Format (Alpaca-style)

Best for: General instruction-following fine-tuning

```json
{
  "instruction": "Explain the concept of gradient descent in machine learning.",
  "input": "Context from the book about optimization algorithms...",
  "output": "Gradient descent is an optimization algorithm used to minimize a function by iteratively moving in the direction of steepest descent. In machine learning, it's used to minimize the loss function by updating model parameters..."
}
```

**When to use**:
- Fine-tuning models like LLaMA, Mistral, Alpaca
- Teaching models to follow instructions
- General-purpose assistant training

**Format variations**:
- With input: `{instruction, input, output}`
- Without input: `{instruction, output}` (instruction contains all context)

### 2.2 Question-Answer Format

Best for: Knowledge-focused fine-tuning, RAG training data

```json
{
  "question": "What is the time complexity of quicksort in the average case?",
  "answer": "The average-case time complexity of quicksort is O(n log n). This occurs when the pivot selection consistently divides the array into roughly equal halves...",
  "context": "Quicksort is a divide-and-conquer algorithm that works by selecting a 'pivot' element...",
  "source": "Introduction to Algorithms, Chapter 7",
  "metadata": {
    "book_title": "Introduction to Algorithms",
    "chapter": 7,
    "chunk_id": "chunk_abc123"
  }
}
```

**When to use**:
- Training retrieval-augmented models
- Building knowledge bases
- Question-answering systems

### 2.3 Chat Format (OpenAI Messages)

Best for: Chat model fine-tuning, conversational AI

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a knowledgeable assistant specializing in computer science. Answer questions accurately based on your training."
    },
    {
      "role": "user",
      "content": "Can you explain how binary search trees work?"
    },
    {
      "role": "assistant",
      "content": "A binary search tree (BST) is a hierarchical data structure where each node has at most two children. The key property is that for any node, all values in the left subtree are smaller, and all values in the right subtree are larger. This allows O(log n) search, insertion, and deletion operations in the average case..."
    }
  ]
}
```

**When to use**:
- Fine-tuning chat models (GPT, Claude, LLaMA-chat)
- Multi-turn conversation training
- OpenAI fine-tuning API

---

## 3. Q&A Generation Methodology

### 3.1 Factual Questions

Extract facts directly from the text.

**Pattern**: "What is X?" / "Who is X?" / "When did X?" / "Where is X?"

**Algorithm**:
```
1. Identify named entities (people, places, dates, terms)
2. Identify key facts (definitions, descriptions, properties)
3. Generate question targeting each fact
4. Extract answer from source text
```

**Example**:
```
Source: "Python was created by Guido van Rossum and first released in 1991."

Generated:
Q: "Who created Python?"
A: "Python was created by Guido van Rossum."

Q: "When was Python first released?"
A: "Python was first released in 1991."
```

### 3.2 Inferential Questions

Require reasoning over the text.

**Pattern**: "Why does X?" / "How does X work?" / "What causes X?"

**Algorithm**:
```
1. Identify causal relationships (because, therefore, since)
2. Identify processes (steps, procedures, mechanisms)
3. Generate question requiring inference
4. Construct answer from multiple sentences
```

**Example**:
```
Source: "Binary search requires a sorted array because it relies on comparing 
the target value with the middle element to decide which half to search."

Generated:
Q: "Why does binary search require a sorted array?"
A: "Binary search requires a sorted array because it compares the target 
value with the middle element to determine which half of the array to 
search next. Without sorting, this comparison wouldn't reliably narrow 
down the search space."
```

### 3.3 Comparative Questions

Compare concepts within or across chunks.

**Pattern**: "How does X differ from Y?" / "What are the advantages of X over Y?"

**Algorithm**:
```
1. Identify multiple concepts mentioned together
2. Identify comparison words (unlike, compared to, whereas)
3. Generate comparison question
4. Extract comparative statements as answer
```

### 3.4 Summary Questions

Summarize sections or concepts.

**Pattern**: "Summarize..." / "What are the main points of..." / "Give an overview of..."

**Algorithm**:
```
1. Identify topic sentences in chunk
2. Generate summarization instruction
3. Use key sentences as answer
```

### 3.5 Template-Based Generation

For reliable, structured Q&A generation without an LLM:

```rust
// Question templates
const TEMPLATES: &[(&str, &str)] = &[
    ("What is {term}?", "{definition}"),
    ("Who {action}?", "{person} {action_detail}"),
    ("When did {event}?", "{date}: {event_detail}"),
    ("Why is {concept} important?", "{explanation}"),
    ("How does {process} work?", "{process_steps}"),
    ("What are the benefits of {topic}?", "{benefits_list}"),
];
```

---

## 4. Dataset Quality Control

### 4.1 Validation Checks

| Check | Description | Threshold |
|-------|-------------|-----------|
| Answer grounding | Answer exists in source chunk | 100% |
| Question validity | Ends with question mark, >5 words | Required |
| Answer length | Between min and max tokens | 20-500 tokens |
| No duplicates | No identical questions | 0% duplicates |
| No PII | No personal information | 0% PII |
| Coherence | Grammatically correct | LLM judge |

### 4.2 Deduplication

```rust
// Deduplication strategies
1. Exact match: Remove identical question strings
2. Fuzzy match: Remove questions with >90% similarity
3. Semantic match: Remove questions with cosine similarity >0.95
```

### 4.3 Quality Scoring

Each Q&A pair gets a quality score (0-1):

```
score = 0.3 * grounding_score +     // Answer in source
        0.3 * relevance_score +      // Question relevant to chunk
        0.2 * clarity_score +        // Question is clear
        0.2 * completeness_score     // Answer is complete
```

Filter out pairs below threshold (default: 0.7).

---

## 5. Train/Validation/Test Splits

### 5.1 Split Ratios

| Split | Ratio | Purpose |
|-------|-------|---------|
| Train | 80% | Model training |
| Validation | 10% | Hyperparameter tuning, early stopping |
| Test | 10% | Final evaluation |

### 5.2 Splitting Strategy

**Source-aware splitting**: Ensure no book appears in multiple splits.

```
Books: [A, B, C, D, E, F, G, H, I, J]
Shuffle: [C, A, H, J, B, F, D, I, G, E]

Train (80%): [C, A, H, J, B, F, D, I]  (books)
Validation (10%): [G]  (book)
Test (10%): [E]  (book)
```

This prevents data leakage where similar content appears in train and test.

### 5.3 Stratified Sampling

If books have different lengths:
- Ensure each split has representative book sizes
- Balance topic distribution across splits

### 5.4 Output Files

```
data/processed/jsonl/
├── instruction_train.jsonl
├── instruction_val.jsonl
├── instruction_test.jsonl
├── qa_train.jsonl
├── qa_val.jsonl
├── qa_test.jsonl
├── chat_train.jsonl
├── chat_val.jsonl
└── chat_test.jsonl
```

---

## 6. JSONL Schema Specifications

### 6.1 Instruction Format Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["instruction", "output"],
  "properties": {
    "instruction": {
      "type": "string",
      "minLength": 10,
      "description": "The task or question for the model"
    },
    "input": {
      "type": "string",
      "description": "Optional context or additional input"
    },
    "output": {
      "type": "string",
      "minLength": 20,
      "description": "The expected response"
    },
    "source": {
      "type": "string",
      "description": "Source book and chapter"
    },
    "chunk_id": {
      "type": "string",
      "description": "ID of source chunk"
    }
  }
}
```

### 6.2 Q&A Format Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["question", "answer"],
  "properties": {
    "question": {
      "type": "string",
      "minLength": 10,
      "description": "The question"
    },
    "answer": {
      "type": "string",
      "minLength": 20,
      "description": "The answer"
    },
    "context": {
      "type": "string",
      "description": "Source context for the answer"
    },
    "source": {
      "type": "string",
      "description": "Source reference"
    },
    "metadata": {
      "type": "object",
      "properties": {
        "book_title": {"type": "string"},
        "chapter": {"type": "integer"},
        "chunk_id": {"type": "string"},
        "question_type": {"type": "string", "enum": ["factual", "inferential", "comparative", "summary"]}
      }
    }
  }
}
```

### 6.3 Chat Format Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["messages"],
  "properties": {
    "messages": {
      "type": "array",
      "minItems": 2,
      "items": {
        "type": "object",
        "required": ["role", "content"],
        "properties": {
          "role": {
            "type": "string",
            "enum": ["system", "user", "assistant"]
          },
          "content": {
            "type": "string",
            "minLength": 1
          }
        }
      }
    }
  }
}
```

---

## 7. Usage with Popular Frameworks

### 7.1 OpenAI Fine-Tuning

```bash
# Upload dataset
openai api files.create -f chat_train.jsonl -p fine-tune

# Create fine-tune job
openai api fine_tunes.create -t file-xxx -m gpt-3.5-turbo
```

Chat format required:
```json
{"messages": [{"role": "system", "..."}, {"role": "user", "..."}, {"role": "assistant", "..."}]}
```

### 7.2 Hugging Face Transformers

```python
from datasets import load_dataset

# Load instruction format
dataset = load_dataset("json", data_files="instruction_train.jsonl")

# For training
from transformers import TrainingArguments

training_args = TrainingArguments(
    output_dir="./results",
    num_train_epochs=3,
    per_device_train_batch_size=4,
    save_steps=1000,
)
```

### 7.3 LLaMA Factory

```yaml
# dataset_info.json
{
  "book_instructions": {
    "file_name": "instruction_train.jsonl",
    "columns": {
      "prompt": "instruction",
      "query": "input",
      "response": "output"
    }
  }
}
```

### 7.4 Axolotl

```yaml
# config.yml
datasets:
  - path: data/processed/jsonl/instruction_train.jsonl
    type: alpaca

dataset_prepared_path: last_run_prepared
val_set_size: 0.1
```

### 7.5 Unsloth

```python
from unsloth import FastLanguageModel

model, tokenizer = FastLanguageModel.from_pretrained("unsloth/llama-3-8b")

# Load dataset
from datasets import load_dataset
dataset = load_dataset("json", data_files="instruction_train.jsonl")

# Format for training
def format_prompt(example):
    return f"""### Instruction:
{example['instruction']}

### Input:
{example['input']}

### Response:
{example['output']}"""
```

---

## Appendix: Sample Generated Data

### Sample Instruction Entry

```json
{
  "instruction": "Explain the concept of recursion in programming and provide an example.",
  "input": "Recursion is a programming technique where a function calls itself to solve smaller instances of the same problem. A recursive function must have a base case that stops the recursion and a recursive case that breaks the problem down.",
  "output": "Recursion is a method of solving problems where a function calls itself with a smaller version of the problem. Every recursive function needs two components:\n\n1. **Base case**: A condition that stops the recursion (e.g., when n equals 0 or 1)\n2. **Recursive case**: The function calls itself with a modified argument that moves toward the base case\n\nExample - Factorial:\n```\nfunction factorial(n) {\n  if (n <= 1) return 1;  // Base case\n  return n * factorial(n - 1);  // Recursive case\n}\n```\n\nWhen factorial(5) is called, it expands as: 5 * 4 * 3 * 2 * 1 = 120.",
  "source": "Introduction to Computer Science, Chapter 12",
  "chunk_id": "chunk_def456"
}
```

### Sample Q&A Entry

```json
{
  "question": "What is the difference between a stack and a queue?",
  "answer": "A stack follows Last-In-First-Out (LIFO) ordering where elements are added and removed from the same end (the top), like a stack of plates. A queue follows First-In-First-Out (FIFO) ordering where elements are added at one end (rear) and removed from the other end (front), like people standing in line.",
  "context": "Stacks and queues are fundamental data structures. A stack is a LIFO structure where push and pop operations occur at the top. A queue is a FIFO structure with enqueue at the rear and dequeue at the front.",
  "source": "Data Structures and Algorithms, Chapter 3",
  "metadata": {
    "book_title": "Data Structures and Algorithms",
    "chapter": 3,
    "chunk_id": "chunk_ghi789",
    "question_type": "comparative"
  }
}
```

### Sample Chat Entry

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a computer science tutor. Explain concepts clearly with examples."
    },
    {
      "role": "user",
      "content": "What is Big O notation and why is it important?"
    },
    {
      "role": "assistant",
      "content": "Big O notation is a mathematical notation used to describe the upper bound of an algorithm's time or space complexity as the input size grows. It helps us understand how an algorithm will perform at scale.\n\nCommon Big O complexities (from fastest to slowest):\n- O(1): Constant time - accessing an array element\n- O(log n): Logarithmic time - binary search\n- O(n): Linear time - iterating through a list\n- O(n log n): Linearithmic time - merge sort\n- O(n²): Quadratic time - nested loops\n- O(2ⁿ): Exponential time - recursive Fibonacci\n\nIt's important because it lets us compare algorithms independent of hardware, predict performance on large inputs, and make informed design decisions."
    }
  ]
}
```
