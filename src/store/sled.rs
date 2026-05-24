use async_trait::async_trait;
use anyhow::Result;
use bytes::Bytes;
use cid::Cid;
use crate::block::Block;
use super::BlockStore;

pub struct SledBlockStore {
    db: sled::Db,
}

impl SledBlockStore {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn with_db(db: sled::Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BlockStore for SledBlockStore {
    async fn get(&self, cid: &Cid) -> Result<Option<Block>> {
        let key = cid.to_bytes();
        match self.db.get(key)? {
            Some(data) => {
                let block = Block::from_parts(*cid, Bytes::from(data.to_vec()));
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    async fn put(&self, block: &Block) -> Result<()> {
        let key = block.cid().to_bytes();
        self.db.insert(key, block.data().to_vec())?;
        self.db.flush()?;
        Ok(())
    }

    async fn has(&self, cid: &Cid) -> Result<bool> {
        let key = cid.to_bytes();
        Ok(self.db.contains_key(key)?)
    }

    async fn remove(&self, cid: &Cid) -> Result<()> {
        let key = cid.to_bytes();
        self.db.remove(key)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Cid>> {
        let mut cids = Vec::new();
        for item in self.db.iter() {
            let (key, _) = item?;
            let cid = Cid::read_bytes(&mut std::io::Cursor::new(key.as_ref()))?;
            cids.push(cid);
        }
        Ok(cids)
    }
}
