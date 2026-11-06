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
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

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
  return within(COLUMN, stack("s16", "start", loaded, again));
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
  lede: "A picture in a box of a fixed aspect, held before it loads, so nothing around it moves.",
  code: { rust: "media::media(look, picture, aspect, corner)", qml: "Media", unbuilt: "caption: not built yet" },
  stage: { live, hint: "press Load again", daylight: true },
  variants: [
    { name: "A picture", for: "the box and its picture", sample: () => picture() },
    { name: "With a caption", for: "a line about it under the box", sample: () => picture({ caption: "Shinjuku, night" }) },
    { name: "In a picker", for: "something you press: `radius.control`", sample: () => picture() },
    {
      name: "In a card",
      for: "filling a card less its padding: `radius.row`",
      sample: () => within(HALF, card({ header: { title: "Wallpaper" }, body: [media({ source: photo, corner: role("row") })] })),
    },
  ],
  sizes: [
    { name: "A thumbnail", for: "a picker's cell: `size.thumbnail_width`", sample: () => picture({}, THUMB) },
    { name: "Half a column", for: "beside another on a page", sample: () => picture({}, HALF) },
    { name: "A column", for: "across a column of text", sample: () => picture({}, COLUMN) },
  ],
  states: {
    shown: ["rest", "loading", "error", "disabled"],
    absent: {
      lit: "the `PictureChoice` built on it lights",
      pressed: "it takes no press",
      selected: "a picked picture is a `PictureChoice`",
      empty: "the placeholder: loading, or held still",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    looks: {
      rest: "the picture, covering the box at its corner",
      loading: "a `Skeleton` of the box's shape, breathing over `hum`",
      error: "the placeholder, held still; the box keeps its room",
      disabled: "picture at `opacity.disabled_content`, placeholder the disabled container",
    },
  },
  focus: doesntApply("a picture takes no input; the `PictureChoice` built on it lights."),
  content: {
    text: [{ name: "The caption", words: CAPTION, sample: (words: string) => picture({ caption: words }) }],
    missingPicture: [
      { name: "Not yet loaded", for: "the placeholder breathes", sample: () => held("loading") },
      { name: "Can't be read", for: "the placeholder held still", sample: () => picture({ source: NOWHERE }) },
    ],
    counts: [
      { name: "One", for: "a picture alone", sample: () => picture() },
      { name: "Three side by side", for: "widths a device pixel apart, one height, feet in line", sample: three },
    ],
  },
  motion: {
    arrives: doesntApply("the picture replaces the placeholder in the same box: nothing moves, nothing fades."),
    leaves: doesntApply("it goes with what holds it."),
    between: doesntApply("its only motion is the placeholder's breath, on the Skeleton page."),
  },
  keyboard: doesntApply("a picture takes no input; the `PictureChoice` built on it takes the keys."),
  measures: {
    samples: nonEmpty([
      { name: "With a caption", for: "the box, its corner, the caption", sample: () => picture({ caption: shown(CAPTION) }) },
      { name: "In a card", for: "the row radius, `space.s16` in", sample: () => within(HALF, card({ header: { title: "Wallpaper" }, body: [media({ source: photo, corner: role("row") })] })) },
    ]),
    measures: [
      { part: "Aspect", token: "aspect-wallpaper" },
      { part: "Height, rounded down", token: "grid" },
      { part: "Corner in a picker", token: "radius-control" },
      { part: "Box → caption", token: "space-s8" },
      { part: "Caption line", token: "type-small-line" },
      { part: "Thumbnail width", token: "size-thumbnail_width" },
    ],
    reflow: {
      note: "As wide as its room, as tall as its aspect makes that, down to the grid; the caption breaks between words.",
      sample: () => within("100%", media({ source: wallpaper, corner: CONTROL, caption: "Across the whole window." })),
    },
  },
});
