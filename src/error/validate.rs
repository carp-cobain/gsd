use super::Error;
use validator::{ValidationError, ValidationErrors};

/// Map validation errors into project errors.
impl From<ValidationErrors> for Error {
    fn from(errors: ValidationErrors) -> Self {
        Error::InvalidArgs {
            messages: errors
                .field_errors()
                .iter()
                .map(|pair| format!("{}: {}", pair.0, summarize(pair.1)))
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
