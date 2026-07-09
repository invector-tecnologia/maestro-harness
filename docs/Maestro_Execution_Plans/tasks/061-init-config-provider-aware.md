# 061. Init Config Provider-Aware Setup

## Objective
Enhance `maestro init-config` to be provider-aware (Ollama, OpenAI, Gemini), auto-detect local providers and environment variables, and perform a connection probe upon setup.

## Current State
- `maestro init-config` writes a hardcoded static template for Ollama.
- No auto-detection, no connection testing.

## Implementation Steps
1. **Presentation / CLI (`src/presentation/cli/mod.rs`)**
   - Add optional flags to `InitConfig`: `--provider`, `--endpoint`, `--model`.
   - Update `dispatch()` to route `InitConfig` to a new `providers::init_config_with_provider`.

2. **Presentation / Providers (`src/presentation/cli/providers.rs`)**
   - Create `ProviderPreset` enum (Ollama, OpenAi, Gemini) with defaults.
   - Create `detect_providers()` to auto-discover Ollama (TCP connect to 11434) and API keys.
   - Create `config_for_provider()` to generate the YAML content dynamically.
   - Create `init_config_with_provider()` to write the config and run an async connection probe via `tokio::runtime::Runtime::new()`.

3. **Domain & Infrastructure (No new capabilities needed)**
   - Reuse `application::readiness::probe_provider` for connection tests.
   - Reuse `infrastructure::llm::registry::ProviderRegistry` to build the provider for probing.

## Acceptance Criteria
- [ ] `maestro init-config --provider openai` sets `default_provider: openai`.
- [ ] `maestro init-config` (no flags) auto-detects reachable Ollama or cloud keys.
- [ ] Runs a connection probe and prints the result (`provider is reachable ✓` or error).
- [ ] `--endpoint` and `--model` override defaults.
- [ ] Protects against overwriting an existing config.
- [ ] FSM boundaries respected, no unwrap in domain logic.
