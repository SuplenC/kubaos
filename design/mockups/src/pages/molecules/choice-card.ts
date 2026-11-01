// The ChoiceCard page: `docs/design/components/card.md`, the card picked from a set. A card that
// says what each choice means, picked as radios (one of the set) or as checkboxes (any of them):
// picked, its title is the accent's words and its mark is filled; its tube is cold, since the
// accent already means picked.

import { type ChoiceCardOptions, choiceCard, choiceGroup, type Picks } from "../../lib/card.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack, within } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

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
function modes(): HTMLElement {
  return within(COLUMN, choiceGroup("one", 2, [choiceCard({ ...ERASE, picked: true }), choiceCard(ALONGSIDE)]));
}

/** Kōbō's software: four checkbox cards, two picked. */
function suites(): HTMLElement {
  return within(
    COLUMN,
    choiceGroup("many", 2, [
      choiceCard({ ...SHELL, picks: "many", picked: true }),
      choiceCard({ ...DESKTOP, picks: "many" }),
      choiceCard({ ...MEDIA, picks: "many", picked: true }),
      choiceCard({ ...GAMES, picks: "many" }),
    ]),
  );
}

function live(): HTMLElement {
  return stack("s32", "start", modes(), suites());
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
  summary: "A card picked from a set, saying what each choice means: one of the set, as radios, or any of it, as checkboxes.",
  live,
  description: [
    "Rust `card::choice_card`, QML `ChoiceCard`. A choice that needs a sentence to say what it means (Kōbō's disk modes, the software to install) is a card of its own, side by side with the others in a grid, `space.s16` apart. A choice a word says is a `Radio`, a `Checkbox` or `Tabs`.",
    "It is a `Card`: raised one step, `surface_container_high` by day and `surface_container_highest` at night, at the surface radius (24), `space.s16` inside. Its header is the card's: a symbol at `subheading`'s 24, its title in `subheading` `space.s8` after it, and its line in `small`, `on_surface_variant`, `space.s4` below. Its mark ends the title's first line, centred on it: a radio's disc for one of a set, a checkbox's box for any of it, both `size.check` across and drawn as their atoms draw them.",
    "Picked, it says so as every pick does: in the accent's words. Its title and symbol are `primary_lit` (the core at night, the tone by day), with the label glow round them at night while the signs are on (`focus.label_glow` over `focus.label_near`), and its mark is filled with the accent, the disc's dot or the box's tick on it. Its surface doesn't change: a big surface takes few effects. Picking fades its words and its mark over `short`.",
    "Its states follow the neon, as a card's do. Lit, its tube burns inside its edge at its own corner with the glow outside at night, cold (`cold` at `focus.tube`), since the accent already means picked here; its mark lights with it. Pressed, the tube runs white-hot, the glow brightens, by day the cold tone tints it (`opacity.press_tint`), and its title goes to the picked words over `micro` before letting go picks it. Disabled (a choice that can't be had: no room left to install alongside), it keeps its resting surface and dims what it holds: its words, symbol and mark at `opacity.disabled_content` of `on_surface`, the mark's box at `opacity.disabled_container`, picked or not; no line, no glow, no press, and the keyboard passes it by.",
  ],
  variants: [
    { name: "One of a set", for: "Exactly one picked, as radios: a disc. Picking one unpicks the one that was.", sample: modes },
    { name: "Any of a set", for: "Each picked or not, as checkboxes: a box with a tick.", sample: suites },
    { name: "A title alone", for: "A choice its title says: no line under it.", sample: () => boxed({ glyph: "dark_mode", title: "Dark", picked: true }) },
    { name: "Without a symbol", for: "A choice with nothing to draw: the title starts at the padding.", sample: () => boxed({ title: "Erase the disk", subtitle: "Everything on it goes." }) },
  ],
  sizes: doesntApply("a choice card has one padding and one corner, and is as wide as its cell of the set's grid."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "picked and not are its value, and each is a row here, so every state shows in both.",
      loading: "a choice is known as the set shows; one that waits on a scan is disabled until it can be had.",
      empty: "a card always has a title; a set has at least two cards.",
      error: "a choice that can't be had is disabled, and its line says why.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => boxed(options, state) }))),
    described: {
      rest: "Raised one step: `surface_container_highest` at night, `surface_container_high` by day. Not picked, its title and symbol are `on_surface` and its mark empty; picked, they are `primary_lit` with the label glow at night, and the mark filled with the accent. By day the same roles in daylight colours, with no glow.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the cold tube, `cold` at `focus.tube`, inside its edge with the glow outside; the mark lights with it. By day the tube with no glow.",
      pressed: "Lit and pressed: the tube runs white-hot (`cold_core`), the glow `focus.pressed_glow` times the lit one, by day a tint of the cold tone at `opacity.press_tint`; the title goes to `primary_lit` and the mark's fill lifts. Letting go picks it.",
      disabled: "Its resting surface; its words, symbol and mark grey at `opacity.disabled_content` of `on_surface`, the mark's box at `opacity.disabled_container`, picked or not. No line, no glow, no press, skipped by the keyboard.",
    },
  },
  focus: {
    tubes: [
      { name: "Not picked", tone: "cold", filled: false, content: false },
      { name: "Picked", tone: "cold", filled: false, content: false },
    ],
    looks: {
      lit: "At night at rest the sign burns: the cold tube inside the card's edge at `radius.surface`, the glow outside it. A picked card's words keep their label glow; nothing else in it lights.",
      signsOff: "Still lit with the neon unpowered, by day always and at night while a power change has the lights out: the tube is `cold_glass`, with no glow, and a picked card's words have no glow.",
    },
    signsOff: (index: number) => boxed(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => boxed({ ...ERASE, title: words, picked: true }) },
      { name: "The line", words: LINE, sample: (words: string) => boxed({ ...ERASE, subtitle: words }) },
    ],
    missingPicture: doesntApply("a choice card draws no picture: its symbol is one of KubaOS Symbols, always there. A choice made by its picture is a `PictureChoice`."),
    counts: [
      { name: "Two", for: "The least a set holds: Kōbō's disk modes.", sample: () => set("one", 2) },
      { name: "Three", for: "An odd one out leaves its cell of the last row empty: the cards keep their width.", sample: () => set("one", 3) },
      { name: "Many", for: "Rows of two, each row's cards one height.", sample: () => set("many", 6) },
    ],
  },
  motion: {
    arrives: doesntApply("a set comes with the page it is on."),
    leaves: doesntApply("it goes with the page it is on."),
    between: [
      {
        name: "Picked",
        what: "A press, Space or Enter, or an arrow in a set of one: its title and symbol fade to `primary_lit` and its mark fills over `short` on `standard`, as the one that was picked fades back. Never the flicker.",
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
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on it, it lights up over `short` on `standard`: the cold tube and its glow fade in.", tracks: lights(".k-choice-card", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-choice-card", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "A lit card pressed: over `micro` its tube runs white-hot, the glow brightens, by day the cold tone tints it, and its title goes to the picked words.", tracks: press(".k-choice-card", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Into the set or on out of it: the set is one stop, on its picked card (one of a set) or the card the keyboard was last on (any of it), else its first." },
      { keys: ["←", "→", "h", "l"], does: "To the card before or after, in reading order. In a set of one, it picks that card as it goes, as radios do." },
      { keys: ["↑", "↓", "k", "j"], does: "To the card above or below, a row of the grid at a time." },
      { keys: ["Home", "End"], does: "To the first or the last card." },
      { keys: ["Space", "Enter"], does: "Picks the card the keyboard is on (one of a set), or turns it (any of it), once per press, never on key repeat." },
    ],
    order: "One stop in the surface's reading order. The arrows pass a disabled card by and stop at the set's ends.",
    lands: "A page that asks the choice starts on the set, on its picked card. A pick keeps the keyboard on the card.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A set of two", for: "Each card's padding, its symbol, title, line and mark, and the gap between two.", sample: modes },
    ]),
    measures: [
      { what: "Inside its edge, and between two cards across and down.", token: "space-s16" },
      { what: "Its corners.", token: "radius-surface" },
      { what: "The symbol, `subheading`'s.", token: "type-subheading-icon" },
      { what: "Between the symbol, the title and the mark.", token: "space-s8" },
      { what: "The title's line.", token: "type-subheading-line" },
      { what: "Between the title and its line.", token: "space-s4" },
      { what: "The line under the title.", token: "type-small-line" },
      { what: "The mark's disc or box.", token: "size-check" },
      { what: "The mark's corner on a box.", token: "radius-small" },
    ],
    reflow: {
      described: "A set is a grid of two across its room, its columns split in device pixels, each row's cards one height. A card's title and line break between words where they don't fit; the mark stays on the title's first line.",
      sample: () => within("100%", choiceGroup("one", 2, [choiceCard({ ...ERASE, title: shown(TITLE), picked: true }), choiceCard(ALONGSIDE)])),
    },
  },
});
