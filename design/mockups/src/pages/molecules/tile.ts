// The Tile page: `docs/design/components/tile.md`. A quick toggle: the control center's Wi-Fi,
// Bluetooth and the rest, two to a row. Lit as a switch is, off on its container and on filled with
// the accent; a tile with more behind it ends in a quiet detail arrow.

import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { type TileOptions, tile } from "../../lib/tile.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const STATUS: Words = { short: "Off", long: "Home network 5 GHz, through the living room" };
const TITLE: Words = { short: "Wi-Fi", long: "Wallpaper motion" };

/** The control center's body: its panel less the container padding. */
const PANEL = "calc(var(--size-panel) - 2 * var(--container-0-padding))";
/** One tile's room: half the body, less half the gap between two. */
const HALF = "calc((var(--size-panel) - 2 * var(--container-0-padding) - var(--space-s8)) / 2)";

function boxed(options: TileOptions): HTMLElement {
  return within(HALF, tile(options));
}

const nothing = () => undefined;

const WIFI: TileOptions = { glyph: "wifi", title: "Wi-Fi", subtitle: "Home 5G", on: true };
const BLUETOOTH: TileOptions = { glyph: "bluetooth", title: "Bluetooth", subtitle: "Off" };
const KEYBOARD: TileOptions = { glyph: "keyboard", title: "Keyboard", subtitle: "PL · 1 of 2", on: true, detail: nothing };
const PROXY: TileOptions = { glyph: "key", title: "Proxy", subtitle: "Off", detail: nothing };

/** Two tiles side by side, sharing the row across all of its room. */
function pairRow(...tiles: HTMLElement[]): HTMLElement {
  return filling(row("s8", "centre", ...tiles.map((each) => filling(within("100%", each)))));
}

/** The control center's quick toggles, live: a press toggles, an arrow says what it opens. */
function toggles(): HTMLElement {
  const opened = text("A press toggles; an arrow opens what is behind it.", "small", "on_surface_variant");
  const says = (what: string) => () => {
    opened.textContent = what;
  };
  const grid = stack(
    "s8",
    "start",
    pairRow(tile({ ...WIFI, onToggle: (on) => (on ? "Home 5G" : "Off") }), tile({ ...BLUETOOTH, onToggle: (on) => (on ? "On" : "Off") })),
    pairRow(tile({ glyph: "notifications", title: "Do not disturb", subtitle: "Off", onToggle: (on) => (on ? "On" : "Off") }), tile({ glyph: "motion_photos_auto", title: "Wallpaper motion", subtitle: "On AC", on: true })),
    pairRow(tile({ ...KEYBOARD, detail: says("Keyboard's arrow: its layouts open in the control center.") }), tile({ ...PROXY, onToggle: (on) => (on ? "work · 10.0.0.1" : "Off"), detail: says("Proxy's arrow: the tunnel's details open under the tiles.") })),
  );
  return within(PANEL, stack("s16", "start", filling(grid), opened));
}

const ROWS = [
  { name: "Off", options: BLUETOOTH },
  { name: "On", options: WIFI },
  { name: "Off, with a detail arrow", options: PROXY },
  { name: "On, with a detail arrow", options: KEYBOARD },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

function cell(options: TileOptions, state: MatrixState): HTMLElement {
  return boxed({ ...options, ...stateOptions(state as Shown) });
}

/** An off tile and an on one in `state`: what the lighting moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => within(PANEL, pairRow(tile({ ...BLUETOOTH, ...stateOptions(state) }), tile({ ...WIFI, ...stateOptions(state) })));
}

const ON = "an off tile and an on one";

const DESCRIBED =
  "`size.row_two_line` (56) tall at the control radius, filling its room, `space.s12` inside: a symbol (`body_strong`'s 20), then `space.s12` on its title in `body_strong` over its status line in `small`, `space.s4` apart, each on one line ending in an ellipsis where it doesn't fit, the two centred down the tile. A tile with no status line centres its title alone. Off, it is `surface_container_high`, its title `on_surface` and its symbol and status line `on_surface_variant`. On, it is filled with the accent: at night the unlit fill, `primary_fill`, its symbol, title and status line `on_primary_fill`; by day the same roles in daylight colours.";

export default page({
  level: "molecules",
  slug: "tile",
  title: "Tile",
  summary: "A quick toggle: a symbol, a title and a status line, filled with the accent while on.",
  live: toggles,
  description: [
    "Rust `tile::tile`, QML `Tile`. Press a tile, or Tab to it and press Enter or Space; on Keyboard or Proxy press the arrow, or → from the keyboard. The control center's quick toggles, two to a row `space.s8` apart.",
    DESCRIBED,
    "A press, Enter or Space toggles it: its fill and content fade from one look to the other over `short` on `standard`, as a switch's do. What it toggles may take a moment (the proxy): it shows the state it is going to at once and says so in its status line, Turning on…, then the state it reached, or Didn't switch.",
    "Its states follow the neon, as a switch's do. Off and lit, its fill rises to `surface_container_highest` and the tube burns inside its edge, `primary` at `focus.tube`, with the glow outside at night; pressed, the glow brightens, and by day the tone tints it (`opacity.press_tint`). On and lit, the core (`primary_fill_line`) burns inside the fill's edge, `stroke.thin` wide, with the glow outside, and its content brightens to `on_primary_fill_lit`; pressed, the fill lifts to `primary_fill_pressed`. Disabled (nothing to toggle: no Bluetooth adapter), the fill is `on_surface` at `opacity.disabled_container` and its content at `opacity.disabled_content`, on or off.",
    "A tile with more behind it ends in a detail arrow (Keyboard's layouts, Proxy's tunnel): a circle `size.control_sm` across, `space.s12` from the tile's end and centred down it, a 16 chevron (`chevron_right`) in the tile's symbol colour, and no fill. It is a small control that stays quiet: lit, its own tube burns round it with the glow outside at night, and the tile gives way while the pointer is on it, so only one is lit; pressed, its tube runs white-hot, and by day the tone tints it. → opens it from the keyboard; it is no stop for Tab. The words end `space.s12` before it.",
  ],
  variants: [
    { name: "Off", for: "Something switched off: Bluetooth.", sample: () => boxed(BLUETOOTH) },
    { name: "On", for: "Something switched on: Wi-Fi, its network in the status line.", sample: () => boxed(WIFI) },
    { name: "With a detail arrow", for: "More behind it than on and off: the keyboard's layouts, the proxy's tunnel.", sample: () => boxed(KEYBOARD) },
    { name: "Without a status line", for: "A toggle whose title says it all: the title centred alone.", sample: () => boxed({ glyph: "dark_mode", title: "Dark mode", on: true }) },
    { name: "The detail arrow, lit", for: "The pointer on the arrow: its own tube, the tile at rest.", sample: () => boxed({ ...KEYBOARD, detailPinned: "lit" }) },
    { name: "The detail arrow, pressed", for: "Pressed: its tube white-hot; letting go opens what is behind it.", sample: () => boxed({ ...PROXY, detailPinned: "pressed" }) },
  ],
  sizes: doesntApply("a tile is always a two-line row tall (`size.row_two_line`) and fills the room it is given: half a panel's body in the control center, or the whole row for an odd one."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "on is its own row here: a tile is on or off, not picked.",
      loading: "a toggle that takes a moment shows the state it is going to at once, and its status line says so: Turning on….",
      empty: "a tile always has a title and a symbol.",
      error: "a toggle that failed says so in its status line, Didn't switch, back in the state it was in.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "Off: `surface_container_high`, its title `on_surface`, its symbol, status line and arrow `on_surface_variant`. On: `primary_fill`, its content `on_primary_fill`. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on. Off: the fill rises to `surface_container_highest` and the tube burns inside its edge with the glow outside. On: the core burns inside the fill's edge with the glow outside, and its content brightens to `on_primary_fill_lit`. By day the tube or a dark line, with no glow.",
      pressed: "Lit and pressed: the glow `focus.pressed_glow` times the lit one; off, by day the tone tints it at `opacity.press_tint`; on, the fill lifts to `primary_fill_pressed`. Letting go toggles it.",
      disabled: "Grey, on or off: the fill `on_surface` at `opacity.disabled_container`, its content and the arrow at `opacity.disabled_content`. No press, no stop for Tab, and the arrow takes none.",
    },
  },
  focus: {
    tubes: [
      { name: "Off", tone: "accent", filled: false, content: false },
      { name: "On", tone: "accent", filled: true, content: false },
      { name: "The detail arrow", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn: off, the tube inside the tile's edge; on, the core inside the fill's edge, its content `on_primary_fill_lit`; the glow outside both. The detail arrow's own tube burns round it while the pointer is on it.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. Off, the tube is `primary_glass`; on, the line is `primary_fill_glass` on the fill and the content stays `on_primary_fill`; no glow.",
    },
    signsOff: (index: number) => boxed({ ...ROWS[index]!.options, pinned: "signs-off" }),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => boxed({ ...BLUETOOTH, title: words }) },
      { name: "The status line", words: STATUS, sample: (words: string) => boxed({ ...WIFI, subtitle: words }) },
    ],
    missingPicture: doesntApply("a tile's symbol is one of KubaOS Symbols, always there; it changes with the state it shows (Wi-Fi's strength, a lock on the proxy)."),
    counts: [
      { name: "Two to a row", for: "The control center's tiles, each half the body.", sample: () => within(PANEL, pairRow(tile(WIFI), tile(BLUETOOTH))) },
      { name: "One across a row", for: "An odd one out (Keyboard while there is no Proxy) takes the whole row, so the grid has no hole.", sample: () => within(PANEL, stack("s8", "start", pairRow(tile(WIFI), tile(BLUETOOTH)), filling(within("100%", tile(KEYBOARD))))) },
    ],
  },
  motion: {
    arrives: doesntApply("a tile comes with the surface it is on."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      {
        name: "Turning on",
        what: "A press, Enter or Space on an off tile: its fill fades to the accent's and its content to `on_primary_fill` over `short` on `standard`, as a switch's do, and its line turns from the tube to the core. Never the flicker. Reduced motion keeps the fade, which is within its cap.",
        tracks: [{ part: "on", on: "Bluetooth", target: ".k-tile", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => boxed(BLUETOOTH),
      },
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on it, it lights up over `short` on `standard`: off, its fill rises and the tube and glow fade in; on, the core and glow fade in and its content brightens.", tracks: lights(".k-tile", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-tile", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "A lit tile pressed: the glow brightens over `micro`, an on tile's fill lifts and by day an off one's tints; letting go toggles it.", tracks: press(".k-tile", ON), reduced: "same", back: "plays", sample: pair("lit") },
      {
        name: "The pointer onto the arrow",
        what: "The pointer moves from the tile onto its detail arrow: the tile goes back to rest over `short` as the arrow's tube lights, so one thing is lit at a time.",
        tracks: [
          { part: "the tile", on: "Keyboard", target: ".k-tile", property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the arrow", on: "Keyboard", target: ".k-tile-detail", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => boxed(KEYBOARD),
      },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space"], does: "Toggles it, once per press, never on key repeat." },
      { keys: ["→"], does: "On a tile with a detail arrow, opens what is behind it, as the arrow does." },
      { keys: ["Tab", "Shift+Tab"], does: "To the next or previous tile, in reading order: across a row, then down. The arrow is no stop of its own." },
    ],
    order: "Each tile is one stop in the surface's reading order; a disabled one is skipped.",
    lands: "When the control center opens, on its first tile, Wi-Fi. Toggling keeps the keyboard on the tile; the arrow's view takes it where that view says.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a detail arrow", for: "Its height, padding, symbol, the gap to its words, the two lines, and the arrow.", sample: () => boxed(KEYBOARD) },
      { name: "Two to a row", for: "The gap between two tiles.", sample: () => within(PANEL, pairRow(tile(WIFI), tile(BLUETOOTH))) },
    ]),
    measures: [
      { what: "Its height, a two-line row's.", token: "size-row_two_line" },
      { what: "Inside its sides, between the symbol and the words, and the arrow from its end and from the words.", token: "space-s12" },
      { what: "Between the title and the status line.", token: "space-s4" },
      { what: "The symbol, `body_strong`'s.", token: "type-body_strong-icon" },
      { what: "The title's line.", token: "type-body_strong-line" },
      { what: "The status line's line.", token: "type-small-line" },
      { what: "The detail arrow, a small control across.", token: "size-control_sm" },
      { what: "The arrow's chevron.", token: "type-button_small-icon" },
      { what: "Between two tiles in a row and between rows.", token: "space-s8" },
      { what: "Its corners.", token: "radius-control" },
    ],
    reflow: {
      described: "A tile fills the room it is given at every window size; its title and status line end in an ellipsis where they don't fit. The control center is the shell's panel at every screen size, so its tiles are always half its body.",
      sample: () => within("100%", stack("s8", "start", pairRow(tile({ ...WIFI, subtitle: shown(STATUS) }), tile(KEYBOARD)), text("Two tiles across the window.", "small", "on_surface_variant"))),
    },
  },
});
