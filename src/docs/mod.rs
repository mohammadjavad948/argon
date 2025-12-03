use utoipa::OpenApi;

use crate::app::controller::*;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/", api = TestControllerApi)
    ),
    info(description = "API Docs")
)]
pub struct MainApiDoc;
