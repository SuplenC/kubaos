// The EmptyList page: `docs/design/components/list.md`. A list's well with nothing in it, one centred
// line saying so; and while its rows are still being found, loading rows in their place.

import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { emptyList, type ListRowSpec, list } from "../../lib/list.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { WELL_ROOM } from "./list.ts";

const LINE: Words = { short: "No devices found", long: "No Bluetooth devices nearby. Put yours in pairing mode to see it here." };
const NETWORKS: Words = { short: "No networks found", long: "No Wi-Fi networks in range. Move closer to one, or plug in a cable." };

/** A scan's rows still being found, each a different share of its words' room. */
const SEARCHING: readonly ListRowSpec[] = [
  { title: "", loading: true, share: 0.6 },
  { title: "", loading: true, share: 0.45 },
  { title: "", loading: true, share: 0.75 },
];

function empty(words: string): HTMLElement {
  return within(WELL_ROOM, emptyList(words));
}

function loading(count = 3): HTMLElement {
  return within(WELL_ROOM, list(SEARCHING.slice(0, count)));
}

const MATRIX = [
  { name: "Bluetooth", words: LINE },
  { name: "Wi-Fi", words: NETWORKS },
] as const;

type MatrixState = "empty" | "loading";

export default page({
  level: "molecules",
  slug: "empty-list",
  title: "EmptyList",
  summary: "A list's well with nothing in it: one centred line saying so, or rows still being found in their place.",
  live: () => stack("s16", "start", empty(shown(LINE)), loading()),
  description: [
    "Rust `list::empty_list`, QML `EmptyList`. The control center's Bluetooth tab with no devices nearby, and its Wi-Fi tab while a scan has found nothing yet.",
    "The well of a List, as a List draws it (`radius.content`, `surface_container_low` at `opacity.well`, its soft hairline, `space.s8` inside), holding one line in `body`, `on_surface_variant`, centred across it on whole device pixels, the odd one after it, `space.s16` above and below inside the well's padding: 24 from its edge, where a row's words would start. The line says what is missing and, where it can, what to do: never a bare \"No items\". Too long for the well, it ends in an ellipsis.",
    "While the rows are still being found (a scan that has found nothing yet) the well holds loading rows instead: each a tiled two-line row's place, its tile and its two lines as skeletons, their words a different share of the room each, so they don't read as one block. A row found takes a loading row's place, and nothing moves. A scan that ends with nothing found shows the line.",
    "It isn't a stop for Tab: there is nothing in it to move through. A place with nothing in it that isn't a list (a page, a panel) is an EmptyState, with a symbol, a title and an action.",
  ],
  variants: [
    { name: "Nothing found", for: "A list a scan or a search filled with nothing: the line saying so.", sample: () => empty(shown(LINE)) },
    { name: "Still being found", for: "A scan with nothing found yet: loading rows in the rows' place.", sample: () => loading() },
  ],
  sizes: doesntApply("an empty list is as tall as its line and its padding; a loading one as its loading rows. Its width is its place's."),
  states: {
    shown: ["empty", "loading"],
    absent: {
      rest: "an empty list is always its line: with rows it is a List.",
      lit: "it takes neither the pointer nor the keyboard: there is nothing in it to light.",
      pressed: "nothing in it takes a press.",
      disabled: "a list is never disabled as a whole.",
      selected: "there is nothing to pick.",
      error: "a list that failed to load is an ErrorBlock in its place, with what broke and what to do.",
    },
    rows: nonEmpty(MATRIX.map(({ name, words }) => ({ name, cell: (state: MatrixState) => (state === "empty" ? empty(shown(words)) : loading()) }))),
    described: {
      empty: "The well with one centred line, `body` in `on_surface_variant`. By day the same roles in daylight colours.",
      loading: "Loading rows in the well: skeletons in a tiled two-line row's place, breathing as a Skeleton does, taking no press.",
    },
  },
  focus: doesntApply("an empty list takes no keyboard: there is nothing in it to move through, so Tab passes it."),
  content: {
    text: [
      { name: "Bluetooth", words: LINE, sample: (words: string) => empty(words) },
      { name: "Wi-Fi", words: NETWORKS, sample: (words: string) => empty(words) },
    ],
    missingPicture: doesntApply("it draws no symbol: its line says it all, in the room a row would have."),
    counts: [
      { name: "None", for: "The line.", sample: () => empty(shown(NETWORKS)) },
      { name: "One loading row", for: "A scan expecting little: a single place kept.", sample: () => loading(1) },
      { name: "Several loading rows", for: "Three, each a different share.", sample: () => loading() },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface; a list whose last row leaves gives that row's room back as LiveList's do, then shows the line at once."),
    leaves: doesntApply("rows found take the loading rows' places at once, nothing moving; the line goes as the first row arrives."),
    between: doesntApply("it has no states to move between; the loading rows breathe as the Skeleton page shows."),
  },
  keyboard: doesntApply("it takes no keyboard: Tab passes it."),
  measures: {
    samples: nonEmpty([
      { name: "Nothing found", for: "The well's padding, the line's room above and below, its line height.", sample: () => empty(shown(LINE)) },
      { name: "Still being found", for: "A loading row's tile, lines and their shares.", sample: () => loading(1) },
    ]),
    measures: [
      { what: "The well's padding.", token: "space-s8" },
      { what: "The line's room above and below inside it.", token: "space-s16" },
      { what: "The line, `body`'s.", token: "type-body-line" },
      { what: "A loading row: a two-line row's height.", token: "size-row_two_line" },
      { what: "A loading row's tile.", token: "size-control_md" },
      { what: "The well's corners.", token: "radius-content" },
    ],
    reflow: {
      described: "It fills its place's width at every window size, its line centred in it, ending in an ellipsis where the well is narrower than it.",
      sample: () => within("100%", stack("s8", "start", emptyList(shown(LINE)), text("An empty list as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
