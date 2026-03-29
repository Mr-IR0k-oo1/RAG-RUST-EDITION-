use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Cleans and normalizes text extracted from PDFs and books
pub struct TextCleaner {
    extra_whitespace: Regex,
    urls: Regex,
    emails: Regex,
    hyphenation: Regex,
}

impl TextCleaner {
    pub fn new() -> Self {
        Self {
            extra_whitespace: Regex::new(r"\s+").unwrap(),
            urls: Regex::new(r"https?://[^\s]+").unwrap(),
            emails: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            hyphenation: Regex::new(r"(\w)-\n(\w)").unwrap(),
        }
    }

    /// Clean text by removing extra whitespace, normalizing unicode, etc.
    pub fn clean(&self, text: &str) -> String {
        // Normalize unicode (NFKC normalization)
        let normalized: String = text.nfkc().collect();

        // Fix hyphenation across line breaks
        let dehyphenated = self.fix_hyphenation(&normalized);

        // Remove extra whitespace
        let cleaned = self.extra_whitespace.replace_all(&dehyphenated, " ");

        // Trim
        cleaned.trim().to_string()
    }

    /// Remove special characters while preserving sentence structure
    pub fn remove_special_chars(&self, text: &str) -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?;:'\"-()[]{}".contains(*c))
            .collect()
    }

    /// Remove URLs and email addresses
    pub fn remove_links(&self, text: &str) -> String {
        let text = self.urls.replace_all(text, "");
        self.emails.replace_all(&text, "").to_string()
    }

    /// Fix words split across line breaks (hyphenation)
    pub fn fix_hyphenation(&self, text: &str) -> String {
        self.hyphenation.replace_all(text, "$1$2").to_string()
    }

    /// Remove common PDF artifacts
    pub fn remove_artifacts(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Remove standalone page numbers
        let lines: Vec<&str> = result.lines().collect();
        let filtered: Vec<&str> = lines
            .into_iter()
            .filter(|line| {
                let trimmed = line.trim();
                // Skip lines that are just numbers (page numbers)
                !trimmed.parse::<u32>().is_ok() && !trimmed.is_empty()
            })
            .collect();

        result = filtered.join("\n");
        result
    }

    /// Normalize quotes and dashes
    pub fn normalize_punctuation(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Normalize quotes
        result = result.replace('\u{201C}', "\"");
        result = result.replace('\u{201D}', "\"");
        result = result.replace('\u{2018}', "'");
        result = result.replace('\u{2019}', "'");

        // Normalize dashes
        result = result.replace('\u{2014}', "-");
        result = result.replace('\u{2013}', "-");

        result
    }

    /// Full cleaning pipeline
    pub fn full_clean(&self, text: &str) -> String {
        let text = self.remove_links(text);
        let text = self.remove_artifacts(&text);
        let text = self.normalize_punctuation(&text);
        let text = self.clean(&text);
        text
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
        assert_eq!(cleaner.clean("hello\n\nworld"), "hello world");
    }

    #[test]
    fn test_fix_hyphenation() {
        let cleaner = TextCleaner::new();
        assert_eq!(cleaner.fix_hyphenation("exam-\nple"), "example");
        assert_eq!(cleaner.fix_hyphenation("pro-\ngram"), "program");
    }

    #[test]
    fn test_remove_links() {
        let cleaner = TextCleaner::new();
        let text = "Visit https://example.com for info";
        assert!(!cleaner.remove_links(text).contains("https://"));
    }

    #[test]
    fn test_remove_emails() {
        let cleaner = TextCleaner::new();
        let text = "Contact test@example.com for help";
        assert!(!cleaner.remove_links(text).contains("test@example.com"));
    }

    #[test]
    fn test_normalize_punctuation() {
        let cleaner = TextCleaner::new();
        let text = "He said \u{201C}hello\u{201D}";
        assert_eq!(cleaner.normalize_punctuation(text), "He said \"hello\"");
    }

    #[test]
    fn test_full_clean() {
        let cleaner = TextCleaner::new();
        let text = "  This   is   a   test-\nwith   https://example.com  ";
        let cleaned = cleaner.full_clean(text);
        assert!(!cleaned.contains("https://"));
        assert!(cleaned.contains("testwith"));
    }
}
