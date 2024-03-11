use serde::Serialize;

// Http support for errors
mod http;

// Validation support for errors
mod validate;

/// Project level error type
#[derive(thiserror::Error, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("invalid arguments")]
    InvalidArgs { messages: Vec<String> },
    #[error("internal error: {message}")]
    Internal { message: String },
    #[error("not found error: {message}")]
    NotFound { message: String },
}
