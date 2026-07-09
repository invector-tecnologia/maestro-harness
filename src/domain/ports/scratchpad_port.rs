//! Port for the shared scratchpad.

use crate::domain::models::{AgentId, Scratchpad};

/// Read/write access to the shared scratchpad.
pub trait ScratchpadPort: Send + Sync {
    fn write(&self, key: &str, value: &str, author: &AgentId);
    fn read(&self, key: &str) -> Option<String>;
    fn snapshot(&self) -> Scratchpad;
}
