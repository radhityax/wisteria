use std::sync::Arc;
use std::time::Duration;
use std::io::Write;
use anyhow::{Result, Context};
use libp2p::Multiaddr;

use crate::block::{Block, parse_cid};
use crate::store::BlockStore;
use crate::store::sled::SledBlockStore;
use crate::pin::{PinManager, PinMode};
use crate::p2p::P2pNode;

fn default_store() -> Arc<dyn BlockStore> {
    let path = dirs_or_default();
    Arc::new(SledBlockStore::new(&path).expect("failed to open sled store"))
}

fn dirs_or_default() -> String {
    let home = std::env::var("KSHDIR").unwrap_or_else(|_| {
        let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp/ksh".into());
        format!("{}/.ksh/store", base)
    });
    std::fs::create_dir_all(&home).ok();
    format!("{}/blockstore", home)
}

pub fn print_usage() {
    eprintln!(r#"Usage: ksh-cli <command> [args]

Commands:
  add <file>          Store a file, print its CID
  get <cid>           Print block data by CID
  list                List all CIDs in store
  pin <cid>           Pin a CID
  pin ls              List all pins
  unpin <cid>         Unpin a CID
  provide <cid>       Announce block on network
  fetch <cid> <peer> <addr>  Fetch block from peer
  connect <peer> <addr>      Dial a peer
  id                  Show local peer ID"#);
}

pub async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { print_usage(); return Ok(()); }

    let store = default_store();
    let pinner = PinManager::new();

    match args[1].as_str() {
        "add" => {
            let data = if args.len() > 2 {
                std::fs::read(&args[2]).context("read file")?
            } else {
                eprintln!("usage: ksh-cli add <file>");
                return Ok(());
            };
            let block = Block::new(data)?;
            store.put(&block).await?;
            println!("{}", block.cid());
            Ok(())
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("usage: ksh-cli get <cid>");
                return Ok(());
            }
            let cid = parse_cid(&args[2])?;
            match store.get(&cid).await? {
                Some(block) => {
                    std::io::stdout().write_all(block.data())?;
                }
                None => eprintln!("block not found: {}", cid),
            }
            Ok(())
        }
        "list" => {
            for cid in store.list().await? {
                println!("{}", cid);
            }
            Ok(())
        }
        "pin" => {
            if args.len() < 3 {
                eprintln!("usage: ksh-cli pin <cid>");
                return Ok(());
            }
            if args[2] == "ls" {
                for (cid, mode) in pinner.list() {
                    println!("{:?} {}", mode, cid);
                }
            } else {
                let cid = parse_cid(&args[2])?;
                pinner.pin(cid, PinMode::Direct)?;
                println!("pinned {}", cid);
            }
            Ok(())
        }
        "unpin" => {
            if args.len() < 3 {
                eprintln!("usage: ksh-cli unpin <cid>");
                return Ok(());
            }
            let cid = parse_cid(&args[2])?;
            pinner.unpin(&cid)?;
            println!("unpinned {}", cid);
            Ok(())
        }
        "provide" => {
            if args.len() < 3 {
                eprintln!("usage: ksh-cli provide <cid>");
                return Ok(());
            }
            let cid = parse_cid(&args[2])?;
            let mut node = P2pNode::new(store.clone())?;
            println!("peer id: {}", node.peer_id);
            node.listen_on("/ip4/0.0.0.0/tcp/0")?;
            println!("providing {}", cid);
            node.start_providing(&cid)?;
            node.run().await
        }
        "fetch" => {
            if args.len() < 5 {
                eprintln!("usage: ksh-cli fetch <cid> <peer_id> <multiaddr>");
                return Ok(());
            }
            let cid = parse_cid(&args[2])?;
            let peer = args[3].parse()?;
            let addr: Multiaddr = args[4].parse()?;
            let mut node = P2pNode::new(store.clone())?;
            node.listen_on("/ip4/0.0.0.0/tcp/0")?;
            node.add_address(peer, addr);
            tokio::time::sleep(Duration::from_millis(500)).await;
            node.request_block(peer, &cid);
            let handle = tokio::spawn(async move { node.run().await });
            tokio::time::sleep(Duration::from_secs(3)).await;
            handle.abort();
            match store.get(&cid).await? {
                Some(b) => println!("got block: {}",
                    String::from_utf8_lossy(b.data())),
                None => eprintln!("block not found after fetch"),
            }
            Ok(())
        }
        "connect" => {
            if args.len() < 4 {
                eprintln!("usage: ksh-cli connect <peer_id> <multiaddr>");
                return Ok(());
            }
            let peer = args[2].parse()?;
            let addr: Multiaddr = args[3].parse()?;
            let mut node = P2pNode::new(store.clone())?;
            node.listen_on("/ip4/0.0.0.0/tcp/0")?;
            println!("connecting to {} at {}", peer, addr);
            node.add_address(peer, addr.clone());
            node.dial(&addr.to_string())?;
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok(())
        }
        "id" => {
            let mut node = P2pNode::new(store.clone())?;
            node.listen_on("/ip4/0.0.0.0/tcp/0")?;
            println!("peer id: {}", node.peer_id);
            Ok(())
        }
        _ => { print_usage(); Ok(()) }
    }
}

