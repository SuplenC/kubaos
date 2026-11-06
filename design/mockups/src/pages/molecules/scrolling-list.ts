// The ScrollingList page: `docs/design/components/list.md`. A list whose rows scroll inside its
// well, cut by the well's own rounded shape as a select's open list is cut by its surface, the
// well's padding the scroll's padding, and its thumb floating in that padding at its end.

import type { Words } from "../../lib/content.ts";
import { within } from "../../lib/layout.ts";
import { type ListRowSpec, type ScrollingOptions, scrollingList } from "../../lib/list.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { WELL_ROOM } from "./list.ts";

/** Five two-line rows and their gaps, inside the well's padding: the control center's Wi-Fi tab. */
const FIVE = "calc(5 * var(--size-row_two_line) + 4 * var(--space-s4) + 2 * var(--space-s8))";

/** Three, for a well in a short room. */
const THREE = "calc(3 * var(--size-row_two_line) + 2 * var(--space-s4) + 2 * var(--space-s8))";

const NAMES = ["Hoshi 5G", "Kissa", "Station Free Wi-Fi", "Pierogarnia", "DIRECT-7Q-Printer", "Neko Guest", "Library", "Tram 12", "Kawa i Herbata", "Orange_Swiatlowod_4F2A", "Mieszkanie 3", "eduroam"];
const BARS = ["wifi", "network_wifi_3_bar", "network_wifi_3_bar", "network_wifi_2_bar", "network_wifi_2_bar", "network_wifi_2_bar", "network_wifi_1_bar", "network_wifi_1_bar", "network_wifi_1_bar", "network_wifi_1_bar", "network_wifi_1_bar", "network_wifi_1_bar"];

/** Twelve networks, more than the well shows: the first the one you're on. */
const NETWORKS: readonly ListRowSpec[] = NAMES.map((title, index) => ({
  title,
  subtitle: index === 0 ? "Connected · WPA2" : index % 3 === 2 ? "Open" : "Secured · WPA2",
  ...(index % 3 === 2 ? {} : { subtitleIcon: "lock" }),
  icon: BARS[index]!,
  checked: index === 0,
}));

const NAME: Words = { short: "Kissa", long: "Orange_Swiatlowod_4F2A_5GHz_Extender_Upstairs" };

function scrolling(options: Partial<ScrollingOptions> = {}, rows: readonly ListRowSpec[] = NETWORKS): HTMLElement {
  return within(WELL_ROOM, scrollingList(rows, { height: FIVE, ...options }));
}

const MATRIX = [
  { name: "At its top", down: 0 },
  { name: "Scrolled, rows cut by its corners", down: 0.37 },
  { name: "At its end", down: 1 },
] as const;

type MatrixState = "rest" | "lit";

function cell(down: number, state: MatrixState | "signs-off"): HTMLElement {
  if (state === "rest") return scrolling({ down });
  const room = scrolling({ down, current: down === 1 ? NETWORKS.length - 1 : down === 0 ? 0 : 4, pinned: "lit", focused: true });
  if (state === "signs-off") room.querySelector<HTMLElement>(".k-list")!.dataset.pinned = "signs-off";
  return room;
}

export default page({
  level: "molecules",
  slug: "scrolling-list",
  title: "ScrollingList",
  lede: "A list whose rows scroll inside its well, cut by its own rounded shape.",
  code: { rust: "list::scrolling_list", qml: "ScrollingList", unbuilt: "thumb rail: not built yet" },
  stage: { live: () => within("var(--size-column)", scrollingList(NETWORKS, { height: FIVE })), hint: "scroll, drag the thumb, or Tab and ↑ ↓", daylight: true },
  variants: [
    { name: "At its top", for: "the first row the well's padding under its edge", sample: () => scrolling() },
    { name: "Scrolled", for: "rows cut round by the well's corners", sample: () => scrolling({ down: 0.37 }) },
    { name: "With the keyboard", for: "the tube lit, the keyboard's row in view", sample: () => scrolling({ down: 0.5, current: 6, focused: true }) },
    { name: "The thumb under the pointer", for: "the thumb in `outline`, as while dragged", sample: () => scrolling({ down: 0.5, thumbPinned: true }) },
  ],
  sizes: [
    { name: "Its place's height", for: "three rows here; the live one is five", sample: () => scrolling({ height: THREE }) },
  ],
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "its rows press: the List page",
      disabled: "a row can be, never the well",
      selected: "its rows are: the List page",
      loading: "loading rows in the well: the List page",
      empty: "an `EmptyList`, which doesn't scroll",
      error: "an `ErrorBlock` in its place",
    },
    rows: nonEmpty(MATRIX.map(({ name, down }) => ({ name, cell: (state: MatrixState) => cell(down, state) }))),
    looks: {
      rest: "rows cut by the rounded edge, thumb `outline_variant`",
      lit: "the well's tube over the rows, glow at night, row in view",
    },
  },
  focus: {
    tubes: [{ name: "The well", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the tube inside the well's edge, over the rows; the glow outside",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => cell(MATRIX[index]!.down, "signs-off"),
  },
  content: {
    text: [{ name: "A network's name", words: NAME, sample: (words: string) => scrolling({ height: THREE }, [{ ...NETWORKS[1]!, title: words }, ...NETWORKS.slice(2)]) }],
    missingPicture: doesntApply("the well draws no picture; its rows' are on the List page."),
    counts: [
      { name: "Fewer than it shows", for: "no thumb, nothing scrolls, the well empty below", sample: () => scrolling({}, NETWORKS.slice(0, 2)) },
      { name: "Just more", for: "six rows: a long thumb", sample: () => scrolling({}, NETWORKS.slice(0, 6)) },
      { name: "Many", for: "twelve: a short thumb", sample: () => scrolling() },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface."),
    leaves: doesntApply("it goes with its surface."),
    between: [
      {
        name: "The thumb lights",
        what: "The pointer onto the thumb: `outline_variant` to `outline` over `micro` on `standard`; leaving plays it back.",
        tracks: [{ part: "the thumb", on: "a scrolled list", target: ".k-list-thumb", property: "--list-thumb-lit", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => scrolling({ down: 0.5 }),
      },
      { name: "The list takes the keyboard", what: "Tab onto it: the tube and glow fade in over `short`, over the rows.", tracks: lights(".k-list > .k-ring", "a scrolled list", 0, 1), reduced: "same", back: "plays", sample: () => scrolling({ down: 0.37 }) },
      ...powerMoves(() => scrolling({ down: 0.37, focused: true, current: 4 })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "↑", "j", "k"], does: "next or previous row, into view, padding clear" },
      { keys: ["Page Down", "Page Up"], does: "as many rows as show whole, held at the ends" },
      { keys: ["Home", "End"], does: "the first or last row" },
      { keys: ["Enter"], does: "takes the row, once per press" },
    ],
    order: "One stop for Tab: the keys move inside it.",
    lands: "On its picked row, else the row it was last on, else its first.",
  },
  measures: {
    samples: nonEmpty([
      { name: "At its top", for: "well padding, row gap and height, thumb", sample: () => scrolling() },
      { name: "Scrolled", for: "the thumb's rail clear of the corners", sample: () => scrolling({ down: 0.5 }) },
    ]),
    measures: [
      { part: "Well corners", token: "radius-content" },
      { part: "Scroll padding, thumb rail inset", token: "space-s8" },
      { part: "Row gap, thumb inset", token: "space-s4" },
      { part: "Thumb width", token: "size-scrollbar" },
      { part: "Row height", token: "size-row_two_line" },
      { part: "Hairline", token: "stroke-hairline_device_px" },
    ],
    reflow: {
      note: "It fills its place's width and height: a taller place shows more rows.",
      sample: () => within("100%", scrollingList(NETWORKS, { height: THREE })),
    },
  },
});
