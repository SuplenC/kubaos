// What the pages of Kōbō's questions share: the room drawn at a question, the moves between them
// (the question left sliding off, the next one in as its step on the rail lights, the blocker's
// flash), and their measures.

import { type Measure, type Move, type NonEmpty, screen, type Track } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { type Answers, filledAnswers, freshAnswers, type Machine } from "./kobo-parts.ts";
import { LAPTOP, type Panel, room, type RoomOptions } from "./kobo-room.ts";

/** Each screen's place in the rail of the sample laptop, which has no cable in. */
export const PLACE: Record<Panel, number> = { language: 0, network: 1, you: 2, disk: 3, review: 4, details: 5, failed: 5 };

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

/**
 * Continue onto `panel`: the question left slides off to the left and fades over `short`; as it
 * goes, `panel` slides in from the right and fades in over `long`, its footer band rising a `micro`
 * after it; its step's symbol lights as it arrives.
 */
/**
 * The page turning in Kōbō: the page left slides off and fades over `short`; as it goes, the next
 * slides in and fades in over `long`, its footer band, if it has one, rising a `micro` after it.
 */
export function turning(band: boolean): NonEmpty<Track<"emphasized_accel"> | Track<"emphasized_decel">> {
  return [
    { part: "the question left", on: "the page", target: ".k-step-room[data-leaving]", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: "its slide off", on: "the page", target: ".k-step-room[data-leaving]", property: "--step-slide", from: 0, to: 1, duration: "short", curve: "emphasized_accel" },
    { part: "the next question", on: "the page", target: ".k-step-room:not([data-kept])", property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: "its slide in", on: "the page", target: ".k-step-room:not([data-kept])", property: "--step-slide", from: 1, to: 0, duration: "long", cue: "short", curve: "emphasized_decel" },
    ...(band
      ? ([
          { part: "the band", on: "the page", target: ".k-step-room:not([data-kept]) .k-step-footer", property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", "micro"], curve: "emphasized_decel" },
          { part: "the band's rise", on: "the page", target: ".k-step-room:not([data-kept]) .k-step-footer", property: "--step-rise", from: 1, to: 0, duration: "long", cue: ["short", "micro"], curve: "emphasized_decel" },
        ] as const)
      : []),
  ];
}

/** The page turning with reduced motion: the page left goes at once, the next fades in. */
export const TURNING_REDUCED: NonEmpty<Track<"standard"> | Track<"emphasized_decel">> = [
  { part: "the question left", on: "the page", target: ".k-step-room[data-leaving]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" },
  { part: "the next question", on: "the page", target: ".k-step-room:not([data-kept])", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
];

export function onward(panel: Panel, answers: (machine: Machine) => Partial<Answers> = () => ({}), fresh = false): Move {
  const index = PLACE[panel];
  const before = (Object.keys(PLACE) as Panel[]).find((each) => PLACE[each] === index - 1);
  const symbol = `.k-rail-steps > .k-step:nth-child(${index + 1}) .k-step-bead`;
  const tracks: NonEmpty<Track<"emphasized_accel"> | Track<"emphasized_decel"> | Track<"standard">> = [
    // Last look holds its action in its body: it has no band to follow it in.
    ...turning(panel !== "review"),
    { part: "the symbol lit", on: "the step arrived at", target: symbol, property: "--bead-lit", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
    { part: "the symbol's glow", on: "the step arrived at", target: symbol, property: "--bead-glow", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ];
  return {
    name: "Continue: the next question",
    what: "The question left slides off and fades; the next slides in from the right, its band after it; its step lights.",
    tracks,
    reduced: TURNING_REDUCED,
    back: "cuts",
    sample: at(panel, answers, before ? { stepFrom: before } : {}, fresh),
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

/** The questions' own measures, every one of them the same. */
export const PANEL_MEASURES: readonly Measure[] = [
  { part: "Questions from the screen's edge", token: "space-s32" },
  { part: "Rail's width", token: "size-rail" },
  { part: "Page's sides", token: "space-s32" },
  { part: "Header → what it asks", token: "space-s24" },
  { part: "Body → footer band", token: "space-s16" },
  { part: "Band's buttons", token: "size-control_md" },
];
