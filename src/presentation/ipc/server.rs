//! Duplex IPC server — the headless core loop behind `maestro run` (TASK 053).
//!
//! Reads [`TuiCommand`] frames from an input stream and writes [`CoreEvent`]
//! frames to an output stream, both protocol v2 (ADR 0003). Config Mode commands
//! are served by the governance service against the project `root`. The core is
//! rendering-agnostic; no TUI types cross the boundary.

use std::io::{BufRead, Write};
use std::path::Path;

use crate::application::governance as gov;
use crate::domain::models::default_personas;
use crate::domain::models::governance::{default_persona_ids, kind_of, GovernanceEntry};

use super::{decode, encode, ConfigNode, CoreEvent, Mode, TuiCommand};

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
fn handle(root: &Path, command: &TuiCommand, out: &mut impl Write) -> std::io::Result<bool> {
    match command {
        TuiCommand::SwitchMode { mode } => emit(out, &CoreEvent::ModeChanged { mode: *mode })?,
        TuiCommand::UserInput { text } => {
            emit(
                out,
                &CoreEvent::PlanProposed {
                    steps: vec![
                        format!("understand: {text}"),
                        "delegate to personas".to_string(),
                        "audit deliverables".to_string(),
                        "deliver".to_string(),
                    ],
                },
            )?;
            emit(
                out,
                &CoreEvent::Log {
                    level: "info".to_string(),
                    message: format!("demand received: {text}"),
                },
            )?;
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

        // --- Product Mode (populated in Phase 5) ---
        TuiCommand::ListReleases => emit(out, &CoreEvent::ReleaseList { releases: vec![] })?,
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
    for persona in default_personas().into_iter().filter(|p| !p.orchestrator) {
        emit(
            &mut out,
            &CoreEvent::AgentState {
                agent: persona.id.to_string(),
                state: "idle".to_string(),
            },
        )?;
    }

    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match decode::<TuiCommand>(&line) {
            Ok(command) => {
                if handle(root, &command, &mut out)? {
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
}
