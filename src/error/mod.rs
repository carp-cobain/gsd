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
    #[error("invalid argument")]
    InvalidArguments {
        field_errors: HashMap<String, String>,
    },
    #[error("internal error: {message}")]
    Internal { message: String },
    #[error("not found error: {message}")]
    NotFound { message: String },
}

/// Map error types to http status codes.
impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::InvalidArguments { .. } => StatusCode::BAD_REQUEST,
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

/// Map validation errors into project errors.
impl From<ValidationErrors> for Error {
    fn from(errors: ValidationErrors) -> Self {
        Error::InvalidArguments {
            field_errors: errors
                .field_errors()
                .into_iter()
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
