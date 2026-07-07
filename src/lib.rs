//! Maestro — local-first tactical Agentic Workflow Orchestrator (Rust core).
//!
//! This crate is the headless brain. The interactive TUI is a separate Nim/Tatui
//! process under `frontend/` that talks to this core over a line-delimited JSON
//! protocol on stdio (see [`presentation::ipc`]). The core is fully usable with
//! `--no-tui`.
//!
//! Layering (hexagonal / ports & adapters), dependencies point inward only:
//! `presentation -> application -> domain` and `infrastructure -> domain`.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
