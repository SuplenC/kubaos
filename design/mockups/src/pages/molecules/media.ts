// The Media page: `docs/design/components/media.md`. A picture in a box of a fixed aspect that is
// the same before, after and without the picture, cut at the corner its place gives it.

import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { type Arrival, type MediaOptions, media, settle } from "../../lib/media.ts";
import { role } from "../../lib/radius.ts";
import { raw } from "../../lib/tokens.ts";
import { settings } from "../../site/settings.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";

const CAPTION: Words = { short: "Shinjuku, night", long: "Shinjuku, night: the neon sign over the alley behind the station, after the rain" };

/** A picture's room in a picker: a wallpaper's thumbnail. */
const THUMB = "var(--size-thumbnail_width)";
/** A picture's room on a page: half a column of text, less half the gap between two. */
const HALF = "calc((var(--size-column) - var(--space-s16)) / 2)";
/** The widest room shown: a column of text. */
const COLUMN = "var(--size-column)";

const CONTROL = role("control");
/** A path no picture is at: what a picture that can't be read leaves. */
const NOWHERE = "data:image/png;base64,AAAA";

function picture(options: Partial<MediaOptions> = {}, width = THUMB): HTMLElement {
  return within(width, media({ source: photo, corner: CONTROL, ...options }));
}

function held(arrival: Arrival, options: Partial<MediaOptions> = {}): HTMLElement {
  return picture({ arrival, ...options });
}

/** A picture that loads again on a press: the placeholder breathing, then the picture in its place. */
function live(): HTMLElement {
  const loaded = media({ source: wallpaper, corner: CONTROL, caption: "Kōbō's wallpaper, 16:10", arrival: "loaded" });
  const again = button({ label: "Load again", kind: "Tonal", size: "Sm", icon: "restart_alt" });
  let timer = 0;
  again.addEventListener("click", () => {
    window.clearTimeout(timer);
    settle(loaded, "loading");
    // Three of the placeholder's half breaths: long enough to see it breathe in and out.
    timer = window.setTimeout(() => settle(loaded, "loaded"), (3 * Number.parseFloat(raw("duration-hum"))) / settings().speed);
  });
  return within(HALF, stack("s16", "start", loaded, again));
}

/** Three pictures side by side in a room that doesn't divide by three: one height, feet in line. */
function three(): HTMLElement {
  const each = "calc((var(--size-column) - 2 * var(--space-s16) - 1px) / 3)";
  return within(COLUMN, row("s16", "start", ...[photo, wallpaper, NOWHERE].map((source) => within(each, media({ source, corner: CONTROL })))));
}

type MatrixState = "rest" | "loading" | "error" | "disabled";

const ROWS = [
  { name: "A picture", options: {} },
  { name: "With a caption", options: { caption: "Shinjuku, night" } },
] as const;

function cell(options: Partial<MediaOptions>, state: MatrixState): HTMLElement {
  if (state === "disabled") return picture({ ...options, dimmed: true });
  return held(state === "rest" ? "loaded" : state === "loading" ? "loading" : "broken", options);
}

export default page({
  level: "molecules",
  slug: "media",
  title: "Media",
  summary: "A picture in a box of a fixed aspect, held before it loads, so nothing around it moves when it arrives.",
  live,
  description: [
    "Rust `media::media(look, picture, aspect, corner)`, QML `Media`. Every picture KubaOS shows is one: a wallpaper's thumbnail, a picture in a card. Press Load again and the placeholder breathes until the picture covers it.",
    "Its box is as wide as its room and as tall as an `[aspect]` token makes it (a wallpaper's, 16:10), rounded down to the 4 px grid, so it lands on device pixels at every scale and boxes side by side whose widths differ by a device pixel come out one height, their feet in line. The picture covers the box, cut to fill it at its middle and clipped at its corner.",
    "Its corner is its place's, never its own: `radius.control` (12) in a picker, where a picture is something you press; the row radius (`radius.row`, 8) where it fills a card less its `space.s16`, since the card's 16 less 16 would leave no corner.",
    "Until the picture has loaded the box holds a block `Skeleton` of its own shape, breathing, so the box is the same before and after the load and nothing around it moves. A picture that can't be read leaves the placeholder where it is, held still: the box keeps its room and says no more. Dimmed, inside a disabled `PictureChoice`, the picture is at `opacity.disabled_content` and the placeholder is the disabled container, `on_surface` at `opacity.disabled_container`, still.",
    "A caption, where its place gives it one, sits `space.s8` under the box in `small`, `on_surface_variant`, breaking between words in the box's width.",
  ],
  variants: [
    { name: "A picture", for: "The box and its picture, alone.", sample: () => picture() },
    { name: "With a caption", for: "A line about it under the box: where it is from, what it shows.", sample: () => picture({ caption: "Shinjuku, night" }) },
    { name: "In a picker", for: "Something you press: the control's corner, 12.", sample: () => picture() },
    {
      name: "In a card",
      for: "Filling a card less its padding: the row radius, 8. The card's 16 less its 16 would leave no corner.",
      sample: () => within(HALF, card({ header: { title: "Wallpaper" }, body: [media({ source: photo, corner: role("row") })] })),
    },
  ],
  sizes: [
    { name: "A thumbnail", for: "A wallpaper picker's cell: `size.thumbnail_width`, 160 tall.", sample: () => picture({}, THUMB) },
    { name: "Half a column", for: "A picture beside another on a page: its height rounded down to the grid.", sample: () => picture({}, HALF) },
    { name: "A column", for: "A picture across a column of text.", sample: () => picture({}, COLUMN) },
  ],
  states: {
    shown: ["rest", "loading", "error", "disabled"],
    absent: {
      lit: "a picture takes no input: the `PictureChoice` built on it lights.",
      pressed: "it takes no press of its own.",
      selected: "a picked picture is a `PictureChoice`, on its own page.",
      empty: "a box with no picture is the placeholder: loading while it may come, held still once it can't.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "The picture, covering the box at its corner.",
      loading: "The placeholder, a block `Skeleton` of the box's shape: the accent's glass at `opacity.skeleton`, breathing over `hum`, at night with its faint glow at the peak.",
      error: "The picture can't be read: the placeholder stays, held still. The box keeps its room.",
      disabled: "In a disabled `PictureChoice`: the picture at `opacity.disabled_content`; with no picture, the placeholder is `on_surface` at `opacity.disabled_container`, still. The caption dims with it.",
    },
  },
  focus: doesntApply("a picture takes no input and has no line: the `PictureChoice` built on it lights."),
  content: {
    text: [{ name: "The caption", words: CAPTION, sample: (words: string) => picture({ caption: words }) }],
    missingPicture: [
      { name: "Not yet loaded", for: "The placeholder breathes in the box's shape.", sample: () => held("loading") },
      { name: "Can't be read", for: "The placeholder held still: the box keeps its room.", sample: () => picture({ source: NOWHERE }) },
    ],
    counts: [
      { name: "One", for: "A picture alone.", sample: () => picture() },
      { name: "Three side by side", for: "Widths a device pixel apart, one height: their feet line up, whatever each holds.", sample: three },
    ],
  },
  motion: {
    arrives: doesntApply("the picture takes the placeholder's place on the frame it has decoded, in the same box: nothing moves, and nothing fades."),
    leaves: doesntApply("it goes with what holds it."),
    between: doesntApply("its only motion is the placeholder's breath, the `Skeleton`'s, which that page plays with its recipe."),
  },
  keyboard: doesntApply("a picture takes no input: the `PictureChoice` built on it takes the keys."),
  measures: {
    samples: nonEmpty([
      { name: "With a caption", for: "The box, its corner, and the caption under it.", sample: () => picture({ caption: shown(CAPTION) }) },
      { name: "In a card", for: "The row radius, `space.s16` in.", sample: () => within(HALF, card({ header: { title: "Wallpaper" }, body: [media({ source: photo, corner: role("row") })] })) },
    ]),
    measures: [
      { what: "Its width to its height.", token: "aspect-wallpaper" },
      { what: "Its height, rounded down to it.", token: "grid" },
      { what: "Its corner in a picker.", token: "radius-control" },
      { what: "Between the box and its caption.", token: "space-s8" },
      { what: "The caption's line.", token: "type-small-line" },
      { what: "A wallpaper's thumbnail.", token: "size-thumbnail_width" },
    ],
    reflow: {
      described: "A picture is as wide as its room and as tall as its aspect makes that, down to the grid, at every window size. Nothing in it is a fixed width; its caption breaks between words.",
      sample: () => within("100%", media({ source: wallpaper, corner: CONTROL, caption: "Across the whole window." })),
    },
  },
});
