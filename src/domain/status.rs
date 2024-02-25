use serde::Serialize;
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, Display, Serialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Incomplete,
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn status_from_string() {
        let result = Status::from_str("incomplete").unwrap();
        assert_eq!(result, Status::Incomplete);
        let result = Status::from_str("complete").unwrap();
        assert_eq!(result, Status::Complete);
    }

    #[test]
    fn status_from_string_error() {
        let err = Status::from_str("xomplete").unwrap_err();
        assert_eq!(err.to_string(), "Matching variant not found");
    }

    #[test]
    fn status_to_string() {
        assert_eq!(Status::Complete.to_string(), "complete");
        assert_eq!(Status::Incomplete.to_string(), "incomplete");
    }
}
