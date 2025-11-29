

pub trait Controller {
    fn router() -> axum::Router;
}
