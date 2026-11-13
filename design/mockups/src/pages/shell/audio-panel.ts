// The Audio panel (`config/quickshell/modules/audio/`): a PanelPage on a Sheet at the frame's right
// edge, opened by the mute pill's right-click, the Command menu or its hotkey. Its sliders, the
// mic's live Meter, the Device card (the default device, the built-in speakers and mic as one, a
// headset's Calls and Music, each with its tooltip, and its codec),
// then Output, Input and Options over their cards, each side's device and each app's. Its subtitle
// follows its state: a sample of each case, in the order that picks one.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Track, type Variant } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import {
  type AudioOptions,
  audioPanel,
  BUILT_IN_MIC,
  BUILT_IN_SET,
  HEADSET,
  LONG_APP,
  LONG_DEVICE,
  linkOf,
  MANY,
  NO_APPS,
  NO_BUILT_IN_MIC,
  onScreen,
  PLAYING,
  SCREEN,
  shot,
  SHORT,
  SPEAKERS,
  subtitleCase,
  subtitleOf,
  TIPS,
  USB_HEADSET,
} from "./audio-parts.ts";

type Shown = "rest" | "lit" | "selected" | "disabled" | "empty";

const draw = (options: AudioOptions = {}, height?: string) => screen(() => shot(options, height));

/** A Bluetooth headset as the default, in Calls and in Music. */
const CALLS: AudioOptions = { device: HEADSET, mode: "calls" };
const MUSIC: AudioOptions = { device: HEADSET, mode: "music" };
/** A USB headset as the default: it plays and records at once, so no Calls or Music, no codec. */
const USB: AudioOptions = { device: USB_HEADSET };
/** A default with no mic and none picked for Input: the screen. */
const NO_MIC: AudioOptions = { device: SCREEN, inputDevice: null };

/** Nothing recording, the state the subtitle's later cases need: a recording app comes before them. */
const QUIET: AudioOptions = { recording: [] };

/**
 * A panel in a state for each of the subtitle's cases, in their order; balance has a sample for each
 * side. Each state is the case's own: the first case that holds for it.
 */
const SAID: readonly AudioOptions[] = [
  { machine: [BUILT_IN_MIC] },
  { outputMuted: true, inputMuted: true },
  { outputMuted: true },
  { inputMuted: true },
  {},
  { ...QUIET, inputMuted: true },
  { ...QUIET, ...NO_MIC },
  { ...QUIET, ...MUSIC },
  { ...QUIET, ...CALLS },
  { ...QUIET, output: 1.25 },
  { ...QUIET, output: 0 },
  { ...QUIET, outputDevice: HEADSET },
  { ...QUIET, balance: 0.35 },
  { ...QUIET, balance: 0.65 },
  QUIET,
  { ...QUIET, noise: false },
  { ...QUIET, noise: false, playing: [] },
];

/** The subtitle's samples, one per state in the cases' order, each named by the case it picks. */
function subtitles(): NonEmpty<Variant> {
  return nonEmpty(SAID.map((options) => ({ name: subtitleCase(options).when, for: subtitleOf(options), sample: draw(options) })));
}

/**
 * The stage's panel: live, a headset in Calls, the keyboard on the output's slider. Nothing records,
 * so its subtitle answers Calls and Music as well as the mutes.
 */
function live(): HTMLElement {
  const panel = audioPanel({ ...CALLS, ...QUIET, live: true });
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
  sample: () => shot(CALLS),
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
  sample: () => shot(CALLS),
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
  sample: () => shot({ ...CALLS, tab: 1, tabFrom: 0 }),
};

const TO_MUSIC: Move = {
  name: "Calls to Music",
  what: "Music picked: its stretch glides; the codecs, the mic and Input change at once.",
  tracks: picking(".audio-body > :nth-child(2)", false, []),
  reduced: picking(".audio-body > :nth-child(2)", true, []),
  back: "cuts",
  sample: () => shot({ ...MUSIC, modeFrom: "calls" }),
};

const ROWS = [
  { name: "Laptop", options: {} },
  { name: "Bluetooth", options: CALLS },
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
        return shot({ ...base, ...MUSIC });
      case "empty":
        return shot({ ...base, playing: [] });
    }
  };
}

export default page<Shown>({
  level: "shell",
  slug: "audio-panel",
  title: "Audio panel",
  lede: "Output and mic in one sheet: levels, the mic's meter, the default device, then each side's, each app's, and options.",
  code: { qml: "AudioPanel", unbuilt: "QML: not built yet" },
  from: ["panel-page", "slider", "meter", "tabs", "card", "select", "list", "empty-state"],
  stage: { live, hint: "drag, Space mutes; Calls/Music; tabs; Esc", daylight: true },
  variants: [
    { name: "Laptop, Output", for: `${BUILT_IN_SET}: Calls and Music disabled`, sample: draw() },
    { name: "Laptop, Input", for: "Default · the built-in mic, from the set", sample: draw({ tab: 1 }) },
    { name: "Laptop, Options", for: "three switch rows", sample: draw({ tab: 2 }) },
    { name: "The default, open", for: "the built-in set as one; no mic, crossed out", sample: draw({ deviceOpen: true }) },
    { name: "No built-in mic, open", for: "the speakers alone, crossed out", sample: draw({ machine: NO_BUILT_IN_MIC, deviceOpen: true }) },
    { name: "No built-in mic", for: "the speakers alone, the crossed-out mic", sample: draw({ machine: NO_BUILT_IN_MIC }) },
    { name: "No built-in mic, Input", for: `No mic on ${SPEAKERS.name}`, sample: draw({ machine: NO_BUILT_IN_MIC, tab: 1 }) },
    { name: "Bluetooth in Music, Output", for: "its codec; the mic Muted, Input dimmed", sample: draw(MUSIC) },
    { name: "Bluetooth, its tooltip", for: `the link and codec: ${linkOf(HEADSET, "LDAC")}`, sample: draw({ ...MUSIC, tooltip: true }) },
    { name: "Bluetooth in Calls, Output", for: "Calls picked, its codecs", sample: draw(CALLS) },
    { name: "Calls' tooltip", for: TIPS.calls, sample: draw({ ...CALLS, modeTip: "calls" }) },
    { name: "Music's tooltip", for: TIPS.music, sample: draw({ ...MUSIC, modeTip: "music" }) },
    { name: "The codecs, open", for: "each with what it is good for", sample: draw({ ...MUSIC, codecOpen: true }) },
    { name: "Bluetooth in Calls, Input", for: "the headset's mic as the default", sample: draw({ ...CALLS, tab: 1 }) },
    { name: "Music mode, Input", for: "the headset's mic: EmptyState, Switch to Calls", sample: draw({ ...MUSIC, tab: 1 }) },
    { name: "Music mode, another mic", for: "Input enabled, the built-in mic picked", sample: draw({ ...MUSIC, inputDevice: BUILT_IN_MIC, tab: 1 }) },
    { name: "USB headset, Input", for: "Calls and Music disabled, no codec", sample: draw({ ...USB, tab: 1 }) },
    { name: "A default with no mic", for: `Input: No mic on ${SCREEN.name}`, sample: draw({ ...NO_MIC, tab: 1 }) },
    { name: "Output on another device", for: "the speakers over the headset", sample: draw({ ...CALLS, outputDevice: SPEAKERS }) },
    { name: "An app's device, lit", for: "its words and chevron in the accent", sample: draw({ appPicker: "lit" }) },
    { name: "An app's device, open", for: "its Menu under the words", sample: draw({ appPicker: "open" }) },
    { name: "Both muted", for: "each slider's Toggled; the meter empty", sample: draw({ outputMuted: true, inputMuted: true }) },
    { name: "Boosted past 100%", for: "125%, past the dot at 100%", sample: draw({ output: 1.25 }) },
    { name: "Balance off centre", for: "the knob at Left 30%", sample: draw({ balance: 0.35 }) },
  ],
  sizes: [
    { name: "A 1000-high screen", for: "as tall as its content", sample: draw(CALLS) },
    { name: "A short screen", for: "the body scrolls under the header", sample: draw({ ...CALLS, playing: MANY }, SHORT) },
  ],
  groups: [{ title: "Subtitle", rows: subtitles() }],
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
      {
        name: "A device's name",
        words: { short: "WH-1000XM4", long: LONG_DEVICE.name },
        sample: (words: string) => {
          const device = { ...LONG_DEVICE, name: words };
          return shot({ device, playing: [{ ...PLAYING[0]!, device }] });
        },
      },
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
    between: [TO_INPUT, TO_MUSIC, ...powerMoves(() => shot({ ...CALLS, landed: true }))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "round the controls, top to bottom" },
      { keys: ["←", "→", "h", "l"], does: "a slider's level; on Tabs, the choice beside" },
      { keys: ["Home", "End"], does: "a slider to 0 or its most" },
      { keys: ["Space"], does: "on a slider: its icon, muting" },
      { keys: ["Enter", "↑", "↓"], does: "a Select, an app's too: open, then pick" },
      { keys: ["Esc"], does: "an open menu shuts; else the panel closes" },
    ],
    order: "Output's slider, the mic's, the device, Calls/Music, the codec, the tabs, then the picked tab's controls in reading order.",
    lands: "Opening, on the output's slider; closing, back where the keyboard was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Bluetooth, Output", for: "width, sections, cards", sample: draw(CALLS) },
      { name: "A short screen", for: "the body scrolls", sample: draw({ ...CALLS, playing: MANY }, SHORT) },
    ]),
    measures: [
      { part: "Panel's width", token: "size-panel" },
      { part: "Top, foot, sections apart", token: "space-s16" },
      { part: "Sides, the container's padding", token: "container-0-padding" },
      { part: "Sliders' height", token: "size-slider_compact" },
      { part: "Sliders and meter apart", token: "space-s8" },
      { part: "Meter's height", token: "size-progress" },
      { part: "Device card's parts apart", token: "space-s8" },
      { part: "Codec's label to its Select", token: "space-s8" },
      { part: "Tabs' height", token: "size-control_sm" },
      { part: "Tabs to cards, cards apart", token: "space-s12" },
      { part: "Card's padding", token: "space-s16" },
      { part: "Card's label to its parts", token: "space-s8" },
      { part: "Between app rows", token: "space-s16" },
      { part: "App's tile", token: "size-control_md" },
      { part: "Tile to name and device", token: "space-s12" },
    ],
    reflow: {
      note: "It keeps `size.panel`; past the screen's room the body scrolls under the header.",
      sample: draw({ ...CALLS, playing: MANY }, SHORT),
    },
  },
});
