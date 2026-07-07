//! Git-standalone micro-project persistence (TASK 050).
//!
//! When a micro-project reaches Verification, it is packaged into
//! `maestro/releases/<id>/` with a manifest and committed to a standalone git
//! repository (best-effort — a missing `git` is not fatal). Completed releases
//! feed Product Mode.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata for a persisted release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseRecord {
    /// Release id (e.g. `r001`).
    pub id: String,
    /// Human version tag (e.g. `0.1.1`).
    pub version: String,
    /// Changelog / notes body.
    pub changelog: String,
    /// Creation timestamp (unix seconds).
    pub created_at: String,
}

/// Count existing `r*` release directories under `releases`.
fn release_count(releases: &Path) -> usize {
    std::fs::read_dir(releases)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_dir()
                        && e.file_name()
                            .to_str()
                            .map(|n| n.starts_with('r'))
                            .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Best-effort git: initialise the release dir as a standalone repo and commit.
fn git_commit(dir: &Path, message: &str) {
    let run = |args: &[&str]| {
        let _ = Command::new("git").arg("-C").arg(dir).args(args).output();
    };
    run(&["init", "-q"]);
    run(&["add", "-A"]);
    run(&[
        "-c",
        "user.email=maestro@localhost",
        "-c",
        "user.name=Maestro",
        "commit",
        "-q",
        "-m",
        message,
    ]);
}

/// Package a completed micro-project into a new standalone release.
pub fn persist_release(
    root: &Path,
    demand: &str,
    deliverables: &[String],
) -> std::io::Result<ReleaseRecord> {
    let releases = root.join("maestro").join("releases");
    std::fs::create_dir_all(&releases)?;

    let next = release_count(&releases) + 1;
    let id = format!("r{next:03}");
    let version = format!("0.1.{next}");
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .to_string();

    let dir = releases.join(&id);
    std::fs::create_dir_all(&dir)?;

    let mut manifest = format!(
        "# Release {version}\n\n- id: {id}\n- created: {created_at}\n- demand: {demand}\n\n## Deliverables\n"
    );
    for item in deliverables {
        manifest.push_str(&format!("- {item}\n"));
    }
    std::fs::write(dir.join("manifest.md"), &manifest)?;

    // A runnable demo artifact Product Mode executes to "present" the release.
    let mut demo = String::from("#!/bin/sh\n");
    demo.push_str(&format!(
        "echo \"Maestro release {version} \u{2014} demo for: {demand}\"\n"
    ));
    for item in deliverables {
        let safe = item.replace('"', "'");
        demo.push_str(&format!("echo \"- {safe}\"\n"));
    }
    demo.push_str("echo \"demo complete.\"\n");
    std::fs::write(dir.join("demo.sh"), &demo)?;

    git_commit(&dir, &format!("release {version}: {demand}"));

    Ok(ReleaseRecord {
        id,
        version,
        changelog: format!("Micro-project for: {demand}"),
        created_at,
    })
}

/// Read all persisted releases (newest last), parsed from their manifests.
pub fn list_releases(root: &Path) -> Vec<ReleaseRecord> {
    let releases = root.join("maestro").join("releases");
    let mut dirs: Vec<PathBuf> = match std::fs::read_dir(&releases) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect(),
        Err(_) => return Vec::new(),
    };
    dirs.sort();

    let mut out = Vec::new();
    for dir in dirs {
        let Ok(body) = std::fs::read_to_string(dir.join("manifest.md")) else {
            continue;
        };
        let mut version = String::new();
        let mut id = String::new();
        let mut created_at = String::new();
        let mut demand = String::new();
        for line in body.lines() {
            if let Some(v) = line.strip_prefix("# Release ") {
                version = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("- id: ") {
                id = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("- created: ") {
                created_at = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("- demand: ") {
                demand = v.trim().to_string();
            }
        }
        if id.is_empty() {
            id = dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
        }
        out.push(ReleaseRecord {
            id,
            version,
            changelog: format!("Micro-project for: {demand}"),
            created_at,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "maestro-persist-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn persists_a_manifest_and_increments_versions() {
        let root = temp_root("persist");
        let first = persist_release(&root, "build a cli", &["Engineer: done".to_string()]).unwrap();
        assert_eq!(first.id, "r001");
        assert!(root.join("maestro/releases/r001/manifest.md").exists());
        let body = std::fs::read_to_string(root.join("maestro/releases/r001/manifest.md")).unwrap();
        assert!(body.contains("build a cli"));
        assert!(body.contains("Engineer: done"));

        let second = persist_release(&root, "another", &[]).unwrap();
        assert_eq!(second.id, "r002");
        assert_ne!(first.version, second.version);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn writes_a_runnable_demo_and_lists_releases() {
        let root = temp_root("list");
        persist_release(&root, "ship it", &["QA: done".to_string()]).unwrap();
        assert!(root.join("maestro/releases/r001/demo.sh").exists());

        let releases = list_releases(&root);
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].id, "r001");
        assert_eq!(releases[0].version, "0.1.1");
        assert!(releases[0].changelog.contains("ship it"));
        std::fs::remove_dir_all(&root).ok();
    }
}
