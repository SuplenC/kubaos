// The Prompt page: new, in neither library yet. One value asked for in a popover: its label with an
// Enter keycap, the library's Field with all of it selected, and the Field's Note. It floats under
// the text it changes (a counter's name or value), or sits inside a menu's popover (Auto reset's
// type box): one part for both.

import type { Words } from "../../lib/content.ts";
import { listRows, well } from "../../lib/list.ts";
import { stack } from "../../lib/layout.ts";
import { floatUnder, type PromptHeld } from "../../lib/prompt.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { askFor, autoMenu, type Counter, COUNTERS, counterRow, type Editing, LONG_NAME, rowAlone, sheetBox } from "../shell/counters-parts.ts";

type Shown = "lit" | "empty" | "error";

const COFFEE = COUNTERS[0]!;

/** The row's text a prompt for `what` floats under. */
const ANCHOR: Record<Editing, string> = { name: ".ct-name > .k-list-title", value: ".ct-value" };
const HELD: Record<Editing, PromptHeld> = { name: "start", value: "end" };

/** A layer the size of `content`, which a floating prompt is placed in and grows. */
function layer(content: HTMLElement): HTMLElement {
  const room = document.createElement("span");
  room.style.position = "relative";
  room.style.display = "block";
  room.append(content);
  return room;
}

/** Coffee's row with `card`, a prompt, floating under its name or its value. */
function under(what: Editing, card: HTMLElement, counter: Counter = COFFEE): HTMLElement {
  const row = rowAlone(counter, { widest: "000" });
  const room = layer(row);
  const anchor = row.querySelector<HTMLElement>(ANCHOR[what]);
  if (anchor) floatUnder(card, anchor, room, { held: HELD[what], grow: true });
  return room;
}

/** The prompt for Coffee's name or value under its text, the keyboard in it. */
function asking(what: Editing, options: { typed?: string; invalid?: boolean; pinned?: "lit" | "signs-off"; counter?: Counter } = {}): HTMLElement {
  const counter = options.counter ?? COFFEE;
  return under(
    what,
    askFor(what, counter, { pinned: options.pinned ?? "lit", ...(options.typed !== undefined ? { typed: options.typed } : {}), ...(options.invalid ? { invalid: true } : {}) }),
    counter,
  );
}

/** Auto reset's menu, its type box the keyboard's. */
function inMenu(options: { typed?: string; invalid?: boolean; signsOff?: boolean } = {}): HTMLElement {
  const made = autoMenu(COFFEE, { typed: options.typed ?? "every day at 9", fieldLit: true, ...(options.invalid ? { invalid: true } : {}) });
  if (options.signsOff) made.querySelector<HTMLElement>(".k-field")!.dataset.pinned = "signs-off";
  return made;
}

/** A counter's row, live: double-click its name or value, or F2 and Shift+F2 on it. */
function live(): HTMLElement {
  let open: HTMLElement | null = null;
  const close = () => {
    open?.remove();
    open = null;
  };
  const made = counterRow(COFFEE, {
    widest: "000",
    onEdit: (what, anchor, target) => {
      close();
      const back = () => {
        close();
        box.focus({ preventScroll: true });
      };
      open = askFor(what, { ...COFFEE, name: target.name(), value: target.value() }, {
        onCancel: back,
        onSave: (typed) => {
          back();
          if (what === "name") target.setName(typed.trim());
          else target.set(Number(typed), "set");
        },
      });
      floatUnder(open, anchor, room, { held: HELD[what], grow: true });
      requestAnimationFrame(() => open?.querySelector<HTMLElement>(".k-field")?.focus({ preventScroll: true }));
    },
  });
  const column = stack("s4", "start", made.element);
  column.classList.add("k-list-rows");
  const box = well("List", column);
  listRows(box, null, () => [made.element], {});
  box.addEventListener("keydown", (event) => {
    if (event.key === "F2" && event.shiftKey) made.edit();
    else if (event.key === "F2") made.rename();
    else return;
    event.preventDefault();
  });
  const room = layer(sheetBox(box));
  room.addEventListener("pointerdown", (event) => {
    if (open && !open.contains(event.target as Node)) close();
  });
  return room;
}

const ON = "the prompt for Coffee's name";

/** It opens: the popover fades in as the Popover's does, and the Field's caret strikes on. */
export const PROMPT_OPENS: Move = {
  name: "Opens",
  what: "The Popover's fade in, over `short`; the keyboard comes into its Field, the caret striking on as the Field's does.",
  tracks: [
    { part: "popover", on: ON, target: ".k-prompt", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "caret", on: ON, target: ".k-prompt .k-field-caret", property: "opacity", from: 0, to: 1, duration: "medium", curve: "neon.strike" },
    { part: "caret's glow", on: ON, target: ".k-prompt .k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "caret_fade", cue: ["medium", "caret_hold"], curve: "standard" },
  ],
  reduced: [
    { part: "popover", on: ON, target: ".k-prompt", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "caret", on: ON, target: ".k-prompt .k-field-caret", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "caret's glow", on: ON, target: ".k-prompt .k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "instant", cue: "caret_hold", curve: "linear" },
  ],
  unflickered: [
    { part: "popover", on: ON, target: ".k-prompt", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "caret", on: ON, target: ".k-prompt .k-field-caret", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "caret's glow", on: ON, target: ".k-prompt .k-field-caret-glow", property: "opacity", from: 1, to: 0, duration: "caret_fade", cue: ["short", "caret_hold"], curve: "standard" },
  ],
  back: "cuts",
  sample: () => asking("name"),
};

/** It closes, saved or cancelled alike: the Popover's fade out. */
export const PROMPT_CLOSES: Move = {
  name: "Closes",
  what: "Enter, Esc or a click outside: the Popover's fade out over `micro`; a saved count then rolls.",
  tracks: [{ part: "popover", on: ON, target: ".k-prompt", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  reduced: [{ part: "popover", on: ON, target: ".k-prompt", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => asking("name"),
};

/** Enter on what it can't take: the Field's own turn to invalid; its Note shows the reason at once. */
export const PROMPT_INVALID: Move = {
  name: "Can't take it",
  what: "Enter on what it can't take: the Field turns `Invalid` over `short`, its Note the reason at once.",
  tracks: [{ part: "invalid", on: "an empty name, Enter pressed", target: ".k-prompt .k-field", property: "--invalid", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => asking("name", { typed: "" }),
};

const NAME: Words = { short: "Coffee", long: LONG_NAME };
const RULE: Words = { short: "daily", long: "every Monday at 8 in the morning" };

function cell(row: "name" | "value" | "menu", state: Shown | "signs-off"): HTMLElement {
  if (row === "menu") {
    if (state === "empty") return inMenu({ typed: "" });
    if (state === "error") return inMenu({ typed: "whenever", invalid: true });
    return inMenu({ signsOff: state === "signs-off" });
  }
  const pinned = state === "signs-off" ? "signs-off" : "lit";
  if (state === "empty") return asking(row, { typed: "" });
  if (state === "error") return asking(row, row === "name" ? { typed: "", invalid: true } : { typed: "3.5", invalid: true });
  return asking(row, { pinned });
}

const ROWS = [
  { name: "Rename", key: "name" },
  { name: "Set value", key: "value" },
  { name: "In a menu: Auto reset", key: "menu" },
] as const;

export default page<Shown>({
  level: "molecules",
  slug: "prompt",
  title: "Prompt",
  lede: "One value asked for in a popover: its label and Enter, the Field all selected, its Note.",
  code: { unbuilt: "New: in neither library yet" },
  stage: { live, hint: "double-click the name or the value; F2, Shift+F2; Enter, Esc", daylight: true },
  variants: [
    { name: "Floating", for: "on its own Popover, under the text it changes", sample: () => asking("name") },
    { name: "Held to its text's end", for: "text at a panel's end: a count", sample: () => asking("value") },
    { name: "In a menu", for: "between a menu's separators: Auto reset", sample: () => inMenu() },
  ],
  sizes: doesntApply("`size.menu` wide, as tall as its label row, Field and Note."),
  states: {
    shown: ["lit", "empty", "error"],
    absent: {
      rest: "it opens with the keyboard in its Field",
      pressed: "a press in its Field places the caret",
      disabled: "it opens only where the value can change",
      selected: "opening selects the whole value: Lit",
      loading: "its value is there as it opens",
    },
    rows: nonEmpty(ROWS.map((each) => ({ name: each.name, cell: (state: Shown) => cell(each.key, state) }))),
    looks: {
      lit: "open: the Field lit, all of it selected",
      empty: "typed away: no Note, or the phrases it reads",
      error: "Enter on it: the Field `Invalid`, the reason in its Note",
    },
  },
  focus: {
    tubes: [
      { name: "Its Field", tone: "accent", filled: false, content: false },
      { name: "Its Field, invalid", tone: "error", filled: false, content: false },
    ],
    looks: {
      lit: "the Field's tube over its hairline, the glow outside",
      signsOff: "the Field's tube as glass, no glow",
    },
    signsOff: (index: number) => cell(ROWS[index]!.key, "signs-off"),
  },
  content: {
    text: [
      { name: "A long name", words: NAME, sample: (words: string) => asking("name", { counter: { ...COFFEE, name: words } }) },
      { name: "A rule and its reading", words: RULE, sample: (words: string) => inMenu({ typed: words }) },
    ],
    missingPicture: doesntApply("it draws no picture."),
    counts: doesntApply("one Field: it asks for one value."),
  },
  motion: {
    arrives: [PROMPT_OPENS],
    leaves: [PROMPT_CLOSES],
    between: [PROMPT_INVALID, ...powerMoves(() => asking("name"))],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "takes the value; one it can't take turns it invalid" },
      { keys: ["Esc"], does: "cancels; in a menu, back to the menu" },
      { keys: ["Tab", "Shift+Tab"], does: "floating, stays in its Field; in a menu, the menu's" },
      { keys: ["← →", "Home", "End", "Ctrl+A"], does: "the Field's own: caret and selection" },
    ],
    order: "One stop, its Field; its label, keycap and Note are none.",
    lands: "Opening, in its Field, all of it selected; closed, back on what opened it.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Floating", for: "width, padding, label row, Field, Note", sample: () => asking("name", { typed: "", invalid: true }) },
      { name: "Under its text", for: "`space.s8` below it, its end held", sample: () => asking("value") },
      { name: "In a menu", for: "inset to land as the floating one", sample: () => inMenu() },
    ]),
    measures: [
      { part: "Width", token: "size-menu" },
      { part: "Padding, floating", token: "space-s12" },
      { part: "Inset in a menu", token: "space-s8" },
      { part: "Label row", token: "size-control_sm" },
      { part: "Label, Field, Note apart", token: "space-s4" },
      { part: "Field", token: "size-control_md" },
      { part: "Under its text", token: "space-s8" },
      { part: "Corners", token: "radius-control" },
    ],
    reflow: doesntApply("its width at every window size; at a panel's end it holds its text's end."),
  },
});
