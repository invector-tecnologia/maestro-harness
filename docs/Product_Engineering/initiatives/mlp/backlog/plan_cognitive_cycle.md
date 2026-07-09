# Implementation Plan: Cognitive Cycle Enrichment (Item 2.1)

## Goal

The FEATURE_MAP identifies the cognitive cycle as 🔴 **Critical** business value. Today, the `Role`
trait's `think()` is a no-op, `observe()` is a trivial append, and `act()` dumps the inbox to the
LLM with **no system prompt, no reasoning, no self-critique**. Agents don't even know what role
they're playing — the persona's `responsibility` text is never sent to the provider.

This plan implements three concrete improvements that turn agents from dumb prompt→response pipes
into identity-aware, reasoning agents:

1. **System Prompt Injection** — every agent prefixes its conversation with a system message built
   from the persona's responsibility and a structured thinking directive.
2. **Chain-of-Thought in `think()`** — the `think()` phase constructs a structured `ThinkingOutput`
   (task decomposition, approach selection, risk assessment) that is injected into the conversation
   context before `act()` calls the LLM.
3. **`reflect()` Phase** — a new trait method for post-act self-critique. The agent reviews its own
   output against the original task and flags quality concerns or suggests improvements.

> [!IMPORTANT]
> **Model Recommendation:** Claude Opus 4.6 (Thinking). This touches the core domain port (`Role`
> trait), the application layer (`PersonaAgent`), the runtime (`AgentRuntime`), and observability —
> multiple interconnected layers requiring careful reasoning.

---

## User Review Required

> [!WARNING]
> **Breaking change to the `Role` trait.** Adding `reflect()` changes the trait contract. Any future
> `Role` implementors (e.g., `RagCognitiveAgent`) will need to implement it. Since we currently have
> only one implementor (`PersonaAgent`), this is safe today.

> [!IMPORTANT]
> **`think()` remains synchronous and pure.** Per the manifesto, `think()` must never do I/O. The
> chain-of-thought reasoning happens locally using the persona's responsibility + observed context.
> It does **not** call the LLM — that would violate the architecture. The LLM call happens in `act()`,
> which now receives richer context thanks to `think()`.

---

## Proposed Changes

### 1. Domain Layer — ThinkingOutput Model

#### [NEW] src/domain/models/thinking.rs

A structured output from the `think()` phase. Pure data, no I/O.

```rust
//! Structured output from an agent's THINK phase.
//!
//! `ThinkingOutput` captures the agent's local reasoning before acting:
//! task decomposition, approach selection, and risk flagging. It is
//! injected into the conversation context so the LLM receives structured
//! guidance alongside the raw user input.

use serde::{Deserialize, Serialize};

/// The structured result of an agent's `think()` phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingOutput {
    /// How the agent interprets the task in terms of its responsibility.
    pub task_interpretation: String,
    /// The approach the agent will take (methodology, constraints).
    pub approach: String,
    /// Risks or concerns the agent identified.
    pub risks: Vec<String>,
    /// Whether the agent considers this task within its competence.
    pub within_competence: bool,
}

impl ThinkingOutput {
    /// Render the thinking output as a structured prompt fragment
    /// suitable for injection into the LLM conversation context.
    pub fn as_prompt_fragment(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("**Task Interpretation:** {}", self.task_interpretation));
        lines.push(format!("**Approach:** {}", self.approach));
        if !self.risks.is_empty() {
            lines.push(format!("**Risks:** {}", self.risks.join("; ")));
        }
        if !self.within_competence {
            lines.push("**Note:** This task may be outside my primary competence.".to_string());
        }
        lines.join("\n")
    }
}
```

#### [NEW] src/domain/models/reflection.rs

A structured output from the `reflect()` phase. Pure data, no I/O.

```rust
//! Structured output from an agent's REFLECT phase.
//!
//! After `act()`, the agent reviews its own output for quality concerns.

use serde::{Deserialize, Serialize};

/// The structured result of an agent's post-act self-critique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectionOutput {
    /// Whether the agent is satisfied with its output.
    pub satisfied: bool,
    /// Quality concerns identified during reflection.
    pub concerns: Vec<String>,
    /// Suggested improvements (for future iterations or audit).
    pub suggestions: Vec<String>,
}
```

#### [MODIFY] src/domain/models/mod.rs

Export the new models.

---

### 2. Domain Layer — Role Trait Expansion

#### [MODIFY] src/domain/ports/role.rs

Expand the trait from `observe → think → act` to `observe → think → act → reflect`.
`think()` now returns a `ThinkingOutput`. `reflect()` is a new post-act phase.

```rust
//! `Role` — the cognitive contract every agent implements (TASK 010).
//!
//! The innermost loop: `OBSERVE → THINK → ACT → REFLECT`.
//! `think` is synchronous and side-effect-free. `reflect` is synchronous
//! and reviews the agent's own output for quality concerns.

use async_trait::async_trait;

use crate::domain::models::{AgentId, Message, ThinkingOutput, ReflectionOutput};

use super::LlmError;

/// An agent's cognitive contract.
#[async_trait]
pub trait Role: Send + Sync {
    /// Stable identity of this agent.
    fn id(&self) -> &AgentId;

    /// OBSERVE: register incoming messages as the current unit of work.
    fn observe(&mut self, input: &[Message]);

    /// THINK: reason about observed input. Must be pure — no I/O, no external
    /// state mutation beyond the agent's own working memory.
    /// Returns structured reasoning that will be injected into the act() context.
    fn think(&mut self) -> ThinkingOutput;

    /// ACT: produce an optional output message (may perform provider I/O).
    async fn act(&mut self) -> Result<Option<Message>, LlmError>;

    /// REFLECT: review the output of act() for quality concerns.
    /// Pure — no I/O. Called only when act() produced a message.
    fn reflect(&self, output: &Message) -> ReflectionOutput;
}
```

---

### 3. Application Layer — PersonaAgent Enrichment

#### [MODIFY] src/application/persona_agent.rs

The main implementation. Key changes:

1. **System prompt construction**: Build a system message from persona responsibility.
2. **`think()` implementation**: Analyze the inbox to produce a `ThinkingOutput` based on keyword
   matching against the persona's responsibility.
3. **`act()` enrichment**: Prepend system prompt + thinking output to the LLM request.
4. **`reflect()` implementation**: Basic heuristic self-critique (length check, empty response
   detection, off-topic flagging).

```rust
pub struct PersonaAgent {
    persona: Persona,
    provider: Arc<dyn LlmProvider>,
    model: String,
    inbox: Vec<Message>,
    last_thinking: Option<ThinkingOutput>,
}

impl PersonaAgent {
    /// Build the system prompt from the persona's identity.
    fn system_prompt(&self) -> String {
        format!(
            "You are '{}'. Your responsibility: {}\n\n\
             Follow a structured approach:\n\
             1. Interpret the task in terms of your specific role.\n\
             2. Apply your expertise to produce a focused, actionable contribution.\n\
             3. Flag any risks or concerns within your domain.\n\
             4. Stay within your responsibility boundary — delegate what is outside it.",
            self.persona.id, self.persona.responsibility
        )
    }

    /// Heuristic: does the demand overlap with this persona's responsibility keywords?
    fn assess_competence(&self, demand: &str) -> bool {
        let responsibility_lower = self.persona.responsibility.to_lowercase();
        let demand_lower = demand.to_lowercase();
        // Simple word-overlap heuristic
        responsibility_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // skip articles/prepositions
            .any(|word| demand_lower.contains(word))
    }
}

#[async_trait]
impl Role for PersonaAgent {
    fn think(&mut self) -> ThinkingOutput {
        let combined_input: String = self.inbox
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let within_competence = self.assess_competence(&combined_input);

        let output = ThinkingOutput {
            task_interpretation: format!(
                "As {}, I interpret this task through the lens of: {}",
                self.persona.id, self.persona.responsibility
            ),
            approach: format!(
                "I will apply my expertise in '{}' to produce a focused contribution.",
                self.persona.responsibility
            ),
            risks: if within_competence {
                Vec::new()
            } else {
                vec!["This task may be partially outside my primary responsibility.".to_string()]
            },
            within_competence,
        };

        self.last_thinking = Some(output.clone());
        output
    }

    async fn act(&mut self) -> Result<Option<Message>, LlmError> {
        if self.inbox.is_empty() {
            return Ok(None);
        }

        let mut messages = Vec::new();

        // 1. System prompt (persona identity)
        if let Ok(sys) = Message::system(self.system_prompt()) {
            messages.push(sys);
        }

        // 2. Thinking output as context
        if let Some(ref thinking) = self.last_thinking {
            if let Ok(ctx) = Message::system(format!(
                "[Internal Reasoning]\n{}",
                thinking.as_prompt_fragment()
            )) {
                messages.push(ctx);
            }
        }

        // 3. The observed conversation
        messages.extend(std::mem::take(&mut self.inbox));

        let request = CompletionRequest {
            model: self.model.clone(),
            messages,
        };
        let response = self.provider.complete(request).await?;
        self.last_thinking = None;
        let message = Message::assistant(self.persona.id.clone(), response.content)
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
        Ok(Some(message))
    }

    fn reflect(&self, output: &Message) -> ReflectionOutput {
        let mut concerns = Vec::new();
        let mut suggestions = Vec::new();

        // Heuristic 1: Very short responses may lack substance
        if output.content.len() < 20 {
            concerns.push("Response is very short — may lack actionable detail.".to_string());
            suggestions.push("Consider elaborating with specific steps or examples.".to_string());
        }

        // Heuristic 2: Very long responses may lack focus
        if output.content.len() > 5000 {
            concerns.push("Response is very long — may lack focus.".to_string());
            suggestions.push("Consider condensing to key actionable points.".to_string());
        }

        ReflectionOutput {
            satisfied: concerns.is_empty(),
            concerns,
            suggestions,
        }
    }
}
```

---

### 4. Application Layer — Runtime Integration

#### [MODIFY] src/application/agent_runtime.rs

Update `run_cycle` to call `think()` and `reflect()`, and narrate them.

```diff
 set.spawn(async move {
     let id = agent.id().clone();

     emit(&events, RuntimeEvent::AgentObserving { agent: id.clone() }).await;
     agent.observe(&input);

     emit(&events, RuntimeEvent::AgentThinking { agent: id.clone() }).await;
-    agent.think();
+    let thinking = agent.think();

     emit(&events, RuntimeEvent::AgentActing { agent: id.clone() }).await;
     match agent.act().await {
         Ok(output) => {
+            // REFLECT phase: self-critique when output was produced
+            if let Some(ref msg) = output {
+                let reflection = agent.reflect(msg);
+                emit(
+                    &events,
+                    RuntimeEvent::AgentReflected {
+                        agent: id.clone(),
+                        satisfied: reflection.satisfied,
+                    },
+                )
+                .await;
+            }
             emit(
                 &events,
                 RuntimeEvent::AgentActed {
                     agent: id,
                     produced: output.is_some(),
                 },
             )
             .await;
             output
         }
         // ... error handling unchanged
     }
 });
```

---

### 5. Application Layer — Observability

#### [MODIFY] src/application/agent_observability.rs

Add the `AgentReflected` event variant.

```diff
 pub enum RuntimeEvent {
     AgentObserving { agent: AgentId },
     AgentThinking { agent: AgentId },
     AgentActing { agent: AgentId },
+    /// The agent reflected on its output (REFLECT phase).
+    AgentReflected { agent: AgentId, satisfied: bool },
     AgentActed { agent: AgentId, produced: bool },
     AgentFailed { agent: AgentId, error: String },
 }
```

Update `agent()`, `narrate()`, and the test to handle the new variant.

---

### 6. Documentation

#### [MODIFY] docs/Product_Engineering/FEATURE_MAP.md

Update item 2.1 status:

```diff
-- **What It Does Today:** `observe() → think() → act()` trait. `PersonaAgent` implements `Role`.
-  `think()` is a no-op placeholder. `act()` calls LLM provider for completion.
+- **What It Does Today:** `observe() → think() → act() → reflect()` trait with structured
+  `ThinkingOutput` and `ReflectionOutput`. `PersonaAgent` injects persona system prompts,
+  produces chain-of-thought reasoning in `think()`, and self-critiques in `reflect()`.
-- **Gap:** `think()` is empty — agents are pure prompt→response pipes, no reasoning.
+- **Gap:** Reasoning is heuristic-based (not LLM-powered). No tool selection in think().
+  No streaming responses yet. Full 6-phase SENSE→…→DELIVER is at orchestrator level only.
```

#### [MODIFY] docs/Maestro_Manifesto/reference/COGNITIVE_PATTERN.md

Update the per-agent loop section to reflect the expanded cycle.

---

## Summary of All Changes

| File | Change |
|------|--------|
| `src/domain/models/thinking.rs` | **[NEW]** `ThinkingOutput` struct with `as_prompt_fragment()` |
| `src/domain/models/reflection.rs` | **[NEW]** `ReflectionOutput` struct |
| `src/domain/models/mod.rs` | Export new models |
| `src/domain/ports/role.rs` | Expand `Role` trait: `think()` returns `ThinkingOutput`, add `reflect()` |
| `src/application/persona_agent.rs` | System prompt injection, real `think()`, `reflect()`, `assess_competence()` |
| `src/application/agent_runtime.rs` | Call `reflect()` after `act()`, narrate it |
| `src/application/agent_observability.rs` | Add `AgentReflected` event variant |
| `docs/Product_Engineering/FEATURE_MAP.md` | Update item 2.1 |
| `docs/Maestro_Manifesto/reference/COGNITIVE_PATTERN.md` | Update per-agent loop docs |

---

## Verification Plan

### Automated Tests

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

**New/updated tests:**
- `thinking.rs`: `ThinkingOutput::as_prompt_fragment()` renders correctly.
- `reflection.rs`: `ReflectionOutput` construction.
- `role.rs`: No test changes (trait only).
- `persona_agent.rs`:
  - `think_produces_structured_output` — verify `ThinkingOutput` fields are populated.
  - `act_includes_system_prompt` — verify the `CompletionRequest` contains a system message.
  - `reflect_flags_short_response` — verify short output triggers concern.
  - `reflect_satisfied_for_normal_response` — verify normal output passes.
  - Existing tests updated to call `think()` before `act()`.
- `agent_runtime.rs`: Existing tests updated; `AgentReflected` event is narrated.
- `agent_observability.rs`: New variant covered in `exposes_agent_for_every_variant`.

### Manual Verification

```bash
# Full quality gate
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```
