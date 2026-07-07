//! Typed runtime errors (TASK 004).
//!
//! Application-level failures propagate through `?` as these variants; `anyhow`
//! aggregation is confined to the CLI boundary.

use thiserror::Error;

use crate::domain::models::message::MessageError;
use crate::domain::ports::LlmError;

/// Errors surfaced while orchestrating agents.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// A provider call failed.
    #[error(transparent)]
    Provider(#[from] LlmError),
    /// Constructing a message failed.
    #[error(transparent)]
    Message(#[from] MessageError),
    /// No provider was available for the requested work (SENSE failed).
    #[error("no ready provider: {0}")]
    NoReadyProvider(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_message_error() {
        let err: RuntimeError = MessageError::EmptyContent.into();
        assert!(matches!(err, RuntimeError::Message(_)));
    }
}
