// The Text page: `docs/design/components/text.md`. Words in a type style, picked by what the text
// is, in a colour role, in a box as wide as their advances and as tall as their lines. It isn't a
// control: it has no states of its own, and takes its control's colours where it is in one.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { icon } from "../../lib/icon.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { clamped, marked, STYLE_WORDS, styles, text, wrapped } from "../../lib/text.ts";
import { words as said } from "../../lib/tokens.ts";
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

const DESCRIBED =
  "A type style (`[type.*]`: its size, line, weight, family and features) in a colour role. Its box is as wide as the words' advances rounded up to a device pixel, and its style's line tall, so every text box is on the grid. Plain ASCII is laid out unkerned with no ligatures, as iced does; anything else is shaped with kerning.";

export default page({
  level: "atoms",
  slug: "text",
  title: "Text",
  summary: "Words in a type style, picked by what the text is, in a colour role: every word on screen.",
  live: () =>
    stack(
      "s16",
      "start",
      text("Network", "heading"),
      paragraph(),
      row("s12", "centre", icon("wifi", "var(--type-body_strong-icon)", "var(--color-on_surface_variant)"), rowTitle()),
      matched(),
    ),
  description: [
    "Every word on screen is a `Text`: Rust `text::text(content, style, color)` and `text::line`, QML `Text` (`text`, `style`, `color`). The style is picked by what the text is (a row's title is `body_strong`, a note `small`), never by how big it should look; Foundations, Type has every style and what it is for.",
    DESCRIBED,
    "Given less room than its words, it breaks between words onto as many lines as it needs, each the style's line tall, at the same places in both toolkits: QML measures the words as iced lays them out (`Advances`), so a box is as wide in QML as in iced to a hundredth of a pixel.",
    "One line in a box of fixed height (a row's title, a card's name) ends in an ellipsis where it doesn't fit: `text::line`, or `maximumLines: 1`. In the shell `maximumLines` keeps any number of lines (a window's title, a notification's body), the last ending in an ellipsis at its room.",
    "Coloured runs (the characters a Command menu query matched) are QML's `markup`, drawn over the measured words while they keep to one line, so the box is the plain text's. Tabular figures come with the style (`label_number`); a number that ticks in place is a `Figure`.",
    "It has no states of its own: a control's text takes the control's colours, and dims with it when it is disabled. The geometry check reads every text as a `text` node with its words and its natural size.",
  ],
  variants: [
    { name: "Text", for: "Words on one line, as wide as they are: `text::text`.", sample: () => text("Pick a disk", "title") },
    { name: "Wrapped", for: "Running text given a width: it breaks between words, each line the style's line tall.", sample: () => paragraph() },
    { name: "One line", for: "A row's or a card's name in a box of fixed height: it ends in an ellipsis where it doesn't fit (`text::line`, `maximumLines: 1`).", sample: () => rowTitle() },
    { name: "Some lines", for: "The shell's window titles and notification bodies: `maximumLines` lines, the last ending in an ellipsis. QML only.", sample: () => twoLines() },
    { name: "Coloured runs", for: "The characters a Command menu query matched, in the accent: QML's `markup`, on one line.", sample: matched },
    { name: "Tabular figures", for: "A number that changes in place, each figure one width: `label_number`.", sample: figures },
  ],
  sizes: nonEmpty(
    styles().map((style) => ({
      name: style,
      for: said(`type-${style}-use`),
      sample: () => text(STYLE_WORDS[style], style),
    })),
  ),
  states: {
    shown: ["rest", "disabled"],
    absent: {
      lit: "a text takes no pointer and no focus; inside a lit control it takes the control's lit colours.",
      pressed: "it takes no press; a control's text moves with the control.",
      selected: "selecting words in a field is the field's; a text alone isn't selectable.",
      loading: "words still on their way are a `Skeleton` line in their place.",
      empty: "an empty text draws nothing and is a line tall.",
      error: "it has no value to be wrong: words about an error are `error` text or an `ErrorBlock`.",
    },
    rows: [
      { name: "On the surface", cell: (state) => dimmed(text("Wi-Fi", "body_strong"), state === "disabled") },
      { name: "Supporting", cell: (state) => dimmed(text("Connected", "small", "on_surface_variant"), state === "disabled") },
      { name: "In a control", cell: (state) => button({ label: "Continue", kind: "Filled", size: "Md", disabled: state === "disabled" }) },
    ],
    described: {
      rest: `${DESCRIBED} By day the same roles in daylight colours: dark words on a pale surface.`,
      disabled: "A text has no disabled state of its own: inside a disabled control it takes the control's disabled content, `on_surface` at `opacity.disabled_content`, its icon dimming with it.",
    },
  },
  focus: doesntApply("a text isn't a control: it has no line and no glow. A lit control's label lights as its control's page says."),
  content: {
    text: [
      { name: "A row's title, one line", words: TITLE, sample: (words: string) => rowTitle(words) },
      { name: "Running text, wrapped", words: BODY, sample: (words: string) => paragraph(words) },
      { name: "Kept to two lines", words: BODY, sample: (words: string) => twoLines(words) },
    ],
    missingPicture: doesntApply("a text is words: a glyph the font lacks falls back to the system's, as both toolkits do."),
    counts: [
      { name: "None", for: "An empty text draws nothing and keeps its line's height, so a row with none is as tall as one with words.", sample: () => text("", "body") },
      { name: "One word", for: "As wide as the word.", sample: () => text("Wi-Fi", "body") },
      { name: "Many lines", for: "As tall as its lines.", sample: () => paragraph(BODY.long) },
    ],
  },
  motion: {
    arrives: doesntApply("a text comes with the surface it is on; typed text appears at once (`instant`)."),
    leaves: doesntApply("a text goes with the surface it is on."),
    between: powerMoves(() => stack("s12", "start", text("Network", "heading"), inControls())),
  },
  keyboard: doesntApply("a text takes no focus and no key; selecting words is a field's."),
  measures: {
    samples: nonEmpty([
      { name: "One line", for: "As wide as its words, rounded up to a device pixel, the style's line tall.", sample: () => text("Pick a disk", "title") },
      { name: "Wrapped", for: "As wide as its room, as tall as its lines.", sample: () => paragraph(BODY.long) },
      { name: "Beside an icon", for: "The icon takes the style's icon size, centred on the line.", sample: () => row("s12", "centre", icon("wifi", "var(--type-body_strong-icon)", "var(--color-on_surface_variant)"), text("Wi-Fi", "body_strong")) },
    ]),
    measures: [
      { what: "A title's line, which is its box's height.", token: "type-title-line" },
      { what: "Running text's line: each wrapped line is this tall.", token: "type-body-line" },
      { what: "A row title's line.", token: "type-body_strong-line" },
      { what: "The icon beside a row's title.", token: "type-body_strong-icon" },
      { what: "The room the one-line and the wrapped samples are given, a menu's width.", token: "size-menu" },
    ],
    reflow: {
      described: "Running text in the window's width, up to the reading column (`size.column`): it breaks between words as the window narrows, never past its room, and a row's one-line title ends in an ellipsis instead.",
      sample: () => within("min(100%, var(--size-column))", wrapped(BODY.long, "body"), clamped(TITLE.long, "body_strong")),
    },
  },
});
