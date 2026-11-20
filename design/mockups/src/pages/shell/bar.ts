// The Bar page: the frame's top edge as it is designed now. What you're doing on the left (the
// workspaces, the preview, recording, Keep awake, Proxy, the layout, the update, the title, and now
// playing at its end beside the clock), the clock always centred, and on the right what opens a
// panel (the tray or its ⋯, the system monitor, the mute pill, the bell and status, which end it).
// As the bar narrows its parts give way in one order (`RUNGS`), now playing's words first.

import type { Shown } from "../../lib/control.ts";
import { row, stack } from "../../lib/layout.ts";
import { slider } from "../../lib/slider.ts";
import { text } from "../../lib/text.ts";
import { hovers, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { WINDOWS } from "../../site/settings.ts";
import { LONG_SONG } from "./bar-audio-parts.ts";
import { barStrip, type BarContent, EVERYDAY, EVERYTHING, fitBar, logicalWidth, RUNGS, TITLE } from "./bar-parts.ts";

/** The widths the bar is designed at, logical pixels: a 4K screen down to a portrait one at 2. */
export const STEPS = [3840, 2560, 1920, 1440, 1280, 1024, 640, 540] as const;

const LAPTOP = WINDOWS[2]!;
const WIDE = 1600;

function strip(width: number, content: BarContent = EVERYDAY, state: Shown = "rest"): HTMLElement {
  return barStrip({ width, content, state });
}

/** What gave way at `width` against the step above, and what came back, in a line. */
function gaveWay(width: number, above: number | null): string {
  const now = fitBar(width, EVERYTHING).taken;
  const before = above === null ? new Set<number>() : fitBar(above, EVERYTHING).taken;
  const fresh = [...now].filter((index) => !before.has(index)).map((index) => RUNGS[index]!.says);
  const returned = [...before].filter((index) => !now.has(index)).map((index) => RUNGS[index]!.back ?? RUNGS[index]!.says);
  const said = [...fresh, ...returned];
  return said.length === 0 ? (above === null ? "Everything shows" : "Nothing changes") : said.join(" · ");
}

/** The stage: the bar with every pill, at the width the slider picks, 540 to a 4K screen's. */
function live(): HTMLElement {
  const ascending = [...STEPS].reverse();
  const start = ascending.indexOf(1440);
  const room = document.createElement("div");
  room.className = "shell-steps-live";
  const show = (index: number) => {
    const width = ascending[index]!;
    room.replaceChildren(strip(width, EVERYTHING));
  };
  const picker = slider({ icon: "desktop_windows", stops: ascending.length, index: start, names: ascending.map(String) });
  new MutationObserver(() => show(Number(picker.getAttribute("aria-valuenow")))).observe(picker, { attributes: true, attributeFilter: ["aria-valuenow"] });
  show(start);
  const box = stack("s16", "start", picker, room);
  box.classList.add("shell-steps");
  return box;
}

/** Every step, its width and what gave way since the one above, the bar under it at its own size. */
function steps(): HTMLElement {
  const lines = STEPS.map((width, index) => {
    const head = row("s16", "centre", text(String(width), "subheading"), text(gaveWay(width, STEPS[index - 1] ?? null), "small", "on_surface_variant"));
    const room = document.createElement("div");
    room.className = "shell-steps-live";
    room.append(strip(width, EVERYTHING));
    return stack("s8", "start", head, room);
  });
  const box = stack("s24", "start", ...lines);
  box.classList.add("shell-steps");
  return box;
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
  sample: () => strip(logicalWidth(LAPTOP.width)),
};

const QUIET: BarContent = { ...EVERYDAY, tray: false, title: "" };

/** Music on an ordinary day, the focused window's title short: now playing wide. */
const PLAYING: BarContent = { ...EVERYDAY, playing: true, title: "Firefox" };
/** The same with a song that runs past its room. */
const PLAYING_LONG: BarContent = { ...PLAYING, song: LONG_SONG };
/** The title long: now playing folds, so the title keeps its room. */
const PLAYING_TITLED: BarContent = { ...EVERYDAY, playing: true };

export default page<"rest" | "hovered" | "pressed">({
  level: "shell",
  slug: "bar",
  title: "Bar",
  lede: "The frame's top edge: what you're doing on the left, the clock centred, what opens a panel on the right.",
  code: { qml: "Bar", unbuilt: "QML: steps not built yet" },
  stage: { live, hint: "drag the slider: 540 to 3840", daylight: true, stacked: true },
  variants: [
    { name: "Everyday", for: "tray, system monitor, mute, bell, status", sample: () => strip(WIDE) },
    { name: "Everything on", for: "every pill at once, on its own side", sample: () => strip(WIDE, EVERYTHING) },
    { name: "No window", for: "an empty workspace: no title", sample: () => strip(WIDE, { ...EVERYDAY, title: "" }) },
    { name: "Playing, 1440", for: "now playing wide, beside the clock", sample: () => strip(1440, PLAYING) },
    { name: "A long song, 1440", for: "its words scroll; nothing else moves", sample: () => strip(1440, PLAYING_LONG) },
    { name: "A long title, 1440", for: "now playing folds; the title keeps its room", sample: () => strip(1440, PLAYING_TITLED) },
    { name: "Playing, 1024", for: "wide again: the title gone, the date gone", sample: () => strip(1024, PLAYING) },
    { name: "Playing, 640", for: "folded: the wide pill doesn't fit", sample: () => strip(640, PLAYING) },
    { name: "The narrowest", for: "540: the last step, everything on", sample: () => strip(540, EVERYTHING) },
  ],
  sizes: doesntApply("one height, `size.bar`; its width is the screen's, and its steps the Reflow's."),
  states: {
    shown: ["rest", "hovered", "pressed"],
    absent: {
      lit: "no keyboard focus: the bar is never a Tab stop",
      disabled: "a pill with nothing to do is gone",
      selected: "the shown workspace is the Workspaces page's",
      loading: "a pill shows once its service answers",
      empty: "the workspaces, the clock and status always show",
      error: "each pill says its own",
    },
    rows: nonEmpty([
      { name: "Everyday", cell: (state: "rest" | "hovered" | "pressed") => strip(logicalWidth(LAPTOP.width), EVERYDAY, state) },
      { name: "Everything on", cell: (state: "rest" | "hovered" | "pressed") => strip(WIDE, EVERYTHING, state) },
    ]),
    looks: {
      rest: "the frame's glass, each pill on `surface_container`",
      hovered: "the pill under the pointer a step on, its hint",
      pressed: "that pill's fill a step past hover",
    },
  },
  focus: doesntApply("the bar takes no keyboard focus, so a pill never lights its tube."),
  content: {
    text: [{ name: "The window title", words: { short: "Firefox", long: TITLE }, sample: (words: string) => strip(logicalWidth(LAPTOP.width), { ...EVERYDAY, title: words }) }],
    missingPicture: doesntApply("its symbols are always there; a tray app's missing picture is the Tray page's."),
    counts: [
      { name: "The fewest", for: "workspaces, clock, system monitor, mute, bell, status", sample: () => strip(WIDE, QUIET) },
      { name: "Music", for: "and now playing, beside the clock", sample: () => strip(WIDE, { ...QUIET, playing: true }) },
      { name: "Everyday", for: "and the tray", sample: () => strip(WIDE) },
      { name: "Every one", for: "every pill, each on its side", sample: () => strip(WIDE, EVERYTHING) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with the frame: the Frame page's reveal."),
    leaves: doesntApply("it goes with the frame: the Frame page's reveal."),
    between: [
      TITLE_SWAP,
      { name: "Hovered", what: "The pointer onto a pill: its fill a step on over `short`.", tracks: hovers(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => strip(logicalWidth(LAPTOP.width)) },
      { name: "Back to rest", what: "The pointer off: back to rest over `short`.", tracks: hovers(".k-pill", "every pill", 1, 0), reduced: "same", back: "cuts", sample: () => strip(logicalWidth(LAPTOP.width), EVERYDAY, "hovered") },
      { name: "Pressed", what: "Over `micro` its fill goes a step past hovered, with no line and no glow.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => strip(logicalWidth(LAPTOP.width), EVERYDAY, "hovered") },
    ],
  },
  keyboard: doesntApply("never a stop for Tab; its actions are Items in the Command menu."),
  measures: {
    samples: nonEmpty([
      { name: "Everything on", for: "both sides' gaps, the title, the clear space", sample: () => strip(WIDE, EVERYTHING) },
      { name: "Playing", for: "the title, now playing, the clock", sample: () => strip(1440, PLAYING) },
      { name: "The narrowest", for: "540, everything on", sample: () => strip(540, EVERYTHING) },
    ]),
    measures: [
      { part: "Bar's height", token: "size-bar" },
      { part: "Pills' height", token: "size-control_sm" },
      { part: "Ends, between pills", token: "space-s8" },
      { part: "Pills to the title", token: "space-s12" },
      { part: "Clear of the clock", token: "space-s40" },
      { part: "Now playing to the clock", token: "space-s8" },
      { part: "Title, at least", token: "space-s48" },
      { part: "Title to now playing", token: "space-s8" },
      { part: "Song's room", token: "size-now_playing" },
      { part: "Title's line", token: "type-body_strong-line" },
    ],
    reflow: {
      note: "Every step from 3840 to 540, everything on: what gave way since the step above, the clock always centred.",
      sample: steps,
    },
  },
});
