mod bootstrap;
mod app;
mod routes;


#[tokio::main]
async fn main() {
    bootstrap::init_base().await;

    bootstrap::init_server().await;
}
