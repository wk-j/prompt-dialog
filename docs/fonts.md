# Font Requirements

## Selection Criteria

- Freely licensed (OFL / Apache 2.0) for redistribution
- High readability at 16-24px sizes
- Visually distinctive / modern aesthetic
- Good Unicode coverage (Latin, common symbols)
- Available as `.ttf` or `.otf`

## Candidate Fonts

| Font | License | Style | Notes |
|------|---------|-------|-------|
| Inter | OFL 1.1 | Sans-serif, modern | Excellent screen readability |
| JetBrains Mono | OFL 1.1 | Monospace | Good for code-style prompts |
| Fira Code | OFL 1.1 | Monospace, ligatures | Developer-friendly |
| Outfit | OFL 1.1 | Sans-serif, geometric | Clean, modern feel |
| Space Grotesk | OFL 1.1 | Sans-serif, geometric | Distinctive, elegant |

The final font choice will be made during the design phase. Multiple weights (Regular, Medium, Bold) may be bundled.

## Slint Font Loading

Custom fonts are imported at the top of the `.slint` file:

```slint
import "./fonts/CustomFont.ttf";

export component PromptDialog inherits Window {
    default-font-family: "Custom Font";
    default-font-size: 18px;
    default-font-weight: 400;
    // ...
}
```

Slint supports TrueType (`.ttf`), TrueType Collection (`.ttc`), and OpenType (`.otf`) formats. The font is embedded into the binary at compile time.
