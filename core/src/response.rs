#[derive(serde::Serialize, utoipa::ToSchema, Debug, Clone)]
pub struct BaseErrorResponse<T>
    where T: serde::Serialize + utoipa::ToSchema
{
    message: String,
    detail: Option<T>
}


impl<T> BaseErrorResponse<T> 
    where T: serde::Serialize + utoipa::ToSchema
{
    pub fn new(message: impl Into<String>, detail: impl Into<Option<T>>) -> Self {
        Self {
            message: message.into(),
            detail: detail.into()
        }
    }
}
