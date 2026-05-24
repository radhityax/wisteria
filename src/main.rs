use ksh::p2p::P2pNode;
use ksh::store::memory::MemoryBlockStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let store = Arc::new(MemoryBlockStore::new());

    let mut node = P2pNode::new(store)?;
    println!("Peer ID: {}", node.peer_id);

    node.listen_on("/ip4/0.0.0.0/tcp/0")?;
    node.run().await
}
