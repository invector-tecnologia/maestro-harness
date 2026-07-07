//! Presentation layer — the process boundary.
//!
//! Two surfaces: the `cli` (clap parsing, startup wiring) and the `ipc` protocol
//! (line-delimited JSON on stdio) that couples the core to the Nim/Niobium TUI.
//! Depends on `application` and `domain`.

pub mod cli;
pub mod ipc;
