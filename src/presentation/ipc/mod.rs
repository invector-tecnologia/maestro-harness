//! IPC protocol — line-delimited JSON on stdio (TASK 051).
//!
//! The only Rust↔Nim coupling. Every frame carries a protocol version; frames are
//! newline-delimited JSON. Mirrored by `frontend/src/protocol.nim`. Any change to
//! these types requires an ADR in `docs/adr/`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod server;

/// Protocol version. Bumped only alongside an ADR describing the contract change.
/// v2 adds mode switching and mode-scoped payloads (ADR 0003).
pub const PROTOCOL_VERSION: u32 = 2;

/// The active Workspace mode (ADR 0002). One of Config, Maestro, Product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Governance CRUD + archive (config.yml + personas/skills/scopes).
    Config,
    /// Orchestration: user demand -> Maestro plans/delegates -> personas build.
    Maestro,
    /// Live demo of a shipped release + its notes/changelog.
    Product,
}

/// A single node in the Config Mode governance tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigNode {
    /// Stable identifier (path-like, e.g. `personas/project_manager`).
    pub id: String,
    /// Entry kind: `config` | `persona` | `skill` | `scope`.
    pub kind: String,
    /// Origin: `default` | `custom`.
    pub origin: String,
    /// Whether the entry has been archived (soft delete).
    pub archived: bool,
    /// Human-facing label.
    pub label: String,
}

/// A shipped release surfaced in Product Mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseSummary {
    /// Semantic-ish version tag of the release.
    pub version: String,
    /// Release notes / changelog body.
    pub changelog: String,
    /// RFC3339 creation timestamp.
    pub created_at: String,
}

/// Events streamed from the core to the TUI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CoreEvent {
    /// An agent changed cognitive state (idle/observe/think/act/error).
    AgentState { agent: String, state: String },
    /// The micro-project FSM advanced.
    FsmTransition { from: String, to: String },
    /// A log line for the event panel.
    Log { level: String, message: String },
    /// A named metric sample for the metrics panel.
    Metric { name: String, value: f64 },
    /// Liveness signal while long work runs.
    Heartbeat { seq: u64 },
    /// The core is blocked awaiting a user approval.
    ApprovalRequest { id: String, prompt: String },
    /// The core acknowledges the active Workspace mode.
    ModeChanged { mode: Mode },
    /// Config Mode: the full governance tree.
    ConfigTree { entries: Vec<ConfigNode> },
    /// Config Mode: a single entry's body.
    ConfigEntry {
        id: String,
        entry_kind: String,
        origin: String,
        archived: bool,
        body: String,
    },
    /// Config Mode: result of validating an entry or the config.
    ConfigValidation { ok: bool, issues: Vec<String> },
    /// Config Mode: an entry was persisted.
    ConfigSaved { id: String },
    /// Maestro Mode: the plan Maestro proposes for a demand.
    PlanProposed { steps: Vec<String> },
    /// Maestro Mode: Maestro delegates a task to a persona.
    Delegation { persona: String, task: String },
    /// Maestro Mode: a persona delivered an artifact/summary.
    Deliverable {
        persona: String,
        summary: String,
        artifact: Option<String>,
    },
    /// Product Mode: the list of shipped releases.
    ReleaseList { releases: Vec<ReleaseSummary> },
    /// Product Mode: a chunk of live demo output (`stream` = stdout|stderr).
    DemoOutput { stream: String, chunk: String },
    /// Product Mode: the demo process exited with `code`.
    DemoExited { code: i32 },
}

/// Commands sent from the TUI to the core.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TuiCommand {
    /// Free-form user input from the command box.
    UserInput { text: String },
    /// A slash command (e.g. `/status`).
    Command { name: String },
    /// A response to a pending [`CoreEvent::ApprovalRequest`].
    ApprovalResponse { id: String, approved: bool },
    /// Switch the active Workspace mode.
    SwitchMode { mode: Mode },
    /// Config Mode: request the governance tree.
    ConfigList,
    /// Config Mode: open a single entry by id.
    ConfigOpen { id: String },
    /// Config Mode: edit an entry's body (unsaved).
    ConfigEdit { id: String, body: String },
    /// Config Mode: create a new custom entry.
    ConfigCreate {
        entry_kind: String,
        id: String,
        body: String,
    },
    /// Config Mode: archive (soft-delete) an entry.
    ConfigArchive { id: String },
    /// Config Mode: validate an entry body before saving.
    ConfigValidate { id: String, body: String },
    /// Config Mode: persist an entry body.
    ConfigSave { id: String, body: String },
    /// Product Mode: request the release list.
    ListReleases,
    /// Product Mode: run the live demo for a release.
    RunDemo { release: String },
    /// Product Mode: stop the running demo.
    StopDemo,
}

/// Errors decoding a frame.
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// The line was not valid JSON for the expected message.
    #[error("malformed frame: {0}")]
    Malformed(String),
    /// The frame's protocol version is not supported.
    #[error("unsupported protocol version {0} (expected {PROTOCOL_VERSION})")]
    UnsupportedVersion(u32),
}

#[derive(Serialize, Deserialize)]
struct Versioned<T> {
    v: u32,
    #[serde(flatten)]
    message: T,
}

/// Encode a message as a single newline-delimited JSON frame with the version.
pub fn encode<T: Serialize>(message: &T) -> Result<String, ProtocolError> {
    let framed = Versioned {
        v: PROTOCOL_VERSION,
        message,
    };
    let mut line =
        serde_json::to_string(&framed).map_err(|e| ProtocolError::Malformed(e.to_string()))?;
    line.push('\n');
    Ok(line)
}

/// Decode a single newline-delimited JSON frame, rejecting malformed input and
/// unsupported versions without panicking.
pub fn decode<T: DeserializeOwned>(line: &str) -> Result<T, ProtocolError> {
    let value: serde_json::Value =
        serde_json::from_str(line.trim()).map_err(|e| ProtocolError::Malformed(e.to_string()))?;
    let version = value
        .get("v")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ProtocolError::Malformed("missing 'v' version field".into()))?;
    if version != PROTOCOL_VERSION as u64 {
        return Err(ProtocolError::UnsupportedVersion(version as u32));
    }
    let framed: Versioned<T> =
        serde_json::from_value(value).map_err(|e| ProtocolError::Malformed(e.to_string()))?;
    Ok(framed.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_is_newline_delimited() {
        let line = encode(&CoreEvent::Heartbeat { seq: 1 }).unwrap();
        assert!(line.ends_with('\n'));
        assert!(line.contains("\"v\":2"));
    }

    #[test]
    fn core_events_round_trip() {
        let events = vec![
            CoreEvent::AgentState {
                agent: "Maestro".into(),
                state: "think".into(),
            },
            CoreEvent::FsmTransition {
                from: "Planning".into(),
                to: "Approval".into(),
            },
            CoreEvent::Log {
                level: "info".into(),
                message: "hi".into(),
            },
            CoreEvent::Metric {
                name: "tokens".into(),
                value: 12.5,
            },
            CoreEvent::Heartbeat { seq: 9 },
            CoreEvent::ApprovalRequest {
                id: "a1".into(),
                prompt: "proceed?".into(),
            },
        ];
        for event in events {
            let decoded: CoreEvent = decode(&encode(&event).unwrap()).unwrap();
            assert_eq!(decoded, event);
        }
    }

    #[test]
    fn tui_commands_round_trip() {
        let commands = vec![
            TuiCommand::UserInput { text: "go".into() },
            TuiCommand::Command {
                name: "status".into(),
            },
            TuiCommand::ApprovalResponse {
                id: "a1".into(),
                approved: true,
            },
        ];
        for command in commands {
            let decoded: TuiCommand = decode(&encode(&command).unwrap()).unwrap();
            assert_eq!(decoded, command);
        }
    }

    #[test]
    fn malformed_frame_is_rejected() {
        let result: Result<CoreEvent, _> = decode("not json");
        assert!(matches!(result, Err(ProtocolError::Malformed(_))));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let result: Result<CoreEvent, _> = decode("{\"v\":99,\"kind\":\"heartbeat\",\"seq\":1}");
        assert!(matches!(result, Err(ProtocolError::UnsupportedVersion(99))));
    }

    #[test]
    fn unknown_kind_is_rejected() {
        let result: Result<CoreEvent, _> = decode("{\"v\":2,\"kind\":\"bogus\"}");
        assert!(matches!(result, Err(ProtocolError::Malformed(_))));
    }

    #[test]
    fn v2_mode_messages_round_trip() {
        let events = vec![
            CoreEvent::ModeChanged { mode: Mode::Config },
            CoreEvent::ConfigTree {
                entries: vec![ConfigNode {
                    id: "personas/maestro".into(),
                    kind: "persona".into(),
                    origin: "default".into(),
                    archived: false,
                    label: "Maestro".into(),
                }],
            },
            CoreEvent::ConfigEntry {
                id: "personas/maestro".into(),
                entry_kind: "persona".into(),
                origin: "default".into(),
                archived: false,
                body: "# Maestro".into(),
            },
            CoreEvent::ConfigValidation {
                ok: false,
                issues: vec!["missing scope".into()],
            },
            CoreEvent::ConfigSaved {
                id: "personas/x".into(),
            },
            CoreEvent::PlanProposed {
                steps: vec!["scaffold".into(), "build".into()],
            },
            CoreEvent::Delegation {
                persona: "Software Engineer".into(),
                task: "write module".into(),
            },
            CoreEvent::Deliverable {
                persona: "Software Engineer".into(),
                summary: "done".into(),
                artifact: Some("out.rs".into()),
            },
            CoreEvent::ReleaseList {
                releases: vec![ReleaseSummary {
                    version: "0.1.0".into(),
                    changelog: "initial".into(),
                    created_at: "2026-07-07T00:00:00Z".into(),
                }],
            },
            CoreEvent::DemoOutput {
                stream: "stdout".into(),
                chunk: "hello".into(),
            },
            CoreEvent::DemoExited { code: 0 },
        ];
        for event in events {
            let decoded: CoreEvent = decode(&encode(&event).unwrap()).unwrap();
            assert_eq!(decoded, event);
        }
    }

    #[test]
    fn v2_mode_commands_round_trip() {
        let commands = vec![
            TuiCommand::SwitchMode {
                mode: Mode::Product,
            },
            TuiCommand::ConfigList,
            TuiCommand::ConfigOpen {
                id: "scopes/primary".into(),
            },
            TuiCommand::ConfigEdit {
                id: "scopes/primary".into(),
                body: "# scope".into(),
            },
            TuiCommand::ConfigCreate {
                entry_kind: "persona".into(),
                id: "personas/custom".into(),
                body: "# custom".into(),
            },
            TuiCommand::ConfigArchive {
                id: "personas/custom".into(),
            },
            TuiCommand::ConfigValidate {
                id: "scopes/primary".into(),
                body: "# scope".into(),
            },
            TuiCommand::ConfigSave {
                id: "scopes/primary".into(),
                body: "# scope".into(),
            },
            TuiCommand::ListReleases,
            TuiCommand::RunDemo {
                release: "0.1.0".into(),
            },
            TuiCommand::StopDemo,
        ];
        for command in commands {
            let decoded: TuiCommand = decode(&encode(&command).unwrap()).unwrap();
            assert_eq!(decoded, command);
        }
    }
}
