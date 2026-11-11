// The Command menu's footer (`Menu.qml`): under a hairline across the panel, the keys the menu takes
// now, or what the user should know (an Action asking first, why one failed, the Extensions that are
// down), and at its end how many problems the Daemon knows of. A Toast over the body says what
// Forget just did, and the key that undoes it.

import type { KeyHint } from "../../lib/keycap.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { confirmLine, filterHead, footerKeys, footerWords, forgotToast, LAYOUTS, panel, results, titleHead, withProblems } from "./menu-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "error";

type Screen = "full" | "narrow";

const SELECT: readonly KeyHint[] = [
  ["↑↓", "Select"],
  ["⌃K", "Actions"],
];

/** The menu with `footer` in its footer: the narrow panel, a few rows, unless `screen` says. */
const with_ = (footer: HTMLElement | undefined, extra: { screen?: Screen; query?: string } = {}) =>
  panel({ screen: extra.screen ?? "narrow", rows: 3, ...(footer ? { footer } : {}), ...(extra.query !== undefined ? { query: extra.query, results: results(extra.query) } : {}) });

const KEYS: readonly { name: string; for: string; keys: readonly KeyHint[]; sample?: () => HTMLElement }[] = [
  { name: "Search, the pane shown", for: "the pane has the row's keys", keys: [...SELECT, ["Esc", "Close"]], sample: () => with_(footerKeys([...SELECT, ["Esc", "Close"]]), { screen: "full", query: "" }) },
  { name: "Something typed", for: "Esc clears it first", keys: [...SELECT, ["Esc", "Clear"]] },
  { name: "No pane", for: "the selected row's keys", keys: [["↵", "Open"], ["Tab", "Enter app"], ["⌃K", "Actions"]] },
  { name: "In an app", for: "Esc leaves the app", keys: [...SELECT, ["Esc", "Leave app"]] },
  { name: "A list View", for: "Esc pops it", keys: [...SELECT, ["Esc", "Back"]], sample: () => panel({ screen: "full", head: filterHead("Pick a layout"), query: "", results: results("", LAYOUTS), hints: [...SELECT, ["Esc", "Back"]] }) },
  { name: "A form", for: "Tab, its submit, Esc", keys: [["Tab", "Next field"], ["↵", "Save"], ["Esc", "Back"]] },
  { name: "A detail or progress View", for: "Esc alone", keys: [["Esc", "Back"]], sample: () => panel({ screen: "narrow", rows: 3, head: titleHead("Firefox"), query: "", results: [], body: document.createElement("span"), hints: [["Esc", "Back"]] }) },
];

const FAILED = "Couldn't open Files: no such file or directory";
const DOWN = "Unavailable: Clipboard, Web search";

/** The stage: the Toast shows over the Results; Ctrl+Z takes it away. */
function live(): HTMLElement {
  const room = document.createElement("span");
  const draw = (toast: boolean) => {
    const shown = panel({ screen: "full", footer: withProblems(footerKeys([...SELECT, ["Esc", "Clear"]]), 2), ...(toast ? { over: [forgotToast()] } : {}) });
    shown.addEventListener("keydown", (event) => {
      if (event.ctrlKey && event.key.toLowerCase() === "z") draw(false);
    });
    room.replaceChildren(shown);
  };
  draw(true);
  return room;
}

const TOAST_IN: Move = {
  name: "The Toast arrives",
  what: "Forget run: the Toast rises `space.s32` and fades in over the Results' foot.",
  tracks: [
    { part: "fade", on: "the Toast", target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "rise", on: "the Toast", target: ".k-toast", property: "--toast-rise", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "fade", on: "the Toast", target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => panel({ screen: "narrow", rows: 3, over: [forgotToast()] }),
};

const TOAST_OUT: Move = {
  name: "The Toast leaves",
  what: "Undone, or too old to undo: it fades out over `medium`.",
  tracks: [{ part: "fade", on: "the Toast", target: ".k-toast", property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_accel" }],
  reduced: "same",
  back: "cuts",
  sample: () => panel({ screen: "narrow", rows: 3, over: [forgotToast()] }),
};

export default page<Shown>({
  level: "shell",
  slug: "menu-footer",
  title: "Command menu footer",
  lede: "The keys the Command menu takes now, or what needs saying, and the problems at its end.",
  code: { qml: "Menu" },
  from: nonEmpty(["keycap", "button", "toast", "divider"]),
  stage: { live, hint: "Ctrl+Z on the panel takes the Toast away", daylight: true },
  variants: [
    ...KEYS.map((each) => ({ name: each.name, for: each.for, sample: each.sample ?? (() => with_(footerKeys(each.keys))) })),
    { name: "An Action asks first", for: "the question, ↵ and Esc, in `error`", sample: () => with_(confirmLine("Power off?")) },
    { name: "An Action failed", for: "why, until the next Action", sample: () => with_(footerWords(FAILED, true)) },
    { name: "Extensions down", for: "which, in place of the keys", sample: () => with_(footerWords(DOWN, false)) },
    { name: "Problems", for: "how many, at the end; a press lists them", sample: () => with_(withProblems(footerKeys([...SELECT, ["Esc", "Clear"]]), 2)) },
    { name: "A Toast", for: "Forget done, its undo key", sample: () => panel({ screen: "narrow", rows: 3, over: [forgotToast()] }) },
  ],
  sizes: doesntApply("as wide as the panel, `size.control_sm` tall; a hint that doesn't fit goes whole."),
  states: {
    shown: ["rest", "lit", "pressed", "error"],
    absent: {
      disabled: "nothing in it is ever off",
      selected: "nothing in it is picked",
      loading: "jobs show under the field",
      empty: "it always has keys or words",
    },
    rows: [
      {
        name: "Problems",
        cell: (state) => {
          if (state === "error") return with_(withProblems(footerWords(FAILED, true), 1));
          return with_(withProblems(footerKeys([...SELECT, ["Esc", "Clear"]]), 2, state === "rest" ? undefined : state));
        },
      },
    ],
    looks: {
      rest: "keycaps and `caption` words in `on_surface_variant`",
      lit: "the problems button's text tube",
      pressed: "the button's fill presses",
      error: "the failure in `small` and `error`",
    },
  },
  focus: {
    tubes: [{ name: "Problems", tone: "accent", filled: false, content: true }],
    looks: { lit: "its words burn in the accent", signsOff: "its words `primary`, no glow" },
    signsOff: () => with_(withProblems(footerKeys(SELECT), 2, "signs-off")),
  },
  content: {
    text: [
      { name: "Why an Action failed", words: { short: "No network", long: FAILED }, sample: (words: string) => with_(footerWords(words, true)) },
      { name: "A confirmation", words: { short: "Power off?", long: "Forget every clipboard entry from the last month?" }, sample: (words: string) => with_(confirmLine(words)) },
      { name: "A Toast", words: { short: "Files", long: "Document Viewer, the PDF reader" }, sample: (words: string) => panel({ screen: "narrow", rows: 3, over: [forgotToast(words)] }) },
    ],
    missingPicture: doesntApply("the footer has no pictures."),
    counts: [
      { name: "One problem", for: "1 problem", sample: () => with_(withProblems(footerKeys(SELECT), 1)) },
      { name: "Many problems", for: "the count", sample: () => with_(withProblems(footerKeys(SELECT), 12)) },
      { name: "Hints past the room", for: "the one that doesn't fit goes whole", sample: () => with_(withProblems(footerKeys([["↵", "Open"], ["Tab", "Enter app"], ["⌃K", "Actions"], ["Esc", "Leave app"]]), 3)) },
    ],
  },
  motion: {
    arrives: [TOAST_IN],
    leaves: [TOAST_OUT],
    between: [
      { name: "Problems lights", what: "The pointer onto it: its tube fades in.", tracks: lights(".k-menu-footer .k-button", "Problems", 0, 1), reduced: "same", back: "plays", sample: () => with_(withProblems(footerKeys(SELECT), 2)) },
      { name: "Problems pressed", what: "Pressed: its fill presses.", tracks: press(".k-menu-footer .k-button", "Problems"), reduced: "same", back: "plays", sample: () => with_(withProblems(footerKeys(SELECT), 2, "lit")) },
      ...powerMoves(() => with_(withProblems(footerKeys(SELECT), 2, "lit"))),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "an Action asking: runs it" },
      { keys: ["Esc"], does: "an Action asking: drops it" },
      { keys: ["Any other key"], does: "drops the question, then does its own" },
      { keys: ["Ctrl+Z"], does: "while a Toast shows: undoes what it says" },
    ],
    order: "No Tab stops: Problems takes only the pointer.",
    lands: "Nothing in it takes the keyboard.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Keys and problems", for: "the line, the hairline, the button", sample: () => with_(withProblems(footerKeys(SELECT), 2)) },
      { name: "A question", for: "the words, then its keys", sample: () => with_(confirmLine("Power off?")) },
      { name: "A Toast", for: "where it floats", sample: () => panel({ screen: "narrow", rows: 3, over: [forgotToast()] }) },
    ]),
    measures: [
      { part: "Footer's line", token: "size-control_sm" },
      { part: "Clear of the hairline, the edge", token: "space-s12" },
      { part: "Hints apart", token: "space-s16" },
      { part: "Keycaps to their words", token: "space-s8" },
      { part: "Question to its keys", token: "space-s12" },
      { part: "Words to the problems", token: "space-s12" },
    ],
    reflow: doesntApply("the panel's width; hints that don't fit go whole, from the end."),
  },
});
