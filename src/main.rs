//! CLI tool for interacting with GTK4 applications via debug socket.
//!
//! Usage:
//!   gtk-debug list              # List running GTK debug servers
//!   gtk-debug dump [PID]        # Dump widget tree (auto-detect if single server)
//!   gtk-debug dump-json [PID]   # Dump widget tree as JSON
//!   gtk-debug click LABEL [PID] # Click a button by label
//!   gtk-debug input FIELD VALUE [PID] # Set entry text by placeholder
//!   gtk-debug submit [PID]      # Activate focused widget
//!   gtk-debug ping [PID]        # Check if app is responding

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use clap::{Parser, Subcommand};
use gtk_layout_inspector::server::client;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "gtk-debug")]
#[command(about = "Interact with GTK4 applications via debug socket")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List running GTK debug servers
    List,
    /// Dump the widget tree
    Dump {
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Dump the widget tree as JSON
    DumpJson {
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Click a button by label
    Click {
        /// Button label text to match
        label: String,
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Set text in an entry field
    Input {
        /// Entry placeholder text to match
        field: String,
        /// Value to set
        value: String,
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Activate the focused widget (like pressing Enter)
    Submit {
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Check if an app is responding
    Ping {
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Send a key press event (e.g., "t", "Return", "Escape")
    Key {
        /// Key name (single char or GTK key name like "Return", "Escape")
        key: String,
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
    /// Take a screenshot and save to file
    Screenshot {
        /// Output file path (WebP format)
        #[arg(default_value = "screenshot.webp")]
        output: PathBuf,
        /// Process ID (auto-detect if only one server running)
        pid: Option<u32>,
    },
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_servers(),
        Commands::Dump { pid } => dump_tree(pid),
        Commands::DumpJson { pid } => dump_tree_json(pid),
        Commands::Click { label, pid } => click_button(label, pid),
        Commands::Input { field, value, pid } => input_field(field, value, pid),
        Commands::Submit { pid } => submit_focused(pid),
        Commands::Ping { pid } => ping_server(pid),
        Commands::Key { key, pid } => send_key(key, pid),
        Commands::Screenshot { output, pid } => take_screenshot(output, pid),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn list_servers() -> ExitCode {
    let servers = client::find_servers();
    if servers.is_empty() {
        println!("No GTK debug servers running");
    } else {
        println!("Running GTK debug servers:");
        for path in servers {
            if let Some(pid) = extract_pid(&path) {
                println!("  PID {}: {}", pid, path.display());
            } else {
                println!("  {}", path.display());
            }
        }
    }
    ExitCode::SUCCESS
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn dump_tree(pid: Option<u32>) -> ExitCode {
    dump_layout(pid, |path| client::dump(path))
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn dump_tree_json(pid: Option<u32>) -> ExitCode {
    dump_layout(pid, |path| client::dump_json(path))
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn dump_layout<T, E, F>(pid: Option<u32>, fetch: F) -> ExitCode
where
    T: std::fmt::Display,
    E: std::fmt::Display,
    F: FnOnce(&std::path::Path) -> Result<T, E>,
{
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match fetch(&socket) {
        Ok(layout) => {
            println!("{}", layout);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn click_button(label: String, pid: Option<u32>) -> ExitCode {
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match client::click(&socket, &label) {
        Ok(()) => {
            println!("Clicked button '{}'", label);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn input_field(field: String, value: String, pid: Option<u32>) -> ExitCode {
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match client::input(&socket, &field, &value) {
        Ok(()) => {
            println!("Set '{}' to '{}'", field, value);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn submit_focused(pid: Option<u32>) -> ExitCode {
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match client::submit(&socket) {
        Ok(()) => {
            println!("Submitted");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn ping_server(pid: Option<u32>) -> ExitCode {
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match client::ping(&socket) {
        Ok(()) => {
            println!("Pong");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn send_key(key: String, pid: Option<u32>) -> ExitCode {
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match client::key_press(&socket, &key) {
        Ok(()) => {
            println!("Sent key '{}'", key);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn take_screenshot(output: PathBuf, pid: Option<u32>) -> ExitCode {
    let socket = match get_socket(pid) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };
    match client::screenshot_to_file(&socket, &output) {
        Ok(()) => {
            println!("Screenshot saved to {}", output.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn get_socket(pid: Option<u32>) -> Result<PathBuf, String> {
    select_socket(pid, client::find_servers(), |path| path.exists())
}

fn select_socket(
    pid: Option<u32>,
    servers: Vec<PathBuf>,
    exists: impl Fn(&PathBuf) -> bool,
) -> Result<PathBuf, String> {
    if let Some(pid) = pid {
        let path = socket_path_for_pid(pid);
        if exists(&path) {
            Ok(path)
        } else {
            Err(format!("No debug server for PID {}", pid))
        }
    } else {
        match servers.len() {
            0 => Err("No GTK debug servers running".to_string()),
            1 => Ok(servers.into_iter().next().unwrap()),
            n => Err(format!(
                "{} servers running, specify PID: {:?}",
                n,
                servers
                    .iter()
                    .filter_map(|p| extract_pid(p))
                    .collect::<Vec<_>>()
            )),
        }
    }
}

fn socket_path_for_pid(pid: u32) -> PathBuf {
    PathBuf::from(format!("/tmp/gtk-debug-{}.sock", pid))
}

fn extract_pid(path: &std::path::Path) -> Option<u32> {
    path.file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("gtk-debug-"))
        .and_then(|s| s.strip_suffix(".sock"))
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn socket_path_for_pid_uses_tmp_convention() {
        assert_eq!(
            socket_path_for_pid(1234),
            PathBuf::from("/tmp/gtk-debug-1234.sock")
        );
    }

    #[test]
    fn extract_pid_accepts_only_debug_socket_names() {
        assert_eq!(
            extract_pid(PathBuf::from("/tmp/gtk-debug-1234.sock").as_path()),
            Some(1234)
        );
        assert_eq!(
            extract_pid(PathBuf::from("gtk-debug-9.sock").as_path()),
            Some(9)
        );
        assert_eq!(
            extract_pid(PathBuf::from("/tmp/gtk-debug-nope.sock").as_path()),
            None
        );
        assert_eq!(
            extract_pid(PathBuf::from("/tmp/other-1234.sock").as_path()),
            None
        );
        assert_eq!(
            extract_pid(PathBuf::from("/tmp/gtk-debug-1234.txt").as_path()),
            None
        );
    }

    #[test]
    fn select_socket_uses_explicit_pid_when_socket_exists() {
        let selected = select_socket(Some(77), vec![], |path| {
            path == &PathBuf::from("/tmp/gtk-debug-77.sock")
        })
        .expect("pid socket");

        assert_eq!(selected, PathBuf::from("/tmp/gtk-debug-77.sock"));
    }

    #[test]
    fn select_socket_reports_missing_explicit_pid() {
        let error = select_socket(Some(77), vec![], |_| false).expect_err("missing pid");

        assert_eq!(error, "No debug server for PID 77");
    }

    #[test]
    fn select_socket_auto_detects_zero_one_or_many_servers() {
        assert_eq!(
            select_socket(None, vec![], |_| true).expect_err("no servers"),
            "No GTK debug servers running"
        );

        let only = PathBuf::from("/tmp/gtk-debug-10.sock");
        assert_eq!(
            select_socket(None, vec![only.clone()], |_| true).expect("one server"),
            only
        );

        let error = select_socket(
            None,
            vec![
                PathBuf::from("/tmp/gtk-debug-10.sock"),
                PathBuf::from("/tmp/gtk-debug-20.sock"),
            ],
            |_| true,
        )
        .expect_err("many servers");

        assert_eq!(error, "2 servers running, specify PID: [10, 20]");
    }

    #[test]
    fn clap_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
