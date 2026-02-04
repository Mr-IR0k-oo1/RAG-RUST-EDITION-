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

    /// Tokenize into sentences
    pub fn tokenize_sentences(&self, text: &str) -> Vec<String> {
        text.split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
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
}
