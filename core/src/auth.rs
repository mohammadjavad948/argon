use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub trait AuthenticatableUser {
    type Username;
    type Password;
    type Id;

    fn get_username(&self) -> Self::Username;
    fn get_password(&self) -> Self::Password;
    fn get_id(&self) -> Self::Id;
}

pub trait Authenticator<T>
where
    T: AuthenticatableUser,
{
    type Token;

    fn attempt(
        &self,
        username: T::Username,
        password: T::Password,
    ) -> impl std::future::Future<Output = anyhow::Result<T>> + Send;
    fn generate_token(&self, user: T) -> impl std::future::Future<Output = Self::Token> + Send;

    fn verify_header_name(&self) -> &'static str;
    fn verify(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<T, StatusCode>> + Send;
}

#[tracing::instrument(level = "debug", skip(request, next))]
pub async fn auth_middleware<T, R>(mut request: Request, next: Next) -> Result<Response, StatusCode>
where
    T: Authenticator<R> + Send + Sync + 'static,
    R: AuthenticatableUser + Send + Sync + Clone + 'static,
{
    let Some(authenticator) = request.extensions().get::<T>() else {
        tracing::error!("no Authenticator Extension available");

        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(header) = request.headers().get(authenticator.verify_header_name()) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let header = header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = authenticator.verify(header).await?;

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}
