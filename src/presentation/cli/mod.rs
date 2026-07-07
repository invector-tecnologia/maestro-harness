//! CLI surface — argument parsing and command dispatch (TASK 014).
//!
//! The governance command set. `anyhow` aggregation is confined to this boundary;
//! the layers below return typed errors.

use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::domain::models::default_personas;

/// Maestro command-line interface.
#[derive(Debug, Parser)]
#[command(
    name = "maestro",
    version,
    about = "Maestro — tactical agentic orchestrator"
)]
pub struct Cli {
    /// Run headless, without launching the Nim/Niobium TUI.
    #[arg(long, global = true)]
    pub no_tui: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Top-level governance commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print version information.
    Version,
    /// Validate `maestro/config.yml` and its cross-references.
    ValidateConfig,
    /// List the registered personas.
    ListAgents,
    /// Create the mandatory governance markdown scaffold.
    ScaffoldMarkdown,
    /// Generate `maestro/config.yml` from a template.
    InitConfig,
    /// Run readiness checks (config, governance).
    Doctor,
}

/// Dispatch a parsed [`Cli`] invocation.
pub fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    match cli.command.unwrap_or(Command::Version) {
        Command::Version => print_line(&format!("maestro {}", env!("CARGO_PKG_VERSION"))),
        Command::ValidateConfig => validate_config(&root)?,
        Command::ListAgents => {
            for name in agent_names() {
                print_line(&name);
            }
        }
        Command::ScaffoldMarkdown => {
            let created = scaffold_markdown(&root)?;
            print_line(&format!("scaffolded governance: {}", created.join(", ")));
        }
        Command::InitConfig => {
            let path = init_config(&root)?;
            print_line(&format!("wrote {}", path.display()));
        }
        Command::Doctor => doctor(&root)?,
    }
    Ok(())
}

/// The persona names in the default catalog.
pub fn agent_names() -> Vec<String> {
    default_personas()
        .into_iter()
        .map(|p| p.id.to_string())
        .collect()
}

/// Create the `scopes`/`personas`/`skills` governance folders under `root/maestro`.
pub fn scaffold_markdown(root: &Path) -> std::io::Result<Vec<String>> {
    let base = root.join("maestro");
    let mut created = Vec::new();
    for entry in crate::domain::models::REQUIRED_GOVERNANCE_ENTRIES {
        std::fs::create_dir_all(base.join(entry))?;
        created.push(entry.to_string());
    }
    Ok(created)
}

/// Write a starter `maestro/config.yml` if one does not already exist.
pub fn init_config(root: &Path) -> std::io::Result<PathBuf> {
    let dir = root.join("maestro");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.yml");
    if !path.exists() {
        std::fs::write(&path, CONFIG_TEMPLATE)?;
    }
    Ok(path)
}

const CONFIG_TEMPLATE: &str = r#"system:
  default_provider: ollama
  default_model: mistral
  max_concurrency: 4
providers:
  ollama:
    kind: ollama
    endpoint: "http://127.0.0.1:11434/v1"
    models:
      - name: mistral
"#;

fn validate_config(root: &Path) -> anyhow::Result<()> {
    match crate::infrastructure::config::load_from(root) {
        Ok(config) => {
            print_line(&format!(
                "config OK: {} provider(s), default {}/{}",
                config.providers.len(),
                config.system.default_provider,
                config.system.default_model
            ));
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

fn doctor(root: &Path) -> anyhow::Result<()> {
    let config = crate::infrastructure::config::load_from(root);
    print_line(&format!("[{}] configuration", pass_fail(config.is_ok())));

    let governance = crate::application::governance::validate_dir(&root.join("maestro"))?;
    print_line(&format!(
        "[{}] governance scaffold{}",
        pass_fail(governance.is_valid()),
        if governance.is_valid() {
            String::new()
        } else {
            format!(" (missing: {})", governance.missing.join(", "))
        }
    ));
    Ok(())
}

fn pass_fail(ok: bool) -> &'static str {
    if ok {
        "PASS"
    } else {
        "FAIL"
    }
}

fn print_line(text: &str) {
    // User-facing CLI output (not diagnostic logging).
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{text}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_no_tui_flag() {
        let cli = Cli::parse_from(["maestro", "--no-tui", "version"]);
        assert!(cli.no_tui);
        assert!(matches!(cli.command, Some(Command::Version)));
    }

    #[test]
    fn defaults_to_no_subcommand() {
        let cli = Cli::parse_from(["maestro"]);
        assert!(!cli.no_tui);
        assert!(cli.command.is_none());
    }

    #[test]
    fn lists_default_personas() {
        let names = agent_names();
        assert!(names.contains(&"Maestro".to_string()));
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn scaffold_creates_governance_dirs() {
        let root =
            std::env::temp_dir().join(format!("maestro-cli-scaffold-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let created = scaffold_markdown(&root).unwrap();
        assert_eq!(created.len(), 3);
        for entry in ["scopes", "personas", "skills"] {
            assert!(root.join("maestro").join(entry).is_dir());
        }
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn init_config_writes_template() {
        let root = std::env::temp_dir().join(format!("maestro-cli-initcfg-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = init_config(&root).unwrap();
        assert!(path.exists());
        // The written template must load and validate.
        let cfg = crate::infrastructure::config::load_from(&root).unwrap();
        assert_eq!(cfg.system.default_provider, "ollama");
        std::fs::remove_dir_all(&root).ok();
    }
}
