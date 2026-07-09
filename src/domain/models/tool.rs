//! Tool-use domain types.
//!
//! Defines the vocabulary for agent tool interactions: what tools exist,
//! how they are invoked, and what they return.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The kind of tool — determines dispatch and governance policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    /// Read a file from the project.
    FileRead,
    /// Write or create a file in the project.
    FileWrite,
    /// Execute a shell command.
    ShellExec,
}

impl ToolKind {
    /// Whether this tool kind requires explicit user approval before execution.
    pub fn requires_approval(self) -> bool {
        matches!(self, Self::FileWrite | Self::ShellExec)
    }

    /// Human-readable label for display.
    pub fn label(self) -> &'static str {
        match self {
            Self::FileRead => "read_file",
            Self::FileWrite => "write_file",
            Self::ShellExec => "shell_exec",
        }
    }
}

/// A tool definition in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (matches ToolKind label).
    pub name: String,
    /// Human-readable description shown to agents.
    pub description: String,
    /// The tool kind for dispatch.
    pub kind: ToolKind,
    /// Whether execution requires user approval.
    pub requires_approval: bool,
}

/// A tool invocation request parsed from LLM output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Which tool to invoke.
    pub tool: String,
    /// Arguments as a JSON-compatible map.
    pub arguments: std::collections::BTreeMap<String, String>,
}

/// The outcome of executing a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    /// The tool that was called.
    pub tool: String,
    /// Whether the call succeeded.
    pub success: bool,
    /// Output content (file contents, command stdout, error message).
    pub output: String,
}

/// Errors from tool dispatch.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("missing argument: {0}")]
    MissingArgument(String),
    #[error("tool execution failed: {0}")]
    ExecutionFailed(String),
    #[error("tool requires approval")]
    RequiresApproval,
    #[error("tool timed out after {0}s")]
    Timeout(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_read_does_not_require_approval() {
        assert!(!ToolKind::FileRead.requires_approval());
    }

    #[test]
    fn file_write_requires_approval() {
        assert!(ToolKind::FileWrite.requires_approval());
    }

    #[test]
    fn shell_exec_requires_approval() {
        assert!(ToolKind::ShellExec.requires_approval());
    }

    #[test]
    fn tool_call_serde_round_trip() {
        let call = ToolCall {
            tool: "read_file".to_string(),
            arguments: vec![("path".to_string(), "src/main.rs".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::to_string(&call).unwrap();
        let back: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(call, back);
    }

    #[test]
    fn tool_result_serde_round_trip() {
        let result = ToolResult {
            tool: "read_file".to_string(),
            success: true,
            output: "fn main() {}".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ToolResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
