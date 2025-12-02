pub struct TestController;

#[argon_macros::controller]
impl TestController {
    #[argon_macros::get("/hello")]
    pub async fn index() -> anyhow::Result<()> {
        Ok(())
    }
}
