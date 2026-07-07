//! Domain layer — the sacred core.
//!
//! Pure business types and port traits. This module MUST NOT import from
//! `crate::infrastructure` or `crate::presentation`, and MUST NOT perform I/O
//! or depend on provider SDKs. Enforced by `tests/domain_boundary.rs`.

pub mod models;
pub mod ports;
