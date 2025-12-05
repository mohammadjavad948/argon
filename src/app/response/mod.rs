

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SimpleResponse {
    pub message: String
}

pub type BasicResponse = argon_core::response::CoreResponse<SimpleResponse, SimpleResponse, SimpleResponse, SimpleResponse>;
