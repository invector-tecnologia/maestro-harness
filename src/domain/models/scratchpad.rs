//! Shared scratchpad (blackboard pattern) for inter-agent state sharing.

use crate::domain::models::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single write recorded in the scratchpad log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScratchpadEntry {
    pub key: String,
    pub value: String,
    pub author: AgentId,
}

/// A shared key–value workspace visible to all agents in a cycle.
#[derive(Debug, Clone, Default)]
pub struct Scratchpad {
    state: BTreeMap<String, String>,
    log: Vec<ScratchpadEntry>,
}

impl Scratchpad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write a key–value pair, recording who wrote it.
    pub fn write(&mut self, key: impl Into<String>, value: impl Into<String>, author: AgentId) {
        let key = key.into();
        let value = value.into();
        self.state.insert(key.clone(), value.clone());
        self.log.push(ScratchpadEntry { key, value, author });
    }

    /// Read a value by key.
    pub fn read(&self, key: &str) -> Option<&str> {
        self.state.get(key).map(|s| s.as_str())
    }

    /// Snapshot the entire state.
    pub fn snapshot(&self) -> &BTreeMap<String, String> {
        &self.state
    }

    /// The full write log (append-only, oldest first).
    pub fn log(&self) -> &[ScratchpadEntry] {
        &self.log
    }

    /// Format the scratchpad as context for injection into an LLM prompt.
    pub fn as_prompt_context(&self) -> String {
        if self.state.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("[Shared Scratchpad]\n");
        for (k, v) in &self.state {
            ctx.push_str(&format!("  {k}: {v}\n"));
        }
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let mut pad = Scratchpad::new();
        let author = AgentId::new("tester").unwrap();
        pad.write("key1", "val1", author.clone());
        assert_eq!(pad.read("key1"), Some("val1"));
    }

    #[test]
    fn overwrite_replaces_value() {
        let mut pad = Scratchpad::new();
        let author = AgentId::new("tester").unwrap();
        pad.write("key1", "val1", author.clone());
        pad.write("key1", "val2", author.clone());
        assert_eq!(pad.read("key1"), Some("val2"));
    }

    #[test]
    fn log_records_all_writes() {
        let mut pad = Scratchpad::new();
        let author = AgentId::new("tester").unwrap();
        pad.write("k1", "v1", author.clone());
        pad.write("k1", "v2", author.clone());
        assert_eq!(pad.log().len(), 2);
        assert_eq!(pad.log()[0].value, "v1");
        assert_eq!(pad.log()[1].value, "v2");
    }

    #[test]
    fn as_prompt_context_is_empty_when_no_entries() {
        let pad = Scratchpad::new();
        assert_eq!(pad.as_prompt_context(), "");
    }
}
