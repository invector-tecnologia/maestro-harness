//! `Message` and `MessageRole` — the unit of work exchanged between agents.
//!
//! Note: `MessageRole` describes *authorship* of a message (system/user/assistant).
//! It is distinct from the cognitive `Role` trait in [`crate::domain::ports`], which
//! defines the `observe → think → act` contract (TASK 010).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::agent_id::AgentId;

/// Who authored a [`Message`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System / instruction context.
    System,
    /// End-user input.
    User,
    /// Model or agent output.
    Assistant,
}

/// Errors produced when constructing a [`Message`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MessageError {
    /// The message content was empty or whitespace-only.
    #[error("message content must not be empty")]
    EmptyContent,
}

/// An immutable message: a role, its content, and an optional authoring agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Authorship role.
    pub role: MessageRole,
    /// The message body (never empty).
    pub content: String,
    /// The agent that authored the message, when applicable.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub author: Option<AgentId>,
    /// If set, this message is addressed to a specific agent.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub recipient: Option<AgentId>,
}

impl Message {
    /// Construct a validated message, rejecting empty content.
    pub fn new(
        role: MessageRole,
        content: impl Into<String>,
        author: Option<AgentId>,
    ) -> Result<Self, MessageError> {
        let content = content.into();
        if content.trim().is_empty() {
            return Err(MessageError::EmptyContent);
        }
        Ok(Self {
            role,
            content,
            author,
            recipient: None,
        })
    }

    /// Convenience constructor for a directed agent-to-agent message.
    pub fn directed(
        author: AgentId,
        recipient: AgentId,
        content: impl Into<String>,
    ) -> Result<Self, MessageError> {
        let content = content.into();
        if content.trim().is_empty() {
            return Err(MessageError::EmptyContent);
        }
        Ok(Self {
            role: MessageRole::Assistant,
            content,
            author: Some(author),
            recipient: Some(recipient),
        })
    }

    /// Whether this message is addressed to a specific agent (or is broadcast).
    pub fn is_directed(&self) -> bool {
        self.recipient.is_some()
    }

    /// Whether this message is relevant to the given agent: either broadcast
    /// (no recipient), or explicitly addressed to them.
    pub fn is_visible_to(&self, agent: &AgentId) -> bool {
        match &self.recipient {
            None => true,
            Some(r) => r == agent,
        }
    }

    /// Convenience constructor for a system message.
    pub fn system(content: impl Into<String>) -> Result<Self, MessageError> {
        Self::new(MessageRole::System, content, None)
    }

    /// Convenience constructor for a user message.
    pub fn user(content: impl Into<String>) -> Result<Self, MessageError> {
        Self::new(MessageRole::User, content, None)
    }

    /// Convenience constructor for an agent-authored assistant message.
    pub fn assistant(author: AgentId, content: impl Into<String>) -> Result<Self, MessageError> {
        Self::new(MessageRole::Assistant, content, Some(author))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_valid_message() {
        let msg = Message::user("hello").expect("valid");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "hello");
        assert!(msg.author.is_none());
    }

    #[test]
    fn rejects_empty_content() {
        assert_eq!(Message::system("   "), Err(MessageError::EmptyContent));
    }

    #[test]
    fn assistant_carries_author() {
        let author = AgentId::new("Software Engineer").unwrap();
        let msg = Message::assistant(author.clone(), "done").unwrap();
        assert_eq!(msg.author, Some(author));
        assert_eq!(msg.role, MessageRole::Assistant);
    }

    #[test]
    fn serde_round_trips() {
        let msg = Message::user("hi").unwrap();
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn directed_message_carries_recipient() {
        let sender = AgentId::new("sender").unwrap();
        let recipient = AgentId::new("recipient").unwrap();
        let msg = Message::directed(sender.clone(), recipient.clone(), "hello").unwrap();

        assert!(msg.is_directed());
        assert_eq!(msg.author, Some(sender));
        assert_eq!(msg.recipient, Some(recipient));
        assert_eq!(msg.role, MessageRole::Assistant);
    }

    #[test]
    fn broadcast_message_visible_to_all() {
        let msg = Message::user("broadcast").unwrap();
        let any_agent = AgentId::new("any").unwrap();

        assert!(!msg.is_directed());
        assert!(msg.is_visible_to(&any_agent));
    }

    #[test]
    fn directed_message_not_visible_to_wrong_agent() {
        let sender = AgentId::new("sender").unwrap();
        let recipient = AgentId::new("recipient").unwrap();
        let wrong_agent = AgentId::new("wrong").unwrap();

        let msg = Message::directed(sender, recipient.clone(), "secret").unwrap();

        assert!(msg.is_visible_to(&recipient));
        assert!(!msg.is_visible_to(&wrong_agent));
    }

    #[test]
    fn directed_serde_round_trips() {
        let sender = AgentId::new("sender").unwrap();
        let recipient = AgentId::new("recipient").unwrap();
        let msg = Message::directed(sender, recipient, "secret").unwrap();

        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(msg, back);
        assert!(back.is_directed());
    }
}
