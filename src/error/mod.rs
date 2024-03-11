use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use validator::{ValidationError, ValidationErrors};

/// Project level error type
#[derive(thiserror::Error, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("invalid arguments")]
    InvalidArguments {
        field_errors: HashMap<String, String>,
    },
    #[error("internal error: {message}")]
    Internal { message: String },
    #[error("not found error: {message}")]
    NotFound { message: String },
}

#[derive(Debug, Serialize)]
struct ErrorDto {
    #[serde(rename = "errors")]
    messages: Vec<String>,
}

/// Get the http status code for an error.
fn http_status(err: &Error) -> StatusCode {
    match err {
        Error::NotFound { .. } => StatusCode::NOT_FOUND,
        Error::InvalidArguments { .. } => StatusCode::BAD_REQUEST,
        Error::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Map error types to http status codes.
impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        http_status(&err)
    }
}

impl From<Error> for ErrorDto {
    fn from(err: Error) -> Self {
        let messages = match err {
            Error::NotFound { message } => vec![message],
            Error::Internal { message } => {
                log::error!("internal error: {}", message);
                vec![message]
            }
            Error::InvalidArguments { field_errors } => field_errors
                .into_iter()
                .map(|pair| format!("{}: {}", pair.0, pair.1))
                .collect(),
        };
        ErrorDto { messages }
    }
}
/// Map error into a http response
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = http_status(&self);
        let dto: ErrorDto = self.into();
        (status, Json(dto)).into_response()
    }
}

/// Map validation errors into project errors.
impl From<ValidationErrors> for Error {
    fn from(errors: ValidationErrors) -> Self {
        Error::InvalidArguments {
            field_errors: errors
                .field_errors()
                .iter()
                .map(|pair| (pair.0.to_string(), summarize(pair.1)))
                .collect(),
        }
    }
}

/// Summarize a set of validation errors into a CSV string.
fn summarize(errors: &[ValidationError]) -> String {
    let messages: Vec<String> = errors
        .iter()
        .map(|error| error.to_owned().message.unwrap_or("invalid field".into()))
        .map(|s| s.to_string())
        .collect();

    messages.join(", ")
}
