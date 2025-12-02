use argon_core::controller::Controller;
use axum::Router;

use crate::app::controller::TestController;

pub fn routes() -> Router {
    Router::new()
        .nest("/", TestController::router())
        .layer(axum::middleware::from_fn(
            crate::app::middleware::auth::auth_middleware::<
                crate::app::middleware::auth::BasicAuthenticator,
                crate::app::middleware::auth::BasicUser,
            >,
        ))
}
