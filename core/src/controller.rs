

pub trait Controller {
    fn router() -> impl std::future::Future<Output = axum::Router> + Send;
}
