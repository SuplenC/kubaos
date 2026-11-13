// The Space foundation: the 4 grid, the spacing steps, every fixed size, the container's padding
// at each breakpoint, and the "no wasted space" rules, each shown with its measures by token
// (`docs/DESIGN.md`, "Space and shape").

import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { row, stack } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { length, names, raw } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, table, token } from "../../site/parts/swatches.ts";
import { SCALES } from "../../site/settings.ts";
import { across } from "./across.ts";

/** Which way a size runs, so it is drawn as it is measured. */
type Axis = "width" | "height" | "side";

/** What each size is for and which way it runs, from the tokens' own words. */
const SIZE_FOR: Record<string, { axis: Axis; for: string }> = {
  control_sm: { axis: "height", for: "A small button or field" },
  control_md: { axis: "height", for: "A regular button, and a field to line up with it" },
  control_lg: { axis: "height", for: "A big page's one main action" },
  row: { axis: "height", for: "A list row of one line" },
  row_two_line: { axis: "height", for: "A list row, title over subtitle" },
  row_three_line: { axis: "height", for: "A menu row whose description wraps" },
  row_compact: { axis: "height", for: "A table's row" },
  menu: { axis: "width", for: "A menu" },
  chip: { axis: "height", for: "A chip" },
  badge: { axis: "side", for: "A count's pill on an icon's corner" },
  badge_dot: { axis: "side", for: "A badge's dot" },
  switch_width: { axis: "width", for: "A switch's track" },
  switch_height: { axis: "height", for: "A switch's track" },
  switch_knob_off: { axis: "side", for: "A switch's knob off, 6 of track round it" },
  switch_knob_on: { axis: "side", for: "A switch's knob on, 4 round it" },
  check: { axis: "side", for: "A checkbox's box, a radio's disc" },
  radio_dot: { axis: "side", for: "A picked radio's dot" },
  progress: { axis: "height", for: "A progress bar" },
  dialog: { axis: "width", for: "A dialog" },
  password: { axis: "width", for: "The lock screen's password field" },
  column: { axis: "width", for: "The widest column of text and fields" },
  rail: { axis: "width", for: "Kōbō's step rail" },
  sidebar: { axis: "width", for: "An app's sidebar" },
  app_width: { axis: "width", for: "An app's window as it opens" },
  app_height: { axis: "height", for: "An app's window as it opens" },
  app_min_width: { axis: "width", for: "The narrowest a tiled app goes" },
  app_min_height: { axis: "height", for: "The lowest a tiled app goes" },
  log: { axis: "height", for: "A log under an error or a progress bar" },
  panel: { axis: "width", for: "A shell panel, a notification beside one" },
  bar: { axis: "height", for: "The shell's bar" },
  bar_label: { axis: "width", for: "The widest name in the bar" },
  thumbnail_width: { axis: "width", for: "A wallpaper's thumbnail, 16:10" },
  thumbnail_height: { axis: "height", for: "A wallpaper's thumbnail, 16:10" },
  slider: { axis: "height", for: "A slider" },
  slider_compact: { axis: "height", for: "A slider in a dense panel" },
  slider_stop: { axis: "side", for: "A step's dot, off the grid" },
  slider_anchor: { axis: "side", for: "The anchor's dot, off the grid" },
  slider_anchor_compact: { axis: "side", for: "The anchor's dot on a compact slider" },
  slider_hold: { axis: "width", for: "How near a drag holds on a sticky point" },
  scrollbar: { axis: "width", for: "A scrollbar's thumb" },
  keycap: { axis: "height", for: "A keycap, never narrower than tall" },
  neon_mark: { axis: "side", for: "The クバ mark on Kōbō's done screen" },
  hanko: { axis: "side", for: "The 完了 hanko beside it" },
  sparkline: { axis: "height", for: "A sparkline in a panel" },
  motion_plot: { axis: "height", for: "The motion player's plot" },
  canvas_settings: { axis: "width", for: "CanvasPage's settings beside its canvas" },
  canvas_wide: { axis: "width", for: "Where CanvasPage's settings move beside its canvas" },
  collapsible_rail: { axis: "width", for: "The strip round a collapsible's rail that takes the pointer" },
  find: { axis: "width", for: "SidebarPage's find box" },
  halo: { axis: "side", for: "The soft blur round words on a picture" },
  halo_edge: { axis: "side", for: "The tight blur round words on a picture" },
  lock_blur: { axis: "side", for: "How far the lock's wallpaper blurs, awake" },
};

/** A length in logical pixels, as the page writes it. */
function logical(name: string): string {
  return String(length(name));
}

/** A length's device pixels at `scale`, and whether they are whole. */
function device(name: string, scale: number): { words: string; whole: boolean } {
  const value = length(name) * scale;
  return { words: String(value), whole: Number.isInteger(value) };
}

/** A bar `token` long, or a square `token` on a side. */
function bar(token: string, axis: Axis): HTMLElement {
  const shape = el("div", axis === "side" ? "f-square" : "f-bar");
  shape.style.setProperty("--length", `var(--${token})`);
  return shape;
}

/** Every spacing step: its bar, its value, and its device pixels at every scale. */
function spacing(): HTMLElement {
  return across(table(
    ["Step", "Its length", ...SCALES.map((scale) => `${scale}×`)],
    names("space-").map((name) => [
      token(name.replace("-", "."), logical(name)),
      bar(name, "width"),
      ...SCALES.map((scale) => label(device(name, scale).words, "mono", device(name, scale).whole ? "on_surface" : "error")),
    ]),
  ));
}

/** Every fixed size: its value, drawn the way it runs, and what it is for. */
function sizes(): HTMLElement {
  return across(
    table(
      ["Size", "Runs", "For", "Drawn"],
      names("size-").map((name) => {
        const short = name.slice("size-".length);
        const use = SIZE_FOR[short];
        return [
          token(`size.${short}`, logical(name)),
          label(use?.axis ?? "?", "body", "on_surface"),
          prose(use?.for ?? "No words for this size: add them to `SIZE_FOR`.", "body", use ? "on_surface" : "error"),
          bar(name, use?.axis ?? "width"),
        ];
      }),
    ),
  );
}

/** The container's steps: from which width each padding applies. */
function steps(): { from: number; padding: string }[] {
  const found: { from: number; padding: string }[] = [];
  for (let i = 0; raw(`container-${i}-from`) !== ""; i++) {
    found.push({ from: Number(raw(`container-${i}-from`)), padding: `container-${i}-padding` });
  }
  return found;
}

/** The space token a container step's padding is, by its value. */
function stepName(padding: string): string {
  return names("space-").find((name) => length(name) === length(padding))?.replace("-", ".") ?? padding;
}

/** `child` inside `inline` at its sides and `block` above and below, if any, the padding shaded. */
function padded(inline: string, block: string | null, child: Element): HTMLElement {
  const box = el("div", "f-pad", child);
  box.style.setProperty("--pad-inline", `var(--${inline})`);
  if (block) box.style.setProperty("--pad-block", `var(--${block})`);
  return box;
}

/** A box of a page's or a panel's content: a heading over a line of words. */
function content(heading: string, words: string): HTMLElement {
  return el("div", "f-content", stack("s4", "start", text(heading, "subheading", "on_surface"), text(words, "body", "on_surface_variant")));
}

/** A container as wide as `width`, padded at its sides by the step its width reaches. */
function containerAt(width: string, name: string): HTMLElement {
  const logicalWidth = length(width);
  const step = steps().filter((each) => logicalWidth >= each.from).at(-1)!;
  const box = padded(step.padding, null, content(name, `${logicalWidth} wide: ${stepName(step.padding)} at its sides`));
  box.style.width = `var(--${width})`;
  return el("div", "f-swatch-cell", token(`size.${width.slice("size-".length)}`, `${logicalWidth} → ${stepName(step.padding)} = ${length(step.padding)}`), box);
}

/** The container's breakpoints, and a container at a width in each. */
function container(): HTMLElement {
  return el(
    "div",
    "page-part",
    across(
      table(
        ["From", "Padding at its sides"],
        steps().map((each) => [label(`${each.from}`, "mono", "on_surface"), token(stepName(each.padding), String(length(each.padding)))]),
      ),
    ),
    el("div", "f-scroll", el("div", "f-stack", containerAt("size-panel", "A panel"), containerAt("size-app_min_width", "An app at its minimum"), containerAt("size-app_width", "An app at its default"))),
  );
}

/** A toolbar band: a small control, a step above and below, and nothing more. */
function bandBox(): HTMLElement {
  const inner = row("s8", "start", button({ label: "Back", kind: "Text", size: "Sm" }), button({ label: "Continue", kind: "Filled", size: "Sm" }));
  return padded("container-0-padding", "space-s16", inner);
}

/** The band over its sum. */
function band(): HTMLElement {
  return el(
    "div",
    "f-swatch-cell",
    bandBox(),
    token(`band = space.s16 ${length("space-s16")} + size.control_sm ${length("size-control_sm")} + space.s16 ${length("space-s16")} = ${length("space-s16") * 2 + length("size-control_sm")}`),
  );
}

/** Rows at their type's natural height, one under another. */
function rows(): HTMLElement {
  const one = (title: string) => {
    const box = el("div", "f-row-box", text(title, "body_strong", "on_surface"));
    box.style.setProperty("--height", "var(--size-row)");
    return box;
  };
  const two = (title: string, subtitle: string) => {
    const box = el("div", "f-row-box", stack(0, "start", text(title, "body_strong", "on_surface"), text(subtitle, "small", "on_surface_variant")));
    box.style.setProperty("--height", "var(--size-row_two_line)");
    return box;
  };
  return el(
    "div",
    "f-swatch-cell",
    el("div", "f-rows", one("Wi-Fi"), two("Home network", "Connected · secured"), one("Bluetooth")),
    token(`size.row ${length("size-row")}, size.row_two_line ${length("size-row_two_line")}, size.row_three_line ${length("size-row_three_line")}: no row taller`),
  );
}

/** A card inside a page: the page's padding is the only one round it. */
function nestedContainer(): HTMLElement {
  const inside = card({ body: [content("A card", "Its own padding inside, none added outside")] });
  return el(
    "div",
    "f-swatch-cell",
    padded("container-0-padding", null, inside),
    token(`page: ${stepName("container-0-padding")} at its sides; the card: its own, inside it`),
  );
}

/**
 * The live thing: a panel's body on the grid, the container's padding at its sides and the panel's
 * own step above and below, shaded.
 */
function live(): HTMLElement {
  const body = stack(
    "s16",
    "start",
    content("Network", "Home network · secured"),
    row("s8", "start", button({ label: "Forget", kind: "Tonal", size: "Sm" }), button({ label: "Connect", kind: "Filled", size: "Sm" })),
  );
  const panel = padded("container-0-padding", "space-s16", body);
  panel.style.width = "var(--size-panel)";
  return el("div", "f-grid-paper", panel);
}

/** Every fixed size's device pixels at every scale; a size off the grid is red where it isn't whole. */
function deviceSizes(): HTMLElement {
  return across(table(
    ["Size", ...SCALES.map((scale) => `${scale}×`)],
    names("size-").map((name) => [
      token(`size.${name.slice("size-".length)}`),
      ...SCALES.map((scale) => label(device(name, scale).words, "mono", device(name, scale).whole ? "on_surface" : "error")),
    ]),
  ));
}

export default foundation({
  level: "foundations",
  slug: "space",
  title: "Space, sizes and the container",
  lede: "Every length on the 4 grid: spacing steps, components' fixed sizes, and the container that pads every page and panel.",
  stage: {
    live: () => el("div", "f-row", live(), bandBox()),
    hint: "pick a Scale: every length stays on whole pixels",
    daylight: true,
  },
  tokens: [
    { name: "The spacing steps", note: "Each step as a bar, and its device pixels at every scale.", sample: spacing },
    { name: "Every fixed size", note: "Each component's fixed measure, drawn the way it runs.", sample: sizes },
    { name: "The container", note: "Padding follows the box's own width, not the screen's; it is shaded here.", sample: container },
  ],
  rules: [
    { name: "A band: content and a step", note: "As tall as what is in it and one step above and below, and no taller.", sample: band },
    { name: "Rows are their type's height", note: "`row` with one line, `row_two_line` with two, `row_three_line` where a description wraps; a taller row holds air.", sample: rows },
    { name: "Padding is applied once", note: "A card keeps its padding inside; the page's is the only one round it.", sample: nestedContainer },
  ],
  modes: doesntApply("geometry never changes with the mode."),
  themes: doesntApply("a length is the same in every theme."),
  scales: [
    { name: "Every size in device pixels", note: "A multiple of 4 is whole at all four; only the off-grid dots may not be.", sample: deviceSizes },
    {
      name: "The same panel",
      note: "Each box at its own scale, whatever the top bar picks.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, live())))),
    },
  ],
  content: doesntApply("space holds no words; the Type page shows how text wraps."),
  motion: doesntApply("a length doesn't animate on its own."),
  checks: [
    { check: "kubaos-tokens", holds: "refuses any length off `grid`, but those in `rules.off_grid`" },
    { check: "kubaos-design lint", holds: "fails a length written out in UI code" },
    { check: "The geometry check", holds: "holds boxes to whole device pixels, pages to the container's padding" },
  ],
});
