use ksh::api::http::{self, HttpConfig};
use ksh::store::memory::MemoryBlockStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let store = Arc::new(MemoryBlockStore::new());
    let config = HttpConfig {
        port: std::env::args()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080),
            store,
            ..Default::default()
    };
    http::serve(config).await
}
