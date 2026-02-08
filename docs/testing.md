# Testing Requirements

## Test Plan

| Test ID | Type | Description |
|---------|------|-----------|
| T-01 | Manual | Verify no OS window decorations are visible on macOS, Windows, Linux |
| T-02 | Manual | Verify drop shadow is visible and visually correct on all platforms |
| T-03 | Manual | Verify custom font is rendered (not system default) |
| T-04 | Manual | Verify text input receives focus on dialog open |
| T-05 | Manual | Verify Enter key sends prompt to OpenCode server and dialog closes |
| T-06 | Manual | Verify Escape key dismisses the dialog without sending |
| T-07 | Manual | Verify clicking shadow/transparent area dismisses the dialog |
| T-08 | Manual | Verify dialog appears centered on primary display |
| T-09 | Performance | Measure startup-to-ready time is < 200ms |
| T-10 | Unit | Rust unit test: HTTP client correctly formats TUI publish request |
| T-11 | Unit | Rust unit test: server discovery extracts port from `--port 8080` and `--port=8080` |
| T-12 | Integration | With OpenCode running on `--port 8080`, submit a prompt and verify it appears in OpenCode TUI |
| T-13 | Manual | Verify error display when no OpenCode server is running |
| T-14 | Manual | Verify `--port` CLI flag overrides auto-discovery |
