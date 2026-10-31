// The Keycap page, with KeyHints: `docs/design/components/keycap.md`. A key drawn as a key, and a
// line of keys with what each does. A keycap is a picture of a key, not a control: it rests, by
// night and by day, and never lights.

import type { Words } from "../../lib/content.ts";
import { inline, type KeyHint, keycap, keyHints } from "../../lib/keycap.ts";
import { stack } from "../../lib/layout.ts";
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

const DESCRIBED =
  "The `keycap` type (Geist Mono 11, medium) in `on_surface_variant` on `surface_container_highest`, a hairline edge in `outline_variant` with a second hairline at the foot so the key stands on the page, at the small radius (`radius.small`). It never lights: no line, no glow.";

const DECIDED = [
  "Decided, lit: a keycap never lights. It is a picture of a key, not a control, so it has no line and no glow, by night or by day, and a power change only recolours it.",
  "Decided, pressed: a keycap doesn't go down when its key is pressed (the Command menu's footer could show it); it stays still, and what the key does shows the press.",
  "Decided, a room in iced: QML's `KeyHints` leaves out a hint that would pass its `room`, with every one after it, and iced's `key_hints` takes the same room. Today iced's has none and is cut by its container.",
];

export default page({
  level: "atoms",
  slug: "keycap",
  title: "Keycap and KeyHints",
  summary: "A key drawn as a key, as tall as a line of body text, and keys side by side with what each does: every key named on screen.",
  live: () => stack("s24", "start", keyHints(MENU), keyHints(KOBO), sentence()),
  description: [
    "A keycap draws a key as a key, so a hint is never text on text: the eye finds the key first and reads what it does after. It is `size.keycap` tall, as a line of body text, so it sits inline with body and caption text without making the line taller. It is as wide as its key and `space.s4` either side of it, and never narrower than tall, so a one-letter key is square; the key is centred across it on a whole device pixel.",
    DESCRIBED,
    "`KeyHints` puts keys and what each does side by side, `space.s16` apart: a combination's keys as keycaps `space.s4` apart, one keycap per key (`Ctrl+K` is Ctrl and K; a modifier's or an arrow's glyph is a key of its own however it is written, so `⌃K` is ⌃ and K and `← →` two arrows; a `+` alone is the plus key), then what it does in `caption`, `on_surface_variant`, `space.s8` after them. QML also reads Kotodama's form, `↵ Open · Tab Enter app · ⌃K Actions`. Given a room, a hint that would pass it is left out whole, with every hint after it, so none is cut through a keycap: the Command menu's footer and detail pane.",
    "Every key named on screen is a keycap: the Command menu's footer, its Action panel and its detail pane, Kōbō's rail, footers and screens, the control center, the wallpaper picker, the clock. The one exception is a `Hint` bubble, a single line of `label` text, which names a key in words. In a sentence, a keycap sits in a row centred on the line, `space.s4` from the words.",
    ...DECIDED,
  ],
  variants: [
    { name: "Keycap", for: "One key, alone or among others: Esc, Tab, ↵, a letter.", sample: caps },
    { name: "Key hints", for: "A footer's or a panel's keys and what each does.", sample: () => keyHints(MENU) },
    { name: "In a sentence", for: "A key named in a line of text, centred on the line, `space.s4` from the words.", sample: sentence },
    {
      name: "Given a room",
      for: "Hints in a fixed room (the Command menu's footer): a hint that would pass it is left out whole, and every one after it.",
      sample: () => stack("s8", "start", keyHints(MENU, { room: "calc(180 * var(--px))" }), keyHints(MENU)),
    },
  ],
  sizes: doesntApply("a keycap has one size, `size.keycap` tall, as a line of body text; key hints are a line of them."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a keycap is a picture of a key: it takes no pointer and no focus, and never lights.",
      pressed: "it doesn't go down when its key is pressed; what the key does shows the press.",
      disabled: "a key that does nothing here isn't shown: its hint is left out.",
      selected: "it holds no choice.",
      loading: "its key is known with the surface it is on.",
      empty: "a keycap always has a key; key hints with none draw nothing.",
      error: "it has no value to be wrong.",
    },
    rows: [
      { name: "A letter", cell: () => keycap("K") },
      { name: "A word", cell: () => keycap("Esc") },
      { name: "A glyph", cell: () => keycap("↵") },
      { name: "A long key", cell: () => keycap("Backspace") },
      { name: "Key hints", cell: () => keyHints(KOBO) },
    ],
    described: {
      rest: `Always. ${DESCRIBED} By day the same roles in daylight colours: a pale key with a darker edge.`,
    },
  },
  focus: doesntApply("a keycap isn't a control and isn't neon: it has no line and no glow, by night or by day."),
  content: {
    text: [
      { name: "A key", words: KEY, sample: (words: string) => keycap(words) },
      { name: "What it does", words: DOES, sample: (words: string) => keyHints([["Ctrl+↵", words]]) },
    ],
    missingPicture: doesntApply("a keycap is drawn: its key is text, and a glyph is a character of the font."),
    counts: [
      { name: "None", for: "Key hints with no hints draw nothing, and take no room.", sample: () => keyHints([]) },
      { name: "One", for: "A key alone, with no words: it says enough.", sample: () => keyHints([["Esc", ""]]) },
      { name: "A combination", for: "One keycap per key, `space.s4` apart.", sample: () => keyHints([["Ctrl+Shift+K", "Actions"]]) },
      { name: "Many", for: "A footer's hints, `space.s16` apart.", sample: () => keyHints([...MENU, ["Esc", "Close"]]) },
    ],
  },
  motion: {
    arrives: doesntApply("a keycap comes with the surface it is on; key hints that change as a selection moves are swapped at once, on the keystroke's frame."),
    leaves: doesntApply("a keycap goes with the surface it is on."),
    between: powerMoves(() => keyHints(MENU)),
  },
  keyboard: doesntApply("a keycap names a key: it takes no focus and no key of its own."),
  measures: {
    samples: nonEmpty([
      { name: "A letter", for: "Square: never narrower than tall.", sample: () => keycap("K") },
      { name: "A word", for: "The key and its padding.", sample: () => keycap("Ctrl") },
      { name: "Key hints", for: "The gaps between keys, words and hints.", sample: () => keyHints([["Ctrl+K", "Actions"], ["Esc", "back"]]) },
    ]),
    measures: [
      { what: "A keycap's height, and its narrowest width.", token: "size-keycap" },
      { what: "The key's line, which is the keycap's height.", token: "type-keycap-line" },
      { what: "The padding either side of the key, and the gap between a combination's keys and between a keycap and words in a sentence.", token: "space-s4" },
      { what: "The corners.", token: "radius-small" },
      { what: "The edge, and the second at the foot: one device pixel each.", token: "stroke-hairline_device_px" },
      { what: "The gap between the keys and what they do.", token: "space-s8" },
      { what: "The gap between hints.", token: "space-s16" },
      { what: "What a hint does, its line.", token: "type-caption-line" },
    ],
    reflow: {
      described: "Key hints given the window's width as their room: as the window narrows, the last hint that would pass it is left out whole, and so is every one after it.",
      sample: () => keyHints([...MENU, ["Ctrl+N", "Next"], ["Ctrl+P", "Previous"], ["Esc", "Close"], ["Ctrl+Shift+K", "Settings"]], { room: "100%" }),
    },
  },
});
