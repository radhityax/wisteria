use std::sync::Arc;
use anyhow::Result;
use bytes::Bytes;
use cid::Cid;
use crate::block::Block;
use crate::store::BlockStore;
use crate::chunker::Chunker;

pub struct DagBuilder {
    store: Arc<dyn BlockStore>,
    chunker: Box<dyn Chunker>,
}

impl DagBuilder {
    pub fn new(store: Arc<dyn BlockStore>, chunker: Box<dyn Chunker>) -> Self {
        Self { store, chunker }
    }

    pub async fn add_bytes(&self, data: Bytes) -> Result<Cid> {
        let cursor = std::io::Cursor::new(data);
        let chunks = self.chunker.chunk(cursor.get_ref())?;

        if chunks.len() == 1 {
            let block = Block::new(chunks.into_iter().next().unwrap())?;
            self.store.put(&block).await?;
            Ok(*block.cid())
        } else {
            let mut child_cids = Vec::new();
            let mut total_size = 0u64;

            for chunk in chunks {
                let len = chunk.len() as u64;
                let block = Block::new(chunk)?;
                child_cids.push(*block.cid());
                total_size += len;
                self.store.put(&block).await?;
            }

            let mut parent_data = Vec::new();
            parent_data.extend_from_slice(&(child_cids.len() as u32).to_le_bytes());
            for cid in &child_cids {
                let cid_bytes = cid.to_bytes();
                parent_data.extend_from_slice(&(cid_bytes.len() as u32).to_le_bytes());

                parent_data.extend_from_slice(&cid_bytes);
            }
            parent_data.extend_from_slice(&total_size.to_le_bytes());

            let parent_block = Block::new(Bytes::from(parent_data))?;
            self.store.put(&parent_block).await?;
            Ok(*parent_block.cid())
        }
    }
}

pub mod resolver;
pub use resolver::DagResolver;
