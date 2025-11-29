mod bootstrap;
mod core;


#[tokio::main]
async fn main() {
    bootstrap::init_base().await;

    bootstrap::init_server().await;
}
