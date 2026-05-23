use std::sync::Arc;
use anyhow::Result;
use bytes::Bytes;
use cid::Cid;
use crate::store::BlockStore;

pub struct DagResolver {
    store: Arc<dyn BlockStore>,
}

impl DagResolver {
    pub fn new(store: Arc<dyn BlockStore>) -> Self {
        Self { store }
    }

    pub async fn cat(&self, root: &Cid) -> Result<Bytes> {
        let mut stack = vec![*root];
        let mut result = Vec::new();

        while let Some(cid) = stack.pop() {
            let block = self.store.get(&cid).await?
                .ok_or_else(|| anyhow::anyhow!("block not found: {}", cid))?;
            let data = block.data();

            if data.len() >= 4 {
                let num_children = u32::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                ]) as usize;

                if num_children > 0 && data.len() >= 4 + num_children * 4 {
                    let mut offset = 4usize;
                    let mut child_cids = Vec::new();
                    for _ in 0..num_children {
                        if offset + 4 > data.len() { break; }
                        let cid_len = u32::from_le_bytes([
                            data[offset], data[offset+1], data[offset+2], data[offset+3],
                        ]) as usize;
                        offset += 4;
                        if offset + cid_len > data.len() { break; }
                        let cid = Cid::read_bytes(&data[offset..offset + cid_len])?;
                        child_cids.push(cid);
                        offset += cid_len;
                    }
                    stack.extend(child_cids.into_iter().rev());
                    continue;
                }
            }

            result.extend_from_slice(data);
        }

        Ok(Bytes::from(result))
    }
}
