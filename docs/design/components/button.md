# Button

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Button`

Rust `button::button(label)`; QML `Button`.

**Kinds**, by what the button is for: an emphasis ladder, then three that carry a meaning.

| Kind | At night, rest | By day, rest | For |
|---|---|---|---|
| `Filled` | `primary_fill`, the tone at OKLCH lightness × 0.6 and chroma × 1.2, gamut fitted; label `on_primary_fill`, near white tinted with the tone | the tone at lightness 0.80, label `on_primary_fill`, the tone's dark | The one thing to do next on a screen: the most emphasis. One per screen. |
| `Tonal` | `secondary_container` (matugen's secondary), label `on_secondary_container_rest`, dimmed | `secondary_container`, label `on_secondary_container` | Medium emphasis: an alternative to the filled one. |
| `Text` | no fill, label `primary_rest`, the tone unlit | no fill, label `primary` | Minor actions: Back, Skip, Cancel, Show log. |
| `Danger` | `burn_danger`, a bright fixed red, label `on_burn_danger`, dark | its deeper daylight red, label fitted (white) | The action that destroys something, after a confirmation. |
| `Success` | `burn_success`, a bright fixed green, label `on_burn_success`, dark | its deeper daylight green, label fitted | The action that completes something good: Finish. |
| `Warning` | `burn_warning`, a bright fixed amber, label `on_burn_warning`, dark | its deeper daylight amber, label fitted | The action with a cost the user should weigh: Restart now. |

`primary` comes from the wallpaper (matugen), so every role built on it is a recipe over the live tone (`[color.derived]` in `design/tokens.toml`), never a value. Danger, Success and Warning burn on their own fixed tones, so none can be taken for a wallpaper's accent. Every label holds 4.5:1 on every fill it sits on, at rest, lit and pressed.

**Sizes:**

| Size | Height | Label | Icon | Gap | Padding (Text kind) | For |
|---|---|---|---|---|---|---|
| sm | 32 (`control_sm`) | `button_small` | 16 | 4 | 12 (12) | Chrome around a screen, beside a field's label. |
| md | 40 (`control_md`) | `button` | 20 | 8 | 24 (12) | A screen's own actions; as tall as a field. |
| lg | 48 (`control_lg`) | `button_large` | 24 | 8 | 32 (16) | The one main action of a big page: Kōbō's Install. |

- A label button has the control radius, 12; an icon button is a circle. An optional icon leads the label, or trails it on a button that goes forward (Continue →). The icon sits in a box the label's line tall, centred on a device pixel, so an icon never changes a button's height.
- **A held button** (`Button::on_hold`) is pressed for a while instead of clicked: the press going down and coming up (or the pointer leaving) are the app's to time, and a `timed_bar` beside it shows how far along the hold is. It is for the one action of a screen that destroys what was there, where a dialog would only ask the same question again: Kōbō's Install, held for 1.1 s with the mouse or Enter.

**States** follow the neon (`docs/DESIGN.md`, "States"), in every kind and size. There is no state layer: each state is its own roles.

- **Rest.** At night the button is unlit, as above; Danger, Success and Warning burn even at rest.
- **Lit:** hovered or focused from the keyboard, one state. It has two looks, which the scene picks:
  - **Signs on**, at night while the lights burn: the line inside the edge (the core, `stroke.thin` wide, on Filled and the burning kinds; the tube, `primary` at `focus.tube`, on Tonal and Text) and the glow outside it in the kind's tone. Filled's label brightens to `on_primary_fill_lit` with a faint glow (`focus.label_glow` over `focus.label_near`); Text's label burns as the core (`primary_lit`) with the content glow; Tonal's goes full (`on_secondary_container`). The burning kinds' fill moves 50% of the way toward the core (`burn_danger_lit` and the rest).
  - **Signs off**, by day always and at night while a power change has the lights out: the same line as unlit glass with no glow, and the label at its rest colour. Filled's glass is its unlit fill +0.12 lightness (`primary_fill_glass`); Tonal's and Text's is solid, the accent unlit (`primary_glass`); Danger, Success and Warning keep their lit fill, with the tone unlit as glass on it (`burn_danger_glass` and the rest).
- **Pressed:** overdriven, and lit as well. At night Filled's fill lifts 0.035 (`primary_fill_pressed`), Tonal's container 0.035, Text's tube runs to the core, and the burning kinds' lit fill goes 8% further toward white (`burn_danger_pressed`); every glow is `focus.pressed_glow` (1.25) times the lit one.
- **Disabled:** no action (`on_press_maybe(None)`, `enabled: false`): grey, with no tone, no line and no glow, its content at `opacity.disabled_content` of `on_surface` and its fill at `opacity.disabled_container`; it takes no pointer and no focus.

**By day nothing glows.** Filled rests at lightness 0.80 with a dark label, lit adds a dark line inside its edge (`primary_fill_line`, the fill at 62% of its lightness), and pressed goes 0.04 darker. Tonal is its container, lit adds the tone's line, and pressed goes 0.04 darker. Text is a tone label, lit adds the tone's line, and pressed lays the tone under it at `opacity.press_tint` (10%). The burning kinds go 0.035 darker lit, with a dark line, and 0.06 darker pressed.

**Motion.** Lighting up, going back to rest and focus moving are fades over `short` on `standard`; a press goes down over `micro`. The flicker (`neon.strike`) is for power changes only: in a theme change and dark ⇄ light, a lit button flickers down to its signs-off look and strikes back, while a button at rest just recolours.

*Not built yet: `kubaos-ui` and `KubaOS.Components` have `Filled`, `Tonal`, `Text` and `Danger` on `error`, draw hover and press as state layers, and strike focus; the mockups' Button page (`design/mockups/src/pages/atoms/button.ts`) draws the rule.*
