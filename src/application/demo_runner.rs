//! Product Mode demo runner (TASK — Phase 5 / W5).
//!
//! Executes a persisted release's `demo.sh` artifact as a subprocess and streams
//! its output line by line, returning the exit code. Product Mode uses this to
//! "present" a shipped release live. Locating and running the artifact is I/O, so
//! this sits at the application boundary alongside `persistence`.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Find the release directory whose manifest declares `version`.
fn find_release_dir(root: &Path, version: &str) -> Option<PathBuf> {
    let releases = root.join("maestro").join("releases");
    for entry in std::fs::read_dir(&releases).ok()?.filter_map(|e| e.ok()) {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        if let Ok(body) = std::fs::read_to_string(dir.join("manifest.md")) {
            let matches = body.lines().any(|l| {
                l.strip_prefix("# Release ")
                    .map(|v| v.trim() == version)
                    .unwrap_or(false)
            });
            if matches {
                return Some(dir);
            }
        }
    }
    None
}

/// Run the demo for the release tagged `version`, streaming each output line to
/// `on_output(stream, line)` (`stream` is `stdout` or `stderr`). Returns the exit
/// code, or `127` when the release has no demo artifact.
pub fn run_demo(
    root: &Path,
    version: &str,
    mut on_output: impl FnMut(&str, &str) -> std::io::Result<()>,
) -> std::io::Result<i32> {
    let dir = find_release_dir(root, version).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("release {version} not found"),
        )
    })?;

    let script = dir.join("demo.sh");
    if !script.exists() {
        on_output("stderr", "no demo artifact for this release")?;
        return Ok(127);
    }

    let mut child = Command::new("sh")
        .arg(&script)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines() {
            on_output("stdout", &line?)?;
        }
    }
    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr).lines() {
            on_output("stderr", &line?)?;
        }
    }

    Ok(child.wait()?.code().unwrap_or(-1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::persistence::persist_release;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "maestro-demo-{tag}-{}-{}",
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
    fn runs_a_release_demo_and_streams_output() {
        let root = temp_root("run");
        let record =
            persist_release(&root, "build a cli", &["Engineer: done".to_string()]).unwrap();

        let mut lines = Vec::new();
        let code = run_demo(&root, &record.version, |stream, line| {
            lines.push(format!("{stream}:{line}"));
            Ok(())
        })
        .unwrap();

        assert_eq!(code, 0);
        assert!(lines.iter().any(|l| l.contains("demo for: build a cli")));
        assert!(lines.iter().any(|l| l.contains("Engineer: done")));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn missing_release_is_an_error() {
        let root = temp_root("missing");
        std::fs::create_dir_all(root.join("maestro/releases")).unwrap();
        let result = run_demo(&root, "9.9.9", |_, _| Ok(()));
        assert!(result.is_err());
        std::fs::remove_dir_all(&root).ok();
    }
}
