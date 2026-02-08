//! Server discovery for OpenCode processes
//!
//! Finds running OpenCode servers by scanning processes and validating via HTTP.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use sysinfo::System;

/// A discovered OpenCode server
#[derive(Debug, Clone)]
pub struct Server {
    /// Process ID
    pub pid: u32,
    /// HTTP server port
    pub port: u16,
    /// Working directory of the server
    pub cwd: PathBuf,
}

/// Find OpenCode processes with --port flag
fn find_opencode_processes() -> Vec<(u32, String)> {
    let system = System::new_all();
    let mut processes = Vec::new();

    for (pid, process) in system.processes() {
        let cmd_str: String = process
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        if cmd_str.contains("opencode") && cmd_str.contains("--port") {
            processes.push((pid.as_u32(), cmd_str));
        }
    }

    processes
}

/// Extract port number from command line arguments
fn extract_port_from_cmdline(cmdline: &str) -> Option<u16> {
    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "--port" {
            if let Some(port_str) = parts.get(i + 1) {
                if let Ok(port) = port_str.parse() {
                    return Some(port);
                }
            }
        } else if let Some(port_str) = part.strip_prefix("--port=") {
            if let Ok(port) = port_str.parse() {
                return Some(port);
            }
        }
    }
    None
}

/// Validate a port is an OpenCode server and get its working directory
async fn validate_server(port: u16) -> Result<Server> {
    let client = super::client::Client::new(port);
    let path_response = client
        .get_path()
        .await
        .context("Failed to connect to OpenCode server")?;

    let cwd = path_response
        .directory
        .or(path_response.worktree)
        .ok_or_else(|| anyhow!("Server did not return a working directory"))?;

    Ok(Server {
        pid: 0,
        port,
        cwd: PathBuf::from(cwd),
    })
}

/// Discover an OpenCode server for the given working directory
///
/// If `port` is specified, validates and uses that port directly.
/// Otherwise, scans for OpenCode processes and finds one matching the cwd.
pub async fn discover_server(cwd: &Path, port: Option<u16>) -> Result<Server> {
    // If port is specified, use it directly
    if let Some(p) = port {
        return validate_server(p)
            .await
            .context(format!("No OpenCode server responding on port {}", p));
    }

    // Find all OpenCode processes
    let processes = find_opencode_processes();
    if processes.is_empty() {
        return Err(anyhow!(
            "No OpenCode processes found. Start OpenCode with: opencode --port 8080"
        ));
    }

    // Try each process to find one matching our cwd
    let mut last_error = None;
    for (pid, cmdline) in processes {
        if let Some(port) = extract_port_from_cmdline(&cmdline) {
            match validate_server(port).await {
                Ok(mut server) => {
                    server.pid = pid;

                    let server_cwd = server.cwd.canonicalize().unwrap_or(server.cwd.clone());
                    let our_cwd = cwd.canonicalize().unwrap_or(cwd.to_path_buf());

                    if our_cwd.starts_with(&server_cwd) || server_cwd.starts_with(&our_cwd) {
                        return Ok(server);
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| anyhow!("No OpenCode server found for directory: {}", cwd.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_port_space_separated() {
        assert_eq!(
            extract_port_from_cmdline("opencode --port 12345"),
            Some(12345)
        );
    }

    #[test]
    fn test_extract_port_with_other_flags() {
        assert_eq!(
            extract_port_from_cmdline("node opencode.js --port 8080 --other"),
            Some(8080)
        );
    }

    #[test]
    fn test_extract_port_equals_syntax() {
        assert_eq!(
            extract_port_from_cmdline("opencode --port=9999"),
            Some(9999)
        );
    }

    #[test]
    fn test_extract_port_missing() {
        assert_eq!(extract_port_from_cmdline("opencode --other"), None);
    }

    #[test]
    fn test_extract_port_no_value() {
        assert_eq!(extract_port_from_cmdline("opencode --port"), None);
    }

    #[test]
    fn test_extract_port_invalid_value() {
        assert_eq!(extract_port_from_cmdline("opencode --port abc"), None);
    }
}
