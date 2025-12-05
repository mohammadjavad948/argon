use axum::{response::IntoResponse, http::StatusCode, Json};

#[derive(utoipa::IntoResponses)]
pub enum CoreResponse<T, N, U, I>
where T: serde::Serialize + utoipa::ToSchema,
      N: serde::Serialize + utoipa::ToSchema,
      U: serde::Serialize + utoipa::ToSchema,
      I: serde::Serialize + utoipa::ToSchema 
{
    #[response(status = 200, description = "Ok")]
    Ok(T),

    #[response(status = 404, description = "Not found")]
    NotFound(N),

    #[response(status = 401, description = "Unauthorized")]
    Unauthorized(U),

    #[response(status = 500, description = "Internal server error")]
    InternalServerError(I),
}

impl<T, N, U, I> IntoResponse for CoreResponse<T, N, U, I>
where T: serde::Serialize + utoipa::ToSchema,
      N: serde::Serialize + utoipa::ToSchema,
      U: serde::Serialize + utoipa::ToSchema,
      I: serde::Serialize + utoipa::ToSchema 
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Self::NotFound(error) => (StatusCode::NOT_FOUND, Json(error)).into_response(),
            Self::Unauthorized(error) => (StatusCode::UNAUTHORIZED, Json(error)).into_response(),
            Self::InternalServerError(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response(),
        }
    }
}
