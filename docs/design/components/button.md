# Button

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Button`

Rust `button::button(label)`; QML `Button`.

**Kinds**, by what the button is for:

| Kind | Fill, content | For |
|---|---|---|
| `Filled` | `primary`, `on_primary` | The one thing to do next on a screen. One per screen. |
| `Tonal` | `secondary_container`, `on_secondary_container` | An alternative to the filled one. |
| `Text` | none, `primary` | Back, Skip, Cancel, Show log. |
| `Danger` | `error`, `on_error` | The action that destroys something, after a confirmation. |

**Sizes:**

| Size | Height | Label | Icon | Gap | Padding (Text kind) | For |
|---|---|---|---|---|---|---|
| sm | 32 (`control_sm`) | `button_small` | 16 | 4 | 12 (12) | Chrome around a screen, beside a field's label. |
| md | 40 (`control_md`) | `button` | 20 | 8 | 24 (12) | A screen's own actions; as tall as a field. |
| lg | 48 (`control_lg`) | `button_large` | 24 | 8 | 32 (16) | The one main action of a big page: Kōbō's Install. |

- A label button has the control radius, 12; an icon button is a circle. An optional icon leads the label, or trails it on a button that goes forward (Continue →). The icon sits in a box the label's line tall, centred on a device pixel, so an icon never changes a button's height.
- **A held button** (`Button::on_hold`) is pressed for a while instead of clicked: the press going down and coming up (or the pointer leaving) are the app's to time, and a `timed_bar` beside it shows how far along the hold is. It is for the one action of a screen that destroys what was there, where a dialog would only ask the same question again: Kōbō's Install, held for 1.1 s with the mouse or Enter.

**States:** resting, hovered, pressed, focused and disabled (no action: `on_press_maybe(None)`, `enabled: false`), in every kind and size.
