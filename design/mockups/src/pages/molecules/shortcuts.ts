// The Shortcuts page: `docs/design/components/shortcuts.md`. Every key a surface takes, grouped by
// what they do, in a popover over the `info` button that opens it, so the surface itself shows no
// key hints.

import { row, stack } from "../../lib/layout.ts";
import { type ShortcutGroup, shortcuts } from "../../lib/shortcuts.ts";
import { shortcutsButton, takeShortcutKeys } from "../../lib/shortcuts-button.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** The wallpaper picker's keys, as its popover lists them. */
export const PICKER_KEYS: readonly ShortcutGroup[] = [
  { name: "Browse", keys: [["← →", "The picture before or after"], ["↑ ↓", "The floor above or below"]] },
  { name: "Views", keys: [["1", "Picture"], ["2", "Desktop"], ["3", "Lock"], ["F", "Full screen and back"], ["W", "Wake the lock, or let it rest"]] },
  { name: "Depth", keys: [["D", "Depth on or off"]] },
  { name: "Apply and leave", keys: [["↵", "Apply"], ["Esc", "Cancel: the old wallpaper back"], ["?", "These shortcuts"]] },
];

const ONE: readonly ShortcutGroup[] = [{ name: "Browse", keys: [["← →", "The picture before or after"]] }];

const LONG: readonly ShortcutGroup[] = [
  { name: "Apply and leave", keys: [["Ctrl+Shift+Enter", "Apply to every screen, and keep the old one for the lock"], ["Esc", "Cancel"]] },
];

/** The button with its popover open or shut, with room above for the popover. */
function opened(open = true, groups = PICKER_KEYS): HTMLElement {
  const made = shortcutsButton(groups, { shown: open });
  const room = stack("s8", "start", made.element);
  room.style.paddingTop = open ? "calc(var(--size-panel) * 1.4)" : "0";
  return room;
}

/** Live: press the button or `?` in the box, Esc or a press outside closes it. */
function live(): HTMLElement {
  const made = shortcutsButton(PICKER_KEYS);
  const room = row("s8", "end", made.element);
  room.style.minHeight = "calc(var(--size-panel) * 1.6)";
  room.tabIndex = 0;
  takeShortcutKeys(room, made);
  return room;
}

const ON = "the shortcuts";

const OPENS: Move = {
  name: "Opening",
  what: "A press or `?`: it fades in over `long` above its button.",
  tracks: [{ part: "the popover", on: ON, target: ".k-shortcuts", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the popover", on: ON, target: ".k-shortcuts", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => opened(true),
};

const CLOSES: Move = {
  name: "Closing",
  what: "Esc, a press outside or on the button: it fades out over `medium`.",
  tracks: [{ part: "the popover", on: ON, target: ".k-shortcuts", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: "the popover", on: ON, target: ".k-shortcuts", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => opened(true),
};

export default page({
  level: "molecules",
  slug: "shortcuts",
  title: "Shortcuts",
  lede: "Every key a surface takes, grouped by what they do, in a popover over its `info` button.",
  code: { unbuilt: "QML Shortcuts: not built yet" },
  from: ["popover", "keycap", "icon-button", "tooltip"],
  stage: { live, hint: "press the button, or click the box and ?", daylight: true },
  variants: [
    { name: "The wallpaper picker's", for: "four groups: browse, views, depth, leave", sample: () => shortcuts(PICKER_KEYS) },
    { name: "One group", for: "a surface of few keys", sample: () => shortcuts(ONE) },
    { name: "Over its button", for: "`space.s8` above it, from its start", sample: () => opened(true) },
  ],
  sizes: doesntApply("`size.panel` wide; as tall as its groups."),
  states: {
    shown: ["rest", "open"],
    absent: {
      lit: "its button lights as an IconButton does",
      pressed: "its button presses as an IconButton does",
      disabled: "a surface with keys always has them",
      selected: "it holds no choice",
      loading: "its keys are the surface's own",
      empty: "a surface with no keys has no button",
      error: "it has no value to be wrong",
    },
    rows: nonEmpty([{ name: "The button", cell: (state: "rest" | "open") => opened(state === "open") }]),
    looks: { rest: "the `info` button alone", open: "the popover over it, step 2" },
  },
  focus: doesntApply("its button lights as an IconButton; the popover has no line."),
  content: {
    text: doesntApply("its rows' words are each surface's own; a long one wraps."),
    missingPicture: doesntApply("keys and words only."),
    counts: [
      { name: "One key", for: "a group, a row", sample: () => shortcuts(ONE) },
      { name: "Many", for: "every group its rows", sample: () => shortcuts(PICKER_KEYS) },
      { name: "A long combination", for: "its words wrap; the keys hold their column", sample: () => shortcuts(LONG) },
    ],
  },
  motion: { arrives: [OPENS], leaves: [CLOSES], between: doesntApply("it is open or shut.") },
  keyboard: {
    keys: [
      { keys: ["?"], does: "opens or closes it, wherever focus is on the surface" },
      { keys: ["Enter", "Space"], does: "on its button: opens or closes it" },
      { keys: ["Esc"], does: "closes it, before the surface takes Esc" },
    ],
    order: "Its button is one stop; the popover holds no stops.",
    lands: "Focus stays where it was as it opens and closes.",
  },
  measures: {
    samples: nonEmpty([{ name: "The popover", for: "width, groups, rows, keys", sample: () => shortcuts(PICKER_KEYS) }]),
    measures: [
      { part: "Width", token: "size-panel" },
      { part: "Padding", token: "space-s12" },
      { part: "Between groups", token: "space-s16" },
      { part: "Group's name → rows", token: "space-s4" },
      { part: "Row", token: "size-row_compact" },
      { part: "Words → keys, at least", token: "space-s16" },
      { part: "Between keycaps", token: "space-s4" },
      { part: "Over its button", token: "space-s8" },
      { part: "Corner", token: "radius-control" },
    ],
    reflow: doesntApply("`size.panel` wide at every window size."),
  },
});
