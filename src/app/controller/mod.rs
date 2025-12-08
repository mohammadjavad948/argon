pub struct TestController;

#[argon_macros::controller]
impl TestController {
    #[argon_macros::get("/hello/{id}")]
    #[argon_macros::utoipa_response(response = crate::app::response::BasicResponse)]
    pub async fn index(axum::extract::Path(id): axum::extract::Path<u64>) -> Result<crate::app::response::BasicResponse, crate::app::response::BasicResponse> {
        unimplemented!()
    }
}
