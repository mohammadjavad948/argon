use std::net::SocketAddr;

use axum::Extension;
use sea_orm::{Database, DatabaseConnection};

use crate::config::app::AppConfig;

pub async fn init_server() -> anyhow::Result<()> {
    crate::docs::generate_docs().await?;

    let database_url = AppConfig::database_url().await;

    let db: DatabaseConnection = Database::connect(database_url).await?;

    let port = AppConfig::port().await;

    // Build the router
    let app = crate::routes::routes()
        .layer(Extension(db));

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
