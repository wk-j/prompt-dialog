# AGENTS.md

## Project Overview

AI Prompt Dialog — a frameless, drop-shadowed desktop dialog built with **Rust** and **Slint**.
Sends prompts to a running [OpenCode](https://github.com/anomalyco/opencode) instance via HTTP.
Reference implementation: [opencode-helix](https://github.com/wk-j/opencode-helix).

Design docs live in `docs/` (SRS, drop-shadow, fonts, testing, risks).

## Project Structure

```
prompt-dialog/
  Cargo.toml
  build.rs                     # slint_build::compile("ui/prompt-dialog.slint")
  src/
    main.rs                    # Entry point, CLI parsing (clap), server discovery, Slint event loop
    server/
      mod.rs                   # Re-exports client::Client, discovery::discover_server
      client.rs                # HTTP client for OpenCode TUI API (reqwest)
      discovery.rs             # Process scanning (sysinfo) & server validation
  ui/
    prompt-dialog.slint        # Slint UI definition (frameless window, shadow, text input)
    fonts/
      *.ttf                    # Bundled custom font(s)
  docs/
    SRS.md                     # Software requirements specification
    drop-shadow.md             # Shadow implementation details + Slint skeleton
    fonts.md                   # Font selection criteria
    testing.md                 # Test plan
    risks.md                   # Risks and future considerations
```

## Build / Run / Test Commands

```bash
# Build (debug)
cargo build

# Build (release, optimized + stripped)
cargo build --release

# Run (with explicit OpenCode port)
cargo run -- --port 8080

# Run (auto-discover OpenCode server)
cargo run

# Run with custom placeholders
cargo run -- --param path=/src/main.rs --param selection="some code"

# Run all tests
cargo test

# Run a single test by name
cargo test test_name
cargo test server::discovery::tests::test_extract_port

# Run tests in a specific module
cargo test server::client::tests

# Check without building
cargo check

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Format fix
cargo fmt

# Slint UI preview (if slint-viewer is installed)
slint-viewer ui/prompt-dialog.slint
```

## Code Style Guidelines

### Rust

- **Edition:** 2021
- **Formatting:** `rustfmt` defaults — run `cargo fmt` before committing
- **Linting:** `cargo clippy -- -D warnings` must pass with zero warnings
- **Error handling:** Use `anyhow::Result` for application errors. Use `thiserror` only if
  defining library-level error enums. Propagate with `?`, add context with `.context("msg")`
- **Unwrap:** Never use `.unwrap()` in production code. Acceptable only in tests and `build.rs`
- **Async:** Use `tokio` runtime. HTTP calls via `reqwest` are async. Slint runs on the main
  thread; bridge async work through `slint::invoke_from_event_loop` or `spawn_local`

### Naming Conventions

- **Files/modules:** `snake_case` (e.g., `client.rs`, `discovery.rs`)
- **Types/structs:** `PascalCase` (e.g., `Client`, `Server`, `PathResponse`)
- **Functions/methods:** `snake_case` (e.g., `discover_server`, `send_prompt`)
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Slint component:** `PascalCase` (e.g., `PromptDialog`)
- **Slint properties/callbacks:** `kebab-case` in `.slint` files (e.g., `no-frame`, `font-size`)

### Import Order

Group imports in this order, separated by blank lines:

```rust
// 1. Standard library
use std::path::{Path, PathBuf};

// 2. External crates
use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};

// 3. Internal modules
use crate::server::Client;
```

### Module Organization

- One module per file. Use `mod.rs` only for directory modules with re-exports
- Keep `main.rs` thin — CLI parsing, server discovery, Slint setup, callback wiring
- All HTTP logic in `server/client.rs`, all process scanning in `server/discovery.rs`
- UI definition stays in `.slint` files, not in Rust code

### Slint (.slint files)

- Use `kebab-case` for properties: `border-radius`, `font-size`, `no-frame`
- Use `:=` for named elements: `input := TextInput { ... }`
- Declare callbacks on the root component: `callback submit(string);`
- Import fonts at file top: `import "./fonts/CustomFont.ttf";`
- Window must set: `no-frame: true; background: transparent; always-on-top: true;`

### Structs and Serialization

- Derive `Serialize`/`Deserialize` for HTTP request/response types
- Use `#[serde(rename = "type")]` for JSON field name conflicts with Rust keywords
- Keep request/response structs in `client.rs`, co-located with the methods that use them

## OpenCode HTTP Integration

The dialog communicates with OpenCode's TUI API. Key endpoints:

```
GET  /path                → validate server, get working directory
POST /tui/publish          → append prompt text or execute TUI commands
GET  /agent               → list available agents (future)
GET  /command             → list custom commands (future)
```

Submit sequence (fire-and-forget, dialog closes after step 2):

```
1. POST /tui/publish  { "type": "tui.prompt.append", "properties": { "text": "..." } }
2. POST /tui/publish  { "type": "tui.command.execute", "properties": { "command": "prompt.submit" } }
```

HTTP client timeout: 5 seconds. On failure, keep dialog open and show error.

## Dependencies

Key crates and their roles:

| Crate | Purpose |
|-------|---------|
| `slint` / `slint-build` | UI framework + build-time `.slint` compilation |
| `tokio` | Async runtime for HTTP calls |
| `reqwest` | HTTP client (with `json` feature) |
| `serde` / `serde_json` | JSON serialization for OpenCode API |
| `sysinfo` | Process scanning for server auto-discovery |
| `clap` | CLI argument parsing (`--port`, `--debug`, etc.) |
| `anyhow` | Application-level error handling |

## Testing

- Unit tests go in the same file as the code, inside `#[cfg(test)] mod tests { ... }`
- Test server discovery port extraction from various cmdline formats
- Test HTTP client request formatting (URL construction, JSON body shape)
- Integration tests (with a live OpenCode server) are manual — see `docs/testing.md`
- No snapshot or UI tests are required at this stage

## Git Conventions

- Commit messages: imperative mood, concise first line (e.g., "Add server discovery module")
- Do not commit `target/`, font binaries over 5MB, or `.env` files
- Keep `Cargo.lock` committed (this is a binary application, not a library)

## Release Build

```toml
[profile.release]
lto = true
strip = true
```

Target binary size: under 10 MB (excluding bundled fonts).
