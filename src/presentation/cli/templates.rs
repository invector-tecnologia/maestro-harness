//! Project templates for `maestro init --template <name>`.
//!
//! Templates are baked into the binary to preserve the local-first philosophy.
//! Each template provides pre-configured governance content for a specific
//! project type.

/// A project template definition.
#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    /// Template key (used with `--template <key>`).
    pub key: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Scope file content.
    pub scope_content: &'static str,
    /// Starter task spec content.
    pub task_spec: &'static str,
}

/// The built-in template catalog.
pub const TEMPLATES: &[ProjectTemplate] = &[
    ProjectTemplate {
        key: "web-app",
        description: "Web application (frontend + backend)",
        scope_content: "\
# Scope: Web Application

## Boundary
- Frontend: HTML/CSS/JS or framework (React, Vue, Svelte, etc.)
- Backend: API server with database
- Deployment: Container or serverless

## Constraints
- Must be responsive (mobile + desktop)
- Must pass Lighthouse performance audit ≥ 90
- Must include basic authentication

## Out of Scope
- Native mobile apps
- Real-time collaboration
",
        task_spec: "\
# Task 001: Web App Foundation

## Acceptance Criteria
- AC1: Project structure created with frontend and backend directories.
- AC2: Development server starts and serves a health-check endpoint.
- AC3: Basic HTML landing page renders in the browser.

## Risks
- Framework choice may constrain future architecture decisions.

## Rollback
- Delete the generated project directory.
",
    },
    ProjectTemplate {
        key: "cli-tool",
        description: "Command-line tool or utility",
        scope_content: "\
# Scope: CLI Tool

## Boundary
- A single binary (or script) invoked from the terminal
- Reads input from arguments, stdin, or config files
- Outputs structured text (plain, JSON, or table)

## Constraints
- Must support `--help` and `--version` flags
- Exit codes: 0 = success, 1 = user error, 2 = system error
- Must be cross-platform (Linux + macOS)

## Out of Scope
- GUI or TUI (unless explicitly requested)
- Daemon / long-running service mode
",
        task_spec: "\
# Task 001: CLI Foundation

## Acceptance Criteria
- AC1: Binary compiles and prints version with `--version`.
- AC2: `--help` shows usage with at least one subcommand.
- AC3: Non-zero exit code on invalid arguments.

## Risks
- Argument parser choice affects future extensibility.

## Rollback
- Delete the generated project directory.
",
    },
    ProjectTemplate {
        key: "library",
        description: "Reusable library or SDK",
        scope_content: "\
# Scope: Library

## Boundary
- Published as a package (crate, npm module, pip package, etc.)
- Public API surface must be documented
- Consumed by other projects, not end users

## Constraints
- Must have ≥ 80% test coverage on public API
- Must include API documentation with examples
- Semver versioning

## Out of Scope
- CLI wrapper (create a separate project)
- Application-specific logic
",
        task_spec: "\
# Task 001: Library Foundation

## Acceptance Criteria
- AC1: Package manifest exists with name, version, and license.
- AC2: At least one public type or function is exported.
- AC3: Unit test suite runs with at least one passing test.

## Risks
- API design decisions are hard to change after v1.0.

## Rollback
- Delete the generated project directory.
",
    },
    ProjectTemplate {
        key: "infra",
        description: "Infrastructure automation (scripts, IaC, deployment)",
        scope_content: "\
# Scope: Infrastructure Automation

## Boundary
- Scripts, configuration, or IaC modules for provisioning and deployment
- Targets: cloud providers, containers, CI/CD pipelines
- Idempotent operations preferred

## Constraints
- Must include a dry-run / plan mode before applying changes
- Must log all state-changing operations
- Secrets must never be committed to the repository

## Out of Scope
- Application code (this is infra only)
- Monitoring and alerting (separate concern)
",
        task_spec: "\
# Task 001: Infra Foundation

## Acceptance Criteria
- AC1: Directory structure matches target platform conventions.
- AC2: A dry-run / plan command executes without errors.
- AC3: A README documents prerequisites and usage.

## Risks
- Cloud provider API changes may break automation.

## Rollback
- Destroy provisioned resources using the inverse script.
",
    },
];

/// Look up a template by key (case-insensitive).
pub fn find(key: &str) -> Option<&'static ProjectTemplate> {
    TEMPLATES.iter().find(|t| t.key.eq_ignore_ascii_case(key))
}

/// List all available template keys with descriptions.
pub fn list() -> Vec<(&'static str, &'static str)> {
    TEMPLATES.iter().map(|t| (t.key, t.description)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_known_template() {
        assert!(find("web-app").is_some());
        assert!(find("cli-tool").is_some());
        assert!(find("library").is_some());
        assert!(find("infra").is_some());
    }

    #[test]
    fn find_is_case_insensitive() {
        assert!(find("Web-App").is_some());
        assert!(find("CLI-TOOL").is_some());
    }

    #[test]
    fn find_returns_none_for_unknown() {
        assert!(find("blockchain-dapp").is_none());
    }

    #[test]
    fn list_returns_all_templates() {
        let all = list();
        assert_eq!(all.len(), TEMPLATES.len());
        assert!(all.iter().any(|(k, _)| *k == "web-app"));
    }

    #[test]
    fn template_content_is_non_empty() {
        for t in TEMPLATES {
            assert!(!t.scope_content.is_empty(), "{} scope is empty", t.key);
            assert!(!t.task_spec.is_empty(), "{} task_spec is empty", t.key);
        }
    }
}
