// The Space foundation: the 4 grid, the spacing steps, every fixed size, the container's padding
// at each breakpoint, and the "no wasted space" rules, each shown with its measures by token
// (`docs/DESIGN.md`, "Space and shape").

import { button } from "../../lib/button.ts";
import { row, stack } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { length, names, raw } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, table, token } from "../../site/parts/swatches.ts";
import { SCALES } from "../../site/settings.ts";

/** Which way a size runs, so it is drawn as it is measured. */
type Axis = "width" | "height" | "side";

/** What each size is for and which way it runs, from the tokens' own words. */
const SIZE_FOR: Record<string, { axis: Axis; for: string }> = {
  control_sm: { axis: "height", for: "A small button or field: chrome around a screen, beside a field's label." },
  control_md: { axis: "height", for: "A regular button, and a field, so the two line up side by side." },
  control_lg: { axis: "height", for: "The one main action of a big page: Kōbō's Install." },
  row: { axis: "height", for: "A list row with one line." },
  row_two_line: { axis: "height", for: "A list row with a title over a subtitle." },
  row_compact: { axis: "height", for: "A table's row: a line of body text with 4 above and below, dense and still easy to hit." },
  menu: { axis: "width", for: "A menu: an item's symbol, its words and a shortcut's keycaps side by side." },
  chip: { axis: "height", for: "A chip." },
  badge: { axis: "side", for: "A count's pill over an icon's corner, a line of label text tall." },
  badge_dot: { axis: "side", for: "A badge's dot, the size of a picked radio button's." },
  switch_width: { axis: "width", for: "A switch's track." },
  switch_height: { axis: "height", for: "A switch's track." },
  switch_knob_off: { axis: "side", for: "A switch's knob off, leaving 6 of track round it." },
  switch_knob_on: { axis: "side", for: "A switch's knob on: it grows, leaving 4." },
  check: { axis: "side", for: "A checkbox's box and a radio button's disc." },
  radio_dot: { axis: "side", for: "The dot of a picked radio button." },
  progress: { axis: "height", for: "A progress bar." },
  dialog: { axis: "width", for: "A dialog." },
  password: { axis: "width", for: "A password field alone on a screen, under the lock screen's clock: wider than the clock without outweighing it." },
  column: { axis: "width", for: "The widest a column of text and fields grows." },
  rail: { axis: "width", for: "Kōbō's step rail on a wide screen." },
  sidebar: { axis: "width", for: "An app's sidebar, its list of pages beside the page: wider than the rail, for settings' names." },
  app_width: { axis: "width", for: "An app's window opens centred and floating at this width." },
  app_height: { axis: "height", for: "And at this height." },
  app_min_width: { axis: "width", for: "Tiled, an app's window never goes narrower." },
  app_min_height: { axis: "height", for: "Nor lower." },
  log: { axis: "height", for: "A log opened under an error or a progress bar." },
  panel: { axis: "width", for: "A panel of the shell, and a notification beside one: the control center's side sheet." },
  bar: { axis: "height", for: "The shell's bar along the top of every screen: a small control with 4 above and below." },
  bar_label: { axis: "width", for: "The widest a name the bar shows grows, past which it ends in an ellipsis." },
  thumbnail_width: { axis: "width", for: "A wallpaper's thumbnail in the shell's picker, 16:10 as the screens are." },
  thumbnail_height: { axis: "height", for: "Its height." },
  slider: { axis: "height", for: "A slider, a pill whose fill marks the value." },
  slider_compact: { axis: "height", for: "A slider in a dense panel, as tall as a compact row." },
  slider_stop: { axis: "side", for: "The dot of each step; off the grid, drawn centred on a point that moves with the value." },
  slider_anchor: { axis: "side", for: "The solid dot of the mark the value rests on; off the grid, as the stop is." },
  slider_anchor_compact: { axis: "side", for: "The anchor dot on a compact slider." },
  slider_hold: { axis: "width", for: "How near a drag holds on a sticky point before it pulls free." },
  scrollbar: { axis: "width", for: "A scrollbar's thumb, a thin pill: it only says where you are." },
  keycap: { axis: "height", for: "A keycap: as tall as a line of body text, and never narrower than tall." },
  neon_mark: { axis: "side", for: "The クバ mark drawn as lit neon tubes on Kōbō's done screen, with room for the glow." },
  hanko: { axis: "side", for: "The 完了 hanko stamped beside it." },
  sparkline: { axis: "height", for: "A sparkline in a panel, as tall as a large control so a graph and the buttons beside it line up." },
  motion_plot: { axis: "height", for: "The mockups' motion player's plot, from 0 to 1." },
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
  return table(
    ["Step", "Its length", ...SCALES.map((scale) => `${scale}×`)],
    names("space-").map((name) => [
      token(name.replace("-", "."), logical(name)),
      bar(name, "width"),
      ...SCALES.map((scale) => label(device(name, scale).words, "mono", device(name, scale).whole ? "on_surface" : "error")),
    ]),
  );
}

/** Every fixed size: its value, drawn the way it runs, and what it is for. */
function sizes(): HTMLElement {
  return el(
    "div",
    "f-scroll",
    table(
      ["Size", "Runs", "Drawn", "For"],
      names("size-").map((name) => {
        const short = name.slice("size-".length);
        const use = SIZE_FOR[short];
        return [
          token(`size.${short}`, logical(name)),
          label(use?.axis ?? "?", "body", "on_surface"),
          bar(name, use?.axis ?? "width"),
          prose(use?.for ?? "No words for this size: add them to `SIZE_FOR`.", "body", use ? "on_surface" : "error"),
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
    table(
      ["From", "Padding at its sides"],
      steps().map((each) => [label(`${each.from}`, "mono", "on_surface"), token(stepName(each.padding), String(length(each.padding)))]),
    ),
    el("div", "f-scroll", el("div", "f-stack", containerAt("size-panel", "A panel"), containerAt("size-app_min_width", "An app at its minimum"), containerAt("size-app_width", "An app at its default"))),
  );
}

/** A toolbar band: a small control, a step above and below, and nothing more. */
function band(): HTMLElement {
  const inner = row("s8", "start", button({ label: "Back", kind: "Text", size: "Sm" }), button({ label: "Continue", kind: "Filled", size: "Sm" }));
  const box = padded("container-0-padding", "space-s16", inner);
  return el(
    "div",
    "f-swatch-cell",
    box,
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
    token(`size.row ${length("size-row")}, size.row_two_line ${length("size-row_two_line")}: no row taller`),
  );
}

/** A card inside a page: the page's padding is the only one round it. */
function nestedContainer(): HTMLElement {
  const card = el("div", "f-card-box", content("A card", "Its own padding inside, none added outside"));
  return el(
    "div",
    "f-swatch-cell",
    padded("container-0-padding", null, card),
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
  return table(
    ["Size", ...SCALES.map((scale) => `${scale}×`)],
    names("size-").map((name) => [
      token(`size.${name.slice("size-".length)}`),
      ...SCALES.map((scale) => label(device(name, scale).words, "mono", device(name, scale).whole ? "on_surface" : "error")),
    ]),
  );
}

export default foundation({
  level: "foundations",
  slug: "space",
  title: "Space, sizes and the container",
  summary: "Every length on the 4 grid: the spacing steps, the fixed sizes of components, and the container that pads every page and panel at its sides.",
  live,
  description: [
    "Every spacing, size, radius, icon size, line height and shadow offset is a multiple of `grid`, 4 logical pixels, because the scales are 1, 1.25, 1.5 and 2: a multiple of 4 is a whole number of device pixels at every one of them. Hairlines are the exception, one device pixel at every scale, and two dots drawn centred on a moving point are allowed off the grid.",
    "Gaps, paddings and insets take a spacing step; a component's fixed measure takes its size, by name. A box sized by its words is rounded up to the next device pixel, and centring puts an odd spare pixel after the child, never before it.",
    "Every page's and panel's body is padded at its sides by the container, by its own width: 16 under 640, 24 from 640, 32 from 1024, Tailwind's container on its breakpoints. Above and below, content keeps the steps it is laid out with.",
    "Dense, never cramped: room that holds nothing is a bug. A band is its content and a step, a row its type's natural height, content runs to the container's padding, and padding is applied once.",
  ],
  tokens: [
    {
      name: "The spacing steps",
      described: "Every step as a bar of its length at the page's scale, and its device pixels at each scale: all whole.",
      sample: spacing,
    },
    {
      name: "Every fixed size",
      described: "Every component's fixed measure, by name, drawn the way it runs (a bar along a width or a height, a square for a side), with what it is for. A size wider than the page at this scale scrolls.",
      sample: sizes,
    },
    {
      name: "The container",
      described: "From which width of its own each padding applies, and a container at a width in each step, its padding shaded: a panel, an app at its minimum, an app at its default. A container wider than the page at this scale scrolls.",
      sample: container,
    },
  ],
  rules: [
    {
      name: "A band is its content and a step",
      described: "A footer, toolbar or header band is as tall as what is in it and one step above and below, and no taller: the specimen's bar is a small control with 16 above and below.",
      sample: band,
    },
    {
      name: "Rows are their type's natural height",
      described: "A row is `row` with one line or `row_two_line` with two, a line of text its style's line. A row taller than that holds air.",
      sample: rows,
    },
    {
      name: "Padding is applied once",
      described: "A container inside a container doesn't pad again: a card inside a page keeps its own padding inside it, and the page's padding is the only one round it. Content runs to the container's padding at either side; nothing pads it further in.",
      sample: nestedContainer,
    },
  ],
  modes: doesntApply("geometry never changes with the mode: the light and dark pages are the same boxes, and the geometry check holds them to it."),
  themes: doesntApply("a length is the same in every theme."),
  scales: [
    {
      name: "Every size in device pixels",
      described: "Each fixed size at each scale. A multiple of 4 is whole at all four; the off-grid dots aren't, which they may be, being drawn centred on a point that moves.",
      sample: deviceSizes,
    },
    {
      name: "The same panel at every scale",
      described: "Each box below is drawn at its own scale, whatever the toolbar's: its padding, its gaps and its controls are that scale's device pixels.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, live())))),
    },
  ],
  content: doesntApply("space holds no words: how a box grows with its words is each component's page, and how text wraps the type page."),
  motion: doesntApply("a length doesn't animate on its own; what moves, moves by its component's motion."),
  checks: [
    { check: "kubaos-tokens", holds: "Refuses a spacing step, size, radius, icon size, line height or shadow that isn't a multiple of `grid`, but for the sizes listed in `rules.off_grid`, and a container breakpoint or padding off the grid." },
    { check: "kubaos-design lint", holds: "Fails on a length written out in QML or Rust UI code: a length is a `Space`, `Size` or `Radius` token." },
    { check: "The geometry check", holds: "Holds every box to whole device pixels at every scale, and every `Page` and `Container` to the container's padding at its width (`container-padding`). An empty band is judged by eye in the shots." },
  ],
});
