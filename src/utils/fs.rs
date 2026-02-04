use anyhow::Result;
use std::path::{Path, PathBuf};

/// Boring glue code for file operations
pub struct FileUtils;

impl FileUtils {
    /// Find all PDF files in a directory
    pub fn find_pdfs<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
        let mut pdfs = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "pdf" {
                        pdfs.push(path);
                    }
                }
            } else if path.is_dir() {
                // Recursively find PDFs
                let mut sub_pdfs = Self::find_pdfs(&path)?;
                pdfs.append(&mut sub_pdfs);
            }
        }

        Ok(pdfs)
    }

    /// Ensure directory exists, create if not
    pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Get filename from path
    pub fn filename<P: AsRef<Path>>(path: P) -> String {
        path.as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}
