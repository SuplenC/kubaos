// The LiveList page, with LiveRow: `docs/design/components/list.md`. A list a scan updates while it
// shows, which never jumps: rows kept by their key arrive, leave and glide to their places, and the
// row under the pointer or the keyboard stays where it is on screen.

import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { type LiveRowSpec, liveList } from "../../lib/live-list.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { WELL_ROOM } from "./list.ts";

/** Five two-line rows and their gaps, inside the well's padding. */
const FIVE = "calc(5 * var(--size-row_two_line) + 4 * var(--space-s4) + 2 * var(--space-s8))";
const THREE = "calc(3 * var(--size-row_two_line) + 2 * var(--space-s4) + 2 * var(--space-s8))";

/** A network as a scan finds it, its signal from 0 to 4 bars. */
function network(title: string, bars: number, secured = true, connected = false): LiveRowSpec {
  const glyph = bars >= 4 ? "wifi" : `network_wifi_${Math.max(1, bars)}_bar`;
  const subtitle = connected ? "Connected · WPA2" : secured ? "Secured · WPA2" : "Open";
  return { key: title, title, subtitle, ...(secured ? { subtitleIcon: "lock" } : {}), icon: glyph, checked: connected };
}

const FOUND: readonly LiveRowSpec[] = [
  network("Hoshi 5G", 4, true, true),
  network("Kissa", 3),
  network("Pierogarnia", 3),
  network("Station Free Wi-Fi", 2, false),
  network("Neko Guest", 2),
  network("Library", 1, false),
  network("Tram 12", 1),
];

/** Networks a later scan may find. */
const NEARBY = ["DIRECT-7Q-Printer", "Kawa i Herbata", "Mieszkanie 3", "eduroam", "Orange_Swiatlowod_4F2A"];

/** The connected network first, then by signal, strongest first. */
function sorted(rows: readonly LiveRowSpec[]): LiveRowSpec[] {
  const bars = (each: LiveRowSpec) => (each.icon === "wifi" ? 4 : Number(each.icon?.match(/\d/)?.[0] ?? 0));
  return [...rows].sort((a, b) => Number(b.checked) - Number(a.checked) || bars(b) - bars(a));
}

/** The Wi-Fi tab's list, with buttons that play what a scan finds. */
function scanning(): HTMLElement {
  let rows = [...FOUND];
  let spare = [...NEARBY];
  let turn = 0;
  const live = liveList(rows, { height: FIVE });
  const status = text("", "small", "on_surface_variant");
  const say = (words: string) => {
    status.textContent = words;
  };
  live.onWaiting = (waiting) => say(waiting ? "An update waits until you let go." : "");
  const set = (next: LiveRowSpec[]) => {
    rows = next;
    live.update(rows);
  };
  const arrive = () => {
    const name = spare.shift();
    if (!name) return;
    turn += 1;
    set(sorted([...rows, network(name, 1 + (turn % 4))]));
  };
  const leave = () => {
    const gone = rows.filter((each) => !each.checked);
    const last = gone[(turn += 1) % gone.length];
    if (!last) return;
    spare.push(last.title);
    set(rows.filter((each) => each !== last));
  };
  const resort = () => {
    turn += 1;
    set(sorted(rows.map((each, index) => (each.checked ? each : network(each.title, 1 + ((index + turn) % 4), each.subtitleIcon !== undefined)))));
  };
  const reset = () => {
    spare = [...NEARBY];
    set([...FOUND]);
  };
  const actions: [string, () => void][] = [
    ["A network arrives", arrive],
    ["One leaves", leave],
    ["Signals change", resort],
    ["Start again", reset],
  ];
  const buttons = actions.map(([label, does], index) => {
    const each = button({ label, kind: index === 0 ? "Tonal" : "Text", size: "Sm" });
    each.addEventListener("click", does);
    return each;
  });
  return row("s24", "start", within(WELL_ROOM, live.element), stack("s8", "start", ...buttons, status));
}

/** Rows of a live list drawn still, for a player to move. */
function still(rows: readonly LiveRowSpec[] = FOUND.slice(0, 3), height = THREE): HTMLElement {
  return within(WELL_ROOM, liveList(rows, { height }).element);
}

const NAME: Words = { short: "Kissa", long: "Orange_Swiatlowod_4F2A_5GHz_Extender_Upstairs" };

const ROOM = ".k-live-row:nth-child(2)";

const ARRIVES: Move = {
  name: "A row arrives",
  what: "Its room opens over `medium` on `standard_decel`, the rows below moving down, as it fades in over `slow`.",
  tracks: [
    { part: "its room", on: "the second row", target: ROOM, property: "--grow", from: 0, to: 1, duration: "medium", curve: "standard_decel" },
    { part: "its row", on: "the second row", target: ROOM, property: "opacity", from: 0, to: 1, duration: "slow", curve: "standard_decel" },
  ],
  reduced: [
    { part: "its room", on: "the second row", target: ROOM, property: "--grow", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "its row", on: "the second row", target: ROOM, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => still(),
};

const LEAVES: Move = {
  name: "A row leaves",
  what: "It fades and its room closes over `medium` on `standard_accel`; it takes no press while it goes.",
  tracks: [
    { part: "its room", on: "the second row", target: ROOM, property: "--grow", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
    { part: "its row", on: "the second row", target: ROOM, property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
  ],
  reduced: [
    { part: "its row", on: "the second row", target: ROOM, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "its room", on: "the second row", target: ROOM, property: "--grow", from: 1, to: 0, duration: "instant", cue: "short", curve: "linear" },
  ],
  back: "cuts",
  sample: () => still(),
};

/** Two rows that have changed places, each drawn from where it was. */
function swapped(): HTMLElement {
  const box = still([FOUND[0]!, FOUND[2]!, FOUND[1]!]);
  const step = "calc(var(--size-row_two_line) + var(--space-s4))";
  box.querySelector<HTMLElement>(".k-live-row:nth-child(2)")?.style.setProperty("--from-y", step);
  box.querySelector<HTMLElement>(".k-live-row:nth-child(3)")?.style.setProperty("--from-y", `calc(-1 * ${step})`);
  return box;
}

const MOVES: Move = {
  name: "Rows change places",
  what: "Each row that moved glides there over `medium` on `standard_decel`; the rest stay.",
  tracks: [
    { part: "the row going down", on: "two rows", target: ".k-live-row:nth-child(3)", property: "--glide", from: 1, to: 0, duration: "medium", curve: "standard_decel" },
    { part: "the row going up", on: "two rows", target: ".k-live-row:nth-child(2)", property: "--glide", from: 1, to: 0, duration: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "the row going down", on: "two rows", target: ".k-live-row:nth-child(3)", property: "--glide", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the row going up", on: "two rows", target: ".k-live-row:nth-child(2)", property: "--glide", from: 1, to: 0, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: swapped,
};

/** A scrolled list whose second row arrives above the row held under the pointer. */
function heldSample(): HTMLElement {
  const box = still(FOUND, THREE);
  const pane = box.querySelector<HTMLElement>(".k-list-pane")!;
  const column = box.querySelector<HTMLElement>(".k-live-rows")!;
  column.dataset.held = "";
  box.querySelector(".k-live-row:nth-child(4) > .k-list-row")?.setAttribute("data-current", "");
  requestAnimationFrame(() => {
    pane.scrollTop = Math.round(column.querySelector<HTMLElement>(".k-live-row:nth-child(3)")!.offsetTop);
  });
  return box;
}

const HOLDS: Move = {
  name: "The held row stays",
  what: "A row arrives above the held one: the list scrolls by as much, or the update waits for you to let go.",
  tracks: [
    { part: "the arriving room", on: "a scrolled list", target: ROOM, property: "--grow", from: 0, to: 1, duration: "medium", curve: "standard_decel" },
    { part: "the arriving row", on: "a scrolled list", target: ROOM, property: "opacity", from: 0, to: 1, duration: "slow", curve: "standard_decel" },
    { part: "the scroll, following", on: "a scrolled list", target: ".k-live-rows", property: "--scrolled", from: 0, to: 1, duration: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "the arriving room", on: "a scrolled list", target: ROOM, property: "--grow", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the arriving row", on: "a scrolled list", target: ROOM, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the scroll, following", on: "a scrolled list", target: ".k-live-rows", property: "--scrolled", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "cuts",
  sample: heldSample,
};

type MatrixState = "rest" | "lit";

const MATRIX = [
  { name: "Settled", rows: FOUND },
  { name: "Fewer than it shows", rows: FOUND.slice(0, 3) },
] as const;

function cell(rows: readonly LiveRowSpec[], state: MatrixState | "signs-off"): HTMLElement {
  const box = within(WELL_ROOM, liveList(rows, state === "rest" ? { height: FIVE } : { height: FIVE, current: 1, pinned: "lit", focused: true }).element);
  if (state === "signs-off") box.querySelector<HTMLElement>(".k-list")!.dataset.pinned = "signs-off";
  return box;
}

export default page({
  level: "molecules",
  slug: "live-list",
  title: "LiveList and LiveRow",
  lede: "A list a scan updates while it shows, which never jumps: rows arrive, leave and glide, and the row looked at stays still.",
  code: { qml: "LiveList, LiveRow", unbuilt: "Rust: not built yet" },
  stage: {
    live: scanning,
    hint: "press a button; rest on a row and it stays",
    daylight: true,
  },
  variants: [
    { name: "A scan's networks", for: "the network you're on first, then by signal", sample: () => still(FOUND, FIVE) },
    { name: "The keyboard in it", for: "that row held still as the scan goes on", sample: () => cell(FOUND, "lit") },
  ],
  sizes: doesntApply("a `ScrollingList`, as tall as its place; its rows are a `List`'s."),
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "its rows press; a leaving row takes none",
      disabled: "never as a whole; a row can be",
      selected: "a picked row is its tick",
      loading: "loading rows in the well: `EmptyList`",
      empty: "a scan that found nothing: `EmptyList`",
      error: "a failed scan: `ErrorBlock`",
    },
    rows: nonEmpty(MATRIX.map(({ name, rows }) => ({ name, cell: (state: MatrixState) => cell(rows, state) }))),
    looks: {
      rest: "rows settled, the well unlit",
      lit: "the well's tube lit, the keyboard's row lit and held",
    },
  },
  focus: {
    tubes: [{ name: "The well", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the tube inside the well's edge, the glow outside",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => cell(MATRIX[index]!.rows, "signs-off"),
  },
  content: {
    text: [{ name: "A network's name", words: NAME, sample: (words: string) => still([FOUND[0]!, { ...FOUND[1]!, title: words }, FOUND[2]!]) }],
    missingPicture: doesntApply("its rows' symbols are the `List` page's."),
    counts: [
      { name: "One", for: "only the network you're on", sample: () => still(FOUND.slice(0, 1)) },
      { name: "A few", for: "nothing scrolls: an update above the held row waits", sample: () => still(FOUND.slice(0, 3)) },
      { name: "Many", for: "it scrolls: an update plays as the list follows", sample: () => still(FOUND, FIVE) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: [
      MOVES,
      HOLDS,
      { name: "The list takes the keyboard", what: "The well's tube and glow fade in over `short` on `standard`.", tracks: lights(".k-list > .k-ring", "the networks", 0, 1), reduced: "same", back: "plays", sample: () => still() },
      ...powerMoves(() => cell(FOUND.slice(0, 3), "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "↑", "j", "k"], does: "next or before; it stays on that network" },
      { keys: ["Page Down", "Page Up", "Home", "End"], does: "as a `List`" },
      { keys: ["Enter"], does: "takes the row; a leaving row takes none" },
    ],
    order: "One stop for Tab; a row leaving under the keyboard hands it to the nearest after it.",
    lands: "On the network you're on, else the row it was last on, else its first.",
  },
  measures: {
    samples: nonEmpty([{ name: "A scan's networks", for: "each row's room, the padding under the last", sample: () => still(FOUND, FIVE) }]),
    measures: [
      { part: "Row in its room", token: "size-row_two_line" },
      { part: "Gap in each room", token: "space-s4" },
      { part: "Well padding", token: "space-s8" },
      { part: "Well corner", token: "radius-content" },
    ],
    reflow: {
      note: "It fills its place, showing more rows in a taller one.",
      sample: () => within("100%", liveList(FOUND, { height: THREE }).element),
    },
  },
});
