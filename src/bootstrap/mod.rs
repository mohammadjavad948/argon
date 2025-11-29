mod env;
mod tracing;
mod server;

pub async fn init_base(){
    env::init_env().await;
    tracing::init_tracing().await;
}
