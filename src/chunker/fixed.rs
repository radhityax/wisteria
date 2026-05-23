use anyhow::Result;
use bytes::Bytes;
use super::Chunker;

pub struct FixedChunker {
    pub chunk_size: usize,
}

impl FixedChunker {
    pub fn new(chunk_size: usize) -> Self { Self { chunk_size } }
}

impl Default for FixedChunker {
    fn default() -> Self { Self { chunk_size: 256 * 1024 } }
}

impl Chunker for FixedChunker {
    fn chunk(&self, data: &[u8]) -> Result<Vec<Bytes>> {
        Ok(data.chunks(self.chunk_size).map(Bytes::copy_from_slice).collect())
    }
}
