// The Icons foundation: the whole symbol set, the sizes by type style, optical centring and app
// tiles (`docs/DESIGN.md`, "Iconography" and "Icons by type").

import { icon, symbols } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { type TypeStyle, text } from "../../lib/text.ts";
import { iconTile, TILE_MEASURES, TILE_SIZES } from "../../lib/tile.ts";
import { length, names, number } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, themes, token } from "../../site/parts/swatches.ts";
import { SCALES, settings } from "../../site/settings.ts";
import appIcon from "../../../../../branding/png/kubaos-icon-128.png?url";
import photo from "../../../../../branding/photo/neon-sign-front.png?url";

/** The symbols drawn by us where Tabler has none, how each is drawn, and its Tabler relatives in the set. */
const OURS: Record<string, { readonly drawn: string; readonly relatives: readonly string[] }> = {
  hard_drive: { drawn: "A drive's case in server-2's proportions, its board below a line, server-2's light and slot.", relatives: ["storage", "memory"] },
  wifi_lock: { drawn: "Tabler's Wi-Fi fan at full size, stopped 2 units clear of its lock badge.", relatives: ["wifi", "lock"] },
  wifi_find: { drawn: "Tabler's Wi-Fi fan at full size, stopped 2 units clear of its search badge.", relatives: ["wifi", "search"] },
  bluetooth_searching: { drawn: "Tabler's bluetooth-x, a magnifier where its x is.", relatives: ["bluetooth", "search"] },
  star_filled: { drawn: "Tabler's star, filled, its outline kept so it is the outline star's size.", relatives: ["star"] },
};

/** The sizes a symbol is drawn at beside text, in the set's cells. */
const SET_SIZES = ["icon-s16", "icon-s20", "icon-s24"];

/** Every icon size token: the sizes, then the one beside two lines. */
function sizes(): string[] {
  return names("icon-");
}

/** Every type style, by name, from its icon token: the largest icon first. */
function styles(): TypeStyle[] {
  return names("type-")
    .filter((name) => name.endsWith("-icon"))
    .map((name) => name.slice("type-".length, -"-icon".length) as TypeStyle)
    .sort((a, b) => length(`type-${b}-icon`) - length(`type-${a}-icon`));
}

/** The type styles whose icon is `size` logical pixels. */
function beside(size: number): TypeStyle[] {
  return styles().filter((style) => length(`type-${style}-icon`) === size);
}

/** `name` in its box, the box outlined. */
function boxed(name: string, size: string, colour = "on_surface"): HTMLElement {
  const box = el("div", "f-icon-box", icon(name, `var(--${size})`, `var(--color-${colour})`));
  box.style.width = `var(--${size})`;
  box.style.height = `var(--${size})`;
  return box;
}

/** Every symbol at 16, 20 and 24, under its name; ours outlined in the accent. */
function symbolSet(): HTMLElement {
  return el(
    "div",
    "f-symbols",
    ...symbols().map((name) => {
      const cell = el(
        "div",
        "f-symbol",
        row("s12", "end", ...SET_SIZES.map((size) => icon(name, `var(--${size})`, "var(--color-on_surface)"))),
        label(name, "mono", "on_surface_variant"),
      );
      cell.toggleAttribute("data-ours", name in OURS);
      return cell;
    }),
  );
}

/** Every icon size, its box drawn round a symbol, and the styles it sits beside. */
function sizeTable(): HTMLElement {
  return table(
    ["Size", "Drawn in its box", "Beside"],
    sizes().map((size) => {
      const px = length(size);
      const styles = size === "icon-two_line" ? "a title over a subtitle, `body_strong` over `small`" : beside(px).map((style) => `\`${style}\``).join(", ");
      return [token(size.replace("-", "."), String(px)), boxed("settings", size), prose(styles || "above text: an empty state's, a page's one big symbol", "body", "on_surface")];
    }),
  );
}

/** The five drawn by us, each beside its Tabler relatives, at 16, 20, 24 and 48, all one colour. */
function ours(): HTMLElement {
  const at = (name: string) => row("s16", "end", ...["icon-s16", "icon-s20", "icon-s24", "icon-s48"].map((size) => icon(name, `var(--${size})`, "var(--color-on_surface)")));
  return table(
    ["Ours", "Its relatives", "How it is drawn"],
    Object.entries(OURS).map(([name, { drawn, relatives }]) => [
      stack("s8", "start", at(name), label(name, "mono", "on_surface_variant")),
      stack("s8", "start", ...relatives.map((relative) => stack("s4", "start", at(relative), label(relative, "mono", "on_surface_variant")))),
      prose(drawn, "body", "on_surface"),
    ]),
  );
}

/** `style`'s symbol in a box as wide as itself and its line tall, centred down it. */
function iconBeside(style: TypeStyle, name = "info"): HTMLElement {
  const slot = el("span", "k-icon-line");
  slot.style.setProperty("--line", `var(--type-${style}-line)`);
  slot.style.setProperty("--icon-size", `var(--type-${style}-icon)`);
  slot.append(icon(name, `var(--type-${style}-icon)`, "var(--color-on_surface_variant)"));
  return slot;
}

/** `style`'s words with its icon before them, its line box tinted so its height shows. */
function line(style: TypeStyle, words = "Wi-Fi"): HTMLElement {
  return el("div", "f-line", row("s8", "start", iconBeside(style, "wifi"), text(words, style, "on_surface")));
}

/** Every style, its icon and the line it sits on. */
function byType(): HTMLElement {
  return table(
    ["Style", "Icon", "On its line"],
    styles().map((style) => [token(style), label(String(length(`type-${style}-icon`)), "mono", "on_surface"), line(style)]),
  );
}

/** A title over a subtitle, the icon spanning both, and a tile beside the same two lines. */
function twoLines(): HTMLElement {
  const words = () => stack(0, "start", text("Kitchen", "body_strong", "on_surface"), text("Connected · WPA2", "small", "on_surface_variant"));
  const slot = el("div", "f-slot", icon("wifi", "var(--icon-two_line)", "var(--color-on_surface_variant)"));
  slot.style.setProperty("--side", "calc(var(--type-body_strong-line) + var(--type-small-line))");
  return el("div", "f-row", row("s12", "start", slot, words()), row("s12", "start", iconTile({ glyph: "wifi", size: "Md" }), words()));
}

/** A symbol centred in slots of each tile size, the middle marked, then on its 24 unit grid. */
function centring(): HTMLElement {
  const slots = TILE_SIZES.map((size) => {
    const { side, icon: glyph } = TILE_MEASURES[size];
    const slot = el("div", "f-slot", icon("terminal", `var(--${glyph})`, "var(--color-on_surface)"));
    slot.style.setProperty("--side", `var(--${side})`);
    return el("div", "f-swatch-cell", label(`${size.toLowerCase()}: ${length(glyph)} in ${length(side)}`, "label"), slot);
  });
  const grids = ["terminal", "wifi_lock", "star_filled"].map((name) => el("div", "f-swatch-cell", label(name, "label"), el("div", "f-icon-grid", icon(name, "var(--icon-s48)", "var(--color-on_surface)"))));
  return el("div", "page-part", el("div", "f-row", ...slots), el("div", "f-row", ...grids));
}

/** A symbol in each role it is drawn in, and dimmed as a disabled label dims. */
function roles(): HTMLElement {
  const roleIcon = (role: string, words: string) => el("div", "f-swatch-cell", label(words, "label"), icon("notifications", "var(--icon-s24)", `var(--color-${role})`));
  const disabled = icon("notifications", "var(--icon-s24)", "var(--color-on_surface)");
  disabled.style.opacity = String(number("opacity-disabled_content"));
  return el(
    "div",
    "f-row",
    roleIcon("on_surface", "on_surface"),
    roleIcon("on_surface_variant", "on_surface_variant"),
    roleIcon("primary", "primary"),
    roleIcon("cold", "cold"),
    roleIcon("error", "error"),
    el("div", "f-swatch-cell", label("disabled", "label"), disabled),
  );
}

/** Every kind of tile: each size with a symbol, the accent's, a critical one, an app's icon and a photo. */
function tiles(): HTMLElement {
  const tile = (words: string, made: HTMLElement) => el("div", "f-swatch-cell", label(words, "label"), made);
  return el(
    "div",
    "f-row",
    ...TILE_SIZES.map((size) => tile(size.toLowerCase(), iconTile({ glyph: "terminal", size }))),
    tile("accent", iconTile({ glyph: "star_filled", size: "Md", accent: true })),
    tile("danger", iconTile({ glyph: "warning", size: "Md", danger: true })),
    tile("an app's icon", iconTile({ glyph: "terminal", size: "Md", picture: appIcon })),
    tile("a photo", iconTile({ glyph: "notifications", size: "Md", photo })),
  );
}

/** A symbol's stroke in device pixels at `size` logical pixels and `scale`: 2 of its 24 units, rounded half up, never under one. */
function symbolStroke(size: number, scale: number): number {
  return Math.max(1, Math.round((size * scale * number("stroke-symbol")) / 24));
}

function strokeTable(): HTMLElement {
  return table(
    ["Size", ...SCALES.map((scale) => `${scale}×`)],
    names("icon-s").map((size) => [token(size.replace("-", "."), String(length(size))), ...SCALES.map((scale) => label(String(symbolStroke(length(size), scale)), "mono", "on_surface"))]),
  );
}

/** A few symbols at every size, in the theme and `mode`. */
function sampler(mode: "dark" | "light", theme = settings().theme): HTMLElement {
  return painted(
    theme,
    mode,
    row("s16", "end", ...names("icon-s").map((size) => icon("wifi_lock", `var(--${size})`, "var(--color-on_surface)"))),
    row("s16", "start", iconTile({ glyph: "terminal", size: "Md" }), iconTile({ glyph: "star_filled", size: "Md", accent: true }), iconTile({ glyph: "warning", size: "Md", danger: true })),
  );
}

export default foundation({
  level: "foundations",
  slug: "icons",
  title: "Icons",
  summary: "One set, KubaOS Symbols: Tabler's outline icons and five of our own, drawn from SVGs at the size of the text beside them, their strokes whole device pixels at every scale.",
  live: () =>
    painted(
      settings().theme,
      settings().mode,
      line("body"),
      row("s12", "start", iconTile({ glyph: "wifi", size: "Md" }), stack(0, "start", text("Kitchen", "body_strong", "on_surface"), text("Connected · WPA2", "small", "on_surface_variant"))),
      row("s16", "start", ...TILE_SIZES.map((size) => iconTile({ glyph: "terminal", size }))),
    ),
  description: [
    "One set: KubaOS Symbols, Tabler's outline icons, picked by Kuba from a sheet of the same 40 glyphs in four sets, each named for the Material Symbol it replaced. Five are ours, where Tabler has none, drawn on Tabler's 24 grid with its 2 unit stroke and round caps and joins, as Tabler builds its own badged icons.",
    "A symbol is drawn from its SVG, not a font, with its stroke set for its size and scale: 2 of its 24 units scaled to the size, rounded half up to whole device pixels and never under one, so no line falls between device pixels.",
    "A symbol takes its size from the text beside it, never a size of its own: each type style names its icon, one or two icon sizes above its font size, never taller than its line. Beside a title over a subtitle it spans both, 24. Above text it takes the size its place names: an empty state's 32, a page's one big symbol 48.",
    "A symbol's box is centred in its slot on a whole device pixel, the odd one after it; the drawing's own optical centre is Tabler's grid, which ours follow. Its colour is a role, and its alpha dims it as its label dims. Apps show as uniform tiles: the app's icon on a tile of one size and radius, so a list of apps reads as one set, whatever each vendor drew.",
  ],
  tokens: [
    {
      name: "The whole symbol set",
      described: "Every symbol at 16, 20 and 24, under its name; the five drawn by us are outlined in the accent. A named symbol is never missing: the set is closed, and a new one is drawn and shown here before it ships.",
      sample: symbolSet,
    },
    {
      name: "The sizes",
      described: "Every `[icon]` size, a symbol in its box with the box drawn round it (its ink 2 units clear inside, Tabler's margin), and the styles it sits beside.",
      sample: sizeTable,
    },
    {
      name: "Drawn by us",
      described: "Each of our five beside the Tabler symbols it borrows from, all one colour, at 16, 20, 24 and 48: its ink as dense as theirs.",
      sample: ours,
    },
  ],
  rules: [
    {
      name: "Icons by type",
      described: "Every type style with its icon before its words on its line box, tinted so its height shows: the icon is centred down the line on a whole device pixel and never makes it taller, so a row with an icon is as tall as one without. `kubaos-tokens` refuses an icon taller than its style's line.",
      sample: byType,
    },
    {
      name: "Beside two lines",
      described: "A title over a subtitle (`body_strong` over `small`, a 40 block) takes the icon that spans both, `icon.two_line`, 24, centred down the block; a tile beside the same two lines is md, 40, the block's height.",
      sample: twoLines,
    },
    {
      name: "Optically centred",
      described: "A symbol centred in a slot of each tile's size, the slot's middle marked across and down: its box sits on a whole device pixel, the odd one after it. Then symbols on their 24 unit grid, a line every unit, the 2 unit margin shaded: Tabler's grid is the drawing's optical centre, and ours keep to it.",
      sample: centring,
    },
    {
      name: "Colour is a role",
      described: "A symbol takes the colour role of what it says: the content's, the quiet content's, the accent, the cold accent, the error; disabled, it dims to `opacity.disabled_content` as its label does.",
      sample: roles,
    },
    {
      name: "Apps as uniform tiles",
      described: "An `IconTile` at sm 32 beside one line (a 20 symbol), md 40 beside a title over a subtitle (24) and lg 48 over a `headline` name (32), at the control radius, on `surface_container_highest`; its symbol `on_surface_variant`, or `primary` on the one tile of a list that comes first, and on `error` for a critical notification. An app's icon is inset at the symbol's size, keeping its own margins; a photo covers the tile.",
      sample: tiles,
    },
  ],
  modes: [
    {
      name: "Dark and light",
      described: "The symbols at every size and the tiles in each mode: a symbol is its role's colour in either, so it reads on both surfaces.",
      sample: () => el("div", "f-row", ...MODES.map((mode) => el("div", "f-swatch-cell", label(modeName(mode), "label"), sampler(mode)))),
    },
  ],
  themes: [
    {
      name: "In every theme",
      described: "The same symbols and tiles in each theme the toolbar paints in, in the mode picked: only the accent's symbol and the tiles' tint follow the wallpaper.",
      sample: () => el("div", "f-row", ...themes().map((theme) => el("div", "f-swatch-cell", label(theme.name, "label"), sampler(settings().mode, theme.slug)))),
    },
  ],
  scales: [
    {
      name: "The stroke in device pixels",
      described: "Each size's stroke at each scale: 2 of 24 units, rounded half up, never under one. 16 at 1× stays 1: 1.33 rounds to 1, and 2 would make the small symbols heavier than the text beside them.",
      sample: strokeTable,
    },
    {
      name: "The symbols at every scale",
      described: "Each box is drawn at its own scale, whatever the toolbar's: the strokes thicken in whole device pixels.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, row("s16", "end", ...names("icon-s").map((size) => icon("wifi_lock", `var(--${size})`, "var(--color-on_surface)"))))))),
    },
  ],
  content: [
    {
      name: "No picture",
      described: "A tile shows its symbol until the app's picture has loaded, and keeps it when there is none, so every row reads the same size; a symbol of the set is never missing.",
      sample: () => el("div", "f-row", iconTile({ glyph: "terminal", size: "Md" }), iconTile({ glyph: "terminal", size: "Md", picture: appIcon })),
    },
  ],
  motion: doesntApply("a symbol moves only with what it is on; one symbol changing for another (a Wi-Fi strength's bars) swaps at once, with its state."),
  checks: [
    { check: "kubaos-tokens", holds: "Refuses an icon taller than its style's line." },
    { check: "make-symbols.sh", holds: "Fetches each Tabler symbol hash-pinned, from `@tabler/icons` 3.49.0, by the name `[symbols]` gives it." },
    { check: "The geometry check", holds: "Holds each symbol's box to whole device pixels at every scale, centred in its slot, and an `IconTile` to its twin on the specimen's page." },
    { check: "kubaos-design lint", holds: "Fails on a size written out as a number in QML or Rust UI code: an icon takes its size from its text's style." },
    { check: "Not enforced yet", holds: "Uniform app tiles for every list of apps land with the apps catalog." },
  ],
});
