# 063. Doctor System Health Improvements

## Objective
Enhance `maestro doctor` to become a comprehensive health report card by adding system-level checks (Git, Nim, GPU) and active provider connectivity probes.

## Implementation Steps
1. **Infrastructure (`src/infrastructure/system.rs`)**
   - Create a new module to handle non-blocking environment probing using `std::process::Command`.
   - Implement `check_system()` returning a `SystemHealth` struct (checks `git --version` and `nim --version`).
   - Implement `check_gpu()` using heuristics for `nvidia-smi` and macOS Apple Silicon (`sysctl`).
   - Export `system` in `src/infrastructure/mod.rs`.

2. **Presentation (`src/presentation/cli/mod.rs`)**
   - Revamp `doctor(root: &Path)` output into a structured report card with headers.
   - Run system checks, print toolchain availability and GPU presence.
   - Validate config and governance folder (existing, but reformatted).
   - If config is valid, loop through configured providers and run active `probe_provider` connection checks using Tokio.

## Acceptance Criteria
- [ ] `maestro doctor` prints a formatted report card with a header.
- [ ] Command checks for `git` and `nim` in PATH without panicking.
- [ ] Command checks for GPU presence (gracefully falling back).
- [ ] Command performs active provider network probes for all valid configurations.
- [ ] No `unwrap` in normal paths.
