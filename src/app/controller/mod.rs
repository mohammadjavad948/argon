pub struct Test;

#[argon_macros::controller]
impl Test {

    #[argon_macros::get("/hello")]
    pub async fn index() {

    }
}
