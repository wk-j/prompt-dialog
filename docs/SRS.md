# Software Requirements Specification (SRS)

## AI Prompt Dialog

**Version:** 1.1
**Date:** 2026-02-08
**Author:** prompt-dialog team

---

## 1. Introduction

### 1.1 Purpose

This document defines the software requirements for the AI Prompt Dialog application -- a lightweight, frameless desktop dialog window built with Rust and the Slint UI framework. The dialog provides a single text input field with custom font rendering and a drop shadow effect, intended as a quick-launch AI prompt entry point (similar to Spotlight or Alfred). Upon submission, the prompt is sent to a running [OpenCode](https://github.com/anomalyco/opencode) instance via its HTTP server API.

### 1.2 Scope

The application is a single-window desktop utility that:

- Presents a floating, undecorated (frameless) text input dialog
- Renders text with a custom, visually appealing font
- Displays a drop shadow around the dialog to visually separate it from the desktop background
- Sends the submitted prompt to a running OpenCode instance via its HTTP server API
- Follows the same integration pattern as [opencode-helix](https://github.com/wk-j/opencode-helix), adapted from a terminal TUI to a native GUI dialog

### 1.3 Definitions and Acronyms

| Term | Definition |
|------|-----------|
| Slint | A declarative UI toolkit for Rust, C++, JavaScript, and Python (https://slint.dev) |
| OpenCode | An open source AI coding agent with client/server architecture (https://github.com/anomalyco/opencode) |
| opencode-helix | A reference Rust CLI/TUI that integrates OpenCode with the Helix editor (https://github.com/wk-j/opencode-helix) |
| Frameless window | A window with no OS-provided title bar, borders, or controls (`no-frame: true` in Slint) |
| Drop shadow | A visual effect simulating a shadow cast by the dialog onto the surface behind it |
| `.slint` file | Slint markup language file defining UI layout and behavior |
| TrueType/OpenType | Font formats (`.ttf`, `.otf`) supported by Slint for custom font embedding |

### 1.4 Technology Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2021) |
| UI Framework | Slint (latest stable, currently 1.x) |
| HTTP Client | `reqwest` (with JSON support) |
| Async Runtime | `tokio` |
| Process Discovery | `sysinfo` |
| Build | Cargo + `slint-build` crate (build.rs) |
| Backend | Winit (default Slint desktop backend) |
| AI Backend | OpenCode server (`opencode --port <N>`) |
| Target Platforms | macOS, Windows, Linux (X11/Wayland) |

---

## 2. Overall Description

### 2.1 Product Perspective

The AI Prompt Dialog is a standalone desktop application that acts as a lightweight front-end for OpenCode. The user types a prompt, presses Enter, and the application sends the text to a running OpenCode server instance via HTTP. The OpenCode TUI (running in a separate terminal/tmux pane) processes the prompt and displays the AI response.

This is functionally similar to `opencode-helix` but replaces the terminal-based ratatui TUI with a native Slint GUI dialog, making it invocable as a global desktop prompt (e.g., via a system-wide hotkey) rather than from within a specific editor.

**Prerequisite:** OpenCode must be running with the `--port` flag:
```
opencode --port 8080
```

### 2.2 User Classes

| User Class | Description |
|------------|-----------|
| End User | A desktop user who invokes the dialog (e.g., via global hotkey) to type an AI prompt |
| Developer/Integrator | Embeds or extends the dialog within a larger AI assistant pipeline |

### 2.3 Operating Environment

- macOS 12+, Windows 10+, Linux with X11 or Wayland
- No GPU requirement beyond what the Slint renderer needs (software rendering fallback available)

### 2.4 Constraints

- The Slint `Window` element provides `no-frame: true` to remove OS decorations, but does **not** natively support window-level drop shadows on all platforms. The shadow must therefore be implemented as a visual element rendered inside an oversized transparent window.
- Custom fonts must be TrueType (`.ttf`), TrueType Collection (`.ttc`), or OpenType (`.otf`) and imported at the `.slint` file level.
- The Slint `resize-border-width` property (winit backend) allows resizable frameless windows; this application will **not** be resizable.
- OpenCode must be running with `--port` before this dialog can function. The dialog is a client only -- it does not start or manage the OpenCode process.
- The OpenCode TUI API uses `POST /tui/publish` for prompt interaction. This is a fire-and-forget pattern; the dialog does not display AI responses (those appear in the OpenCode TUI).

---

## 3. Functional Requirements

### 3.1 FR-01: Frameless Window

| Field | Value |
|-------|-------|
| ID | FR-01 |
| Priority | High |
| Description | The application shall display a window with no OS-provided title bar, borders, or window controls. |
| Slint mechanism | Set `no-frame: true` on the root `Window` element. |
| Acceptance Criteria | On all target platforms, no title bar, minimize/maximize/close buttons, or OS border chrome is visible. |

### 3.2 FR-02: Drop Shadow Effect

| Field | Value |
|-------|-------|
| ID | FR-02 |
| Priority | High |
| Description | The dialog shall appear to float above the desktop with a soft drop shadow surrounding it on all four sides. |
| Implementation approach | The root `Window` shall be larger than the visible dialog area by a configurable shadow margin (e.g., 20px per side). The `Window.background` shall be set to `transparent`. A `Rectangle` element rendered behind the main dialog content shall simulate the shadow using multiple layered semi-transparent rectangles with increasing size and decreasing opacity, or a blurred border technique. |
| Acceptance Criteria | A visible, soft shadow is rendered around the dialog. The shadow does not receive mouse click events (clicks on the shadow region pass through or close the dialog). |

### 3.3 FR-03: Text Input Field

| Field | Value |
|-------|-------|
| ID | FR-03 |
| Priority | High |
| Description | The dialog shall contain a single-line text input field where the user can type an AI prompt. |
| Slint mechanism | Use a `TextInput` element (or the std-widgets `LineEdit`). The field shall receive focus automatically on window show. |
| Acceptance Criteria | The text input is visible and editable. The cursor appears in the field immediately when the dialog opens. Standard keyboard operations (typing, backspace, select-all, clipboard copy/paste) function correctly. |

### 3.4 FR-04: Custom Font Rendering

| Field | Value |
|-------|-------|
| ID | FR-04 |
| Priority | High |
| Description | Text in the input field (and any labels) shall be rendered using an embedded custom font to achieve a visually distinctive ("fancy") appearance. |
| Slint mechanism | Import the font file at the top of the `.slint` file using `import "./fonts/MyFont.ttf";` and set `default-font-family` on the `Window`, or `font-family` on the `TextInput`. Also configure `default-font-size` and `default-font-weight` as needed. |
| Font selection | A visually appealing, freely licensed font shall be bundled (e.g., Inter, JetBrains Mono, Fira Code, or similar). The specific font will be chosen during design. |
| Acceptance Criteria | The rendered text uses the custom font -- not the system default -- on all platforms. Font weight and size are consistent across platforms. |

### 3.5 FR-05: Submit Prompt to OpenCode

| Field | Value |
|-------|-------|
| ID | FR-05 |
| Priority | High |
| Description | When the user presses the Enter key, the application shall capture the entered text and send it to the running OpenCode server via HTTP. |
| Slint mechanism | A `callback submit(string)` declared on the root component. The Rust backend registers a handler via `on_submit()`. |
| HTTP integration | The Rust `on_submit` handler sends the prompt to OpenCode using the TUI publish API (see Section 6 for details). After successful send, the dialog dismisses itself. |
| Acceptance Criteria | Pressing Enter sends the prompt text to the OpenCode server. The OpenCode TUI (in a separate terminal) receives and begins processing the prompt. The dialog closes after successful submission. |

### 3.6 FR-06: Dismiss Dialog

| Field | Value |
|-------|-------|
| ID | FR-06 |
| Priority | Medium |
| Description | The user shall be able to dismiss (close) the dialog by pressing the Escape key or by clicking outside the dialog area. |
| Acceptance Criteria | Pressing Escape hides/closes the window. Clicking in the shadow (transparent) area dismisses the dialog. |

### 3.7 FR-07: Window Positioning

| Field | Value |
|-------|-------|
| ID | FR-07 |
| Priority | Medium |
| Description | The dialog shall appear centered on the primary display when shown. |
| Slint mechanism | Set window position programmatically from Rust using `Window::set_position()` after querying screen dimensions. |
| Acceptance Criteria | The dialog appears horizontally and vertically centered on the primary monitor. |

### 3.8 FR-08: OpenCode Server Discovery

| Field | Value |
|-------|-------|
| ID | FR-08 |
| Priority | High |
| Description | The application shall automatically discover a running OpenCode server, or accept an explicit port via CLI argument. |
| Discovery mechanism | **Option A (CLI flag):** Accept `--port <N>` to connect to a specific port. **Option B (Auto-discovery):** Scan running processes for `opencode --port <N>` patterns using the `sysinfo` crate, extract the port, then validate via `GET /path`. Match the server whose working directory contains or matches the current working directory. This is the same approach used by `opencode-helix`. |
| Acceptance Criteria | When `--port 8080` is passed, the dialog connects to `http://localhost:8080`. When no port is given, the dialog discovers the OpenCode server automatically. If no server is found, an error message is displayed and the dialog exits. |

### 3.9 FR-09: Connection Status Indicator

| Field | Value |
|-------|-------|
| ID | FR-09 |
| Priority | Low |
| Description | The dialog shall provide visual feedback indicating whether it is connected to an OpenCode server. |
| Implementation | A small status indicator (dot or icon) in the dialog. Green = connected, red/absent = no server found. |
| Acceptance Criteria | User can visually tell whether the dialog has a valid OpenCode connection before typing. |

---

## 4. Non-Functional Requirements

### 4.1 NFR-01: Startup Latency

| Field | Value |
|-------|-------|
| ID | NFR-01 |
| Description | The dialog shall be visible and ready for input within 200ms of invocation on modern hardware. |
| Rationale | As a quick-launch utility, perceived latency must be minimal. |

### 4.2 NFR-02: Memory Footprint

| Field | Value |
|-------|-------|
| ID | NFR-02 |
| Description | The application's resident memory usage shall not exceed 50 MB under normal operation. |

### 4.3 NFR-03: Cross-Platform Visual Consistency

| Field | Value |
|-------|-------|
| ID | NFR-03 |
| Description | The dialog shall look visually identical (within renderer differences) on macOS, Windows, and Linux. The custom font, drop shadow, and layout shall not vary across platforms. |

### 4.4 NFR-04: Accessibility

| Field | Value |
|-------|-------|
| ID | NFR-04 |
| Description | The text input shall be compatible with OS-level accessibility features (screen readers, high-contrast modes) to the extent supported by the Slint framework. |

### 4.5 NFR-05: Binary Size

| Field | Value |
|-------|-------|
| ID | NFR-05 |
| Description | The release binary (stripped) should remain under 10 MB, excluding bundled font files. |

---

## 5. Architecture Overview

### 5.1 Project Structure

```
prompt-dialog/
  Cargo.toml
  build.rs                     # slint_build::compile("ui/prompt-dialog.slint")
  src/
    main.rs                    # Entry point, CLI args, server discovery, event loop
    server/
      mod.rs                   # Re-exports
      client.rs                # HTTP client for OpenCode TUI API
      discovery.rs             # Process scanning & server validation
  ui/
    prompt-dialog.slint        # Slint UI definition
    fonts/
      CustomFont.ttf           # Bundled custom font
```

### 5.2 Component Diagram

```
+-----------------+          HTTP           +-------------------+
|  prompt-dialog  | ----------------------> |  opencode server  |
|  (Slint GUI)    |  POST /tui/publish      |  (--port 8080)    |
|                 |  GET  /path             |                   |
|  - TextInput    |                         |  - AI processing  |
|  - Drop shadow  |                         |  - TUI display    |
|  - No frame     |                         |  - Tool execution |
+-----------------+                         +-------------------+

Discovery flow:
  1. Scan processes for "opencode --port <N>" (sysinfo)
  2. Extract port from command line
  3. Validate via GET /path (match working directory)
  4. Use port for all subsequent requests
```

---

## 6. OpenCode Integration

This is the core backend integration. The dialog acts as a thin client to the OpenCode server, following the same pattern established by [opencode-helix](https://github.com/wk-j/opencode-helix).

### 6.1 OpenCode Server Prerequisites

OpenCode must be started with the `--port` flag in a separate terminal or tmux pane:

```bash
opencode --port 8080
```

The server exposes an HTTP API on `http://localhost:<port>`. Full API spec is available at `http://localhost:<port>/doc`.

### 6.2 Server Discovery

The discovery logic (identical to `opencode-helix`) works as follows:

1. **Explicit port** -- If `--port <N>` is passed to prompt-dialog, connect directly to `http://localhost:<N>`
2. **Auto-discovery** -- Scan running processes using `sysinfo` for command lines containing `opencode` and `--port`. Extract the port number, then validate each candidate by calling `GET /path` and matching the working directory.

```rust
// Pseudocode for discovery
fn discover_server(cwd: &Path, port: Option<u16>) -> Result<Server> {
    if let Some(p) = port {
        return validate_server(p);  // GET /path
    }
    for process in find_processes_matching("opencode --port") {
        let port = extract_port(process.cmdline);
        let server = validate_server(port)?;
        if cwd.starts_with(&server.cwd) {
            return Ok(server);
        }
    }
    Err("No opencode server found")
}
```

### 6.3 HTTP API Usage

The dialog uses two OpenCode server endpoints:

#### Validation

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/path` | Returns `{ directory, worktree }`. Used to validate the server and match working directory. |

#### Prompt Submission

| Method | Path | Body | Purpose |
|--------|------|------|---------|
| `POST` | `/tui/publish` | `{ "type": "tui.prompt.append", "properties": { "text": "<prompt>" } }` | Append the prompt text to the OpenCode TUI input |
| `POST` | `/tui/publish` | `{ "type": "tui.command.execute", "properties": { "command": "prompt.submit" } }` | Submit the prompt for AI processing |

The full submit sequence (matching `opencode-helix`'s `Client::send_prompt`):

```
1. POST /tui/publish  { type: "tui.prompt.append", properties: { text: "<user input>" } }
2. POST /tui/publish  { type: "tui.command.execute", properties: { command: "prompt.submit" } }
3. Dialog closes itself
```

### 6.4 Error Handling

| Scenario | Behavior |
|----------|----------|
| No OpenCode server running | Display error message in the dialog, disable submit |
| Server unreachable after initial discovery | Show connection error, allow retry or dismiss |
| HTTP request timeout (>5s) | Treat as server unavailable |
| Prompt send fails | Show brief error, keep dialog open so user doesn't lose input |

---

## 7. Dependencies

### 7.1 Cargo Dependencies

```toml
[package]
name = "prompt-dialog"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
slint = "1.15"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sysinfo = "0.32"
anyhow = "1"
clap = { version = "4", features = ["derive"] }

[build-dependencies]
slint-build = "1.15"
```

### 7.2 System Dependencies

- On Linux: `fontconfig`, `libxkbcommon` (for Wayland), or X11 libraries
- On macOS/Windows: no additional system dependencies

---

Additional design details:

- [drop-shadow.md](drop-shadow.md) -- Drop shadow implementation approaches
- [fonts.md](fonts.md) -- Font selection criteria and candidates
- [testing.md](testing.md) -- Test plan
- [risks.md](risks.md) -- Risks, mitigations, and future considerations
