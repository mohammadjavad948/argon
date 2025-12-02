use argon_core::controller::Controller;
use axum::Router;

use crate::app::controller::TestController;

pub fn routes() -> Router {
    Router::new().nest("/", TestController::router())
}
