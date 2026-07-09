//! `AgentId` — a validated newtype identifying an agent/persona.
//!
//! Wrapping the primitive gives compile-time safety (per CONVENTIONS §3) and a
//! single validation point.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced when constructing an [`AgentId`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentIdError {
    /// The identifier was empty or whitespace-only.
    #[error("agent id must not be empty")]
    Empty,
}

/// A non-empty, immutable agent identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// Construct an [`AgentId`], rejecting empty/whitespace-only values.
    pub fn new(value: impl Into<String>) -> Result<Self, AgentIdError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AgentIdError::Empty);
        }
        Ok(Self(value))
    }

    /// Borrow the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_non_empty() {
        let id = AgentId::new("Maestro").expect("valid id");
        assert_eq!(id.as_str(), "Maestro");
        assert_eq!(id.to_string(), "Maestro");
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(AgentId::new(""), Err(AgentIdError::Empty));
        assert_eq!(AgentId::new("   "), Err(AgentIdError::Empty));
    }

    #[test]
    fn equal_ids_are_equal() {
        assert_eq!(AgentId::new("a").unwrap(), AgentId::new("a").unwrap());
        assert_ne!(AgentId::new("a").unwrap(), AgentId::new("b").unwrap());
    }
}
