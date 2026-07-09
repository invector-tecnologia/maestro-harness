//! Short-term agent memory — a bounded sliding window of messages.

use super::Message;

/// A bounded sliding window of messages representing an agent's short-term memory.
#[derive(Debug, Clone)]
pub struct ShortTermMemory {
    messages: Vec<Message>,
    capacity: usize,
}

impl ShortTermMemory {
    /// Create memory with the given capacity (max messages retained).
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity.min(128)),
            capacity: capacity.max(1),
        }
    }

    /// Record a message. If at capacity, the oldest message is evicted.
    pub fn record(&mut self, message: Message) {
        if self.messages.len() == self.capacity {
            self.messages.remove(0);
        }
        self.messages.push(message);
    }

    /// All retained messages, oldest first.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Number of messages currently held.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether memory is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear all memory.
    pub fn clear(&mut self) {
        self.messages.clear();
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
