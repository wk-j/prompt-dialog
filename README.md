# prompt-dialog

Frameless, floating prompt dialog for [OpenCode](https://github.com/anomalyco/opencode). Built with Rust and [Slint](https://slint.dev).

Dark translucent window with multi-line input, custom underscore cursor, and mononoki font. Sends prompts to a running OpenCode instance via its TUI HTTP API.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Auto-discover running OpenCode server
prompt-dialog

# Specify port explicitly
prompt-dialog --port 8080

# With custom placeholders
prompt-dialog --param path=/src/main.rs --param lang=rust

# Debug mode
prompt-dialog --debug
```

## Keyboard

| Key | Action |
|-----|--------|
| **Cmd+Enter** / **Ctrl+Enter** | Submit prompt |
| **Tab** | Accept autocomplete suggestion |
| **Escape** | Dismiss dialog |

## Placeholders

Type `@` in your prompt to use placeholders. They expand to real values on submit.

### Autocomplete

Type `@` followed by a partial name to see suggestions. Press **Tab** to accept.

### Built-in tokens

| Token | Description |
|-------|-------------|
| `@clipboard` | Current system clipboard text content |

### Custom parameters

Pass `--param key=value` to define custom placeholders:

```bash
prompt-dialog --param path=/src/main.rs --param lang=rust
```

Then type:

```
Fix the bug in @path, it's written in @lang. Here's context: @clipboard
```

Placeholders are highlighted in purple as you type and expand to actual values on submit to OpenCode.

## Build

```bash
cargo build --release
```

## License

MIT
