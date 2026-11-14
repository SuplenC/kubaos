// The Step page: `docs/design/components/step.md`. The steps of a flow: Kōbō's step rail, each a
// mark at its end, and first boot's checklist, each a chip saying where it stands.

import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { well } from "../../lib/list.ts";
import { step, type StepOptions } from "../../lib/step.ts";
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
  what: "Pointer or keyboard: the accent's tube fades in over `short` on `standard`, the glow outside at night.",
  tracks: lights(TARGET, STEP, 0, 1),
  reduced: "same",
  back: "plays",
  sample: () => one({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, box: "rail", mark: true }, RAIL),
};

const PRESSED: Move = {
  name: "Pressed",
  what: "Over `micro` the tube runs white-hot at night; by day the tone tints its fill.",
  tracks: press(TARGET, STEP),
  reduced: "same",
  back: "plays",
  sample: () => one({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, box: "rail", mark: true, pinned: "lit" }, RAIL),
};

export default page<"rest" | "lit" | "pressed" | "selected" | "error">({
  level: "molecules",
  slug: "step",
  title: "Step",
  lede: "One step of a flow: to do, current, done, waiting on an action, or failed, each its own look.",
  code: { rust: "step::step(title)", qml: "Step", unbuilt: "neon, rail bead: not built yet" },
  stage: { live: () => {
      const both = row("s32", "start", rail(SCREENS), checklist(CARDS));
      // Side by side where the pane holds both, one over the other where it doesn't, never scaled.
      both.style.flexWrap = "wrap";
      both.style.maxWidth = "100%";
      return both;
    }, hint: "point at a step, or Tab and ↑ ↓", daylight: true },
  variants: [
    { name: "In a rail", for: "Kōbō's rail: a symbol, the title, a mark at its end", sample: () => rail(SCREENS, false) },
    { name: "In a checklist", for: "first boot's cards: two lines and a chip", sample: () => checklist(CARDS) },
    { name: "Failed", for: "its symbol `error` in `error`: Kōbō's Install", sample: () => rail([...SCREENS.slice(0, 4).map((each) => ({ ...each, state: { is: "done" } as const })), { title: "Install", icon: "download", failed: true, state: { is: "current" } }], false) },
  ],
  sizes: [
    { name: "One line", for: "a title alone: `size.row`", sample: () => one({ title: "Language", icon: "language", state: { is: "done" } }) },
    { name: "Two lines", for: "a subtitle: `size.row_two_line`, taller per extra line", sample: () => one({ title: "Fingerprint", subtitle: "Unlock with a finger on the laptop's reader.", icon: "fingerprint", state: { is: "todo" } }) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "error"],
    absent: {
      disabled: "not reached: passed over, not greyed out",
      loading: "its chip says Running",
      empty: "a step always has its title",
    },
    rows: [
      { name: "To do", cell: (state) => one({ title: "YubiKey", subtitle: "Touch the key to unlock the disk.", icon: "key", state: state === "selected" ? { is: "current" } : { is: "todo" }, failed: state === "error", pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }) },
      { name: "Done", cell: (state) => one({ title: "Fingerprint", subtitle: "Unlock with a finger.", icon: "fingerprint", state: { is: "done" }, selected: state === "selected", failed: state === "error", pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }) },
      { name: "Action", cell: (state) => one({ title: "Secure Boot", subtitle: "Enroll the key, then turn it on.", icon: "lock", state: REBOOT, selected: state === "selected", failed: state === "error", pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }) },
      { name: "Rail", cell: (state) => one({ title: "Network", icon: "wifi", state: { is: "done" }, selected: state === "selected", failed: state === "error", box: "rail", mark: true, pressable: true, ...(state === "lit" || state === "pressed" ? { pinned: state } : {}) }, RAIL) },
    ],
    looks: {
      rest: "no fill; its chip or mark says where it stands",
      lit: "the accent's tube inside its edge, glow at night",
      pressed: "tube white-hot at night; by day `opacity.press_tint`",
      selected: "`secondary_container`; in a rail, a quiet `opacity.hover` fill",
      error: "its symbol is `error`, in `error`",
    },
  },
  focus: {
    tubes: [{ name: "A step that takes a press", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the accent's tube inside its edge, the glow outside",
      signsOff: "the tube as `primary_glass`, no glow",
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
    missingPicture: doesntApply("its symbol is optional; without one it starts with its title."),
    counts: [
      { name: "No symbol", for: "its title first", sample: () => one({ title: "Account", state: { is: "todo" } }) },
      { name: "Other words", for: "its chip saying Waiting, Running, Not here", sample: () => checklist([{ title: "Snapshots", subtitle: "Roll back a bad update.", icon: "schedule", words: { label: "Running", glyph: "progress_activity" } }, { title: "Fingerprint", subtitle: "No reader on this machine.", icon: "fingerprint", words: { label: "Not here" } }]) },
      { name: "Many", for: "a whole rail: done, current, to come", sample: () => rail(SCREENS, false) },
    ],
  },
  motion: {
    arrives: doesntApply("steps come with their rail or list."),
    leaves: doesntApply("they go with it."),
    between: [LIGHTS, PRESSED, ...powerMoves(() => one({ title: "Network", icon: "wifi", state: { is: "done" }, box: "rail", mark: true, pressable: true, pinned: "lit" }, RAIL))],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "k", "j"], does: "to the step above or below that takes a press" },
      { keys: ["Enter", "Space"], does: "goes to the step it names" },
      { keys: ["Tab", "Shift+Tab"], does: "onto the steps and past them" },
    ],
    order: "Each step that takes a press, top to bottom.",
    lands: "Where its flow puts it; going to a step leaves it on that page.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Two lines, in a checklist", for: "padding, symbol, words, chip, the well", sample: () => checklist(CARDS.slice(0, 2)) },
      { name: "In a rail", for: "one line, its mark's room", sample: () => rail(SCREENS.slice(2, 4), false) },
    ]),
    measures: [
      { part: "One line", token: "size-row" },
      { part: "Two lines", token: "size-row_two_line" },
      { part: "Sides, gaps", token: "space-s12" },
      { part: "Above and below, with subtitle", token: "space-s8" },
      { part: "Title → subtitle, between steps", token: "space-s4" },
      { part: "Symbol beside two lines", token: "icon-two_line" },
      { part: "Symbol, mark beside one line", token: "type-body_strong-icon" },
      { part: "Corners in a well", token: "radius-row" },
    ],
    reflow: {
      note: "Fills its rail or well; its title ends in an ellipsis, its subtitle wraps and the step grows.",
      sample: () => within("100%", checklist([{ title: "Fingerprint", subtitle: shown(SUMMARY), icon: "fingerprint", state: { is: "done" } }, ...CARDS.slice(1)], "100%")),
    },
  },
});
