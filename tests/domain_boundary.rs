//! Architecture boundary guard.
//!
//! Enforces the hexagonal rule: the `domain` layer must not depend on
//! `infrastructure` or `presentation`. This is a static check over the source
//! tree so violations fail CI as an ordinary test.

use std::fs;
use std::path::Path;

/// Recursively collect the contents of every `.rs` file under `dir`.
fn read_rs_files(dir: &Path, out: &mut Vec<(String, String)>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            read_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                out.push((path.display().to_string(), content));
            }
        }
    }
}

#[test]
fn domain_does_not_depend_on_outer_layers() {
    let domain_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domain");
    let mut files = Vec::new();
    read_rs_files(&domain_dir, &mut files);

    assert!(
        !files.is_empty(),
        "expected at least one .rs file under src/domain"
    );

    let forbidden = ["crate::infrastructure", "crate::presentation"];
    for (path, content) in &files {
        // Only inspect real code lines; doc/line comments may legitimately mention
        // the outer layers (e.g. to state the boundary rule).
        let code: String = content
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for needle in &forbidden {
            assert!(
                !code.contains(needle),
                "domain purity violation: {path} references {needle}"
            );
        }
    }
}
