// The Command menu's Action panel (`ActionPanel.qml`): Ctrl+K lists the selected Result's Actions
// over the bottom right of the Results, the Item's own first, then each Extension's Contributed
// ones, with their keys. ↑ ↓ choose, Enter runs, Esc or Ctrl+K closes it.

import type { Words } from "../../lib/content.ts";
import { stack } from "../../lib/layout.ts";
import type { Result } from "../../lib/list-detail.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { ACTIONS, actionPanel, MANY_ACTIONS, measuresOf, panel, withActions } from "./menu-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "open";

/** The stage: Ctrl+K on the panel opens it, ↑ ↓ choose, Enter, Esc or Ctrl+K close it. */
function live(): HTMLElement {
  const room = document.createElement("span");
  let open = true;
  const draw = () => {
    const shown = open ? withActions() : panel();
    room.replaceChildren(shown);
    const target = shown.querySelector<HTMLElement>(open ? ".m-actions .k-menu" : ".k-command");
    target?.addEventListener("keydown", (event) => {
      const ctrlK = event.ctrlKey && event.key.toLowerCase() === "k";
      if (ctrlK || (open && (event.key === "Escape" || event.key === "Enter"))) {
        event.preventDefault();
        event.stopPropagation();
        open = !open;
        draw();
      }
    });
    target?.focus();
  };
  draw();
  return room;
}

const TITLE: Words = { short: "Open", long: "Open in a new window on the workspace you were last on" };

const titled = (title: string): Result["actions"] => [{ ...ACTIONS[0]!, title }, ...ACTIONS.slice(1)];

const OPENS: Move = {
  name: "Opens",
  what: "Ctrl+K: the panel fades in and rises `space.s8` into place, as the menu opens.",
  tracks: [
    { part: "fade", on: "the Action panel", target: ".m-actions", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "rise", on: "the Action panel", target: ".m-actions", property: "--actions-drop", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "fade", on: "the Action panel", target: ".m-actions", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => withActions({ screen: "narrow", rows: 6 }),
};

const CLOSES: Move = {
  name: "Closes",
  what: "Esc, Ctrl+K or an Action run: it fades and drops `space.s8`.",
  tracks: [
    { part: "fade", on: "the Action panel", target: ".m-actions", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" },
    { part: "drop", on: "the Action panel", target: ".m-actions", property: "--actions-drop", from: 0, to: 1, duration: "micro", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "fade", on: "the Action panel", target: ".m-actions", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => withActions({ screen: "narrow", rows: 6 }),
};

const alone = (actions: Result["actions"] = ACTIONS, at = 0, pinned?: "lit" | "pressed") => stack(0, "start", actionPanel(actions, measuresOf("full"), at, pinned));

export default page<Shown>({
  level: "shell",
  slug: "menu-actions",
  title: "Action panel",
  lede: "Ctrl+K in the Command menu: the selected Result's Actions and their keys, over the Results.",
  code: { qml: "ActionPanel" },
  from: nonEmpty(["menu", "popover", "keycap"]),
  stage: { live, hint: "↑ ↓ choose; Esc or Ctrl+K closes, Ctrl+K opens", daylight: true },
  variants: [
    { name: "Own and Contributed", for: "the Item's first, each Extension's after a separator", sample: () => withActions() },
    { name: "Own only", for: "an Item no Extension adds to", sample: () => withActions({ actions: ACTIONS.slice(0, 4) }) },
    { name: "More than fit", for: "as tall as the body, then it scrolls", sample: () => withActions({ actions: MANY_ACTIONS }) },
  ],
  sizes: [
    { name: "Full HD", for: "half the panel wide, a grid step down", sample: () => withActions() },
    { name: "1280 × 800", for: "the narrow panel's half", sample: () => withActions({ screen: "narrow" }) },
  ],
  states: {
    shown: ["open", "rest", "lit", "pressed"],
    absent: {
      disabled: "an Action that can't run isn't listed",
      selected: "the lit entry is the one Enter runs",
      loading: "Actions come with the Result",
      empty: "Ctrl+K opens nothing for a Result with none",
      error: "a failed Action says why in the footer",
    },
    rows: [
      {
        name: "An entry",
        cell: (state) => {
          if (state === "open") return withActions({ screen: "narrow", rows: 6 });
          if (state === "rest") return alone(ACTIONS, -1);
          return alone(ACTIONS, 1, state);
        },
      },
    ],
    looks: {
      open: "over the Results' bottom right, `space.s12` above the footer",
      rest: "`on_surface` words, keycaps at the end",
      lit: "`secondary_container`; keycaps in `primary`",
      pressed: "the container lifts to `secondary_container_pressed`",
    },
  },
  focus: doesntApply("it takes the keyboard whole; the lit entry is its only light."),
  content: {
    text: [{ name: "An Action's title", words: TITLE, sample: (words: string) => alone(titled(words)) }],
    missingPicture: doesntApply("Actions have no symbols."),
    counts: [
      { name: "One", for: "Open alone, Enter's", sample: () => alone(ACTIONS.slice(0, 1)) },
      { name: "A few", for: "own, then two Extensions'", sample: () => alone() },
      { name: "Many", for: "it scrolls; the lit entry stays in view", sample: () => alone(MANY_ACTIONS, 19) },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: [
      { name: "An entry lights", what: "↓ or the pointer: the entry fades onto its container.", tracks: lights(".k-menu-item:nth-child(3)", "an entry", 0, 1), reduced: "same", back: "plays", sample: () => alone() },
      { name: "An entry pressed", what: "A lit entry pressed: its container lifts.", tracks: press(".k-menu-item[data-current]", "the lit entry"), reduced: "same", back: "plays", sample: () => alone(ACTIONS, 1, "lit") },
      ...powerMoves(() => alone(ACTIONS, 1, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "Ctrl+P", "Ctrl+N"], does: "moves between entries, stopping at the ends" },
      { keys: ["Enter"], does: "runs the lit Action, closes the panel" },
      { keys: ["Esc", "Ctrl+K"], does: "closes it" },
      { keys: ["An Action's keys"], does: "runs that Action" },
      { keys: ["Ctrl+Z"], does: "while a Toast shows: undoes it, closes the panel" },
    ],
    order: "No Tab stops: the panel holds the keyboard until it closes.",
    lands: "Opens on the first Action; closing, back in the field.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Over the menu", for: "where it sits, how wide", sample: () => withActions() },
      { name: "Alone", for: "entries, separators, keycaps", sample: () => alone() },
    ]),
    measures: [
      { part: "Width: half the panel", token: "space-s4" },
      { part: "Clear of footer rule and edge", token: "space-s12" },
      { part: "Popover's padding", token: "space-s4" },
      { part: "An entry", token: "size-control_sm" },
      { part: "Entry's inset", token: "space-s8" },
      { part: "Corners", token: "radius-control" },
      { part: "Entries' corners", token: "radius-row" },
    ],
    reflow: doesntApply("half the panel wide, as tall as its Actions up to the body."),
  },
});
