
pub async fn init_tracing(){
    // Initialize tracing with customizable log level from RUST_LOG environment variable
    // Default to "info" if not set
    let default_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&default_level)),
        )
        .init();
}
