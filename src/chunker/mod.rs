use anyhow::Result;
use bytes::Bytes;

pub trait Chunker: Send + Sync {
    fn chunk(&self, data: &[u8]) -> Result<Vec<Bytes>>;
}

pub mod fixed;
