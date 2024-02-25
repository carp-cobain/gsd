use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub mod api;
pub mod config;
pub mod domain;
pub mod repo;
pub mod util;

/// Project level error type
#[derive(thiserror::Error, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("invalid argument: {message}")]
    InvalidArgument { message: String },
    #[error("internal error: {message}")]
    Internal { message: String },
    #[error("not found error: {message}")]
    NotFound { message: String },
}

/// Project level result type
pub type Result<T> = std::result::Result<T, Error>;

/// Map error types to http status codes.
impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::InvalidArgument { .. } => StatusCode::BAD_REQUEST,
            Error::Internal { message } => {
                log::error!("internal server error: {}", message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

/// Map error into a http response
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let jval = serde_json::to_value(&self).unwrap_or_default();
        let status: StatusCode = self.into();
        (status, Json(jval)).into_response()
    }
}
