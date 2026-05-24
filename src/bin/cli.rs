use ksh::api::cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    cli::run().await
}
