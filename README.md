# ⚡ MAESTRO HARNESS FOR AI

**You are the orchestrator. This harness executes your vision.**

Maestro is a **relentless AI command deck**: a headless **Rust core** for blazing speed and rock-solid safety, paired with a separate **Nim/Tatui TUI** for a flicker-free interactive command deck. Instead of coordinating human developers, you coordinate, test, and manage a **team of AI agents** to architect and build software on your command.

Fire up the TUI. Define your personas, scopes, and skills. Watch your AI team synthesize, execute, and iterate. No memorized commands. No friction. Just pure orchestration.

> 🚧 **PRE-RELEASE / ACTIVE DEVELOPMENT**
>
> Maestro **0.3.0** ships the **three-mode Workspace** (Config · Maestro · Product) end-to-end, but remains **pre-release**. The orchestration cascade produces deterministic placeholder deliverables (real LLM-driven work through the provider registry is the next milestone). This is **not production-ready software**. See [Feature Status](#-feature-status) below. Expect breaking changes.

<p align="center">
  <img src="https://raw.githubusercontent.com/invector-tecnologia/maestro-ai-harness/main/docs/assets/dream-tui.png" alt="Maestro Dream TUI" width="800">
</p>

**▓▒░ SYNTH PROFILE ░▒▓**

## Contents

* [Core Capabilities](#-core-capabilities)
* [Feature Status](#-feature-status)
* [Boot Sequence](#-boot-sequence)
* [Control Deck Initialization](#-control-deck-initialization)
* [Command Execution](#-command-execution)
* [Governance Override](#-governance-override)
* [Reference Grid](#-reference-grid)

## 🌟 Core Capabilities

Maestro's brain is **Rust-native**. Fast. Uncompromising. Its interactive **TUI** (Terminal User Interface) is a separate Nim process that consumes [Tatui](https://github.com/invector-tecnologia/tatui) — an immediate-mode, ratatui-inspired Nim TUI library — delivering menus, tables, real-time logs, and keyboard shortcuts over a line-delimited JSON protocol, all without terminal bloat or command memorization. The core stays fully usable headless with `--no-tui`.

### ⚡ What You Control
* **🚧 Multi-Provider AI Synthesis:** Run models locally and free with **Ollama** (no API key) — the reference adapter is **operational** and resolved through the provider registry. An **OpenAI-compatible** adapter is also operational (key via `OPENAI_API_KEY`, never config files); native **Anthropic** and **Google Gemini** adapters are planned. Configuration supports **per-agent model routing** (each persona can pin a provider + model); unlisted personas fall back to `system.default_provider`/`default_model`, and Maestro **fails fast** on startup if a referenced pair is undeclared.
* **🚧 Governance Codex (Config Mode):** Define *Personas* (AI profiles), *Scopes* (execution domains), *Skills* (tool capabilities). **Config Mode** is the single governance surface — view, create, edit, update, and **archive** both defaults and customs; the Maestro orchestrator persona is immutable. Deeper skill schemas and compliance enforcement are in development.
* **📋 Secure Credentials (Planned):** OAuth2 browser login to Google Gemini planned. Basic local config auth operational; keychain integration roadmap v0.2+.
* **✅ Agent Observability (Operational):** Structured `tracing` narration of the `observe → think → act` cognitive cycle is implemented. Cost tracking and full audit logs planned for v0.2+.
* **✅ Inter-Agent Collaboration (Operational):** Agents can communicate directly via directed messaging (`[SEND_TO Persona]`) and share state through a globally injected key-value Scratchpad, enabling complex multi-agent workflows.

### ⚡ Dependency Matrix (Planned v0.2+)
Maestro will partition the dependency graph into **two isolation zones** (tracked for v0.2+):

**Zone 1: Harness Domain** — Maestro runtime readiness. LLM provider config, model catalog, connection health.

**Zone 2: Project Domain** — Your repo's AI companion. Toolchain checks, command availability, framework validation (defined in `maestro/project-deps.yml`).

Validate each zone independently (roadmap):

```bash
maestro deps check --scope harness      # Check Maestro runtime only
maestro deps check --scope project      # Check project toolchain only
maestro deps check --scope all          # Full validation
```

---

## ⚡ FEATURE STATUS

Maestro's capabilities are organized by maturity level. **Current release: 0.3.0 (three-mode Workspace)**

| Level | Status | Examples |
|-------|--------|----------|
| **Foundational** | ✅ Complete | `maestro init-config`, `maestro validate-config`, `maestro doctor` readiness checks, `maestro scaffold-markdown` |
| **Core** | 🚧 Partial | Multi-agent runtime (`observe→think→act`, failure isolation), inter-agent communication (directed messaging, shared scratchpad), three-mode Nim/Tatui Workspace TUI (Config · Maestro · Product) over a stdio JSON protocol v2, Ollama provider adapter + registry, default persona catalog |
| **Advanced** | 📋 Planned | Governed micro-project execution (FSM · cascade · rollback · git persistence), accessibility controls, cross-platform packaging |
| **Enterprise** | 📋 Roadmap | Compliance reporting, policy extension, audit analytics |

**See [`docs/Maestro_Manifesto/FEATURE_LEVELS.md`](docs/Maestro_Manifesto/FEATURE_LEVELS.md) for detailed capability breakdown.**

---

## ⚡ BOOT SEQUENCE

**⚠️ FOR DEVELOPMENT & TESTING ONLY** — This is pre-release software. Suitable for local testing and development workflows. Do not deploy to production environments.

> **0.1.0 reality check:** Only the Debian build script (`./scripts/build-deb.sh`) exists today. The install one-liner and the macOS/Arch packaging scripts below are on the roadmap. For now, build from source: `cargo build --release` for the core, and `./scripts/install-tatui.sh` + `nimble build` in `frontend/` for the Nim/Tatui TUI.

**Open your terminal.** On macOS and Linux: search for "Terminal". On Windows: open "PowerShell" or "Command Prompt".

### 🪄 AUTO-DEPLOY (macOS & Linux)
Run this one-liner to synthesize and install:

**Copy and paste this command, then press `Enter`:**
```bash
curl -sSL https://raw.githubusercontent.com/invector-tecnologia/maestro-ai-harness/main/scripts/install.sh | bash
```
*Note: You may need to enter your system password to install. Characters won't show as you type—this is normal. Just type and press Enter. The installer downloads a pre-compiled release binary when one is available for your platform, and automatically falls back to building from source (requires the Rust toolchain) otherwise.*

---

### 🔧 MANUAL OVERRIDE (Build from Source)
If auto-deploy fails, or you prefer direct control, follow your OS track:

#### 🍎 MACOS
Generate the native `.pkg` installer from source:
1. Open the terminal in the project folder.
2. Generate the package: `./scripts/build-macos-pkg.sh 0.1.0`
3. Install by double-clicking the generated file or run in terminal:
```bash
sudo installer -pkg target/macos/build/maestro-ai-0.1.0-macos-$(uname -m).pkg -target /
```

#### 🐧 DEBIAN / UBUNTU / LINUX MINT
1. Navigate to project folder.
2. Build: `./scripts/build-deb.sh 0.1.0`
3. Deploy:
```bash
sudo dpkg -i target/deb/maestro-ai_0.1.0_amd64/maestro-ai.deb
```

#### 🎩 ARCH LINUX / OMARCHY
1. Navigate to project folder.
2. Build: `./scripts/build-omarchy-pkg.sh 0.1.0`
3. Deploy:
```bash
sudo pacman -U --noconfirm target/omarchy/build/maestro-ai-0.1.0-1-$(uname -m).pkg.tar.zst
```

> **⚡ Validation Override:** Run the smoke test to verify installation integrity:
> `./scripts/smoke-test-omarchy.sh target/omarchy/build/maestro-ai-0.1.0-1-$(uname -m).pkg.tar.zst`

---

## ⚡ CONTROL DECK INITIALIZATION

All governance, TUI state, and project configurations live inside the `maestro/` folder in your project root. Maestro reads `./maestro/config.yml` first; if not found, it scans the global system config path. A legacy `config.yaml` is still accepted with a deprecation warning.

**This is your control deck schema.** Define providers, models, concurrency limits, rate limits, retry logic. Example: orchestrating Ollama locally:

```yaml
system:
  default_provider: "ollama"
  default_model: "mistral"
  max_concurrency: 4
  rate_limit_per_minute: 120
  retry_max_attempts: 3

providers:
  ollama:
    kind: "ollama"
    endpoint: "http://127.0.0.1:11434/v1"
    auth_mode: "none"
    timeout_ms: 60000
    models:
      - name: "mistral"
        context_window: 32000
    capabilities:
      supports_tools: false
      supports_streaming: true
      supports_json_mode: false
      supports_reasoning_controls: false
      max_context_tokens: 32000
```

**Auth Override:** For Bearer token authentication, adjust `auth_mode` to `"bearer"` and export the token as an environment variable before launching Maestro.

**Per-Agent Models:** Every model declared under a provider is available to any agent. Assign a specific provider + model to individual personas with the optional top-level `agents:` map. Agents that are not listed fall back to `system.default_provider` + `system.default_model`. The pair must exist in `providers:` or Maestro fails fast on startup.

```yaml
agents:
  "Maestro":
    provider: "openai"
    model: "gpt-4-turbo"
  "Software Engineer":
    provider: "anthropic"
    model: "claude-3-opus"
  "Quality Assurance":
    provider: "ollama"
    model: "mistral"
```

> The keys above are persona names (the default catalog ships `Maestro`, `Project Manager`, `Quality Assurance`, `User Experience`, `Software Engineer`). Every provider and model referenced under `agents:` must also be declared under `providers:`. See [`maestro/config.yml.example`](maestro/config.yml.example) for a complete multi-provider catalog.

---

## ⚡ COMMAND EXECUTION

**Execution Protocol.** The commands shipped in **0.1.0**:

1. **VALIDATE** — `maestro validate-config` checks configuration integrity and cross-references.
2. **SCAFFOLD** — `maestro scaffold-markdown` creates the mandatory `scopes`/`personas`/`skills` governance folders; `maestro init-config` writes a starter `maestro/config.yml`.
3. **INSPECT** — `maestro list-agents` catalogs the registered personas; `maestro doctor` runs readiness checks (configuration + governance scaffold).
4. **HEADLESS** — every command accepts the global `--no-tui` flag for CI/automation.

> **Interactive Workspace (0.3.0):** `maestro init` (plain-CLI bootstrap), `maestro tui`, and `maestro run` — the three-mode Nim/Tatui Workspace (Config · Maestro · Product) over the stdio protocol v2. Interview/onboarding mode was removed; see [ADR 0002](docs/adr/0002-three-mode-workspace-and-interview-removal.md).

### ⚡ Utility Commands (0.1.0)

* **`maestro version`** — Print version information.
* **`maestro validate-config`** — Validate `maestro/config.yml` and its cross-references.
* **`maestro list-agents`** — Catalog the registered personas.
* **`maestro doctor`** — Readiness scan (configuration + governance scaffold).
* **`maestro scaffold-markdown`** — Generate the governance Markdown folders.
* **`maestro init-config`** — Generate a starter `maestro/config.yml`.

### 🐞 DEBUG OVERRIDE

Control tracing verbosity via the `RUST_LOG` environment variable (defaults to `info`):

```bash
RUST_LOG=debug maestro doctor
```

All logging flows through `tracing`; `println!`/`eprintln!` are forbidden in the core.

---

## ⚡ GOVERNANCE OVERRIDE

Every release passes through a **Quality Gate**. Validate locally:

```bash
./scripts/quality-gate.sh              # Run full quality validation (Rust core + Nim TUI)
```

### ⚡ PR Governance Protocol

This repository enforces **CI-gated governance** through `.github/workflows/ai-governance-gate.yml`.

**Required PR Structure:**
1. `## Linked Plan Task` — exactly one line:
  - `- Path: docs/Maestro_Execution_Plans/tasks/<task>.md`
2. `## Acceptance Criteria` — IDs like `AC1`, `AC2`, `AC3`.
3. `## Validation Evidence` — one evidence line per AC ID.

**Acceptance Criteria Floor:** Configurable via repository variable `MIN_ACCEPTANCE_ITEMS`. Defaults to `3` if not set.

**Configure in GitHub:**
1. Repo Settings → `Secrets and variables` → `Actions` → `Variables`
2. Create `MIN_ACCEPTANCE_ITEMS` with numeric value (e.g., `4`)

**License:** GPL-3.0

---

## ⚡ REFERENCE GRID

The `docs/` folder is your knowledge base, organized by execution domain:

* **`docs/Maestro_Execution_Plans/`** — Roadmap: execution plans, release candidates, milestone specs.
* **`docs/Practical_Guides/`** — Tutorials: onboarding, smoke tests, adoption playbooks.
* **`docs/User_Manual/`** — Runtime reference: commands, panels, day-to-day operations.
* **`docs/Maestro_Manifesto/`** — Architecture truth: design philosophy, conventions, feature matrix, value streams.

**Project meta:** [Contributing Guide](CONTRIBUTING.md) · [Security Policy](.github/SECURITY.md) · [License (GPL-3.0)](LICENSE)
