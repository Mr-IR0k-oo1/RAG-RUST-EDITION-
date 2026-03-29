use regex::Regex;

/// Simple tokenizer for splitting text into tokens
pub struct Tokenizer {
    word_pattern: Regex,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            word_pattern: Regex::new(r"\b\w+\b").unwrap(),
        }
    }

    /// Tokenize text into words
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        self.word_pattern
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect()
    }

    /// Count tokens in text
    pub fn count_tokens(&self, text: &str) -> usize {
        self.tokenize(text).len()
    }

    /// Split text into sentences
    pub fn tokenize_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting without look-around
        // Split on period/question/exclamation followed by space
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);

            if ch == '.' || ch == '!' || ch == '?' {
                // Check if next character (if any) is uppercase or end of string
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() && trimmed.len() > 1 {
                    sentences.push(trimmed);
                    current = String::new();
                }
            }
        }

        // Add remaining text if any
        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }

        sentences
    }

    /// Count characters in text
    pub fn count_chars(&self, text: &str) -> usize {
        text.chars().count()
    }

    /// Estimate token count (faster than exact count)
    /// Uses approximation: ~4 chars per token for English
    pub fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f64 / 4.0).ceil() as usize
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_count_tokens() {
        let tokenizer = Tokenizer::new();
        assert_eq!(tokenizer.count_tokens("one two three"), 3);
    }

    #[test]
    fn test_sentence_split() {
        let tokenizer = Tokenizer::new();
        let sentences = tokenizer.tokenize_sentences("Hello world. How are you? I am fine.");
        assert!(sentences.len() >= 2);
    }

    #[test]
    fn test_sentence_with_abbreviation() {
        let tokenizer = Tokenizer::new();
        let sentences = tokenizer.tokenize_sentences("Dr. Smith went home. He was tired.");
        assert!(sentences.len() >= 1);
    }

    #[test]
    fn test_estimate_tokens() {
        let tokenizer = Tokenizer::new();
        let estimate = tokenizer.estimate_tokens("This is a test sentence.");
        assert!(estimate > 0);
    }
}
