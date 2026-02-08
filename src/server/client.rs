//! HTTP client for OpenCode server API
//!
//! Communicates with the OpenCode server via HTTP/JSON.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// HTTP client for OpenCode server
#[derive(Debug, Clone)]
pub struct Client {
    port: u16,
    http: reqwest::Client,
}

/// Response from /path endpoint
#[derive(Debug, Deserialize)]
pub struct PathResponse {
    pub directory: Option<String>,
    pub worktree: Option<String>,
}

/// TUI publish request body
#[derive(Debug, Serialize)]
struct TuiPublishRequest {
    #[serde(rename = "type")]
    event_type: String,
    properties: serde_json::Value,
}

impl Client {
    /// Create a new client for the given port
    pub fn new(port: u16) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self { port, http }
    }

    /// Base URL for the server
    fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// GET /path - Get server working directory
    pub async fn get_path(&self) -> Result<PathResponse> {
        let url = format!("{}/path", self.base_url());
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to connect to OpenCode server")?;

        response
            .json()
            .await
            .context("Failed to parse path response")
    }

    /// POST /tui/publish - Append text to the TUI prompt
    async fn tui_append_prompt(&self, text: &str) -> Result<()> {
        let url = format!("{}/tui/publish", self.base_url());
        let request = TuiPublishRequest {
            event_type: "tui.prompt.append".to_string(),
            properties: serde_json::json!({ "text": text }),
        };

        self.http
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to append prompt")?;

        Ok(())
    }

    /// POST /tui/publish - Execute a TUI command
    async fn tui_execute_command(&self, command: &str) -> Result<()> {
        let url = format!("{}/tui/publish", self.base_url());
        let request = TuiPublishRequest {
            event_type: "tui.command.execute".to_string(),
            properties: serde_json::json!({ "command": command }),
        };

        self.http
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to execute command")?;

        Ok(())
    }

    /// Send a prompt to OpenCode: append text then submit
    pub async fn send_prompt(&self, text: &str) -> Result<()> {
        self.tui_append_prompt(text)
            .await
            .context("Failed to append prompt text")?;

        self.tui_execute_command("prompt.submit")
            .await
            .context("Failed to submit prompt")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url() {
        let client = Client::new(12345);
        assert_eq!(client.base_url(), "http://localhost:12345");
    }

    #[test]
    fn test_base_url_default_port() {
        let client = Client::new(4096);
        assert_eq!(client.base_url(), "http://localhost:4096");
    }
}
