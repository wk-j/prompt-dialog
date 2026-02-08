# Software Requirements Specification (SRS)

## AI Prompt Dialog

**Version:** 1.0
**Date:** 2026-02-08
**Author:** prompt-dialog team

---

## 1. Introduction

### 1.1 Purpose

This document defines the software requirements for the AI Prompt Dialog application -- a lightweight, frameless desktop dialog window built with Rust and the Slint UI framework. The dialog provides a single text input field with custom font rendering and a drop shadow effect, intended as a quick-launch AI prompt entry point (similar to Spotlight or Alfred).

### 1.2 Scope

The application is a single-window desktop utility that:

- Presents a floating, undecorated (frameless) text input dialog
- Renders text with a custom, visually appealing font
- Displays a drop shadow around the dialog to visually separate it from the desktop background
- Accepts user text input and exposes it via a callback/event for downstream AI processing

### 1.3 Definitions and Acronyms

| Term | Definition |
|------|-----------|
| Slint | A declarative UI toolkit for Rust, C++, JavaScript, and Python (https://slint.dev) |
| Frameless window | A window with no OS-provided title bar, borders, or controls (`no-frame: true` in Slint) |
| Drop shadow | A visual effect simulating a shadow cast by the dialog onto the surface behind it |
| `.slint` file | Slint markup language file defining UI layout and behavior |
| TrueType/OpenType | Font formats (`.ttf`, `.otf`) supported by Slint for custom font embedding |

### 1.4 Technology Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2021) |
| UI Framework | Slint (latest stable, currently 1.x) |
| Build | Cargo + `slint-build` crate (build.rs) |
| Backend | Winit (default Slint desktop backend) |
| Target Platforms | macOS, Windows, Linux (X11/Wayland) |

---

## 2. Overall Description

### 2.1 Product Perspective

The AI Prompt Dialog is a standalone desktop application. It acts as a front-end input surface -- the user types a prompt, presses Enter, and the application delivers the text to the host system (via stdout, IPC, or a callback depending on future integration).

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

### 3.5 FR-05: Submit Prompt

| Field | Value |
|-------|-------|
| ID | FR-05 |
| Priority | High |
| Description | When the user presses the Enter key, the application shall capture the entered text and emit it via a Slint callback. |
| Slint mechanism | A `callback submit(string)` declared on the root component. The Rust backend registers a handler via `on_submit()`. |
| Acceptance Criteria | Pressing Enter invokes the callback with the current text content. The text content is accessible from the Rust side. |

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
  build.rs                   # slint_build::compile("ui/prompt-dialog.slint")
  src/
    main.rs                  # Rust entry point, callback wiring, window positioning
  ui/
    prompt-dialog.slint      # Slint UI definition
    fonts/
      CustomFont.ttf         # Bundled custom font
```

### 5.2 Component Diagram

```
+--------------------------------------------------+
|  Window (no-frame: true, background: transparent) |
|  +----------------------------------------------+ |
|  |  Shadow Layer (Rectangle, semi-transparent)   | |
|  |  +------------------------------------------+| |
|  |  |  Dialog Body (Rectangle, opaque)          || |
|  |  |  +--------------------------------------+|| |
|  |  |  |  TextInput (custom font, auto-focus) ||| |
|  |  |  +--------------------------------------+|| |
|  |  +------------------------------------------+| |
|  +----------------------------------------------+ |
+--------------------------------------------------+
```

### 5.3 Slint UI Skeleton

```slint
import "./fonts/CustomFont.ttf";

export component PromptDialog inherits Window {
    no-frame: true;
    background: transparent;
    always-on-top: true;
    default-font-family: "Custom Font";
    default-font-size: 18px;

    width: 640px;
    height: 80px;

    callback submit(string);
    callback dismiss();

    // Shadow layer
    Rectangle {
        x: 0px; y: 0px;
        width: parent.width;
        height: parent.height;
        background: transparent;

        // Outer shadow (multiple layered rects for soft shadow)
        Rectangle {
            x: 4px; y: 4px;
            width: parent.width - 8px;
            height: parent.height - 8px;
            border-radius: 14px;
            background: #00000018;
        }
        Rectangle {
            x: 8px; y: 8px;
            width: parent.width - 16px;
            height: parent.height - 16px;
            border-radius: 12px;
            background: #00000030;
        }

        // Dialog body
        Rectangle {
            x: 12px; y: 12px;
            width: parent.width - 24px;
            height: parent.height - 24px;
            border-radius: 10px;
            background: white;

            input := TextInput {
                x: 16px;
                y: (parent.height - self.height) / 2;
                width: parent.width - 32px;
                font-size: 20px;
                accepted => {
                    root.submit(self.text);
                }
            }
        }
    }

    // Click on shadow area to dismiss
    TouchArea {
        clicked => { root.dismiss(); }
    }

    // Escape key to dismiss
    FocusScope {
        key-pressed(event) => {
            if (event.text == Key.Escape) {
                root.dismiss();
                return accept;
            }
            return reject;
        }
    }

    // Auto-focus the input
    init => { input.focus(); }
}
```

### 5.4 Rust Integration Skeleton

```rust
// build.rs
fn main() {
    slint_build::compile("ui/prompt-dialog.slint").unwrap();
}

// src/main.rs
slint::include_modules!();

fn main() {
    let dialog = PromptDialog::new().unwrap();

    // Center the window on screen
    // (platform-specific screen size query, then Window::set_position)

    let weak = dialog.as_weak();
    dialog.on_submit(move |text| {
        println!("{}", text);
        let d = weak.upgrade().unwrap();
        d.hide().unwrap();
        slint::quit_event_loop().unwrap();
    });

    let weak = dialog.as_weak();
    dialog.on_dismiss(move || {
        let d = weak.upgrade().unwrap();
        d.hide().unwrap();
        slint::quit_event_loop().unwrap();
    });

    dialog.run().unwrap();
}
```

---

## 6. Drop Shadow Implementation Details

Since Slint does not provide a built-in `box-shadow` property, the shadow effect must be constructed manually. Two viable approaches:

### 6.1 Approach A: Layered Rectangles (Recommended)

Render 3-5 concentric `Rectangle` elements behind the dialog body, each progressively larger with lower opacity and larger `border-radius`. This creates a stepped approximation of a Gaussian blur shadow.

| Layer | Offset from dialog edge | Opacity | Border Radius |
|-------|------------------------|---------|--------------|
| 1 (outermost) | 16px | 0.03 | 18px |
| 2 | 12px | 0.06 | 16px |
| 3 | 8px | 0.10 | 14px |
| 4 | 4px | 0.15 | 12px |
| Dialog body | 0px | 1.0 | 10px |

### 6.2 Approach B: Pre-rendered Shadow Image

Export a shadow texture as a PNG with transparency and render it as an `Image` element behind the dialog body using Slint's `Image` with `nine-slice` rendering if the dialog size is dynamic.

### 6.3 Chosen Approach

Approach A (Layered Rectangles) is recommended because:
- No external asset dependency
- Scales with any dialog size
- Fully declarative in `.slint` markup
- Easy to tune colors/offsets

---

## 7. Font Requirements

### 7.1 Font Selection Criteria

- Freely licensed (OFL / Apache 2.0) for redistribution
- High readability at 16-24px sizes
- Visually distinctive / modern aesthetic
- Good Unicode coverage (Latin, common symbols)
- Available as `.ttf` or `.otf`

### 7.2 Candidate Fonts

| Font | License | Style | Notes |
|------|---------|-------|-------|
| Inter | OFL 1.1 | Sans-serif, modern | Excellent screen readability |
| JetBrains Mono | OFL 1.1 | Monospace | Good for code-style prompts |
| Fira Code | OFL 1.1 | Monospace, ligatures | Developer-friendly |
| Outfit | OFL 1.1 | Sans-serif, geometric | Clean, modern feel |
| Space Grotesk | OFL 1.1 | Sans-serif, geometric | Distinctive, elegant |

The final font choice will be made during the design phase. Multiple weights (Regular, Medium, Bold) may be bundled.

---

## 8. Dependencies

### 8.1 Cargo Dependencies

```toml
[package]
name = "prompt-dialog"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
slint = "1.15"

[build-dependencies]
slint-build = "1.15"
```

### 8.2 System Dependencies

- On Linux: `fontconfig`, `libxkbcommon` (for Wayland), or X11 libraries
- On macOS/Windows: no additional system dependencies

---

## 9. Testing Requirements

| Test ID | Type | Description |
|---------|------|-----------|
| T-01 | Manual | Verify no OS window decorations are visible on macOS, Windows, Linux |
| T-02 | Manual | Verify drop shadow is visible and visually correct on all platforms |
| T-03 | Manual | Verify custom font is rendered (not system default) |
| T-04 | Manual | Verify text input receives focus on dialog open |
| T-05 | Manual | Verify Enter key triggers submit callback with correct text |
| T-06 | Manual | Verify Escape key dismisses the dialog |
| T-07 | Manual | Verify clicking shadow/transparent area dismisses the dialog |
| T-08 | Manual | Verify dialog appears centered on primary display |
| T-09 | Performance | Measure startup-to-ready time is < 200ms |
| T-10 | Unit | Rust unit test: callback wiring delivers correct string |

---

## 10. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Transparent window not supported on some Linux compositors | Shadow not visible; dialog has opaque background | Detect compositor support at runtime; fall back to opaque background with border |
| `no-frame` behavior varies across window managers | Window may still show decorations on some Linux WMs | Document known WM compatibility; provide `SLINT_BACKEND` override option |
| Custom font missing or fails to load | Text falls back to system default, losing visual identity | Include font in binary via Slint import; add startup check |
| Click-through on shadow area may not work on all platforms | User cannot dismiss by clicking outside | Provide Escape key as reliable fallback; document behavior |

---

## 11. Future Considerations

- Global hotkey registration (e.g., via `global-hotkey` crate) to summon/dismiss the dialog
- Streaming AI response display below the input field
- Theming support (dark/light mode)
- Multi-line input mode toggle
- History/autocomplete from previous prompts
- Plugin system for different AI backends
