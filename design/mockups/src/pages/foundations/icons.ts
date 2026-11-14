// The Icons foundation: the whole symbol set, the sizes by type style, optical centring and app
// tiles (`docs/DESIGN.md`, "Iconography" and "Icons by type").

import { icon, symbols } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { type TypeStyle, text } from "../../lib/text.ts";
import { iconTile, TILE_MEASURES, TILE_SIZES } from "../../lib/icon-tile.ts";
import { length, names, number } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, themes, token } from "../../site/parts/swatches.ts";
import { SCALES, settings } from "../../site/settings.ts";
import { across } from "./across.ts";
import appIcon from "../../../../../branding/png/kubaos-icon-128.png?url";
// An app's icon with no plate, a star: a shape on its tile (`site/app-icons.ts`).
import glyph from "../../assets/icon-tile/star.svg?url";
import photo from "../../../../../branding/photo/neon-sign-lit.jpg?url";

/** The symbols drawn by us where Tabler has none, how each is drawn, and its Tabler relatives in the set. */
const OURS: Record<string, { readonly drawn: string; readonly relatives: readonly string[] }> = {
  hard_drive: { drawn: "A drive's case in server-2's proportions", relatives: ["storage", "memory"] },
  wifi_lock: { drawn: "Tabler's Wi-Fi fan, stopped 2 units clear of its lock", relatives: ["wifi", "lock"] },
  wifi_find: { drawn: "Tabler's Wi-Fi fan, stopped 2 units clear of its search", relatives: ["wifi", "search"] },
  bluetooth_searching: { drawn: "Tabler's bluetooth-x, a magnifier for its x", relatives: ["bluetooth", "search"] },
  star_filled: { drawn: "Tabler's star filled, its outline kept", relatives: ["star"] },
  more: { drawn: "New, the mockups' alone so far: three dots on the grid, the bar's ⋯", relatives: ["expand_more"] },
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
  return across(table(
    ["Size", "Drawn in its box", "Beside"],
    sizes().map((size) => {
      const px = length(size);
      const styles = size === "icon-two_line" ? "a title over a subtitle: `body_strong` over `small`" : beside(px).map((style) => `\`${style}\``).join(", ");
      return [token(size.replace("-", "."), String(px)), boxed("settings", size), prose(styles || "above text: an empty state's, a page's one big symbol", "body", "on_surface")];
    }),
  ));
}

/** The five drawn by us, each beside its Tabler relatives, at 16, 20, 24 and 48, all one colour. */
function ours(): HTMLElement {
  const at = (name: string) => row("s16", "end", ...["icon-s16", "icon-s20", "icon-s24", "icon-s48"].map((size) => icon(name, `var(--${size})`, "var(--color-on_surface)")));
  return across(table(
    ["Ours", "Its relatives", "How it is drawn"],
    Object.entries(OURS).map(([name, { drawn, relatives }]) => [
      stack("s8", "start", at(name), label(name, "mono", "on_surface_variant")),
      stack("s8", "start", ...relatives.map((relative) => stack("s4", "start", at(relative), label(relative, "mono", "on_surface_variant")))),
      prose(drawn, "body", "on_surface"),
    ]),
  ));
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
  return across(table(
    ["Style", "Icon", "On its line"],
    styles().map((style) => [token(style), label(String(length(`type-${style}-icon`)), "mono", "on_surface"), line(style)]),
  ));
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

/** Every kind of tile: each size with a symbol, the accent's, a critical one, app icons as a plate and as a shape, and a photo. */
function tiles(): HTMLElement {
  const tile = (words: string, made: HTMLElement) => el("div", "f-swatch-cell", label(words, "label"), made);
  return el(
    "div",
    "f-row",
    ...TILE_SIZES.map((size) => tile(size.toLowerCase(), iconTile({ glyph: "terminal", size }))),
    tile("accent", iconTile({ glyph: "star_filled", size: "Md", accent: true })),
    tile("danger", iconTile({ glyph: "warning", size: "Md", danger: true })),
    tile("an app's icon, a plate", iconTile({ glyph: "terminal", size: "Md", picture: appIcon })),
    tile("an app's icon, a shape", iconTile({ glyph: "terminal", size: "Md", picture: glyph })),
    tile("a photo", iconTile({ glyph: "notifications", size: "Md", photo })),
  );
}

/** A symbol's stroke in device pixels at `size` logical pixels and `scale`: 2 of its 24 units, rounded half up, never under one. */
function symbolStroke(size: number, scale: number): number {
  return Math.max(1, Math.round((size * scale * number("stroke-symbol")) / 24));
}

function strokeTable(): HTMLElement {
  return across(table(
    ["Size", ...SCALES.map((scale) => `${scale}×`)],
    names("icon-s").map((size) => [token(size.replace("-", "."), String(length(size))), ...SCALES.map((scale) => label(String(symbolStroke(length(size), scale)), "mono", "on_surface"))]),
  ));
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

/** The set at work: a symbol on each kind of line, beside two lines and on every tile, on the surface it is drawn on. */
function atWork(): HTMLElement {
  const twoLine = row("s12", "start", iconTile({ glyph: "wifi", size: "Md" }), stack(0, "start", text("Kitchen", "body_strong", "on_surface"), text("Connected · WPA2", "small", "on_surface_variant")));
  return el(
    "div",
    "f-row",
    stack("s8", "start", line("heading"), line("body"), line("small")),
    twoLine,
    row("s16", "start", ...TILE_SIZES.map((size) => iconTile({ glyph: "terminal", size })), iconTile({ glyph: "star_filled", size: "Md", accent: true }), iconTile({ glyph: "warning", size: "Md", danger: true })),
    row("s16", "end", ...names("icon-s").map((size) => icon("wifi_lock", `var(--${size})`, "var(--color-on_surface)"))),
  );
}

export default foundation({
  level: "foundations",
  slug: "icons",
  title: "Icons",
  lede: "One set: Tabler's outline icons and five of our own, drawn at the size of the text beside them, strokes on whole device pixels.",
  stage: { live: atWork, hint: "pick a Scale: strokes thicken in whole pixels", daylight: true },
  tokens: [
    { name: "The whole symbol set", note: "Every symbol at 16, 20 and 24; ours outlined in the accent; the set is closed.", sample: symbolSet },
    { name: "The sizes", note: "Each size in its box, its ink 2 units clear, and the styles it sits beside.", sample: sizeTable },
    { name: "Drawn by us", note: "Our five beside the Tabler symbols they borrow from, their ink as dense.", sample: ours },
  ],
  rules: [
    { name: "Icons by type", note: "Centred down the line and never taller, so a row with an icon is as tall as one without.", sample: byType },
    { name: "Beside two lines", note: "`icon.two_line`, 24, spans the 40 block; a tile beside it is md, 40.", sample: twoLines },
    { name: "Optically centred", note: "Its box on a whole device pixel, the odd one after; Tabler's 24 grid is the optical centre.", sample: centring },
    { name: "Colour is a role", note: "The role of what it says; disabled, it dims to `opacity.disabled_content`.", sample: roles },
    { name: "Apps as uniform tiles", note: "sm 32, md 40, lg 48 at the control radius; a plate fills it, a shape sits `space.s4` inside.", sample: tiles },
  ],
  modes: [
    {
      name: "Dark and light",
      note: "A symbol is its role's colour, so it reads on both surfaces.",
      sample: () => el("div", "f-row", ...MODES.map((mode) => el("div", "f-swatch-cell", label(modeName(mode), "label"), sampler(mode)))),
    },
  ],
  themes: [
    {
      name: "In every theme",
      note: "Only the accent's symbol and the tiles' tint follow the wallpaper.",
      sample: () => el("div", "f-row", ...themes().map((theme) => el("div", "f-swatch-cell", label(theme.name, "label"), sampler(settings().mode, theme.slug)))),
    },
  ],
  scales: [
    { name: "The stroke in device pixels", note: "2 of 24 units, rounded half up, never under one; 16 at 1× stays 1.", sample: strokeTable },
    {
      name: "The symbols",
      note: "Each box at its own scale, whatever the top bar picks.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, row("s16", "end", ...names("icon-s").map((size) => icon("wifi_lock", `var(--${size})`, "var(--color-on_surface)"))))))),
    },
  ],
  content: [
    {
      name: "No picture",
      note: "A tile keeps its symbol until the picture loads, or when there is none.",
      sample: () => el("div", "f-row", iconTile({ glyph: "terminal", size: "Md" }), iconTile({ glyph: "terminal", size: "Md", picture: appIcon })),
    },
  ],
  motion: doesntApply("a symbol moves with what it is on; a swap is at once, with its state."),
  checks: [
    { check: "kubaos-tokens", holds: "refuses an icon taller than its style's line" },
    { check: "make-symbols.sh", holds: "fetches each Tabler symbol hash-pinned, from `@tabler/icons` 3.49.0" },
    { check: "The geometry check", holds: "holds each symbol's box to device pixels, an `IconTile` to its twin" },
    { check: "kubaos-design lint", holds: "fails an icon size written as a number in UI code" },
    { check: "Not enforced yet", holds: "uniform tiles for every list of apps land with the catalog" },
  ],
});
