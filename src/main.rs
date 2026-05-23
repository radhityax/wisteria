use std::sync::Arc;
use bytes::Bytes;
use ksh::store::memory::MemoryBlockStore;
use ksh::chunker::fixed::FixedChunker;
use ksh::dag::{DagBuilder, DagResolver};
use ksh::pin::{PinManager, PinMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(MemoryBlockStore::new());

    let builder = DagBuilder::new(
        store.clone(),
        Box::new(FixedChunker::new(256 * 1024)),
    );
    let resolver = DagResolver::new(store);

    let data = Bytes::from(vec![0xABu8; 1024]);
    let root = builder.add_bytes(data.clone()).await?;
    println!("Root CID (1KB): {}", root);

    let result: Bytes = resolver.cat(&root).await?;
    assert_eq!(result.len(), 1024);
    assert_eq!(result[0], 0xAB);
    println!("Cat OK - {} bytes", result.len());

    let big_data = Bytes::from(vec![0x42u8; 512 * 1024]);
    let root2 = builder.add_bytes(big_data.clone()).await?;
    println!("Root CID (512KB): {}", root2);

    let result2: Bytes = resolver.cat(&root2).await?;
    assert_eq!(result2.len(), 512 * 1024);
    println!("Multi chunk cat OK - {} bytes", result2.len());

    let pinner = PinManager::new();
    pinner.pin(root, PinMode::Recursive)?;
    pinner.pin(root2, PinMode::Direct)?;
    println!("Pinned: {:?}", pinner.list());

    assert!(pinner.is_pinned(&root));
    assert!(pinner.is_pinned(&root2));
    pinner.unpin(&root2)?;
    assert!(!pinner.is_pinned(&root2));
    println!("PinManager works!");
    Ok(())
}
