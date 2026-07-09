//! `SessionStore` — port for persisting and loading agent session transcripts.

use crate::domain::models::Message;
use thiserror::Error;

/// A persisted session transcript.
#[derive(Debug, Clone)]
pub struct SessionTranscript {
    /// Demand fingerprint (SHA-256 hex of the original demand).
    pub fingerprint: String,
    /// The original demand string.
    pub demand: String,
    /// Agent transcripts keyed by persona id.
    pub transcripts: Vec<AgentTranscript>,
}

/// Transcript for a single agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTranscript {
    /// Persona id.
    pub agent_id: String,
    /// Messages from the agent's memory at session end.
    pub messages: Vec<Message>,
    /// Eviction summary, if any.
    pub summary: Option<String>,
}

/// Errors from session store operations.
#[derive(Debug, Error)]
pub enum SessionStoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(String),
}

/// Port for session transcript storage.
pub trait SessionStore: Send + Sync {
    /// Save a session transcript.
    fn save(&self, transcript: &SessionTranscript) -> Result<(), SessionStoreError>;
    /// Load a prior session transcript by demand fingerprint.
    fn load(&self, fingerprint: &str) -> Result<Option<SessionTranscript>, SessionStoreError>;
}
