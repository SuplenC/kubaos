// The control center's Bluetooth tab (`BluetoothList.qml`, `RadioTab.qml`, `ConnectForm.qml`):
// paired devices (Known), the connected first with their battery, then devices in range (Nearby),
// nearest first; the unnamed ones in a drawer under a line. A nearby device opens the pairing form,
// which shows what the agent asks: a code to type or check, or a PIN or passkey to type here.

import type { Words } from "../../lib/content.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { alone, BT_FORM, BT_NEARBY, btOff, btRows, connectForm, type Form, radioWell, wellOf } from "./control-center-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "open" | "loading" | "empty" | "error";

const list = (rows = btRows(), current?: number, pinned?: "lit" | "pressed", down = 0) => {
  const made = alone(radioWell(rows, { ...(current !== undefined ? { current, focused: true } : {}), ...(pinned ? { pinned } : {}) }));
  if (down > 0) requestAnimationFrame(() => made.querySelector<HTMLElement>(".k-list-pane")?.scrollTo({ top: 1e6 * down }));
  return made;
};
const form = (patch: Partial<Form> = {}) => alone(wellOf(connectForm({ ...BT_FORM, ...patch })));

/** The stage: Show and Hide open the drawer; a nearby device opens the form. */
function live(): HTMLElement {
  const room = alone(radioWell(btRows()));
  let open = false;
  const draw = () => {
    const made = radioWell(btRows("rest", { unnamed: open ? "open" : "closed" }));
    made.querySelector<HTMLElement>(".cc-line .k-button")?.addEventListener("click", () => {
      open = !open;
      draw();
    });
    made.querySelectorAll<HTMLElement>(".k-list-row").forEach((row) => {
      if (!BT_NEARBY.some((device) => row.textContent?.includes(device.title))) return;
      row.addEventListener("click", () => {
        const opened = wellOf(connectForm(BT_FORM));
        opened.querySelector<HTMLElement>(".k-button")?.addEventListener("click", draw);
        room.replaceChildren(opened);
      });
    });
    room.replaceChildren(made);
  };
  draw();
  return room;
}

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return list();
    case "lit":
    case "pressed":
      return list(btRows(), 7, state);
    case "selected":
      return list(btRows(), 1);
    case "open":
      return list(btRows("rest", { unnamed: "open" }), undefined, undefined, 1);
    case "loading":
      return list(btRows("scanning"));
    case "empty":
      return alone(wellOf(btOff()));
    case "error":
      return form({ status: "Authentication failed: the device said no", failed: true });
  }
}

const NAME: Words = { short: "Pixel 9", long: "Kuba's Sony WH-1000XM4 Wireless Noise Cancelling Headphones" };

export default page<Shown>({
  level: "shell",
  slug: "control-center-bluetooth",
  title: "Bluetooth tab",
  lede: "Devices Known then Nearby, the unnamed ones in a drawer, and the pairing form in the list's place.",
  code: { qml: "BluetoothList" },
  stage: { live, hint: "Show opens the drawer; press a Nearby device", daylight: true },
  variants: [
    { name: "Known and Nearby", for: "connected first with their battery", sample: () => list() },
    { name: "Drawer open", for: "Hide, the unnamed devices under it", sample: () => list(btRows("rest", { unnamed: "open" }), undefined, undefined, 1) },
    { name: "Pair", for: "a nearby device: Pair", sample: () => form() },
    { name: "Pairing", for: "the status says so; Pair off", sample: () => form({ status: "Pairing…", ready: false, focus: "cancel" }) },
    { name: "A code to type there", for: "in `primary`, typed so far under it", sample: () => form({ subtitle: "Type this code on it, then press its Enter", ask: "code", code: "482 913", status: "3 of 6 typed", ready: false, focus: "cancel", title: "MX Keys Mini", icon: "keyboard" }) },
    { name: "A code to check", for: "Pair says yes", sample: () => form({ subtitle: "Check that it shows the same code", ask: "code", code: "204 117" }) },
    { name: "A passkey to type here", for: "the field, six digits", sample: () => form({ subtitle: "Type the passkey it shows", ask: "field", placeholder: "Six digits", focus: "field" }) },
    { name: "It asks to pair", for: "nothing to type: Pair", sample: () => form({ subtitle: "It asks to pair" }) },
  ],
  sizes: doesntApply("the well takes the room under the tabs."),
  states: {
    shown: ["rest", "lit", "pressed", "selected", "open", "loading", "empty", "error"],
    absent: { disabled: "a row is never off; the form's Pair is" },
    rows: [{ name: "The tab", cell }],
    looks: {
      rest: "connected checked, their tiles `primary`, a battery before the status",
      lit: "the well's tube; the row filled",
      pressed: "the row's fill presses",
      selected: "the keyboard on a row",
      open: "the drawer: Hide, its chevron turned",
      loading: "two skeletons under Nearby while a scan finds nothing",
      empty: "off: `EmptyState` with Turn on",
      error: "the status in `error`; the form stays",
    },
  },
  focus: {
    tubes: [
      { name: "The well", tone: "accent", filled: false, content: false },
      { name: "Show or Hide", tone: "accent", filled: false, content: false },
      { name: "Pair", tone: "accent", filled: true, content: false },
    ],
    looks: { lit: "the well's tube; Show's tube on its line", signsOff: "each tube as `primary_glass`, no glow" },
    signsOff: () => list(btRows(), 7, "lit"),
  },
  content: {
    text: [
      { name: "A device's name", words: NAME, sample: (words: string) => list(btRows("rest", { nearby: [{ ...BT_NEARBY[0]!, title: words }, ...BT_NEARBY.slice(1)] })) },
      { name: "The form's name", words: NAME, sample: (words: string) => form({ title: words }) },
    ],
    missingPicture: [{ name: "No device class", for: "the Bluetooth symbol stands in", sample: () => list(btRows("rest", { nearby: BT_NEARBY.slice(2), unnamed: "none" })) }],
    counts: [
      { name: "Nothing nearby", for: "Nearby says so", sample: () => list(btRows("none", { unnamed: "none" })) },
      { name: "Discovery failed", for: "the note in `error`", sample: () => list(btRows("failed", { unnamed: "none" })) },
      { name: "Blocked", for: "rfkill: no button", sample: () => alone(wellOf(btOff("blocked"))) },
      { name: "No adapter", for: "nothing to pair with", sample: () => alone(wellOf(btOff("none"))) },
    ],
  },
  motion: {
    arrives: doesntApply("the form cross-fades in as Wi-Fi's does, over `short`."),
    leaves: doesntApply("paired, cancelled or Esc: the same cross-fade back."),
    between: [
      { name: "The well lights up", what: "The keyboard into the list: its tube fades in.", tracks: lights(".k-list > .k-ring", "the well", 0, 1), reduced: "same", back: "plays", sample: () => list() },
      { name: "A row pressed", what: "A lit row pressed: its fill presses.", tracks: press(".k-list-row[data-current]", "a nearby row"), reduced: "same", back: "plays", sample: () => list(btRows(), 7, "lit") },
      ...powerMoves(() => list(btRows(), 7, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["j", "k", "↑", "↓"], does: "through the devices and the drawer's line" },
      { keys: ["Enter"], does: "Known: connects or disconnects; Nearby: the form" },
      { keys: ["Enter"], does: "on the drawer's line: Show or Hide" },
      { keys: ["Delete"], does: "forgets a paired device" },
      { keys: ["Tab", "Shift+Tab"], does: "in the form: field, Cancel, Pair" },
      { keys: ["Esc"], does: "in the form: says no, back to the list" },
    ],
    order: "The tabs, then the well; in the form, the field, Cancel, Pair.",
    lands: "A pairing the device starts opens the form, the keyboard on it.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The list", for: "labels, rows and the drawer's line", sample: () => list() },
      { name: "A code", for: "the code in `headline`", sample: () => form({ ask: "code", code: "482 913", subtitle: "Type this code on it, then press its Enter" }) },
    ]),
    measures: [
      { part: "A device's row", token: "size-row_two_line" },
      { part: "The drawer's line", token: "size-row" },
      { part: "Show and Hide", token: "size-control_sm" },
      { part: "Between rows", token: "space-s4" },
      { part: "Label clear of rows above", token: "space-s12" },
      { part: "Form's tile", token: "size-control_lg" },
      { part: "The code's line", token: "type-headline-line" },
      { part: "To the code, to the buttons", token: "space-s16" },
    ],
    reflow: doesntApply("the well is as tall as the room the sections leave."),
  },
});
