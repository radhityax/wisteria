use ksh::block::Block;
use ksh::store::memory::MemoryBlockStore;
use ksh::store::BlockStore;

const DATA: &[u8] = b"the other woman will never have his love to keep\n\
and as the years go by, the other woman will spend her life alone";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = MemoryBlockStore::new();

    let block = Block::new(DATA).unwrap();

    println!("Data: {}", String::from_utf8_lossy(block.data()));
    println!("CID: {}", block.cid());

    store.put(&block).await?;
    assert!(store.has(block.cid()).await?);

    let retrieved = store.get(block.cid()).await?.unwrap();
    assert_eq!(retrieved.data(), block.data());
    assert!(retrieved.verify()?);

    println!("kshblock works!");
    Ok(())
}
