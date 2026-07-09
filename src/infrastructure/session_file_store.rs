//! JSON file adapter for `SessionStore` (infrastructure layer).

use crate::domain::ports::session_store::*;
use std::path::{Path, PathBuf};

/// Persists session transcripts to `maestro/sessions/<fingerprint>.json`.
pub struct JsonSessionStore {
    sessions_dir: PathBuf,
}

impl JsonSessionStore {
    pub fn new(project_root: &Path) -> Self {
        Self {
            sessions_dir: project_root.join("maestro").join("sessions"),
        }
    }
}

impl SessionStore for JsonSessionStore {
    fn save(&self, transcript: &SessionTranscript) -> Result<(), SessionStoreError> {
        std::fs::create_dir_all(&self.sessions_dir).map_err(SessionStoreError::Io)?;
        let path = self
            .sessions_dir
            .join(format!("{}.json", transcript.fingerprint));
        let json = serde_json::to_string_pretty(&transcript.transcripts)
            .map_err(|e| SessionStoreError::Serde(e.to_string()))?;
        std::fs::write(&path, json).map_err(SessionStoreError::Io)?;
        tracing::info!(
            fingerprint = %transcript.fingerprint,
            agents = transcript.transcripts.len(),
            "session transcript saved"
        );
        Ok(())
    }

    fn load(&self, fingerprint: &str) -> Result<Option<SessionTranscript>, SessionStoreError> {
        let path = self.sessions_dir.join(format!("{fingerprint}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(&path).map_err(SessionStoreError::Io)?;
        let transcripts: Vec<AgentTranscript> =
            serde_json::from_str(&json).map_err(|e| SessionStoreError::Serde(e.to_string()))?;
        tracing::info!(
            fingerprint = fingerprint,
            agents = transcripts.len(),
            "prior session transcript loaded"
        );
        Ok(Some(SessionTranscript {
            fingerprint: fingerprint.to_string(),
            demand: String::new(), // demand not stored separately; fingerprint is the key
            transcripts,
        }))
    }
}
