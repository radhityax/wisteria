use ksh::p2p::P2pNode;
use ksh::store::memory::MemoryBlockStore;
use ksh::block::Block;
use std::sync::Arc;
use ksh::store::BlockStore;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <message>", args[0]);
        std::process::exit(1);
    }

    let target_message = args[1].as_bytes().to_vec();
    let store = Arc::new(MemoryBlockStore::new());
    let mut node = P2pNode::new(store.clone())?;
    println!("peer id: {}", node.peer_id);

    let block = Block::new(target_message).unwrap();
    let cid = *block.cid();
    store.put(&block).await?;
    node.start_providing(&cid)?;
    println!("Stored block: {}", cid);

    node.listen_on("/ip4/0.0.0.0/tcp/0")?;
    node.run().await
}

