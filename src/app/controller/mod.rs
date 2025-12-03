pub struct TestController;

#[argon_macros::controller]
impl TestController {
    #[argon_macros::get("/hello/{id}")]
    pub async fn index(axum::extract::Path(id): axum::extract::Path<u64>,) -> () {
        ()
    }
}
