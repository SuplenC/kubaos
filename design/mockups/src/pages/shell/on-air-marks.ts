// The on-air marks: what is shared, marked on the desktop for the person alone, never in the stream.
// A shared window keeps a violet border, and its tube burns violet while it has the keyboard; a
// shared region has a violet line just outside it; a shared screen has a violet square in each of
// its outer corners. Nothing else changes. The windows and the marks are draft parts, proposed for
// the library (`sharing-parts.ts`).

import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Strike, type Track } from "../../site/page.ts";
import { DESK, desktop, type DesktopOptions, deskWindow, REGION, regionMark, screenMarks, WINDOWS } from "./sharing-parts.ts";

/** A shared window's desktop, the keyboard on window `focused`. */
const windowShared = (focused = 0, index = 1) => desktop({ shared: { kind: "window", index }, focused });

/** The stage: a click on a window gives it the keyboard; the shared one's tube burns violet. */
function live(): HTMLElement {
  const holder = document.createElement("span");
  holder.style.display = "contents";
  const draw = (focused: number) => {
    const made = windowShared(focused);
    made.querySelectorAll<HTMLElement>(".share-window").forEach((each, index) => each.addEventListener("click", () => draw(index)));
    holder.replaceChildren(made);
  };
  draw(0);
  return holder;
}

/** A window alone in a room of its own, as a library part's sample. */
function windowAlone(shared: boolean, focused: boolean, app = DESK[1]!.app): HTMLElement {
  const room = document.createElement("span");
  room.className = "share-sample";
  room.append(deskWindow({ app, title: app.titles[0]!, rect: { x: 16, y: 16, width: 392, height: 160 }, shared, focused }));
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
  lede: "What is shared, marked in violet for the person alone, never in the stream.",
  code: { unbuilt: "Not built: Hyprland, the shell" },
  stage: { live, hint: "click a window: the shared one's tube burns violet", daylight: true },
  variants: [
    { name: "A window, not focused", for: "its border violet; nothing else", sample: shown({ shared: { kind: "window", index: 1 }, focused: 0 }) },
    { name: "A window, focused", for: "its tube violet, glow at night", sample: shown({ shared: { kind: "window", index: 1 }, focused: 1 }) },
    { name: "A region", for: "a violet line just outside it", sample: shown({ shared: { kind: "region" } }) },
    { name: "A screen", for: "four violet squares in its corners", sample: shown({ shared: { kind: "screen" } }) },
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
        { name: "Region mark", for: "draft: `stroke.thin` outside the region", sample: () => sampleOf(regionMark({ x: 16, y: 16, width: 240, height: 120 })) },
        { name: "Screen marks", for: "draft: `size.badge_dot` squares, no rounding", sample: () => sampleOf(screenMarks()) },
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
    looks: { rest: "a violet border, the window border's width", lit: "the keyboard on it: the violet tube and glow" },
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
    text: doesntApply("the marks hold no words; the pill's hint names what is shared."),
    missingPicture: [{ name: "An app with no icon", for: "its initial on the window's tile", sample: () => windowAlone(true, false, WINDOWS[5]!) }],
    counts: doesntApply("one share at a time: its marks alone."),
  },
  motion: {
    arrives: [
      strikes("A window's share starts", "Its violet border strikes on, as a sign powering up.", "the border", ".share-border", true, screen(() => windowShared(0))),
      strikes("A region's share starts", "Its violet line strikes on.", "the line", ".share-region", true, screen(() => desktop({ shared: { kind: "region", rect: REGION } }))),
      strikes("A screen's share starts", "Its four corner squares strike on together.", "the squares", ".share-corner", true, screen(() => desktop({ shared: { kind: "screen" } }))),
    ],
    leaves: [
      strikes("A window's share stops", "Its border flickers out; the neutral border is left.", "the border", ".share-border", false, screen(() => windowShared(0))),
      strikes("A screen's share stops", "Its corner squares flicker out together.", "the squares", ".share-corner", false, screen(() => desktop({ shared: { kind: "screen" } }))),
    ],
    between: [FOCUS],
  },
  keyboard: doesntApply("the marks take no keyboard; Hyprland's focus keys move the tube."),
  measures: {
    samples: nonEmpty([
      { name: "A shared window", for: "its border and tube", sample: () => windowAlone(true, true) },
      { name: "A shared screen", for: "the squares in its corners", sample: shown({ shared: { kind: "screen" } }) },
    ]),
    measures: [
      { part: "Window's border, shared", token: "stroke-thin" },
      { part: "Window's corner", token: "radius-surface" },
      { part: "Region's line, outside it", token: "stroke-thin" },
      { part: "Corner square's side", token: "size-badge_dot" },
    ],
    reflow: doesntApply("the squares sit in the screen's own corners at every size."),
  },
});

/** A mark in a room of its own, as a library part's sample. */
function sampleOf(mark: HTMLElement): HTMLElement {
  const room = document.createElement("span");
  room.className = "share-sample";
  room.append(mark);
  return room;
}
