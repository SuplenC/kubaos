// SidebarPage's page turns as moves, for every page that shows the template: Settings' too. A turn
// moves the page, its title with it, as a panel swapped for its twin moves: the page shown leaves
// `space.s24` toward one side and fades over `micro` on `emphasized_accel`; the next is drawn as it
// has gone and comes in `space.s24` from the other side, fading in over `short` on `emphasized_decel`. Along the sidebar (`y`) a section lower in
// the list rises in from below, Home the top of the list; deeper (`x`) a page of a section's own comes
// in from the end, and Back turns the other way. With reduced motion the page left goes at once and
// the next fades in over `short` on `standard`.

import type { Easing, Move, NonEmpty, Track } from "../../site/page.ts";

/** The page a turn moves: the section's or Home's, its title with it. */
const PAGE = ".k-sp-rise > .k-page";

/** A turn's tracks, the next page coming from the `sign` side. */
function tracks(sign: 1 | -1): NonEmpty<Track<Easing>> {
  return [
    { part: "the page leaving", on: "the page shown", target: PAGE, property: "--page-shift", from: 0, to: -sign, duration: "micro", curve: "emphasized_accel" },
    { part: "its fade out", on: "the page shown", target: PAGE, property: "--page-shown", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" },
    { part: "the next page arriving", on: "the next page", target: PAGE, property: "--page-shift", from: sign, to: 0, duration: "short", cue: "micro", curve: "emphasized_decel" },
    { part: "its fade in", on: "the next page", target: PAGE, property: "--page-shown", from: 0, to: 1, duration: "short", cue: "micro", curve: "emphasized_decel" },
  ];
}

/** Reduced motion's turn: the page left gone at once, the next fading in. */
const REDUCED: NonEmpty<Track<Easing>> = [{ part: "the next page", on: "the next page", target: PAGE, property: "--page-shown", from: 0, to: 1, duration: "short", curve: "standard" }];

/** `window`, a sample, turning sideways: its page's room set to the `x` axis. */
function sideways(window: HTMLElement): HTMLElement {
  window.querySelector<HTMLElement>(".k-sp-rise")?.setAttribute("data-axis", "x");
  return window;
}

/** The samples a page's turns play on: a section's page, and Home. */
export interface TurnSamples {
  readonly section: () => HTMLElement;
  readonly home: () => HTMLElement;
  /** A page landed on from search, its row's highlight drawn on. */
  readonly landed: () => HTMLElement;
}

/** The two sideways turns alone, into a page of a section's own and back out: a section page's. */
export function deeperMoves(section: () => HTMLElement): Move[] {
  return turnMoves({ section, home: section, landed: section }).filter((move) => move.name === "Into a page of its own" || move.name === "Back out");
}

/** The template's page turns, played on `samples`. */
export function turnMoves(samples: TurnSamples): Move[] {
  const turn = (name: string, what: string, sign: 1 | -1, sample: () => HTMLElement): Move => ({ name, what, tracks: tracks(sign), reduced: REDUCED, back: "cuts", sample });
  return [
    turn("To a section below", "The page leaves upward `space.s24`, fading; the next rises in from below.", 1, samples.section),
    turn("To a section above", "The page leaves downward `space.s24`, fading; the next comes down from above.", -1, samples.section),
    turn("From Home", "Home is the list's top: it leaves upward, the section rising in.", 1, samples.home),
    turn("Into a page of its own", "Deeper: the page leaves toward the start, the next comes in from the end.", 1, () => sideways(samples.section())),
    turn("Back out", "Back: the page leaves toward the end, the section comes in from the start.", -1, () => sideways(samples.section())),
    {
      name: "Landed",
      what: "Search or the Command menu: the page turns in, then the landed highlight fades over `slow`.",
      tracks: [
        ...tracks(1),
        { part: "the landed highlight", on: "the row landed on", target: ".k-list-row[data-landed]", property: "--landed", from: 1, to: 0, duration: "slow", cue: ["micro", "short"], curve: "standard" },
      ] as unknown as NonEmpty<Track<Easing>>,
      reduced: [...REDUCED, { part: "the landed highlight", on: "the row landed on", target: ".k-list-row[data-landed]", property: "--landed", from: 1, to: 0, duration: "slow", cue: "short", curve: "standard" }],
      back: "cuts",
      sample: samples.landed,
    },
  ];
}
