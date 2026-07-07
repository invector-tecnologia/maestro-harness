//! Infrastructure layer — port implementations and external adapters.
//!
//! Depends on `domain` only (via trait ports), never on `application` or
//! `presentation`. Houses provider adapters, the event bus, and the safety harness.

pub mod bus;
pub mod config;
pub mod harness;
pub mod llm;
