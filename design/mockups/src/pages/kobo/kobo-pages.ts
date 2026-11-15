// What the pages of Kōbō's panel screens share: the room drawn at a screen, the moves between
// screens (the next one sliding in as the rail's wire grows to it, the blocker's flash), and the
// panel's measures.

import { type Measure, type Move, type NonEmpty, screen, type Track } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { type Answers, filledAnswers, freshAnswers, type Machine } from "./kobo-parts.ts";
import { LAPTOP, type Panel, room, type RoomOptions } from "./kobo-room.ts";

/** Each screen's place in the rail of the sample laptop, which has no cable in. */
export const PLACE: Record<Panel, number> = { language: 0, network: 1, you: 2, disk: 3, yours: 4, review: 5, details: 6, failed: 6 };

/** The room at `panel` with `answers` made from the laptop's, `more` on top. */
export function at(panel: Panel, answers: (machine: Machine) => Partial<Answers> = () => ({}), more: Partial<RoomOptions> = {}, fresh = false): () => HTMLElement {
  return screen(() => {
    const machine = more.machine ?? LAPTOP;
    const base = fresh ? freshAnswers(machine) : filledAnswers(machine);
    return room({ at: { panel }, answers: { ...base, ...answers(machine) }, ...more });
  });
}

/** The live stage: the room at `panel`, at the toolbar's window size, going on through the flow. */
export function live(panel: Panel, answers: (machine: Machine) => Partial<Answers> = () => ({}), fresh = false): () => HTMLElement {
  return screen(() => {
    const base = fresh ? freshAnswers(LAPTOP) : filledAnswers(LAPTOP);
    return room({ at: { panel }, answers: { ...base, ...answers(LAPTOP) }, live: true, width: windowSize().width, height: windowSize().height });
  });
}

/** Continue: `panel` slides in from the right as the wire's light grows to its step. */
export function onward(panel: Panel, sample: () => HTMLElement): Move {
  const index = PLACE[panel];
  const tracks: NonEmpty<Track<"emphasized_decel"> | Track<"standard">> = [
    { part: "slide", on: "the page", target: ".k-step-room", property: "--step-slide", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "fade", on: "the page", target: ".k-step-room", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "wire", on: "the rail", target: ".k-rail-wire-lit", property: "--wire-lit", from: Math.max(0, index - 1), to: index, duration: "long", curve: "emphasized_decel" },
    { part: "symbol", on: "the step arrived at", target: `.k-rail-steps > .k-step:nth-child(${index + 1}) .k-step-bead`, property: "--bead-glow", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ];
  return {
    name: "Continue: this screen arrives",
    what: "It slides `space.s32` in from the right and fades in over `long`; the wire's light grows to its step.",
    tracks,
    reduced: [{ part: "fade", on: "the page", target: ".k-step-room", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
    back: "cuts",
    sample,
  };
}

/** Enter refused: the reason Continue waits for flashes `error` and settles. */
export function refused(sample: () => HTMLElement): Move {
  return {
    name: "Enter refused",
    what: "Enter while something blocks it: the reason burns `error` at once and settles over `slow`.",
    tracks: [{ part: "reason", on: "the blocker", target: ".k-step-blocker", property: "--nudge", from: 1, to: 0, duration: "slow", curve: "standard_accel" }],
    reduced: "same",
    back: "cuts",
    sample,
  };
}

/** The panel's own measures, every screen in it the same. */
export const PANEL_MEASURES: readonly Measure[] = [
  { part: "Panel from the screen's edge", token: "space-s32" },
  { part: "Panel's corners", token: "radius-surface" },
  { part: "Rail's width", token: "size-rail" },
  { part: "Page's sides", token: "space-s32" },
  { part: "Header → what it asks", token: "space-s24" },
  { part: "Body → footer band", token: "space-s16" },
  { part: "Band's buttons", token: "size-control_md" },
];
