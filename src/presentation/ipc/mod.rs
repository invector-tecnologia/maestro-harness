//! IPC protocol — line-delimited JSON on stdio (TASK 051).
//!
//! The only Rust↔Nim coupling. Every frame carries a protocol version; frames are
//! newline-delimited JSON. Mirrored by `frontend/src/protocol.nim`. Any change to
//! these types requires an ADR in `docs/adr/`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol version. Bumped only alongside an ADR describing the contract change.
pub const PROTOCOL_VERSION: u32 = 1;

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
        assert!(line.contains("\"v\":1"));
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
        let result: Result<CoreEvent, _> = decode("{\"v\":1,\"kind\":\"bogus\"}");
        assert!(matches!(result, Err(ProtocolError::Malformed(_))));
    }
}
