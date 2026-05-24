use ksh::p2p::P2pNode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut node = P2pNode::new()?;
    println!("Peer ID: {}", node.peer_id);

    node.listen_on("/ip4/0.0.0.0/tcp/0")?;
    node.run().await
}
