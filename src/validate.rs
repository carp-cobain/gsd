use crate::{Error, Result};

/// Validation helpers
pub struct Validate {}

impl Validate {
    /// Max string length allowed by all DB columns for this trivial project.
    const MAX_STR_LEN: usize = 100;
}

impl Validate {
    /// Validate string length. The string cannot be empty or longer than 100 bytes.
    pub fn string_length(value: &str) -> Result<String> {
        let value = value.trim().to_string();
        if value.is_empty() {
            return Err(Error::InvalidArgument {
                message: format!("string is empty"),
            });
        }
        if value.len() > Validate::MAX_STR_LEN {
            return Err(Error::InvalidArgument {
                message: format!("string is too long"),
            });
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_string_success() {
        let result = Validate::string_length(" test ").unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn non_empty_fail() {
        let error = Validate::string_length("  ").unwrap_err();
        assert!(error.to_string().starts_with("invalid argument"));
    }

    #[test]
    fn max_len_fail() {
        let input = "0123456789!".repeat(10);
        let error = Validate::string_length(&input).unwrap_err();
        assert!(error.to_string().starts_with("invalid argument"));
    }
}
