// The theme change and the dark ⇄ light switch, as moves (`docs/DESIGN.md`, "Motion"): one
// sequence on one clock for every light and every colour on every screen. They are power changes,
// so the lights flicker through `neon.strike`'s steps; a control's own states only ever fade. Their
// players and the toolbar's theme and mode play these same tracks, so what the recipe says is what
// the page does.
//
// The tracks move the numbers `motion.css` names on the scene they play on: `--before`, the lights
// in the old colours; `--recolour`, every colour role along its path; `--day`, night to daylight;
// `--after`, the lights in the new colours. A lit control is lit with the signs on while the lights
// burn and with them off while they're out, so as they flicker out it flickers down to its
// signs-off look, its line turning to glass and staying, and as they strike it strikes back: a lit
// control is lit with the signs on or off, never gone. By day the signs are off, so a lit control
// shows its signs-off look, in daylight colours.

import type { Curve, Duration, Easing, Move, NonEmpty, Track } from "./page.ts";
import type { ColourPath } from "./recolour.ts";

/** What a sequence's tracks move: the scene's root, which every light and colour inside follows. */
export const SCENE = ".page-scene";

const LIGHTS = "every light: a lit control's line, glow and burning label, which go to its signs-off look as they go out, the line turning to glass, a switch's and a slider's glow, the active window's border";

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

/** Night turning to daylight, or back, with the colours, while every light is out. */
function daylight(from: 0 | 1, cue?: NonEmpty<Duration>, duration: Duration = "recolour"): Track<Easing> {
  return {
    part: from === 0 ? "daylight" : "nightfall",
    on: "every light: free to burn at night; in daylight the neon and its glow can't light",
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

/** The theme change's lights going out and coming back, unseen in daylight, where the neon is off. */
function unseen<C extends Curve>(track: Track<C>): Track<C> {
  return { ...track, on: `${track.on}, unseen: daylight puts the neon out` };
}

/** What every sequence holds to, in its words. */

const STRIKE_CUE: NonEmpty<Duration> = ["medium", "medium", "recolour", "settle"];

/**
 * The theme change: the lights flicker out, every lit control down to its signs-off look, a dark
 * pause, every colour through grey, a dark beat in the new colours, then the lights strike on and
 * every lit control strikes back. In daylight the same clock runs with nothing to flicker, so only
 * the colours are seen to change.
 */
export function themeChange(name: string, between: Between, sample: () => Element): Move {
  const { from, to } = between;
  return {
    name,
    what: `A power change, ${from} to ${to}: the lights flicker out, the colours pass through grey, the lights strike on.`,
    tracks: [lightsOut("neon.strike.reversed"), colours("through grey", from, to, ["medium", "medium"]), lightsOn(STRIKE_CUE, "neon.strike")],
    reduced: [colours("through grey", from, to, undefined, "instant")],
    unflickered: [lightsOut("standard"), colours("through grey", from, to, ["medium", "medium"]), lightsOn(STRIKE_CUE, "standard")],
    daylight: [unseen(lightsOut("standard")), colours("through grey", from, to, ["medium", "medium"]), unseen(lightsOn(STRIKE_CUE, "standard"))],
    back: "cuts",
    sample,
  };
}

/**
 * To light: the lit signs flicker out, every lit control down to its signs-off look, a pause, then
 * the colours and the daylight change on the dawn path, the signs-off look becoming the daylight
 * lit look as the day comes, and a closing beat. The signs stay off in daylight.
 */
export function toLight(between: Between, sample: () => Element): Move {
  return {
    name: "To light",
    what: `Dark to light, ${between.from} to ${between.to}: the signs flicker out, then the colours dawn into daylight.`,
    tracks: [lightsOut("neon.strike.reversed"), colours("dawn", between.from, between.to, ["medium", "medium"]), daylight(0, ["medium", "medium"])],
    tail: "settle",
    reduced: [colours("dawn", between.from, between.to, undefined, "instant"), daylight(0, undefined, "instant")],
    unflickered: [lightsOut("standard"), colours("dawn", between.from, between.to, ["medium", "medium"]), daylight(0, ["medium", "medium"])],
    back: "cuts",
    sample,
  };
}

/**
 * To dark: a pause, the colours and the night changing on the dawn path, a lit control holding its
 * daylight lit look, its glass following the palette, a beat, then every sign strikes on and every
 * lit control with it.
 */
export function toDark(between: Between, sample: () => Element): Move {
  return {
    name: "To dark",
    what: `Light to dark, ${between.from} to ${between.to}: the colours fall to night, then every sign strikes on.`,
    tracks: [daylight(1, ["medium"]), colours("dawn", between.from, between.to, ["medium"]), lightsOn(["medium", "recolour", "settle"], "neon.strike")],
    reduced: [colours("dawn", between.from, between.to, undefined, "instant"), daylight(1, undefined, "instant"), lightsOn(undefined, "linear", "instant")],
    unflickered: [daylight(1, ["medium"]), colours("dawn", between.from, between.to, ["medium"]), lightsOn(["medium", "recolour", "settle"], "standard")],
    back: "cuts",
    sample,
  };
}
