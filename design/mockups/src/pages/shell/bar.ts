// The Bar page: the bar along the screen's top, its items flat on the glass in groups parted by
// hairlines. At the start the Command menu button, the workspaces, what you're doing (the preview,
// recording, Keep awake, Proxy, the layout, the update), the title, and now playing beside the
// clock; the clock always centred; at the end what opens a card (what is on air, the counters, the
// tray or its ⋯, the system monitor, the mute pill, the bell and status, which end it). An item
// that opens a card answers the pointer with the edge light. As the bar narrows its parts give way
// in one order (`RUNGS`), the quiet pills folding into ⋯ first.

import type { Shown } from "../../lib/control.ts";
import { type BarLayout, BAR_LAYOUTS } from "../../lib/sheet.ts";
import { row, stack } from "../../lib/layout.ts";
import { slider } from "../../lib/slider.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { WINDOWS } from "../../site/settings.ts";
import { LONG_SONG } from "./bar-audio-parts.ts";
import { barScreen, barStrip, type BarContent, EVERYDAY, EVERYTHING, fitBar, logicalWidth, RUNGS, TITLE } from "./bar-parts.ts";
import { setCommandOpen } from "./bar-command.ts";

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

const LAYOUT_NAMES = ["Bar", "Floating", "Islands"] as const;

/** The bar on a screen's top in `layout`, a strip of desktop under it, at 1600. */
function onScreen(layout: BarLayout, content: BarContent = EVERYDAY): HTMLElement {
  return barScreen(WIDE, "calc(var(--size-bar) + var(--size-control_md))", { layout, content }).element;
}

/**
 * The bar in the layout picked, everything on; the Command menu opened lights its button's sign
 * stroke by stroke as it is picked, and a press on the button lights or puts it out.
 */
function switched(picked: readonly number[]): HTMLElement {
  const layout = BAR_LAYOUTS[picked[0] ?? 0]!;
  const room = onScreen(layout, EVERYTHING);
  const button = room.querySelector<HTMLElement>(".shell-cmd");
  if (button && picked[1] === 1) requestAnimationFrame(() => requestAnimationFrame(() => setCommandOpen(button, true)));
  return room;
}

/** The Command menu button's strokes lighting in writing order, each a `duration.swap_lag` after the one before. */
const SIGN_LIGHTS: Move = {
  name: "The Command menu opens",
  what: "By a click or its key: the button's strokes light in writing order, a `swap_lag` apart.",
  tracks: nonEmpty(
    ["ク, first", "ク, second", "バ, first", "バ, second", "the dakuten, first", "the dakuten, second"].map((part, order) => ({
      part,
      on: "the Command menu button",
      target: `.shell-cmd-stroke:nth-child(${order + 1})`,
      property: "--stroke-lit",
      from: 0,
      to: 1,
      duration: "short" as const,
      ...(order > 0 ? { cue: nonEmpty(Array.from({ length: order }, () => "swap_lag" as const)) } : {}),
      curve: "standard" as const,
    })),
  ),
  reduced: "same",
  back: "plays",
  sample: () => strip(1440),
};

const SIGN_OFF: Move = {
  name: "The Command menu closes",
  what: "Its sign goes out, every stroke at once, over `short`.",
  tracks: [{ part: "every stroke", on: "the Command menu button", target: ".shell-cmd-stroke", property: "--stroke-lit", from: 1, to: 0, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => barStrip({ width: 1440, command: "open" }),
};

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
  lede: "Flat on the glass: what you're doing at the start, the clock centred, what opens a card at the end.",
  code: { qml: "Bar", unbuilt: "QML: steps not built yet" },
  stage: {
    live,
    switches: {
      each: [
        { label: "Bar layout", choices: ["Bar", "Floating", "Islands"] },
        { label: "Command menu", choices: ["Closed", "Open"] },
      ],
      live: switched,
    },
    hint: "drag the slider; point at the end; pick Open",
    daylight: true,
    stacked: true,
  },
  variants: [
    ...BAR_LAYOUTS.map((layout, index) => ({ name: LAYOUT_NAMES[index]!, for: "the same items, flat on the glass", sample: () => onScreen(layout, EVERYTHING) })),
    { name: "Everyday", for: "tray, system monitor, mute, bell, status", sample: () => strip(WIDE) },
    { name: "Command menu open", for: "its sign lit", sample: () => barStrip({ width: WIDE, command: "open" }) },
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
      rest: "flat on the glass; recording, sharing, camera in a capsule",
      hovered: "an opener's edge light, `opacity.track_lit`; its hint",
      pressed: "its content dips; the edge light full",
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
    arrives: doesntApply("it comes with the desktop: the Desktop page."),
    leaves: doesntApply("it goes with the desktop: the Desktop page."),
    between: [
      TITLE_SWAP,
      SIGN_LIGHTS,
      SIGN_OFF,
      { name: "Hovered", what: "The pointer onto an opener: the edge light comes in under it over `short`.", tracks: [{ part: "the edge light", on: "the item under the pointer", target: ".k-edge-hover", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }], reduced: "same", back: "cuts", sample: () => strip(logicalWidth(LAPTOP.width), EVERYDAY, "hovered") },
      { name: "Back to rest", what: "The pointer off: the light goes over `short`.", tracks: [{ part: "the edge light", on: "the item it leaves", target: ".k-edge-hover", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }], reduced: "same", back: "cuts", sample: () => strip(logicalWidth(LAPTOP.width), EVERYDAY, "hovered") },
      {
        name: "Pressed",
        what: "Over `micro` the content dips and the edge light goes to full.",
        tracks: [
          { part: "the content", on: "the item pressed", target: ".k-bar-dip", property: "scale", from: 1, to: { scale: "press_dip" }, duration: "micro", curve: "standard" },
          { part: "the edge light", on: "the item pressed", target: ".k-edge-hover", property: "--rail-lit", from: 0, to: 1, duration: "micro", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => strip(logicalWidth(LAPTOP.width), EVERYDAY, "hovered"),
      },
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
      { part: "Band's height", token: "size-bar" },
      { part: "A strip's height", token: "size-bar_strip" },
      { part: "Items' row", token: "size-bar_row" },
      { part: "Glass end to the mark", token: "size-bar_start" },
      { part: "Mark to its hairline", token: "size-bar_mark_air" },
      { part: "Hairline to the workspaces", token: "size-bar_start" },
      { part: "Ink to a hairline", token: "space-s12" },
      { part: "A hairline's height", token: "size-bar_divider" },
      { part: "Command menu mark", token: "size-bar_mark" },
      { part: "A capsule's height", token: "size-bar_capsule" },
      { part: "Between items", token: "space-s8" },
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
