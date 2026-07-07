//! Duplex IPC server — the headless core loop behind `maestro run` (TASK 053).
//!
//! Reads [`TuiCommand`] frames from an input stream and writes [`CoreEvent`]
//! frames to an output stream, both protocol v2 (ADR 0003). Config Mode commands
//! are served by the governance service against the project `root`. The core is
//! rendering-agnostic; no TUI types cross the boundary.

use std::io::{BufRead, Write};
use std::path::Path;

use crate::application::demo_runner;
use crate::application::governance as gov;
use crate::application::model_router::model_for;
use crate::application::orchestrator::{placeholder_deliverable, Session, Signal};
use crate::application::persistence;
use crate::domain::models::governance::{default_persona_ids, kind_of, GovernanceEntry};
use crate::domain::models::Message;
use crate::domain::ports::{CompletionRequest, LlmProvider, ProviderStatus};
use crate::infrastructure::llm::ProviderRegistry;

use super::{decode, encode, ConfigNode, CoreEvent, Mode, ReleaseSummary, TuiCommand};

/// Serialize and write a single event, flushing so the TUI sees it promptly.
fn emit(out: &mut impl Write, event: &CoreEvent) -> std::io::Result<()> {
    let line = encode(event)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    out.write_all(line.as_bytes())?;
    out.flush()
}

/// Map a governance entry to its IPC node.
fn to_node(entry: &GovernanceEntry) -> ConfigNode {
    ConfigNode {
        id: entry.id.clone(),
        kind: entry.kind.as_str().to_string(),
        origin: entry.origin.as_str().to_string(),
        archived: entry.archived,
        label: entry.id.clone(),
    }
}

/// The origin label for an id (built-in defaults vs custom).
fn origin_for(id: &str) -> &'static str {
    if id == "config.yml" || default_persona_ids().iter().any(|d| d == id) {
        "default"
    } else {
        "custom"
    }
}

/// Emit the current governance tree.
fn emit_tree(root: &Path, out: &mut impl Write) -> std::io::Result<()> {
    let entries = gov::list(root).unwrap_or_default();
    let nodes = entries.iter().map(to_node).collect();
    emit(out, &CoreEvent::ConfigTree { entries: nodes })
}

/// An in-progress, gated orchestration awaiting a user approval.
struct PendingRun {
    session: Session,
    prev_stage: String,
}

/// Map orchestration signals to IPC events and emit them, tracking FSM transitions.
fn map_and_emit(
    prev: &mut String,
    signals: Vec<Signal>,
    out: &mut impl Write,
) -> std::io::Result<()> {
    for signal in signals {
        let event = match signal {
            Signal::Stage(stage) => {
                let event = CoreEvent::FsmTransition {
                    from: prev.clone(),
                    to: stage.as_str().to_string(),
                };
                *prev = stage.as_str().to_string();
                event
            }
            Signal::Plan(steps) => CoreEvent::PlanProposed { steps },
            Signal::Agent { persona, state } => CoreEvent::AgentState {
                agent: persona,
                state,
            },
            Signal::Delegation { persona, task } => CoreEvent::Delegation { persona, task },
            Signal::Deliverable { persona, summary } => CoreEvent::Deliverable {
                persona,
                summary,
                artifact: None,
            },
            Signal::ApprovalRequest { id, prompt } => CoreEvent::ApprovalRequest { id, prompt },
            Signal::Rollback { action } => CoreEvent::Log {
                level: "warn".to_string(),
                message: format!("rollback: {action}"),
            },
            Signal::Log { level, message } => CoreEvent::Log { level, message },
        };
        emit(out, &event)?;
    }
    Ok(())
}

/// Begin a gated orchestration for `demand`; returns the pending run if it blocks.
fn begin_run(
    root: &Path,
    demand: &str,
    out: &mut impl Write,
) -> std::io::Result<Option<PendingRun>> {
    emit(
        out,
        &CoreEvent::Log {
            level: "info".to_string(),
            message: format!("demand received: {demand}"),
        },
    )?;
    let config = crate::infrastructure::config::load_from(root).ok();
    let resolve = |persona: &str| -> String {
        config
            .as_ref()
            .map(|c| model_for(c, persona))
            .unwrap_or_else(|| "default".to_string())
    };
    let (session, signals) = Session::start(demand, &gov::load_personas(root), resolve);
    let mut prev_stage = "start".to_string();
    map_and_emit(&mut prev_stage, signals, out)?;
    Ok(if session.is_pending() {
        Some(PendingRun {
            session,
            prev_stage,
        })
    } else {
        None
    })
}

/// Build a completer (runtime + provider) only when the default provider is
/// reachable and serving. `None` means "fall back to deterministic deliverables".
fn build_completer(
    root: &Path,
) -> Option<(tokio::runtime::Runtime, std::sync::Arc<dyn LlmProvider>)> {
    let config = crate::infrastructure::config::load_from(root).ok()?;
    let registry = ProviderRegistry::from_config(&config).ok()?;
    let provider = registry.default_provider(&config)?;
    let runtime = tokio::runtime::Runtime::new().ok()?;
    if runtime.block_on(provider.probe()) != ProviderStatus::Available {
        return None;
    }
    Some((runtime, provider))
}

/// A deliverer that calls the provider for each persona when a model is available,
/// and falls back to the deterministic placeholder otherwise.
fn build_deliverer(root: &Path) -> impl Fn(&str, &str, &str) -> String {
    let completer = build_completer(root);
    move |persona: &str, model: &str, demand: &str| -> String {
        if let Some((runtime, provider)) = &completer {
            let mut messages = Vec::new();
            if let Ok(system) = Message::system(format!(
                "You are the {persona} persona on a software team. Deliver your concise contribution to the task."
            )) {
                messages.push(system);
            }
            if let Ok(user) = Message::user(demand) {
                messages.push(user);
            }
            let request = CompletionRequest {
                model: model.to_string(),
                messages,
            };
            if let Ok(response) = runtime.block_on(provider.complete(request)) {
                let text = response.content.trim();
                if !text.is_empty() {
                    let first = text.lines().next().unwrap_or(text);
                    return first.chars().take(200).collect();
                }
            }
        }
        placeholder_deliverable(persona, model, demand)
    }
}

/// Resume a pending run with the user's decision; persists a release on success.
/// Returns `true` when the run has finished (done, aborted, or rolled back).
fn resume_run(
    root: &Path,
    run: &mut PendingRun,
    approved: bool,
    out: &mut impl Write,
) -> std::io::Result<bool> {
    // Only build the provider-backed deliverer at the execution gate.
    let signals = if approved && run.session.awaiting_execution() {
        let deliver = build_deliverer(root);
        run.session.resume(true, &deliver)
    } else {
        run.session.resume(approved, &placeholder_deliverable)
    };
    map_and_emit(&mut run.prev_stage, signals, out)?;
    if run.session.is_done() {
        match persistence::persist_release(root, run.session.demand(), run.session.deliverables()) {
            Ok(record) => emit(
                out,
                &CoreEvent::Log {
                    level: "info".to_string(),
                    message: format!("release {} persisted ({})", record.version, record.id),
                },
            )?,
            Err(e) => warn(out, format!("persistence failed: {e}"))?,
        }
        Ok(true)
    } else {
        Ok(!run.session.is_pending())
    }
}

/// A warning log line.
fn warn(out: &mut impl Write, message: String) -> std::io::Result<()> {
    emit(
        out,
        &CoreEvent::Log {
            level: "warn".to_string(),
            message,
        },
    )
}

/// Handle one command. Returns `Ok(true)` when the loop should stop.
fn handle(
    root: &Path,
    command: &TuiCommand,
    pending: &mut Option<PendingRun>,
    out: &mut impl Write,
) -> std::io::Result<bool> {
    match command {
        TuiCommand::SwitchMode { mode } => emit(out, &CoreEvent::ModeChanged { mode: *mode })?,
        TuiCommand::UserInput { text } => {
            if pending.is_some() {
                warn(
                    out,
                    "a run is awaiting approval — respond first".to_string(),
                )?;
            } else {
                *pending = begin_run(root, text, out)?;
            }
        }
        TuiCommand::ApprovalResponse { approved, .. } => {
            if let Some(run) = pending.as_mut() {
                let finished = resume_run(root, run, *approved, out)?;
                if finished {
                    *pending = None;
                }
            } else {
                warn(out, "no pending approval".to_string())?;
            }
        }
        TuiCommand::Command { name } if name == "quit" || name == "exit" => return Ok(true),
        TuiCommand::Command { name } => emit(
            out,
            &CoreEvent::Log {
                level: "info".to_string(),
                message: format!("/{name}"),
            },
        )?,

        // --- Config Mode ---
        TuiCommand::ConfigList => emit_tree(root, out)?,
        TuiCommand::ConfigOpen { id } => match gov::read(root, id) {
            Ok(body) => emit(
                out,
                &CoreEvent::ConfigEntry {
                    id: id.clone(),
                    entry_kind: kind_of(id)
                        .map(|k| k.as_str().to_string())
                        .unwrap_or_default(),
                    origin: origin_for(id).to_string(),
                    archived: false,
                    body,
                },
            )?,
            Err(e) => warn(out, e.to_string())?,
        },
        TuiCommand::ConfigValidate { id, body } => {
            let (ok, issues) = gov::validate(id, body);
            emit(out, &CoreEvent::ConfigValidation { ok, issues })?;
        }
        TuiCommand::ConfigCreate { id, body, .. } => match gov::create(root, id, body) {
            Ok(()) => {
                emit(out, &CoreEvent::ConfigSaved { id: id.clone() })?;
                emit_tree(root, out)?;
            }
            Err(e) => warn(out, e.to_string())?,
        },
        TuiCommand::ConfigSave { id, body } => {
            let (ok, issues) = gov::validate(id, body);
            if !ok {
                emit(out, &CoreEvent::ConfigValidation { ok, issues })?;
            } else {
                match gov::save(root, id, body) {
                    Ok(()) => emit(out, &CoreEvent::ConfigSaved { id: id.clone() })?,
                    Err(e) => warn(out, e.to_string())?,
                }
            }
        }
        TuiCommand::ConfigArchive { id } => match gov::archive(root, id) {
            Ok(_) => {
                emit(
                    out,
                    &CoreEvent::Log {
                        level: "info".to_string(),
                        message: format!("archived {id}"),
                    },
                )?;
                emit_tree(root, out)?;
            }
            Err(e) => warn(out, e.to_string())?,
        },

        // --- Product Mode ---
        TuiCommand::ListReleases => {
            let releases = persistence::list_releases(root)
                .into_iter()
                .map(|r| ReleaseSummary {
                    version: r.version,
                    changelog: r.changelog,
                    created_at: r.created_at,
                })
                .collect();
            emit(out, &CoreEvent::ReleaseList { releases })?;
        }
        TuiCommand::RunDemo { release } => {
            let result = demo_runner::run_demo(root, release, |stream, chunk| {
                emit(
                    out,
                    &CoreEvent::DemoOutput {
                        stream: stream.to_string(),
                        chunk: chunk.to_string(),
                    },
                )
            });
            match result {
                Ok(code) => emit(out, &CoreEvent::DemoExited { code })?,
                Err(e) => {
                    warn(out, format!("demo failed: {e}"))?;
                    emit(out, &CoreEvent::DemoExited { code: -1 })?;
                }
            }
        }
        other => emit(
            out,
            &CoreEvent::Log {
                level: "debug".to_string(),
                message: format!("unhandled command: {other:?}"),
            },
        )?,
    }
    Ok(false)
}

/// Run the duplex core loop until the input closes or a quit command arrives.
pub fn run_core(root: &Path, input: impl BufRead, mut out: impl Write) -> std::io::Result<()> {
    emit(
        &mut out,
        &CoreEvent::ModeChanged {
            mode: Mode::Maestro,
        },
    )?;
    emit(
        &mut out,
        &CoreEvent::Log {
            level: "info".to_string(),
            message: "Maestro core online".to_string(),
        },
    )?;
    for persona in gov::load_personas(root)
        .into_iter()
        .filter(|p| !p.orchestrator)
    {
        emit(
            &mut out,
            &CoreEvent::AgentState {
                agent: persona.id.to_string(),
                state: "idle".to_string(),
            },
        )?;
    }

    let mut pending: Option<PendingRun> = None;
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match decode::<TuiCommand>(&line) {
            Ok(command) => {
                if handle(root, &command, &mut pending, &mut out)? {
                    break;
                }
            }
            Err(e) => warn(&mut out, format!("rejected frame: {e}"))?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "maestro-server-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("maestro").join("personas")).unwrap();
        std::fs::create_dir_all(root.join("maestro").join("skills")).unwrap();
        std::fs::create_dir_all(root.join("maestro").join("scopes")).unwrap();
        root
    }

    /// Decode every event frame the core wrote to `out`.
    fn events(out: &[u8]) -> Vec<CoreEvent> {
        String::from_utf8(out.to_vec())
            .unwrap()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| decode::<CoreEvent>(l).unwrap())
            .collect()
    }

    fn run(root: &Path, commands: &[TuiCommand]) -> Vec<CoreEvent> {
        let input: String = commands.iter().map(|c| encode(c).unwrap()).collect();
        let mut out = Vec::new();
        run_core(root, std::io::Cursor::new(input), &mut out).unwrap();
        events(&out)
    }

    #[test]
    fn greets_with_maestro_mode_and_idle_personas() {
        let root = temp_root("greet");
        let evs = run(&root, &[]);
        assert_eq!(
            evs[0],
            CoreEvent::ModeChanged {
                mode: Mode::Maestro
            }
        );
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::AgentState { state, .. } if state == "idle")));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn switch_mode_is_echoed() {
        let root = temp_root("switch");
        let evs = run(
            &root,
            &[TuiCommand::SwitchMode {
                mode: Mode::Product,
            }],
        );
        assert!(evs.iter().any(|e| matches!(
            e,
            CoreEvent::ModeChanged {
                mode: Mode::Product
            }
        )));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn user_input_proposes_a_plan() {
        let root = temp_root("plan");
        let evs = run(
            &root,
            &[TuiCommand::UserInput {
                text: "build a cli".to_string(),
            }],
        );
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::PlanProposed { steps } if !steps.is_empty())));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn config_list_returns_the_default_catalog() {
        let root = temp_root("list");
        let evs = run(&root, &[TuiCommand::ConfigList]);
        assert!(evs.iter().any(|e| matches!(
            e,
            CoreEvent::ConfigTree { entries } if entries.iter().any(|n| n.id == "personas/maestro")
        )));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn config_create_then_save_and_archive() {
        let root = temp_root("crud");
        let evs = run(
            &root,
            &[
                TuiCommand::ConfigCreate {
                    entry_kind: "scope".to_string(),
                    id: "scopes/primary".to_string(),
                    body: "# scope".to_string(),
                },
                TuiCommand::ConfigSave {
                    id: "scopes/primary".to_string(),
                    body: "# scope v2".to_string(),
                },
                TuiCommand::ConfigArchive {
                    id: "scopes/primary".to_string(),
                },
            ],
        );
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::ConfigSaved { id } if id == "scopes/primary")));
        assert!(root.join("maestro/archive/scopes/primary.md").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn immutable_persona_save_is_reported() {
        let root = temp_root("immutable");
        let evs = run(
            &root,
            &[TuiCommand::ConfigSave {
                id: "personas/maestro".to_string(),
                body: "# hijack".to_string(),
            }],
        );
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::Log { level, message } if level == "warn" && message.contains("immutable"))));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn quit_command_stops_the_loop() {
        let root = temp_root("quit");
        let evs = run(
            &root,
            &[TuiCommand::Command {
                name: "quit".to_string(),
            }],
        );
        assert!(!evs.is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn malformed_frame_is_reported_not_fatal() {
        let root = temp_root("bad");
        let mut out = Vec::new();
        run_core(&root, std::io::Cursor::new("not json\n"), &mut out).unwrap();
        assert!(events(&out)
            .iter()
            .any(|e| matches!(e, CoreEvent::Log { level, .. } if level == "warn")));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn approval_flow_completes_and_persists_a_release() {
        let root = temp_root("approve");
        let evs = run(
            &root,
            &[
                TuiCommand::UserInput {
                    text: "improve quality assurance".to_string(),
                },
                TuiCommand::ApprovalResponse {
                    id: "approve-plan".to_string(),
                    approved: true,
                },
                TuiCommand::ApprovalResponse {
                    id: "approve-exec".to_string(),
                    approved: true,
                },
            ],
        );
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::ApprovalRequest { .. })));
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::Deliverable { persona, .. } if persona == "Maestro")));
        assert!(root.join("maestro/releases/r001/manifest.md").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn rejecting_the_plan_emits_no_delegation() {
        let root = temp_root("reject");
        let evs = run(
            &root,
            &[
                TuiCommand::UserInput {
                    text: "build".to_string(),
                },
                TuiCommand::ApprovalResponse {
                    id: "approve-plan".to_string(),
                    approved: false,
                },
            ],
        );
        assert!(!evs
            .iter()
            .any(|e| matches!(e, CoreEvent::Delegation { .. })));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn product_mode_lists_and_runs_a_release() {
        let root = temp_root("product");
        let evs = run(
            &root,
            &[
                TuiCommand::UserInput {
                    text: "build a cli".to_string(),
                },
                TuiCommand::ApprovalResponse {
                    id: "approve-plan".to_string(),
                    approved: true,
                },
                TuiCommand::ApprovalResponse {
                    id: "approve-exec".to_string(),
                    approved: true,
                },
                TuiCommand::ListReleases,
                TuiCommand::RunDemo {
                    release: "0.1.1".to_string(),
                },
            ],
        );
        assert!(evs.iter().any(|e| matches!(
            e,
            CoreEvent::ReleaseList { releases } if releases.iter().any(|r| r.version == "0.1.1")
        )));
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::DemoOutput { .. })));
        assert!(evs
            .iter()
            .any(|e| matches!(e, CoreEvent::DemoExited { code: 0 })));
        std::fs::remove_dir_all(&root).ok();
    }
}
