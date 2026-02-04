use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Cleans and normalizes text extracted from PDFs
pub struct TextCleaner {
    extra_whitespace: Regex,
}

impl TextCleaner {
    pub fn new() -> Self {
        Self {
            extra_whitespace: Regex::new(r"\s+").unwrap(),
        }
    }

    /// Clean text by removing extra whitespace, normalizing unicode, etc.
    pub fn clean(&self, text: &str) -> String {
        // Normalize unicode
        let normalized: String = text.nfkc().collect();

        // Remove extra whitespace
        let cleaned = self.extra_whitespace.replace_all(&normalized, " ");

        // Trim
        cleaned.trim().to_string()
    }

    /// Remove special characters while preserving sentence structure
    pub fn remove_special_chars(&self, text: &str) -> String {
        // TODO: Implement special character removal
        text.to_string()
    }

    /// Remove URLs and email addresses
    pub fn remove_links(&self, text: &str) -> String {
        // TODO: Implement link removal
        text.to_string()
    }
}

impl Default for TextCleaner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_whitespace() {
        let cleaner = TextCleaner::new();
        assert_eq!(cleaner.clean("hello   world"), "hello world");
    }
}
