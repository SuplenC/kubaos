// The control center's Output and Keyboard tabs (`OutputList.qml`, `KeyboardList.qml`,
// `KeyboardAddView.qml`): the audio outputs, the default checked; the keyboard layouts, a list as
// the radio tabs' are (the first the default by its place, the one typed with ticked on an accent tile, the row under
// the pointer or the keys lit), with the toolbar at the well's foot, and "Add a language" in the
// same well.

import type { Words } from "../../lib/content.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { addView, alone, type KeyboardOptions, keyboardTab, LAYOUTS, outputs, OUTPUTS } from "./control-center-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "disabled" | "loading" | "empty" | "error";

const keys = (options: KeyboardOptions = {}) => alone(keyboardTab(options));

const FOUR = [...LAYOUTS, { name: "Polish", spelling: "pl" }, { name: "Japanese", spelling: "jp" }];

/** The stage: a row taken types with it, its tick moving; + opens the add view; Back goes back. */
function live(): HTMLElement {
  let active = 0;
  const room = alone(keyboardTab());
  const draw = (adding: boolean) => {
    const made = adding
      ? addView()
      : keyboardTab({
          active,
          onTake: (index) => {
            active = index;
            draw(false);
          },
        });
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
      return keys({ selected: 1 });
    case "pressed":
      return keys({ selected: 1, pinned: "pressed" });
    case "selected":
      return keys({ active: 1 });
    case "disabled":
      return keys({ layouts: FOUR });
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
  stage: { live, hint: "point at a layout; press it to type with it; + adds", daylight: true },
  variants: [
    { name: "Outputs", for: "the default checked, its line says so", sample: () => alone(outputs()) },
    { name: "Layouts", for: "the first is the default; the one typed ticked", sample: () => keys() },
    { name: "The keys on a row", for: "lit as any list's; the toolbar acts on it", sample: () => keys({ selected: 1 }) },
    { name: "Typing with the second", for: "its tick; the first stays the default", sample: () => keys({ active: 1 }) },
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
      rest: "no row lit; the one typed with ticked",
      lit: "the row under the pointer or keys, as any list's",
      pressed: "the lit row's fill presses",
      selected: "typing with the second: the tick moves, the order stays",
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
    missingPicture: doesntApply("every row's symbol is fixed: a keyboard, a speaker."),
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
      { keys: ["d"], does: "makes it the default: moves it to the top" },
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
