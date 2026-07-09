//! Built-in tool implementations (infrastructure layer).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::domain::models::{ToolCall, ToolDefinition, ToolError, ToolKind, ToolResult};
use crate::domain::ports::ToolRegistry;

/// Maximum file size to read (bytes).
const MAX_READ_SIZE: u64 = 512_000; // 500 KB

/// Registry of built-in tools scoped to a project root.
pub struct BuiltinToolRegistry {
    project_root: PathBuf,
}

impl BuiltinToolRegistry {
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    fn read_file(&self, args: &BTreeMap<String, String>) -> Result<ToolResult, ToolError> {
        let path_str = args
            .get("path")
            .ok_or_else(|| ToolError::MissingArgument("path".to_string()))?;
        let path = self.resolve_path(path_str);
        let metadata = std::fs::metadata(&path).map_err(|e| {
            ToolError::ExecutionFailed(format!("cannot stat {}: {e}", path.display()))
        })?;
        if metadata.len() > MAX_READ_SIZE {
            return Err(ToolError::ExecutionFailed(format!(
                "file too large ({} bytes, max {})",
                metadata.len(),
                MAX_READ_SIZE
            )));
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            ToolError::ExecutionFailed(format!("cannot read {}: {e}", path.display()))
        })?;
        tracing::info!(path = %path.display(), bytes = content.len(), "tool: read_file");
        Ok(ToolResult {
            tool: "read_file".to_string(),
            success: true,
            output: content,
        })
    }

    fn write_file(&self, args: &BTreeMap<String, String>) -> Result<ToolResult, ToolError> {
        let path_str = args
            .get("path")
            .ok_or_else(|| ToolError::MissingArgument("path".to_string()))?;
        let content = args
            .get("content")
            .ok_or_else(|| ToolError::MissingArgument("content".to_string()))?;
        let path = self.resolve_path(path_str);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ToolError::ExecutionFailed(format!("cannot create dirs: {e}")))?;
        }
        std::fs::write(&path, content).map_err(|e| {
            ToolError::ExecutionFailed(format!("cannot write {}: {e}", path.display()))
        })?;
        tracing::info!(path = %path.display(), bytes = content.len(), "tool: write_file");
        Ok(ToolResult {
            tool: "write_file".to_string(),
            success: true,
            output: format!("wrote {} bytes to {}", content.len(), path.display()),
        })
    }

    fn shell_exec(&self, args: &BTreeMap<String, String>) -> Result<ToolResult, ToolError> {
        let command = args
            .get("command")
            .ok_or_else(|| ToolError::MissingArgument("command".to_string()))?;
        tracing::info!(command = command.as_str(), "tool: shell_exec");
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| ToolError::ExecutionFailed(format!("cannot spawn shell: {e}")))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{stdout}\n[stderr]\n{stderr}")
        };
        // Truncate to prevent token explosion
        let truncated = if combined.len() > 4096 {
            format!("{}…(truncated)", &combined[..4096])
        } else {
            combined
        };
        Ok(ToolResult {
            tool: "shell_exec".to_string(),
            success: output.status.success(),
            output: truncated,
        })
    }

    /// Resolve a relative path against the project root.
    fn resolve_path(&self, path_str: &str) -> PathBuf {
        let p = Path::new(path_str);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.project_root.join(p)
        }
    }
}

impl ToolRegistry for BuiltinToolRegistry {
    fn available_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read the contents of a file".to_string(),
                kind: ToolKind::FileRead,
                requires_approval: false,
            },
            ToolDefinition {
                name: "write_file".to_string(),
                description: "Write content to a file (creates parent dirs)".to_string(),
                kind: ToolKind::FileWrite,
                requires_approval: true,
            },
            ToolDefinition {
                name: "shell_exec".to_string(),
                description: "Execute a shell command in the project root".to_string(),
                kind: ToolKind::ShellExec,
                requires_approval: true,
            },
        ]
    }

    fn execute(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        match call.tool.as_str() {
            "read_file" => self.read_file(&call.arguments),
            "write_file" => self.write_file(&call.arguments),
            "shell_exec" => self.shell_exec(&call.arguments),
            other => Err(ToolError::UnknownTool(other.to_string())),
        }
    }
}
