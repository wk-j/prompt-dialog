# Risks, Mitigations, and Future Considerations

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Transparent window not supported on some Linux compositors | Shadow not visible; dialog has opaque background | Detect compositor support at runtime; fall back to opaque background with border |
| `no-frame` behavior varies across window managers | Window may still show decorations on some Linux WMs | Document known WM compatibility; provide `SLINT_BACKEND` override option |
| Custom font missing or fails to load | Text falls back to system default, losing visual identity | Include font in binary via Slint import; add startup check |
| Click-through on shadow area may not work on all platforms | User cannot dismiss by clicking outside | Provide Escape key as reliable fallback; document behavior |
| OpenCode server not running | Dialog cannot submit prompts | Show clear error state; document requirement to start `opencode --port` first |
| OpenCode TUI publish API changes in future versions | HTTP requests fail silently or with errors | Pin to known API version; add health check on startup via `GET /global/health` |
| Process scanning (sysinfo) may not find OpenCode on all platforms | Auto-discovery fails | Always support explicit `--port` flag as reliable fallback |

## Future Considerations

- Global hotkey registration (e.g., via `global-hotkey` crate) to summon/dismiss the dialog
- Streaming AI response display below the input field (via `GET /event` SSE stream)
- Theming support (dark/light mode)
- Multi-line input mode toggle
- History/autocomplete from previous prompts
- Context placeholders (`@this`, `@buffer`, `@selection`) as in opencode-helix
- Select mode: menu of predefined prompts, commands, and agents fetched from `GET /command` and `GET /agent`
