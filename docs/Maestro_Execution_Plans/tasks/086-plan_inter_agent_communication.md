# Plan: Inter-Agent Communication (Domain 2.10)

## Goal Description

Agents in Maestro currently operate in isolation during a cognitive cycle — they receive shared input and produce output, but cannot **address messages to specific peers**, **read from a shared workspace**, or **delegate sub-tasks hierarchically** (Maestro → PM → SWE). This plan delivers the MLP scope of inter-agent communication: a **directed messaging** primitive, a **shared scratchpad** (blackboard pattern), and the **narration events** that make these flows visible in the TUI.

### Current State
- `BroadcastBus<Message>` (the `agent_bus`) already fans out every agent's output to all agents via `history()`. This is broadcast-only — agents cannot address a specific peer.
- `AgentRuntime::run_cycle` injects the full bus history as context for each agent. Agents have no concept of "messages addressed to me".
- No shared workspace or scratchpad exists.
- The `Message` model carries an optional `author: Option<AgentId>` but no `recipient` field.

### What This Plan Delivers (MLP Scope)
1. **Directed messaging** — `Message` gains an optional `recipient: Option<AgentId>` field. Agents can send to a named peer. The runtime filters the bus history so each agent only observes messages addressed to it (or broadcast).
2. **Shared scratchpad** — A `Scratchpad` domain model (key–value + append-only log) that agents read/write via a port. The runtime injects scratchpad state into agent context.
3. **Delegation signal** — The `Signal::Delegation` and `RuntimeEvent` vocabulary are extended so Maestro (or any orchestrator persona) can emit a structured `delegate_to(persona, sub_task)` signal that the runtime routes as a directed message.
4. **Narration** — New `RuntimeEvent` variants for directed sends, scratchpad writes, and delegation make all flows observable in tracing and the TUI.

> [!IMPORTANT]
> **Not in scope for MLP:** Full hierarchical delegation chains (PM→SWE→Tester), critic/reviewer loop patterns, or MCP inter-agent protocol. These are future iterations.

## User Review Required

> [!IMPORTANT]
> The `Message` struct gains a new optional `recipient` field. This is a wire-format change — existing serialized messages (session transcripts) without the field will deserialize to `recipient: None` via `#[serde(default)]`, so it is **backward-compatible**.

## Proposed Changes

### Domain Models

#### [MODIFY] [message.rs](file:///home/bro/projects/maestro-harness/src/domain/models/message.rs)

Add an optional `recipient` field and a new constructor for directed messages.

```diff
 pub struct Message {
     pub role: MessageRole,
     pub content: String,
     #[serde(skip_serializing_if = "Option::is_none", default)]
     pub author: Option<AgentId>,
+    /// If set, this message is addressed to a specific agent.
+    #[serde(skip_serializing_if = "Option::is_none", default)]
+    pub recipient: Option<AgentId>,
 }
```

New constructor:
```rust
/// Convenience constructor for a directed agent-to-agent message.
pub fn directed(
    author: AgentId,
    recipient: AgentId,
    content: impl Into<String>,
) -> Result<Self, MessageError> {
    let content = content.into();
    if content.trim().is_empty() {
        return Err(MessageError::EmptyContent);
    }
    Ok(Self {
        role: MessageRole::Assistant,
        content,
        author: Some(author),
        recipient: Some(recipient),
    })
}

/// Whether this message is addressed to a specific agent (or is broadcast).
pub fn is_directed(&self) -> bool {
    self.recipient.is_some()
}

/// Whether this message is relevant to the given agent: either broadcast
/// (no recipient), or explicitly addressed to them.
pub fn is_visible_to(&self, agent: &AgentId) -> bool {
    match &self.recipient {
        None => true,
        Some(r) => r == agent,
    }
}
```

---

#### [NEW] [scratchpad.rs](file:///home/bro/projects/maestro-harness/src/domain/models/scratchpad.rs)

A shared key–value workspace with an append-only log of all writes.

```rust
//! Shared scratchpad (blackboard pattern) for inter-agent state sharing.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::domain::models::AgentId;

/// A single write recorded in the scratchpad log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScratchpadEntry {
    pub key: String,
    pub value: String,
    pub author: AgentId,
}

/// A shared key–value workspace visible to all agents in a cycle.
#[derive(Debug, Clone, Default)]
pub struct Scratchpad {
    state: BTreeMap<String, String>,
    log: Vec<ScratchpadEntry>,
}

impl Scratchpad {
    pub fn new() -> Self { Self::default() }

    /// Write a key–value pair, recording who wrote it.
    pub fn write(&mut self, key: impl Into<String>, value: impl Into<String>, author: AgentId) {
        let key = key.into();
        let value = value.into();
        self.state.insert(key.clone(), value.clone());
        self.log.push(ScratchpadEntry { key, value, author });
    }

    /// Read a value by key.
    pub fn read(&self, key: &str) -> Option<&str> {
        self.state.get(key).map(|s| s.as_str())
    }

    /// Snapshot the entire state.
    pub fn snapshot(&self) -> &BTreeMap<String, String> {
        &self.state
    }

    /// The full write log (append-only, oldest first).
    pub fn log(&self) -> &[ScratchpadEntry] {
        &self.log
    }

    /// Format the scratchpad as context for injection into an LLM prompt.
    pub fn as_prompt_context(&self) -> String {
        if self.state.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("[Shared Scratchpad]\n");
        for (k, v) in &self.state {
            ctx.push_str(&format!("  {k}: {v}\n"));
        }
        ctx
    }
}
```

---

#### [MODIFY] [mod.rs](file:///home/bro/projects/maestro-harness/src/domain/models/mod.rs)

Export the new `scratchpad` module and re-export key types.

```diff
 pub mod tool;
+pub mod scratchpad;

 ...
 pub use tool::{ToolCall, ToolDefinition, ToolError, ToolKind, ToolResult};
+pub use scratchpad::{Scratchpad, ScratchpadEntry};
```

---

### Domain Ports

#### [NEW] [scratchpad_port.rs](file:///home/bro/projects/maestro-harness/src/domain/ports/scratchpad_port.rs)

A port trait so the domain stays I/O-free and the scratchpad can be backed by anything.

```rust
//! Port for the shared scratchpad.

use crate::domain::models::{AgentId, Scratchpad};

/// Read/write access to the shared scratchpad.
pub trait ScratchpadPort: Send + Sync {
    fn write(&self, key: &str, value: &str, author: &AgentId);
    fn read(&self, key: &str) -> Option<String>;
    fn snapshot(&self) -> Scratchpad;
}
```

---

#### [MODIFY] [ports/mod.rs](file:///home/bro/projects/maestro-harness/src/domain/ports/mod.rs)

Export the new port.

---

### Application Layer

#### [MODIFY] [agent_observability.rs](file:///home/bro/projects/maestro-harness/src/application/agent_observability.rs)

Add new `RuntimeEvent` variants for inter-agent messaging and scratchpad.

```diff
+    /// An agent sent a directed message to a specific peer.
+    AgentDirectedSend {
+        sender: AgentId,
+        recipient: AgentId,
+    },
+    /// An agent wrote to the shared scratchpad.
+    ScratchpadWrite {
+        agent: AgentId,
+        key: String,
+    },
```

These variants get `narrate()` implementations and are covered by the `agent()` accessor.

---

#### [MODIFY] [agent_runtime.rs](file:///home/bro/projects/maestro-harness/src/application/agent_runtime.rs)

The `run_cycle` method is updated to:

1. **Filter bus history** — instead of injecting the entire bus history, filter it through `msg.is_visible_to(agent_id)` so agents only see broadcast + messages addressed to them.
2. **Accept a shared `Scratchpad`** — inject scratchpad state as a system-context message into each agent's observation. After the cycle, any `[SCRATCHPAD_WRITE]` blocks in agent output are parsed and applied.
3. **Detect directed messages** — after an agent acts, if the output `Message` has a `recipient`, emit `RuntimeEvent::AgentDirectedSend`.

```diff
-    pub async fn run_cycle(&self, agents: Vec<Box<dyn Role>>, input: Vec<Message>) -> Vec<Message> {
+    pub async fn run_cycle(
+        &self,
+        agents: Vec<Box<dyn Role>>,
+        input: Vec<Message>,
+        scratchpad: Option<Arc<RwLock<Scratchpad>>>,
+    ) -> Vec<Message> {
```

Inside the per-agent task:
```rust
// Filter history to messages visible to this agent
let history = agent_bus.history().await;
let visible: Vec<Message> = history
    .into_iter()
    .filter(|m| m.is_visible_to(&id))
    .collect();
let mut enriched_input = visible;

// Inject scratchpad context
if let Some(ref pad) = scratchpad {
    let ctx = pad.read().await.as_prompt_context();
    if !ctx.is_empty() {
        if let Ok(ctx_msg) = Message::system(ctx) {
            enriched_input.push(ctx_msg);
        }
    }
}

enriched_input.extend(input);
```

After act, detect directed sends:
```rust
if let Some(ref msg) = output {
    if msg.is_directed() {
        if let (Some(sender), Some(recipient)) = (&msg.author, &msg.recipient) {
            emit(&events, RuntimeEvent::AgentDirectedSend {
                sender: sender.clone(),
                recipient: recipient.clone(),
            }).await;
        }
    }
    // ... existing publish + reflect logic
}
```

---

#### [MODIFY] [persona_agent.rs](file:///home/bro/projects/maestro-harness/src/application/persona_agent.rs)

Add a `parse_directed_message` helper in the `act()` method. If the LLM output contains a `[SEND_TO agent_id]...[/SEND_TO]` block, parse it into a `Message::directed(self.id, recipient, content)`.

```rust
/// Check if the LLM output contains a directed message marker.
fn parse_directed_send(output: &str, sender: &AgentId) -> Option<Message> {
    let start = output.find("[SEND_TO ")?;
    let end_marker = output.find("[/SEND_TO]")?;
    let header_end = output[start..].find(']')? + start;
    let recipient_str = output[start + 9..header_end].trim();
    let content = output[header_end + 1..end_marker].trim();
    let recipient = AgentId::new(recipient_str).ok()?;
    Message::directed(sender.clone(), recipient, content).ok()
}
```

The system prompt is also extended to inform agents about directed messaging:
```
## Directed Messaging
To send a message to a specific agent, use:
[SEND_TO agent_name]
Your message content here.
[/SEND_TO]
```

---

### FEATURE_MAP Update

#### [MODIFY] [FEATURE_MAP.md](file:///home/bro/projects/maestro-harness/docs/Product_Engineering/FEATURE_MAP.md)

Update item 2.10 status from 📋 Planned → ✅ Implemented (MLP scope):

```diff
-  **Status:** 📋 Planned
-  **Source:** Not implemented
+  **Status:** ✅ Implemented (MLP scope)
+  **Source:** `src/domain/models/message.rs`, `src/domain/models/scratchpad.rs`,
+    `src/application/agent_runtime.rs`, `src/application/persona_agent.rs`
   **What It Does Today:** Directed agent-to-agent messaging via recipient field.
     Shared scratchpad (blackboard pattern) for cross-agent state.
     Filtered bus history (agents only see broadcast + addressed messages).
     Runtime narration events for directed sends and scratchpad writes.
   **Gap:** Missing hierarchical delegation chains, critic/reviewer loop patterns.
```

---

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
./scripts/quality-gate.sh
```

#### New tests to write:

| Module | Test | Asserts |
|--------|------|---------|
| `message.rs` | `directed_message_carries_recipient` | `msg.recipient` is `Some`, `is_directed()` is true |
| `message.rs` | `broadcast_message_visible_to_all` | `is_visible_to()` returns true for any agent |
| `message.rs` | `directed_message_not_visible_to_wrong_agent` | `is_visible_to(wrong)` returns false |
| `message.rs` | `directed_serde_round_trips` | Serialize/deserialize preserves `recipient` |
| `scratchpad.rs` | `write_and_read` | Written key is readable |
| `scratchpad.rs` | `overwrite_replaces_value` | Second write to same key wins |
| `scratchpad.rs` | `log_records_all_writes` | Log length matches write count |
| `scratchpad.rs` | `as_prompt_context_is_empty_when_no_entries` | Empty scratchpad returns empty string |
| `agent_runtime.rs` | `directed_message_only_visible_to_recipient` | After one agent sends directed, the non-recipient does not observe it |
| `tool_dispatch.rs` | (existing) | All existing tests still pass |

### Manual Verification

1. Run `cargo test` and confirm all 170+ existing tests plus the new ones pass.
2. Run `./scripts/quality-gate.sh` end-to-end (Rust + Nim).

## Model Recommendation

> [!NOTE]
> **Suggested model:** Gemini 3.1 Pro (Low) — this is incremental feature work following established patterns, no complex architectural reasoning needed.
