use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use super::{ChatEntry, DataSplits, InstructionEntry, QAEntry};

/// Writes dataset to JSONL format
pub struct DatasetWriter {
    writer: Option<BufWriter<File>>,
}

impl DatasetWriter {
    /// Create a new dataset writer
    pub fn new() -> Self {
        Self { writer: None }
    }

    /// Create writer for a specific file path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer: Some(writer),
        })
    }

    /// Write instruction format entries to JSONL
    pub fn write_instruction<P: AsRef<Path>>(path: P, entries: &[InstructionEntry]) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for entry in entries {
            let line = serde_json::to_string(entry)?;
            use std::io::Write;
            writeln!(writer, "{}", line)?;
        }

        Ok(())
    }

    /// Write Q&A format entries to JSONL
    pub fn write_qa<P: AsRef<Path>>(path: P, entries: &[QAEntry]) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for entry in entries {
            let line = serde_json::to_string(entry)?;
            use std::io::Write;
            writeln!(writer, "{}", line)?;
        }

        Ok(())
    }

    /// Write chat format entries to JSONL
    pub fn write_chat<P: AsRef<Path>>(path: P, entries: &[ChatEntry]) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for entry in entries {
            let line = serde_json::to_string(entry)?;
            use std::io::Write;
            writeln!(writer, "{}", line)?;
        }

        Ok(())
    }

    /// Write all data splits to files
    pub fn write_splits<P: AsRef<Path>>(base_path: P, splits: &DataSplits) -> Result<()> {
        let base = base_path.as_ref();

        // Write instruction format
        Self::write_instruction(
            base.join("instruction_train.jsonl"),
            &splits.train.instructions,
        )?;
        Self::write_instruction(
            base.join("instruction_val.jsonl"),
            &splits.validation.instructions,
        )?;
        Self::write_instruction(
            base.join("instruction_test.jsonl"),
            &splits.test.instructions,
        )?;

        // Write Q&A format
        Self::write_qa(base.join("qa_train.jsonl"), &splits.train.qa_pairs)?;
        Self::write_qa(base.join("qa_val.jsonl"), &splits.validation.qa_pairs)?;
        Self::write_qa(base.join("qa_test.jsonl"), &splits.test.qa_pairs)?;

        // Write chat format
        Self::write_chat(base.join("chat_train.jsonl"), &splits.train.chat_entries)?;
        Self::write_chat(base.join("chat_val.jsonl"), &splits.validation.chat_entries)?;
        Self::write_chat(base.join("chat_test.jsonl"), &splits.test.chat_entries)?;

        Ok(())
    }

    /// Write a single chunk to JSONL (legacy support)
    pub fn write_chunk(&mut self, chunk: &crate::ingestion::Chunk) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            let line = json!({
                "id": chunk.id,
                "document_id": chunk.document_id,
                "content": chunk.content,
                "chunk_index": chunk.chunk_index,
                "total_chunks": chunk.total_chunks,
                "start_char": chunk.start_char,
                "end_char": chunk.end_char,
                "token_count": chunk.token_count,
            });

            use std::io::Write;
            writeln!(writer, "{}", line.to_string())?;
        }
        Ok(())
    }

    /// Write multiple chunks (legacy support)
    pub fn write_chunks(&mut self, chunks: &[crate::ingestion::Chunk]) -> Result<()> {
        for chunk in chunks {
            self.write_chunk(chunk)?;
        }
        Ok(())
    }

    /// Flush buffer to disk
    pub fn flush(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            use std::io::Write;
            writer.flush()?;
        }
        Ok(())
    }
}

impl Default for DatasetWriter {
    fn default() -> Self {
        Self::new()
    }
}
