mod app;
mod bootstrap;
mod routes;
mod docs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::init_base().await;

    bootstrap::init_server().await?;

    Ok(())
}
