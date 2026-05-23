use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use cid::Cid;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    Direct,
    Recursive,
}

pub struct PinManager {
    pins: Mutex<HashMap<Cid, PinMode>>,
}

impl PinManager {
    pub fn new() -> Self {
        Self { pins: Mutex::new(HashMap::new()) }
    }
    pub fn pin(&self, cid: Cid, mode: PinMode) -> Result<()> {
        let mut pins = self.pins.lock().unwrap();
        pins.insert(cid, mode);
        Ok(())
    }
    pub fn is_pinned(&self, cid: &Cid) -> bool {
        let pins = self.pins.lock().unwrap();
        pins.contains_key(cid)
    }
    pub fn unpin(&self, cid: &Cid) -> Result<()> {
        let mut pins = self.pins.lock().unwrap();
        pins.remove(cid);
        Ok(())
    }

    pub fn list(&self) -> Vec<(Cid, PinMode)> {
        let pins = self.pins.lock().unwrap();
        pins.iter().map(|(c, m)| (*c, *m)).collect()
    }
    pub fn all_pinned_cids(&self) -> HashSet<Cid> {
        let pins = self.pins.lock().unwrap();
        pins.keys().copied().collect()
    }
}

