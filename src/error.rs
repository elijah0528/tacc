use pyo3::exceptions::PyException;
use pyo3::prelude::*;

/// Library error type, auto-generates error messages. Converts to Python exceptions.
#[derive(thiserror::Error, Debug)]
pub enum TaccError {
    /// Tokenizer mapping missing
    #[error("Mapping for tokenizer '{tokenizer}' not found. Available: {}", available.join(", "))]
    MappingNotFound {
        tokenizer: String,
        available: Vec<String>,
    },
    /// Payload corrupted/invalid
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    /// JSON parsing failure, auto-converted from serde_json
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    /// Token ID not found
    #[error("Token ID {0} not found in mapping")]
    TokenIdNotFound(u32),
    /// Mapped ID not found in reverse mapping
    #[error("Mapped ID {0} not found in reverse mapping")]
    MappedIdNotFound(i32),
    /// Token string not found
    #[error("Token '{0}' not found in mapping")]
    TokenNotFound(String),
}

/// Map Rust error to Python: lookup errors → KeyError, others → Exception
impl From<TaccError> for PyErr {
    fn from(err: TaccError) -> PyErr {
        match err {
            TaccError::TokenIdNotFound(_)
            | TaccError::MappedIdNotFound(_)
            | TaccError::TokenNotFound(_) => {
                pyo3::exceptions::PyKeyError::new_err(err.to_string())
            }
            _ => PyException::new_err(err.to_string()),
        }
    }
}
