use std::net::SocketAddr;


pub async fn init_server(){
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

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
