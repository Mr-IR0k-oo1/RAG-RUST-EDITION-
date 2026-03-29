use crate::rag::RetrievedChunk;

/// Context injection for RAG
pub struct PromptBuilder;

impl PromptBuilder {
    /// Build a prompt with retrieved context
    pub fn build_prompt(query: &str, chunks: &[RetrievedChunk]) -> String {
        let mut prompt = String::from("Answer the question based on the provided context.\n\n");
        prompt.push_str("Context:\n");

        for (i, chunk) in chunks.iter().enumerate() {
            prompt.push_str(&format!(
                "--- Source {} (relevance: {:.2}) ---\n",
                i + 1,
                chunk.score
            ));
            let content_text = Self::get_string_field(&chunk.payload, "content");
            if let Some(text) = content_text {
                prompt.push_str(&text);
            }
            prompt.push('\n');
        }

        prompt.push_str("\n--- Question ---\n");
        prompt.push_str(query);
        prompt.push_str("\n\nAnswer:");

        prompt
    }

    /// Build RAG-specific prompt with instructions for grounded generation
    pub fn build_rag_prompt(query: &str, chunks: &[RetrievedChunk]) -> String {
        let mut prompt = String::from(
            "You are a helpful assistant that answers questions based on the provided context.\n\
            Instructions:\n\
            - Answer ONLY using information from the context below\n\
            - If the context doesn't contain enough information, say 'I don't have enough information to answer this question'\n\
            - Cite your sources when possible\n\
            - Be concise and accurate\n\n"
        );

        prompt.push_str("Context:\n");

        for (i, chunk) in chunks.iter().enumerate() {
            let source_str = Self::get_string_field(&chunk.payload, "source")
                .unwrap_or_else(|| "Unknown source".to_string());

            prompt.push_str(&format!("[Source {} - {}]\n", i + 1, source_str));

            let content_text = Self::get_string_field(&chunk.payload, "content");
            if let Some(text) = content_text {
                prompt.push_str(&text);
            }
            prompt.push_str("\n\n");
        }

        prompt.push_str("Question: ");
        prompt.push_str(query);
        prompt.push_str("\n\nAnswer:");

        prompt
    }

    /// Build a prompt for generating Q&A pairs from context
    pub fn build_qa_generation_prompt(context: &str, num_questions: usize) -> String {
        format!(
            "Given the following text, generate {} diverse question-answer pairs.\n\
            The questions should be:\n\
            - Factual (can be answered directly from the text)\n\
            - Clear and specific\n\
            - Varied in type (what, why, how, when, who)\n\n\
            Text:\n{}\n\n\
            Generate {} question-answer pairs in JSON format:\n\
            [\n  {{\"question\": \"...\", \"answer\": \"...\"}},\n  ...\n]",
            num_questions, context, num_questions
        )
    }

    /// Build a prompt for instruction generation
    pub fn build_instruction_generation_prompt(context: &str) -> String {
        format!(
            "Given the following text, generate an instruction-input-output triplet.\n\
            The instruction should ask the model to explain, summarize, or analyze the content.\n\
            The input should contain the relevant context.\n\
            The output should be a clear, accurate response.\n\n\
            Text:\n{}\n\n\
            Generate in JSON format:\n\
            {{\"instruction\": \"...\", \"input\": \"...\", \"output\": \"...\"}}",
            context
        )
    }

    /// Helper to extract string field from JSON value
    fn get_string_field(payload: &serde_json::Value, field: &str) -> Option<String> {
        match payload.get(field) {
            Some(val) => match val {
                serde_json::Value::String(s) => Some(s.clone()),
                _ => None,
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_building_with_retrieved_chunks() {
        let query = "What is machine learning?";
        let chunks = vec![
            RetrievedChunk {
                score: 0.95,
                payload: serde_json::json!({
                    "content": "Machine learning is a subset of artificial intelligence."
                }),
            },
            RetrievedChunk {
                score: 0.87,
                payload: serde_json::json!({
                    "content": "It enables computers to learn from data without being explicitly programmed."
                }),
            },
        ];

        let prompt = PromptBuilder::build_prompt(query, &chunks);

        assert!(prompt.contains("Machine learning"));
        assert!(prompt.contains("artificial intelligence"));
        assert!(prompt.contains("What is machine learning?"));
        assert!(prompt.contains("Context:"));
    }

    #[test]
    fn test_prompt_building_with_empty_chunks() {
        let query = "Test query";
        let chunks = vec![];

        let prompt = PromptBuilder::build_prompt(query, &chunks);

        assert!(prompt.contains("Test query"));
        assert!(prompt.contains("Context:"));
    }

    #[test]
    fn test_rag_prompt_structure() {
        let chunks = vec![RetrievedChunk {
            score: 0.9,
            payload: serde_json::json!({
                "content": "Sample content",
                "source": "Test Book, Chapter 1"
            }),
        }];

        let prompt = PromptBuilder::build_rag_prompt("Query?", &chunks);

        assert!(prompt.contains("Context:"));
        assert!(prompt.contains("Question:"));
        assert!(prompt.contains("Query?"));
        assert!(prompt.contains("Sample content"));
        assert!(prompt.contains("Test Book"));
    }

    #[test]
    fn test_qa_generation_prompt() {
        let prompt = PromptBuilder::build_qa_generation_prompt("Test context", 3);
        assert!(prompt.contains("3"));
        assert!(prompt.contains("Test context"));
        assert!(prompt.contains("question-answer"));
    }

    #[test]
    fn test_instruction_generation_prompt() {
        let prompt = PromptBuilder::build_instruction_generation_prompt("Test context");
        assert!(prompt.contains("Test context"));
        assert!(prompt.contains("instruction"));
    }
}
