// The control center's Output and Keyboard tabs (`OutputList.qml`, `KeyboardList.qml`,
// `KeyboardAddView.qml`): the audio outputs, the default checked; the keyboard layouts, the default
// first on a star, with the toolbar at the well's foot, and "Add a language" in the same well.

import type { Words } from "../../lib/content.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { addView, alone, type KeyboardOptions, keyboardTab, LAYOUTS, outputs, OUTPUTS } from "./control-center-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "disabled" | "loading" | "empty" | "error";

const keys = (options: KeyboardOptions = {}) => alone(keyboardTab(options));

const FOUR = [...LAYOUTS, { name: "Polish", spelling: "pl" }, { name: "Japanese", spelling: "jp" }];

/** The stage: + opens the add view in the well; Back goes back. */
function live(): HTMLElement {
  const room = alone(keyboardTab({ selected: 1 }));
  const draw = (adding: boolean) => {
    const made = adding ? addView() : keyboardTab({ selected: 1 });
    const target = adding ? made.querySelector<HTMLElement>(".k-button") : [...made.querySelectorAll<HTMLElement>(".k-button")].find((each) => each.getAttribute("aria-label") === "Add a language");
    target?.addEventListener("click", () => draw(!adding));
    room.replaceChildren(made);
  };
  draw(false);
  return room;
}

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return keys();
    case "lit":
      return keys({ selected: 1, explaining: true });
    case "pressed":
      return keys({ drag: { from: 1, to: 0 }, selected: 1 });
    case "selected":
      return keys({ selected: 1, active: 1 });
    case "disabled":
      return keys({ layouts: FOUR, selected: 0 });
    case "loading":
      return alone(addView({ empty: "searching", query: "" }));
    case "empty":
      return alone(addView({ empty: "none", query: "xyzzy" }));
    case "error":
      return keys({ problem: "The keyboard layouts weren't changed: Hyprland didn't reload." });
  }
}

const NAME: Words = { short: "German", long: "English (international, with AltGr dead keys)" };

export default page<Shown>({
  level: "shell",
  slug: "control-center-keyboard",
  title: "Output and Keyboard tabs",
  lede: "The audio outputs, and the keyboard layouts with their toolbar and the add view in one well.",
  code: { qml: "KeyboardList" },
  stage: { live, hint: "press + to add a language; Back returns", daylight: true },
  variants: [
    { name: "Outputs", for: "the default checked, its line says so", sample: () => alone(outputs()) },
    { name: "Layouts", for: "the default on a star, the one typed checked", sample: () => keys() },
    { name: "A row focused", for: "the toolbar acts on it", sample: () => keys({ selected: 1 }) },
    { name: "Mac accents' tooltip", for: "what the chip does, its keys", sample: () => keys({ explaining: true }) },
    { name: "Dragging a layout", for: "lifted; the drop line where it lands", sample: () => keys({ drag: { from: 1, to: 0 }, selected: 1 }) },
    { name: "Add a language", for: "Back, the search, its matches", sample: () => alone(addView()) },
  ],
  sizes: doesntApply("the well takes the room under the tabs; a short screen scrolls it."),
  states: {
    shown: ["rest", "lit", "pressed", "selected", "disabled", "loading", "empty", "error"],
    absent: {},
    rows: [{ name: "Keyboard tab", cell }],
    looks: {
      rest: "the default's star `primary`; the one typed checked",
      lit: "the chip lit: its tooltip over it",
      pressed: "a row dragged at `opacity.subdued`; the drop line `primary`",
      selected: "typing with the second: its check",
      disabled: "four layouts, the most: + off",
      loading: "the add view while the search starts",
      empty: "no layout matches",
      error: "under the well, in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "The well", tone: "accent", filled: false, content: false },
      { name: "Mac accents", tone: "accent", filled: true, content: false },
      { name: "The search field", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the well's tube; the chip's core; the field's tube", signsOff: "each tube as `primary_glass`, no glow" },
    signsOff: () => keys({ selected: 1 }),
  },
  content: {
    text: [
      { name: "A layout's name", words: NAME, sample: (words: string) => keys({ layouts: [LAYOUTS[0]!, { name: words, spelling: "us(altgr-intl)" }] }) },
      { name: "An output's name", words: { short: "WH-1000XM4", long: "Built-in Audio Analog Stereo, Headphones on the front jack" }, sample: (words: string) => alone(outputs([OUTPUTS[0]!, { ...OUTPUTS[1]!, title: words }])) },
    ],
    missingPicture: doesntApply("every row's symbol is fixed: a star, a keyboard, a speaker."),
    counts: [
      { name: "One layout", for: "× and the arrows off", sample: () => keys({ layouts: LAYOUTS.slice(0, 1), selected: 0 }) },
      { name: "Four layouts", for: "the most: + off", sample: () => keys({ layouts: FOUR }) },
      { name: "One output", for: "the default alone", sample: () => alone(outputs(OUTPUTS.slice(0, 1))) },
      { name: "Search not running", for: "nothing to search", sample: () => alone(addView({ empty: "offline", query: "" })) },
    ],
  },
  motion: {
    arrives: doesntApply("the add view swaps in at once, made anew each time."),
    leaves: doesntApply("Back or Esc swaps the layouts in at once."),
    between: [
      { name: "The chip lights up", what: "Pointer or keyboard on Mac accents: tube and glow fade in.", tracks: lights(".k-chip", "Mac accents", 0, 1), reduced: "same", back: "plays", sample: () => keys() },
      { name: "A row pressed", what: "The focused row pressed: its fill presses.", tracks: press(".k-list-row[data-current]", "a layout"), reduced: "same", back: "plays", sample: () => keys({ selected: 1 }) },
      ...powerMoves(() => keys({ selected: 1 })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "Ctrl+J", "Ctrl+K"], does: "through the layouts or outputs" },
      { keys: ["Enter"], does: "types with the layout; an output becomes default" },
      { keys: ["Alt+↑", "Alt+↓", "Shift+J", "Shift+K"], does: "moves the focused layout" },
      { keys: ["d"], does: "makes it the default" },
      { keys: ["x", "Delete"], does: "removes it" },
      { keys: ["Tab"], does: "on to the toolbar" },
      { keys: ["←", "Esc"], does: "in the add view: back to the layouts" },
    ],
    order: "The rows, then the chip, ↑, ↓, + and ×; in the add view, Back, the field, the matches.",
    lands: "Keyboard's tile arrow opens this tab; + lands in the search field.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Layouts", for: "rows, the toolbar at the foot", sample: () => keys({ selected: 1 }) },
      { name: "Add a language", for: "the header, the field, the matches", sample: () => alone(addView()) },
    ]),
    measures: [
      { part: "Well's padding", token: "space-s12" },
      { part: "A layout's row", token: "size-row_two_line" },
      { part: "Between rows and the toolbar", token: "space-s4" },
      { part: "Toolbar's buttons", token: "size-control_sm" },
      { part: "The drop line", token: "stroke-thin" },
      { part: "Header's gap", token: "space-s8" },
      { part: "An output's row", token: "size-row" },
    ],
    reflow: doesntApply("the well is as tall as the room; on a short screen it scrolls."),
  },
});
