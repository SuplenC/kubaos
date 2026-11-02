// The LiveList page, with LiveRow: `docs/design/components/list.md`. A list a scan updates while it
// shows, which never jumps: rows kept by their key arrive, leave and glide to their places, and the
// row under the pointer or the keyboard stays where it is on screen.

import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { type LiveRowSpec, liveList } from "../../lib/live-list.ts";
import { text, wrapped } from "../../lib/text.ts";
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
  live.onWaiting = (waiting) => say(waiting ? "An update waits: the row under the pointer or the keyboard would move. It plays as they let go." : "");
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
  return stack("s12", "start", within(WELL_ROOM, live.element), row("s8", "centre", ...buttons), within(WELL_ROOM, status));
}

/** Rows of a live list drawn still, for a player to move. */
function still(rows: readonly LiveRowSpec[] = FOUND.slice(0, 3), height = THREE): HTMLElement {
  return within(WELL_ROOM, liveList(rows, { height }).element);
}

const NAME: Words = { short: "Kissa", long: "Orange_Swiatlowod_4F2A_5GHz_Extender_Upstairs" };

const ROOM = ".k-live-row:nth-child(2)";

const ARRIVES: Move = {
  name: "A row arrives",
  what: "A scan finds a network: its room opens from nothing over `medium` on `standard_decel`, the rows below moving down with it, and its row fades in over `slow` on the same curve, cut by its room while it opens. With reduced motion its room is there at once and it fades in over `short`.",
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
  what: "A network goes: its row fades and its room closes over `medium` on `standard_accel`, giving back its gap with it, the rows below moving up. It takes no press while it goes, so a click can't land on the row sliding under the pointer. With reduced motion it fades over `short`, then its room goes at once.",
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
  what: "A scan re-sorts by signal: each row that changes place glides from where it was to where it goes over `medium` on `standard_decel`, as a list the user reorders does; the rest stay as they are. With reduced motion they are in their new places at once.",
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
  what: "A network arrives above the row under the pointer, in a list scrolled down: as its room opens over `medium` on `standard_decel`, the list scrolls by the same amount on the same curve, so the row under the pointer, lit here, stays exactly where it was and the new row opens out of sight above it. Where the list can't scroll that far, or the held row itself would change place, the update waits until the pointer leaves and the keyboard does.",
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
  summary: "A list a scan updates while it shows, which never jumps: rows arrive, leave and glide, and the row being looked at stays still.",
  live: scanning,
  description: [
    "QML `LiveList`, its delegates `LiveRow`s; iced has none yet. Play a scan with the buttons. Rest the pointer on a row, or Tab in and pick one with the keys, and play them again: that row doesn't move. The control center's Wi-Fi and Bluetooth tabs.",
    "A ScrollingList whose rows (each with a `key`, a network's name, a device's address) are kept in step by key, with the fewest removes, inserts and moves, so only what changed changes: a row whose facts change (its signal) is redrawn in place, and the rest stay the same rows.",
    "A row that arrives opens its room from nothing over `medium` on `standard_decel` and fades in over `slow`; one that leaves fades and gives its room back, its `space.s4` gap with it, over `medium` on `standard_accel`, taking no press while it goes; one that changes place glides there over `medium` on `standard_decel`. Each row's room is the row and the gap after it, so the list's padding under its last row is the well's `space.s8` less that `space.s4`. Only while the list shows: an update nobody sees is in place when it next shows.",
    "**It never jumps.** The row under the pointer, or the one the keyboard is on while the list has it, is held still. An update that would move it, a row arriving or leaving above it, plays with the list scrolling by the same amount as their rooms open and close, so the held row stays where it is on screen. Where the list can't scroll that far (it fits its well, or is at its top) or the held row would itself change place in a re-sort, the update waits, and plays once the pointer leaves and the keyboard does. Nothing else scrolls it: it moves only when the person scrolls it or moves with the keys.",
    "With reduced motion a row arrives in its room and fades in over `short`, leaves by fading over `short` and giving its room back at once, and changes place at once. The held row's rule is the same.",
  ],
  variants: [
    { name: "A scan's networks", for: "The Wi-Fi tab: the network you're on first, then by signal.", sample: () => still(FOUND, FIVE) },
    { name: "The keyboard in it", for: "The list with the keyboard on a row: that row is held still while the scan goes on.", sample: () => cell(FOUND, "lit") },
  ],
  sizes: doesntApply("a live list is a ScrollingList, as tall as its place makes it; its rows are a List's."),
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "the well takes no press; its rows press as the List page shows, and a leaving row takes none.",
      disabled: "a live list is never disabled as a whole; a row that can't be taken is.",
      selected: "a picked row is its tick, as the List page shows: the network you're on.",
      loading: "a scan with nothing found yet shows loading rows in the well (the EmptyList page); a row found takes one's place.",
      empty: "a scan that found nothing is an EmptyList.",
      error: "a scan that failed is an ErrorBlock in the list's place.",
    },
    rows: nonEmpty(MATRIX.map(({ name, rows }) => ({ name, cell: (state: MatrixState) => cell(rows, state) }))),
    described: {
      rest: "Its rows settled; the well unlit. By day the same roles in daylight colours.",
      lit: "The list has the keyboard: the well's tube lit, the row the keyboard is on lit and held still while updates play round it.",
    },
  },
  focus: {
    tubes: [{ name: "The well", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the well's edge, the glow outside it, while the list has the keyboard.",
      signsOff: "By day always, and at night while a power change has the lights out: the tube is solid glass, `primary_glass`, with no glow.",
    },
    signsOff: (index: number) => cell(MATRIX[index]!.rows, "signs-off"),
  },
  content: {
    text: [{ name: "A network's name", words: NAME, sample: (words: string) => still([FOUND[0]!, { ...FOUND[1]!, title: words }, FOUND[2]!]) }],
    missingPicture: doesntApply("its rows' symbols are the List page's."),
    counts: [
      { name: "One", for: "Only the network you're on.", sample: () => still(FOUND.slice(0, 1)) },
      { name: "A few", for: "Fewer than the well shows: nothing scrolls, so a row arriving above the held one waits.", sample: () => still(FOUND.slice(0, 3)) },
      { name: "Many", for: "More than it shows: it scrolls, and a row arriving above the held one plays as the list follows.", sample: () => still(FOUND, FIVE) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: [
      MOVES,
      HOLDS,
      { name: "The list takes the keyboard", what: "Tab onto it: the well's tube and glow fade in over `short` on `standard`.", tracks: lights(".k-list > .k-ring", "the networks", 0, 1), reduced: "same", back: "plays", sample: () => still() },
      ...powerMoves(() => cell(FOUND.slice(0, 3), "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "↑", "j", "k"], does: "The next or the row before, round from either end; the row it is on stays on that network as rows come and go round it." },
      { keys: ["Page Down", "Page Up", "Home", "End"], does: "As a List: by a page, or to the ends." },
      { keys: ["Enter"], does: "Takes the row, once per press; a row leaving takes none." },
    ],
    order: "One stop for Tab, as a List. A row that leaves while the keyboard is on it gives the keyboard to the nearest row after it.",
    lands: "Tabbing in, on the network you're on, else the row it was last on, else its first.",
  },
  measures: {
    samples: nonEmpty([{ name: "A scan's networks", for: "The rows' rooms, each the row and the gap after it, and the padding under the last.", sample: () => still(FOUND, FIVE) }]),
    measures: [
      { what: "A row's room: the row and the gap after it.", token: "size-row_two_line" },
      { what: "The gap each row's room carries.", token: "space-s4" },
      { what: "The padding above the first row; under the last, it less the last room's gap.", token: "space-s8" },
      { what: "The well's corners.", token: "radius-content" },
    ],
    reflow: {
      described: "As a ScrollingList: it fills its place, showing more rows in a taller one.",
      sample: () => within("100%", stack("s8", "start", liveList(FOUND, { height: THREE }).element, wrapped("A live list as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
