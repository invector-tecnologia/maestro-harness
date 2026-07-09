//! Provider-aware config generation for `maestro init-config`.

use std::path::{Path, PathBuf};

/// Result of an init-config operation.
pub struct InitConfigResult {
    /// Path to the written config file.
    pub path: PathBuf,
    /// Whether the connection probe succeeded.
    pub probe_ok: bool,
    /// Human-readable probe message.
    pub probe_msg: String,
}

/// Known provider presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderPreset {
    Ollama,
    OpenAi,
    Gemini,
}

impl ProviderPreset {
    /// Parse from a CLI string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::OpenAi),
            "gemini" => Ok(Self::Gemini),
            _ => Err(format!("unknown provider '{}'. Supported: ollama, openai, gemini", s)),
        }
    }

    /// Default name for each provider.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::Gemini => "gemini",
        }
    }

    /// Default endpoint for each provider.
    pub fn default_endpoint(&self) -> &'static str {
        match self {
            Self::Ollama => "http://127.0.0.1:11434/v1",
            Self::OpenAi => "https://api.openai.com/v1",
            Self::Gemini => "https://generativelanguage.googleapis.com",
        }
    }

    /// Default model for each provider.
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Ollama => "mistral",
            Self::OpenAi => "gpt-4o-mini",
            Self::Gemini => "gemini-2.0-flash",
        }
    }
}

/// Detect which providers are available in the environment.
/// Checks: (1) Ollama at localhost:11434, (2) OPENAI_API_KEY set, (3) GEMINI_API_KEY set.
/// Returns the list of detected presets, best-first.
pub fn detect_providers() -> Vec<ProviderPreset> {
    let mut found = Vec::new();

    // 1. Check for a running Ollama instance (TCP connect to 127.0.0.1:11434)
    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        std::time::Duration::from_millis(500),
    )
    .is_ok()
    {
        found.push(ProviderPreset::Ollama);
    }

    // 2. Check for cloud API keys in the environment
    if std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .is_some()
    {
        found.push(ProviderPreset::OpenAi);
    }
    if std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .is_some()
    {
        found.push(ProviderPreset::Gemini);
    }

    found
}

/// Generate a config YAML string for the given preset + overrides.
pub fn config_for_provider(
    preset: ProviderPreset,
    endpoint: Option<&str>,
    model: Option<&str>,
) -> String {
    let endpoint = endpoint.unwrap_or(preset.default_endpoint());
    let model = model.unwrap_or(preset.default_model());
    
    match preset {
        ProviderPreset::Ollama => format!(
            r#"system:
  default_provider: ollama
  default_model: {model}
  max_concurrency: 4
providers:
  ollama:
    kind: ollama
    endpoint: "{endpoint}"
    models:
      - name: {model}
"#,
            model = model,
            endpoint = endpoint,
        ),
        ProviderPreset::OpenAi => format!(
            r#"system:
  default_provider: openai
  default_model: {model}
  max_concurrency: 4
providers:
  openai:
    kind: openai
    endpoint: "{endpoint}"
    models:
      - name: {model}
  # API key is read from $OPENAI_API_KEY (never store keys in config files).
"#,
            model = model,
            endpoint = endpoint,
        ),
        ProviderPreset::Gemini => format!(
            r#"system:
  default_provider: gemini
  default_model: {model}
  max_concurrency: 4
providers:
  gemini:
    kind: gemini
    endpoint: "{endpoint}"
    models:
      - name: {model}
  # API key is read from $GEMINI_API_KEY (never store keys in config files).
"#,
            model = model,
            endpoint = endpoint,
        ),
    }
}

/// Main entry point: resolve provider, write config, probe connectivity.
pub fn init_config_with_provider(
    root: &Path,
    provider: Option<String>,
    endpoint: Option<String>,
    model: Option<String>,
) -> anyhow::Result<InitConfigResult> {
    // 1. Resolve which provider to use
    let preset = if let Some(ref p) = provider {
        ProviderPreset::from_str_loose(p).map_err(|e| anyhow::anyhow!(e))?
    } else {
        // Auto-detect: pick the first available, default to Ollama
        let detected = detect_providers();
        if !detected.is_empty() {
            super::print_line(&format!(
                "auto-detected: {}",
                detected.iter().map(|p| p.name()).collect::<Vec<_>>().join(", ")
            ));
        }
        detected.into_iter().next().unwrap_or(ProviderPreset::Ollama)
    };

    // 2. Generate and write config
    let config_yaml = config_for_provider(preset, endpoint.as_deref(), model.as_deref());
    let dir = root.join("maestro");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.yml");
    if path.exists() {
        anyhow::bail!(
            "config already exists at {}; delete it first to regenerate",
            path.display()
        );
    }
    std::fs::write(&path, config_yaml)?;

    // 3. Connection test
    let (probe_ok, probe_msg) = run_connection_test(root);

    Ok(InitConfigResult {
        path,
        probe_ok,
        probe_msg,
    })
}

fn run_connection_test(root: &Path) -> (bool, String) {
    let config = match crate::infrastructure::config::load_from(root) {
        Ok(c) => c,
        Err(e) => return (false, format!("config load failed: {}", e)),
    };
    let registry = match crate::infrastructure::llm::registry::ProviderRegistry::from_config(&config)
    {
        Ok(r) => r,
        Err(e) => return (false, format!("provider build failed: {}", e)),
    };
    let provider = registry.default_provider(&config);

    // Build a one-shot Tokio runtime for the probe — acceptable at CLI boundary
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return (false, format!("runtime error: {}", e)),
    };
    let status = rt.block_on(crate::application::readiness::probe_provider(provider));
    match status {
        crate::domain::ports::ProviderStatus::Available => {
            (true, "provider is reachable ✓".to_string())
        }
        crate::domain::ports::ProviderStatus::Unreachable => (
            false,
            "provider is unreachable — is it running?".to_string(),
        ),
        crate::domain::ports::ProviderStatus::Unauthorized => (
            false,
            "API key missing or rejected — check your environment variable".to_string(),
        ),
        crate::domain::ports::ProviderStatus::ModelMissing => (
            false,
            "endpoint reached but model not found — check model name".to_string(),
        ),
    }
}
