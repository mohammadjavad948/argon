use std::net::SocketAddr;

use sea_orm::{Database, DatabaseConnection};


pub async fn init_server() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").map_err(|err| {
            anyhow::anyhow!("cannot read `DATABASE_URL`: {:?}", err)
    })?;

    let db: DatabaseConnection = Database::connect(database_url).await?;

    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| {
            tracing::warn!("cannot read `SERVER_PORT` defaulting to `3000`");

            "3000".into()
        })
        .parse()
        .unwrap_or_else(|_| {
            tracing::error!("cannot parse `SERVER_PORT`. defaulting to 3000");
            3000
        });

    // Build the router
    let app = crate::routes::routes().await;

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
