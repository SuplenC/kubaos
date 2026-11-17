# Keycap, KeyHints and KeyPrompt

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Keycap`

Rust `keycap::keycap(look, key)`; QML `Keycap`.

A key drawn as a key: 20 tall (`keycap`), as a line of body text, so it sits inline with body and caption text without making the line taller. The `keycap` type (Geist Mono 11, medium) in `on_surface_variant` on `surface_container_highest`, a hairline edge in `outline_variant` with a second hairline at the bottom, so the key stands on the page, the small radius (4), 4 either side of the key and never narrower than tall, so a one-letter key is square; the key is centred across it on a whole device pixel. A keycap is a picture of a key, not a control: it has no states of its own, never lights and takes no focus, and a power change only recolours it. On a lit menu item its key and its edge take the item's light, in `primary` (`menu.md`).

## `KeyHints`

Rust `keycap::key_hints(look, hints)`; QML `KeyHints`.

Keys and what each does, side by side, 16 apart: a combination's keys as keycaps 4 apart, one keycap per key (`Ctrl+K` is `Ctrl` and `K`; a modifier's or an arrow's glyph is a key of its own however it is written, so `⌃K` is `⌃` and `K` and `← →` two arrows; a `+` alone is the plus key), then what it does in `caption`, `on_surface_variant`, 8 after them. QML also reads Kotodama's form, `↵ Open · Tab Enter app · ⌃K Actions`. Given a `room`, QML leaves out a hint that would pass it whole, and every hint after it, so none is cut through a keycap: the Command menu's footer and detail pane. *Not built yet: iced's `key_hints` takes no room; its container cuts it.*

- **A key hint is never text on text.** Every key named on screen is a `Keycap`: the Command menu's footer, its Action panel and its detail pane, Kōbō's rail, footers and screens, the control center's Bluetooth and keyboard sections, the wallpaper picker, the clock. The one exception is a `Hint` bubble, a single line of `label` text, which names a key in words (the keyboard layout pill's "SUPER+SHIFT+Space"). The key stands out as a key, so the eye finds it first and reads what it does after.
- **Inline with text**: a keycap in a sentence ("Press `Space` to pick") sits in a row centred on the line, 4 from the words.

## `KeyPrompt`

The mockups' `keyPrompt(combination)`. *Not built yet: neither library has it.*

The keys a surface asks to be pressed now, drawn large: each key 48 tall (`control_lg`) and never narrower than tall, its name in `button_large`, 20 either side (`s20`), on `surface_container_high` at the control radius (12), in the primary's tube (`focus.tube`), glowing at night with the focus family's near and far glow and not by day, as a lit control does. A `+` in `subheading` and `on_surface_variant` stands between each two keys, 12 apart (`s12`). It is the one key a step is about, alone on its screen: first boot's Kotodama card shows the key that opens the Command menu with it on What, and on Try it the keys picked on Yours. It takes no focus, but it answers its keys: pressed (or clicked, where a click does what the keys do), each key's fill lifts to `surface_container_highest` and its glow swells to `focus.pressed_glow` times its own over `micro`, then settles over `short`, as a pressed control's does; by day only the fill lifts. The press moves nothing, so reduce motion keeps it. A line of keys among words is a `Keycap` or `KeyHints`, never a prompt.
