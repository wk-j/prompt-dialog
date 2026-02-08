# Drop Shadow Implementation

Since Slint does not provide a built-in `box-shadow` property, the shadow effect must be constructed manually.

## Approach A: Layered Rectangles (Recommended)

Render 3-5 concentric `Rectangle` elements behind the dialog body, each progressively larger with lower opacity and larger `border-radius`. This creates a stepped approximation of a Gaussian blur shadow.

| Layer | Offset from dialog edge | Opacity | Border Radius |
|-------|------------------------|---------|--------------|
| 1 (outermost) | 16px | 0.03 | 18px |
| 2 | 12px | 0.06 | 16px |
| 3 | 8px | 0.10 | 14px |
| 4 | 4px | 0.15 | 12px |
| Dialog body | 0px | 1.0 | 10px |

### Slint Skeleton

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

## Approach B: Pre-rendered Shadow Image

Export a shadow texture as a PNG with transparency and render it as an `Image` element behind the dialog body using Slint's `Image` with `nine-slice` rendering if the dialog size is dynamic.

## Chosen Approach

Approach A (Layered Rectangles) is recommended because:
- No external asset dependency
- Scales with any dialog size
- Fully declarative in `.slint` markup
- Easy to tune colors/offsets
