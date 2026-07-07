//! Governance folder scanning (TASK 005).
//!
//! Lists the entries under a governance directory and validates them against the
//! required set using the pure `crate::domain::models::governance` logic.

use std::path::{Path, PathBuf};

use crate::domain::models::governance::{
    default_persona_ids, is_immutable, kind_of, slug, validate_entries, GovernanceEntry,
    GovernanceKind, GovernanceReport, Origin,
};
use crate::domain::models::{default_personas, AgentId, MaestroConfig, Persona};

/// Scan `governance_dir` and validate that `scopes`, `personas`, and `skills`
/// are present. A missing directory yields a report listing everything as missing.
pub fn validate_dir(governance_dir: &Path) -> std::io::Result<GovernanceReport> {
    let mut present = Vec::new();
    if governance_dir.is_dir() {
        for entry in std::fs::read_dir(governance_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                present.push(name.to_string());
            }
        }
    }
    Ok(validate_entries(&present))
}

/// Errors from Config Mode governance operations.
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    /// The entry may never be edited or archived (the Maestro persona).
    #[error("entry '{0}' is immutable")]
    Immutable(String),
    /// The entry does not exist.
    #[error("entry '{0}' not found")]
    NotFound(String),
    /// A create targeted an id that already exists.
    #[error("entry '{0}' already exists")]
    Exists(String),
    /// Underlying filesystem failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Resolve an entry id to its on-disk file path (`None` for unrecognised ids).
fn entry_file(root: &Path, id: &str) -> Option<PathBuf> {
    let base = root.join("maestro");
    if id == "config.yml" {
        return Some(base.join("config.yml"));
    }
    let (dir, name) = id.split_once('/')?;
    if name.is_empty() {
        return None;
    }
    Some(base.join(dir).join(format!("{name}.md")))
}

/// Collect the `.md` entries in `dir_path` into `out`.
fn collect_md(
    dir_path: &Path,
    dir: &str,
    kind: GovernanceKind,
    defaults: &[String],
    archived: bool,
    out: &mut Vec<GovernanceEntry>,
) -> std::io::Result<()> {
    if !dir_path.is_dir() {
        return Ok(());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
        let path = entry?.path();
        if path.extension().and_then(|x| x.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|x| x.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    for name in names {
        let id = format!("{dir}/{name}");
        let is_default = defaults.iter().any(|d| d == &id);
        // Non-archived default personas are already represented by the built-in entry.
        if !archived && is_default {
            continue;
        }
        let origin = if is_default {
            Origin::Default
        } else {
            Origin::Custom
        };
        out.push(GovernanceEntry {
            id,
            kind,
            origin,
            archived,
        });
    }
    Ok(())
}

/// List the governance tree: config, the built-in default personas, any on-disk
/// custom personas/skills/scopes, and archived entries.
pub fn list(root: &Path) -> std::io::Result<Vec<GovernanceEntry>> {
    let base = root.join("maestro");
    let mut entries = Vec::new();

    if base.join("config.yml").exists() {
        entries.push(GovernanceEntry {
            id: "config.yml".to_string(),
            kind: GovernanceKind::Config,
            origin: Origin::Default,
            archived: false,
        });
    }

    let defaults = default_persona_ids();
    for id in &defaults {
        entries.push(GovernanceEntry {
            id: id.clone(),
            kind: GovernanceKind::Persona,
            origin: Origin::Default,
            archived: false,
        });
    }

    let dirs = [
        (GovernanceKind::Persona, "personas"),
        (GovernanceKind::Skill, "skills"),
        (GovernanceKind::Scope, "scopes"),
    ];
    for (kind, dir) in dirs {
        collect_md(&base.join(dir), dir, kind, &defaults, false, &mut entries)?;
    }
    for (kind, dir) in dirs {
        collect_md(
            &base.join("archive").join(dir),
            dir,
            kind,
            &defaults,
            true,
            &mut entries,
        )?;
    }
    Ok(entries)
}

/// Read an entry's body. Built-in default personas without an override file are
/// synthesized from the catalog.
pub fn read(root: &Path, id: &str) -> Result<String, GovernanceError> {
    if let Some(path) = entry_file(root, id) {
        if path.exists() {
            return Ok(std::fs::read_to_string(path)?);
        }
    }
    if default_persona_ids().iter().any(|d| d == id) {
        if let Some(persona) = default_personas()
            .into_iter()
            .find(|p| format!("personas/{}", slug(&p.id.to_string())) == id)
        {
            return Ok(format!(
                "# {}\n\n## Responsibility\n{}\n",
                persona.id, persona.responsibility
            ));
        }
    }
    Err(GovernanceError::NotFound(id.to_string()))
}

/// Create a new custom entry. Fails if it exists or targets the immutable persona.
pub fn create(root: &Path, id: &str, body: &str) -> Result<(), GovernanceError> {
    if is_immutable(id) {
        return Err(GovernanceError::Immutable(id.to_string()));
    }
    let path = entry_file(root, id).ok_or_else(|| GovernanceError::NotFound(id.to_string()))?;
    if path.exists() {
        return Err(GovernanceError::Exists(id.to_string()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body)?;
    Ok(())
}

/// Overwrite an entry's body. Rejects the immutable persona and unknown kinds.
pub fn save(root: &Path, id: &str, body: &str) -> Result<(), GovernanceError> {
    if is_immutable(id) {
        return Err(GovernanceError::Immutable(id.to_string()));
    }
    if kind_of(id).is_none() {
        return Err(GovernanceError::NotFound(id.to_string()));
    }
    let path = entry_file(root, id).ok_or_else(|| GovernanceError::NotFound(id.to_string()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body)?;
    Ok(())
}

/// Soft-delete an entry by moving it under `maestro/archive/<dir>/`.
pub fn archive(root: &Path, id: &str) -> Result<PathBuf, GovernanceError> {
    if is_immutable(id) {
        return Err(GovernanceError::Immutable(id.to_string()));
    }
    let (dir, name) = id
        .split_once('/')
        .ok_or_else(|| GovernanceError::Immutable(id.to_string()))?;
    let base = root.join("maestro");
    let src = base.join(dir).join(format!("{name}.md"));
    if !src.exists() {
        return Err(GovernanceError::NotFound(id.to_string()));
    }
    let dest_dir = base.join("archive").join(dir);
    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(format!("{name}.md"));
    std::fs::rename(&src, &dest)?;
    Ok(dest)
}

/// Validate an entry body before saving. Config is parsed + cross-checked; other
/// kinds must be non-empty. Returns `(ok, issues)`.
pub fn validate(id: &str, body: &str) -> (bool, Vec<String>) {
    if id == "config.yml" {
        match serde_yaml::from_str::<MaestroConfig>(body) {
            Ok(config) => match config.validate() {
                Ok(()) => (true, Vec::new()),
                Err(e) => (false, vec![e.to_string()]),
            },
            Err(e) => (false, vec![format!("YAML parse error: {e}")]),
        }
    } else if body.trim().is_empty() {
        (false, vec!["entry body is empty".to_string()])
    } else {
        (true, Vec::new())
    }
}

/// Parse a persona markdown body into a [`Persona`] (custom, non-orchestrator).
/// Name comes from the first `# ` heading; responsibility from the first line of
/// the `## Responsibility` section. Returns `None` if there is no name.
fn parse_custom_persona(body: &str, maestro: &AgentId) -> Option<Persona> {
    let mut name = String::new();
    let mut responsibility = String::new();
    let mut in_responsibility = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            if name.is_empty() {
                name = heading.trim().to_string();
            }
        } else if trimmed.eq_ignore_ascii_case("## responsibility") {
            in_responsibility = true;
        } else if trimmed.starts_with("## ") {
            in_responsibility = false;
        } else if in_responsibility && responsibility.is_empty() && !trimmed.is_empty() {
            responsibility = trimmed.to_string();
        }
    }
    if name.is_empty() {
        return None;
    }
    if responsibility.is_empty() {
        responsibility = format!("Custom persona: {name}.");
    }
    let id = AgentId::new(name).ok()?;
    Persona::new(id, responsibility, vec![maestro.clone()], false).ok()
}

/// The governed persona catalog: the built-in defaults plus any non-archived
/// custom personas authored in Config Mode. This is what Maestro Mode routes over.
pub fn load_personas(root: &Path) -> Vec<Persona> {
    let mut personas = default_personas();
    let maestro = match AgentId::new("Maestro") {
        Ok(id) => id,
        Err(_) => return personas,
    };
    for entry in list(root).unwrap_or_default() {
        if entry.kind != GovernanceKind::Persona || entry.origin != Origin::Custom || entry.archived
        {
            continue;
        }
        if let Ok(body) = read(root, &entry.id) {
            if let Some(persona) = parse_custom_persona(&body, &maestro) {
                if !personas.iter().any(|existing| existing.id == persona.id) {
                    personas.push(persona);
                }
            }
        }
    }
    personas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_when_all_entries_present() {
        let dir = std::env::temp_dir().join(format!("maestro-gov-{}", std::process::id()));
        for entry in ["scopes", "personas", "skills"] {
            std::fs::create_dir_all(dir.join(entry)).unwrap();
        }
        let report = validate_dir(&dir).unwrap();
        assert!(report.is_valid());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_dir_reports_all_missing() {
        let dir =
            std::env::temp_dir().join(format!("maestro-gov-absent-{}", std::process::id() + 1));
        let report = validate_dir(&dir).unwrap();
        assert_eq!(report.missing.len(), 3);
    }

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "maestro-govsvc-{tag}-{}-{}",
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

    #[test]
    fn list_includes_default_personas() {
        let root = temp_root("list");
        let entries = list(&root).unwrap();
        assert!(entries
            .iter()
            .any(|e| e.id == "personas/maestro" && e.origin == Origin::Default));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn create_read_and_list_custom_persona() {
        let root = temp_root("create");
        create(&root, "personas/api_designer", "# API Designer\n").unwrap();
        let body = read(&root, "personas/api_designer").unwrap();
        assert!(body.contains("API Designer"));
        let entries = list(&root).unwrap();
        assert!(entries
            .iter()
            .any(|e| e.id == "personas/api_designer" && e.origin == Origin::Custom));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_synthesizes_default_persona_body() {
        let root = temp_root("synth");
        let body = read(&root, "personas/software_engineer").unwrap();
        assert!(body.contains("Responsibility"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn immutable_persona_cannot_be_saved_or_archived() {
        let root = temp_root("immutable");
        assert!(matches!(
            save(&root, "personas/maestro", "x"),
            Err(GovernanceError::Immutable(_))
        ));
        assert!(matches!(
            archive(&root, "personas/maestro"),
            Err(GovernanceError::Immutable(_))
        ));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn archive_moves_entry_and_marks_it() {
        let root = temp_root("archive");
        create(&root, "scopes/primary", "# scope\n").unwrap();
        archive(&root, "scopes/primary").unwrap();
        assert!(root.join("maestro/archive/scopes/primary.md").exists());
        let entries = list(&root).unwrap();
        assert!(entries
            .iter()
            .any(|e| e.id == "scopes/primary" && e.archived));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn validate_rejects_bad_config_and_empty_body() {
        let (ok, issues) = validate("config.yml", "not: [valid");
        assert!(!ok);
        assert!(!issues.is_empty());
        let (ok, _) = validate("scopes/x", "   ");
        assert!(!ok);
        let (ok, _) = validate("scopes/x", "# real");
        assert!(ok);
    }

    #[test]
    fn load_personas_merges_defaults_and_customs() {
        let root = temp_root("catalog");
        // Defaults only until a custom persona is authored.
        let before = load_personas(&root);
        assert!(before.iter().any(|p| p.id.to_string() == "Maestro"));
        assert!(!before.iter().any(|p| p.id.to_string() == "API Designer"));

        create(
            &root,
            "personas/api_designer",
            "# API Designer\n\n## Responsibility\nDesign REST and gRPC contracts.\n",
        )
        .unwrap();

        let after = load_personas(&root);
        let api = after.iter().find(|p| p.id.to_string() == "API Designer");
        assert!(api.is_some());
        assert!(api.unwrap().responsibility.contains("REST"));
        assert!(!api.unwrap().orchestrator);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn archived_custom_persona_is_excluded_from_catalog() {
        let root = temp_root("catalog-archived");
        create(&root, "personas/temp_helper", "# Temp Helper\n").unwrap();
        assert!(load_personas(&root)
            .iter()
            .any(|p| p.id.to_string() == "Temp Helper"));
        archive(&root, "personas/temp_helper").unwrap();
        assert!(!load_personas(&root)
            .iter()
            .any(|p| p.id.to_string() == "Temp Helper"));
        std::fs::remove_dir_all(&root).ok();
    }
}
