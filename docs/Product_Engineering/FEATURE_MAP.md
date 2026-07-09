# Maestro Feature Map

> **Version:** 0.3.0 · **Date:** 2026-07-09 · **Status:** Living Document

## Overview

This document is the **strategic feature inventory** for Maestro Harness — a local-first, tactical
Agentic Workflow Orchestrator for disposable micro-projects. It catalogs every current and planned
capability, describes what each feature does today and what it should do to be competitive, and
ranks each feature by **business value** benchmarked against named competitors.

### How to Read This Map

Each feature is described with a standardized card:

| Field | Meaning |
|-------|---------|
| **Status** | ✅ Implemented · 🚧 Partial · ⬜ Stub · 📋 Planned |
| **Source** | File path(s) in the codebase |
| **Business Value** | 🔴 Critical · 🟠 High · 🟡 Medium · 🟢 Low — ranked by competitive impact |
| **What It Does Today** | Current behavior |
| **What It Should Do** | Target behavior for competitive parity or advantage |
| **Gap** | Specific delta between current and target |
| **Competitor Benchmark** | What named competitors do for this capability |

### Competitors Benchmarked

| Competitor | Category | Key Strength |
|------------|----------|-------------|
| **OpenCode** | Terminal-native AI agent (160k+ ★) | Multi-provider TUI, LSP integration, MCP support, 75+ providers |
| **Maestri** | Visual agent canvas (macOS) | Infinite canvas orchestration, shared memory, on-device AI (Ombro) |
| **MetaGPT** | Multi-agent SOP framework | SOP-driven "Code = SOP(Team)", role-based assembly line |
| **Aider** | Git-first terminal pair programmer | Repo-map, atomic git commits, architect/editor mode, voice-to-code |
| **Claude Code** | Terminal-native agentic coding | MCP ecosystem, managed memory, 40+ commands, auto-pilot debugging |
| **Gemini CLI / Antigravity** | Google terminal agent | 1M+ token context, multi-agent orchestration, Google Cloud integration |

### Status Dashboard

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Implemented | 40 | 62% |
| 🚧 Partial | 5 | 8% |
| ⬜ Stub | 2 | 3% |
| 📋 Planned | 18 | 27% |
| **Total** | **65** | 100% |

### Business Value Distribution

| Rank | Count | Meaning |
|------|-------|---------|
| 🔴 Critical | 12 | Must-have for competitive viability — competitors all ship this |
| 🟠 High | 18 | Strong differentiation or market expectation |
| 🟡 Medium | 20 | Nice-to-have, improves polish and adoption |
| 🟢 Low | 15 | Future consideration, niche, or already adequate |

---

## Domain 1: Project Bootstrap & CLI Surface

### 1.1 `maestro init` — Project Scaffolding

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`, `src/presentation/cli/templates.rs`, `src/presentation/cli/detect.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Interactive prompts with project auto-detection (Cargo.toml, package.json, go.mod, etc.). Template gallery with 4 templates (web-app, cli-tool, library, infra). Generates starter task specs in `maestro/tasks/`. Supports `--template <name>` for non-interactive CI-friendly bootstrapping. `list-templates` command shows available options.
- **What It Should Do:** Community template registry (remote fetch, opt-in). AI-assisted scope generation from a one-sentence description. Template composition (combine templates). Framework-specific templates (e.g., "rust-axum-api", "react-nextjs").
- **Gap:** Templates are generic (not framework-specific). No remote registry. No AI-assisted generation.
- **Competitor Benchmark:**
  - *OpenCode*: Zero-config init — auto-discovers project from current directory
  - *Aider*: No init needed — works with any existing Git repo immediately
  - *MetaGPT*: Init generates PRD, system design, and API spec from a single sentence. **Maestro now matches MetaGPT's template concept but not its AI-generated content.**

---

### 1.2 `maestro config` — Provider-Aware Setup

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`, `src/presentation/cli/providers.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Supports `--provider ollama|openai|gemini` to pre-populate the correct provider block. Auto-detects running Ollama and environment API keys. Runs an instant connection test after config generation. Supports `--endpoint` and `--model` for CI/scripting.
- **What It Should Do:** Full interactive TUI-based provider wizard with model browsing from live Ollama catalog. Multi-provider config (configure all detected providers at once).
- **Gap:** No interactive TUI wizard. No live model listing from Ollama `/api/tags`.
- **Competitor Benchmark:**
  - *OpenCode*: Interactive model selection on first run, auto-discovers Ollama
  - *Claude Code*: API key prompt on first launch, instant connection test.
    **Maestro now matches Claude Code's connection-test and OpenCode's auto-discovery.**

---

### 1.3 `maestro validate-config` — Config Validation

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`, `src/infrastructure/config.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Parses + validates `config.yml` with cross-reference checking
  (providers/models/agents). Reports typed errors with inline suggestions. Runs active network probes against configured endpoints. Supports `--fix` for auto-repairing dangling defaults.
- **What It Should Do:** Offer interactive CLI prompts for repairing complex errors instead of just `--fix` or dropping them.
- **Gap:** No interactive repair wizard.
- **Competitor Benchmark:**
  - *OpenCode*: Auto-validates on startup with inline error messages
  - *Aider*: Model validation with fallback suggestions
    **Maestro now matches OpenCode and Aider with inline suggestions, plus adds auto-fix and active probing.**

---

### 1.4 `maestro doctor` — System Health Check

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`, `src/application/readiness.rs`, `src/infrastructure/system.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Produces a comprehensive health report card covering Toolchain (Git, Nim), local hardware accelerators (NVIDIA/Apple Silicon), Governance integrity, and active network provider probes.
- **What It Should Do:** Add automatic check for Maestro binary updates via crates.io or GitHub releases. Add disk space warning for huge LLM caches.
- **Gap:** No update checker. No disk space check.
- **Competitor Benchmark:**
  - *OpenCode*: Comprehensive diagnostics including LSP, environment, and model availability
  - *Claude Code*: `/doctor` command with permission, config, and connection checks
    **Maestro matches competitors and uniquely detects hardware for local LLM acceleration.**

---

### 1.5 `maestro list-agents` — Persona Catalog

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Prints a formatted table showing Name, Role (orchestrator/operational), Provider, Model, and Responsibility for each persona. Resolves per-agent config bindings or falls back to system defaults. Includes a Handoff Matrix showing the interaction graph. `--json` flag for machine-readable output.
- **What It Should Do:** Show per-agent last activity timestamp, skill inventory, and current task assignment.
- **Gap:** No activity tracking or skill listing (requires runtime state).
- **Competitor Benchmark:**
  - *MetaGPT*: Lists roles with responsibilities, skills, and assigned models
  - *CrewAI*: Agent catalog with backstories, tools, and delegation rules
    **Maestro now matches competitors on catalog detail and adds a unique handoff matrix visualization.**

---

### 1.6 `maestro config` — Governance Folders

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`, `src/presentation/cli/providers.rs`
- **Business Value:** 🟢 Low
- **What It Does Today:** The standalone `scaffold-markdown` command was removed. Governance scaffolding is now triggered automatically by `maestro config`, reducing the number of setup steps required.
- **What It Should Do:** Auto-populate with contextual starter content. Include example persona and skill definitions. Generate from project type template.
- **Gap:** Empty folder creation only.
- **Competitor Benchmark:** No direct equivalent — governance folders are unique to Maestro.

---

### 1.7 `maestro version` — Version Info

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`
- **Business Value:** 🟢 Low
- **What It Does Today:** Prints version, commit hash, build date, Rust edition, and active provider/model from config. Supports `--json` for CI scripting.
- **What It Should Do:** Include build metadata (commit hash, build date, toolchain). Opt-in
  update check.
- **Gap:** No opt-in update check (future work).
- **Competitor Benchmark:**
  - *Claude Code*: `/version` shows build, model, and API version
  - *Aider*: `--version` includes model info and package versions

---

### 1.8 `--no-tui` Headless Mode

- **Status:** ✅ Implemented (enhanced)
- **Source:** `src/presentation/cli/mod.rs`, `src/presentation/ipc/server.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** Runs core without TUI. Full duplex IPC server over stdin/stdout.
  `--message` flag for fully non-interactive CI/CD: sends demand, auto-approves gates,
  streams structured JSON events, exits with code 0/1/2. Human-readable `tracing` on stderr.
- **What It Should Do:** Structured JSON event stream for CI/CD integration. `--json` output format.
  Machine-readable event stream. Exit codes for pass/fail. JUnit XML test report output.
- **Gap:** No JUnit XML test report output (future work).
- **Competitor Benchmark:**
  - *Claude Code*: `--json` flag for structured output, `--print` for non-interactive
  - *Aider*: `--message` flag for non-interactive, exit codes for CI
  - *OpenCode*: Headless mode with structured JSON events

---

### 1.9 Project Templates

- **Status:** ✅ Implemented
- **Source:** `src/presentation/cli/templates.rs`, `src/presentation/cli/mod.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** 6 built-in templates (web-app, cli-tool, library, infra, api-service,
  data-pipeline) with pre-configured scope docs, task specs, persona hints, and starter skills.
  `maestro list-templates --json` for CI scripting.
- **What It Should Do:** Library of templates (web app, API service, CLI tool, data pipeline, infra
  module) with pre-configured personas, skills, and scope documents. Community template registry.
- **Gap:** No community template registry (future network feature).
- **Competitor Benchmark:**
  - *MetaGPT*: Built-in project types with full SOP generation
  - *OpenCode*: Custom skill templates and configurations

---

## Domain 2: Multi-Agent Runtime & Cognitive Architecture

### 2.1 Cognitive Cycle (Role Trait)

- **Status:** ✅ Implemented
- **Source:** `src/domain/ports/role.rs`, `src/application/persona_agent.rs`
- **Business Value:** 🔴 Critical
- **What It Does Today:** `observe() → think() → act() → reflect()` trait with structured
  `ThinkingOutput` and `ReflectionOutput`. `PersonaAgent` injects persona system prompts,
  produces chain-of-thought reasoning in `think()`, and self-critiques in `reflect()`.
- **What It Should Do:** `think()` should implement chain-of-thought reasoning, tool selection, and
  plan refinement. Add `reflect()` phase for self-critique. Support streaming responses. Implement
  the full SENSE → OBSERVE → THINK → ACT → AUDIT → DELIVER cycle from the manifesto.
- **Gap:** Reasoning is heuristic-based (not LLM-powered). No tool selection in think().
  No streaming responses yet. Full 6-phase SENSE→…→DELIVER is at orchestrator level only.
- **Competitor Benchmark:**
  - *Claude Code*: Full ReAct loop with tool use, reflection, and retry
  - *MetaGPT*: Each role has structured thinking with SOP-guided output
  - *OpenCode*: Agentic loop with plan, execute, observe, iterate

---

### 2.2 Default Persona Catalog

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/persona.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** 8 personas: Maestro (orchestrator), Project Manager, QA, UX, Software
  Engineer, DevOps Engineer, Security Analyst, Technical Writer. Each has responsibility text,
  curated system prompt, expertise keywords, skill tags, and interaction matrix.
- **What It Should Do:** Expand to 8-10 personas (add DevOps, Security, Data Engineer,
  Documentation Writer). Each with detailed system prompts, skill bindings, tool access lists,
  temperature settings, and example outputs.
- **Gap:** No temperature settings or example outputs yet. Skill tags are defined but
  not yet wired to runtime skill selection. Tool access lists are not implemented.
- **Competitor Benchmark:**
  - *MetaGPT*: Product Manager, Architect, Project Manager, Engineer — each with full SOP
  - *Maestri*: Arbitrary agent types connected on visual canvas with shared memory

---

### 2.3 Custom Personas (Config Mode)

- **Status:** ✅ Implemented
- **Source:** `src/application/governance.rs`, `frontend/src/panels/config.nim`
- **Business Value:** 🟡 Medium
- **What It Does Today:** YAML persona files in `maestro/personas/`, loaded and merged with defaults
  for Two-Towers routing. Config Mode governance navigator + editor in TUI.
- **What It Should Do:** Rich persona schema: system prompt template, skill list, tool whitelist,
  model preferences, temperature settings, context window strategy. Visual persona editor.
- **Gap:** Schema is minimal (name + responsibility). No system prompts or tool bindings.
- **Competitor Benchmark:**
  - *CrewAI*: Agent definition with backstory, goal, tools, and delegation rules
  - *Claude Code*: `CLAUDE.md` project memory defines conventions and style

---

### 2.4 Multi-Agent Runtime

- **Status:** ✅ Implemented
- **Source:** `src/application/agent_runtime.rs`
- **Business Value:** 🔴 Critical
- **What It Does Today:** Concurrent `JoinSet` runs all agents' cognitive cycles in parallel
  (read-only). Failing agents are isolated. `BroadcastBus<RuntimeEvent>` for event fan-out.
- **What It Should Do:** Agent memory (short-term context, long-term vector store). Agent-to-agent
  messaging. Backpressure and token budgets per agent. Streaming intermediate results. Agent
  lifecycle management (spawn, pause, resume, terminate).
- **Gap:** Agents are stateless, memoryless, and can't communicate with each other.
- **Competitor Benchmark:**
  - *MetaGPT*: Agents observe each other's outputs via shared environment
  - *Maestri*: Visual cables connect agents for direct communication + shared memory
  - *Claude Code*: Subagent spawning with message-passing and task delegation

---

### 2.5 Agent Observability

- **Status:** ✅ Implemented
- **Source:** `src/application/agent_observability.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** `RuntimeEvent` with 5 variants (Observing, Thinking, Acting, Acted,
  Failed) emitted via structured `tracing`.
- **What It Should Do:** Real-time dashboard in TUI. Token usage per agent per cycle. Cost
  estimation. Performance metrics (latency, success rate). Exportable audit logs.
- **Gap:** Events are emitted to tracing only — no aggregation, no dashboard, no cost tracking.
- **Competitor Benchmark:**
  - *Claude Code*: `/usage` and `/cost` commands, per-session token tracking
  - *Maestri*: On-device Ombro AI summarizes agent activity while you're away
  - *OpenCode*: Token and cost tracking per session

---

### 2.6 Two-Towers Persona↔Skill Routing

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/routing.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** Lexical token-overlap scorer. Deterministic ranking with stable sort and
  tie-breaking by ID. Fallback to Software Engineer. Min score threshold of 1.
- **What It Should Do:** Embedding-based scoring (local embeddings via Ollama). Hybrid
  lexical+semantic scoring. Learned routing from historical success data. Confidence-weighted
  delegation. Routing explanation in narration.
- **Gap:** Lexical-only — misses semantic matches. No learning from outcomes.
- **Competitor Benchmark:**
  - *MetaGPT*: Role selection by SOP-driven task type matching
  - *Claude Code*: Single agent, no routing needed (but skill-based tool selection)
  - *Maestri*: Manual visual routing via canvas cables

---

### 2.7 Per-Agent Model Routing

- **Status:** ✅ Implemented
- **Source:** `src/application/model_router.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** Config `agents:` map binds persona ID → provider+model. Falls back to
  system default.
- **What It Should Do:** Model cascading (try fast model first, escalate on failure). Per-task model
  selection based on complexity estimation. Cost-aware routing.
- **Gap:** Static binding only. No cascading or cost awareness.
- **Competitor Benchmark:**
  - *OpenCode*: Per-session model switching, multi-provider hot-swap
  - *Aider*: Architect mode uses strong model for planning, weaker model for editing
  - *Gemini CLI*: Model selection by task complexity

---

### 2.8 Agent Memory

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🔴 Critical
- **What It Does Today:** Nothing — every session starts from zero context.
- **What It Should Do:** Short-term: sliding window context with summarization. Long-term: vector
  store (local embeddings) for project knowledge. Cross-session persistence. RAG integration for
  codebase grounding.
- **Gap:** Full feature gap. This is the #1 competitive disadvantage.
- **Competitor Benchmark:**
  - *Claude Code*: `CLAUDE.md` managed memory + cross-session project context
  - *OpenCode*: SQLite-backed persistent session history
  - *Aider*: Repo-map for automatic codebase context selection
  - *Maestri*: Shared markdown memory nodes between agents

---

### 2.9 Agent Tool Use

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🔴 Critical
- **What It Does Today:** Nothing — agents can only generate text via LLM completion.
- **What It Should Do:** Tool-use framework: file read/write, shell execution (sandboxed), web
  search, code analysis (AST parsing), test runner, git operations. Tool approval gates consistent
  with governance model. MCP (Model Context Protocol) client support.
- **Gap:** Full feature gap. This is the #2 competitive disadvantage.
- **Competitor Benchmark:**
  - *Claude Code*: File editing, shell commands, MCP tools, web search, subagent spawning
  - *OpenCode*: File editing, shell execution, LSP integration, MCP support
  - *Aider*: File editing, test running, linting, voice input
  - *MetaGPT*: Code writing, execution, web browsing, data analysis tools

---

### 2.10 Inter-Agent Communication

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟠 High
- **What It Does Today:** Nothing — agents don't communicate during a cycle.
- **What It Should Do:** Inter-agent messaging bus. Shared scratchpad/blackboard pattern.
  Hierarchical delegation (Maestro → PM → SWE). Critic/reviewer patterns.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *MetaGPT*: Agents publish artifacts to shared environment, others observe
  - *Maestri*: Visual cables for agent-to-agent communication + shared memory nodes
  - *Claude Code*: Subagent messaging via `send_message`

---

## Domain 3: Orchestration Engine — FSM & Cascade

### 3.1 Six-Stage FSM

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/fsm.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** `Ideation → Planning → Approval → Instrumentation → Execution →
  Verification`. Typed stages with illegal transition rejection. `tracing` events on each
  transition. `MicroProject` tracks id, demand, and current stage.
- **What It Should Do:** Optional sub-stages (e.g., Execution → {Setup, Run, Teardown}).
  Conditional branching (skip stages for simple tasks). Stage timeout policies. Stage retry with
  backoff. FSM visualization in TUI.
- **Gap:** Rigid linear flow. No sub-stages, branching, or timeouts.
- **Competitor Benchmark:**
  - *LangGraph*: Arbitrary graph-based state machines with branching and cycles
  - *MetaGPT*: Linear SOP pipeline but with structured artifact output per stage

---

### 3.2 Serial Cascade Executor

- **Status:** ✅ Implemented
- **Source:** `src/application/orchestrator.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** Strictly sequential execution of environment-affecting steps. Failed step
  halts cascade. Max 64 steps. Steps are text descriptions, not executable commands.
- **What It Should Do:** Step-level progress reporting. Conditional steps (if/else). Parallel
  read-only substeps. Step timeout with configurable policy (fail/skip/retry). **Executable steps**
  (shell commands, file operations, git commands).
- **Gap:** Steps are symbolic text only — not executable. No progress reporting.
- **Competitor Benchmark:**
  - *Claude Code*: Executes real shell commands with approval gates
  - *OpenCode*: Runs commands, reads output, iterates autonomously
  - *Aider*: Executes tests, lint, and iterates on failures

---

### 3.3 Approval Gates

- **Status:** ✅ Implemented
- **Source:** `src/application/orchestrator.rs`, `frontend/src/panels/maestro.nim`
- **Business Value:** 🟠 High — **Maestro differentiator**
- **What It Does Today:** Plan approval + execution approval. Blocks on IPC until user responds
  y/n. Rejection aborts session. Approval prompt displayed in Maestro Mode.
- **What It Should Do:** Rich approval UI: detailed plan with diffs, estimated impact, rollback
  preview, cost estimate. Partial approval (approve some steps, modify others). Auto-approve for
  trusted operations. Approval delegation (approve via API for CI).
- **Gap:** Binary y/n only. No diff preview, no partial approval, no auto-approve.
- **Competitor Benchmark:**
  - *Claude Code*: Permission modes (ask, auto-approve, always-ask) with sandboxed execution
  - *OpenCode*: Per-command approval with smart defaults
  - *Aider*: Diff display before applying changes, with y/n per file

---

### 3.4 Rollback-as-a-Service

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/rollback.rs`
- **Business Value:** 🟠 High — **Maestro differentiator**
- **What It Does Today:** `CascadeStep` with forward+inverse text pairs. `RollbackPlan` with
  reverse-order inverse execution. Currently **symbolic** (inverse descriptions, not executable).
- **What It Should Do:** **Executable rollback inverses**: git revert, file restore, command undo.
  Automatic rollback on cascade failure. Rollback dry-run preview. Partial rollback (undo last N
  steps). Rollback audit trail.
- **Gap:** Inverses are text descriptions only — cannot actually undo anything.
- **Competitor Benchmark:**
  - *Aider*: Git-native rollback — every change is a commit, `git revert` works naturally
  - *Claude Code*: Checkpoint-based undo with `/undo` command
  - No competitor has Maestro's **structured rollback plan concept** — this is a unique advantage
    once made executable.

---

### 3.5 Orchestrator Session Management

- **Status:** ✅ Implemented
- **Source:** `src/application/orchestrator.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Full flow: demand → route → plan → approve → instrument → cascade →
  verify → deliver. Provider-backed deliverables with deterministic fallback. Single-flight
  enforcement. Session state machine (Idle, Running, AwaitingPlan, AwaitingExecution, Done).
- **What It Should Do:** Multi-demand sessions (queue). Parallel independent micro-projects. Session
  checkpointing and resume. Session history with replay. Demand decomposition.
- **Gap:** Single demand per session. No checkpointing or resume.
- **Competitor Benchmark:**
  - *Claude Code*: Multi-turn sessions with `/resume` and context persistence
  - *Maestri*: Parallel sessions on visual canvas with independent context

---

### 3.6 Instrumentation (Context Injection)

- **Status:** 🚧 Partial
- **Source:** `src/application/orchestrator.rs`
- **Business Value:** 🔴 Critical
- **What It Does Today:** Generates persona bindings and injects system prompt. Minimal prompt
  enrichment — only name + demand text.
- **What It Should Do:** Rich context injection: persona instructions, skill definitions, scope
  documents, RAG-retrieved codebase context, prior conversation history, project conventions.
  Template-based prompt construction.
- **Gap:** Nearly empty prompts — agents have no project context.
- **Competitor Benchmark:**
  - *Claude Code*: Automatic codebase indexing, `CLAUDE.md` conventions, relevant file injection
  - *Aider*: Repo-map with automatic relevant context selection
  - *OpenCode*: LSP integration for code intelligence injection
  - *MetaGPT*: Full PRD/design/spec context passed between SOP stages

---

### 3.7 Verification Stage

- **Status:** 🚧 Partial
- **Source:** `src/domain/models/fsm.rs`, `src/application/orchestrator.rs`
- **Business Value:** 🔴 Critical
- **What It Does Today:** Stage exists in FSM but verification is a pass-through. No actual output
  validation.
- **What It Should Do:** Automated checks: compile/build, test suite, lint. AI-powered output
  review by QA persona. Acceptance criteria validation against task spec.
- **Gap:** Verification does nothing.
- **Competitor Benchmark:**
  - *Aider*: Auto-runs tests + lint after changes, iterates on failures
  - *Claude Code*: Auto-executes tests, reviews diffs, retries on failure
  - *OpenCode*: Runs tests, checks compilation, iterates autonomously
  - *MetaGPT*: QA role reviews code and generates test cases

---

### 3.8 Demand Decomposition

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing — single demand → single cascade.
- **What It Should Do:** AI-powered demand analysis: break complex requests into atomic sub-tasks.
  Dependency graph. Parallel independent sub-tasks. Progress tracking.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *MetaGPT*: Automatic requirement → PRD → design → tasks decomposition
  - *Claude Code*: `/plan` command with task breakdown

---

### 3.9 Session Checkpointing

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing — crash = total state loss.
- **What It Should Do:** Persist session state at each FSM transition. Resume interrupted sessions.
  Crash recovery. Session export/import.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: Session persistence with `/resume`
  - *LangGraph*: Full checkpointing and state persistence

---

## Domain 4: Provider Ecosystem & LLM Integration

### 4.1 Ollama Adapter (Local-First Default)

- **Status:** ✅ Implemented
- **Source:** `src/infrastructure/llm/ollama.rs`
- **Business Value:** 🟠 High — **Maestro differentiator** (local-first)
- **What It Does Today:** Non-streaming `/api/generate` completion. `/api/tags` probe. URL
  normalization. No API key required.
- **What It Should Do:** Streaming support. Chat API (`/api/chat`). Model pull/management.
  Concurrent request pooling. Embedding generation (`/api/embed`) for Two-Towers upgrade.
- **Gap:** No streaming. No chat API. No embeddings.
- **Competitor Benchmark:**
  - *OpenCode*: Full Ollama support with streaming and model management
  - *Aider*: Ollama support with chat completions and streaming

---

### 4.2 OpenAI Adapter

- **Status:** ✅ Implemented
- **Source:** `src/infrastructure/llm/openai.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** `/chat/completions` completion. `/models` probe. Bearer auth via
  `OPENAI_API_KEY` env var.
- **What It Should Do:** Streaming (SSE). Function calling / tool use. Vision. Token counting. Rate
  limit handling with retry. Cost tracking. Azure OpenAI endpoint support.
- **Gap:** No streaming. No function calling. No vision.
- **Competitor Benchmark:**
  - *OpenCode*: Full OpenAI support including streaming, vision, and function calling
  - *Claude Code*: Not applicable (Anthropic-only)
  - *Aider*: Full OpenAI support with streaming and function calling

---

### 4.3 Gemini Adapter

- **Status:** ✅ Implemented
- **Source:** `src/infrastructure/llm/gemini.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** `generateContent` completion. Gemini-specific body format
  (`systemInstruction`, `user`/`model` roles). API key auth.
- **What It Should Do:** Streaming. Function calling. Multimodal. Token counting. Vertex AI support.
  OAuth2 browser login. Grounding with Google Search.
- **Gap:** No streaming. No function calling. No Vertex AI.
- **Competitor Benchmark:**
  - *Gemini CLI*: Full Gemini API with 1M+ token context, streaming, and tools
  - *OpenCode*: Gemini support via generic OpenAI-compatible endpoint

---

### 4.4 Anthropic Adapter

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟠 High
- **What It Does Today:** Nothing.
- **What It Should Do:** Full Claude API: `/messages` completion, streaming, system prompt, tool
  use, vision. `ANTHROPIC_API_KEY` env var. Extended thinking support.
- **Gap:** Full feature gap. Claude models are among the strongest for coding.
- **Competitor Benchmark:**
  - *Claude Code*: Native Anthropic integration (it IS the Anthropic tool)
  - *OpenCode*: Full Anthropic support with streaming
  - *Aider*: Full Anthropic support with streaming and tool use

---

### 4.5 Provider Registry & Factory

- **Status:** ✅ Implemented
- **Source:** `src/infrastructure/llm/registry.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** `BTreeMap`-based registry. Factory builds providers from config by kind
  ("ollama"/"openai"/"gemini"). Default provider resolution. API keys from env vars.
- **What It Should Do:** Hot-reload on config change. Health monitoring with automatic failover.
  Load balancing. Provider capability matrix (tools? streaming? vision?).
- **Gap:** No hot-reload, no failover, no capability tracking.
- **Competitor Benchmark:**
  - *OpenCode*: 75+ provider support with hot-switching
  - *Aider*: Dynamic model switching mid-session

---

### 4.6 Streaming Responses

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🔴 Critical
- **What It Does Today:** All completions are blocking request/response.
- **What It Should Do:** SSE streaming for all providers. Real-time token-by-token display in TUI.
  Streaming through IPC to Nim frontend. Cancel in-flight requests.
- **Gap:** Full feature gap. Users wait for entire response — feels unresponsive.
- **Competitor Benchmark:**
  - *ALL competitors* stream responses in real-time. This is table-stakes in 2026.

---

### 4.7 Token & Cost Management

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟠 High
- **What It Does Today:** Nothing — no counting, budgeting, or tracking.
- **What It Should Do:** Per-request token counting (prompt + completion). Per-agent budgets.
  Session cost estimation. Cost alerts and limits. Token dashboard in TUI.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: `/cost` and `/usage` with per-session tracking
  - *OpenCode*: Per-session token and cost display
  - *Aider*: Token cost tracking with model comparison

---

### 4.8 Model Context Protocol (MCP)

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🔴 Critical
- **What It Does Today:** Nothing.
- **What It Should Do:** MCP client for tool-use integration. Connect to external MCP servers
  (filesystem, databases, APIs). Expose Maestro's tools as MCP server.
- **Gap:** Full feature gap. MCP is the 2026 standard for tool integration.
- **Competitor Benchmark:**
  - *Claude Code*: First-class MCP support with server management, dynamic tool loading
  - *OpenCode*: MCP client support
  - *Gemini CLI / Antigravity*: MCP support for external tool connectivity

---

### 4.9 Embedding Generation

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟠 High
- **What It Does Today:** Nothing — Two-Towers uses lexical scoring only.
- **What It Should Do:** Local embedding via Ollama (`/api/embed`). OpenAI embeddings API.
  Sentence-transformers fallback. Used for: Two-Towers routing, RAG retrieval, semantic search.
- **Gap:** Full feature gap. Required by Two-Towers upgrade and RAG pipeline.
- **Competitor Benchmark:**
  - *Aider*: Repo-map with embedding-based context selection
  - *OpenCode*: Embedding support for codebase indexing

---

## Domain 5: Governance & Configuration

### 5.1 Config Schema (YAML)

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/config.rs`, `src/infrastructure/config.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** `config.yml`: system defaults (provider, model, concurrency), provider
  declarations (kind, endpoint, models, capabilities), agent bindings. Cross-ref validation.
  XDG fallback path resolution.
- **What It Should Do:** Environment-specific overrides (dev/staging/prod). Config inheritance.
  Secret management (keychain, env var references). Config migration tooling.
- **Gap:** No environment overrides, no secrets management.
- **Competitor Benchmark:**
  - *OpenCode*: Per-project and per-user config files, model-specific settings
  - *Claude Code*: `CLAUDE.md` + `~/.claude/settings.json` with permission controls

---

### 5.2 Governance CRUD

- **Status:** ✅ Implemented
- **Source:** `src/application/governance.rs`, `frontend/src/panels/config.nim`
- **Business Value:** 🟠 High — **Maestro differentiator**
- **What It Does Today:** List, read, create, save, archive governance entries (config, personas,
  skills, scopes). Archive = soft delete to `maestro/archive/`. Immutable Maestro persona.
  Full TUI navigator + editor.
- **What It Should Do:** Versioning (git-tracked changes). Entry templates. Bulk operations.
  Import/export governance sets. Governance diffing between environments.
- **Gap:** No versioning, no templates, no import/export.
- **Competitor Benchmark:** **No competitor has this.** Governance CRUD is unique to Maestro.

---

### 5.3 Governance Validation

- **Status:** ✅ Implemented
- **Source:** `src/application/governance.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Validates required entries exist (scopes, personas, skills). Config cross-
  ref validation.
- **What It Should Do:** Deep semantic validation: persona skill references exist, scope boundaries
  consistent, no circular dependencies. Governance health score. Fix suggestions.
- **Gap:** Structural validation only — no semantic checks.
- **Competitor Benchmark:** **No competitor has governance validation.**

---

### 5.4 Scope Documents

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/governance.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Governance kind exists. Directory-based storage. Placeholder content.
- **What It Should Do:** Rich scope definition: boundary rules, allowed file patterns, dependency
  constraints, security policies. Scope enforcement during execution.
- **Gap:** Scopes are stored but not enforced during execution.
- **Competitor Benchmark:**
  - *Claude Code*: Permission boundaries via `CLAUDE.md` and settings
  - *OpenCode*: Custom skill configurations define boundaries

---

### 5.5 Skill Definitions

- **Status:** ✅ Implemented
- **Source:** `src/domain/models/governance.rs`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Governance kind exists. Directory-based storage. Placeholder content.
- **What It Should Do:** Structured skill schema: description, triggers, required tools, example
  prompts, success criteria. Skill marketplace. Skill composition. Versioning.
- **Gap:** Skills are stored but have no structured schema.
- **Competitor Benchmark:**
  - *Claude Code*: Skills as bundled instructions, slash commands, and MCP configs
  - *OpenCode*: Custom agent skills with configuration

---

### 5.6 Standard Operating Procedures (SOPs)

- **Status:** ⬜ Stub
- **Source:** `src/application/sops/mod.rs` (empty module)
- **Business Value:** 🟠 High
- **What It Does Today:** Empty module with doc comment: "Populated by later tasks."
- **What It Should Do:** SOP engine: reusable, parameterized workflows (e.g., "deploy to staging",
  "run security audit"). SOPs are governance-approved cascade templates. SOP library with
  versioning. **This is Maestro's equivalent of MetaGPT's core concept.**
- **Gap:** Full feature gap. This directly maps to Maestro's manifesto philosophy.
- **Competitor Benchmark:**
  - *MetaGPT*: **Core design principle** — "Code = SOP(Team)". Entire framework built around SOPs.
  - *Maestri / Maestro (competitor)*: Playbooks — repeatable workflows with history tracking

---

### 5.7 AI Safety Harness

- **Status:** ⬜ Stub
- **Source:** `src/infrastructure/harness/mod.rs` (empty module)
- **Business Value:** 🔴 Critical
- **What It Does Today:** Empty module: "sandbox, token limits, destructive-action safety auditing."
- **What It Should Do:** Sandbox execution (container/namespace isolation). Token limits per request
  and session. Destructive-action detection and blocking. Output content filtering. PII detection.
  Injection prevention. Rate limiting.
- **Gap:** Full feature gap. Essential for production trust.
- **Competitor Benchmark:**
  - *Claude Code*: Permission modes, sandboxed execution, background safety checks, auto-mode guards
  - *OpenCode*: Command approval with smart defaults
  - *Aider*: Change preview before applying, git-based safety net

---

### 5.8 Policy Engine

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟡 Medium (🔴 Critical for Enterprise)
- **What It Does Today:** Nothing.
- **What It Should Do:** Declarative policy language for governance rules. Policy enforcement at
  every FSM stage. Audit logging. Violation alerts. Per-organization customization.
- **Gap:** Full feature gap.
- **Competitor Benchmark:** No direct competitor has this — enterprise opportunity.

---

## Domain 6: TUI & User Experience

### 6.1 Three-Mode Workspace

- **Status:** ✅ Implemented
- **Source:** `frontend/src/workspace.nim`
- **Business Value:** 🟠 High — **Maestro differentiator**
- **What It Does Today:** Config · Maestro · Product tabs. F1/F2/F3/Tab switching. Footer with
  input line. `WorkspaceState` with unidirectional event folding.
- **What It Should Do:** Mode indicators (notification badges, activity dots). Mode-specific help
  overlay. Quick-switch with fuzzy search.
- **Gap:** No notifications or activity indicators.
- **Competitor Benchmark:**
  - *OpenCode*: Single-mode TUI (chat + file changes), but with rich syntax highlighting
  - *Maestri*: Infinite canvas with spatial layout (different paradigm entirely)
  - *Aider*: Single-mode chat interface

---

### 6.2 Config Mode Panel

- **Status:** ✅ Implemented
- **Source:** `frontend/src/panels/config.nim`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Left: governance navigator with selection marker and archived badge.
  Right: entry editor (read-only display). Keyboard navigation.
- **What It Should Do:** In-place editing. Syntax highlighting for YAML/Markdown. Create/archive
  via keyboard shortcuts. Search/filter. Schema validation preview.
- **Gap:** Read-only display — editing requires CLI commands.
- **Competitor Benchmark:** **No competitor has a config management TUI panel.** Unique.

---

### 6.3 Maestro Mode Panel

- **Status:** ✅ Implemented
- **Source:** `frontend/src/panels/maestro.nim`
- **Business Value:** 🟠 High
- **What It Does Today:** Left: personas list with cognitive state. Right: FSM stage, approval
  prompt (y/n), narration log.
- **What It Should Do:** Streaming output display. Progress bar for cascade. Agent activity
  sparklines. Expandable narration entries. Plan diff preview. Token counter.
- **Gap:** No streaming, no progress visualization, no expandable entries.
- **Competitor Benchmark:**
  - *OpenCode*: Real-time streaming with syntax-highlighted diffs
  - *Claude Code*: Real-time output with tool-use visualization
  - *Aider*: Real-time diff display with git integration

---

### 6.4 Product Mode Panel

- **Status:** ✅ Implemented
- **Source:** `frontend/src/panels/product.nim`
- **Business Value:** 🟡 Medium — **Maestro differentiator**
- **What It Does Today:** Left: release list with selection. Right: live demo output with
  running/idle status.
- **What It Should Do:** Release comparison (diff between versions). ANSI color passthrough.
  Interactive demo (stdin forwarding). Recording and replay. Artifact browser.
- **Gap:** No diff, no color passthrough, no artifact browsing.
- **Competitor Benchmark:** **No competitor has a product showcase mode.** Unique to Maestro.

---

### 6.5 IPC Protocol v2

- **Status:** ✅ Implemented
- **Source:** `src/presentation/ipc/mod.rs`, `frontend/src/protocol.nim`
- **Business Value:** 🟡 Medium
- **What It Does Today:** 14 core→TUI event kinds. 13 TUI→core command kinds. 3 modes. Strict
  version + kind validation. Line-delimited JSON on stdio.
- **What It Should Do:** Streaming event kind for real-time LLM output. Binary payload for file
  transfers. Protocol compression. Bidirectional heartbeat.
- **Gap:** No streaming event type. No binary payloads.
- **Competitor Benchmark:** Internal architecture — no direct competitor comparison.

---

### 6.6 Accessibility

- **Status:** ✅ Implemented
- **Source:** `frontend/src/theme.nim`
- **Business Value:** 🟢 Low
- **What It Does Today:** `MAESTRO_ASCII_ONLY` env var: drops Unicode borders, uses `[Title]`
  text headers.
- **What It Should Do:** WCAG compliance. Screen reader support. High-contrast theme. Configurable
  color schemes. Font size awareness. Full keyboard-only navigation.
- **Gap:** ASCII-only toggle only. No screen reader support.
- **Competitor Benchmark:**
  - *OpenCode*: Configurable themes
  - *Claude Code*: Theme selection

---

### 6.7 Keyboard Navigation

- **Status:** ✅ Implemented
- **Source:** `frontend/src/app.nim`
- **Business Value:** 🟢 Low
- **What It Does Today:** Full keyboard handling: F1-F3 mode switch, Up/Down navigation, text
  input, `/` commands, y/n approval, Esc/Ctrl+Q quit, Tab cycling.
- **What It Should Do:** Vim-style bindings (j/k, `:` command mode). Customizable key maps.
  Keyboard cheatsheet (F10 or `?`).
- **Gap:** Fixed keybindings. No vim mode. No help overlay.
- **Competitor Benchmark:**
  - *OpenCode*: Customizable keybindings
  - *Aider*: Readline-compatible input with history

---

### 6.8 Theming

- **Status:** 🚧 Partial
- **Source:** `frontend/src/theme.nim`
- **Business Value:** 🟢 Low
- **What It Does Today:** ASCII-only toggle. Hard-coded colors.
- **What It Should Do:** User-configurable themes (dark, light, solarized, catppuccin). Theme
  selection in Config Mode. Per-mode color accents.
- **Gap:** No theme support beyond ASCII toggle.
- **Competitor Benchmark:**
  - *OpenCode*: Multiple color themes
  - *Claude Code*: Theme configuration

---

### 6.9 Notifications

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing.
- **What It Should Do:** Terminal bell on approval request. Desktop notification for long-running
  task completion. Sound alerts (opt-in).
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: Terminal bell on completion
  - *Maestri*: Ombro AI summarizes what happened while you were away

---

### 6.10 Help System

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing — no in-app help.
- **What It Should Do:** Context-sensitive help overlay. Built-in tutorial/onboarding. Command
  palette (Ctrl+P). Searchable docs.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: `/help`, 40+ slash commands with descriptions
  - *OpenCode*: Built-in command help
  - *Aider*: `/help` with comprehensive command listing

---

## Domain 7: Delivery, Persistence & DevOps

### 7.1 Git-Standalone Release Persistence

- **Status:** ✅ Implemented
- **Source:** `src/application/persistence.rs`
- **Business Value:** 🟡 Medium — **Maestro differentiator**
- **What It Does Today:** Releases stored in `maestro/releases/rNNN/`. `manifest.md` + `demo.sh`.
  Best-effort `git init + commit`. Incrementing version numbering.
- **What It Should Do:** Meaningful commit messages. Branch-per-release strategy. Tagging. Release
  signing. Remote push (opt-in). Artifact checksums.
- **Gap:** Minimal git integration. No tags, no signing, no remote push.
- **Competitor Benchmark:**
  - *Aider*: Every change is an atomic git commit with descriptive messages
  - *Claude Code*: Git integration for change tracking

---

### 7.2 Release Listing

- **Status:** ✅ Implemented
- **Source:** `src/application/persistence.rs`, `frontend/src/panels/product.nim`
- **Business Value:** 🟢 Low
- **What It Does Today:** Reads manifest files from disk. Shows version + changelog in Product Mode.
- **What It Should Do:** Rich metadata (creation date, agent contributions, cost, duration).
  Filtering and search. Pagination.
- **Gap:** Basic version + changelog only.
- **Competitor Benchmark:** **No competitor has a release catalog.** Unique.

---

### 7.3 Demo Runner

- **Status:** ✅ Implemented
- **Source:** `src/application/demo_runner.rs`
- **Business Value:** 🟡 Medium — **Maestro differentiator**
- **What It Does Today:** Executes `demo.sh` in release dir. Streams stdout/stderr line-by-line.
  Reports exit code.
- **What It Should Do:** Multiple demo scripts. ANSI color passthrough. Interactive demo (stdin
  forwarding). Recording and replay. Timeout configuration.
- **Gap:** Single script only. No color passthrough.
- **Competitor Benchmark:** **No competitor has a built-in demo runner.** Unique.

---

### 7.4 Packaging — Debian

- **Status:** ✅ Implemented
- **Source:** `scripts/`, `docs/Practical_Guides/SMOKE_TEST_DEBIAN.md`
- **Business Value:** 🟢 Low
- **What It Does Today:** Build script produces `.deb`. Remove/purge lifecycle. Smoke test.
- **What It Should Do:** Automated release pipeline. APT repository. Auto-update checks.
- **Gap:** Manual build process.
- **Competitor Benchmark:**
  - *OpenCode*: `curl` install, Homebrew, npm, cargo
  - *Claude Code*: npm global install
  - *Aider*: pip install, Homebrew

---

### 7.5 Packaging — macOS

- **Status:** ✅ Implemented
- **Source:** `scripts/`, `docs/Practical_Guides/SMOKE_TEST_MACOS.md`
- **Business Value:** 🟢 Low
- **What It Does Today:** Build script produces `.pkg`. Smoke test.
- **What It Should Do:** Code signing and notarization. Homebrew formula. DMG installer.
- **Gap:** No signing, no Homebrew.
- **Competitor Benchmark:**
  - *OpenCode*: Homebrew tap
  - *Aider*: Homebrew formula

---

### 7.6 Packaging — Arch Linux

- **Status:** ✅ Implemented
- **Source:** `scripts/`, `docs/Practical_Guides/SMOKE_TEST_OMARCHY.md`
- **Business Value:** 🟢 Low
- **What It Does Today:** Build script produces `.pkg.tar.zst`. Smoke test via pacman.
- **What It Should Do:** AUR package. PKGBUILD generation.
- **Gap:** No AUR presence.
- **Competitor Benchmark:** Limited — most competitors don't target Arch specifically.

---

### 7.7 CI/CD Pipeline

- **Status:** ✅ Implemented
- **Source:** `.github/workflows/`
- **Business Value:** 🟡 Medium
- **What It Does Today:** GitHub Actions CI (Rust + Nim, multi-OS matrix). AI governance gate (PR
  body validation, linked plan task, AC ≥ 3). Release workflow (multi-platform binaries +
  checksums).
- **What It Should Do:** Integration test job. Nightly builds. Performance regression detection.
  Automatic changelog. Security scanning (`cargo-audit`). Coverage reporting.
- **Gap:** No integration tests in CI. No security scanning. No coverage.
- **Competitor Benchmark:**
  - *OpenCode*: Comprehensive CI with coverage and security scanning
  - *Aider*: CI with benchmark suite (SWE-bench)

---

### 7.8 Headless CI Integration

- **Status:** 🚧 Partial
- **Source:** `src/presentation/cli/mod.rs`
- **Business Value:** 🟠 High
- **What It Does Today:** `--no-tui` flag. Core runs headless. Output is human-readable tracing.
- **What It Should Do:** Structured JSON event stream. Exit codes for pass/fail. JUnit XML output.
  GitHub Actions marketplace action. GitLab CI integration.
- **Gap:** Not machine-parseable.
- **Competitor Benchmark:**
  - *Claude Code*: `--json`, `--print` for non-interactive CI use
  - *Aider*: `--message` for CI pipelines with exit codes

---

### 7.9 Plugin/Extension System

- **Status:** 📋 Planned
- **Source:** Not implemented
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing.
- **What It Should Do:** Plugin architecture for custom providers, tools, personas, and governance
  rules. Plugin manifest. Discovery and installation. Sandboxed execution.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: Plugin ecosystem with skills, commands, and MCP bundles
  - *OpenCode*: Custom skills and MCP configurations

---

### 7.10 RAG Pipeline (Codebase Grounding)

- **Status:** 📋 Planned
- **Source:** `docs/Maestro_Manifesto/reference/rag_eval_dataset.json` (eval questions only)
- **Business Value:** 🔴 Critical
- **What It Does Today:** 5-question eval dataset exists. No runtime RAG.
- **What It Should Do:** Codebase indexing (file chunking). Local embedding generation. Vector store
  (SQLite-vec or similar). Retrieval at instrumentation. Reranking. Citation tracking. Incremental
  index updates.
- **Gap:** Full feature gap. Agents have zero codebase context.
- **Competitor Benchmark:**
  - *Aider*: Repo-map with automatic context selection — gold standard for token efficiency
  - *Claude Code*: Full codebase indexing with relevant file injection
  - *OpenCode*: LSP-based code intelligence
  - *MetaGPT*: Context accumulation through SOP pipeline stages

---

## Enterprise Features

### E.1 Compliance Reporting

- **Status:** 📋 Planned (Enterprise Roadmap)
- **Source:** `docs/Maestro_Manifesto/FEATURE_LEVELS.md`
- **Business Value:** 🟡 Medium (🔴 for enterprise adoption)
- **What It Does Today:** Nothing.
- **What It Should Do:** Structured compliance reports (SOC2, ISO 27001 evidence). Audit trail
  export. Change provenance tracking (who approved what, when). Regulatory report generation.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: Audit logs for enterprise deployments
  - No open-source competitor targets compliance reporting

---

### E.2 Policy Extension Framework

- **Status:** 📋 Planned (Enterprise Roadmap)
- **Source:** `docs/Maestro_Manifesto/FEATURE_LEVELS.md`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing.
- **What It Should Do:** Custom policy rules per organization. Policy-as-code (OPA/Rego-style).
  Policy inheritance (org → team → project). Pre-built policy packs for common frameworks.
- **Gap:** Full feature gap.
- **Competitor Benchmark:** No open-source competitor targets this niche.

---

### E.3 Audit Analytics

- **Status:** 📋 Planned (Enterprise Roadmap)
- **Source:** `docs/Maestro_Manifesto/FEATURE_LEVELS.md`
- **Business Value:** 🟡 Medium
- **What It Does Today:** Nothing.
- **What It Should Do:** Analytics dashboard: agent activity trends, approval rates, rollback
  frequency, cost per micro-project, time-to-delivery metrics. Historical comparison.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Maestri*: On-device Ombro AI summarizes agent activity
  - No competitor has structured audit analytics

---

### E.4 Multi-Tenant Governance

- **Status:** 📋 Planned (Enterprise Roadmap)
- **Source:** Not documented
- **Business Value:** 🟢 Low (future enterprise need)
- **What It Does Today:** Nothing — single-user, single-project only.
- **What It Should Do:** Team governance (shared personas, skills, policies). Role-based access
  control. Centralized governance server. Team activity visibility.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: Enterprise admin controls
  - *Maestri / Maestro*: Multi-session management

---

### E.5 Cost Governance

- **Status:** 📋 Planned (Enterprise Roadmap)
- **Source:** Not documented
- **Business Value:** 🟠 High (for enterprise adoption)
- **What It Does Today:** Nothing.
- **What It Should Do:** Per-project and per-team token budgets. Cost alerts and hard limits.
  Provider cost comparison. Usage dashboards. Budget approval workflows.
- **Gap:** Full feature gap.
- **Competitor Benchmark:**
  - *Claude Code*: `/cost` per-session tracking
  - *OpenCode*: Token cost display per session

---

## Competitive Positioning

### Unique Differentiators — Where Maestro Leads

These features are unique to Maestro or significantly stronger than any competitor. **Double down
on these.**

| # | Differentiator | Status | Why It Matters |
|---|----------------|--------|---------------|
| 1 | **Local-first / Offline by default** | ✅ | Only tool with full offline operation + governance. Ollama default, no API key required. |
| 2 | **Governed execution (approval gates + rollback)** | ✅ | No competitor has mandatory rollback-as-a-service. Approval gates are more structured than any competitor. |
| 3 | **Spec-driven development** | ✅ | Enforced development methodology — task specs → failing tests → implementation. Unique in agentic space. |
| 4 | **Deterministic Two-Towers routing** | ✅ | Auditable, testable persona selection. CrewAI/AutoGen use ad-hoc delegation. |
| 5 | **Six-stage typed FSM** | ✅ | Explicit, tested state machine. Most competitors have implicit execution stages. |
| 6 | **Governance CRUD** | ✅ | Full lifecycle management of personas, skills, scopes. No competitor has this. |
| 7 | **Three-mode workspace** | ✅ | Config · Maestro · Product separation. No competitor has a product showcase mode. |
| 8 | **Hexagonal architecture (plugin-ready)** | ✅ | Clean domain/port/adapter separation enables future plugin ecosystem. |
| 9 | **Two-process architecture** | ✅ | Rust core + Nim TUI enables both headless CI and rich terminal UX. |
| 10 | **Demo runner** | ✅ | Built-in product demo execution. No competitor has this. |

### Key Gaps — Where Maestro Must Catch Up

These features are **table-stakes** in 2026. Without them, Maestro cannot compete.

| # | Gap | Business Value | Impact | Who Has It |
|---|-----|---------------|--------|------------|
| 1 | **Agent tool use (file edit, shell, MCP)** | 🔴 Critical | Agents can only generate text — can't interact with the environment | OpenCode, Claude Code, Aider, MetaGPT |
| 2 | **Streaming responses** | 🔴 Critical | Users wait for full completion — feels broken in 2026 | ALL competitors |
| 3 | **Codebase RAG / context injection** | 🔴 Critical | Agents have zero project context beyond the demand string | Aider (repo-map), Claude Code, OpenCode |
| 4 | **Agent memory (cross-session)** | 🔴 Critical | Every session starts from zero | Claude Code (CLAUDE.md), OpenCode (SQLite), Maestri |
| 5 | **AI safety harness** | 🔴 Critical | No sandbox, no token limits, no destructive-action guard | Claude Code (permission modes), OpenCode |
| 6 | **Verification (real test/lint execution)** | 🔴 Critical | Verification stage is a pass-through | Aider (auto-test), Claude Code, OpenCode |
| 7 | **Instrumentation (rich prompts)** | 🔴 Critical | Prompts contain only name + demand — nearly empty | Claude Code, Aider, MetaGPT |
| 8 | **Anthropic adapter** | 🟠 High | Missing access to top-tier coding models | OpenCode, Aider |
| 9 | **SOP engine** | 🟠 High | Stub exists but empty — this is Maestro's MetaGPT equivalent | MetaGPT (core design) |
| 10 | **Inter-agent communication** | 🟠 High | Agents can't collaborate during execution | MetaGPT, Maestri, Claude Code |

### Competitive Benchmark Matrix

Feature coverage comparison across competitors (● = full, ◐ = partial, ○ = missing):

| Feature | Maestro | OpenCode | Aider | MetaGPT | Claude Code | Maestri |
|---------|---------|----------|-------|---------|-------------|---------|
| Local-first / Ollama | ● | ● | ● | ◐ | ○ | ○ |
| Multi-provider support | ◐ | ● | ● | ● | ○ | ○ |
| Streaming responses | ○ | ● | ● | ● | ● | ● |
| Tool use (files, shell) | ○ | ● | ● | ● | ● | ◐ |
| MCP support | ○ | ● | ○ | ○ | ● | ○ |
| Codebase context (RAG) | ○ | ● | ● | ◐ | ● | ○ |
| Multi-agent runtime | ● | ○ | ○ | ● | ◐ | ● |
| Agent memory | ○ | ● | ◐ | ◐ | ● | ● |
| Approval gates | ● | ◐ | ◐ | ○ | ◐ | ○ |
| Rollback plans | ● | ○ | ● | ○ | ◐ | ○ |
| Governance CRUD | ● | ○ | ○ | ○ | ○ | ○ |
| SOPs / Playbooks | ○ | ○ | ○ | ● | ○ | ◐ |
| FSM state machine | ● | ○ | ○ | ◐ | ○ | ○ |
| TUI interface | ● | ● | ◐ | ○ | ○ | ● |
| Visual orchestration | ○ | ○ | ○ | ○ | ○ | ● |
| Git integration | ◐ | ◐ | ● | ○ | ◐ | ○ |
| Headless / CI mode | ◐ | ● | ● | ● | ● | ○ |
| Safety harness | ○ | ◐ | ◐ | ○ | ● | ○ |
| Token/cost tracking | ○ | ● | ● | ○ | ● | ○ |
| Persona routing | ● | ○ | ○ | ● | ○ | ◐ |
| Product demo runner | ● | ○ | ○ | ○ | ○ | ○ |

---

## Prioritized Feature Roadmap (by Business Value)

### 🔴 Critical — Ship Next (Competitive Viability)

| Priority | Feature | Domain | Current Status |
|----------|---------|--------|----------------|
| C-1 | Agent tool use (file, shell, MCP) | 2 — Runtime | 📋 Planned |
| C-2 | Streaming responses | 4 — Providers | 📋 Planned |
| C-3 | Codebase RAG pipeline | 7 — Delivery | 📋 Planned |
| C-4 | Agent memory | 2 — Runtime | 📋 Planned |
| C-5 | AI safety harness | 5 — Governance | ⬜ Stub |
| C-6 | Verification (real execution) | 3 — Orchestration | 🚧 Partial |
| C-7 | Instrumentation (rich prompts) | 3 — Orchestration | 🚧 Partial |
| C-8 | Cognitive cycle (real think()) | 2 — Runtime | ✅ (no-op think) |

### 🟠 High — Ship Soon (Strong Differentiation)

| Priority | Feature | Domain | Current Status |
|----------|---------|--------|----------------|
| H-1 | SOPs engine | 5 — Governance | ⬜ Stub |
| H-2 | Anthropic adapter | 4 — Providers | 📋 Planned |
| H-3 | Inter-agent communication | 2 — Runtime | 📋 Planned |
| H-4 | Embedding generation | 4 — Providers | 📋 Planned |
| H-5 | Two-Towers semantic upgrade | 2 — Runtime | ✅ (lexical only) |
| H-6 | Token/cost management | 4 — Providers | 📋 Planned |
| H-7 | Headless CI integration (JSON) | 7 — Delivery | 🚧 Partial |
| H-8 | Per-agent model cascading | 2 — Runtime | ✅ (static only) |
| H-9 | Executable rollback inverses | 3 — Orchestration | ✅ (symbolic) |
| H-10 | MCP protocol | 4 — Providers | 📋 Planned |
| H-11 | Cost governance (enterprise) | Enterprise | 📋 Planned |

### 🟡 Medium — Planned (Polish & Adoption)

| Priority | Feature | Domain | Current Status |
|----------|---------|--------|----------------|
| M-1 | Demand decomposition | 3 — Orchestration | 📋 Planned |
| M-2 | Session checkpointing | 3 — Orchestration | 📋 Planned |
| M-3 | Project templates | 1 — CLI | 📋 Planned |
| M-4 | Help system | 6 — TUI | 📋 Planned |
| M-5 | Notifications | 6 — TUI | 📋 Planned |
| M-6 | Plugin/extension system | 7 — Delivery | 📋 Planned |
| M-7 | Policy engine | 5 — Governance | 📋 Planned |
| M-8 | Doctor improvements | 1 — CLI | ✅ (basic) |
| M-9 | Config in-place editing | 6 — TUI | ✅ (read-only) |
| M-10 | Compliance reporting | Enterprise | 📋 Planned |

### 🟢 Low — Future (Adequate or Niche)

| Priority | Feature | Domain | Current Status |
|----------|---------|--------|----------------|
| L-1 | Init-config improvements | 1 — CLI | ✅ |
| L-2 | Validate-config auto-fix | 1 — CLI | ✅ |
| L-3 | List-agents enrichment | 1 — CLI | ✅ |
| L-4 | Version metadata | 1 — CLI | ✅ |
| L-5 | Scaffold-markdown templates | 1 — CLI | ✅ |
| L-6 | Accessibility (WCAG) | 6 — TUI | ✅ (ASCII toggle) |
| L-7 | Keyboard customization | 6 — TUI | ✅ |
| L-8 | Theming | 6 — TUI | 🚧 Partial |
| L-9 | Packaging improvements | 7 — Delivery | ✅ |
| L-10 | Multi-tenant governance | Enterprise | 📋 Planned |
| L-11 | Audit analytics | Enterprise | 📋 Planned |
| L-12 | Policy extension | Enterprise | 📋 Planned |

---

## Source File Index

### Rust Core (`src/`)

| Module | File | Features |
|--------|------|----------|
| **domain/models** | `agent_id.rs` | AgentId newtype |
| | `message.rs` | Message, MessageRole |
| | `config.rs` | SystemConfig, MaestroConfig |
| | `fsm.rs` | FsmStage, MicroProject |
| | `governance.rs` | GovernanceEntry, GovernanceKind |
| | `persona.rs` | Persona, default_personas |
| | `rollback.rs` | CascadeStep, RollbackPlan |
| | `routing.rs` | Two-Towers scorer, PersonaMatch |
| **domain/ports** | `llm_provider.rs` | LlmProvider trait |
| | `role.rs` | Role trait (cognitive cycle) |
| **application** | `agent_runtime.rs` | AgentRuntime, JoinSet |
| | `agent_observability.rs` | RuntimeEvent |
| | `orchestrator.rs` | Session, orchestrate() |
| | `governance.rs` | Governance CRUD |
| | `persona_agent.rs` | PersonaAgent (Role impl) |
| | `model_router.rs` | Per-agent model resolution |
| | `persistence.rs` | Git-standalone releases |
| | `demo_runner.rs` | Demo script execution |
| | `wizard.rs` | WizardForm |
| | `readiness.rs` | Provider probe (SENSE) |
| | `sops/mod.rs` | ⬜ STUB |
| | `error.rs` | RuntimeError |
| **infrastructure** | `config.rs` | Config loader (YAML) |
| | `bus/broadcast_bus.rs` | BroadcastBus |
| | `llm/ollama.rs` | Ollama adapter |
| | `llm/openai.rs` | OpenAI adapter |
| | `llm/gemini.rs` | Gemini adapter |
| | `llm/registry.rs` | ProviderRegistry |
| | `harness/mod.rs` | ⬜ STUB |
| **presentation** | `cli/mod.rs` | CLI surface, commands |
| | `ipc/mod.rs` | IPC protocol v2 |
| | `ipc/server.rs` | Duplex IPC server |

### Nim Frontend (`frontend/`)

| File | Features |
|------|----------|
| `src/maestro_tui.nim` | Entry point |
| `src/app.nim` | Tick loop, keyboard handler, core process management |
| `src/workspace.nim` | WorkspaceState, event folding, render |
| `src/protocol.nim` | IPC protocol v2 (Nim side) |
| `src/theme.nim` | ASCII-only accessibility |
| `src/panels/config.nim` | Config Mode panel |
| `src/panels/maestro.nim` | Maestro Mode panel |
| `src/panels/product.nim` | Product Mode panel |
| `tests/test_protocol.nim` | 8 protocol tests |
| `tests/test_workspace.nim` | 10 workspace tests |

---

*This is a living document. Update it when features are added, modified, or retired.*
*Last generated: 2026-07-09 by automated analysis of the full codebase.*
