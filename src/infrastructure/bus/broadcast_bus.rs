//! Asynchronous broadcast bus (TASK 002).
//!
//! A `tokio::sync::broadcast` fan-out with a bounded replay history. Generic over
//! any `Clone + Send` payload; the runtime uses it for [`crate::domain::models::Message`]
//! and for `RuntimeEvent` narration.

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};

/// A 1:N broadcast bus with bounded history for late subscribers.
#[derive(Clone)]
pub struct BroadcastBus<T: Clone + Send + 'static> {
    sender: broadcast::Sender<T>,
    history: Arc<Mutex<VecDeque<T>>>,
    history_cap: usize,
}

impl<T: Clone + Send + 'static> BroadcastBus<T> {
    /// Create a bus with `channel_capacity` in-flight slots per receiver and a
    /// replayable `history_cap` most-recent events.
    pub fn new(channel_capacity: usize, history_cap: usize) -> Self {
        let (sender, _rx) = broadcast::channel(channel_capacity.max(1));
        Self {
            sender,
            history: Arc::new(Mutex::new(VecDeque::with_capacity(history_cap))),
            history_cap: history_cap.max(1),
        }
    }

    /// Subscribe for future events. A lagging receiver observes
    /// `RecvError::Lagged` rather than crashing the bus.
    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    /// Publish an event to all subscribers and record it in history. Returns the
    /// number of receivers that got it (0 is not an error).
    pub async fn publish(&self, event: T) -> usize {
        {
            let mut history = self.history.lock().await;
            if history.len() == self.history_cap {
                history.pop_front();
            }
            history.push_back(event.clone());
        }
        // A send error only means there are no live receivers; that is fine.
        self.sender.send(event).unwrap_or(0)
    }

    /// Snapshot the retained history, oldest first.
    pub async fn history(&self) -> Vec<T> {
        self.history.lock().await.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fans_out_to_multiple_subscribers() {
        let bus: BroadcastBus<u32> = BroadcastBus::new(8, 4);
        let mut a = bus.subscribe();
        let mut b = bus.subscribe();

        let delivered = bus.publish(42).await;
        assert_eq!(delivered, 2);
        assert_eq!(a.recv().await.unwrap(), 42);
        assert_eq!(b.recv().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn retains_bounded_history() {
        let bus: BroadcastBus<u32> = BroadcastBus::new(8, 2);
        bus.publish(1).await;
        bus.publish(2).await;
        bus.publish(3).await;
        assert_eq!(bus.history().await, vec![2, 3]);
    }

    #[tokio::test]
    async fn publish_without_subscribers_is_ok() {
        let bus: BroadcastBus<u32> = BroadcastBus::new(8, 4);
        assert_eq!(bus.publish(7).await, 0);
        assert_eq!(bus.history().await, vec![7]);
    }
}
