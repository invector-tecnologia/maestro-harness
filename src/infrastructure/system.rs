//! System environment and health diagnostics.

use std::process::Command;

/// Aggregated system health metrics.
pub struct SystemHealth {
    pub git_available: bool,
    pub nim_available: bool,
    pub gpu_detected: bool,
    pub gpu_info: Option<String>,
}

/// Run non-blocking checks to determine system health.
pub fn check_system() -> SystemHealth {
    let git_available = Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let nim_available = Command::new("nim")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let (gpu_detected, gpu_info) = check_gpu();

    SystemHealth {
        git_available,
        nim_available,
        gpu_detected,
        gpu_info,
    }
}

fn check_gpu() -> (bool, Option<String>) {
    // Attempt nvidia-smi (Linux/Windows)
    if let Ok(output) = Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
    {
        if output.status.success() {
            let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !info.is_empty() {
                return (true, Some(info));
            }
        }
    }

    // Attempt Apple Silicon (macOS)
    if cfg!(target_os = "macos") {
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if info.contains("Apple M") {
                    return (true, Some(info));
                }
            }
        }
    }

    (false, None)
}
