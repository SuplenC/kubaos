// The ScrollingList page: `docs/design/components/list.md`. A list whose rows scroll inside its
// well, cut by the well's own rounded shape as a select's open list is cut by its surface, the
// well's padding the scroll's padding, and its thumb floating in that padding at its end.

import type { Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type ListRowSpec, type ScrollingOptions, scrollingList } from "../../lib/list.ts";
import { text } from "../../lib/text.ts";
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

const DESCRIBED =
  "A List whose rows are taller than its room: the well is as tall as its place makes it, and the rows scroll inside it, right up to its edge, cut by its own rounded shape (`radius.content`), not by a rectangle inside its padding, as a select's options are cut by the select's own surface. The `space.s8` above the first row and below the last is the scroll's padding: at rest they sit where a List puts them, and scrolled, a row passes the well's edge and its corners round it off. The well's hairline and its focus tube are drawn over the rows, so a row scrolled to the edge never covers them.";

const THUMB =
  "The thumb is `size.scrollbar` (4) wide, a pill in `outline_variant`, `outline` under the pointer and while it is dragged, floating `space.s4` in from the well's end across its padding, so it never covers a row. It runs along the rows' room, `space.s8` clear of the well's top and bottom, where the corners would cut it, its length the share of the rows the well shows. It shows only while the rows are taller than the well; dragging it scrolls them.";

export default page({
  level: "molecules",
  slug: "scrolling-list",
  title: "ScrollingList",
  summary: "A list whose rows scroll inside its well, cut by its own rounded shape, the well's padding the scroll's.",
  live: () => scrolling(),
  description: [
    "Rust `list::scrolling_list`, QML `ScrollingList` holding a `ListRows`. Scroll it with the wheel or a touchpad, drag its thumb, or Tab into it and use ↑ ↓, Page Up and Page Down. The control center's Wi-Fi tab.",
    DESCRIBED,
    THUMB,
    "The wheel scrolls a row and its gap (`size.row` and `space.s4`) a notch, at once, whatever the display's scale; a touchpad's glide follows the fingers pixel for pixel. At either end the wheel goes on to scroll what is around the well. The keys bring the row they reach into view with the well's padding clear of its edge, so a row the keyboard is on is never cut by the corners. Nothing else scrolls it: rows changing never carry the one being looked at away (LiveList).",
  ],
  variants: [
    { name: "At its top", for: "At rest: the first row the well's padding under its edge, the thumb at the top of its rail.", sample: () => scrolling() },
    { name: "Scrolled", for: "Rows passing the well's top and bottom edges, cut round by its corners.", sample: () => scrolling({ down: 0.37 }) },
    { name: "With the keyboard", for: "The well's tube lit, the row the keyboard is on in view, its padding clear.", sample: () => scrolling({ down: 0.5, current: 6, focused: true }) },
    { name: "The thumb under the pointer", for: "The thumb in `outline`, as while it is dragged.", sample: () => scrolling({ down: 0.5, thumbPinned: true }) },
  ],
  sizes: [
    { name: "Its place's height", for: "As tall as its place makes it: here three rows tall in a short room; the live one above is five.", sample: () => scrolling({ height: THREE }) },
  ],
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "the well takes no press of its own; its rows press, as the List page shows.",
      disabled: "a list is never disabled as a whole; a row that can't be taken is.",
      selected: "the well isn't picked; its rows are, as the List page shows.",
      loading: "rows still being found are loading rows in the well (the List and EmptyList pages).",
      empty: "a well with no rows is an EmptyList, which doesn't scroll.",
      error: "a list that failed to load is an ErrorBlock in its place.",
    },
    rows: nonEmpty(MATRIX.map(({ name, down }) => ({ name, cell: (state: MatrixState) => cell(down, state) }))),
    described: {
      rest: "The well unlit, its rows cut by its rounded edge where they pass it, the thumb in `outline_variant`. By day the same roles in daylight colours.",
      lit: "The list has the keyboard: the well's tube inside its edge, `primary` at `focus.tube`, the glow outside at night, drawn over the rows; the row the keyboard is on lit on the secondary container, in view, the padding clear of the edge.",
    },
  },
  focus: {
    tubes: [{ name: "The well", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the well's edge, the glow outside it, drawn over the rows that scroll under it.",
      signsOff: "By day always, and at night while a power change has the lights out: the tube is solid glass, `primary_glass`, with no glow.",
    },
    signsOff: (index: number) => cell(MATRIX[index]!.down, "signs-off"),
  },
  content: {
    text: [{ name: "A network's name", words: NAME, sample: (words: string) => scrolling({ height: THREE }, [{ ...NETWORKS[1]!, title: words }, ...NETWORKS.slice(2)]) }],
    missingPicture: doesntApply("the well draws no picture; its rows' are on the List page."),
    counts: [
      { name: "Fewer than it shows", for: "Two rows in a well five tall: no thumb, nothing scrolls, the rest of the well empty below them.", sample: () => scrolling({}, NETWORKS.slice(0, 2)) },
      { name: "Just more", for: "Six rows: a row and a half to scroll, a long thumb.", sample: () => scrolling({}, NETWORKS.slice(0, 6)) },
      { name: "Many", for: "Twelve: a short thumb.", sample: () => scrolling() },
    ],
  },
  motion: {
    arrives: doesntApply("a scrolling list comes with the surface it is on."),
    leaves: doesntApply("it goes with its surface."),
    between: [
      {
        name: "The thumb lights",
        what: "The pointer onto the thumb: it goes from `outline_variant` to `outline` over `micro` on `standard`, a colour change under the pointer; the pointer leaving plays it back.",
        tracks: [{ part: "the thumb", on: "a scrolled list", target: ".k-list-thumb", property: "--list-thumb-lit", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => scrolling({ down: 0.5 }),
      },
      { name: "The list takes the keyboard", what: "Tab onto it: the well's tube and glow fade in over `short` on `standard`, over the rows; Tab away plays it back.", tracks: lights(".k-list > .k-ring", "a scrolled list", 0, 1), reduced: "same", back: "plays", sample: () => scrolling({ down: 0.37 }) },
      ...powerMoves(() => scrolling({ down: 0.37, focused: true, current: 4 })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "↑", "j", "k"], does: "The next or the row before, round from either end, brought into view with the well's padding clear." },
      { keys: ["Page Down", "Page Up"], does: "As many rows on as show whole, held at the ends." },
      { keys: ["Home", "End"], does: "The first or the last row, scrolled to the top or the end." },
      { keys: ["Enter"], does: "Takes the row, once per press." },
    ],
    order: "One stop for Tab, as a List: the keys move inside it, and the view follows only them and the person scrolling.",
    lands: "Tabbing in, on its picked row, else the row it was last on, else its first, scrolled into view.",
  },
  measures: {
    samples: nonEmpty([
      { name: "At its top", for: "The well's padding, its rows' gap and height, the thumb's width and its inset.", sample: () => scrolling() },
      { name: "Scrolled", for: "The thumb's rail clear of the corners, rows cut by the edge.", sample: () => scrolling({ down: 0.5 }) },
    ]),
    measures: [
      { what: "The well's corners, which cut the rows.", token: "radius-content" },
      { what: "The scroll's padding above the first row and below the last; the thumb's rail clear of the top and bottom.", token: "space-s8" },
      { what: "Between rows; the thumb in from the well's end.", token: "space-s4" },
      { what: "The thumb's width.", token: "size-scrollbar" },
      { what: "A row, and so a notch of the wheel with its gap.", token: "size-row_two_line" },
      { what: "The well's hairline over the rows: one device pixel.", token: "stroke-hairline_device_px" },
    ],
    reflow: {
      described: "A scrolling list fills its place's width and height at every window size: in a taller place it shows more rows and its thumb grows; in a short one it shows fewer. Here as wide as the window.",
      sample: () => within("100%", stack("s8", "start", scrollingList(NETWORKS, { height: THREE }), text("A scrolling list as wide as the window, three rows tall.", "small", "on_surface_variant"))),
    },
  },
});
