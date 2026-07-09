//! Tool-call detection and dispatch.
//!
//! Parses `[TOOL_CALL]...[/TOOL_CALL]` blocks from LLM output,
//! dispatches them through a `ToolRegistry`, and formats results
//! for re-injection into the conversation.

use crate::domain::models::{ToolCall, ToolResult};
use crate::domain::ports::ToolRegistry;

/// Attempt to parse a tool call from LLM output.
/// Returns the parsed call and the surrounding text if found.
pub fn parse_tool_call(output: &str) -> Option<(ToolCall, String)> {
    let start_marker = "[TOOL_CALL]";
    let end_marker = "[/TOOL_CALL]";
    let start = output.find(start_marker)?;
    let end = output.find(end_marker)?;
    if end <= start {
        return None;
    }
    let json_str = &output[start + start_marker.len()..end].trim();
    let call: ToolCall = serde_json::from_str(json_str).ok()?;
    // Text before and after the tool call block
    let surrounding = format!(
        "{}{}",
        output[..start].trim(),
        output[end + end_marker.len()..].trim()
    );
    Some((call, surrounding))
}

/// Execute a tool call through the registry and format the result.
pub fn dispatch(registry: &dyn ToolRegistry, call: &ToolCall) -> ToolResult {
    match registry.execute(call) {
        Ok(result) => result,
        Err(e) => ToolResult {
            tool: call.tool.clone(),
            success: false,
            output: e.to_string(),
        },
    }
}

/// Format a tool result for injection into the LLM conversation.
pub fn format_result(result: &ToolResult) -> String {
    if result.success {
        format!(
            "[TOOL_RESULT tool=\"{}\" success=true]\n{}\n[/TOOL_RESULT]",
            result.tool, result.output
        )
    } else {
        format!(
            "[TOOL_RESULT tool=\"{}\" success=false]\n{}\n[/TOOL_RESULT]",
            result.tool, result.output
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_tool_call() {
        let output = r#"I need to read a file.
[TOOL_CALL]
{"tool": "read_file", "arguments": {"path": "src/main.rs"}}
[/TOOL_CALL]
Then I'll analyze it."#;
        let (call, surrounding) = parse_tool_call(output).unwrap();
        assert_eq!(call.tool, "read_file");
        assert_eq!(call.arguments.get("path").unwrap(), "src/main.rs");
        assert!(surrounding.contains("I need to read a file."));
        assert!(surrounding.contains("Then I'll analyze it."));
    }

    #[test]
    fn returns_none_for_no_marker() {
        assert!(parse_tool_call("just plain text").is_none());
    }

    #[test]
    fn returns_none_for_invalid_json() {
        let output = "[TOOL_CALL]\nnot json\n[/TOOL_CALL]";
        assert!(parse_tool_call(output).is_none());
    }

    #[test]
    fn formats_success_result() {
        let result = ToolResult {
            tool: "read_file".to_string(),
            success: true,
            output: "fn main() {}".to_string(),
        };
        let formatted = format_result(&result);
        assert!(formatted.contains("success=true"));
        assert!(formatted.contains("fn main() {}"));
    }

    #[test]
    fn formats_failure_result() {
        let result = ToolResult {
            tool: "read_file".to_string(),
            success: false,
            output: "file not found".to_string(),
        };
        let formatted = format_result(&result);
        assert!(formatted.contains("success=false"));
    }
}
