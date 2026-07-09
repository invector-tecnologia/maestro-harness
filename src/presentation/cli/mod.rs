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
    /// Run headless, without launching the Nim/Tatui TUI.
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
    /// Run the headless duplex IPC core (reads commands on stdin, writes events on stdout).
    Run,
    /// Launch the Nim/Tatui Workspace TUI (which spawns the core).
    Tui,
    /// Bootstrap a new project (plain-CLI questionnaire), scaffold defaults, then open the Workspace.
    Init {
        /// Optional project name; prompted if omitted.
        name: Option<String>,
    },
}

/// Dispatch a parsed [`Cli`] invocation.
pub fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    match cli.command {
        Some(Command::Version) => print_line(&format!("maestro {}", env!("CARGO_PKG_VERSION"))),
        Some(Command::ValidateConfig) => validate_config(&root)?,
        Some(Command::ListAgents) => {
            for name in agent_names() {
                print_line(&name);
            }
        }
        Some(Command::ScaffoldMarkdown) => {
            let created = scaffold_markdown(&root)?;
            print_line(&format!("scaffolded governance: {}", created.join(", ")));
        }
        Some(Command::InitConfig) => {
            let path = init_config(&root)?;
            print_line(&format!("wrote {}", path.display()));
        }
        Some(Command::Doctor) => doctor(&root)?,
        Some(Command::Run) => run_core()?,
        Some(Command::Tui) => launch_tui(None)?,
        Some(Command::Init { name }) => init_project(name, cli.no_tui)?,
        None => interactive_main_menu(cli.no_tui)?,
    }
    Ok(())
}

fn interactive_main_menu(no_tui: bool) -> anyhow::Result<()> {
    loop {
        {
            let mut out = std::io::stdout().lock();
            let _ = writeln!(out, "\nMaestro Interactive CLI");
            let _ = writeln!(out, "1) Init new project");
            let _ = writeln!(out, "2) Launch TUI");
            let _ = writeln!(out, "3) Validate Config");
            let _ = writeln!(out, "4) List Agents");
            let _ = writeln!(out, "5) Scaffold Governance");
            let _ = writeln!(out, "6) Doctor");
            let _ = writeln!(out, "7) Exit");
            let _ = write!(out, "\nSelect an option [1-7]: ");
            let _ = out.flush();
        }
        
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let choice = line.trim();
        
        let root = std::env::current_dir()?;
        
        match choice {
            "1" => {
                init_project(None, no_tui)?;
                break;
            }
            "2" => {
                launch_tui(None)?;
                break;
            }
            "3" => {
                validate_config(&root)?;
                break;
            }
            "4" => {
                for name in agent_names() {
                    print_line(&name);
                }
                break;
            }
            "5" => {
                let created = scaffold_markdown(&root)?;
                print_line(&format!("scaffolded governance: {}", created.join(", ")));
                break;
            }
            "6" => {
                doctor(&root)?;
                break;
            }
            "7" | "q" | "exit" | "" => {
                break;
            }
            _ => {
                let mut out = std::io::stdout().lock();
                let _ = writeln!(out, "Invalid option, try again.");
            }
        }
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
  # Optional cloud provider (set OPENAI_API_KEY in the environment, never here):
  # openai:
  #   kind: openai
  #   endpoint: "https://api.openai.com/v1"
  #   models:
  #     - name: gpt-4o-mini
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

/// The answers collected by `maestro init` (plain-CLI, no LLM).
#[derive(Debug, Clone, PartialEq)]
pub struct InitAnswers {
    /// Project name (required).
    pub name: String,
    /// Primary scope (required).
    pub scope: String,
    /// Optional project type (one of `PROJECT_TYPES`).
    pub kind: Option<String>,
    /// Optional layout-reference image paths.
    pub layout_refs: Vec<String>,
}

const PROJECT_TYPES: [&str; 4] = ["library", "Web", "Desktop", "Mobile"];

/// Run the headless duplex IPC core over stdin/stdout, rooted at the current dir.
fn run_core() -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    crate::presentation::ipc::server::run_core(&root, stdin.lock(), stdout.lock())?;
    Ok(())
}

const TUI_BINARY: &[u8] = include_bytes!("../../../frontend/maestro_tui");

/// Extract and launch the bundled Nim/Tatui TUI binary.
fn launch_tui(cwd: Option<&std::path::Path>) -> anyhow::Result<()> {
    let temp_dir = std::env::temp_dir();
    let tui_path = temp_dir.join(format!("maestro_tui_{}", std::process::id()));
    
    // Write bundled binary to temp directory
    let mut file = std::fs::File::create(&tui_path)?;
    file.write_all(TUI_BINARY)?;
    
    // Set executable permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        file.set_permissions(perms)?;
    }
    drop(file);

    let mut cmd = std::process::Command::new(&tui_path);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("failed to launch TUI: {e}"))?;
        
    // Cleanup temporary file
    let _ = std::fs::remove_file(&tui_path);
    
    if !status.success() {
        anyhow::bail!("TUI exited with status {status}");
    }
    Ok(())
}

/// Prompt for the bootstrap answers over `input`/`out`. Pure over its streams.
pub fn prompt_answers(
    mut input: impl std::io::BufRead,
    mut out: impl Write,
    name_arg: Option<String>,
) -> std::io::Result<InitAnswers> {
    fn read_line(input: &mut impl std::io::BufRead) -> std::io::Result<String> {
        let mut line = String::new();
        input.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

    let mut name = name_arg.unwrap_or_default().trim().to_string();
    while name.is_empty() {
        write!(out, "Project name (required): ")?;
        out.flush()?;
        name = read_line(&mut input)?;
    }

    let mut scope = String::new();
    while scope.is_empty() {
        write!(out, "Primary scope (required): ")?;
        out.flush()?;
        scope = read_line(&mut input)?;
    }

    write!(
        out,
        "Project type [library/Web/Desktop/Mobile] (optional): "
    )?;
    out.flush()?;
    let type_raw = read_line(&mut input)?;
    let kind = PROJECT_TYPES
        .iter()
        .find(|t| t.eq_ignore_ascii_case(&type_raw))
        .map(|t| (*t).to_string());

    let mut layout_refs = Vec::new();
    loop {
        write!(out, "Add a layout reference image path? (path or 'No'): ")?;
        out.flush()?;
        let ans = read_line(&mut input)?;
        if ans.is_empty() || ans.eq_ignore_ascii_case("no") || ans.eq_ignore_ascii_case("n") {
            break;
        }
        layout_refs.push(ans);
    }

    Ok(InitAnswers {
        name,
        scope,
        kind,
        layout_refs,
    })
}

/// Scaffold governance defaults and write the project's primary scope file.
pub fn scaffold_project(root: &Path, answers: &InitAnswers) -> std::io::Result<Vec<String>> {
    let mut created = scaffold_markdown(root)?;
    let _ = init_config(root)?;
    created.push("config.yml".to_string());

    let slug: String = answers
        .scope
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let scope_path = root
        .join("maestro")
        .join("scopes")
        .join(format!("{slug}.md"));

    let mut body = format!(
        "# Scope: {}\n\n- Project: {}\n",
        answers.scope, answers.name
    );
    if let Some(kind) = &answers.kind {
        body.push_str(&format!("- Type: {kind}\n"));
    }
    if !answers.layout_refs.is_empty() {
        body.push_str("- Layout references:\n");
        for reference in &answers.layout_refs {
            body.push_str(&format!("  - {reference}\n"));
        }
    }
    std::fs::write(&scope_path, body)?;
    created.push(format!("scopes/{slug}.md"));
    Ok(created)
}

/// Interactive bootstrap: collect answers, scaffold, then open the Workspace if requested.
fn init_project(name: Option<String>, no_tui: bool) -> anyhow::Result<()> {
    let base = std::env::current_dir()?;
    let stdin = std::io::stdin();
    let answers = prompt_answers(stdin.lock(), std::io::stdout(), name)?;
    
    let target_dir = base.join(&answers.name);
    std::fs::create_dir_all(&target_dir)?;

    let created = scaffold_project(&target_dir, &answers)?;
    print_line(&format!(
        "scaffolded project '{}' in '{}': {}",
        answers.name,
        target_dir.display(),
        created.join(", ")
    ));
    if !no_tui {
        print_line("opening Workspace (Maestro Mode)\u{2026}");
        if let Err(e) = launch_tui(Some(&target_dir)) {
            print_line(&format!("(TUI not launched: {e})"));
        }
    }
    Ok(())
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

    #[test]
    fn prompt_collects_required_and_optional_fields() {
        let input: &[u8] = b"\nMyProj\nprimary\nWeb\n/img/a.png\nno\n";
        let mut out = Vec::new();
        let answers = prompt_answers(input, &mut out, None).unwrap();
        assert_eq!(answers.name, "MyProj");
        assert_eq!(answers.scope, "primary");
        assert_eq!(answers.kind.as_deref(), Some("Web"));
        assert_eq!(answers.layout_refs, vec!["/img/a.png".to_string()]);
    }

    #[test]
    fn prompt_uses_name_arg_without_prompting() {
        let input: &[u8] = b"primary\n\nno\n";
        let mut out = Vec::new();
        let answers = prompt_answers(input, &mut out, Some("Given".to_string())).unwrap();
        assert_eq!(answers.name, "Given");
        assert_eq!(answers.scope, "primary");
        assert_eq!(answers.kind, None);
        assert!(answers.layout_refs.is_empty());
    }

    #[test]
    fn scaffold_project_writes_scope_file() {
        let root = std::env::temp_dir().join(format!("maestro-init-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let answers = InitAnswers {
            name: "P".to_string(),
            scope: "Primary Scope".to_string(),
            kind: Some("library".to_string()),
            layout_refs: vec![],
        };
        let created = scaffold_project(&root, &answers).unwrap();
        assert!(created.iter().any(|c| c.starts_with("scopes/")));
        let scope_file = root.join("maestro/scopes/primary_scope.md");
        assert!(scope_file.exists());
        let body = std::fs::read_to_string(&scope_file).unwrap();
        assert!(body.contains("Type: library"));
        std::fs::remove_dir_all(&root).ok();
    }
}
