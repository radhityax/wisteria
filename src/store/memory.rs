use std::sync::Mutex;
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use cid::Cid;
use crate::block::Block;
use super::BlockStore;

pub struct MemoryBlockStore {
    blocks: Mutex<HashMap<Cid, Block>>
}

impl MemoryBlockStore {
    pub fn new() -> Self {
        Self { blocks: Mutex::new(HashMap::new()) }
    }
}

#[async_trait]
impl BlockStore for MemoryBlockStore {
    async fn get(&self, cid: &Cid) -> Result<Option<Block>> {
        let map = self.blocks.lock().unwrap();
        Ok(map.get(cid).cloned())
    }

    async fn put(&self, block: &Block) -> Result<()> {
        let mut map = self.blocks.lock().unwrap();
        map.insert(*block.cid(), block.clone());
        Ok(())
    }

    async fn has(&self, cid: &Cid) -> Result<bool> {
        let map = self.blocks.lock().unwrap();
        Ok(map.contains_key(cid))
    }

    async fn remove(&self, cid: &Cid) -> Result<()> {
        let mut map = self.blocks.lock().unwrap();
        map.remove(cid);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Cid>> {
        let map = self.blocks.lock().unwrap();
        Ok(map.keys().copied().collect())
    }
}
