pub mod memory;
pub mod sled;
use async_trait::async_trait;
use anyhow::Result;
use crate::block::Block;
use cid::Cid;

#[async_trait]
pub trait BlockStore: Send + Sync {
    async fn get(&self, cid: &Cid) -> Result<Option<Block>>;
    async fn put(&self, block: &Block) -> Result<()>;
    async fn has(&self, cid: &Cid) -> Result<bool>;
    async fn remove(&self, cid: &Cid) -> Result<()>;
    async fn list(&self) -> Result<Vec<Cid>>;
}
