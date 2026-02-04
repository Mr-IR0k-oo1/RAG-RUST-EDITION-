use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::dataset::Chunk;

/// Writes dataset chunks to JSONL format
pub struct DatasetWriter {
    writer: BufWriter<File>,
}

impl DatasetWriter {
    /// Create a new dataset writer for JSONL output
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self { writer })
    }

    /// Write a single chunk to JSONL
    pub fn write_chunk(&mut self, chunk: &Chunk) -> Result<()> {
        let line = json!({
            "id": chunk.id,
            "document_id": chunk.document_id,
            "content": chunk.content,
            "chunk_index": chunk.chunk_index,
            "total_chunks": chunk.total_chunks,
            "metadata": {
                "filename": chunk.metadata.filename,
                "source": chunk.metadata.source,
                "token_count": chunk.metadata.token_count,
                "created_at": chunk.metadata.created_at,
            }
        });

        use std::io::Write;
        writeln!(self.writer, "{}", line.to_string())?;
        Ok(())
    }

    /// Write multiple chunks to JSONL
    pub fn write_chunks(&mut self, chunks: &[Chunk]) -> Result<()> {
        for chunk in chunks {
            self.write_chunk(chunk)?;
        }
        Ok(())
    }

    /// Flush buffer to disk
    pub fn flush(&mut self) -> Result<()> {
        use std::io::Write;
        self.writer.flush()?;
        Ok(())
    }
}
