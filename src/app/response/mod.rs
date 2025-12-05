#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SimpleResponse {
    pub message: String
}

argon_macros::response! {
    BasicResponse {
        StatusCode::OK = String, "record found!"
    }
}

