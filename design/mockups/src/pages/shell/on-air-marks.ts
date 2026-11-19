// The on-air marks: what is shared, marked on the desktop for the person alone, never in the stream.
// A shared window keeps a violet border, and its tube burns violet while it is focused; a shared
// region has a violet line just outside it, at a window's corner, by which it moves and resizes
// live; a shared screen has the same line round its whole edge, inside it. One mark for all three.
// Nothing else changes. The windows and the marks are draft parts, proposed for the library
// (`sharing-parts.ts`).

import { stack } from "../../lib/layout.ts";
import { tabs } from "../../lib/tabs.ts";
import { length } from "../../lib/tokens.ts";
import { label } from "../../site/dom.ts";
import { stageControls } from "../../site/parts/stage.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Strike, type Track } from "../../site/page.ts";
import { DESK, desktop, type DesktopOptions, deskWindow, GHOSTTY, REGION, regionMark, screenMark } from "./sharing-parts.ts";

/** A shared window's desktop, window `focused` focused. */
const windowShared = (focused = 0, index = 1) => desktop({ shared: { kind: "window", index }, focused });

/** The marks the stage switches between, each a desktop with that share on air, `focused` focused. */
const MARKS: readonly { readonly label: string; readonly draw: (focused: number) => HTMLElement }[] = [
  { label: "Screen", draw: (focused) => desktop({ shared: { kind: "screen" }, focused }) },
  { label: "Window", draw: (focused) => windowShared(focused) },
  { label: "Region", draw: (focused) => desktop({ shared: { kind: "region" }, focused }) },
];

/**
 * The stage: what is shared picked over the desktop, as the site's other stages offer their states,
 * the screen first, the choice lifted above the sample by the stage (`stageControls`). Focus
 * follows the pointer, as on the system: the window under it is the focused one, the shared one's
 * tube burning violet, and the last one keeps the focus while the pointer is off the desktop. Only
 * the windows' attributes change as the pointer moves; a new choice draws its desktop.
 */
function live(): HTMLElement {
  const holder = document.createElement("span");
  // A box of its own, as wide as what it holds: the stage measures a sample by its box, and one
  // with none (`contents`) reads as reaching from the page's edge.
  holder.style.display = "inline-block";
  // The desktop's own room, a box as large as the desktop, so the check reads the desktop inside it.
  const room = document.createElement("span");
  room.style.display = "block";
  let mark = 0;
  let focused = 0;
  const draw = () => {
    const made = MARKS[mark]!.draw(focused);
    const windows = [...made.querySelectorAll<HTMLElement>(".share-window")];
    windows.forEach((each, index) =>
      each.addEventListener("pointerenter", () => {
        focused = index;
        for (const [at, other] of windows.entries()) {
          if (at === index) other.setAttribute("data-pinned", "lit");
          else other.removeAttribute("data-pinned");
        }
      }),
    );
    room.replaceChildren(made);
  };
  const choice = tabs({
    choices: MARKS.map((each) => ({ label: each.label })),
    picked: 0,
    onPick: (index) => {
      mark = index;
      draw();
    },
  });
  holder.append(stack("s12", "start", stageControls(stack("s8", "start", label("What is shared", "label"), choice)), room));
  draw();
  return holder;
}

/** A draft part's sample room in logical pixels, as `.share-sample` sizes it: a panel and a thumbnail, with room round them. */
const sampleRoom = () => ({ width: length("size-panel") + 2 * length("space-s16"), height: length("size-thumbnail_height") + 2 * length("space-s16") });

/** A window alone in a room of its own, as a library part's sample. */
function windowAlone(shared: boolean, focused: boolean, app = DESK[1]!.app): HTMLElement {
  const room = document.createElement("span");
  room.className = "share-sample";
  room.append(deskWindow({ app, title: app.titles[0]!, column: "1", row: "1", shared, focused }));
  return room;
}

/** A mark striking on as its share starts, or off as it stops. */
function mark(part: string, target: string, on: boolean, power: Strike | Easing): NonEmpty<Track> {
  return [{ part, on: "the shared thing", target, property: "opacity", from: on ? 0 : 1, to: on ? 1 : 0, duration: "medium", curve: power }];
}

function strikes(name: string, what: string, part: string, target: string, on: boolean, sample: () => HTMLElement): Move {
  return {
    name,
    what,
    tracks: mark(part, target, on, on ? "neon.strike" : "neon.strike.reversed"),
    reduced: mark(part, target, on, "standard") as NonEmpty<Track<Easing>>,
    unflickered: mark(part, target, on, "standard") as NonEmpty<Track<Easing>>,
    back: "cuts",
    sample,
  };
}

const FOCUS: Move = {
  name: "The shared window lit",
  what: "Focus moves onto it: its violet tube and glow fade in over `short`, the border staying.",
  tracks: [{ part: "tube and glow", on: "the shared window", target: ".share-window[data-shared] > .k-ring", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: screen(() => windowShared(0)),
};

type State = "rest" | "lit";

const shown = (options: DesktopOptions) => screen(() => desktop(options));

export default page<State>({
  level: "shell",
  slug: "on-air-marks",
  title: "On-air marks",
  lede: "What is shared, marked in violet for the person alone, never in the stream; a region's corner twins and grab band too.",
  code: { unbuilt: "Not built: Hyprland, the shell" },
  stage: { live, hint: "pick what is shared; point at a window to focus it", daylight: true },
  variants: [
    { name: "A window, not focused", for: "its border violet; nothing else", sample: shown({ shared: { kind: "window", index: 1 }, focused: 0 }) },
    { name: "A window, focused", for: "its tube violet, glow at night", sample: shown({ shared: { kind: "window", index: 1 }, focused: 1 }) },
    { name: "A region", for: "a violet line just outside it", sample: shown({ shared: { kind: "region" } }) },
    { name: "A screen", for: "a violet line round its whole edge", sample: shown({ shared: { kind: "screen" } }) },
    { name: "With the camera too", for: "the marks unchanged; the bar's two pills", sample: shown({ shared: { kind: "window", index: 1 }, camera: true }) },
    { name: "Nothing shared", for: "no mark, no pill", sample: shown({}) },
  ],
  sizes: doesntApply("each mark follows what it marks, at any size."),
  groups: [
    {
      title: "Proposed library parts",
      rows: [
        { name: "Window, at rest", for: "draft: a desktop window, Hyprland's border", sample: () => windowAlone(false, false) },
        { name: "Window, shared", for: "draft: the violet border over it", sample: () => windowAlone(true, false) },
        { name: "Window, shared and focused", for: "draft: the tube in `sharing`", sample: () => windowAlone(true, true) },
        { name: "Region mark", for: "draft: `stroke.thin` outside it; drag its line or a corner", sample: () => sampleOf(regionMark({ x: 16, y: 16, width: 240, height: 120 }, sampleRoom())) },
        { name: "Screen mark", for: "draft: `stroke.thin` inside the screen's edge", sample: () => sampleOf(screenMark()) },
      ],
    },
  ],
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "a mark takes no press",
      disabled: "shown only while shared",
      selected: "nothing to pick",
      loading: "it shows as the share starts",
      empty: "nothing shared: no mark",
      error: "a failed share ends; its marks go",
    },
    rows: [{ name: "A shared window", cell: (state: State) => windowAlone(true, state === "lit") }],
    looks: { rest: "not focused: a violet border, the window border's width", lit: "focused: the violet tube and glow" },
  },
  focus: {
    tubes: [
      { name: "The shared window", tone: "sharing", filled: false, content: false },
      { name: "Any other window", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the tube over the border, the glow outside", signsOff: "the tube as violet glass, no glow" },
    signsOff: () => {
      const made = windowAlone(true, true);
      made.querySelector<HTMLElement>(".share-window")!.dataset.pinned = "signs-off";
      return made;
    },
  },
  content: {
    text: doesntApply("no words but a region's size while it moves; the pill's hint names what is shared."),
    missingPicture: [{ name: "An app with no icon", for: "its initial on the window's tile", sample: () => windowAlone(true, false, GHOSTTY) }],
    counts: doesntApply("one share at a time: its marks alone."),
  },
  motion: {
    arrives: [
      strikes("A window's share starts", "Its violet border strikes on, as a sign powering up.", "the border", ".share-border", true, screen(() => windowShared(0))),
      strikes("A region's share starts", "Its violet line strikes on.", "the line", ".share-region", true, screen(() => desktop({ shared: { kind: "region", rect: REGION } }))),
      strikes("A screen's share starts", "Its violet edge strikes on, round the whole screen.", "the edge", ".share-edge", true, screen(() => desktop({ shared: { kind: "screen" } }))),
    ],
    leaves: [
      strikes("A window's share stops", "Its border flickers out; the neutral border is left.", "the border", ".share-border", false, screen(() => windowShared(0))),
      strikes("A screen's share stops", "Its violet edge flickers out.", "the edge", ".share-edge", false, screen(() => desktop({ shared: { kind: "screen" } }))),
    ],
    between: [FOCUS],
  },
  keyboard: doesntApply("the marks take no keyboard; Hyprland's focus keys move the tube."),
  measures: {
    samples: nonEmpty([
      { name: "A shared window", for: "its border and tube", sample: () => windowAlone(true, true) },
      { name: "A shared screen", for: "the line round its edge, clear of the bar's pills", sample: shown({ shared: { kind: "screen" } }) },
    ]),
    measures: [
      { part: "Window's border, shared", token: "stroke-thin" },
      { part: "Window's corner", token: "radius-surface" },
      { part: "Region's line, outside it", token: "stroke-thin" },
      { part: "Screen's line, inside its edge", token: "stroke-thin" },
    ],
    reflow: doesntApply("the line runs round the screen's own edge at every size."),
  },
});

/** A mark in a room of its own, as a library part's sample. */
function sampleOf(mark: HTMLElement): HTMLElement {
  const room = document.createElement("span");
  room.className = "share-sample";
  room.append(mark);
  return room;
}
