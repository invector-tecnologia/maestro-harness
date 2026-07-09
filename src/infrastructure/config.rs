//! Configuration loading (TASK 006).
//!
//! Resolves `./maestro/config.yml` first, then the global XDG config path. A
//! legacy `config.yaml` is accepted with a deprecation warning. Parsing and
//! cross-reference validation use the pure types in `crate::domain::models::config`.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::models::config::MaestroConfig;

/// Errors from locating, reading, or parsing configuration.
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// No configuration file found in any known location.
    #[error("no configuration found (looked in ./maestro and the XDG config path)")]
    NotFound,
    /// The file could not be read.
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// The file was not valid YAML for the schema.
    #[error("invalid configuration in {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
    /// The parsed configuration failed cross-reference validation.
    #[error("configuration validation failed: {0}")]
    Invalid(#[from] crate::domain::models::config::ConfigError),
}

/// Candidate config paths under a project root, in precedence order.
fn candidate_paths(project_root: &Path) -> Vec<(PathBuf, bool)> {
    vec![
        (project_root.join("maestro/config.yml"), false),
        (project_root.join("maestro/config.yaml"), true), // legacy
    ]
}

/// Load and validate configuration starting from `project_root`, falling back to
/// the global XDG config directory (`$XDG_CONFIG_HOME/maestro/config.yml`).
pub fn load_from(project_root: &Path) -> Result<MaestroConfig, ConfigLoadError> {
    let mut candidates = candidate_paths(project_root);
    if let Some(xdg) = xdg_config_path() {
        candidates.push((xdg, false));
    }

    for (path, legacy) in candidates {
        if path.exists() {
            if legacy {
                tracing::warn!(
                    path = %path.display(),
                    "using legacy config.yaml; rename to config.yml"
                );
            }
            return parse_and_validate(&path);
        }
    }
    Err(ConfigLoadError::NotFound)
}

/// The global XDG config path, if `$XDG_CONFIG_HOME` or `$HOME` is set.
fn xdg_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir).join("maestro/config.yml"));
        }
    }
    std::env::var("HOME")
        .ok()
        .filter(|h| !h.is_empty())
        .map(|home| PathBuf::from(home).join(".config/maestro/config.yml"))
}

fn parse_and_validate(path: &Path) -> Result<MaestroConfig, ConfigLoadError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigLoadError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let config: MaestroConfig =
        serde_yaml::from_str(&text).map_err(|source| ConfigLoadError::Parse {
            path: path.display().to_string(),
            source,
        })?;
    config.validate()?;
    Ok(config)
}

/// Save a configuration to the given project root's maestro/config.yml.
pub fn save_to(project_root: &Path, config: &MaestroConfig) -> Result<(), ConfigLoadError> {
    let path = project_root.join("maestro/config.yml");
    let text = serde_yaml::to_string(config).map_err(|source| ConfigLoadError::Parse {
        path: path.display().to_string(),
        source,
    })?;
    std::fs::write(&path, text).map_err(|source| ConfigLoadError::Io {
        path: path.display().to_string(),
        source,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
system:
  default_provider: ollama
  default_model: mistral
  max_concurrency: 2
providers:
  ollama:
    kind: ollama
    endpoint: "http://127.0.0.1:11434/v1"
    models:
      - name: mistral
"#;

    #[test]
    fn loads_primary_config_yml() {
        let dir = std::env::temp_dir().join(format!("maestro-cfg-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("maestro")).unwrap();
        std::fs::write(dir.join("maestro/config.yml"), SAMPLE).unwrap();

        let cfg = load_from(&dir).expect("load");
        assert_eq!(cfg.system.default_model, "mistral");
        assert_eq!(cfg.system.max_concurrency, 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_config_is_not_found() {
        let dir = std::env::temp_dir().join(format!("maestro-cfg-none-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Prevent XDG/HOME fallback from finding a real user config during the test.
        let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let prev_home = std::env::var("HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", dir.join("empty"));
        std::env::remove_var("HOME");

        let result = load_from(&dir);
        assert!(matches!(result, Err(ConfigLoadError::NotFound)));

        if let Some(v) = prev_xdg {
            std::env::set_var("XDG_CONFIG_HOME", v);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(v) = prev_home {
            std::env::set_var("HOME", v);
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
