// The IconTile page: `docs/design/components/icon-tile.md`. What an app or a sender looks like in a
// row: a symbol or a picture on a tinted square of one size, so a list of them reads as one set.

import appIcon from "../../../../../branding/png/kubaos-icon-128.png?url";
import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import type { Words } from "../../lib/content.ts";
import { iconTile, measure, TILE_MEASURES, TILE_SIZES, type TileOptions, type TileSize } from "../../lib/icon-tile.ts";
import { filling, flow, row, stack, within } from "../../lib/layout.ts";
import { clamped, text } from "../../lib/text.ts";
import { APP_ICONS } from "../../site/app-icons.ts";
import { el } from "../../site/dom.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { MODES, painted } from "../../site/parts/swatches.ts";
import { settings } from "../../site/settings.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

const NAME: Words = { short: "Kitchen", long: "Kitchen and living room, the 5 GHz network upstairs" };
/** A path no picture is at: what a picture that can't be read leaves. */
const NOWHERE = "data:image/png;base64,AAAA";

/** What each size sits beside, by the type the words beside it are set in. */
const BESIDE: Record<TileSize, string> = {
  Sm: "beside one line of `body_strong`",
  Md: "beside a title over a subtitle",
  Lg: "over a `headline` name",
};

/** What each measured kind of app icon comes to in the tile. */
const FITS: Record<string, string> = {
  "A full plate": "fills the tile, cut by its corners",
  "A plate in a margin": "measured, it fills the tile all the same",
  "A disc": "too little of its box: a shape, `space.s4` in",
  "A glyph": "square but mostly empty: a shape",
  "A wide logo": "a shape, fitted to the tile's width",
};

/** A tile beside its words, laid out as the row it sits in lays it out. */
function besideWords(size: TileSize, options: Partial<TileOptions> = {}, name = "Kitchen"): HTMLElement {
  const tile = iconTile({ glyph: "wifi", size, ...options });
  if (size === "Sm") return row("s12", "centre", tile, clamped(name, "body_strong", "on_surface"));
  if (size === "Md") return row("s12", "centre", tile, stack(0, "start", clamped(name, "body_strong", "on_surface"), text("Connected · WPA2", "small", "on_surface_variant")));
  return stack("s12", "centre", tile, text(name, "headline", "on_surface"));
}

/** A column of rows of different kinds, each tile one size: the set reads as one. */
function senders(): HTMLElement {
  const rows: [Partial<TileOptions>, string, string][] = [
    [{ glyph: "terminal", picture: appIcon }, "KubaOS", "Updates are ready"],
    [{ glyph: "notifications", photo }, "Aiko", "Sent a photo"],
    [{ glyph: "star_filled", accent: true }, "Polish", "The default layout"],
    [{ glyph: "keyboard" }, "English (US)", "A layout"],
    [{ glyph: "warning", danger: true }, "Battery", "5% left: plug in now"],
  ];
  return stack(
    "s12",
    "start",
    ...rows.map(([options, title, line]) => row("s12", "centre", iconTile({ glyph: "terminal", size: "Md", ...options }), stack(0, "start", text(title, "body_strong", "on_surface"), text(line, "small", "on_surface_variant")))),
  );
}

/** What the measurement of `picture` found, as the measures table prints a value. */
function found(picture: string): HTMLElement {
  const line = el("code", "page-code");
  void measure(picture).then((drawing) => {
    const coverage = drawing ? String(Math.round(drawing.coverage * 100) / 100) : "";
    line.textContent = drawing ? `${drawing.plate ? "plate" : "shape"}, coverage = ${coverage}` : "can't be read";
  });
  return line;
}

/** An app's icon on a tile of every size, in dark and in light, under what its measurement found. */
function measured(picture: string): HTMLElement {
  const tiles = (mode: (typeof MODES)[number]) => painted(settings().theme, mode, row("s16", "centre", ...TILE_SIZES.map((size) => iconTile({ glyph: "terminal", size, picture }))));
  return stack("s12", "start", found(picture), row("s16", "start", ...MODES.map(tiles)));
}

type MatrixState = "rest" | "loading";

const ROWS: readonly { name: string; options: TileOptions }[] = [
  { name: "A symbol", options: { glyph: "terminal", size: "Md" } },
  { name: "An app's icon", options: { glyph: "terminal", size: "Md", picture: appIcon } },
  { name: "A photo", options: { glyph: "notifications", size: "Md", photo } },
];

export default page({
  level: "molecules",
  slug: "icon-tile",
  title: "IconTile",
  lede: "A symbol or a picture on a tinted square of one size, so a list of apps, senders or devices reads as one set.",
  code: { rust: "icon_tile::icon_tile", qml: "IconTile" },
  stage: {
    live: () => within(ACROSS, filling(flow("s48", senders(), stack("s16", "start", ...TILE_SIZES.map((size) => besideWords(size)))))),
    hint: "a mark: its row lights, not the tile",
    daylight: true,
  },
  variants: [
    { name: "A symbol", for: "something KubaOS draws", sample: () => iconTile({ glyph: "keyboard", size: "Md" }) },
    { name: "Accent", for: "the one of a list that comes first", sample: () => iconTile({ glyph: "star_filled", size: "Md", accent: true }) },
    { name: "Danger", for: "a critical notification", sample: () => iconTile({ glyph: "warning", size: "Md", danger: true }) },
    ...APP_ICONS.map((app) => ({ name: app.name, for: FITS[app.name]!, sample: () => measured(app.picture) })),
    { name: "A photo", for: "covers the tile", sample: () => iconTile({ glyph: "notifications", size: "Md", photo }) },
  ],
  sizes: nonEmpty(TILE_SIZES.map((size) => ({ name: size.toLowerCase(), for: BESIDE[size], sample: () => besideWords(size) }))),
  states: {
    shown: ["rest", "loading"],
    absent: {
      lit: "a mark: its row lights",
      pressed: "takes no press",
      disabled: "dims with its row",
      selected: "the look is the row's",
      empty: "always a symbol",
      error: "an unreadable picture leaves the symbol",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => iconTile({ ...options, pending: state === "loading" }) }))),
    looks: {
      rest: "symbol, plate or photo on `surface_container_highest`",
      loading: "the symbol holds the place; nothing moves when it arrives",
    },
  },
  focus: doesntApply("it takes no input and has no line: its row lights."),
  content: {
    text: [{ name: "The words beside it", words: NAME, sample: (words: string) => within("var(--size-panel)", besideWords("Md", {}, words)) }],
    missingPicture: [
      { name: "No picture", for: "the symbol", sample: () => iconTile({ glyph: "notifications", size: "Md" }) },
      { name: "Can't be read", for: "the symbol stays", sample: () => iconTile({ glyph: "notifications", size: "Md", picture: NOWHERE }) },
    ],
    counts: [
      { name: "One", for: "a row alone", sample: () => besideWords("Md") },
      { name: "Many, of every kind", for: "every row one size", sample: senders },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its row."),
    leaves: doesntApply("it goes with its row."),
    between: doesntApply("its picture replaces the symbol in the same box, with no fade."),
  },
  keyboard: doesntApply("it takes no input: its row takes the keys."),
  measures: {
    samples: nonEmpty(TILE_SIZES.map((size) => ({ name: size.toLowerCase(), for: "its side and symbol", sample: () => iconTile({ glyph: "wifi", size }) }))),
    measures: nonEmpty([
      ...TILE_SIZES.map((size) => ({ part: `Side, ${size.toLowerCase()}`, token: TILE_MEASURES[size].side })),
      ...TILE_SIZES.map((size) => ({ part: `Symbol, ${size.toLowerCase()}`, token: TILE_MEASURES[size].icon })),
      { part: "Round a shape", token: "space-s4" },
      { part: "Corner, cuts a plate", token: "radius-control" },
      { part: "Tile → words", token: "space-s12" },
    ]),
    reflow: doesntApply("a fixed side at every window size."),
  },
});
