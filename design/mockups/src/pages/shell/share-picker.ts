// The share picker: the system's share prompt, which an app's request to share the screen opens
// in place of the portal's own. A Dialog over the dimmed screen, on the shell's plane above the bar:
// the asking app and site, Screen · Window · Region, the box to let the app share again without
// asking, Cancel and Share. Its parts are `sharing-parts.ts`'.

import { backdrop } from "../../lib/backdrop.ts";
import type { Words } from "../../lib/content.ts";
import { type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Track } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { barScreen } from "./bar-parts.ts";
import { NARROW_WIDTH, SCREEN_WIDTH, TALL } from "./screen-parts.ts";
import {
  DESK,
  desktop,
  DISCORD,
  DISPLAYS,
  drawingScreen,
  LONG_ASKER,
  MEET,
  NO_ICON,
  type Pick,
  picker,
  pickerScreen,
  type PickerOptions,
  type Rect,
  regionDraw,
  THREE_DISPLAYS,
  WINDOWS,
  windowsOf,
} from "./sharing-parts.ts";

/** The picker alone, the keyboard in its pane unless said otherwise. */
const alone = (options: PickerOptions = {}) => picker({ keyboard: true, ...options }).dialog.element;

/** The window of the desktop a pick shares, or none when it is on another workspace. */
function deskIndex(pick: Pick): number {
  const title = windowsOf(WINDOWS)[pick.window]?.title;
  return DESK.findIndex((each) => each.title === title);
}

/**
 * The stage: pick and Enter or Share, or Esc or Cancel. Shared, the desktop shows its marks and the
 * bar its sharing pill; a region is drawn first, Enter sharing it and Esc going back. A click on
 * the desktop asks again.
 */
function live(): HTMLElement {
  const holder = document.createElement("span");
  holder.style.display = "contents";
  const again = (shown: HTMLElement) => {
    shown.addEventListener("dblclick", ask, { once: true });
    holder.replaceChildren(shown);
  };
  const shared = (pick: Pick, rect?: Rect) =>
    again(
      desktop({
        shared: pick.kind === "screen" ? { kind: "screen" } : pick.kind === "window" ? { kind: "window", index: deskIndex(pick) } : { kind: "region", ...(rect ? { rect } : {}) },
        focused: 0,
      }),
    );
  const draw = () => {
    const desk = desktop();
    const surface = regionDraw({ onDone: (rect) => shared({ kind: "region", screen: 0, window: 0 }, rect), onBack: ask });
    desk.append(surface);
    holder.replaceChildren(desk);
    if (!ALL) requestAnimationFrame(() => surface.focus({ preventScroll: true }));
  };
  function ask(): void {
    const made = picker();
    made.dialog.onAnswer = (answer) => {
      if (answer !== "Share") again(desktop());
      else if (made.choice.picked().kind === "region") draw();
      else shared(made.choice.picked());
    };
    holder.replaceChildren(backdrop({ width: `calc(${SCREEN_WIDTH} * var(--px))`, height: TALL, scrim: "dim", content: made.dialog.element, under: barScreen(SCREEN_WIDTH, TALL).element }));
    if (!ALL) requestAnimationFrame(() => made.land());
  }
  ask();
  return holder;
}

const DIM = "the dim";
const FADE = "the dialog's fade";
const RISE = "the dialog's rise";

function arriving(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "long";
  const tracks: Track<Easing>[] = [
    { part: DIM, on: "the picker", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
    { part: FADE, on: "the picker", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
  ];
  if (!reduced) tracks.push({ part: RISE, on: "the picker", target: ".k-dialog", property: "--dialog-rise", from: 1, to: 0, duration, curve: "emphasized_decel" });
  return nonEmpty(tracks);
}

function leaving(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "medium";
  return [
    { part: DIM, on: "the picker", target: ".k-backdrop-layers", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
    { part: FADE, on: "the picker", target: ".k-dialog", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
  ];
}

const ARRIVES: Move = {
  name: "Asked",
  what: "An app asks to share: the screen dims and the picker rises in, the keyboard on the main screen.",
  tracks: arriving(false),
  reduced: arriving(true),
  back: "cuts",
  sample: screen(() => pickerScreen()),
};

const LEAVES: Move = {
  name: "Shared or cancelled",
  what: "The picker and the dim fade off; shared, the sharing pill arrives in the bar as they go.",
  tracks: leaving(false),
  reduced: leaving(true),
  back: "cuts",
  sample: screen(() => pickerScreen()),
};

const KIND: Move = {
  name: "Kind switched",
  what: "Tab: the Tabs' stretch travels as a pick does; the new pane fades in over `short`.",
  tracks: [{ part: "the pane", on: "the picker", target: ".share-pane > *", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: () => alone({ pick: { kind: "window" } }),
};

const PICKED: Move = {
  name: "Screen picked",
  what: "← →: the picked line leaves one thumbnail and lights the next over `short`.",
  tracks: [
    { part: "the line leaving", on: "the first screen", target: ".k-picture-choice:first-child", property: "--checked", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the line arriving", on: "the second screen", target: ".k-picture-choice:nth-child(2)", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: () => alone({ keyboard: false }),
};

type Shown = "rest" | "lit" | "selected";

const SITE: Words = { short: "meet.google.com", long: LONG_ASKER.site! };

export default page<Shown>({
  level: "shell",
  slug: "share-picker",
  title: "Share picker",
  lede: "The system's share prompt: pick a screen, a window or a region, keyboard first.",
  code: { qml: "SharePicker", unbuilt: "Not built yet" },
  from: ["dialog", "tabs", "picture-choice", "scrolling-list", "empty-state", "checkbox", "backdrop"],
  stage: { live, hint: "← → pick, Tab kind, Enter shares; double-click asks again", daylight: true },
  variants: [
    { name: "A site asks", for: "the site in the title, the app above it", sample: () => alone() },
    { name: "An app asks", for: "the app in the title and above it", sample: () => alone({ asker: DISCORD }) },
    { name: "Window", for: "grouped by app, each with its icon", sample: () => alone({ pick: { kind: "window", window: 1 } }) },
    { name: "Region", for: "drawn on the screen after Share", sample: () => alone({ pick: { kind: "region" } }) },
    { name: "A region drawn before", for: "the pane says its size and screen", sample: () => alone({ pick: { kind: "region" }, region: "1792 × 1152 on DP-10, drawn last time" }) },
    { name: "Remembered", for: "the box ticked: no picker next time", sample: () => alone({ remember: true }) },
  ],
  sizes: [
    { name: "The shots' screen", for: "1600 by 1000", sample: screen(() => pickerScreen()) },
    { name: "A narrow screen", for: "the picker keeps `size.dialog`", sample: screen(() => pickerScreen({}, NARROW_WIDTH)) },
  ],
  groups: [
    {
      title: "Proposed: drawing a region",
      rows: [
        { name: "Before the drag", for: "the screen dimmed, a crosshair", sample: screen(() => drawingScreen(null)) },
        { name: "Drawn", for: "clear, the accent's line, its size", sample: screen(() => drawingScreen()) },
      ],
    },
  ],
  states: {
    shown: ["rest", "lit", "selected"],
    absent: {
      pressed: "its parts press on their own pages",
      disabled: "open or gone",
      loading: "a thumbnail holds Media's placeholder until drawn",
      empty: "a screen is always there to share",
      error: "a failed share ends; a notification says so",
    },
    rows: [
      { name: "Screen", cell: (state: Shown) => alone({ keyboard: state === "lit", pick: { screen: state === "selected" ? 1 : 0 } }) },
      { name: "Window", cell: (state: Shown) => alone({ keyboard: state === "lit", pick: { kind: "window", window: state === "selected" ? 3 : 0 } }) },
    ],
    looks: {
      rest: "the main screen picked: the accent's line",
      lit: "the keyboard on the pick: the cold tube",
      selected: "another pick: its line, its name in `on_surface`",
    },
  },
  focus: {
    tubes: [
      { name: "The picked screen", tone: "accent", filled: false, content: false },
      { name: "The screen with the keyboard", tone: "cold", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the thumbnail, the glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: () => alone(),
  },
  content: {
    text: [{ name: "The site", words: SITE, sample: (words: string) => alone({ asker: { ...MEET, site: words } }) }],
    missingPicture: [
      { name: "An app with no icon", for: "its initial on the caller line", sample: () => alone({ asker: NO_ICON }) },
      { name: "A window's app with no icon", for: "its initial on the row's tile", sample: () => alone({ pick: { kind: "window", window: 6 } }) },
    ],
    counts: [
      { name: "One screen", for: "the first column; the second stays empty", sample: () => alone({ displays: DISPLAYS.slice(0, 1) }) },
      { name: "Three screens", for: "three columns, thumbnails smaller", sample: () => alone({ displays: THREE_DISPLAYS }) },
      { name: "One window", for: "its app's label over one row", sample: () => alone({ pick: { kind: "window" }, windows: WINDOWS.slice(0, 1) }) },
    ],
  },
  motion: { arrives: [ARRIVES], leaves: [LEAVES], between: [KIND, PICKED] },
  keyboard: {
    keys: [
      { keys: ["←", "→", "↑", "↓"], does: "the screen or window before or after, round" },
      { keys: ["Tab", "Shift+Tab"], does: "the next or the last kind, round" },
      { keys: ["Enter"], does: "shares; for a region, starts drawing it" },
      { keys: ["Esc"], does: "cancels; while drawing, back to the picker" },
    ],
    order: "Tab never walks the buttons: it switches the kind; the box and the buttons take the pointer.",
    lands: "Opening, in the pane on the main screen; switching kind, it stays in the pane.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Screen", for: "tabs, thumbnails, the box, the buttons", sample: () => alone() },
      { name: "Window", for: "the list in the same pane", sample: () => alone({ pick: { kind: "window" } }) },
    ]),
    measures: [
      { part: "Picker's width", token: "size-dialog" },
      { part: "Padding", token: "space-s24" },
      { part: "Body to tabs", token: "space-s16" },
      { part: "Tabs to pane", token: "space-s12" },
      { part: "Pane, five rows tall", token: "size-row" },
      { part: "Between thumbnails", token: "space-s16" },
      { part: "Pane to box", token: "space-s16" },
      { part: "Box to buttons", token: "space-s24" },
    ],
    reflow: { note: "The picker keeps its width, centred; the dim fills the screen.", sample: screen(() => pickerScreen({}, NARROW_WIDTH)) },
  },
});
