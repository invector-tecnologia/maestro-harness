//! Short-term agent memory — a bounded sliding window of messages.

use super::Message;

/// A bounded sliding window of messages with summarization of evicted content.
#[derive(Debug, Clone)]
pub struct ShortTermMemory {
    messages: Vec<Message>,
    capacity: usize,
    /// Accumulated summary of messages evicted from the sliding window.
    summary: Option<String>,
}

impl ShortTermMemory {
    /// Create a new bounded memory with the given capacity (min 1, max 128).
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity.min(128)),
            capacity: capacity.max(1),
            summary: None,
        }
    }

    /// Record a message. If at capacity, evict the oldest and summarize it.
    pub fn record(&mut self, message: Message) {
        if self.messages.len() == self.capacity {
            let evicted = self.messages.remove(0);
            self.summarize_evicted(&evicted);
        }
        self.messages.push(message);
    }

    /// Read the current messages in the window.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Accumulated summary of evicted messages, if any.
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    /// Seed memory from a prior session transcript.
    pub fn hydrate(&mut self, prior: &[Message]) {
        for msg in prior {
            self.record(msg.clone());
        }
    }

    /// Export all current messages for persistence.
    pub fn export(&self) -> Vec<Message> {
        self.messages.clone()
    }

    /// Number of messages currently held.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether the memory is currently empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear all memory and summary.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.summary = None;
    }

    fn summarize_evicted(&mut self, evicted: &Message) {
        let prefix = match evicted.author {
            Some(ref id) => format!("[{}] ", id),
            None => String::new(),
        };
        // Truncate long messages to keep summary bounded
        let content = if evicted.content.len() > 120 {
            format!("{}{}…", prefix, &evicted.content[..120])
        } else {
            format!("{}{}", prefix, evicted.content)
        };
        match self.summary.as_mut() {
            Some(s) => {
                s.push_str(" | ");
                s.push_str(&content);
                // Cap total summary length
                if s.len() > 2048 {
                    let truncated = s[s.len() - 1800..].to_string();
                    *s = format!("…{}", truncated);
                }
            }
            None => {
                self.summary = Some(content);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::message::MessageRole;

    fn msg(content: &str) -> Message {
        Message::new(MessageRole::User, content, None).unwrap()
    }

    #[test]
    fn records_up_to_capacity() {
        let mut mem = ShortTermMemory::new(2);
        mem.record(msg("1"));
        mem.record(msg("2"));
        mem.record(msg("3"));
        assert_eq!(mem.len(), 2);
        assert_eq!(mem.messages()[0].content, "2");
        assert_eq!(mem.messages()[1].content, "3");
    }

    #[test]
    fn clear_empties_all() {
        let mut mem = ShortTermMemory::new(2);
        mem.record(msg("1"));
        mem.clear();
        assert!(mem.is_empty());
    }
}
