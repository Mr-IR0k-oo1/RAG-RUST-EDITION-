use crate::rag::RetrievedChunk;

/// Context injection for RAG
pub struct PromptBuilder;

impl PromptBuilder {
    /// Build a prompt with retrieved context
    pub fn build_prompt(query: &str, chunks: &[RetrievedChunk]) -> String {
        let mut prompt = String::from("Answer the question based on the provided context.\n\n");
        prompt.push_str("Context:\n");

        for (i, chunk) in chunks.iter().enumerate() {
            prompt.push_str(&format!("--- Chunk {} ---\n", i + 1));
            if let Some(content) = chunk.payload.get("content").and_then(|v| v.as_str()) {
                prompt.push_str(content);
            }
            prompt.push('\n');
        }

        prompt.push_str("\n--- Question ---\n");
        prompt.push_str(query);

        prompt
    }
}
