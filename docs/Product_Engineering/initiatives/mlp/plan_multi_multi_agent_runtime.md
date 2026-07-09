# Plan: 2.4 — Multi-Agent Runtime Enrichment

## Goal

Close the three most critical gaps in the multi-agent runtime: agents are **stateless** (no memory
between cognitive cycles), **can't communicate** with each other, and have **no lifecycle
management**. This plan adds short-term memory, an inter-agent message bus, and agent
lifecycle tracking — all within the existing hexagonal architecture.

## Background

The current `AgentRuntime` runs the `OBSERVE → THINK → ACT → REFLECT` cycle for all agents
concurrently via a Tokio `JoinSet`, narrating each phase through `BroadcastBus<RuntimeEvent>`.
However:

- **Stateless**: `PersonaAgent.inbox` is consumed on `act()` and cleared. There is no accumulated
  context between cycles. Each cycle starts from scratch.
- **No inter-agent communication**: Agents cannot see each other's outputs. The `BroadcastBus`
  carries `RuntimeEvent` (observability only), not agent `Message`s.
- **No lifecycle**: Agents are created as ephemeral `Vec<Box<dyn Role>>`, passed into `run_cycle()`,
  and discarded. No spawn/terminate/status tracking.

### What This Plan Does NOT Do (Future Work)

- Long-term vector-store memory (requires embedding infrastructure — see item 4.9)
- Token budgets / backpressure (requires token counting — see item 4.7)
- Streaming intermediate results (requires streaming infra — see item 4.6)
- Cross-session persistence (requires persistence layer — see item 3.9)

## FEATURE_MAP Entry (Before)

```
- **What It Does Today:** Concurrent JoinSet runs all agents' cognitive cycles in parallel
  (read-only). Failing agents are isolated. BroadcastBus<RuntimeEvent> for event fan-out.
- **Gap:** Agents are stateless, memoryless, and can't communicate with each other.
```

## FEATURE_MAP Entry (After — proposed revalidation)

```
- **What It Does Today:** Concurrent JoinSet runs all agents' cognitive cycles in parallel
  (read-only). Failing agents are isolated. BroadcastBus<RuntimeEvent> for event fan-out.
  Short-term memory accumulates context across cycles with configurable capacity. Inter-agent
  message bus lets agents observe each other's outputs. Agent lifecycle tracking with
  spawn/terminate/status.
- **Gap:** No long-term vector-store memory (requires embeddings). No token budgets or
  backpressure. No streaming intermediate results. No cross-session persistence.
```

---

## Proposed Changes

### 1. Domain — `src/domain/models/memory.rs` [NEW]

Add a short-term memory model to the domain layer. This is a pure data structure with no I/O
— domain-safe.

```rust
//! Short-term agent memory — a bounded sliding window of messages.

use super::Message;

/// A bounded sliding window of messages representing an agent's short-term memory.
#[derive(Debug, Clone)]
pub struct ShortTermMemory {
    messages: Vec<Message>,
    capacity: usize,
}

impl ShortTermMemory {
    /// Create memory with the given capacity (max messages retained).
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity.min(128)),
            capacity: capacity.max(1),
        }
    }

    /// Record a message. If at capacity, the oldest message is evicted.
    pub fn record(&mut self, message: Message) {
        if self.messages.len() == self.capacity {
            self.messages.remove(0);
        }
        self.messages.push(message);
    }

    /// All retained messages, oldest first.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Number of messages currently held.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether memory is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear all memory.
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
```

**Tests:**
- `records_up_to_capacity` — fill to capacity, assert oldest evicted
- `clear_empties_all` — clear and assert empty

---

### 2. Domain — `src/domain/models/mod.rs` [MODIFY]

Add `mod memory;` and re-export `ShortTermMemory`.

---

### 3. Application — `src/application/agent_runtime.rs` [MODIFY]

Enrich `AgentRuntime` with three capabilities:

#### 3a. Agent Registry (Lifecycle)

Add an `AgentRegistry` that tracks registered agents with their status:

```rust
/// Lifecycle status of a registered agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Terminated,
}

/// Tracks registered agents and their lifecycle status.
struct AgentEntry {
    status: AgentStatus,
}
```

The runtime will hold `agents: Arc<tokio::sync::RwLock<HashMap<AgentId, AgentEntry>>>` to track
agent lifecycles across cycles.

**New methods:**
- `register(&self, id: AgentId)` — register an agent as `Idle`
- `terminate(&self, id: &AgentId)` — mark as `Terminated`
- `status(&self, id: &AgentId) -> Option<AgentStatus>` — query status
- `active_agents(&self) -> Vec<AgentId>` — list non-terminated agents

#### 3b. Inter-Agent Message Bus

Add a second `BroadcastBus<Message>` specifically for agent-to-agent messages:

```rust
pub struct AgentRuntime {
    events: BroadcastBus<RuntimeEvent>,
    agent_bus: BroadcastBus<Message>,       // NEW: inter-agent messages
    registry: Arc<RwLock<HashMap<AgentId, AgentEntry>>>,
}
```

After each agent's `act()` succeeds, the output is published to `agent_bus` so other agents
can observe it in future cycles. New `RuntimeEvent` variants narrate messaging:

```rust
/// An agent published a message to the agent bus.
AgentPublished { agent: AgentId },
```

#### 3c. Cycle Memory Injection

Before calling `agent.observe()`, inject the recent `agent_bus` history so agents see each
other's prior outputs as additional context:

```diff
 for mut agent in agents {
     let events = self.events.clone();
+    let agent_bus = self.agent_bus.clone();
     let input = input.clone();
     set.spawn(async move {
         let id = agent.id().clone();

+        // Inject inter-agent messages from prior cycles as additional context
+        let history = agent_bus.history().await;
+        let mut enriched_input = history;
+        enriched_input.extend(input);

         emit(&events, RuntimeEvent::AgentObserving { agent: id.clone() }).await;
-        agent.observe(&input);
+        agent.observe(&enriched_input);
```

---

### 4. Application — `src/application/persona_agent.rs` [MODIFY]

Add `ShortTermMemory` to `PersonaAgent` so it retains context across cycles:

```diff
 pub struct PersonaAgent {
     persona: Persona,
     provider: Arc<dyn LlmProvider>,
     model: String,
     inbox: Vec<Message>,
     last_thinking: Option<ThinkingOutput>,
+    memory: ShortTermMemory,
 }
```

In `observe()`, record incoming messages into memory:

```diff
 fn observe(&mut self, input: &[Message]) {
     self.inbox.extend_from_slice(input);
+    for msg in input {
+        self.memory.record(msg.clone());
+    }
 }
```

In `act()`, prepend memory context before the inbox:

```diff
     // 3. Memory context (prior cycles)
+    let memory_msgs: Vec<_> = self.memory.messages()
+        .iter()
+        .filter(|m| !self.inbox.contains(m))
+        .cloned()
+        .collect();
+    messages.extend(memory_msgs);

     // 4. The observed conversation (current cycle)
     messages.extend(std::mem::take(&mut self.inbox));
```

The constructor takes a configurable memory capacity (default 32):

```diff
-pub fn new(persona: Persona, provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
+pub fn new(persona: Persona, provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
     Self {
         persona,
         provider,
         model: model.into(),
         inbox: Vec::new(),
         last_thinking: None,
+        memory: ShortTermMemory::new(32),
     }
 }
```

---

### 5. Application — `src/application/agent_observability.rs` [MODIFY]

Add the new `RuntimeEvent` variant:

```diff
 pub enum RuntimeEvent {
     AgentObserving { agent: AgentId },
     AgentThinking { agent: AgentId },
     AgentActing { agent: AgentId },
     AgentReflected { agent: AgentId, satisfied: bool },
     AgentActed { agent: AgentId, produced: bool },
     AgentFailed { agent: AgentId, error: String },
+    /// An agent published a message to the inter-agent bus.
+    AgentPublished { agent: AgentId },
+    /// An agent's lifecycle status changed.
+    AgentLifecycle { agent: AgentId, status: String },
 }
```

Update `agent()` and `narrate()` for the new variants.

---

### 6. Documentation — `docs/Product_Engineering/FEATURE_MAP.md` [MODIFY]

Update item 2.4 to reflect the narrowed gap.

---

## Architecture Compliance

| Invariant | Compliance |
|-----------|-----------|
| Domain pure (no I/O) | ✅ `ShortTermMemory` is a pure data structure |
| Async safety | ✅ Registry uses `Arc<tokio::sync::RwLock<T>>` |
| No unwrap/expect/panic | ✅ All paths use `?` or graceful handling |
| Observability via tracing | ✅ New events narrate through `tracing` |
| Serial cascade preserved | ✅ Only read-only cognitive cycles run concurrently |
| IPC boundary | ✅ No changes to Rust↔Nim protocol |

---

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

#### New Tests

| Test | File | Validates |
|------|------|-----------|
| `records_up_to_capacity` | `memory.rs` | Sliding window eviction |
| `clear_empties_all` | `memory.rs` | Memory clear |
| `agents_see_prior_outputs` | `agent_runtime.rs` | Inter-agent bus history injected |
| `agent_bus_publishes_outputs` | `agent_runtime.rs` | Messages appear on agent bus |
| `register_and_terminate` | `agent_runtime.rs` | Lifecycle status tracking |
| `memory_accumulates_across_observe` | `persona_agent.rs` | Memory persists between calls |
| `new_event_variants_narrate` | `agent_observability.rs` | New variants emit tracing |

### Manual Verification

1. Run `cargo test --all-targets` — all 140+ tests pass
2. Run `scripts/quality-gate.sh` — full gate passes

---

## Model Recommendation

**Gemini 3.1 Pro (Low)** — This is mechanical struct additions, plumbing, and test writing.
No architectural decisions or complex reasoning required.
