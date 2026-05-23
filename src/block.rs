use anyhow::{Result, ensure};
use bytes::Bytes;
use cid::{Cid, Version};
use multihash::Multihash;
use multihash_codetable::{Code, MultihashDigest};

#[derive(Debug, Clone)]
pub struct Block {
    cid: Cid,
    data: Bytes,
}

impl Block {
    /* hash the data with sha256, then create a CID v1, then save the block :) */
    pub fn new(data: impl Into<Bytes>) -> Result<Self> {
        let data = data.into();
        let mh = Code::Sha2_256.digest(&data);
        // https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
        let cid = Cid::new_v1(0x55, mh);
        Ok(Self { cid, data })
    }
    /* create block from cid + data that we have knew (without hash) :) */
    pub fn from_parts(cid: Cid, data: Bytes) -> Self {
        Self { cid, data }
    }

    pub fn cid(&self) -> &Cid { &self.cid }
    pub fn data(&self) -> &Bytes { &self.data }
    pub fn size(&self) -> u64 { self.data.len() as u64 }

    /* verification :) */
    pub fn verify(&self) -> Result<bool> {
        let mh = Code::Sha2_256.digest(&self.data);
        Ok(*self.cid.hash() == mh)
    }
}

/* hash data -> multihash sha2-256 */
pub fn hash_data(data: &[u8]) -> Multihash<64> {
    Code::Sha2_256.digest(data)
}

/* create cid v0 from multihash */
pub fn cid_v0_from_hash(mh: &Multihash<64>) -> Result<Cid> {
    Cid::new_v0(*mh).map_err(Into::into)
}

pub fn cid_v1_raw(mh: Multihash<64>) -> Cid {
    Cid::new_v1(0x55, mh)
}

pub fn parse_cid(s: &str) -> Result<Cid> {
    s.parse::<Cid>().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_create_and_verify() {
        let block = Block::new(b"hello world").unwrap();
        assert!(block.verify().unwrap());
        assert_eq!(block.size(), 11);
    }

    #[test]
    fn test_cid_roundtrip() {
        let block = Block::new(b"test data").unwrap();
        let s = block.cid().to_string();
        let parsed = parse_cid(&s).unwrap();
        assert_eq!(*block.cid(), parsed);
    }

    #[test]
    fn test_cid_v1_raw() {
        let mh = hash_data(b"hello");
        let cid = cid_v1_raw(mh);
        assert_eq!(cid.version(), Version::V1);
        assert_eq!(cid.codec(), 0x55);
    }
}

