// The Audio panel (`config/quickshell/modules/audio/`): a PanelPage on a Sheet at the frame's right
// edge, opened by the mute pill's right-click, the Command menu or its hotkey. Its sliders, the
// mic's live Meter, a headset's Calls and Music, then Output, Input and Options over their cards.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Track } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { type AudioOptions, audioPanel, LONG_APP, LONG_DEVICE, MANY, NO_APPS, onScreen, PLAYING, shot, SHORT } from "./audio-parts.ts";

type Shown = "rest" | "lit" | "selected" | "disabled" | "empty";

const draw = (options: AudioOptions = {}, height?: string) => screen(() => shot(options, height));

/** The stage's panel: live, a headset in Calls, the keyboard on the output's slider. */
function live(): HTMLElement {
  const panel = audioPanel({ headset: "calls", live: true });
  const room = onScreen(panel.element);
  if (!ALL) requestAnimationFrame(panel.land);
  return room;
}

const SHEET = "the sheet";
const SCRIM = "the desktop's dim";
const ON = "the Audio panel";

const OPENING: Move = {
  name: "Opening",
  what: "A right-click on the mute pill, its hotkey or the Command menu: the sheet grows from the right edge.",
  tracks: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: SCRIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: SCRIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot({ headset: "calls" }),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, a press outside, or the mute pill's right-click again: the sheet goes back into the edge.",
  tracks: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: SCRIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: SCRIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot({ headset: "calls" }),
};

/** A pick along a row of Tabs, `well` the row's place among the panel's: Tabs' own move, then what changes. */
function picking(well: string, reduced: boolean, after: Track<Easing>[]): NonEmpty<Track<Easing>> {
  const lit = `${well} .k-tabs-lit`;
  const edge = (part: string, property: string, travel: "short" | "medium", cue?: "micro"): Track<Easing> =>
    reduced
      ? { part, on: "the lit stretch", target: lit, property, from: 0, to: 1, duration: "instant", curve: "linear" }
      : { part, on: "the lit stretch", target: lit, property, from: 0, to: 1, duration: travel, ...(cue ? { cue } : {}), curve: "emphasized_decel" };
  return [
    edge("leading edge", "--lead", "short"),
    edge("trailing edge", "--trail", "medium", "micro"),
    { part: "flash", on: "the lit stretch", target: lit, property: "--flash", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: "flash", on: "the lit stretch", target: lit, property: "--flash", from: 1, to: 0, duration: reduced ? "short" : "long", cue: "micro", curve: "standard" },
    { part: "words", on: "the choice picked", target: `${well} .k-tab[aria-selected="true"]`, property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "words", on: "the choice it leaves", target: `${well} .k-tab[data-was]`, property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" },
    ...after,
  ];
}

const CARDS = "the tab's cards";
const TO_INPUT: Move = {
  name: "Another tab",
  what: "Input picked: the stretch glides there; the new cards fade in over `short`.",
  tracks: picking(".audio-body > :last-child", false, [{ part: CARDS, on: "Input", target: ".audio-tab", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }]),
  reduced: picking(".audio-body > :last-child", true, [{ part: CARDS, on: "Input", target: ".audio-tab", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }]),
  back: "cuts",
  sample: () => shot({ headset: "calls", tab: 1, tabFrom: 0 }),
};

const LINE = "the line under the mode";
const TO_MUSIC: Move = {
  name: "Calls to Music",
  what: "Music picked: its stretch glides, its line fades in over `short`; the mic and Input change at once.",
  tracks: picking(".audio-body > :nth-child(2)", false, [
    { part: LINE, on: "Music", target: ".audio-line", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ]),
  reduced: picking(".audio-body > :nth-child(2)", true, [
    { part: LINE, on: "Music", target: ".audio-line", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ]),
  back: "cuts",
  sample: () => shot({ headset: "music", modeFrom: "calls" }),
};

const ROWS = [
  { name: "Laptop", options: {} },
  { name: "Headset", options: { headset: "calls" } },
] as const satisfies readonly { name: string; options: AudioOptions }[];

function cell(base: AudioOptions): (state: Shown) => HTMLElement {
  return (state) => {
    switch (state) {
      case "rest":
        return shot(base);
      case "lit":
        return shot({ ...base, landed: true });
      case "selected":
        return shot({ ...base, tab: 1 });
      case "disabled":
        return shot({ ...base, headset: "music" });
      case "empty":
        return shot({ ...base, playing: [] });
    }
  };
}

export default page<Shown>({
  level: "shell",
  slug: "audio-panel",
  title: "Audio panel",
  lede: "Output and mic in one sheet: levels, the mic's meter, a headset's mode, then devices, apps and options.",
  code: { qml: "AudioPanel", unbuilt: "QML: not built yet" },
  from: ["panel-page", "slider", "meter", "tabs", "card", "select", "list", "empty-state"],
  stage: { live, hint: "drag, Space mutes; Calls/Music; tabs; Esc", daylight: true },
  variants: [
    { name: "Laptop, Output", for: "no headset: no Calls or Music", sample: draw() },
    { name: "Laptop, Input", for: "the mic's Device and the apps recording", sample: draw({ tab: 1 }) },
    { name: "Laptop, Options", for: "three switch rows", sample: draw({ tab: 2 }) },
    { name: "Headset in Calls, Output", for: "Calls picked, its line", sample: draw({ headset: "calls" }) },
    { name: "Headset in Calls, Input", for: "the headset's mic picked", sample: draw({ headset: "calls", tab: 1 }) },
    { name: "Headset in Music, Output", for: "the mic Muted, its meter empty, Input dimmed", sample: draw({ headset: "music" }) },
    { name: "Headset in Music, Input", for: "Input was open: its EmptyState, Switch to Calls", sample: draw({ headset: "music", tab: 1 }) },
    { name: "Both muted", for: "each slider's Toggled; the meter empty", sample: draw({ outputMuted: true, inputMuted: true }) },
    { name: "Boosted past 100%", for: "125%, past the dot at 100%", sample: draw({ output: 1.25 }) },
    { name: "Balance off centre", for: "the knob at Left 30%", sample: draw({ balance: 0.35 }) },
  ],
  sizes: [
    { name: "A 1000-high screen", for: "as tall as its content", sample: draw({ headset: "calls" }) },
    { name: "A short screen", for: "the body scrolls under the header", sample: draw({ headset: "calls", playing: MANY }, SHORT) },
  ],
  states: {
    shown: ["rest", "lit", "selected", "disabled", "empty"],
    absent: {
      pressed: "its parts press on their own pages",
      loading: "it opens once PipeWire answers; never half drawn",
      error: "no device: the Select says so; see questions",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: cell(options) }))),
    looks: {
      rest: "the sheet at the frame's glass; nothing lit",
      lit: "opened from the keyboard: the output's slider lit",
      selected: "Input picked: its cards in place",
      disabled: "Music mode: Input dimmed, the mic Muted",
      empty: "nothing playing: the Apps card's line",
    },
  },
  focus: doesntApply("no line of its own; each control lights on its page, the sheet's edge cutting no glow."),
  content: {
    text: [
      { name: "An app's name", words: { short: "mpv", long: LONG_APP.name }, sample: (words: string) => shot({ playing: [{ ...PLAYING[0]!, name: words }] }) },
      { name: "A device's name", words: { short: "WH-1000XM4", long: LONG_DEVICE }, sample: (words: string) => shot({ outputDevice: words, playing: [{ ...PLAYING[0]!, device: words }] }) },
    ],
    missingPicture: [{ name: "An app with no icon", for: "its tile keeps the symbol", sample: draw({ playing: [{ ...PLAYING[0]!, picture: "" }, PLAYING[1]!] }) }],
    counts: [
      { name: "Nothing playing", for: `the Apps card: ${NO_APPS.output}`, sample: draw({ playing: [] }) },
      { name: "No app recording", for: `Input: ${NO_APPS.input}`, sample: draw({ tab: 1, recording: [] }) },
      { name: "One app", for: "one row", sample: draw({ playing: PLAYING.slice(0, 1) }) },
      { name: "Many apps", for: "the body scrolls", sample: draw({ playing: MANY }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [TO_INPUT, TO_MUSIC, ...powerMoves(() => shot({ headset: "calls", landed: true }))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "round the controls, top to bottom" },
      { keys: ["←", "→", "h", "l"], does: "a slider's level; on Tabs, the choice beside" },
      { keys: ["Home", "End"], does: "a slider to 0 or its most" },
      { keys: ["Space"], does: "on a slider: its icon, muting" },
      { keys: ["Enter", "↑", "↓"], does: "a Select or a device menu: open, then pick" },
      { keys: ["Esc"], does: "an open menu shuts; else the panel closes" },
    ],
    order: "Output's slider, the mic's, Calls/Music, the tabs, then the picked tab's controls in reading order.",
    lands: "Opening, on the output's slider; closing, back where the keyboard was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Headset, Output", for: "width, sections, cards", sample: draw({ headset: "calls" }) },
      { name: "A short screen", for: "the body scrolls", sample: draw({ headset: "calls", playing: MANY }, SHORT) },
    ]),
    measures: [
      { part: "Panel's width", token: "size-panel" },
      { part: "Top, foot, sections apart", token: "space-s16" },
      { part: "Sides, the container's padding", token: "container-0-padding" },
      { part: "Sliders' height", token: "size-slider_compact" },
      { part: "Sliders and meter apart", token: "space-s8" },
      { part: "Meter's height", token: "size-progress" },
      { part: "Calls/Music to its line", token: "space-s4" },
      { part: "Line's inset", token: "space-s12" },
      { part: "Tabs' height", token: "size-control_sm" },
      { part: "Tabs to cards, cards apart", token: "space-s12" },
      { part: "Card's padding", token: "space-s16" },
      { part: "Card's label to its parts", token: "space-s8" },
      { part: "Between app rows", token: "space-s16" },
      { part: "App's tile", token: "size-control_md" },
    ],
    reflow: {
      note: "It keeps `size.panel`; past the screen's room the body scrolls under the header.",
      sample: draw({ headset: "calls", playing: MANY }, SHORT),
    },
  },
});
