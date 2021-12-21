use axum::{
    async_trait,
    body::{Bytes, HttpBody},
    extract::{FromRequest, RequestParts},
    http::{Response, StatusCode},
    response::IntoResponse,
    BoxError, Json,
};
use http_body::Full;
use serde::de::DeserializeOwned;
use validator::Validate;

use super::{errors::AppError, validate::ValidatedRequest};

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            AppError::Validation(_) => {
                let msg = format!("Input validation error: [{}]", self).replace('\n', ", ");
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::JsonRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedRequest<T>
where
    T: DeserializeOwned + Validate,
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req).await?;
        value.validate()?;
        Ok(ValidatedRequest(value))
    }
}
