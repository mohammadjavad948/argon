mod env;
mod server;
mod tracing;

pub use server::init_server;

pub async fn init_base() {
    env::init_env().await;
    tracing::init_tracing().await;
}
