// The Backdrop page: `docs/design/components/backdrop.md`. A full-screen moment, a picture, its scrim
// and one thing in the middle: polkit's prompt, Kōbō's welcome and done screens, and the glass panel
// Kōbō's step-rail screens hold.

import { backdrop } from "../../lib/backdrop.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { powerMoves } from "../../site/moves.ts";
import { windowSize } from "../../site/settings.ts";
import { atSize, DONE, done, HERO, LEDE, type Moment, polkit, SAMPLE, stepRailHost, welcome, welcomeColumn } from "./backdrops.ts";

const LAYERS = "the picture and its scrim";
const FADE = "the thing's fade";
const RISE = "the thing's rise";

const ARRIVES: Move = {
  name: "Arriving",
  what: "Kōbō starts: the picture and scrim fade in as the thing rises into place.",
  tracks: [
    { part: LAYERS, on: "Kōbō's welcome", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "scene", curve: "emphasized_decel" },
    { part: FADE, on: "Kōbō's welcome", target: ".k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "scene", curve: "emphasized_decel" },
    { part: RISE, on: "Kōbō's welcome", target: ".k-backdrop-thing", property: "--backdrop-rise", from: 1, to: 0, duration: "scene", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: LAYERS, on: "Kōbō's welcome", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: FADE, on: "Kōbō's welcome", target: ".k-backdrop-thing", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
  ],
  back: "cuts",
  sample: () => welcome(),
};

const LIFTS: Move = {
  name: "Lifting",
  what: "Kōbō's next screen: everything fades off what is under it, the thing rising on.",
  tracks: [
    { part: LAYERS, on: "Kōbō's welcome", target: ".k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "scene", curve: "emphasized_accel" },
    { part: FADE, on: "Kōbō's welcome", target: ".k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "scene", curve: "emphasized_accel" },
    { part: RISE, on: "Kōbō's welcome", target: ".k-backdrop-thing", property: "--backdrop-rise", from: 0, to: -1, duration: "scene", curve: "emphasized_accel" },
  ],
  reduced: [
    { part: LAYERS, on: "Kōbō's welcome", target: ".k-backdrop-layers", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: FADE, on: "Kōbō's welcome", target: ".k-backdrop-thing", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
  ],
  back: "cuts",
  sample: () => welcome(),
};

const PROMPT: Move = {
  name: "A prompt arriving",
  what: "Polkit asks: the desktop dims and the dialog rises in, quicker than a scene.",
  tracks: [
    { part: "the dim", on: "polkit", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the dialog's fade", on: "polkit", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the dialog's rise", on: "polkit", target: ".k-dialog", property: "--dialog-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the dim", on: "polkit", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the dialog's fade", on: "polkit", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
  ],
  back: "cuts",
  sample: () => polkit(),
};

const ROWS = [
  { name: "Polkit", cell: (state: Moment) => polkit(state) },
  { name: "Kōbō's welcome", cell: (state: Moment) => welcome(state) },
] as const;

function reflow(): HTMLElement {
  const size = windowSize();
  return atSize("welcome", size.width, size.height);
}

export default page({
  level: "templates",
  slug: "backdrop",
  title: "Backdrop",
  lede: "A full-screen moment: a picture, its scrim and one thing in the middle.",
  code: { rust: "layout::backdrop", qml: "Backdrop", unbuilt: "not built yet" },
  stage: {
    live: () => welcome(),
    hint: "Tab to Let's go, or point at it",
    daylight: true,
  },
  variants: [
    { name: "Polkit", for: "the desktop dimmed, the prompt's `Dialog` in the middle", sample: () => polkit() },
    { name: "Kōbō's welcome", for: "the words' band over the wallpaper", sample: () => welcome() },
    { name: "Kōbō's done screen", for: "the same band: neon mark, headline, two actions", sample: () => done() },
    { name: "Kōbō's step-rail screens", for: "the dim, and the glass panel StepRailPage fills", sample: () => stepRailHost() },
  ],
  sizes: doesntApply("as large as its screen; the thing keeps its own width."),
  states: {
    shown: ["rest", "loading", "error"],
    absent: {
      lit: "its field and buttons light on their pages",
      pressed: "a press on it does nothing",
      disabled: "an action not ready is Loading",
      selected: "it holds no choice",
      empty: "it always holds its one thing",
    },
    rows: nonEmpty(ROWS.map(({ name, cell }) => ({ name, cell }))),
    looks: {
      rest: "the keyboard on polkit's field or Let's go",
      loading: "Authenticate disabled; Let's go waits beside a spinner",
      error: "polkit's field Invalid; Kōbō's cause in `error`",
    },
  },
  focus: doesntApply("no line of its own; its field and buttons light on their pages."),
  content: {
    text: [
      { name: "Kōbō's hero line", words: HERO, sample: (words: string) => welcome("rest", words) },
      { name: "Kōbō's lede", words: LEDE, sample: (words: string) => welcome("rest", undefined, words) },
      { name: "The done screen's headline", words: DONE, sample: (words: string) => done(words) },
    ],
    missingPicture: [
      { name: "No picture yet", for: "`surface_container_lowest`, then the wallpaper fades in over `slow`", sample: () => backdrop({ ...SAMPLE, scrim: "band", content: welcomeColumn() }) },
    ],
    counts: doesntApply("it holds one thing, always."),
  },
  motion: {
    arrives: [ARRIVES, PROMPT],
    leaves: [LIFTS],
    between: powerMoves(() => polkit()),
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "polkit answers; Kōbō takes its `Filled` action" },
      { keys: ["Esc"], does: "polkit's Cancel; nothing on Kōbō's welcome and done" },
      { keys: ["Tab", "Shift+Tab"], does: "round the thing's controls, never out of it" },
    ],
    order: "The thing's controls in reading order; nothing under it takes the keyboard.",
    lands: "Arriving, on polkit's field or Kōbō's action; lifting, back where it was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Kōbō's welcome", for: "the column and the gaps in it", sample: () => welcome() },
      { name: "Kōbō's step-rail screens", for: "the glass panel's margin", sample: () => stepRailHost() },
    ]),
    measures: [
      { part: "Polkit's dialog", token: "size-dialog" },
      { part: "Band past the thing, its fade", token: "space-s64" },
      { part: "Kōbō's column, its sign", token: "size-column" },
      { part: "Sign → hero line", token: "space-s20" },
      { part: "Hero → lede; action → hints", token: "space-s16" },
      { part: "Lede → action", token: "space-s40" },
      { part: "Neon mark", token: "size-neon_mark" },
      { part: "Rise, arriving and lifting", token: "space-s24" },
      { part: "Sign's room; glass panel's margin", token: "space-s32" },
      { part: "Rail, in the panel's widest", token: "size-rail" },
      { part: "Glass panel's corners", token: "radius-surface" },
    ],
    reflow: {
      note: "The column stays `size.column` in the middle; at the minimum the sign gives way and the words stay.",
      sample: reflow,
    },
  },
});
