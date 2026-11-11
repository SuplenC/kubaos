// The power menu (`config/quickshell/modules/powermenu/PowerMenu.qml`): lock, suspend, log out,
// reboot and shut down as list rows on a narrow sheet centred on the frame's right edge. It opens
// from the control center's Power button, the bar having none, and dims the desktop as a dialog does.

import type { Words } from "../../lib/content.ts";
import { type ListRowSpec, listRow, listRows, type RowPinned } from "../../lib/list.ts";
import { inset, stack } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { controlCenter, onScreen } from "./control-center-parts.ts";
import { hung, live as liven } from "./screen-parts.ts";

/** The rows, Shut down the one that can't be taken back. */
export const ACTIONS: readonly ListRowSpec[] = [
  { icon: "lock", title: "Lock" },
  { icon: "bedtime", title: "Suspend" },
  { icon: "logout", title: "Log out" },
  { icon: "restart_alt", title: "Reboot" },
  { icon: "power_settings_new", title: "Shut down", danger: true },
];

/** The menu's length: five rows `space.s4` apart, `space.s12` above and below them. */
const LENGTH = "(5 * var(--size-row) + 4 * var(--space-s4) + 2 * var(--space-s12))";
/** Centred along the right edge: half the room the frame leaves under the bar, less half its length, on a whole device pixel. */
const CENTRED = `round(down, calc((var(--sheet-room) - ${LENGTH}) / 2), 1px)`;

export interface MenuOptions {
  /** The row the keyboard is on; the first, as it opens. */
  readonly at?: number;
  readonly pinned?: RowPinned;
  readonly actions?: readonly ListRowSpec[];
  /** What taking a row does, live. */
  readonly onTake?: (index: number) => void;
}

/** The rows `space.s4` apart, `space.s12` inside the sheet, `size.rail` wide: the sheet is their well. */
export function powerMenu(options: MenuOptions = {}): HTMLElement {
  const at = options.at ?? 0;
  const specs = options.actions ?? ACTIONS;
  const rows = specs.map((spec, index) => listRow(spec, index === at, index === at ? options.pinned : undefined));
  const column = stack("s4", "start", ...rows);
  column.classList.add("k-list-rows");
  const box = inset(stack(0, "start", column), "s12", "s12");
  box.style.width = "var(--size-rail)";
  // h and l, the side arrows, Tab and Space move and run as the rows' own keys do.
  box.addEventListener(
    "keydown",
    (event) => {
      const back = event.key === "h" || event.key === "ArrowLeft" || (event.key === "Tab" && event.shiftKey);
      const on = event.key === "l" || event.key === "ArrowRight" || (event.key === "Tab" && !event.shiftKey);
      const key = back ? "ArrowUp" : on ? "ArrowDown" : event.key === " " ? "Enter" : null;
      if (key === null) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      box.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true }));
    },
    { capture: true },
  );
  listRows(box, null, () => rows, { current: at, ...(options.pinned ? { pinned: options.pinned } : {}), ...(options.onTake ? { onTake: options.onTake } : {}) });
  return box;
}

function shot(options: MenuOptions = {}, reveal?: number): HTMLElement {
  return hung(powerMenu(options), { edge: "right", corner: true, dim: "dialog", start: CENTRED, ...(reveal !== undefined ? { reveal } : {}) }).screen.element;
}

/** The stage: the menu open, the keyboard on Lock; taking a row or Esc shuts it, a click opens it again. */
function live(): HTMLElement {
  let set: (open: boolean) => void = () => {};
  const content = powerMenu({ onTake: () => shut() });
  const made = hung(content, { edge: "right", corner: true, dim: "dialog", start: CENTRED });
  const room = made.screen.element;
  function shut(): void {
    set(false);
    delete room.dataset.open;
  }
  set = liven(made.sheet, true);
  room.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    shut();
  });
  room.addEventListener("click", (event) => {
    if (content.contains(event.target as Node)) return;
    if (made.sheet.parts[0]!.hasAttribute("data-shown")) shut();
    else {
      room.dataset.open = "";
      set(true);
      requestAnimationFrame(() => content.focus({ preventScroll: true }));
    }
  });
  if (!ALL) requestAnimationFrame(() => content.focus({ preventScroll: true }));
  return room;
}

const SHEET = "the sheet";
const DIM = "the desktop's dim";

const OPENING: Move = {
  name: "Opening",
  what: "The control center's Power, Super+Escape or the power key: the sheet slides out, the desktop dims.",
  tracks: [
    { part: SHEET, on: "the power menu", target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: DIM, on: "the power menu", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: "the power menu", target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: DIM, on: "the power menu", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot(),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, a press outside, or a row taken: the sheet goes back into the edge.",
  tracks: [
    { part: SHEET, on: "the power menu", target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIM, on: "the power menu", target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: "the power menu", target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: DIM, on: "the power menu", target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot(),
};

type Shown = "rest" | "lit" | "pressed";

const ROWS = [
  { name: "On Lock", at: 0 },
  { name: "On Shut down", at: 4 },
] as const;

const SUSPEND: Words = { short: "Suspend", long: "Suspend to memory until the lid opens" };

export default page<Shown>({
  level: "shell",
  slug: "power-menu",
  title: "Power menu",
  lede: "Lock, suspend, log out, reboot, shut down: rows on a sheet centred on the right edge.",
  code: { qml: "PowerMenu" },
  from: ["panel-page", "list"],
  stage: { live, hint: "j/k or arrows move, Enter runs, Esc closes", daylight: true },
  variants: [
    { name: "Open", for: "the keyboard on Lock; the desktop dimmed as for a dialog", sample: () => shot() },
    { name: "Its way in", for: "the control center's Power, lit", sample: () => onScreen(controlCenter({ powerLit: "lit" })) },
  ],
  sizes: doesntApply("one size: `size.rail` wide, as tall as its five rows."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "every action is always there",
      selected: "the keyboard's row is lit, not picked",
      loading: "a row runs at once and the menu goes",
      empty: "always five rows",
      error: "a failed action is the session's to say",
    },
    rows: nonEmpty(
      ROWS.map(({ name, at }) => ({
        name,
        cell: (state: Shown) => shot({ at, ...(state === "rest" ? {} : { pinned: state }) }),
      })),
    ),
    looks: {
      rest: "the first row current: the list's lit row",
      lit: "pointer or keyboard: the row's tube",
      pressed: "the tube white-hot; the action runs",
    },
  },
  focus: {
    tubes: [
      { name: "A row", tone: "accent", filled: false, content: false },
      { name: "Shut down", tone: "error", filled: false, content: true },
    ],
    looks: { lit: "the tube inside the row, glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => shot({ at: ROWS[index]!.at, pinned: "lit" }),
  },
  content: {
    text: [{ name: "A row's name", words: SUSPEND, sample: (words: string) => shot({ actions: ACTIONS.map((each, index) => (index === 1 ? { ...each, title: words } : each)) }) }],
    missingPicture: doesntApply("every row's symbol is the font's."),
    counts: doesntApply("always the same five rows."),
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [
      { name: "Lit", what: "The keyboard or pointer onto a row: it lights over `short`.", tracks: lights(".k-list-row", "a row", 0, 1), reduced: "same", back: "cuts", sample: () => shot({ at: 2 }) },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot.", tracks: press(".k-list-row", "a row"), reduced: "same", back: "plays", sample: () => shot({ at: 2, pinned: "lit" }) },
      ...powerMoves(() => shot({ at: 2, pinned: "lit" })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Super+Escape", "XF86PowerOff"], does: "opens or closes it, even locked" },
      { keys: ["j", "k", "↓", "↑"], does: "the next or the last row, round" },
      { keys: ["l", "h", "→", "←"], does: "the same, the next or the last" },
      { keys: ["Tab", "Shift+Tab"], does: "the same, the next or the last" },
      { keys: ["Enter", "Space"], does: "runs the row; the menu closes" },
      { keys: ["Esc"], does: "closes it" },
    ],
    order: "One stop: every key moves along the five rows, round.",
    lands: "Opening, on Lock; closing, back where the keyboard was.",
  },
  measures: {
    samples: nonEmpty([{ name: "Open", for: "the width, the padding, the rows", sample: () => shot() }]),
    measures: [
      { part: "Width", token: "size-rail" },
      { part: "Padding round the rows", token: "space-s12" },
      { part: "Between rows", token: "space-s4" },
      { part: "Row", token: "size-row" },
      { part: "Corners, fillets", token: "radius-surface" },
    ],
    reflow: doesntApply("one size at every window size, centred on the right edge."),
  },
});
