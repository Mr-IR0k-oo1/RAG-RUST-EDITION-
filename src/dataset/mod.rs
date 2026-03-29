pub mod writer;

pub use writer::DatasetWriter;

use serde::{Deserialize, Serialize};

/// Instruction format entry for fine-tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionEntry {
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_id: Option<String>,
}

/// Q&A format entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAEntry {
    pub question: String,
    pub answer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<QAMetadata>,
}

/// Q&A metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAMetadata {
    pub book_title: Option<String>,
    pub chapter: Option<usize>,
    pub chunk_id: Option<String>,
    pub question_type: Option<String>,
}

/// Chat format message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat format entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEntry {
    pub messages: Vec<ChatMessage>,
}

/// Question-answer pair (internal)
#[derive(Debug, Clone)]
pub struct QAPair {
    pub question: String,
    pub answer: String,
    pub context: String,
    pub source: String,
    pub question_type: String,
}

/// Complete dataset
#[derive(Debug, Clone)]
pub struct Dataset {
    pub instructions: Vec<InstructionEntry>,
    pub qa_pairs: Vec<QAEntry>,
    pub chat_entries: Vec<ChatEntry>,
}

/// Data splits for train/val/test
#[derive(Debug, Clone)]
pub struct DataSplits {
    pub train: Dataset,
    pub validation: Dataset,
    pub test: Dataset,
}

/// Dataset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    pub num_qa_pairs_per_chunk: usize,
    pub min_answer_length: usize,
    pub max_answer_length: usize,
    pub question_types: Vec<String>,
    pub train_ratio: f64,
    pub validation_ratio: f64,
    pub test_ratio: f64,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            num_qa_pairs_per_chunk: 3,
            min_answer_length: 20,
            max_answer_length: 500,
            question_types: vec![
                "factual".to_string(),
                "inferential".to_string(),
                "comparative".to_string(),
            ],
            train_ratio: 0.8,
            validation_ratio: 0.1,
            test_ratio: 0.1,
        }
    }
}

/// Dataset generator
pub struct DatasetGenerator {
    config: DatasetConfig,
}

impl DatasetGenerator {
    pub fn new(config: DatasetConfig) -> Self {
        Self { config }
    }

    /// Generate dataset from text chunks
    pub fn generate_from_chunks(
        &self,
        chunks: &[crate::ingestion::Chunk],
    ) -> Result<Dataset, anyhow::Error> {
        let mut instructions = Vec::new();
        let mut qa_pairs = Vec::new();
        let mut chat_entries = Vec::new();

        for chunk in chunks {
            let qa_pairs_chunk = self.generate_qa_pairs(chunk);

            for qa in &qa_pairs_chunk {
                instructions.push(InstructionEntry {
                    instruction: qa.question.clone(),
                    input: Some(qa.context.clone()),
                    output: qa.answer.clone(),
                    source: Some(qa.source.clone()),
                    chunk_id: Some(chunk.id.clone()),
                });

                qa_pairs.push(QAEntry {
                    question: qa.question.clone(),
                    answer: qa.answer.clone(),
                    context: Some(qa.context.clone()),
                    source: Some(qa.source.clone()),
                    metadata: Some(QAMetadata {
                        book_title: None,
                        chapter: None,
                        chunk_id: Some(chunk.id.clone()),
                        question_type: Some(qa.question_type.clone()),
                    }),
                });

                chat_entries.push(ChatEntry {
                    messages: vec![
                        ChatMessage {
                            role: "system".to_string(),
                            content: "You are a knowledgeable assistant. Answer questions accurately based on your training.".to_string(),
                        },
                        ChatMessage {
                            role: "user".to_string(),
                            content: qa.question.clone(),
                        },
                        ChatMessage {
                            role: "assistant".to_string(),
                            content: qa.answer.clone(),
                        },
                    ],
                });
            }
        }

        Ok(Dataset {
            instructions,
            qa_pairs,
            chat_entries,
        })
    }

    /// Generate Q&A pairs from a chunk
    fn generate_qa_pairs(&self, chunk: &crate::ingestion::Chunk) -> Vec<QAPair> {
        let mut pairs = Vec::new();
        let content: &String = &chunk.content;

        let sentences: Vec<&str> = content.split(". ").collect();

        for (i, sentence_ref) in sentences.iter().enumerate() {
            let sentence: &str = sentence_ref;
            if sentence.len() < 20 {
                continue;
            }

            let words: Vec<&str> = sentence.split_whitespace().collect();

            if words.len() > 5 {
                let mut found_term: Option<&str> = None;
                for word in words.iter() {
                    let w: &str = word;
                    if w.len() > 4 && w.starts_with(char::is_uppercase) {
                        found_term = Some(w);
                        break;
                    }
                }

                if let Some(term) = found_term {
                    pairs.push(QAPair {
                        question: format!("What is {}?", term),
                        answer: sentence.to_string(),
                        context: content.clone(),
                        source: format!("chunk_{}", chunk.id),
                        question_type: "factual".to_string(),
                    });
                }

                if i == 0 && sentences.len() > 1 {
                    let preview_words = &words[..std::cmp::min(5, words.len())];
                    let explain_answer = sentences[..std::cmp::min(3, sentences.len())].join(". ");
                    pairs.push(QAPair {
                        question: format!(
                            "Explain the following concept: {}",
                            preview_words.join(" ")
                        ),
                        answer: explain_answer,
                        context: content.clone(),
                        source: format!("chunk_{}", chunk.id),
                        question_type: "inferential".to_string(),
                    });
                }
            }
        }

        pairs.truncate(self.config.num_qa_pairs_per_chunk);
        pairs
    }

    /// Split dataset into train/val/test
    pub fn generate_splits(&self, dataset: &Dataset) -> Result<DataSplits, anyhow::Error> {
        let total = dataset.instructions.len();
        let train_end = (total as f64 * self.config.train_ratio) as usize;
        let val_end = train_end + (total as f64 * self.config.validation_ratio) as usize;

        Ok(DataSplits {
            train: Dataset {
                instructions: dataset.instructions[..train_end].to_vec(),
                qa_pairs: dataset.qa_pairs[..train_end].to_vec(),
                chat_entries: dataset.chat_entries[..train_end].to_vec(),
            },
            validation: Dataset {
                instructions: dataset.instructions[train_end..val_end].to_vec(),
                qa_pairs: dataset.qa_pairs[train_end..val_end].to_vec(),
                chat_entries: dataset.chat_entries[train_end..val_end].to_vec(),
            },
            test: Dataset {
                instructions: dataset.instructions[val_end..].to_vec(),
                qa_pairs: dataset.qa_pairs[val_end..].to_vec(),
                chat_entries: dataset.chat_entries[val_end..].to_vec(),
            },
        })
    }
}
