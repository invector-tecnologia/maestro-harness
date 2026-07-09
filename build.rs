//! Build script — bake git commit and build date into the binary.

use std::process::Command;

fn main() {
    // Git short hash
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=MAESTRO_COMMIT={commit}");

    // Build date (UTC, YYYY-MM-DD)
    let date = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=MAESTRO_BUILD_DATE={date}");

    // Only re-run when HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
