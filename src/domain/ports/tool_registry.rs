//! `ToolRegistry` — port for dispatching tool calls.

use crate::domain::models::tool::{ToolCall, ToolDefinition, ToolError, ToolResult};

/// Port for registering and executing tools.
#[cfg_attr(test, mockall::automock)]
pub trait ToolRegistry: Send + Sync {
    /// List all available tool definitions.
    fn available_tools(&self) -> Vec<ToolDefinition>;

    /// Execute a tool call and return the result.
    fn execute(&self, call: &ToolCall) -> Result<ToolResult, ToolError>;
}
