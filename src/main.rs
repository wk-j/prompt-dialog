//! prompt-dialog: Frameless AI prompt dialog for OpenCode
//!
//! A lightweight Slint GUI that sends prompts to a running OpenCode instance.

mod server;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    // Create tokio runtime for async HTTP calls
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;

    // Discover or connect to the OpenCode server
    let discovery_result = rt.block_on(discover_and_connect(&cwd, cli.port, cli.debug));

    // Create the Slint dialog
    let dialog = PromptDialog::new().context("Failed to create dialog window")?;

    // Center window on screen
    center_window(&dialog, cli.debug);

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

            if let Some(ref client) = client {
                let client = client.clone();
                let weak = weak.clone();

                rt_handle.spawn(async move {
                    match client.send_prompt(&text).await {
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
