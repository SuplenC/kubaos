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
  const opened = text("Nothing opened yet.", "small", "on_surface_variant");
  const says = (what: string) => () => {
    opened.textContent = what;
  };
  const grid = stack(
    "s8",
    "start",
    pairRow(tile({ ...WIFI, onToggle: (on) => (on ? "Home 5G" : "Off") }), tile({ ...BLUETOOTH, onToggle: (on) => (on ? "On" : "Off") })),
    pairRow(tile({ glyph: "notifications", title: "Do not disturb", subtitle: "Off", onToggle: (on) => (on ? "On" : "Off") }), tile({ glyph: "motion_photos_auto", title: "Wallpaper motion", subtitle: "On AC", on: true })),
    pairRow(tile({ ...KEYBOARD, detail: says("Keyboard's layouts open.") }), tile({ ...PROXY, onToggle: (on) => (on ? "work · 10.0.0.1" : "Off"), detail: says("Proxy's tunnel opens.") })),
  );
  return within("var(--size-column)", stack("s16", "start", filling(grid), opened));
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

export default page({
  level: "molecules",
  slug: "tile",
  title: "Tile",
  lede: "A quick toggle: a symbol, a title and a status line, filled with the accent while on.",
  code: { rust: "tile::tile", qml: "Tile", unbuilt: "primary_fill look: not built yet" },
  stage: { live: toggles, hint: "press a tile or its arrow; Enter, Space, →", daylight: true },
  variants: [
    { name: "Off", for: "something switched off: Bluetooth", sample: () => boxed(BLUETOOTH) },
    { name: "On", for: "switched on: Wi-Fi, its network below", sample: () => boxed(WIFI) },
    { name: "With a detail arrow", for: "more behind it: layouts, a tunnel", sample: () => boxed(KEYBOARD) },
    { name: "Without a status line", for: "the title centred alone", sample: () => boxed({ glyph: "dark_mode", title: "Dark mode", on: true }) },
    { name: "The detail arrow, lit", for: "its own tube, the tile at rest", sample: () => boxed({ ...KEYBOARD, detailPinned: "lit" }) },
    { name: "The detail arrow, pressed", for: "its tube white-hot; letting go opens it", sample: () => boxed({ ...PROXY, detailPinned: "pressed" }) },
  ],
  sizes: doesntApply("always a two-line row tall, filling the room it is given."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "on is its own row here",
      loading: "status line says Turning on…",
      empty: "a tile always has a title and a symbol",
      error: "status line says Didn't switch, back where it was",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    looks: {
      rest: "off: `surface_container_high`; on: `primary_fill`, `on_primary_fill`",
      lit: "tube, or on the core, inside its edge; glow at night",
      pressed: "glow × `focus.pressed_glow`; on lifts to `primary_fill_pressed`",
      disabled: "grey, on or off: no press, no Tab stop",
    },
  },
  focus: {
    tubes: [
      { name: "Off", tone: "accent", filled: false, content: false },
      { name: "On", tone: "accent", filled: true, content: false },
      { name: "The detail arrow", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "off: the tube; on: the core; the glow outside both",
      signsOff: "off: `primary_glass`; on: `primary_fill_glass`; no glow",
    },
    signsOff: (index: number) => boxed({ ...ROWS[index]!.options, pinned: "signs-off" }),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => boxed({ ...BLUETOOTH, title: words }) },
      { name: "The status line", words: STATUS, sample: (words: string) => boxed({ ...WIFI, subtitle: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, always there."),
    counts: [
      { name: "Two to a row", for: "the control center's tiles, each half the body", sample: () => within(PANEL, pairRow(tile(WIFI), tile(BLUETOOTH))) },
      { name: "One across a row", for: "an odd one takes the whole row: no hole", sample: () => within(PANEL, stack("s8", "start", pairRow(tile(WIFI), tile(BLUETOOTH)), filling(within("100%", tile(KEYBOARD))))) },
    ],
  },
  motion: {
    arrives: doesntApply("a tile comes with the surface it is on."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      {
        name: "Turning on",
        what: "Its fill fades to the accent's and its content to `on_primary_fill` over `short` on `standard`.",
        tracks: [{ part: "on", on: "Bluetooth", target: ".k-tile", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => boxed(BLUETOOTH),
      },
      { name: "Focus arrives", what: "It lights up over `short` on `standard`: off, tube and glow; on, core, glow and brighter content.", tracks: lights(".k-tile", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-tile", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "The glow brightens over `micro`; an on tile's fill lifts, by day an off one's tints.", tracks: press(".k-tile", ON), reduced: "same", back: "plays", sample: pair("lit") },
      {
        name: "The pointer onto the arrow",
        what: "From tile to arrow: the tile rests over `short` as the arrow lights, so one thing is lit.",
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
      { keys: ["Enter", "Space"], does: "toggles it, never on key repeat" },
      { keys: ["→"], does: "opens what is behind its arrow" },
      { keys: ["Tab", "Shift+Tab"], does: "next or previous tile; the arrow is no stop" },
    ],
    order: "One stop each in reading order; a disabled one is skipped.",
    lands: "On Wi-Fi, the first tile, as the control center opens; toggling keeps it there.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With a detail arrow", for: "height, padding, symbol, lines, arrow", sample: () => boxed(KEYBOARD) },
      { name: "Two to a row", for: "the gap between two tiles", sample: () => within(PANEL, pairRow(tile(WIFI), tile(BLUETOOTH))) },
    ]),
    measures: [
      { part: "Height", token: "size-row_two_line" },
      { part: "Sides, symbol → words, arrow", token: "space-s12" },
      { part: "Title → status", token: "space-s4" },
      { part: "Symbol", token: "type-body_strong-icon" },
      { part: "Title line", token: "type-body_strong-line" },
      { part: "Status line", token: "type-small-line" },
      { part: "Detail arrow", token: "size-control_sm" },
      { part: "Arrow's chevron", token: "type-button_small-icon" },
      { part: "Between tiles", token: "space-s8" },
      { part: "Corners", token: "radius-control" },
    ],
    reflow: {
      note: "It fills the room it is given; title and status end in an ellipsis.",
      sample: () => within("100%", pairRow(tile({ ...WIFI, subtitle: shown(STATUS) }), tile(KEYBOARD))),
    },
  },
});
