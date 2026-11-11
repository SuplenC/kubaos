// The control center's Wi-Fi tab (`WifiList.qml`, `RadioTab.qml`, `ConnectForm.qml`): saved
// networks in range (Known), the one in use first and checked, then the rest (Nearby) by signal; a
// nearby one opens the connection form in the list's place, which asks a secured one's password.

import type { Words } from "../../lib/content.ts";
import type { ListRowSpec } from "../../lib/list.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { alone, connectForm, type Form, radioWell, swapping, WIFI_FORM, WIFI_KNOWN, WIFI_NEARBY, wellOf, wifiOff, wifiRows } from "./control-center-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "loading" | "empty" | "error";

const list = (rows = wifiRows(), current?: number, pinned?: "lit" | "pressed") => alone(radioWell(rows, { ...(current !== undefined ? { current, focused: true } : {}), ...(pinned ? { pinned } : {}) }));
const form = (patch: Partial<Form> = {}) => alone(wellOf(connectForm({ ...WIFI_FORM, ...patch })));

const OPEN: Form = { icon: "network_wifi_2_bar", title: "7SPOT", subtitle: "Open network", action: "Connect", focus: "action" };

/** The stage: a nearby network pressed opens the form; Cancel or Esc goes back. */
function live(): HTMLElement {
  const room = alone(radioWell(wifiRows()));
  const show = (element: HTMLElement) => room.replaceChildren(element);
  const back = () => {
    const made = radioWell(wifiRows());
    wire(made);
    show(made);
  };
  const wire = (made: HTMLElement) => {
    made.querySelectorAll<HTMLElement>(".k-list-row").forEach((row, index) => {
      if (index < WIFI_KNOWN.length) return;
      row.addEventListener("click", () => {
        const opened = wellOf(connectForm(index === WIFI_KNOWN.length + 1 ? WIFI_FORM : OPEN));
        opened.querySelector<HTMLElement>(".k-button")?.addEventListener("click", back);
        opened.addEventListener("keydown", (event) => event.key === "Escape" && back());
        show(opened);
      });
    });
  };
  wire(room.firstElementChild as HTMLElement);
  return room;
}

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return list();
    case "lit":
    case "pressed":
      return list(wifiRows(), 4, state);
    case "selected":
      return list(wifiRows(), 1);
    case "loading":
      return list(wifiRows("scanning"));
    case "empty":
      return alone(wellOf(wifiOff()));
    case "error":
      return form({ value: "hunter22", status: "Wrong password", failed: true, ready: true });
  }
}

const NAME: Words = { short: "7SPOT", long: "Kabukicho Free Wi-Fi 5GHz Extender, Second Floor" };

const many = (count: number): ListRowSpec[] => Array.from({ length: count }, (_, index) => ({ ...WIFI_NEARBY[index % WIFI_NEARBY.length]!, title: `${WIFI_NEARBY[index % WIFI_NEARBY.length]!.title} ${index + 1}` }));

const SWAP_IN: Move = {
  name: "The form takes the list's place",
  what: "A nearby network pressed: the list fades out as the form fades in, over `short`.",
  tracks: [
    { part: "the list", on: "the well", target: ".cc-list", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the form", on: "the well", target: ".cc-overlay", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: () => alone(swapping(wifiRows(), WIFI_FORM, false)),
};

export default page<Shown>({
  level: "shell",
  slug: "control-center-wifi",
  title: "Wi-Fi tab",
  lede: "Networks in range, Known then Nearby by signal, and the connection form that takes the list's place.",
  code: { qml: "WifiList" },
  stage: { live, hint: "press a Nearby network; Cancel goes back", daylight: true },
  variants: [
    { name: "Known and Nearby", for: "the one in use first, checked", sample: () => list() },
    { name: "Secured, asking", for: "the password field, Connect off until 8", sample: () => form() },
    { name: "Connecting", for: "the status says so; Connect off", sample: () => form({ value: "hunter2hunter2", status: "Connecting…", ready: false, focus: "cancel" }) },
    { name: "Open network", for: "nothing to ask: Connect", sample: () => alone(wellOf(connectForm(OPEN))) },
    { name: "Saved password failed", for: "the form opens with why", sample: () => form({ title: "Golden Gai", icon: "network_wifi_3_bar", status: "The saved password didn't work", failed: true }) },
    { name: "Out of range", for: "the network went: the status says so", sample: () => form({ status: "Out of range", failed: true, ready: false, subtitle: "", subtitleIcon: "", icon: "signal_wifi_0_bar" }) },
  ],
  sizes: doesntApply("the well takes the room under the tabs."),
  states: {
    shown: ["rest", "lit", "pressed", "selected", "loading", "empty", "error"],
    absent: {
      disabled: "a row is never off; the form's Connect is",
    },
    rows: [{ name: "The tab", cell }],
    looks: {
      rest: "rows on tiles; Known's in use checked, its tile `primary`",
      lit: "the well's tube; the row filled `surface_container_high`",
      pressed: "the row's fill presses",
      selected: "the keyboard on the in-use row",
      loading: "Nearby's three skeletons while a scan finds nothing yet",
      empty: "Wi-Fi off: `EmptyState`, the tile turns it on",
      error: "the field `Invalid`, the status in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "The well", tone: "accent", filled: false, content: false },
      { name: "The password field", tone: "accent", filled: false, content: false },
      { name: "Connect", tone: "accent", filled: true, content: false },
    ],
    looks: { lit: "the well's tube round it; the field's tube", signsOff: "each tube as `primary_glass`, no glow" },
    signsOff: () => list(wifiRows(), 4, "lit"),
  },
  content: {
    text: [
      { name: "A network's name", words: NAME, sample: (words: string) => list(wifiRows("rest", [{ ...WIFI_NEARBY[0]!, title: words }, ...WIFI_NEARBY.slice(1)])) },
      { name: "The form's name", words: NAME, sample: (words: string) => form({ title: words }) },
    ],
    missingPicture: doesntApply("a network's symbol is its signal's bars."),
    counts: [
      { name: "None in range", for: "Nearby says so", sample: () => list(wifiRows("none", [], [])) },
      { name: "Only Known", for: "No other networks in range", sample: () => list(wifiRows("none", [])) },
      { name: "Many", for: "the well scrolls; its thumb shows", sample: () => list(wifiRows("rest", many(12))) },
      { name: "No adapter", for: "nothing to connect with", sample: () => alone(wellOf(wifiOff(false))) },
    ],
  },
  motion: {
    arrives: [SWAP_IN],
    leaves: doesntApply("Cancel, Esc or connecting plays the same cross-fade back."),
    between: [
      { name: "The well lights up", what: "The keyboard into the list: its tube fades in.", tracks: lights(".k-list > .k-ring", "the well", 0, 1), reduced: "same", back: "plays", sample: () => list() },
      { name: "A row pressed", what: "A lit row pressed: its fill presses.", tracks: press(".k-list-row[data-current]", "a nearby row"), reduced: "same", back: "plays", sample: () => list(wifiRows(), 4, "lit") },
      ...powerMoves(() => list(wifiRows(), 4, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["j", "k", "↑", "↓"], does: "through the networks" },
      { keys: ["Enter"], does: "Known: connects; Nearby: opens the form" },
      { keys: ["Delete"], does: "forgets a saved network, as its × does" },
      { keys: ["Tab", "Shift+Tab"], does: "in the form: field, Cancel, Connect" },
      { keys: ["Enter"], does: "in the form: Connect" },
      { keys: ["Esc"], does: "in the form: back to the list" },
    ],
    order: "The tabs, then the well; in the form, the field, Cancel, Connect.",
    lands: "The form opens on its field, or Connect; closing it, back on the list.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The list", for: "labels, rows and the well's padding", sample: () => list() },
      { name: "The form", for: "the tile, the gaps, the buttons", sample: () => form() },
    ]),
    measures: [
      { part: "Well's padding", token: "space-s12" },
      { part: "A network's row", token: "size-row_two_line" },
      { part: "Between rows", token: "space-s4" },
      { part: "Label clear of rows above", token: "space-s12" },
      { part: "Label's line", token: "type-label-line" },
      { part: "A note's row", token: "size-row" },
      { part: "Form's tile", token: "size-control_lg" },
      { part: "Tile to the name", token: "space-s12" },
      { part: "Name to its line", token: "space-s4" },
      { part: "To the field, to the buttons", token: "space-s16" },
      { part: "Field to the status", token: "space-s8" },
      { part: "Status line, kept when empty", token: "type-small-line" },
    ],
    reflow: doesntApply("the well is as tall as the room the sections leave."),
  },
});
