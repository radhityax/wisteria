use ksh::p2p::P2pNode;
use ksh::store::memory::MemoryBlockStore;
use ksh::block::parse_cid;
use std::sync::Arc;
use std::time::Duration;
use std::env;
use ksh::store::BlockStore;
use libp2p::Multiaddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: {} <peer_id> <multiaddr> <cid>", args[0]);
        std::process::exit(1);
    }
    let target_peer_id = args[1].parse()?;
    let target_addr = args[2].parse::<Multiaddr>()?;
    let target_cid = parse_cid(&args[3])?;

    let store = Arc::new(MemoryBlockStore::new());
    let mut node = P2pNode::new(store.clone())?;

    node.listen_on("/ip4/0.0.0.0/tcp/0")?;
    node.add_address(target_peer_id, target_addr.clone());

    tokio::time::sleep(Duration::from_secs(2)).await;

    node.request_block(target_peer_id, &target_cid);
    println!("Requested block {}", target_cid);

    let handle = tokio::spawn(async move { node.run().await });
    tokio::time::sleep(Duration::from_secs(2)).await;
    handle.abort();
    if let Some(b) = store.get(&target_cid).await? {
        println!("Got block: {:?}", String::from_utf8_lossy(b.data()));
    } else {
        println!("Block not found yet");
    }

    Ok(())
}
