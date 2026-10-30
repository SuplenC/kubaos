// The theme change and the dark ⇄ light switch, as moves (`docs/DESIGN.md`, "Motion"): one
// sequence on one clock for every light and every colour on every screen. They are power changes,
// so the lights flicker with `neon.strike`'s stutter; a control's own states only ever fade. Their
// players and the toolbar's theme and mode play these same tracks, so what the recipe says is what
// the page does.
//
// The tracks move the numbers `motion.css` names on the scene they play on: `--before`, the lights
// in the old colours; `--recolour`, every colour role along its path; `--day`, night to daylight;
// `--after`, the lights in the new colours. A lit control shows as lit as the lights burn, so as they
// flicker out it flickers down to its rest look, and as they strike it strikes back: a control is
// always lit or at rest, never gone. By day nothing is a light, so a lit control stays lit.

import type { Curve, Duration, Easing, Move, NonEmpty, Track } from "./page.ts";
import type { ColourPath } from "./recolour.ts";

/** What a sequence's tracks move: the scene's root, which every light and colour inside follows. */
export const SCENE = ".page-scene";

const LIGHTS = "every light: a lit control's line, glow and burning label, which go to its rest look as they go out, a switch's and a slider's glow, the active window's border";

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

const STRIKE_CUE: NonEmpty<Duration> = ["medium", "medium", "recolour", "settle"];

/**
 * The theme change: the lights flicker out, every lit control down to its rest look, a dark pause,
 * every colour through grey, a dark beat in the new colours, then the lights strike on and every
 * lit control strikes back. In daylight the same clock runs with nothing to flicker, so only the
 * colours are seen to change.
 */
export function themeChange(name: string, between: Between, sample: () => Element): Move {
  const { from, to } = between;
  return {
    name,
    what: `A new wallpaper's accent or a custom colour, here ${from} to ${to}: a power change, so the lights flicker. A control is always lit or at rest, in this sequence as in every other. Every light on every screen flickers out together over \`medium\` (\`neon.strike\` played backwards), and at each step every lit control goes to its rest look and back: its line and glow out, its label and fill to their rest colours. Everything holds for \`medium\`, every colour role moves through grey over \`recolour\` on \`gather\`, and a beat of \`settle\` passes in the new colours, the controls at rest. Then every light strikes on together over \`strike\` (\`neon.strike\`), and every lit control strikes back to its lit look in the new theme. 2.3 s in all, on one clock: a control lit mid-way waits and strikes with the rest. In light mode the same clock runs with nothing to flicker, a lit control staying lit: only the colours are seen to change, at the same moment.`,
    tracks: [lightsOut("neon.strike.reversed"), colours("through grey", from, to, ["medium", "medium"]), lightsOn(STRIKE_CUE, "neon.strike")],
    reduced: [colours("through grey", from, to, undefined, "instant")],
    unstuttered: [lightsOut("standard"), colours("through grey", from, to, ["medium", "medium"]), lightsOn(STRIKE_CUE, "standard")],
    daylight: [unseen(lightsOut("standard")), colours("through grey", from, to, ["medium", "medium"]), unseen(lightsOn(STRIKE_CUE, "standard"))],
    back: "cuts",
    sample,
  };
}

/**
 * To light: the lit signs flicker out, every lit control down to its rest look, a pause, then the
 * colours and the daylight change on the dawn path, a lit control fading to its daylight lit look
 * as the day comes, and a closing beat. The signs stay off in daylight.
 */
export function toLight(between: Between, sample: () => Element): Move {
  return {
    name: "To light",
    what: `Dark to light, ${between.from} to ${between.to}: a power change. The lit signs flicker out together over \`medium\` (\`neon.strike\` played backwards), every lit control flickering down to its rest look: a control is always lit or at rest, as in every sequence. Everything pauses for \`medium\`, then the colours change on the dawn path over \`recolour\` on \`gather\` as the night turns to daylight, and a lit control fades to its daylight lit look with it, its dark line or the tone's coming as the day does, since by day nothing is a light. A closing beat of \`settle\` passes. About 1.7 s. The signs stay off in daylight.`,
    tracks: [lightsOut("neon.strike.reversed"), colours("dawn", between.from, between.to, ["medium", "medium"]), daylight(0, ["medium", "medium"])],
    tail: "settle",
    reduced: [colours("dawn", between.from, between.to, undefined, "instant"), daylight(0, undefined, "instant")],
    unstuttered: [lightsOut("standard"), colours("dawn", between.from, between.to, ["medium", "medium"]), daylight(0, ["medium", "medium"])],
    back: "cuts",
    sample,
  };
}

/**
 * To dark: a pause, the colours and the night changing on the dawn path, a lit control fading to its
 * rest look as the daylight goes, a beat, then every sign strikes on and every lit control with it.
 */
export function toDark(between: Between, sample: () => Element): Move {
  return {
    name: "To dark",
    what: `Light to dark, ${between.from} to ${between.to}: a power change. After a pause of \`medium\`, the colours change on the dawn path over \`recolour\` on \`gather\` as the daylight goes, and a lit control fades to its rest look with it, its daylight line going as the night comes, since at night it shows lit only while the lights burn. A beat of \`settle\` passes, every control at rest. Then every sign strikes on over \`strike\` (\`neon.strike\`), and every lit control strikes to its lit look: the core or the tube inside its edge, its glow and its lit label. A control is always lit or at rest, as in every sequence. About 2 s.`,
    tracks: [daylight(1, ["medium"]), colours("dawn", between.from, between.to, ["medium"]), lightsOn(["medium", "recolour", "settle"], "neon.strike")],
    reduced: [colours("dawn", between.from, between.to, undefined, "instant"), daylight(1, undefined, "instant"), lightsOn(undefined, "linear", "instant")],
    unstuttered: [daylight(1, ["medium"]), colours("dawn", between.from, between.to, ["medium"]), lightsOn(["medium", "recolour", "settle"], "standard")],
    back: "cuts",
    sample,
  };
}
