// The Keycap page, with KeyHints: `docs/design/components/keycap.md`. A key drawn as a key, and a
// line of keys with what each does. A keycap is a picture of a key, not a control: it rests, by
// night and by day, and never lights.

import type { Words } from "../../lib/content.ts";
import { inline, type KeyHint, keycap, keyHints } from "../../lib/keycap.ts";
import { row, stack } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const KEY: Words = { short: "K", long: "Print Screen" };
const DOES: Words = { short: "back", long: "Open it in a new window beside this one" };

/** The Command menu's footer. */
const MENU: readonly KeyHint[] = [
  ["↵", "Open"],
  ["Tab", "Enter app"],
  ["Ctrl+K", "Actions"],
];

/** Kōbō's footer. */
const KOBO: readonly KeyHint[] = [
  ["Enter", "continue"],
  ["Ctrl+K", "actions"],
  ["Esc", "back"],
];

/** A key named in a sentence: `Space` and `Del` inline with body text. */
function sentence(): HTMLElement {
  return inline(text("Press", "body"), keycap("Space"), text("to pick a layout, or", "body"), keycap("Del"), text("to drop it.", "body"));
}

/** The keys the specimen shows, side by side. */
function caps(): HTMLElement {
  return stack("s16", "start", inline(...["Esc", "Tab", "Enter", "↵", "K", "Ctrl"].map(keycap)), keyHints([["← →", ""], ["+", ""]]));
}

export default page({
  level: "atoms",
  slug: "keycap",
  title: "Keycap and KeyHints",
  lede: "A key drawn as a key, and keys side by side with what each does.",
  code: { rust: "keycap::keycap(look, key), keycap::key_hints(look, hints)", qml: "Keycap, KeyHints" },
  stage: {
    live: () => row("s64", "start", stack("s24", "start", keyHints(MENU), keyHints(KOBO), sentence()), caps()),
    hint: "a picture of a key: nothing to press",
    daylight: true,
  },
  variants: [
    { name: "Keycap", for: "one key, alone or among others", sample: caps },
    { name: "Key hints", for: "a footer's keys and what each does", sample: () => keyHints(MENU) },
    { name: "In a sentence", for: "centred on the line, `space.s4` from the words", sample: sentence },
    {
      name: "Given a room",
      for: "a hint that would pass it goes, with all after it",
      sample: () => stack("s8", "start", keyHints(MENU, { room: "calc(180 * var(--px))" }), keyHints(MENU)),
    },
  ],
  sizes: doesntApply("one size: `size.keycap` tall, a line of body text."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a picture of a key: it never lights",
      pressed: "what the key does shows the press",
      disabled: "a key that does nothing here isn't shown",
      selected: "it holds no choice",
      loading: "its key comes with its surface",
      empty: "key hints with none draw nothing",
      error: "it has no value to be wrong",
    },
    rows: [
      { name: "A letter", cell: () => keycap("K") },
      { name: "A word", cell: () => keycap("Esc") },
      { name: "A glyph", cell: () => keycap("↵") },
      { name: "A long key", cell: () => keycap("Backspace") },
      { name: "Key hints", cell: () => keyHints(KOBO) },
    ],
    looks: {
      rest: "`keycap` type on `surface_container_highest`, a hairline and a foot",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow."),
  content: {
    text: [
      { name: "A key", words: KEY, sample: (words: string) => keycap(words) },
      { name: "What it does", words: DOES, sample: (words: string) => keyHints([["Ctrl+↵", words]]) },
    ],
    missingPicture: doesntApply("its key is text, and a glyph is a character of the font."),
    counts: [
      { name: "None", for: "draws nothing, takes no room", sample: () => keyHints([]) },
      { name: "One", for: "a key alone, with no words", sample: () => keyHints([["Esc", ""]]) },
      { name: "A combination", for: "one keycap per key, `space.s4` apart", sample: () => keyHints([["Ctrl+Shift+K", "Actions"]]) },
      { name: "Many", for: "a footer's hints, `space.s16` apart", sample: () => keyHints([...MENU, ["Esc", "Close"]]) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface; hints swap at once with the selection."),
    leaves: doesntApply("it goes with its surface."),
    between: powerMoves(() => keyHints(MENU)),
  },
  keyboard: doesntApply("it names a key: no focus and no key of its own."),
  measures: {
    samples: nonEmpty([
      { name: "A letter", for: "square: never narrower than tall", sample: () => keycap("K") },
      { name: "A word", for: "the key and its padding", sample: () => keycap("Ctrl") },
      { name: "Key hints", for: "the gaps", sample: () => keyHints([["Ctrl+K", "Actions"], ["Esc", "back"]]) },
    ]),
    measures: [
      { part: "Height, narrowest width", token: "size-keycap" },
      { part: "Key's line", token: "type-keycap-line" },
      { part: "Padding, gap between keys", token: "space-s4" },
      { part: "Corner", token: "radius-small" },
      { part: "Edge and foot", token: "stroke-hairline_device_px" },
      { part: "Keys → words", token: "space-s8" },
      { part: "Between hints", token: "space-s16" },
      { part: "Words' line", token: "type-caption-line" },
    ],
    reflow: {
      note: "As the window narrows, the hint that would pass it goes whole, with every one after it.",
      sample: () => keyHints([...MENU, ["Ctrl+N", "Next"], ["Ctrl+P", "Previous"], ["Esc", "Close"], ["Ctrl+Shift+K", "Settings"]], { room: "100%" }),
    },
  },
});
