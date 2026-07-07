//! Event bus adapter (`tokio::sync::broadcast` with bounded history).
//!
//! TASK 002. Generic 1:N fan-out used for messages and runtime narration.

pub mod broadcast_bus;

pub use broadcast_bus::BroadcastBus;
