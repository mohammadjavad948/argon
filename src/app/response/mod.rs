
use axum::http::StatusCode;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SimpleResponse {
    pub message: String
}

argon_macros::response! {
    StatusCode::OK = String, "user found"
}

