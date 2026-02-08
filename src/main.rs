//! prompt-dialog: Frameless AI prompt dialog for OpenCode
//!
//! A lightweight Slint GUI that sends prompts to a running OpenCode instance.

mod server;

use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::Parser;

slint::include_modules!();

/// Frameless AI prompt dialog for OpenCode
#[derive(Parser, Debug)]
#[command(name = "prompt-dialog", version, about)]
struct Cli {
    /// OpenCode server port (auto-discovers if not specified)
    #[arg(short, long)]
    port: Option<u16>,

    /// Enable debug logging
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Prompt parameters as key=value pairs, usable as @key placeholders
    /// Example: --param path=/src/main.rs --param selection="some code"
    #[arg(long = "param", value_name = "KEY=VALUE")]
    params: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    // Parse --param key=value pairs into a HashMap
    let params = parse_params(&cli.params);
    if cli.debug && !params.is_empty() {
        eprintln!(
            "Params: {}",
            params
                .iter()
                .map(|(k, v)| format!("@{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Create tokio runtime for async HTTP calls
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;

    // Discover or connect to the OpenCode server
    let discovery_result = rt.block_on(discover_and_connect(&cwd, cli.port, cli.debug));

    // Create the Slint dialog
    let dialog = PromptDialog::new().context("Failed to create dialog window")?;

    // Center window on screen
    center_window(&dialog, cli.debug);

    // Show available placeholders in the UI (built-ins + user params)
    {
        let mut hints: Vec<String> = vec!["@clipboard".to_string()];
        let mut param_keys: Vec<&String> = params.keys().collect();
        param_keys.sort();
        for k in param_keys {
            hints.push(format!("@{}", k));
        }
        dialog.set_placeholder_hint(hints.join(" ").into());
    }

    // Set connection state based on discovery
    match &discovery_result {
        Ok(server) => {
            dialog.set_connected(true);
            if cli.debug {
                eprintln!(
                    "Connected to OpenCode server on port {} (cwd: {})",
                    server.port,
                    server.cwd.display()
                );
            }
        }
        Err(e) => {
            dialog.set_connected(false);
            dialog.set_error_text(format!("{}", e).into());
            if cli.debug {
                eprintln!("Server discovery failed: {}", e);
            }
        }
    }

    // Collect all known placeholder names (built-ins + user params)
    let all_placeholders: Vec<String> = {
        let mut names = vec!["clipboard".to_string()];
        let mut param_keys: Vec<String> = params.keys().cloned().collect();
        param_keys.sort();
        names.append(&mut param_keys);
        names
    };

    // Wire up text-changed callback for autocomplete + highlighting
    {
        let weak = dialog.as_weak();
        let placeholders = all_placeholders.clone();

        dialog.on_text_changed(move |text| {
            let text = text.to_string();
            if let Some(d) = weak.upgrade() {
                // Generate highlight overlay text
                let highlight = build_highlight_text(&text, &placeholders);
                d.set_highlight_text(highlight.into());

                // Find autocomplete suggestion
                let (suggestion, visible) = find_autocomplete(&text, &placeholders);
                d.set_autocomplete_suggestion(suggestion.into());
                d.set_autocomplete_visible(visible);
            }
        });
    }

    // Wire up accept-autocomplete callback
    {
        let weak = dialog.as_weak();
        let placeholders = all_placeholders.clone();

        dialog.on_accept_autocomplete(move || {
            if let Some(d) = weak.upgrade() {
                let text = d.get_input_text().to_string();
                let completed = apply_autocomplete(&text, &placeholders);
                d.set_input_text(completed.into());
                d.invoke_move_cursor_to_end();

                // Trigger highlight update
                let highlight = build_highlight_text(d.get_input_text().as_ref(), &placeholders);
                d.set_highlight_text(highlight.into());
                d.set_autocomplete_visible(false);
            }
        });
    }

    // Wire up the submit callback
    let client = discovery_result
        .as_ref()
        .ok()
        .map(|s| server::Client::new(s.port));

    {
        let weak = dialog.as_weak();
        let rt_handle = rt.handle().clone();

        dialog.on_submit(move |text| {
            let text = text.to_string();
            if text.is_empty() {
                return;
            }

            // Expand @placeholders with param values
            let expanded = expand_placeholders(&text, &params);

            if let Some(ref client) = client {
                let client = client.clone();
                let weak = weak.clone();

                rt_handle.spawn(async move {
                    match client.send_prompt(&expanded).await {
                        Ok(()) => {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(d) = weak.upgrade() {
                                    let _ = d.hide();
                                }
                                slint::quit_event_loop().ok();
                            });
                        }
                        Err(e) => {
                            let err_msg = format!("Send failed: {}", e);
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(d) = weak.upgrade() {
                                    d.set_error_text(err_msg.into());
                                }
                            });
                        }
                    }
                });
            }
        });
    }

    // Wire up the dismiss callback
    {
        let weak = dialog.as_weak();
        dialog.on_dismiss(move || {
            if let Some(d) = weak.upgrade() {
                let _ = d.hide();
            }
            slint::quit_event_loop().ok();
        });
    }

    // Run the Slint event loop
    dialog.run().context("Slint event loop failed")?;

    Ok(())
}

/// Center the dialog window on the primary monitor
fn center_window(dialog: &PromptDialog, debug: bool) {
    let window = dialog.window();
    let scale = window.scale_factor();
    let win_width = 680.0_f32;
    let win_height = 240.0_f32;

    // Try to get screen size via winit backend
    #[cfg(not(target_os = "android"))]
    {
        use i_slint_backend_winit::WinitWindowAccessor;
        window.with_winit_window(|winit_win| {
            if let Some(monitor) = winit_win
                .current_monitor()
                .or_else(|| winit_win.primary_monitor())
            {
                let screen_size = monitor.size();
                let screen_w = screen_size.width as f32;
                let screen_h = screen_size.height as f32;

                let x = (screen_w - win_width * scale) / 2.0;
                let y = (screen_h - win_height * scale) / 3.0; // Slightly above center

                window.set_position(slint::PhysicalPosition::new(x as i32, y as i32));

                if debug {
                    eprintln!(
                        "Screen: {}x{}, scale: {}, window pos: ({}, {})",
                        screen_w, screen_h, scale, x as i32, y as i32
                    );
                }
            }
        });
    }
}

/// Parse --param key=value pairs into a HashMap
fn parse_params(raw: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for param in raw {
        if let Some((key, value)) = param.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

/// Expand @placeholder tokens in text with param values and built-in special tokens.
///
/// Built-in tokens (always available):
///   - `@clipboard` — current system clipboard text content
///
/// User params from `--param key=value` are expanded as `@key`.
/// Matches the longest key first to avoid partial replacements.
fn expand_placeholders(text: &str, params: &HashMap<String, String>) -> String {
    let mut result = text.to_string();

    // Expand built-in special tokens first
    result = expand_builtins(&result);

    // Expand user params
    if !params.is_empty() {
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

        for key in keys {
            let placeholder = format!("@{}", key);
            if let Some(value) = params.get(key) {
                result = result.replace(&placeholder, value);
            }
        }
    }

    result
}

/// Expand built-in special tokens like @clipboard
fn expand_builtins(text: &str) -> String {
    let mut result = text.to_string();

    if result.contains("@clipboard") {
        let clipboard_text = read_clipboard().unwrap_or_default();
        result = result.replace("@clipboard", &clipboard_text);
    }

    result
}

/// Read text content from the system clipboard
fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok())
        .filter(|s| !s.is_empty())
}

/// Build a highlight overlay text where only @placeholder tokens are visible
/// and all other characters are replaced with spaces (preserving positions).
///
/// This works because the overlay Text uses the same font/size/wrap as the input,
/// so characters at the same positions line up exactly.
fn build_highlight_text(text: &str, placeholders: &[String]) -> String {
    let mut mask = vec![false; text.len()];

    // Mark character positions that are part of @placeholder tokens
    for name in placeholders {
        let token = format!("@{}", name);
        let mut search_from = 0;
        while let Some(pos) = text[search_from..].find(&token) {
            let abs_pos = search_from + pos;
            let end = abs_pos + token.len();
            // Check that the token ends at a word boundary
            let at_end = end >= text.len()
                || !text.as_bytes()[end].is_ascii_alphanumeric() && text.as_bytes()[end] != b'_';
            if at_end {
                for item in mask.iter_mut().take(end).skip(abs_pos) {
                    *item = true;
                }
            }
            search_from = abs_pos + 1;
        }
    }

    // Build overlay: keep @token chars, replace everything else with spaces
    text.char_indices()
        .map(|(i, c)| {
            if i < mask.len() && mask[i] {
                c
            } else if c == '\n' {
                '\n' // Preserve newlines for wrap alignment
            } else {
                ' '
            }
        })
        .collect()
}

/// Find autocomplete suggestion for the current @partial token being typed.
///
/// Looks for an `@` followed by partial text at the end of the input (or before
/// trailing whitespace), and returns the best matching placeholder name.
fn find_autocomplete(text: &str, placeholders: &[String]) -> (String, bool) {
    // Find the last '@' that starts an incomplete token
    if let Some(at_pos) = text.rfind('@') {
        let after_at = &text[at_pos + 1..];

        // The partial must be at the end (no spaces after it)
        if after_at.contains(' ') || after_at.contains('\n') {
            return (String::new(), false);
        }

        let partial = after_at.to_lowercase();

        // Don't suggest if the token already exactly matches a placeholder
        if placeholders.iter().any(|p| p == &partial) {
            return (String::new(), false);
        }

        // Find matching placeholders (prefix match)
        if !partial.is_empty() {
            let matches: Vec<&String> = placeholders
                .iter()
                .filter(|p| p.to_lowercase().starts_with(&partial))
                .collect();

            if let Some(best) = matches.first() {
                return (format!("@{}", best), true);
            }
        } else {
            // Just typed '@', show first placeholder
            if let Some(first) = placeholders.first() {
                return (format!("@{}", first), true);
            }
        }
    }

    (String::new(), false)
}

/// Apply the autocomplete: replace the current @partial token with the full suggestion.
fn apply_autocomplete(text: &str, placeholders: &[String]) -> String {
    if let Some(at_pos) = text.rfind('@') {
        let after_at = &text[at_pos + 1..];

        if after_at.contains(' ') || after_at.contains('\n') {
            return text.to_string();
        }

        let partial = after_at.to_lowercase();
        let matches: Vec<&String> = if partial.is_empty() {
            placeholders.iter().collect()
        } else {
            placeholders
                .iter()
                .filter(|p| p.to_lowercase().starts_with(&partial))
                .collect()
        };

        if let Some(best) = matches.first() {
            let mut result = text[..at_pos].to_string();
            result.push_str(&format!("@{} ", best));
            return result;
        }
    }

    text.to_string()
}

/// Discover and connect to an OpenCode server
async fn discover_and_connect(
    cwd: &std::path::Path,
    port: Option<u16>,
    debug: bool,
) -> Result<server::Server> {
    if debug {
        eprintln!("Discovering OpenCode server (cwd: {})...", cwd.display());
    }

    server::discover_server(cwd, port).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params() {
        let raw = vec![
            "path=/src/main.rs".to_string(),
            "selection=some code".to_string(),
        ];
        let params = parse_params(&raw);
        assert_eq!(params.get("path").unwrap(), "/src/main.rs");
        assert_eq!(params.get("selection").unwrap(), "some code");
    }

    #[test]
    fn test_parse_params_empty() {
        let params = parse_params(&[]);
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_params_invalid() {
        let raw = vec!["noequals".to_string(), "=nokey".to_string()];
        let params = parse_params(&raw);
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_params_value_with_equals() {
        let raw = vec!["query=a=b=c".to_string()];
        let params = parse_params(&raw);
        assert_eq!(params.get("query").unwrap(), "a=b=c");
    }

    #[test]
    fn test_expand_placeholders() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/src/main.rs".to_string());
        params.insert("selection".to_string(), "fn main()".to_string());

        let text = "Fix the bug in @path near @selection";
        let result = expand_placeholders(text, &params);
        assert_eq!(result, "Fix the bug in /src/main.rs near fn main()");
    }

    #[test]
    fn test_expand_placeholders_no_match() {
        let params = HashMap::new();
        let text = "No placeholders here";
        let result = expand_placeholders(text, &params);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_expand_placeholders_multiple_occurrences() {
        let mut params = HashMap::new();
        params.insert("file".to_string(), "test.rs".to_string());

        let text = "Compare @file with @file";
        let result = expand_placeholders(text, &params);
        assert_eq!(result, "Compare test.rs with test.rs");
    }

    #[test]
    fn test_expand_longest_key_first() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "short".to_string());
        params.insert("pathname".to_string(), "long".to_string());

        let text = "Use @pathname and @path";
        let result = expand_placeholders(text, &params);
        assert_eq!(result, "Use long and short");
    }

    #[test]
    fn test_build_highlight_text() {
        let placeholders = vec!["path".to_string(), "clipboard".to_string()];
        let text = "Fix @path and @clipboard now";
        let result = build_highlight_text(text, &placeholders);
        // @path and @clipboard should be visible, rest spaces
        assert_eq!(result, "    @path     @clipboard    ");
    }

    #[test]
    fn test_build_highlight_preserves_newlines() {
        let placeholders = vec!["file".to_string()];
        let text = "hello\n@file";
        let result = build_highlight_text(text, &placeholders);
        assert_eq!(result, "     \n@file");
    }

    #[test]
    fn test_find_autocomplete_partial() {
        let placeholders = vec!["clipboard".to_string(), "path".to_string()];
        let (suggestion, visible) = find_autocomplete("hello @cl", &placeholders);
        assert!(visible);
        assert_eq!(suggestion, "@clipboard");
    }

    #[test]
    fn test_find_autocomplete_at_only() {
        let placeholders = vec!["clipboard".to_string(), "path".to_string()];
        let (suggestion, visible) = find_autocomplete("hello @", &placeholders);
        assert!(visible);
        assert_eq!(suggestion, "@clipboard");
    }

    #[test]
    fn test_find_autocomplete_exact_match_no_suggest() {
        let placeholders = vec!["clipboard".to_string()];
        let (_suggestion, visible) = find_autocomplete("hello @clipboard", &placeholders);
        assert!(!visible);
    }

    #[test]
    fn test_find_autocomplete_no_at() {
        let placeholders = vec!["clipboard".to_string()];
        let (_suggestion, visible) = find_autocomplete("hello world", &placeholders);
        assert!(!visible);
    }

    #[test]
    fn test_apply_autocomplete() {
        let placeholders = vec!["clipboard".to_string(), "path".to_string()];
        let result = apply_autocomplete("Fix @cl", &placeholders);
        assert_eq!(result, "Fix @clipboard ");
    }

    #[test]
    fn test_apply_autocomplete_at_only() {
        let placeholders = vec!["clipboard".to_string()];
        let result = apply_autocomplete("Fix @", &placeholders);
        assert_eq!(result, "Fix @clipboard ");
    }

    #[test]
    fn test_expand_clipboard_token() {
        // @clipboard expands to whatever is on the system clipboard.
        // We can't control clipboard in CI, so just verify it doesn't panic
        // and the token is consumed (replaced with something).
        let params = HashMap::new();
        let text = "Paste: @clipboard";
        let result = expand_placeholders(text, &params);
        assert!(!result.contains("@clipboard"));
    }

    #[test]
    fn test_expand_clipboard_with_params() {
        let mut params = HashMap::new();
        params.insert("file".to_string(), "main.rs".to_string());

        let text = "Fix @file using @clipboard";
        let result = expand_placeholders(text, &params);
        assert!(!result.contains("@clipboard"));
        assert!(result.contains("main.rs"));
    }
}
