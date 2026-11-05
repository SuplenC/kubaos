// The IconTile page: `docs/design/components/icon-tile.md`. What an app or a sender looks like in a
// row: a symbol or a picture on a tinted square of one size, so a list of them reads as one set.

import appIcon from "../../../../../branding/png/kubaos-icon-128.png?url";
import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import type { Words } from "../../lib/content.ts";
import { iconTile, measure, TILE_MEASURES, TILE_SIZES, type TileOptions, type TileSize } from "../../lib/icon-tile.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { clamped, text } from "../../lib/text.ts";
import { APP_ICONS } from "../../site/app-icons.ts";
import { el } from "../../site/dom.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { MODES, painted } from "../../site/parts/swatches.ts";
import { settings } from "../../site/settings.ts";

const NAME: Words = { short: "Kitchen", long: "Kitchen and living room, the 5 GHz network upstairs" };
/** A path no picture is at: what a picture that can't be read leaves. */
const NOWHERE = "data:image/png;base64,AAAA";

/** What each size sits beside, by the type the words beside it are set in. */
const BESIDE: Record<TileSize, string> = {
  Sm: "One line: a menu's or a compact list's row, beside `body_strong`.",
  Md: "A title over a subtitle: a two-line row, the Command menu's, a notification's.",
  Lg: "Over a `headline` name: the one thing a view is about, a network or a device being connected.",
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
  summary: "A symbol or a picture on a tinted square of one size, so a list of apps, senders or devices reads as one set.",
  live: senders,
  description: [
    "Rust `icon_tile::icon_tile`, QML `IconTile`. The Command menu's rows, the notifications', a `ListRow`'s with `tile` and the control center's connection form show what each thing is on one: whatever shape each vendor drew, every row reads the same size.",
    "A square at the control radius (`radius.control`) on `surface_container_highest`, which reads on a page, a card, a well and a selected row alike. Its size is the words' beside it: sm, `size.control_sm` (32), beside one line, holding a 20 symbol (`body`'s); md, `size.control_md` (40), beside a title over a subtitle, holding 24 (`icon.two_line`); lg, `size.control_lg` (48), over a `headline` name, holding 32 (the headline's).",
    "Its symbol is `on_surface_variant`, centred on a whole device pixel, the odd one after it. Two tones say more: `accent`, the symbol in `primary`, on the one tile of a list that comes first (the default keyboard layout's filled star); and `danger`, the tile on `error` with its symbol `on_error`, for a critical notification.",
    "An app's icon is measured once when it loads, and the result kept for that icon: the box of what it draws (its pixels more opaque than `icon_tile.ink`), so the margin its theme left round it drops away, and whether that box is a plate. It is a plate when the box is square (its short side at least `icon_tile.square`, 0.95, of its long one) and its pixels more opaque than `icon_tile.opaque` cover at least `icon_tile.plate`, 85%, of it: a rounded square covers about 95% and a disc 78.5%, so the line falls in a clear gap.",
    "A plate fills the whole tile, centred, and the tile's corners cut it, so the two make one edge. Anything else is a shape, fitted `space.s4` inside the tile, centred: 24 in sm, 32 in md, 40 in lg. Either is kept whole, never cropped and never stretched, its box snapped to whole device pixels and centred as the symbol is, the odd one after it. A photo (a notification's image) covers the whole tile, cut at its corner. The symbol shows until the picture has loaded, and stays when there is none or it can't be read, so a row keeps its size whether or not a picture comes.",
    "It is a mark, not a control: it takes no input and has no states of its own. The row it sits in lights; the tile doesn't change with it.",
  ],
  variants: [
    { name: "A symbol", for: "Something KubaOS draws: a setting, a device, a layout.", sample: () => iconTile({ glyph: "keyboard", size: "Md" }) },
    { name: "Accent", for: "The one of a list that comes first: the default layout's filled star.", sample: () => iconTile({ glyph: "star_filled", size: "Md", accent: true }) },
    { name: "Danger", for: "A critical notification: the tile on the error colour.", sample: () => iconTile({ glyph: "warning", size: "Md", danger: true }) },
    ...APP_ICONS.map((app) => ({ name: app.name, for: app.for, sample: () => measured(app.picture) })),
    { name: "A photo", for: "A notification's image, covering the tile.", sample: () => iconTile({ glyph: "notifications", size: "Md", photo }) },
  ],
  sizes: nonEmpty(TILE_SIZES.map((size) => ({ name: size.toLowerCase(), for: BESIDE[size], sample: () => besideWords(size) }))),
  states: {
    shown: ["rest", "loading"],
    absent: {
      lit: "a tile is a mark, not a control: the row it sits in lights, and the tile stays as it is.",
      pressed: "it takes no press of its own.",
      disabled: "a disabled row dims what it holds, the tile with its words, at `opacity.disabled_content`.",
      selected: "a selected row is the row's look; the tile on it is the same tile.",
      empty: "a tile always has a symbol, which stays when there is no picture.",
      error: "a picture that can't be read leaves the symbol, as under Content.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => iconTile({ ...options, pending: state === "loading" }) }))),
    described: {
      rest: "The symbol in `on_surface_variant`, the app's icon filling the tile as a plate, or the photo over the whole tile, on `surface_container_highest`.",
      loading: "Its picture on its way: the symbol in its place meanwhile, so the row keeps its size and nothing moves when the picture arrives.",
    },
  },
  focus: doesntApply("a tile takes no input and has no line: the row it sits in lights."),
  content: {
    text: [{ name: "The words beside it", words: NAME, sample: (words: string) => within("var(--size-panel)", besideWords("Md", {}, words)) }],
    missingPicture: [
      { name: "No picture", for: "The symbol: a sender with no icon of its own.", sample: () => iconTile({ glyph: "notifications", size: "Md" }) },
      { name: "Can't be read", for: "The symbol stays in its place.", sample: () => iconTile({ glyph: "notifications", size: "Md", picture: NOWHERE }) },
    ],
    counts: [
      { name: "One", for: "A row alone.", sample: () => besideWords("Md") },
      { name: "Many, of every kind", for: "Symbols, an app's icon and a photo in one list: every row one size.", sample: senders },
    ],
  },
  motion: {
    arrives: doesntApply("a tile comes with its row."),
    leaves: doesntApply("it goes with its row."),
    between: doesntApply("its picture takes the symbol's place on the frame it has loaded, in the same box: nothing moves, and nothing fades."),
  },
  keyboard: doesntApply("a tile takes no input: the row it sits in takes the keys."),
  measures: {
    samples: nonEmpty(TILE_SIZES.map((size) => ({ name: size.toLowerCase(), for: "Its side and its symbol.", sample: () => iconTile({ glyph: "wifi", size }) }))),
    measures: nonEmpty([
      ...TILE_SIZES.map((size) => ({ what: `The ${size.toLowerCase()} tile's side.`, token: TILE_MEASURES[size].side })),
      ...TILE_SIZES.map((size) => ({ what: `The ${size.toLowerCase()} tile's symbol.`, token: TILE_MEASURES[size].icon })),
      { what: "Round an app's icon that is a shape, inside the tile; a plate fills it.", token: "space-s4" },
      { what: "Its corners, which cut an app's icon that is a plate.", token: "radius-control" },
      { what: "Between the tile and the words beside it, in a row.", token: "space-s12" },
    ]),
    reflow: doesntApply("a tile has a fixed side, the size of the words beside it, at every window size."),
  },
});
