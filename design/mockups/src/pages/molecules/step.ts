// The Step page: `docs/design/components/step.md`. The steps of a flow: Kōbō's step rail, each a
// mark at its end, and first boot's checklist, each a chip saying where it stands.

import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { well } from "../../lib/list.ts";
import { step, type StepOptions } from "../../lib/step.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** Kōbō's step rail, inside its 12 of padding. */
const RAIL = "calc(var(--size-rail) - 2 * var(--space-s12))";
/** First boot's checklist: its well in the left pane. */
const CHECKLIST = "var(--size-column)";

const SUMMARY: Words = {
  short: "Unlock with a finger.",
  long: "Unlock the disk, sudo and the lock screen with a finger on the laptop's reader, and keep the password for when it fails.",
};

const REBOOT = { is: "action", label: "Reboot", glyph: "restart_alt" } as const;

/** First boot's cards as steps: to do, done, waiting on a reboot, and running. */
const CARDS: readonly StepOptions[] = [
  { title: "Fingerprint", subtitle: "Unlock with a finger on the laptop's reader.", icon: "fingerprint", state: { is: "done" } },
  { title: "YubiKey", subtitle: "Touch the key to unlock the disk and sudo.", icon: "key", state: { is: "todo" }, selected: true },
  { title: "Secure Boot", subtitle: "Enroll the key, then turn it on in the firmware.", icon: "lock", state: REBOOT },
  { title: "Snapshots", subtitle: "Roll back a bad update from the boot menu.", icon: "schedule", state: { is: "todo" }, words: { label: "Waiting", glyph: "schedule" } },
];

/** Kōbō's screens as steps: done, the current one, and those to come. */
const SCREENS: readonly StepOptions[] = [
  { title: "Language", icon: "language", state: { is: "done" } },
  { title: "Network", icon: "wifi", state: { is: "done" } },
  { title: "Disk", icon: "storage", state: { is: "current" } },
  { title: "Account", icon: "person", state: { is: "todo" } },
  { title: "Install", icon: "download", state: { is: "todo" } },
];

/** `options` as steps in Kōbō's rail: marks at their end, pressable where they've been reached. */
function rail(options: readonly StepOptions[], pressable = true): HTMLElement {
  const steps = options.map((each, index) => step({ ...each, box: "rail", mark: true, pressable: pressable && index < 2 }));
  arrows(steps);
  return within(RAIL, stack(0, "start", ...steps));
}

/** `options` as steps in a list's well, `space.s4` apart, its corners concentric with theirs. */
function checklist(options: readonly StepOptions[], room = CHECKLIST): HTMLElement {
  const steps = options.map((each) => step({ ...each, pressable: true }));
  arrows(steps);
  const rows = stack("s4", "start", ...steps);
  rows.classList.add("k-list-rows");
  return within(room, well("List", rows));
}

/** ↑ and ↓ (and k and j) move the keyboard between the pressable ones of `steps`. */
function arrows(steps: readonly HTMLElement[]): void {
  const pressable = steps.filter((each) => each.tabIndex === 0);
  pressable.forEach((each, index) => {
    each.addEventListener("keydown", (event) => {
      const by = event.key === "ArrowDown" || event.key === "j" ? 1 : event.key === "ArrowUp" || event.key === "k" ? -1 : 0;
      const next = pressable[index + by];
      if (by === 0 || !next) return;
      event.preventDefault();
      next.focus({ preventScroll: true });
    });
  });
}

function one(options: StepOptions, room = CHECKLIST): HTMLElement {
  return within(room, step(options));
}

const STEP = "a pressable step";
const TARGET = ".k-step";

const LIGHTS: Move = {
  name: "Lighting up",
  what: "The pointer onto a step that takes a press, or the keyboard on it: the accent's tube fades in inside its edge over `short` on `standard`, with the glow outside it at night. The pointer leaving plays it back. A fade, never the flicker.",
  tracks: lights(TARGET, STEP, 0, 1),
  reduced: "same",
  back: "plays",
  sample: () => one({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, box: "rail", mark: true }, RAIL),
};

const PRESSED: Move = {
  name: "Pressed",
  what: "Pressed while lit: over `micro` on `standard` the tube runs white-hot, to the core, at night; by day the tone tints its fill (`opacity.press_tint`). Letting go goes to the step it names.",
  tracks: press(TARGET, STEP),
  reduced: "same",
  back: "plays",
  sample: () => one({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, box: "rail", mark: true, pinned: "lit" }, RAIL),
};

const DESCRIBED =
  "An optional symbol (`icon.two_line`'s 24 beside a subtitle, `body_strong`'s 20 beside a title alone), a `body_strong` title on one line, a `small` subtitle in `on_surface_variant` that wraps to as many lines as it needs, never cut, then where it stands, `space.s12` apart. `size.row` (44) tall with one line; with a subtitle `space.s8` above and below, so `size.row_two_line` (56) with its first line and a line taller for each line more; `space.s12` in at the sides. Its corner is its box's less the box's padding: `radius.row` in a list's well, `radius.surface` less `space.s12` in Kōbō's rail.";

export default page<"rest" | "lit" | "pressed" | "selected" | "error">({
  level: "molecules",
  slug: "step",
  title: "Step",
  summary: "One step of a flow: to do, current, done, waiting on an action, or failed, each its own look.",
  live: () => stack("s32", "start", rail(SCREENS), checklist(CARDS)),
  description: [
    "Rust `step::step(title)` with `subtitle`, `icon`, `state`, `words`, `mark`, `selected` and `on_press_maybe`, drawn by `view`; QML `Step`. Kōbō's step rail, each step a mark at its end, and first boot's checklist, each a chip. Point at the steps, or Tab onto them and use ↑ and ↓.",
    DESCRIBED,
    "Where it stands is a chip: to do a Neutral chip, done an Accent chip with a check, an action (Reboot) a Filled chip with its symbol, the loudest of the three since it is what to do next; the current step is the selected row, on `secondary_container`, its words `on_secondary_container` and its subtitle that at `opacity.subdued_text`, with no chip. On the selected row done is a Neutral chip with its check, since the accent's tint over `secondary_container` leaves its label under 4.5:1. A chip can say other words in its state's look: Waiting, Running, Not here. In a narrow rail a mark takes the chip's place: a check when done, the action's symbol in `primary`, its room kept otherwise so the ends line up. In a rail the symbol is a bead on the rail's progress wire (StepRailPage): `primary` once its step is reached, the current one glowing at night with a lit control's glow and humming while it runs, a step not reached in its ink; and the current step is a quiet fill, `on_surface` at `opacity.hover`, its words as they rest.",
    "A step that failed shows `error` in its symbol's place, in `error`: Kōbō's Install step on its failed screen. A step that takes a press is a control: lit, the accent's tube inside its edge, the glow outside at night; pressed, the tube white-hot at night and the tone's tint by day. One that takes none (a step not reached yet) is passed over.",
  ],
  variants: [
    { name: "In a rail", for: "Kōbō's step rail: a symbol, the title, and a mark at the end.", sample: () => rail(SCREENS, false) },
    { name: "In a checklist", for: "First boot's cards: a two-line step with a chip saying where it stands.", sample: () => checklist(CARDS) },
    { name: "Failed", for: "The step that failed, its symbol `error` in `error`: Kōbō's Install on its failed screen.", sample: () => rail([...SCREENS.slice(0, 4).map((each) => ({ ...each, state: { is: "done" } as const })), { title: "Install", icon: "download", failed: true, state: { is: "current" } }], false) },
  ],
  sizes: [
    { name: "One line", for: "A title alone: `size.row`, 44.", sample: () => one({ title: "Language", icon: "language", state: { is: "done" } }) },
    { name: "Two lines", for: "With a subtitle: `size.row_two_line`, 56, and a line taller for each line more.", sample: () => one({ title: "Fingerprint", subtitle: "Unlock with a finger on the laptop's reader.", icon: "fingerprint", state: { is: "todo" } }) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "error"],
    absent: {
      disabled: "a step not reached takes no press and looks as one to do: it is passed over rather than greyed out.",
      loading: "a step running says so on its chip, in its state's look (Running, with `progress_activity`).",
      empty: "a step always has its title.",
    },
    rows: [
      { name: "To do", cell: (state) => one({ title: "YubiKey", subtitle: "Touch the key to unlock the disk.", icon: "key", state: state === "selected" ? { is: "current" } : { is: "todo" }, failed: state === "error", pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }) },
      { name: "Done", cell: (state) => one({ title: "Fingerprint", subtitle: "Unlock with a finger.", icon: "fingerprint", state: { is: "done" }, selected: state === "selected", failed: state === "error", pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }) },
      { name: "Action", cell: (state) => one({ title: "Secure Boot", subtitle: "Enroll the key, then turn it on.", icon: "lock", state: REBOOT, selected: state === "selected", failed: state === "error", pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }) },
      { name: "Rail", cell: (state) => one({ title: "Network", icon: "wifi", state: { is: "done" }, selected: state === "selected", failed: state === "error", box: "rail", mark: true, pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }, RAIL) },
    ],
    described: {
      rest: "No fill: its words `on_surface` and `on_surface_variant`, its chip or mark saying where it stands. By day the same roles in daylight colours.",
      lit: "The pointer or the keyboard on a step that takes a press: the accent's tube inside its edge at its corner, the glow outside it at night. By day the line alone.",
      pressed: "Overdriven: at night the tube runs white-hot, to the core, the glow a touch stronger; by day the tone tints its fill at `opacity.press_tint`.",
      selected: "The current step, or the one picked: `secondary_container`, its words `on_secondary_container`, its subtitle that at `opacity.subdued_text`; the current step has no chip, a picked done one a Neutral chip. In a rail, a quiet fill, `on_surface` at `opacity.hover`, its words as they rest.",
      error: "Failed: its symbol is `error`, in `error`; the rest as it was.",
    },
  },
  focus: {
    tubes: [{ name: "A step that takes a press", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night: the accent's tube inside its edge at its corner, the glow outside it.",
      signsOff: "By day, and at night while the signs are off: the tube as glass, `primary_glass`, no glow.",
    },
    signsOff: (index: number) => {
      const shapes: StepOptions[] = [
        { title: "YubiKey", subtitle: "Touch the key to unlock the disk.", icon: "key", state: { is: "todo" } },
        { title: "Fingerprint", subtitle: "Unlock with a finger.", icon: "fingerprint", state: { is: "done" } },
        { title: "Secure Boot", subtitle: "Enroll the key, then turn it on.", icon: "lock", state: REBOOT },
      ];
      const shape = shapes[index];
      return shape ? one({ ...shape, pressable: true, pinned: "signs-off" }) : one({ title: "Network", icon: "wifi", state: { is: "done" }, box: "rail", mark: true, pressable: true, pinned: "signs-off" }, RAIL);
    },
  },
  content: {
    text: [{ name: "Its subtitle", words: SUMMARY, sample: (words: string) => one({ title: "Fingerprint", subtitle: words, icon: "fingerprint", state: { is: "todo" } }) }],
    missingPicture: doesntApply("its symbol is optional and one of KubaOS Symbols; a step without one starts with its title."),
    counts: [
      { name: "No symbol", for: "Its title first.", sample: () => one({ title: "Account", state: { is: "todo" } }) },
      { name: "Other words", for: "Its state's chip saying something else: Waiting, Running, Not here.", sample: () => checklist([{ title: "Snapshots", subtitle: "Roll back a bad update.", icon: "schedule", words: { label: "Running", glyph: "progress_activity" } }, { title: "Fingerprint", subtitle: "No reader on this machine.", icon: "fingerprint", words: { label: "Not here" } }]) },
      { name: "Many", for: "A flow's whole rail, done, current and to come.", sample: () => rail(SCREENS, false) },
    ],
  },
  motion: {
    arrives: doesntApply("steps come with their rail or list."),
    leaves: doesntApply("they go with it."),
    between: [LIGHTS, PRESSED, ...powerMoves(() => one({ title: "Network", icon: "wifi", state: { is: "done" }, box: "rail", mark: true, pressable: true, pinned: "lit" }, RAIL))],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "k", "j"], does: "The keyboard to the step above or below that takes a press." },
      { keys: ["Enter", "Space"], does: "Goes to the step it names, once per press." },
      { keys: ["Tab", "Shift+Tab"], does: "Onto the steps and past them." },
    ],
    order: "Each step that takes a press, top to bottom; one that takes none is passed over.",
    lands: "Where its flow puts the keyboard; going to a step leaves it on that step's page.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Two lines, in a checklist", for: "Its padding, the symbol, the words and the chip; the well round it.", sample: () => checklist(CARDS.slice(0, 2)) },
      { name: "In a rail", for: "One line, its mark's room.", sample: () => rail(SCREENS.slice(2, 4), false) },
    ]),
    measures: [
      { what: "One line.", token: "size-row" },
      { what: "Two lines, with its first subtitle line.", token: "size-row_two_line" },
      { what: "In at the sides, and between its parts.", token: "space-s12" },
      { what: "Above and below, with a subtitle.", token: "space-s8" },
      { what: "Between the title and the subtitle, and between steps in a well.", token: "space-s4" },
      { what: "The symbol beside two lines.", token: "icon-two_line" },
      { what: "The symbol and the mark beside one line, `body_strong`'s.", token: "type-body_strong-icon" },
      { what: "Its corners in a list's well.", token: "radius-row" },
    ],
    reflow: {
      described: "A step fills its rail or its well at every window size; its title ends in an ellipsis, its subtitle wraps onto more lines and the step grows a line for each.",
      sample: () => within("100%", stack("s8", "start", checklist([{ title: "Fingerprint", subtitle: shown(SUMMARY), icon: "fingerprint", state: { is: "done" } }, ...CARDS.slice(1)], "100%"), text("A checklist as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
