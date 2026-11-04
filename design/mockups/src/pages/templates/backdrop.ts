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
  what: "Kōbō starts, or moves to its welcome or done screen: the picture and its scrim fade in over `scene` on `emphasized_decel`, and the thing in the middle fades in with them as it rises `space.s24` into place, a whole device pixel at a time. One slow, settled arrival: a full-screen moment is looked at as it comes. With reduce motion everything fades in where it stays, capped at `short`, with no rise.",
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
  what: "Kōbō's next screen: the whole backdrop fades off what is under it over `scene` on `emphasized_accel`, the thing rising on `space.s24` as it goes, so what comes next comes up from under it. With reduce motion it fades out where it is over `short`.",
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
  what: "Polkit asks: the desktop dims as a dialog's scrim does, over `long` on `emphasized_decel`, and the dialog fades in and rises `space.s16` into the middle as the Dialog page plays it. A question is looked at as it comes, but it isn't a scene. With reduce motion both fade in over `short`, with no rise.",
  tracks: [
    { part: "the dim", on: "polkit", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the dialog's fade", on: "polkit", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the dialog's rise", on: "polkit", target: ".k-dialog", property: "--rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
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
  summary: "A full-screen moment: a picture, its scrim, and one thing in the middle. Polkit's prompt, Kōbō's welcome and done screens, and the glass panel Kōbō's steps sit on.",
  live: () => welcome(),
  description: [
    "QML `Backdrop` for polkit, Rust `layout::backdrop` for Kōbō; both libraries, since the shell and Kōbō each have full-screen moments. The lock screen has a page of its own (Lock).",
    "A Backdrop is a picture, a scrim over it, and one thing in its middle, centred on whole device pixels, the odd one after it. The picture is Kōbō's wallpaper; polkit has none of its own and lies over the desktop it asks on. Before the picture has arrived, the screen is `surface_container_lowest`.",
    "The scrim is one of two. Behind words (Kōbō's welcome and done) it is a band of `surface` at `opacity.backdrop` (0.90) across the whole screen, solid from `space.s64` above the thing to `space.s64` below it and fading to nothing over those `space.s64` either side, so every text role keeps 4.5:1 over any picture, black or white, and the picture shows above and below the words. Behind a dialog or a panel (polkit, Kōbō's step-rail screens) it is `scrim` at `opacity.scrim` over all of it, as a Dialog's is.",
    "Polkit's thing is its Dialog. Kōbō's welcome is its accent, its neon sign `size.column` wide, the hero line, the lede, the Lg action and its key hints; the done screen its neon mark, headline, lede, actions and hints; both `size.column` wide, their words centred under one another.",
    "Kōbō's step-rail screens hold a glass panel instead: `space.s32` in from the screen's edges (`space.s16` on a screen narrower than two columns), no wider than a rail and two columns with `space.s40` beside each, at the surface radius, its own blurred copy of the picture lying where the screen's does under `surface` at `opacity.glass` (0.80), a soft hairline inside its edge, casting `shadow.panel`. StepRailPage fills it, on its own page.",
    "Where the room is shorter than the thing and `space.s32` above and below it, the parts marked as giving way go first: Kōbō's sign, then nothing else, so the words and the action always show. A Backdrop takes the whole screen it is on.",
  ],
  variants: [
    { name: "Polkit", for: "The dim over the desktop, the prompt's Dialog in the middle.", sample: () => polkit() },
    { name: "Kōbō's welcome", for: "The words' band over Kōbō's wallpaper: the sign, the hero line, the lede and the action.", sample: () => welcome() },
    { name: "Kōbō's done screen", for: "The same band: the neon mark, the headline, the lede and the two actions.", sample: () => done() },
    { name: "Kōbō's step-rail screens", for: "The dim over Kōbō's wallpaper, and the glass panel StepRailPage fills.", sample: () => stepRailHost() },
  ],
  sizes: doesntApply("a backdrop is as large as its screen; its thing keeps its own width, `size.dialog` for polkit, `size.column` for Kōbō's words, or the glass panel's, which follows the screen."),
  states: {
    shown: ["rest", "loading", "error"],
    absent: {
      lit: "the template isn't a control: the field, the dialog's buttons and Kōbō's action light as their pages say.",
      pressed: "a press on the backdrop does nothing; nothing behind it takes one.",
      disabled: "a backdrop is shown or gone; an action that can't be taken yet is a disabled button (Loading).",
      selected: "it holds no choice of its own.",
      empty: "a backdrop always has its one thing in the middle.",
    },
    rows: nonEmpty(ROWS.map(({ name, cell }) => ({ name, cell }))),
    described: {
      rest: "The thing at rest, the keyboard on its first thing to do: polkit's field, Kōbō's Let's go. The band is `surface` at `opacity.backdrop` in either mode, so by day the words sit on a light band over the picture, by night on a dark one; the dim is the same black at `opacity.scrim` in both.",
      loading: "Waiting on the machine: polkit's Authenticate is disabled until the system asks, and Kōbō's Let's go waits with a spinner and its line beside it. Nothing moves to make room.",
      error: "What went wrong, where you acted: polkit's field Invalid with its note, Kōbō's Try again with the cause beside it in `error`. The band and the picture don't change.",
    },
  },
  focus: doesntApply("the template draws no line of its own; its field, its buttons and its dialog light as their pages say."),
  content: {
    text: [
      { name: "Kōbō's hero line", words: HERO, sample: (words: string) => welcome("rest", words) },
      { name: "Kōbō's lede", words: LEDE, sample: (words: string) => welcome("rest", undefined, words) },
      { name: "The done screen's headline", words: DONE, sample: (words: string) => done(words) },
    ],
    missingPicture: [
      { name: "No picture yet", for: "Before the wallpaper has arrived, or with none: `surface_container_lowest` under the band; the wallpaper fades in over `slow` once it has.", sample: () => backdrop({ ...SAMPLE, scrim: "band", content: welcomeColumn() }) },
    ],
    counts: doesntApply("a backdrop holds one thing, always."),
  },
  motion: {
    arrives: [ARRIVES, PROMPT],
    leaves: [LIFTS],
    between: powerMoves(() => polkit()),
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Polkit: answers with what is typed. Kōbō: the screen's Filled action, Let's go or Reboot; once per press." },
      { keys: ["Esc"], does: "Polkit: Cancel. Kōbō's welcome and done: nothing; there is nowhere to go back to." },
      { keys: ["Tab", "Shift+Tab"], does: "Round the thing's controls and never out of it: polkit's field and buttons, Kōbō's actions." },
    ],
    order: "The thing's controls in reading order, round; nothing under the backdrop takes the keyboard while it is up.",
    lands: "Arriving: on polkit's field, on Kōbō's Let's go or Reboot. Lifting: back on what had the keyboard before it came.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Kōbō's welcome", for: "The column's width and the gaps between its parts.", sample: () => welcome() },
      { name: "Kōbō's step-rail screens", for: "The glass panel's margin.", sample: () => stepRailHost() },
    ]),
    measures: [
      { what: "Polkit's dialog.", token: "size-dialog" },
      { what: "The band's reach past the thing, and its fade.", token: "space-s64" },
      { what: "Kōbō's column and its sign's width.", token: "size-column" },
      { what: "Between Kōbō's sign and its hero line.", token: "space-s20" },
      { what: "Between the hero line and the lede, and the action and its hints.", token: "space-s16" },
      { what: "Between the lede and the action.", token: "space-s40" },
      { what: "The done screen's neon mark.", token: "size-neon_mark" },
      { what: "How far the thing rises as it arrives and lifts.", token: "space-s24" },
      { what: "The room kept above and below the thing before Kōbō's sign gives way, and the glass panel's margin on a wide screen.", token: "space-s32" },
      { what: "A rail, in the glass panel's widest.", token: "size-rail" },
      { what: "The glass panel's corners.", token: "radius-surface" },
    ],
    reflow: {
      described:
        "Kōbō's welcome at the toolbar's window size: the column stays `size.column` wide in the middle at every size, the band follows it, and the picture covers the screen. At the minimum the screen is shorter than the column and `space.s32` above and below it, so the sign gives way and the words and the action stay. Polkit keeps its dialog's width and centres the same way; the glass panel grows with the screen up to its widest.",
      sample: reflow,
    },
  },
});
