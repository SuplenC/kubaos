# KubaOS: look and behaviour

How every KubaOS surface looks and behaves: the shell, the Command menu, Kōbō, the apps to come, the terminals, Plymouth and GRUB. Agents: read this before you build or change anything a person sees, in any toolkit. A rule here has a reason and a picture; follow the rule, and when the reason doesn't fit your case, change the rule here for everyone (see "Changing the design") instead of working round it.

Three files hold the design, each owning one thing:

| File | Owns |
|---|---|
| `design/tokens.toml` | Every value: colour roles, lengths, type styles, durations, curves, opacities, shadows, the list of symbols. `kubaos-tokens` checks its rules and writes each toolkit's copy. |
| `docs/DESIGN.md` (this file) | The rules and their reasons: which value is for what, the components, the keyboard, focus, scaling. |
| `docs/BRAND.md` | The words: name, voice, and how errors and buttons read. |

A rule marked *Not enforced yet* is decided but not built: no code follows it today. The marker names the issue that builds it, or says that none does yet. Every other rule here is what the code does.

Values below are shown so this file reads on its own. Each one names its token, and the token wins: when they disagree, `design/tokens.toml` is right and this file is stale.

The pictures are the specimen, `kubaos-specimen`, which draws every token and component with `kubaos-ui`. The 1× shots are checked in under `design/specimen/`; `kubaos-specimen shots DIR` renders all of them at 1, 1.25, 1.5 and 2, light and dark. The Command menu's "Design specimen" Item opens it live.

## Principles

In priority order: when two conflict, the higher one wins.

1. **Details.** Every pixel is decided, including the ones nobody sees. An edge between two device pixels, a gap that is 13 instead of 12, an icon half a pixel off centre: each is a bug, and the checks below fail on them.
2. **Fast.** Performance is a feature. Every frame lands on time at the display's full refresh rate; a dropped frame is a bug and is measured, not guessed at.
3. **Simple.** One way to do a thing. A closed set of components and values, so the choice is small.
4. **One thing.** Every surface, in every toolkit, is the same surface: the same names, sizes, colours, keys and motion in iced and QML.
5. **Keyboard first.** Everything the mouse does, keys do, with a visible focus ring. The mouse works; you never need it.
6. **Quiet until needed.** Nothing moves, blinks or pops up unless it has something to say. Notifications are for what you'd want to be interrupted for.
7. **Your windows are yours.** KubaOS never resizes, moves or covers your windows on its own. Panels float over them and go.
8. **For devs and users.** Dense, never cramped: the information a developer wants, at sizes everyone can read.

## Reference look

Kōbō is the reference. Kuba designed it; the shell's look was inherited from someone else's config. Every front end aligns towards Kōbō, and Kōbō's own drift (off-grid paddings, hand-made buttons) was fixed first, in `kubaos-ui`. When a shell surface and Kōbō disagree on something this file doesn't settle, Kōbō is right until this file says otherwise.

## Colour

![Colour roles and contrast pairs, dark](../design/specimen/colour-dark-1x.png)

*The roles as swatches, then each foreground and background pair with its contrast ratio and the minimum it needs. Light: [colour-light-1x.png](../design/specimen/colour-light-1x.png).*

- **Roles, never hex.** Code names a role (`primary`, `on_surface`, `surface_container_high`, `error`, `cold`, `warning`), never a colour. The roles are Material 3's, plus `cold` and `warning`. The palette changes with the wallpaper, so a hex value in a component is wrong the moment the wallpaper changes.
- **Matugen derives them from the wallpaper** (`scripts/dot-theme`, `scheme-fidelity`). `[color.source]` in the tokens names the matugen colour each role comes from. `[color.dark]` and `[color.light]` are the fallback: matugen's output for the default wallpaper, used until a theme exists and in the specimen.
- **Cold accent** (`cold`, from matugen's tertiary): the blue of the sign's dakuten. With `scheme-fidelity` tertiary is the wallpaper's complement, so cold follows the wallpaper. Use it rarely: the focus ring on picked cards, where the accent already means picked, and a demo or preview badge. It is never a second primary. It is configurable: the complement is the default, and the user may pick another. *Not enforced yet: lands in #154.*
- **Warning** is ours (Material 3 has none): amber between fine and the error colour, for a reading close to its limit. It keeps its value whatever the wallpaper; the light one is dark enough for text on light surfaces.
- **Contrast minimums** (WCAG 2.1, `kubaos_ui::color::Contrast`): 4.5:1 for body text and labels; 3:1 for large text (24 px and up, or 19 px semibold and up) and for icons, outlines and other shapes that carry meaning. `kubaos-ui`'s tests hold every pair in both fallback palettes. A custom colour that breaks a minimum is refused, not drawn. *Not enforced yet: lands in #154.*
- **Opacities** come from `[opacity]`: the state layer (hover 0.08, focus 0.12, press 0.14), disabled content 0.38 and container 0.10, the scrim, tints. A component never picks its own alpha.
- **Custom colours:** a seed colour, overrides of single roles, and presets (KubaOS neon among them). The contrast minimums hold for all of them. *Not enforced yet: lands in #154.*

### Light and dark

- Both ship in v1. Dark is the default. The switch is manual, with an optional sunset and sunrise schedule. *Not enforced yet: lands in #154.*
- Each mode has its own wallpaper: a pair the user picks, or one image for both. *Not enforced yet: lands in #154.*
- The switch crossfades by default (`slow`, 800 ms), or wipes, or is instant, as the user sets it. *Not enforced yet: lands in #154.*
- Plymouth and GRUB are always dark. Kōbō is dark on the live ISO.
- Geometry never changes with the mode: the light and dark shots of a page are the same boxes, and the geometry check holds them to it.

## Type

![Type styles, dark](../design/specimen/type-dark-1x.png)

*Every style, largest first; the tint behind each is its line box, so the line height is visible. Hero 64 sits on top. Light: [type-light-1x.png](../design/specimen/type-light-1x.png).*

**Families** (`[font.family]`), and nothing else draws text:

| Family | For |
|---|---|
| Geist (`sans`) | The interface. Geist 1.8.1 is also the system font (`geist-font` in the curated set), so the shell, the iced apps and GTK draw the same letters. |
| Geist Mono (`mono`) | Anything typed or read by a machine: logs, paths, commands, keycaps, timings. The terminals too (on trial). |
| M PLUS 1 (`jp`) | The Japanese accents above titles. |

`kubaos-ui` embeds the fonts it uses, so an app draws the same on a machine with none installed.

**Sizes:** 11, 12, 13, 14, 16, 20, 24, 32, 48, 64 (`rules.font_sizes`). 64 is the hero's alone. **Every line height is a multiple of 4**, so text boxes stay on the grid. `kubaos-tokens` refuses any other size, and any line height off the grid.

**Styles** (`[type.*]`, the `Type` enum). Pick by what the text is, never by how big you want it:

| Style | Size / line | Weight, family | For |
|---|---|---|---|
| `hero` | 64 / 72 | semibold | Words at the hero size: Kōbō's welcome line, nothing else. |
| `hero_number` | 64 / 72 | light | Numbers at the hero size: the install percent, the clock. Light, so a big figure doesn't shout. |
| `display` | 48 / 56 | semibold | The heading of a full-screen moment other than the welcome: Kōbō's done screen. |
| `title` | 32 / 40 | semibold | A screen's or page's title. |
| `headline` | 24 / 32 | semibold | A dialog's title, a big number. |
| `heading` | 20 / 28 | semibold | A section's title inside a page. |
| `subheading` | 16 / 24 | semibold | A card's or an error's first line. |
| `body_large` | 16 / 24 | regular | A lede under a display line; the text in a field. |
| `body` | 14 / 20 | regular | Running text, explanations, toasts. |
| `body_strong` | 14 / 20 | medium | A list row's title; a value next to its label. |
| `button_large` | 16 / 24 | semibold | A large button's label (48 control). |
| `button` | 14 / 24 | semibold | A regular button's label (40 control). |
| `button_small` | 12 / 16 | semibold | A small button's label (32 control). |
| `label` | 12 / 16 | medium | A field's label, a chip, a hint. |
| `small` | 12 / 16 | regular | A note under a field, a subtitle under a row's title. |
| `caption` | 11 / 16 | regular | Key hints, version lines, the smallest print. |
| `accent` | 13 / 20 | medium, M PLUS 1 | The Japanese accent above a title. |
| `mono` | 12 / 16 | regular, Geist Mono | Logs, device paths, timings, keycaps. |
| `mono_large` | 14 / 20 | regular, Geist Mono | A command or key that is the subject of its line. |

A single-line control's label takes the control's content box as its line height (a 40 button has a 24 line inside 8 + 8 padding), so the label centres on the grid with no centring arithmetic.

## Space and shape

![Spacing, radii, icon sizes, component sizes, hairlines, the focus ring, concentric corners and columns, dark](../design/specimen/space-dark-1x.png)

*Top to bottom: the spacing scale as bars, the radii, the icon boxes, every fixed component size, the hairline and thin stroke, the focus ring (accent, and cold on a picked card), concentric corners, and a grid split in device pixels. Light: [space-light-1x.png](../design/specimen/space-light-1x.png).*

**The 4 px grid** (`rules.grid`). Every spacing, size, radius, icon size, line height, shadow offset and non-hairline border is a multiple of 4 logical pixels. The reason is the supported scales, 1, 1.25, 1.5 and 2 (`rules.scales`): 4 × 1.25 = 5 and 4 × 1.5 = 6, so a multiple of 4 is a whole number of device pixels at every one of them. A 6 px gap is 7.5 device pixels at 1.25 and blurs. `kubaos-tokens` refuses an off-grid length.

| Scale | Tokens |
|---|---|
| Spacing (`[space]`) | 4, 8, 12, 16, 20, 24, 32, 40, 48, 64 |
| Radii (`[radius]`) | 4, 8, 12, 16, 20, 24, and `full` for pills and circles |
| Icon sizes (`[icon]`) | 16, 20, 24, 32, 48 |
| Component sizes (`[size]`) | control sm 32, md 40, lg 48; row 44, two-line row 56; chip 24; switch 44 × 24, knob 12 off, 16 on; check 20, radio dot 8; progress 8; dialog 500; column 560; rail 236; log 232 |

- **Boxes land on device pixels.** Lengths on the grid do so by themselves. A box sized by its text (a label, a chip, a toast) is a fraction of a pixel wide; `kubaos-ui` rounds every such box up to the next device pixel (`snap`), and centring rounds its offset to a whole one (`centre`). A grid splits its width in device pixels, so its columns differ by one device pixel at most (`grid`).
- **Concentric corners.** A shape nested inside another with an inset smaller than the parent's radius takes the parent's radius minus the inset: a 24 dialog with 8 inside holds 16; a 16 list well with 4 of padding holds 12 rows. `kubaos_ui::look::concentric` computes it; a result off the radius scale means the nesting is wrong. The frame around the windows follows the same rule.
- **Hairlines are one device pixel** at every scale (`stroke.hairline_device_px`): dividers, card, field and well outlines, the switch track. They are the one length not on the grid.
- **The thin stroke** (`stroke.thin`, 2) is the focus ring's, the spinner's and a partial checkbox's dash. It is drawn as whole device pixels, rounded half up: 2, 3, 3, 4 at 1, 1.25, 1.5, 2. A tick (a full checkbox, a checked `ListRow`) is the `check` symbol, drawn with the symbol stroke like every other icon (see "Iconography"), so it matches the icons beside it.
- **The focus ring stands outside its control** (`focus.offset`, 4) and takes no layout space: focus moving never moves anything, and a focusable control's edge lines up with a plain one's. Its radius is the control's plus 4, concentric. A parent that clips (a scrolled list) pads 4 so the ring shows whole.
- **Elevation is two shadows** (`[shadow]`): `panel` and `dialog`, both 16 down and 48 blur in the `shadow` role, plus `glow` under a progress bar's fill in the fill's own colour. Nothing else casts a shadow; a surface sits higher by its container role (`surface_container` up to `surface_container_highest`), not by a stronger shadow.

## Motion

![Durations, curves and two animations frame by frame, dark](../design/specimen/motion-dark-1x.png)

*Each duration as a bar (32 px per 100 ms), each curve as a graph and as nine evenly timed frames, then a switch and a toast frame by frame. Light: [motion-light-1x.png](../design/specimen/motion-light-1x.png).*

Beautiful and fast. Every animation takes a duration and a curve **by what it is**, never by feel:

| Duration (`[duration.*]`) | ms | For |
|---|---|---|
| `instant` | 0 | The caret, text appearing as it is typed, reduce motion's moves. |
| `micro` | 100 | Hover and press state layers, a colour change under the pointer. |
| `short` | 200 | A switch's knob, a check mark, focus moving, a toast fading in. |
| `medium` | 300 | A panel or popup leaving, a log opening, a list reordering, a toast fading out. |
| `long` | 500 | A panel or dialog arriving, a page sliding in. |
| `slow` | 800 | A wallpaper fading in, the light/dark crossfade. |
| `scene` | 1200 | A whole scene changing once: the lock lifting, Kōbō's welcome settling. |

| Curve (`[easing.*]`) | Bézier | For |
|---|---|---|
| `standard` | 0.2, 0, 0, 1 | Small changes that stay on screen: toggles, colour, focus. |
| `standard_decel` | 0, 0, 0, 1 | Something already moving settling in: a list reordering. |
| `standard_accel` | 0.3, 0, 1, 1 | Something small leaving for good: a toast fading out. |
| `emphasized_decel` | 0.05, 0.7, 0.1, 1 | Things arriving: panels, dialogs, pages, the log opening. |
| `emphasized_accel` | 0.3, 0, 0.8, 0.15 | Things leaving: panels and dialogs closing. |
| `linear` | 0, 0, 1, 1 | Progress that tracks a real quantity, and spinners. |

- **Arriving is slower than leaving** (`long` in, `medium` out): you look at what arrives, and what leaves is already done with.
- **Speed** 0.5× to 2× divides every duration (`kubaos_ui::motion::Motion`). *Not enforced yet: lands in #154, the setting; `Motion` applies it once there is one.*
- **Reduce motion** caps every duration at `short` and turns moves into fades, or into nothing. *Not enforced yet: lands in #154, the setting; `Motion` caps the durations once there is one.*
- **What never animates:** typed text and the caret (`instant`), layout around focus (the ring is outside layout), a window's size or place, and anything while the user drags it.
- **Full refresh rate, always.** An animation that drops a frame is a bug, measured with the frame times, not judged by eye. *Not enforced yet: nothing measures frame times today, and no issue builds it yet.*

## Iconography

![KubaOS Symbols, the four we drew beside their Tabler relatives, the stroke table, and the comparison that picked Tabler, dark](../design/specimen/icons-dark-1x.png)

*Every symbol at 16, 20 and 24; "Drawn by us" with each of our four above the Tabler symbols it borrows from, all one colour, at 16, 20, 24 and 48; the stroke at each size and scale; then the comparison sheet, kept as the record of the choice. Light: [icons-light-1x.png](../design/specimen/icons-light-1x.png).*

- **One set: KubaOS Symbols.** They are Tabler's outline icons (`@tabler/icons` 3.49.0, MIT), picked by Kuba from a sheet of the same 40 glyphs in Material Symbols Rounded, Phosphor, Lucide and Tabler. `[symbols]` lists each, named for the Material Symbol it replaced; `crates/kubaos-ui/assets/make-symbols.sh` fetches them, hash-pinned.
- **SVGs, not a font.** A font's stroke scales with its size: Tabler's 2 unit stroke on its 24 unit grid is 1.33 px at 16 and 1.67 px at 20, between device pixels, so every line blurs. Each symbol is drawn from its SVG at runtime, with the stroke set for its size and scale: 2 units scaled to the size, rounded half up to whole device pixels, never under one (`stroke.symbol`, `kubaos_ui::icon::device_stroke`):

  | Size | 1× | 1.25× | 1.5× | 2× |
  |---|---|---|---|---|
  | 16 | 1 | 2 | 2 | 3 |
  | 20 | 2 | 2 | 3 | 3 |
  | 24 | 2 | 3 | 3 | 4 |
  | 32 | 3 | 3 | 4 | 5 |
  | 48 | 4 | 5 | 6 | 8 |

  16 at 1× stays 1 px: 1.33 rounds to 1, and 2 px would make the small symbols heavier than the text beside them.
- **Four are ours**, where Tabler has none: `hard_drive`, `wifi_lock`, `wifi_find` and `bluetooth_searching`. They are drawn on Tabler's 24 grid with its 2 unit stroke, round caps and joins, and built the way Tabler builds its own badged icons: Tabler's Wi-Fi fan at full size, its arcs stopped 2 units clear of folder-lock's lock badge or the search badge; bluetooth-x with a magnifier where its x is; server-2's proportions, light and slot for the drive. Their ink density matches their Tabler relatives'. A new symbol Tabler lacks is drawn the same way and shown beside its relatives on the icons page before it ships.
- **Sizes** are `[icon]`: 16, 20, 24, 32, 48. A symbol sits in a square box of its size, its ink 2 units clear inside, Tabler's margin. 16 beside small text and in chips, 20 in rows, fields and regular buttons, 24 in large buttons, dialogs and errors, 32 in empty states, 48 for a page's one big symbol.
- **Colour** is a role. An icon's alpha becomes the SVG's opacity, so a disabled icon dims like its label.
- **Optically centred.** A symbol's box is centred in its slot on a whole device pixel; the drawing's own optical centre is Tabler's grid, which ours follow.
- **Apps show as uniform tiles** in our UI: the app's icon on a tile of one size and radius, so a list of apps reads as one set, whatever each vendor drew. *Not enforced yet: lands in #93.*
- **The logo** is two things, as `docs/BRAND.md` says: the neon sign photo wherever it shows large (boot splash, GRUB, Kōbō, the README), and the vector クバ mark at small sizes (icons, tray, fastfetch). The neon sign is never redrawn as a symbol, and the mark is never used large.
- **The system icon theme** for GTK and Qt apps replaces Papirus with the theme closest to this look, picked in #152.

## Components

![Every component, every kind, size and state, dark](../design/specimen/components-dark-1x.png)

*The button and icon-button matrices first (kind × size × resting, hovered, pressed, focused, disabled), then fields, switches, checkboxes and radios, lists, chips, cards, dialogs, toasts and progress, empty states and errors with a log. Light: [components-light-1x.png](../design/specimen/components-light-1x.png).*

Two libraries implement one spec, with the same names:

- **`kubaos-ui`** (Rust, iced): Kōbō, Screens, the updater, a future Settings app. `crates/kubaos-ui`.
- **`KubaOS.Components`** (QML): the shell and the Command menu. Built in #147 to match the specimen.

**The shared name** of each component is the PascalCase of its `kubaos-ui` constructor (`icon_button` is `IconButton`); `progress::bar` is `ProgressBar`. The QML type has that name, and the geometry check reports components by it. A front end uses only these; a raw toolkit widget (iced's `button`, Qt Quick Controls' `Button`) in an app is a lint failure.

### What the geometry check compares

Each component reports its shared name, its variant (the words below that apply, in this order, joined by spaces) and the measures it fixes. The `identical` rule holds every instance of one name and variant to the same fixed measures, at every scale, with its icons and label in the same place along them. A measure a component doesn't fix follows its content or its parent. `KubaOS.Components` reports the same names, words and measures. `kubaos-specimen`'s tests fail when this table and the code disagree.

<!-- variants:start -->
| Name | Variant words | Fixed |
|---|---|---|
| `Button` | kind `Filled`, `Tonal`, `Text`, `Danger`; size `Sm`, `Md`, `Lg`; `leading-icon` or `trailing-icon` | height |
| `IconButton` | kind `Filled`, `Tonal`, `Text`, `Danger`; size `Sm`, `Md`, `Lg` | both |
| `Chip` | kind `Neutral`, `Selected`, `Accent`, `Cold`; `icon`; `removable` | height |
| `ListRow` | `one-line` or `two-line`; `icon` | height |
| `Field` | `icon` | height |
| `Spinner` | size `16`, `20`, `24`, `32`, `48` | both |
| `Switch` | none | both |
| `Checkbox` | none | height |
| `Radio` | none | height |
| `ProgressBar` | none | height |
| `Dialog` | tone `Normal`, `Danger` | width |
| `ErrorBlock` | `long` | none |
| `Page`, `Header`, `Section`, `Divider`, `Labelled`, `Note`, `List`, `EmptyList`, `Card`, `ChoiceCard`, `Toast`, `EmptyState`, `Log` | none | none |
<!-- variants:end -->

The set is **closed**. Every component takes the look (colours, mode, scale) and tokens, never a length, colour or duration of its own. A new kind, size or component changes this file first, then both libraries. **Composed blocks** (a settings row with a switch, a Wi-Fi row with its lock) are built from these, shadcn-style, in the app or the library, and add no new look.

Every interactive component has these states, shown in the specimen: **resting, hovered, pressed, focused, disabled**. Hover and press lay a state layer of the content colour over the fill (`opacity.hover` 0.08, `opacity.press` 0.14). The layer fades in and out over `micro`. *Not enforced yet: lands in #147 (QML) and #148 (iced; `kubaos-ui` switches it at once today).* Focus is the focus ring. Disabled dims the content to 0.38 and the container to 0.10 and takes no input.

### Layout

| Name | Rust | What |
|---|---|---|
| `Page` | `layout::page` | A whole page on the surface: 40 at the sides, 32 above, 24 below (Kōbō's panel). |
| `Header` | `layout::header` | A page's Japanese accent (`accent`), title (`title`), and one lede line (`body`) 8 below. |
| `Section` | `layout::section` | A titled part of a page: `heading`, an optional `body` line 4 under it, the content 16 below. |
| `Stack` | `layout::stack` | Children one above the other, a spacing token apart; start-aligned, or centred on whole device pixels. |
| `Row` | `layout::row` | Children side by side, a spacing token apart; top, centre or bottom. |
| `Grid` | `layout::grid` | Equal columns, a spacing token apart both ways, edges on device pixels. |
| `Split` | `layout::split` | Columns of set weights (`[1, 3]` is a label column a quarter wide). |
| `Divider` | `layout::divider` | A hairline across, in the soft outline. |
| `Fill` | `layout::fill` | Room that pushes what follows to the row's far end. |
| `Gap` | `layout::gap` | Empty room of one spacing token, where a stack needs one larger step. |

Gaps, paddings and fixed sizes take only `Space`, `Size` and `Inset` tokens, so a view built from these can't leave the scale. Text in a type style is `Text` (`text::text`); a symbol is `Icon` (`icon::icon`).

### Buttons

| Name | Rust | QML |
|---|---|---|
| `Button` | `button::button(label)` | `Button` |
| `IconButton` | `button::icon_button(glyph, label)` | `IconButton` |
| `Hint` | `hint::hint` | `Hint` |

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
| `Sm` | 32 (`control_sm`) | `button_small` | 16 | 4 | 12 (12) | Chrome around a screen, beside a field's label. |
| `Md` | 40 (`control_md`) | `button` | 20 | 8 | 24 (12) | A screen's own actions; as tall as a field. |
| `Lg` | 48 (`control_lg`) | `button_large` | 24 | 8 | 32 (16) | The one main action of a big page: Kōbō's Install. |

- Every button is a pill (`radius.full`). An optional icon leads the label, or trails it on a button that goes forward (Continue →). The icon sits in a box the label's line tall, centred on a device pixel, so an icon never changes a button's height.
- **`IconButton` is a circle as tall as the label button of its size** (32, 40, 48), in all four kinds. **Its label is a required argument.** It is the accessible name, and shows in a `Hint` while the pointer is over it or the keyboard is on it: a bubble in the inverse colours, `label` type, padding 8 × 4, radius 4, centred under the control 4 below its focus ring (8 below the control). An icon alone says nothing out loud, so an icon button can't be built without its label.

### Inputs

| Name | Rust | What |
|---|---|---|
| `Field` | `field::field(value, placeholder)` | One line of text, 40 tall (`control_md`), radius 12, a hairline outline plus the focus ring. `body_large` text. An optional 20 icon leads it, the text starting at 40. A placeholder is a hint, never a value that looks typed. States: normal, `Invalid` (error colour, the reason in the note), `Locked` (read-only, dimmed, never focused). Passwords show bullets. |
| `Labelled` | `field::labelled` | A control under its `label`-type label, with an optional small action at the label's right (Show, a small button) and an optional note under it. The label row is 32 tall whether or not it has an action, so fields side by side line up. |
| `Note` | `field::note` | The `small` line under a field: muted, or the error colour when invalid. |
| `Switch` | `switch::switch` | On or off, 44 × 24, the knob 12 off and 16 on, a hairline track. It takes effect at once; Space, Enter or a click toggles. |
| `Checkbox` | `check::checkbox` | Square, 20, a hairline edge. Empty, partial (a dash, for some of a group) or full. For several independent picks that apply together. |
| `Radio` | `check::radio` | A 20 disc with an 8 dot. One of a group, exactly one picked. |

A search over a list in the shell is a `KotodamaSearch` over a Kotodama app's Items, never a hand-rolled filter: see the QML rules in `docs/agents/code-quality.md`. A field that parses what is typed (a time, a password) isn't a search.

### Lists, chips and cards

| Name | Rust | What |
|---|---|---|
| `List` | `list::list` | Rows in a well: radius 16, padding 4, rows 4 apart, so the rows take radius 12 (concentric). The focus ring goes round the well; the row shows where the keyboard is. |
| `ListRow` | `list::list_row(title)` | 44 tall with one line (`row`), 56 with a subtitle (`row_two_line`). Padding 12, a 20 icon, the title in `body_strong`, subtitle in `small`, a detail in `mono` at the end, an optional check whose room is kept when unchecked so row ends line up. The selected row takes `secondary_container`. |
| `EmptyList` | `list::empty_list` | The well with nothing in it: one centred line. |
| `Chip` | `chip::chip` | A short fact or pick in a pill, 24 tall (`chip`), `label` type, an optional 16 icon 4 from the label, an optional close. Its left side pads 12, or 8 with an icon; its right pads 12, or nothing when the close button (a 24 circle) ends it. Kinds: `Neutral` (a fact), `Selected` (one of several picks), `Accent` (a tag that matters: Recommended, Default), `Cold` (rarely: demo, preview). |
| `Card` | `card::card` | A surface inside the page: the well's surface, a soft hairline, radius 16, 16 inside. |
| `ChoiceCard` | `card::choice_card` | One of a few cards to pick from. Picked: `secondary_container` and an accent hairline. Its focus ring is cold, since the accent already means picked. |

### Feedback

| Name | Rust | What |
|---|---|---|
| `Dialog` | `dialog::dialog` | A question over a dimmed screen: one width (500, `dialog`), padding 32, radius 24, the dialog shadow. A 24 icon and a `headline` title, the `body` 12 under it, the buttons 32 under that: the dismiss button (Cancel) on the left, the actions on the right, the one Enter presses last. Tone `Normal` or `Danger`. |
| `Scrim` | `dialog::scrim` | Everything behind a dialog, dimmed (`opacity.scrim`). |
| `Toast` | `toast::toast` | One `body` line saying what just happened, in the inverse colours so it reads over anything, an optional 20 icon 12 from the text, padding 16 × 12, radius 12. It arrives over `short` and leaves over `medium`. *Not enforced yet: lands in #147 and #148 (`toast` is static today).* |
| `ProgressBar` | `progress::bar` | Work that knows how far it is: 8 tall (`progress`), the fill glowing in its own colour. The fill follows the work on the `linear` curve. *Not enforced yet: lands in #147 and #148 (the bar jumps today).* |
| `Spinner` | `progress::spinner` | Work that doesn't know: an icon size (16, 20 or 24 beside text; 32 and 48 for a page's one spinner), a thin arc over a faint full ring. The caller passes the time; the arc turns and breathes with it. |
| `EmptyState` | `empty::empty_state` | A place with nothing in it, centred: a 32 icon, 12 below it a `subheading` title, 4 below that a `body` line saying what to do, and 16 below that an optional action. Never a bare "No items". |
| `ErrorBlock` | `error::error_block` | What broke and what to do next: a 24 error icon, 16 from the message in `subheading` (`body_strong` past 90 characters, since a whole command line in 16 px semibold is a wall), the hint in `body` 4 under it, on the error container with an error hairline, radius 16, padding 20. |
| `Log` | `error::log` | The log behind an error or a progress bar, `mono`, newest line at the bottom, opened on demand. The caller sets its height as an `Extent`; `log` (232) is the size the specimen and Kōbō use. |
| `FocusRing` | `focus::focus_ring` | The ring every focusable component draws (see "Focus"). An app never draws its own. |

Rules the feedback components carry:

- **Destructive actions confirm.** Erasing, deleting or overwriting asks first in a `Dialog` with tone `Danger` and a `Danger` button naming the action ("Erase and install"), with Cancel as the dismiss button. The sentence naming what goes is exact and calm (`docs/BRAND.md`, rule 4).
- **Errors carry a hint and a log.** An `ErrorBlock` says what broke and the next action; the `Log` behind it holds the detail, with a way to save it. An error with no next action isn't finished.
- **One `Filled` button per screen**, so the next step is never in doubt.

## Behaviour

### The keyboard

Everything works from the keyboard, in every app, the same way. Shortcuts are on screen where they're needed (a `caption` hint, as the specimen's bar shows). The mouse works, but you never need it.

| Key | Does |
|---|---|
| Tab, Shift+Tab | Focus to the next or previous control, in reading order. |
| ↑ ↓ | Move in a list, a menu or a column of choices. In a text field with a list under it (the Command menu, a search), they move in the list. |
| ← → | Move between choices side by side, cards in a row, cells in a grid; in a text field, the caret (Ctrl or Alt by word, Shift selecting). |
| Ctrl+N, Ctrl+P / Ctrl+J, Ctrl+K | Down and up in a list, also from inside a text field. |
| h j k l | Left, down, up, right in a list or grid that has no text field taking the keys. |
| Home, End | First and last item of a list; start and end of a field's text. |
| Page Up, Page Down | A page of a list at a time. |
| Enter | Press the focused button or open the focused row; elsewhere, go on: the screen's `Filled` action. Fires once per press, never on key repeat. |
| Space | Toggle the focused switch, checkbox or radio; press the focused button. In a text field, a space. |
| Esc | Go back one step: close the popup, the dialog (as its dismiss button), the panel, the menu. Never destroys anything. |

A new surface implements the whole map, not the keys it happens to need. Today neither front end has all of it:

- **Kōbō** (`translate` in `crates/kubaos-kobo/src/app.rs`): Tab and Shift+Tab, the arrows, Home and End, Page Up and Page Down, Enter once per press, Space, Esc. Not yet: Ctrl+N, Ctrl+P, Ctrl+J, Ctrl+K and h j k l; they land in #148.
- **The shell's lists** (`config/quickshell/components/NavList.qml`): ↑ ↓, Ctrl+N, Ctrl+P, Ctrl+J, Ctrl+K, j and k where a list takes them, Enter. Not yet: Home, End, Page Up, Page Down, h and l; they land in #147.

### Focus

- **Always visible.** The focused control draws the focus ring: 4 outside the control (`focus.offset`), concentric (the control's radius plus 4), its outer stroke `stroke.thin` rounded to whole device pixels. It is the accent, or the cold accent on a `ChoiceCard`, where the accent already means picked.
- **Focus never moves layout.** The ring takes no space; a focused control is the same box as a resting one. The geometry check compares them.
- **One place at a time.** The app owns where focus is and passes it to components as a flag; a list shows its focused row by its own selection, and the ring goes round the list.
- **Focus starts on the first thing to do**: the field to type in, or the `Filled` button. Opening a dialog moves focus into it; closing it puts focus back where it was. *Not enforced yet: lands in #147 (QML) and #148 (iced).*
- Focus moves over `short` with `standard`. *Not enforced yet: lands in #147 (QML) and #148 (iced); the ring jumps today.*

### Scaling

- **Sizes are logical pixels.** A display's scale multiplies them; the supported scales are 1, 1.25, 1.5 and 2.
- **Multiples of 4 are whole device pixels at all four**, which is why the grid is 4. Hairlines are one device pixel and thin strokes are rounded to device pixels at each scale, never scaled into fractions. Icons' strokes follow the table in "Iconography".
- **Every page is checked at every scale**, light and dark (see "Checks"). Pixel perfect at 1× and blurred at 1.25× is a failure.
- **Each monitor has its own scale.** A window takes the scale of the output it is on and redraws when it moves to another; the shell's panels are per output. Nothing assumes one scale for the session.
- **Nothing resizes your windows.** Panels, popups, the OSD and notifications float over windows and never push them. The bar and the frame reserve a fixed space; nothing else does.

### Notify or show inline

- **Inline** when the user is looking at the thing: the result of their action shows where they acted. A wrong password is a `Note` under the field; a failed install step is an `ErrorBlock` on the screen; a list with nothing in it is an `EmptyState`.
- **A toast** for a short confirmation of something just done that left no other trace on screen ("Copied. Evidence secured."). It goes by itself and never carries an action the user must take.
- **A notification** only when the user is elsewhere, or the news outlives the view that started it: a download finished, an update is ready, the battery is low. It is worth an interruption, or it isn't sent.
- **Never both** for one event. The same fact twice is noise.

## Settings

- **Every setting is a Command menu Item**, and is written to the User layer, so it can be found, changed and kept like any other. A feature ships with its Items (`CLAUDE.md`).
- **A Settings panel in the shell** groups them: Appearance, Displays, Keyboard, Phone, Updates and the rest (#154). It may become an app of its own later, on `kubaos-ui`.

## Terminals, boot and the rest

What can't use the libraries still follows the tokens as far as it can go:

- **The terminals** (Ghostty, foot, tmux, Neovim, btop, lazygit, bat, fuzzel, oh-my-posh, fastfetch) take the colour roles through matugen templates generated from `design/tokens.toml`, Geist Mono as their font, and KubaOS Symbols' glyphs where they draw icons (#152).
- **Plymouth and GRUB** are always dark and take their values from the tokens (#153). The splash is the neon sign (`docs/BRAND.md`, "Boot").
- **The cursor** is recoloured from the palette (`kubaos-cursor`).
- **GTK and Qt apps** get the colour roles and an icon theme close to KubaOS Symbols (#152).

## Writing

The words are `docs/BRAND.md`'s: read its "Voice" before writing anything a person reads. What it means for UI copy:

- **Sentence case** everywhere: titles, buttons, labels, menu Items. "Save log", never "Save Log".
- **A label stays a label.** Field names, settings' titles and buttons that only say what they do are plain; the attitude goes in messages and subtitles (BRAND.md rule 2).
- **Name things with `GLOSSARY.md`'s terms**: a Suite, the User layer, Kōbō, never a synonym.
- **An error says the cause and the next action**: "Fuck. The disk is in use, so nothing got written. Free it and try again." The `ErrorBlock`'s message is the cause, its hint the next action.
- **The expensive sentences stay plain**: the one naming the disk about to be erased, the passphrase prompts (BRAND.md rule 4).
- **Buttons speak as the user, messages as the OS** (BRAND.md rule 6). A button names its action: "Erase and install", not "OK".

## Checks

Three checks keep every front end on this file. The first two run in `cargo test --workspace` (`just test`), in `nix flake check` (`crates/checks.nix`) and in `just design-check`; a PR that fails either isn't done.

### The lint

`kubaos-design lint` reads every `.qml` file and the sources of every Rust crate that depends on iced. It skips their tests, the files `kubaos-tokens` generates, and `config/quickshell/theme/Theme.qml`, the shell's hand-written token file until #147 generates its replacement: those are where the values belong. Each rule fails on a value written out where a token belongs:

| Rule | Fails on |
|---|---|
| `colour` | A hex colour string; `Qt.rgba`, `Qt.hsla` and friends; a named colour (`"white"`) bound to a colour; iced's `Color::from_rgb*` with numbers, `Color::WHITE`, `color!`, a `Color { r, g, b, a }` literal. `"transparent"` and a colour converted from a token pass. |
| `space` | A number in a gap, padding, margin or inset: QML's `spacing`, `*Margin`, `padding`; iced's `.padding(`, `.spacing(`, `Padding`. |
| `size` | A number in a width, height, position, stroke or icon size: QML's `width`, `implicitHeight`, `Layout.preferred*`, `border.width`; iced's `.width(`, `Length::Fixed(`, `Length::from(`, `Size::new(`, `Pixels(`, `Pixels::from(`, `40.0.into()` in those places. A size given to iced's window API (`window::Settings`, `min_size`, `max_size`, `with_size(`) is the window's, not layout, and passes. Any other window size, or a canvas program's coordinates in units of its own size, carries a marker. |
| `radius` | A number in a corner radius. |
| `duration` | A number in QML's `duration` or `…Duration`; iced's `Duration::from_millis(`, `Duration::new(` and friends, up to the longest duration token (`scene`, 1200 ms): any of them could be motion, whatever it is called. A longer one can't, and passes. A shorter timeout, poll or caret blink is behaviour, not motion, and carries a `design-allow(duration)` marker saying so. |
| `font` | A family named in a string, a pixel or point size, a weight named in place (`Font.Medium`), a line height, letter spacing, in a binding or a `Qt.font({ … })`. |
| `component` | A toolkit's own control where the library has the component: Qt Quick Controls' `Button`, `TextField`, `Switch` and the rest, in a file that imports them; iced's `button`, `text_input`, `text`, `checkbox`, `row`, `column`, `row!`. `kubaos-ui` and `KubaOS.Components` themselves may build on them. |

**A custom number property** set to a literal counts where it is used: `property int rowH: 44` bound to a `height`, alone or in a sum, is a size, whatever it is called. One that only scales something (`parent.width * remaining`) is a factor, not a length. A property used only inside a product (`cell: 40` read only as `cell * 7`) is missed: the rule can't tell a length times a count from a factor. Fixing one of these files finds it.

Zero is never a literal, and neither is halving or doubling a token (`/ 2` for a centre or a pill's radius, `Theme.spacing.md * 2` for both sides), or a whole count of a named length (`cell * 7` for seven columns). A count of a number (`24 * 7`) is still a literal. Comments and strings that aren't colours or families are never read.

**The baseline.** The front ends that predate this file break these rules hundreds of times, until their alignment issues (#148 to #153) fix them. `design/lint-baseline.toml` holds how many violations of each rule each file may still have:

```toml
["config/quickshell/modules/osd/Osd.qml"]
size = 5
radius = 2
font = 1
```

It only shrinks:

- **A file over its count fails**, and the lint lists every violation of that rule in the file, since any of them may be the new one.
- **A file under its count fails too**, until `just design-baseline` writes the lower count, so a fix can't be quietly undone later.
- **No command raises a count.** `kubaos-design baseline --init` writes the first baseline only when git has never had the file, so deleting it and starting again fails too. A count going up is a hand edit to the baseline, which a reviewer sees and refuses unless the lint's rules changed in the same PR.

```text
$ just design-check
config/quickshell/modules/osd/Osd.qml: 3 radius violation(s), the baseline allows 2; fix the new one, or mark it `// design-allow(radius): why`
  config/quickshell/modules/osd/Osd.qml:36:13: radius: `radius: 6`
  config/quickshell/modules/osd/Osd.qml:87:21: radius: `radius: 4`
  config/quickshell/modules/osd/Osd.qml:93:25: radius: `radius: 4`
kubaos-design lint: 166 file(s), 806 violation(s), 805 in the baseline: FAILED
```

**A justified literal** carries a marker on its own line right above it, in the file's line comment, naming the rule and the reason:

```qml
// design-allow(size): the tray icon's size is the StatusNotifierItem spec's, not ours.
implicitSize: 22
```

A marker sits on the line directly above the code it allows, with no other comment between, and allows the rules it names on that one line. It must say why after the colon. A marker with no reason, an unknown rule, or nothing below it to allow fails the lint, so markers can't pile up unread. A marker is for a value that isn't a design value (a protocol's size, an icon font in the specimen's comparison sheet). A design value that has no token gets a token.

### The geometry check

`kubaos-specimen check` lays every page of the specimen out at 1, 1.25, 1.5 and 2, light and dark: 48 frames. A frame is the page's boxes, each with what it is, as `kubaos-ui`'s probes report them. Every layout primitive, component, text and icon wraps itself in a probe that lays out and draws exactly as its content. The rules (`kubaos-geometry`):

| Rule | Fails when |
|---|---|
| `device-grid` | An edge falls between device pixels. For a text, only where it starts counts. |
| `scale-grid` | A component's fixed width or height isn't a multiple of 4, a layout's gap isn't on the spacing scale, an icon's box isn't square or on the grid. |
| `sibling-gap` | Two children of a row, stack or grid aren't exactly the layout's gap apart. |
| `alignment` | Children of a stack don't share their left edge, or their centre when centred; children of a row don't share their top, centre or bottom; a grid's cells leave their column or row. Centres may differ by half a device pixel, where an odd one had to go to a side. |
| `identical` | Two instances of one component and variant (`Button`, `Filled Md`) differ in a measure the component fixes (a button's height, an icon button's both), at any scale, or place their icons and label differently along it. Focused, hovered and disabled are the same variant, so focus moving anything fails here. |
| `clipped-text` | A text needs more room than its box, or a text or icon sticks out of a box around it. |
| `icon-centre` | An icon alone in its slot, or in a centred row, is more than half a device pixel off the slot's centre. |
| `mode-invariant` | A page's light frame has other boxes than its dark one. Words may differ; where each box sits may not. |

```text
$ cargo run -p kubaos-specimen -- check
space-dark-1.25x device-grid: 28.8×16 at (66.4, 294.8) has edges between device pixels: top 368.5
    at Stack[2] > Page[0] > Stack[1] > Stack[0] > Section[0] > Stack[1] > Grid[0] > Row[1] > Text "4 px"
kubaos-specimen check: 48 frame(s), 120 finding(s)
```

That run is iced centring a row's children by half a device pixel at 1.25, before `kubaos-ui` moved them onto whole ones. `kubaos-specimen check DIR` also writes each frame to `DIR` as JSON.

**Optically centred** is two things. A symbol's box is centred in its slot to the device pixel, which the check measures. Its drawing is optically centred in that box by Tabler's grid, which the four we drew follow and the icons page shows beside their relatives.

**Other toolkits.** The checker reads boxes, not widgets, so the shell feeds it the same way (#147): `KubaOS.Components` dumps each page's item tree as a frame, and `kubaos-design geometry FRAME.json…` checks the frames together. A frame is JSON:

```json
{
  "page": "osd", "mode": "dark", "scale": 1.25,
  "root": {
    "role": "layout", "axis": "x", "gap": 8, "align": "centre",
    "bounds": { "x": 0, "y": 0, "width": 96, "height": 40 },
    "children": [
      { "role": "component", "name": "IconButton", "variant": "Text Md",
        "fixed": { "width": true, "height": true },
        "bounds": { "x": 0, "y": 0, "width": 40, "height": 40 },
        "children": [
          { "role": "icon", "name": "volume_up",
            "bounds": { "x": 10, "y": 10, "width": 20, "height": 20 } }
        ] }
    ]
  }
}
```

Bounds are logical pixels from the page's top left. The roles are `box` (a toolkit's container that claims nothing), `layout` (`axis` `x` or `y`, `gap`, `align` `start`, `centre` or `end`: a `RowLayout`, a `ColumnLayout`), `grid` (`columns`, `gap`, `align`), `component` (the shared `name`, the `variant`'s words, which measures are `fixed`), `text` (`content`, and `natural`: the size its laid-out text needs, `implicitWidth` and `implicitHeight` in QML), `icon` (`name`), and `spacer` (`fill`). A component's name and variant are this file's, so a `Button` in QML and one in iced are held to the same measures. That same dump answers #147's cross-toolkit test: the iced and the QML frame of a component must match.

### Screenshot diffs

The specimen's 1× shots are checked in under `design/specimen/`. A PR that changes how anything looks regenerates them with `kubaos-specimen shots DIR` and commits the ones that changed, so the change shows in review as an image diff. Each front end's alignment issue (#148 to #153) attaches its screens before and after, at every scale, light and dark; diffing them on the test machine comes with those issues.

## Changing the design

- **A new kind, size, component or value changes this file first**, then `design/tokens.toml`, then both libraries, in one PR, with the specimen showing it.
- **A rule a front end can't follow changes for everyone.** There is no exception column: if the shell can't keep a rule, the rule changes here and every front end follows the new one.
- **Values change in `design/tokens.toml` only**, then `cargo run -p kubaos-tokens -- write`. A test fails while a generated file is stale.
- **Every decision keeps its reason.** A rule with no reason here is a candidate for deletion, not for following.
