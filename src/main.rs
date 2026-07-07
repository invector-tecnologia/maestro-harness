//! Maestro CLI entry point.
//!
//! Wires structured logging (`tracing`) and argument parsing (`clap`), then
//! dispatches to the presentation layer. `anyhow` is used only here, at the
//! process boundary; everything below returns typed errors.

use clap::Parser;

use maestro::presentation::cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    maestro::presentation::cli::dispatch(cli)
}

/// Initialize the global `tracing` subscriber. Verbosity is controlled by the
/// `RUST_LOG` environment variable (defaults to `info`). Logs go to stderr so
/// that stdout stays reserved for the IPC protocol stream (`maestro run`).
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();
}

// Keep `Command` referenced so the module wiring is exercised at compile time.
#[allow(dead_code)]
fn _command_type_anchor(_c: &Command) {}
