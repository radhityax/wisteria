pub mod cli;
pub mod http;

use anyhow::Result;
use cid::Cid;

pub fn parse_cid(s: &str) -> Result<Cid> {
    s.trim().parse::<Cid>().map_err(Into::into)
}
