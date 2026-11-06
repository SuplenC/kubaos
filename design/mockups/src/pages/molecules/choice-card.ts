// The ChoiceCard page: `docs/design/components/card.md`, the card picked from a set. A card that
// says what each choice means, picked as radios (one of the set) or as checkboxes (any of them):
// picked, its title is the accent's words and its mark is filled; its tube is cold, since the
// accent already means picked.

import { type ChoiceCardOptions, choiceCard, choiceGroup, type Picks } from "../../lib/card.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { filling, stack, within } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

const TITLE: Words = { short: "Erase", long: "Erase the whole disk and install KubaOS on it" };
const LINE: Words = { short: "Everything on it goes.", long: "Wipes all three partitions, Windows and its recovery partition included." };

/** A set's room on a page: a column of text. */
const COLUMN = "var(--size-column)";
/** One card's room: half the column, less half the gap between two. */
const HALF = "calc((var(--size-column) - var(--space-s16)) / 2)";

const ERASE: ChoiceCardOptions = { glyph: "hard_drive", title: "Erase the disk", subtitle: "Everything on it goes." };
const ALONGSIDE: ChoiceCardOptions = { glyph: "storage", title: "Install alongside", subtitle: "Keep Windows, shrink it by 120 GB." };
const SHELL: ChoiceCardOptions = { glyph: "terminal", title: "Developer tools", subtitle: "Compilers, Git and the language servers." };
const DESKTOP: ChoiceCardOptions = { glyph: "desktop_windows", title: "Office", subtitle: "LibreOffice and a PDF reader." };
const MEDIA: ChoiceCardOptions = { glyph: "headphones", title: "Media", subtitle: "Players for music and video." };
const GAMES: ChoiceCardOptions = { glyph: "mood", title: "Games", subtitle: "Steam and its runtime." };

/** `options` as a card in a room of its own, in `state`. */
function boxed(options: ChoiceCardOptions, state: Shown = "rest"): HTMLElement {
  return within(HALF, choiceCard({ ...options, ...stateOptions(state) }));
}

/** Kōbō's disk modes: two radio cards, the first picked. */
function modes(room = COLUMN): HTMLElement {
  return within(room, choiceGroup("one", 2, [choiceCard({ ...ERASE, picked: true }), choiceCard(ALONGSIDE)]));
}

/** Kōbō's software: four checkbox cards, two picked. */
function suites(room = COLUMN): HTMLElement {
  return within(
    room,
    choiceGroup("many", 2, [
      choiceCard({ ...SHELL, picks: "many", picked: true }),
      choiceCard({ ...DESKTOP, picks: "many" }),
      choiceCard({ ...MEDIA, picks: "many", picked: true }),
      choiceCard({ ...GAMES, picks: "many" }),
    ]),
  );
}

function live(): HTMLElement {
  return within(ACROSS, stack("s32", "start", filling(modes("100%")), filling(suites("100%"))));
}

const ROWS: readonly { name: string; options: ChoiceCardOptions }[] = [
  { name: "One of a set, not picked", options: ALONGSIDE },
  { name: "One of a set, picked", options: { ...ERASE, picked: true } },
  { name: "Any of a set, not picked", options: { ...DESKTOP, picks: "many" } },
  { name: "Any of a set, picked", options: { ...SHELL, picks: "many", picked: true } },
];

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

/** A picked card and one not picked, in `state`: what the lighting moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => within(COLUMN, choiceGroup("one", 2, [choiceCard({ ...ERASE, picked: true, ...stateOptions(state) }), choiceCard({ ...ALONGSIDE, ...stateOptions(state) })]));
}

const ON = "a picked card and one not";

/** A set of `picks` with `count` cards, the first picked. */
function set(picks: Picks, count: number): HTMLElement {
  const all = [ERASE, ALONGSIDE, SHELL, DESKTOP, MEDIA, GAMES].slice(0, count);
  return within(COLUMN, choiceGroup(picks, 2, all.map((options, index) => choiceCard({ ...options, picks, picked: index === 0 }))));
}

export default page({
  level: "molecules",
  slug: "choice-card",
  title: "ChoiceCard",
  lede: "A card picked from a set, saying what each choice means: one of the set, as radios, or any of it, as checkboxes.",
  code: { rust: "card::choice_card", qml: "ChoiceCard" },
  stage: {
    live,
    hint: "click a card, or Tab, the arrows and Space",
    daylight: true,
  },
  variants: [
    { name: "One of a set", for: "as radios: picking one unpicks the last", sample: modes },
    { name: "Any of a set", for: "as checkboxes: each picked or not", sample: suites },
    { name: "A title alone", for: "no line under it", sample: () => boxed({ glyph: "dark_mode", title: "Dark", picked: true }) },
    { name: "Without a symbol", for: "the title starts at the padding", sample: () => boxed({ title: "Erase the disk", subtitle: "Everything on it goes." }) },
  ],
  sizes: doesntApply("one padding and one corner; as wide as its cell of the grid."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "picked or not is its value: a row each",
      loading: "one waiting on a scan is disabled",
      empty: "always a title; a set has at least two",
      error: "disabled, its line saying why",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => boxed(options, state) }))),
    looks: {
      rest: "flat, soft hairline; picked, `primary_lit` words and a filled mark",
      lit: "the cold tube inside its edge, glow at night",
      pressed: "`cold_core`, glow × `focus.pressed_glow`; the title goes `primary_lit`",
      disabled: "content at `opacity.disabled_content`, no line, skipped by keys",
    },
  },
  focus: {
    tubes: [
      { name: "Not picked", tone: "cold", filled: false, content: false },
      { name: "Picked", tone: "cold", filled: false, content: false },
    ],
    looks: {
      lit: "cold tube at `radius.content`, glow outside: the accent already means picked",
      signsOff: "the tube as `cold_glass`, no glow on the tube or words",
    },
    signsOff: (index: number) => boxed(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => boxed({ ...ERASE, title: words, picked: true }) },
      { name: "The line", words: LINE, sample: (words: string) => boxed({ ...ERASE, subtitle: words }) },
    ],
    missingPicture: doesntApply("its symbol is always there; a choice made by its picture is a `PictureChoice`."),
    counts: [
      { name: "Two", for: "the least a set holds", sample: () => set("one", 2) },
      { name: "Three", for: "the last row's spare cell stays empty", sample: () => set("one", 3) },
      { name: "Many", for: "rows of two, each one height", sample: () => set("many", 6) },
    ],
  },
  motion: {
    arrives: doesntApply("a set comes with its page."),
    leaves: doesntApply("it goes with its page."),
    between: [
      {
        name: "Picked",
        what: "Its words fade to `primary_lit` and its mark fills over `short` on `standard`, as the last pick fades back.",
        tracks: [
          { part: "words", on: "the card picked", target: ".k-choice-card:last-child", property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" },
          { part: "mark", on: "the card picked", target: ".k-choice-card:last-child", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" },
          { part: "words", on: "the card it unpicks", target: ".k-choice-card:first-child", property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "mark", on: "the card it unpicks", target: ".k-choice-card:first-child", property: "--checked", from: 1, to: 0, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: pair(),
      },
      { name: "Focus arrives", what: "The cold tube and glow fade in over `short` on `standard`.", tracks: lights(".k-choice-card", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-choice-card", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot, the glow brightens, and its title goes to the picked words.", tracks: press(".k-choice-card", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "the set is one stop, on its picked card" },
      { keys: ["←", "→", "h", "l"], does: "the card before or after; radios pick as they go" },
      { keys: ["↑", "↓", "k", "j"], does: "a row of the grid" },
      { keys: ["Home", "End"], does: "the first or last card" },
      { keys: ["Space", "Enter"], does: "picks or turns it, once per press" },
    ],
    order: "One stop in reading order; the arrows pass a disabled card and stop at the ends.",
    lands: "On the picked card; a pick keeps the keyboard there.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A set of two", for: "padding, symbol, title, line, mark, the gap", sample: modes },
    ]),
    measures: [
      { part: "Inset, gap between cards", token: "space-s16" },
      { part: "Corner", token: "radius-content" },
      { part: "Edge", token: "stroke-hairline_device_px" },
      { part: "Symbol", token: "type-subheading-icon" },
      { part: "Symbol → title → mark", token: "space-s8" },
      { part: "Title line", token: "type-subheading-line" },
      { part: "Title → line", token: "space-s4" },
      { part: "Line under the title", token: "type-small-line" },
      { part: "Mark", token: "size-check" },
      { part: "Box mark's corner", token: "radius-small" },
    ],
    reflow: {
      note: "A grid of two across its room, each row one height; the mark stays on the title's first line.",
      sample: () => within("100%", choiceGroup("one", 2, [choiceCard({ ...ERASE, title: shown(TITLE), picked: true }), choiceCard(ALONGSIDE)])),
    },
  },
});
