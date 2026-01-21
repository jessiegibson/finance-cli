//! Additional error type definitions and utilities.

use super::Error;

/// Extension trait for adding context to errors.
pub trait ErrorContext<T> {
    /// Add context to an error.
    fn context(self, msg: &str) -> Result<T, Error>;

    /// Add context with a closure (lazy evaluation).
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T, Error>;
}

impl<T, E: Into<Error>> ErrorContext<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T, Error> {
        self.map_err(|e| {
            let inner = e.into();
            Error::Internal(format!("{}: {}", msg, inner))
        })
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T, Error> {
        self.map_err(|e| {
            let inner = e.into();
            Error::Internal(format!("{}: {}", f(), inner))
        })
    }
}

/// Format an error for display to the user.
pub fn format_error(error: &Error) -> String {
    let mut output = format!("Error: {}", error);

    if let Some(suggestion) = error.suggestion() {
        output.push_str(&format!("\n\nSuggestion: {}", suggestion));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_suggestion() {
        let err = Error::Config("missing field".to_string());
        assert!(err.suggestion().is_some());
    }

    #[test]
    fn test_is_recoverable() {
        assert!(Error::InvalidInput("bad input".to_string()).is_recoverable());
        assert!(!Error::Internal("internal".to_string()).is_recoverable());
    }
}
