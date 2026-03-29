use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::utils::fs::FileUtils;

/// Supported book formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookFormat {
    Pdf,
    Epub,
    Scanned,
}

/// Content extracted from a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookContent {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: BookFormat,
    pub chapters: Vec<Chapter>,
    pub raw_text: String,
    pub metadata: BookMetadata,
}

/// A single chapter in a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub index: usize,
    pub title: String,
    pub content: String,
    pub page_start: Option<usize>,
    pub page_end: Option<usize>,
}

/// Metadata about the book file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub filename: String,
    pub file_size: u64,
    pub page_count: Option<usize>,
    pub created_at: DateTime<Utc>,
}

/// Multi-format book loader supporting PDF, EPUB, and scanned documents
pub struct BookLoader;

impl BookLoader {
    /// Load a single book file
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<BookContent> {
        let path = path.as_ref();
        let format = Self::detect_format(path)?;
        
        info!("Loading book: {:?} (format: {:?})", path, format);
        
        let content = match format {
            BookFormat::Pdf => Self::load_pdf(path).await?,
            BookFormat::Epub => Self::load_epub(path).await?,
            BookFormat::Scanned => Self::load_scanned(path).await?,
        };
        
        Ok(content)
    }

    /// Load all books from a directory (recursive)
    pub async fn load_directory<P: AsRef<Path>>(path: P) -> Result<Vec<BookContent>> {
        let path = path.as_ref();
        let mut books = Vec::new();
        
        let files = Self::find_book_files(path)?;
        info!("Found {} book files in {:?}", files.len(), path);
        
        for file_path in files {
            match Self::load(&file_path).await {
                Ok(book) => {
                    info!("Loaded: {} ({} chapters)", book.title, book.chapters.len());
                    books.push(book);
                }
                Err(e) => {
                    warn!("Failed to load {:?}: {}", file_path, e);
                }
            }
        }
        
        Ok(books)
    }

    /// Detect book format from file extension
    pub fn detect_format<P: AsRef<Path>>(path: P) -> Result<BookFormat> {
        let path = path.as_ref();
        
        // Check if directory (scanned book)
        if path.is_dir() {
            return Ok(BookFormat::Scanned);
        }
        
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        match ext.as_str() {
            "pdf" => Ok(BookFormat::Pdf),
            "epub" => Ok(BookFormat::Epub),
            "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" => Ok(BookFormat::Scanned),
            _ => Err(anyhow::anyhow!("Unsupported file format: {}", ext)),
        }
    }

    /// Find all book files in a directory
    fn find_book_files<P: AsRef<Path>>(dir: P) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        let supported_extensions = ["pdf", "epub", "png", "jpg", "jpeg", "tiff", "tif", "bmp"];
        
        for entry in walkdir(dir)? {
            let path = entry;
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if supported_extensions.contains(&ext_lower.as_str()) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
        
        Ok(files)
    }

    /// Load a PDF file
    async fn load_pdf<P: AsRef<Path>>(path: P) -> Result<BookContent> {
        let path = path.as_ref();
        let filename = FileUtils::filename(path);
        let file_size = std::fs::metadata(path)?.len();
        
        let raw_text = Self::extract_pdf_text(path).await?;
        let title = Self::extract_title_from_filename(&filename);
        let chapters = Self::detect_chapters_from_text(&raw_text);
        
        Ok(BookContent {
            id: Uuid::new_v4().to_string(),
            title,
            author: None,
            format: BookFormat::Pdf,
            chapters,
            raw_text,
            metadata: BookMetadata {
                filename,
                file_size,
                page_count: None,
                created_at: Utc::now(),
            },
        })
    }

    /// Extract text from PDF
    async fn extract_pdf_text<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        
        // Try pdfium first, fall back to basic extraction
        match Self::extract_with_pdfium(path).await {
            Ok(text) => Ok(text),
            Err(e) => {
                warn!("pdfium extraction failed: {}, trying fallback", e);
                Self::extract_pdf_fallback(path).await
            }
        }
    }

    /// Extract text using pdfium-render
    async fn extract_with_pdfium<P: AsRef<Path>>(_path: P) -> Result<String> {
        // TODO: Implement proper pdfium extraction
        Err(anyhow::anyhow!("pdfium not configured"))
    }

    /// Fallback PDF text extraction (basic)
    async fn extract_pdf_fallback<P: AsRef<Path>>(path: P) -> Result<String> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        
        // Simple text extraction from PDF binary
        let mut result = String::new();
        let mut in_stream = false;
        
        for line in text.lines() {
            if line.contains("stream") {
                in_stream = true;
                continue;
            }
            if line.contains("endstream") {
                in_stream = false;
                continue;
            }
            if in_stream && line.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                result.push_str(line);
                result.push(' ');
            }
        }
        
        if result.trim().is_empty() {
            return Err(anyhow::anyhow!("Could not extract text from PDF"));
        }
        
        Ok(result)
    }

    /// Load an EPUB file
    async fn load_epub<P: AsRef<Path>>(path: P) -> Result<BookContent> {
        let path = path.as_ref();
        let filename = FileUtils::filename(path);
        let file_size = std::fs::metadata(path)?.len();
        
        let raw_text = Self::extract_epub_text(path).await?;
        let title = Self::extract_title_from_filename(&filename);
        let chapters = Self::detect_chapters_from_text(&raw_text);
        
        Ok(BookContent {
            id: Uuid::new_v4().to_string(),
            title,
            author: None,
            format: BookFormat::Epub,
            chapters,
            raw_text,
            metadata: BookMetadata {
                filename,
                file_size,
                page_count: None,
                created_at: Utc::now(),
            },
        })
    }

    /// Extract text from EPUB
    async fn extract_epub_text<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        
        // Read EPUB as bytes and extract text content
        let bytes = std::fs::read(path)?;
        
        // EPUB files are ZIP archives containing XHTML files
        // For now, do basic extraction from the binary content
        let content = String::from_utf8_lossy(&bytes);
        
        // Extract text between common HTML tags
        let mut result = String::new();
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        let cleaned = re.replace_all(&content, " ");
        
        // Clean up whitespace
        let ws_re = regex::Regex::new(r"\s+").unwrap();
        let final_text = ws_re.replace_all(&cleaned, " ");
        
        // Filter out non-text content (keep ASCII printable + common unicode)
        for c in final_text.chars() {
            if c.is_ascii_graphic() || c.is_ascii_whitespace() || (c as u32) > 127 {
                result.push(c);
            }
        }
        
        if result.trim().is_empty() {
            return Err(anyhow::anyhow!("Could not extract text from EPUB"));
        }
        
        Ok(result)
    }

    /// Strip HTML tags from text
    fn strip_html_tags(html: &str) -> String {
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        let without_tags = re.replace_all(html, " ");
        
        let ws_re = regex::Regex::new(r"\s+").unwrap();
        ws_re.replace_all(&without_tags, " ").trim().to_string()
    }

    /// Load a scanned book (directory of images)
    async fn load_scanned<P: AsRef<Path>>(path: P) -> Result<BookContent> {
        let path = path.as_ref();
        let filename = if path.is_dir() {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("scanned_book")
                .to_string()
        } else {
            FileUtils::filename(path)
        };
        
        let file_size = if path.is_file() {
            std::fs::metadata(path)?.len()
        } else {
            Self::directory_size(path)?
        };
        
        let raw_text = Self::extract_scanned_text(path).await?;
        let title = Self::extract_title_from_filename(&filename);
        let chapters = Self::detect_chapters_from_text(&raw_text);
        
        Ok(BookContent {
            id: Uuid::new_v4().to_string(),
            title,
            author: None,
            format: BookFormat::Scanned,
            chapters,
            raw_text,
            metadata: BookMetadata {
                filename,
                file_size,
                page_count: None,
                created_at: Utc::now(),
            },
        })
    }

    /// Extract text from scanned documents using OCR
    async fn extract_scanned_text<P: AsRef<Path>>(_path: P) -> Result<String> {
        Err(anyhow::anyhow!(
            "OCR processing not yet implemented. \
            Install ocrs crate and implement OcrProcessor for scanned document support."
        ))
    }

    /// Calculate total size of files in a directory
    fn directory_size<P: AsRef<Path>>(dir: P) -> Result<u64> {
        let mut total = 0u64;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    }

    /// Extract title from filename (remove extension and clean up)
    fn extract_title_from_filename(filename: &str) -> String {
        let title = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);
        
        title.replace(['_', '-'], " ")
    }

    /// Detect chapters from text content
    fn detect_chapters_from_text(text: &str) -> Vec<Chapter> {
        let mut chapters = Vec::new();
        
        let patterns = [
            r"(?i)chapter\s+(\d+|[ivxlcdm]+)[\s:.]",
            r"(?i)^(\d+)\.\s+[A-Z]",
            r"(?i)^(part|section)\s+(\d+|[ivxlcdm]+)[\s:.]",
        ];
        
        let combined_pattern = format!("({})", patterns.join("|"));
        let re = regex::Regex::new(&combined_pattern).unwrap();
        
        let mut chapter_starts: Vec<(usize, String)> = Vec::new();
        
        // Find all chapter boundaries
        for cap in re.captures_iter(text) {
            let match_start = cap.get(0).unwrap().start();
            let chapter_title = cap.get(0).unwrap().as_str().to_string();
            chapter_starts.push((match_start, chapter_title));
        }
        
        // Create chapters
        for (i, (start, title)) in chapter_starts.iter().enumerate() {
            let end = if i + 1 < chapter_starts.len() {
                chapter_starts[i + 1].0
            } else {
                text.len()
            };
            
            let content = text[*start..end].trim().to_string();
            if !content.is_empty() {
                chapters.push(Chapter {
                    index: i,
                    title: title.clone(),
                    content,
                    page_start: None,
                    page_end: None,
                });
            }
        }
        
        // If no chapters detected, treat entire text as one chapter
        if chapters.is_empty() {
            chapters.push(Chapter {
                index: 0,
                title: "Full Text".to_string(),
                content: text.to_string(),
                page_start: None,
                page_end: None,
            });
        }
        
        chapters
    }
}

/// Walk directory recursively
fn walkdir<P: AsRef<Path>>(dir: P) -> Result<Vec<std::path::PathBuf>> {
    let mut results = Vec::new();
    
    fn walk_recursive(path: &Path, results: &mut Vec<std::path::PathBuf>) -> Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    walk_recursive(&path, results)?;
                } else {
                    results.push(path);
                }
            }
        } else {
            results.push(path.to_path_buf());
        }
        Ok(())
    }
    
    walk_recursive(dir.as_ref(), &mut results)?;
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            BookLoader::detect_format("book.pdf").unwrap(),
            BookFormat::Pdf
        );
        assert_eq!(
            BookLoader::detect_format("book.epub").unwrap(),
            BookFormat::Epub
        );
        assert_eq!(
            BookLoader::detect_format("page.png").unwrap(),
            BookFormat::Scanned
        );
    }

    #[test]
    fn test_title_extraction() {
        assert_eq!(
            BookLoader::extract_title_from_filename("my_test_book.pdf"),
            "my test book"
        );
        assert_eq!(
            BookLoader::extract_title_from_filename("another-book.epub"),
            "another book"
        );
    }

    #[test]
    fn test_chapter_detection() {
        let text = "Chapter 1: Introduction\nSome intro text.\nChapter 2: Methods\nSome methods text.";
        let chapters = BookLoader::detect_chapters_from_text(text);
        assert!(chapters.len() >= 2);
    }

    #[test]
    fn test_html_stripping() {
        let html = "<p>Hello <b>world</b></p>";
        assert_eq!(BookLoader::strip_html_tags(html), "Hello world");
    }
}
