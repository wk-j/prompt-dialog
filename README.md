# prompt-dialog

Frameless, floating prompt dialog for [Claude Code](https://github.com/anomalyco/Claude). Built with Rust and [Slint](https://slint.dev).

Dark translucent window with multi-line input, custom underscore cursor, and mononoki font. Sends prompts to a running Claude Code instance via its TUI HTTP API.

## Usage

```bash
# Auto-discover running Claude Code server
cargo run

# Specify port explicitly
cargo run -- --port 8080

# Debug mode
cargo run -- --debug
```

**Cmd+Enter** (macOS) / **Ctrl+Enter** to submit. **Escape** or click outside to dismiss.

## Build

```bash
cargo build --release
```

## License

MIT
