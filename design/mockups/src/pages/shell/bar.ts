// The Bar page: `config/quickshell/modules/bar/Bar.qml`. The frame's top edge: the workspaces and
// the window title at the left, the clock in the middle, the pills at the right.

import type { Shown } from "../../lib/control.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { WINDOWS, windowSize } from "../../site/settings.ts";
import { barScreen, barStrip, CLOCK, everyRight, everydayRight, powerButton, statusPill, TITLE } from "./bar-parts.ts";

const SCREEN = "calc(1600 * var(--px))";
const LAPTOP = WINDOWS[2]!;

function strip(state: Shown = "rest", width = SCREEN): HTMLElement {
  return barStrip(width, { state, clock: { countdown: CLOCK.countdown } });
}

/** The stage: the bar on a laptop's screen, its workspaces live. */
function live(): HTMLElement {
  return barScreen(LAPTOP.width, LAPTOP.height, { workspaces: { live: true }, clock: { countdown: CLOCK.countdown } }).element;
}

const TITLE_SWAP: Move = {
  name: "A new title",
  what: "Another window takes focus: its title fades in over `short` and rises `space.s8` over `medium`.",
  tracks: [
    { part: "the title's fade", on: "the window title", target: ".shell-title > .k-text", property: "--title-fade", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the title's rise", on: "the window title", target: ".shell-title > .k-text", property: "--title-rise", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" },
  ],
  reduced: "same",
  back: "cuts",
  sample: () => strip("rest", LAPTOP.width),
};

export default page<"rest" | "lit" | "pressed">({
  level: "shell",
  slug: "bar",
  title: "Bar",
  lede: "The frame's top edge: workspaces and the window title, the clock centred, the pills at the right.",
  code: { qml: "Bar" },
  stage: { live, hint: "click a workspace; scroll over them", daylight: true },
  variants: [
    { name: "Everyday", for: "tray, system monitor, bell, status, power", sample: () => strip() },
    { name: "Everything on", for: "the shots' stand-ins: every pill at once", sample: () => barStrip(SCREEN, { right: everyRight(), clock: { countdown: CLOCK.countdown } }) },
    { name: "Live preview", for: "a checkout's home: its branch after the workspaces", sample: () => barStrip(SCREEN, { preview: true }) },
    { name: "No window", for: "an empty workspace: no title", sample: () => barStrip(SCREEN, { title: "" }) },
  ],
  sizes: doesntApply("one height, `size.bar`; its width is the screen's."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "a pill with nothing to do is gone",
      selected: "the shown workspace is the Workspaces page's",
      loading: "a pill shows once its service answers",
      empty: "the left and middle always show",
      error: "each pill says its own",
    },
    rows: nonEmpty([
      { name: "Everyday", cell: (state: "rest" | "lit" | "pressed") => strip(state, LAPTOP.width) },
      { name: "Everything on", cell: (state: "rest" | "lit" | "pressed") => barStrip(SCREEN, { right: everyRight(state), state }) },
    ]),
    looks: {
      rest: "the frame's glass, each pill on `surface_container`",
      lit: "pointer only: one pill's tube and its hint",
      pressed: "that pill's tube white-hot",
    },
  },
  focus: {
    tubes: [
      { name: "Its pills", tone: "accent", filled: false, content: false },
      { name: "Recording", tone: "error", filled: false, content: false },
      { name: "Live preview", tone: "cold", filled: false, content: false },
    ],
    looks: { lit: "each pill's tube, glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => (index === 0 ? strip("signs-off", LAPTOP.width) : barStrip(SCREEN, { right: everyRight("signs-off"), state: "signs-off" })),
  },
  content: {
    text: [{ name: "The window title", words: { short: "Firefox", long: TITLE }, sample: (words: string) => barStrip(LAPTOP.width, { title: words }) }],
    missingPicture: doesntApply("its symbols are always there; a tray app's missing picture is the Tray page's."),
    counts: [
      { name: "The fewest", for: "status and power", sample: () => barStrip(SCREEN, { right: [statusPill(), powerButton()] }) },
      { name: "Everyday", for: "five at the right", sample: () => barStrip(SCREEN, { right: everydayRight() }) },
      { name: "Every one", for: "ten: the right reaches the clock", sample: () => barStrip(SCREEN, { right: everyRight() }) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with the frame: the Frame page's reveal."),
    leaves: doesntApply("it goes with the frame: the Frame page's reveal."),
    between: [
      TITLE_SWAP,
      { name: "Lit", what: "The pointer onto a pill: it lights over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => strip("rest", LAPTOP.width) },
      { name: "Back to rest", what: "The pointer off: back to rest over `short`.", tracks: lights(".k-pill", "every pill", 1, 0), reduced: "same", back: "cuts", sample: () => strip("lit", LAPTOP.width) },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot; by day its ink tints it.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => strip("lit", LAPTOP.width) },
      ...powerMoves(() => strip("lit", LAPTOP.width)),
    ],
  },
  keyboard: doesntApply("never a stop for Tab; its actions are Items in the Command menu."),
  measures: {
    samples: nonEmpty([
      { name: "Everyday", for: "insets, gaps, the title's room", sample: () => strip("rest", LAPTOP.width) },
      { name: "Live preview", for: "the preview's gap", sample: () => barStrip(LAPTOP.width, { preview: true }) },
    ]),
    measures: [
      { part: "Bar's height", token: "size-bar" },
      { part: "Pills' height", token: "size-control_sm" },
      { part: "Ends, between pills, preview's gap", token: "space-s8" },
      { part: "Workspaces to title", token: "space-s12" },
      { part: "Title to clock, at least", token: "space-s40" },
      { part: "Title's line", token: "type-body_strong-line" },
    ],
    reflow: {
      note: "The clock stays centred; the title gives way first, then the right group runs under the clock.",
      sample: () => {
        const size = windowSize();
        return barScreen(size.width, size.height, { clock: { countdown: CLOCK.countdown } }).element;
      },
    },
  },
});
