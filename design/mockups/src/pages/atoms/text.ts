// The Text page: `docs/design/components/text.md`. Words in a type style, picked by what the text
// is, in a colour role, in a box as wide as their advances and as tall as their lines. It isn't a
// control: it has no states of its own, and takes its control's colours where it is in one.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { icon } from "../../lib/icon.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { clamped, marked, STYLE_WORDS, styles, text, type TypeStyle, wrapped } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const TITLE: Words = { short: "Wi-Fi", long: "Samsung 990 Pro NVMe M.2 2 TB, the disk KubaOS is installed on" };
const BODY: Words = {
  short: "KubaOS takes the whole disk.",
  long: "KubaOS takes the whole disk and encrypts it with your passphrase. Everything on it now is erased, and nothing can be read back without the passphrase, so keep it somewhere safe.",
};

/** A row's title in a room `size.menu` wide: one line, ending in an ellipsis where it doesn't fit. */
function rowTitle(words = shown(TITLE)): HTMLElement {
  return within("var(--size-menu)", clamped(words, "body_strong"));
}

/** Running text in the reading column, breaking between words. */
function paragraph(words = shown(BODY)): HTMLElement {
  return within("var(--size-menu)", wrapped(words, "body", "on_surface_variant"));
}

/** A notification's body kept to two lines, the second ending in an ellipsis: QML's `maximumLines`. */
function twoLines(words = shown(BODY)): HTMLElement {
  return within("var(--size-menu)", clamped(words, "small", "on_surface_variant", 2));
}

/** A Command menu result with the characters the query `wi` matched in the accent. */
function matched(): HTMLElement {
  return marked("Wi-Fi networks", "body_strong", "on_surface", [{ start: 0, end: 2 }]);
}

/** A slider's value, tabular: each figure one width. */
function figures(): HTMLElement {
  return stack("s4", "start", text("11%", "label_number", "on_surface_variant"), text("88%", "label_number", "on_surface_variant"));
}

/** A text in each place it takes another's colours: on a filled control, and disabled with it. */
function inControls(): HTMLElement {
  return row("s16", "centre", button({ label: "Continue", kind: "Filled", size: "Md" }), button({ label: "Continue", kind: "Filled", size: "Md", disabled: true }));
}

/** `words` in a disabled control's content colour, `on_surface` at `opacity.disabled_content`, when `disabled`. */
function dimmed(words: HTMLElement, disabled: boolean): HTMLElement {
  if (disabled) words.style.color = "rgb(from var(--color-on_surface) r g b / var(--opacity-disabled_content))";
  return words;
}

/** Every style but the rest clock, whose digits are far wider than a card: Foundations, Type has it. */
type Sized = Exclude<TypeStyle, "clock_rest">;
const SIZED = styles().filter((style): style is Sized => style !== "clock_rest");

/** What each style is for, in a few words: Foundations, Type has it in full. */
const USE: Record<Sized, string> = {
  clock: "the clock of the lock surface as built",
  clock_place: "the lock kicker: where and which day",
  clock_date: "the lock kicker: the date",
  hero: "Kōbō's welcome and done lines",
  hero_number: "big figures: install percent, the clock popup's time",
  display: "a full-screen moment's heading",
  title: "a screen's or page's title",
  headline: "a big number",
  heading: "a section's title, a dialog's title",
  subheading: "a card's or an error's first line",
  body_large: "a lede, the text in a field",
  body: "running text, explanations, toasts",
  body_strong: "a list row's title, a value by its label",
  button_large: "a large button's label",
  button: "a regular button's label",
  button_small: "a small button's label",
  label: "a field's label, a chip, a quiet button",
  label_number: "a number changing in place",
  small: "a note under a field, a row's subtitle",
  caption: "version lines, what a key hint's key does",
  accent: "the Japanese accent above a title",
  mono: "logs, device paths, timings",
  keycap: "the key on a keycap",
  mono_large: "commands and keys as a line's subject",
};

export default page({
  level: "atoms",
  slug: "text",
  title: "Text",
  lede: "Words in a type style, picked by what the text is, in a colour role: every word on screen.",
  code: { rust: "text::text(content, style, color), text::line", qml: "Text" },
  stage: {
    live: () =>
      row(
        "s48",
        "start",
        stack("s12", "start", text("Network", "heading"), paragraph()),
        stack("s16", "start", row("s12", "centre", icon("wifi", "var(--type-body_strong-icon)", "var(--color-on_surface_variant)"), rowTitle()), matched(), figures()),
      ),
    hint: "try Text in the top bar: it wraps, or ellipsizes",
    daylight: true,
  },
  variants: [
    { name: "Text", for: "one line, as wide as its words", sample: () => text("Pick a disk", "title") },
    { name: "Wrapped", for: "given a width, breaks between words", sample: () => paragraph() },
    { name: "One line", for: "a fixed height: ends in an ellipsis", sample: () => rowTitle() },
    { name: "Some lines", for: "`maximumLines`, the last elided: QML only", sample: () => twoLines() },
    { name: "Coloured runs", for: "a query's matches: QML's `markup`", sample: matched },
    { name: "Tabular figures", for: "`label_number`: each figure one width", sample: figures },
  ],
  sizes: nonEmpty(
    SIZED.map((style) => ({
      name: style,
      for: USE[style],
      sample: () => text(STYLE_WORDS[style], style),
    })),
  ),
  states: {
    shown: ["rest", "disabled"],
    absent: {
      lit: "in a lit control it takes the lit colours",
      pressed: "it moves with its control",
      selected: "selecting words is a field's",
      loading: "words on their way are a `Skeleton` line",
      empty: "draws nothing, a line tall",
      error: "words about an error are `error` text or an `ErrorBlock`",
    },
    rows: [
      { name: "On the surface", cell: (state) => dimmed(text("Wi-Fi", "body_strong"), state === "disabled") },
      { name: "Supporting", cell: (state) => dimmed(text("Connected", "small", "on_surface_variant"), state === "disabled") },
      { name: "In a control", cell: (state) => button({ label: "Continue", kind: "Filled", size: "Md", disabled: state === "disabled" }) },
    ],
    looks: {
      rest: "its style in its role; a box on the grid",
      disabled: "its control's: `on_surface` at `opacity.disabled_content`",
    },
  },
  focus: doesntApply("not a control: no line and no glow; a lit control's label lights with it."),
  content: {
    text: [
      { name: "A row's title, one line", words: TITLE, sample: (words: string) => rowTitle(words) },
      { name: "Running text, wrapped", words: BODY, sample: (words: string) => paragraph(words) },
      { name: "Kept to two lines", words: BODY, sample: (words: string) => twoLines(words) },
    ],
    missingPicture: doesntApply("words: a glyph the font lacks falls back to the system's."),
    counts: [
      { name: "None", for: "draws nothing, keeps its line's height", sample: () => text("", "body") },
      { name: "One word", for: "as wide as the word", sample: () => text("Wi-Fi", "body") },
      { name: "Many lines", for: "as tall as its lines", sample: () => paragraph(BODY.long) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface; typed text appears at once."),
    leaves: doesntApply("it goes with its surface."),
    between: powerMoves(() => stack("s12", "start", text("Network", "heading"), inControls())),
  },
  keyboard: doesntApply("no focus and no key; selecting words is a field's."),
  measures: {
    samples: nonEmpty([
      { name: "One line", for: "its words' width, its style's line", sample: () => text("Pick a disk", "title") },
      { name: "Wrapped", for: "its room's width, its lines' height", sample: () => paragraph(BODY.long) },
      { name: "Beside an icon", for: "the style's icon size, centred", sample: () => row("s12", "centre", icon("wifi", "var(--type-body_strong-icon)", "var(--color-on_surface_variant)"), text("Wi-Fi", "body_strong")) },
    ]),
    measures: [
      { part: "Title line", token: "type-title-line" },
      { part: "Running text line", token: "type-body-line" },
      { part: "Row title line", token: "type-body_strong-line" },
      { part: "Icon beside a title", token: "type-body_strong-icon" },
      { part: "The samples' room", token: "size-menu" },
    ],
    reflow: {
      note: "Running text breaks between words up to `size.column`; a row's title ends in an ellipsis instead.",
      sample: () => within("min(100%, var(--size-column))", wrapped(BODY.long, "body"), clamped(TITLE.long, "body_strong")),
    },
  },
});
