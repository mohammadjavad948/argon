use argon_core::controller::Controller;
use axum::Router;

use crate::app::controller::TestController;



pub async fn routes() -> Router {
    Router::new()
        .nest("/", TestController::router().await)
}
