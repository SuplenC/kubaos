// The theme change and the dark ⇄ light switch, as moves (`docs/DESIGN.md`, "Motion"): one
// sequence on one clock for every light and every colour on every screen. Their players and the
// toolbar's theme and mode play these same tracks, so what the recipe says is what the page does.
//
// The tracks move the numbers `motion.css` names on the scene they play on: `--before`, the lights
// in the old colours; `--recolour`, every colour role along its path; `--day`, night to daylight;
// `--after`, the lights in the new colours.

import type { Duration, Easing, Move, NonEmpty, Track } from "./page.ts";
import type { ColourPath } from "./recolour.ts";

/** What a sequence's tracks move: the scene's root, which every light and colour inside follows. */
export const SCENE = ".page-scene";

const LIGHTS = "every light: the focus tubes, their cores and glows, a switch's and a slider's glow, the active window's border";

/** The lights in the old colours going out at the start, struck backwards or faded. */
function lightsOut<C extends Easing | "neon.strike.reversed">(curve: C): Track<C> {
  return { part: "lights out", on: LIGHTS, target: SCENE, property: "--before", from: 1, to: 0, duration: "medium", curve };
}

/** The lights in the new colours striking on, from `cue`, struck or faded, or on at once. */
function lightsOn<C extends Easing | "neon.strike">(cue: NonEmpty<Duration> | undefined, curve: C, duration: Duration = "strike"): Track<C> {
  return { part: "lights on", on: LIGHTS, target: SCENE, property: "--after", from: 0, to: 1, duration, ...(cue ? { cue } : {}), curve };
}

/** Every colour role along `path`, from `from`'s palette to `to`'s, over `recolour` or at once. */
function colours(path: ColourPath, from: string, to: string, cue?: NonEmpty<Duration>, duration: Duration = "recolour"): Track<Easing> {
  return {
    part: "colours",
    on: "every colour role",
    target: SCENE,
    property: "--recolour",
    from: 0,
    to: 1,
    duration,
    ...(cue ? { cue } : {}),
    curve: duration === "instant" ? "linear" : "gather",
    colours: { path, from, to },
  };
}

/** Night turning to daylight, or back, with the colours: the lit neon becomes unlit glass. */
function daylight(from: 0 | 1, cue?: NonEmpty<Duration>, duration: Duration = "recolour"): Track<Easing> {
  return {
    part: from === 0 ? "daylight" : "nightfall",
    on: "every focus tube: lit at night, unlit glass in daylight",
    target: SCENE,
    property: "--day",
    from,
    to: from === 0 ? 1 : 0,
    duration,
    ...(cue ? { cue } : {}),
    curve: duration === "instant" ? "linear" : "gather",
  };
}

/** The palettes a sample goes between, by name, as its recipe says them. */
export interface Between {
  readonly from: string;
  readonly to: string;
}

/**
 * The theme change: the lights flicker out, a dark pause, every colour through grey, a dark beat in
 * the new colours, then the lights strike on. In daylight the same clock runs with nothing lit, so
 * only the colours are seen to change.
 */
export function themeChange(name: string, between: Between, sample: () => Element): Move {
  const { from, to } = between;
  return {
    name,
    what: `A new wallpaper's accent or a custom colour, here ${from} to ${to}: every light on every screen flickers out together over \`medium\` (\`neon.strike\` played backwards), everything holds dark for \`medium\`, every colour role moves through grey over \`recolour\` on \`gather\`, everything holds dark in the new colours for \`settle\`, then every light strikes on together over \`strike\`, its core half way to white from the new tone. 2.3 s in all, on one clock: a control focused mid-way waits and strikes with the rest. In light mode the same clock runs, but the neon is off: only the colours are seen to change, at the same moment.`,
    tracks: [
      lightsOut("neon.strike.reversed"),
      colours("through grey", from, to, ["medium", "medium"]),
      lightsOn(["medium", "medium", "recolour", "settle"], "neon.strike"),
    ],
    reduced: [colours("through grey", from, to, undefined, "instant")],
    unstuttered: [
      lightsOut("standard"),
      colours("through grey", from, to, ["medium", "medium"]),
      lightsOn(["medium", "medium", "recolour", "settle"], "standard"),
    ],
    back: "cuts",
    sample,
  };
}

/**
 * To light: the lit signs flicker out, a dark pause, the colours and the daylight change on the
 * dawn path, then a closing beat. The signs stay off in daylight: unlit glass.
 */
export function toLight(between: Between, sample: () => Element): Move {
  return {
    name: "To light",
    what: `Dark to light, ${between.from} to ${between.to}: the lit signs flicker out together over \`medium\`, everything holds dark for \`medium\`, the colours change on the dawn path over \`recolour\` on \`gather\` while the night turns to daylight and the unlit glass of every focus tube comes up, then a closing beat of \`settle\`. About 1.7 s. The signs stay off in daylight.`,
    tracks: [
      lightsOut("neon.strike.reversed"),
      colours("dawn", between.from, between.to, ["medium", "medium"]),
      daylight(0, ["medium", "medium"]),
    ],
    tail: "settle",
    reduced: [colours("dawn", between.from, between.to, undefined, "instant"), daylight(0, undefined, "instant")],
    unstuttered: [
      lightsOut("standard"),
      colours("dawn", between.from, between.to, ["medium", "medium"]),
      daylight(0, ["medium", "medium"]),
    ],
    back: "cuts",
    sample,
  };
}

/**
 * To dark: the unlit glass holds, the colours and the night change on the dawn path, a beat, then
 * every sign strikes on.
 */
export function toDark(between: Between, sample: () => Element): Move {
  return {
    name: "To dark",
    what: `Light to dark, ${between.from} to ${between.to}: the unlit tubes stay as they are for \`medium\`, the colours change on the dawn path over \`recolour\` on \`gather\` while the daylight goes and the glass fades, everything holds for \`settle\`, then every sign strikes on over \`strike\`, its core half way to white, with its full glow. About 2 s.`,
    tracks: [
      daylight(1, ["medium"]),
      colours("dawn", between.from, between.to, ["medium"]),
      lightsOn(["medium", "recolour", "settle"], "neon.strike"),
    ],
    reduced: [colours("dawn", between.from, between.to, undefined, "instant"), daylight(1, undefined, "instant"), lightsOn(undefined, "linear", "instant")],
    unstuttered: [
      daylight(1, ["medium"]),
      colours("dawn", between.from, between.to, ["medium"]),
      lightsOn(["medium", "recolour", "settle"], "standard"),
    ],
    back: "cuts",
    sample,
  };
}
