// The Lock page: `docs/design/components/lock.md`. The lock screen at rest, the wallpaper and its
// clock with the city's cut-out in front of the digits, and awake, the frame closed in and the
// password's sheet up, always in the wallpaper's own night scheme.

import { doesntApply, type Move, nonEmpty, page, type Track } from "../../site/page.ts";
import type { Duration, Easing } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { DATES, kickerFor, LOCK_ERROR, liveLock, PURPLE_HOUR, SHIBUYA, still, TIMES, TOWER, twelveHour, twoScreens } from "./locks.ts";

/** A lock drawn still, wrapped so a move can reach its root: the frame's number is set there. */
function held(phase: "rest" | "awake" | "desktop", wallpaper = PURPLE_HOUR): () => HTMLElement {
  return () => {
    const box = document.createElement("span");
    box.className = "k-lock-held";
    box.append(still({ phase, wallpaper }));
    return box;
  };
}

const FRAME = "the frame";
const BLUR = "the picture's blur";
const DIM = "the dim";
const CLOCK = "the clock";
const CUTOUT = "the cut-out";
const SHEET = "the sheet";
const KICKER_PART = "the kicker";
const SHADE = "the shade";
const WINDOWS = "the windows";
const BAR = "the bar's pills";
const BAR_TIME = "the bar's time";
const TOP = "the frame's top edge";
const GLASS = "the frame's glass";
const ON = "the lock on Shinjuku";
const ON_SHIBUYA = "the lock on Shibuya";
const BLURRED = ".k-lock-blur, .k-lock-cutout";

type Eased = Track<Easing>;

const WAKES: Move = {
  name: "Waking",
  what: "A key, a press or a touch of the reader, on the screen with the pointer: the frame closes in from past the screen's edges to `space.s8` all round over `long` on `emphasized_decel`, its inner corners growing with it; the picture blurs to `size.lock_blur` on the same curve, and dims to `opacity.panel_scrim` over `medium` on `standard`, as a panel's desktop does. The cut-out fades over `short` on `standard`, so the clock comes out from behind the city, and the clock lifts over `long` on `emphasized_decel` from its rest place to `space.s48` under the frame's top edge, its digits going from `type.clock_rest` to `type.clock`, on whole device pixels. The sheet rises from the bottom edge as a Sheet opens, over `long` on `emphasized_decel`. With reduce motion the lock fades to awake over `short`: the frame, the sheet and the clock in their awake places fade in, the blur and the dim fade in, and the cut-out is gone at once.",
  tracks: [
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the glass", on: ON, target: ".k-screen-glass", property: "opacity", from: 0, to: { opacity: "frame" }, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: held("rest"),
};

const RESTS: Move = {
  name: "Going back to rest",
  what: "Esc, or `duration.lock_idle` (20 s) with no key, press or touch: the sheet goes back into the bottom edge as a Sheet closes and the frame opens out past the screen's edges, both over `medium` on `emphasized_accel`; the dim lifts over `medium` on `standard` and the blur clears over `long` on `standard`. The clock settles back to its rest place and size over `long` on `emphasized_decel`, and the cut-out fades back in over `short`, from `medium` on, so the city is in front again as the clock lands. The field empties. With reduce motion the awake lock fades out over `short` and the rest clock and the cut-out fade in over the next `short`.",
  tracks: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "long", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ],
  reduced: reducedRest(),
  back: "cuts",
  sample: held("awake"),
};

function reducedRest(): [Eased, ...Eased[]] {
  return [
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the glass", on: ON, target: ".k-screen-glass", property: "opacity", from: { opacity: "frame" }, to: 0, duration: "short", curve: "standard" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ];
}

/**
 * Super+L: the desktop's still leaves and the bar's time hands over to the lock's clock where it
 * stands, which then grows out of the bar to the rest clock, and the city steps in front of it.
 */
function locking(on: string, cutout: boolean): [Eased, ...Eased[]] {
  const landed: Eased[] = [
    { part: KICKER_PART, on, target: ".k-lock-kicker", property: "opacity", from: 0, to: 1, duration: "short", cue: ["micro", "medium"], curve: "standard" },
    { part: SHADE, on, target: ".k-lock-shade", property: "opacity", from: 0, to: 1, duration: "short", cue: ["micro", "medium"], curve: "standard" },
    { part: "the clock's depth", on, target: ".k-lock-clock", property: "--above", from: 1, to: 0, duration: "instant", cue: ["micro", "long"], curve: "standard" },
    ...(cutout ? [{ part: CUTOUT, on, target: ".k-lock-cutout", property: "opacity", from: 0, to: 1, duration: "short", cue: ["micro", "long"], curve: "standard" } satisfies Eased] : []),
  ];
  return [
    { part: WINDOWS, on, target: ".k-lock-windows", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the windows' size", on, target: ".k-lock-windows", property: "scale", from: 1, to: { scale: "behind" }, duration: "medium", curve: "emphasized_accel" },
    { part: BAR, on, target: ".k-bar", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: BAR_TIME, on, target: ".k-lock-bar-time", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: CLOCK, on, target: ".k-lock-clock", property: "--dock", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: TOP, on, target: ".k-lock", property: "--barred", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: GLASS, on, target: ".k-screen-glass", property: "opacity", from: { opacity: "frame" }, to: 0, duration: "long", cue: "short", curve: "emphasized_decel" },
    ...landed,
  ];
}

/** Super+L with reduce motion: the desktop's still fades out as the rest clock fades in, over `short`. */
function lockingReduced(on: string, cutout: boolean): [Eased, ...Eased[]] {
  return [
    { part: WINDOWS, on, target: ".k-lock-windows", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BAR, on, target: ".k-bar", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: GLASS, on, target: ".k-screen-glass", property: "opacity", from: { opacity: "frame" }, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on, target: ".k-lock-clock", property: "--dock", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's depth", on, target: ".k-lock-clock", property: "--above", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: KICKER_PART, on, target: ".k-lock-kicker", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: SHADE, on, target: ".k-lock-shade", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    ...(cutout ? [{ part: CUTOUT, on, target: ".k-lock-cutout", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" } satisfies Eased] : []),
    { part: TOP, on, target: ".k-lock", property: "--barred", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
  ];
}

const LOCKS: Move = {
  name: "Locking",
  what: "Super+L. The session locks in the first frame; what follows plays on a still of the desktop the lock captured then, drawn inside the lock's own surface, so the lock and the desktop are one surface. The windows fade and shrink to `scale.behind` round the middle of the screen over `medium` on `emphasized_accel`, and the bar's pills fade out with them, riding up out of the screen in the middle of the frame's top edge as it thins. The bar's clock grows out of the bar: its time, Geist Regular, hands over to the lock's clock, Geist Light, where it stands over `micro`, the strokes thinning in place; from `micro` on the clock grows from the bar's time in `type.body` to `type.clock_rest` at its rest place over `long` on `emphasized_decel`, on whole device pixels, its digits taking the picture's tint, and the kicker and its shade fade in above it over `short` from `micro` and `medium` on. Meanwhile, from `micro` on, the frame's top edge thins from the bar's 40 to `space.s8` on the same curve, and from `short` on the whole frame fades out to the bare picture over `long` on `emphasized_decel`. The clock flies above the frame; once it has landed it drops behind the picture, and the cut-out fades in over its digits over `short`: the city steps in front. A lid close or an idle suspend skips all of it and goes straight to rest. With reduce motion the desktop fades out as the rest clock, its kicker, its shade and the cut-out fade in, over `short`.",
  tracks: locking(ON, true),
  reduced: lockingReduced(ON, true),
  back: "cuts",
  sample: held("desktop"),
};

const LOCKS_BARE: Move = {
  name: "Locking, without a cut-out",
  what: "The same on a picture with no cut-out, Shibuya's or one of your own: once the clock has landed it drops behind nothing, and no cut-out fades in: the tinted digits stand on the bare picture, only the kicker shaded.",
  tracks: locking(ON_SHIBUYA, false),
  reduced: lockingReduced(ON_SHIBUYA, false),
  back: "cuts",
  sample: held("desktop", SHIBUYA),
};

/** The clock's flight into the bar and the desktop coming up round it, from `cue` on. */
function intoTheBar(on: string, cue: Eased["cue"]): Eased[] {
  const at = (...more: Duration[]): Eased["cue"] => {
    const before: Duration[] = cue === undefined ? [] : typeof cue === "string" ? [cue] : [...cue];
    const all = [...before, ...more];
    return all.length === 0 ? undefined : (all as [Duration, ...Duration[]]);
  };
  const timed = (track: Omit<Eased, "cue">, when: Eased["cue"]): Eased => (when === undefined ? track : { ...track, cue: when });
  return [
    timed({ part: "the clock's depth", on, target: ".k-lock-clock", property: "--above", from: 0, to: 1, duration: "instant", curve: "standard" }, at()),
    timed({ part: CLOCK, on, target: ".k-lock-clock", property: "--dock", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: TOP, on, target: ".k-lock", property: "--barred", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: BAR, on, target: ".k-bar", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: BAR_TIME, on, target: ".k-lock-bar-time", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" }, at("long")),
    timed({ part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" }, at("long", "micro")),
  ];
}

const UNLOCKS: Move = {
  name: "Unlocking with the password",
  what: "The right password, from awake: about 0.8 s in all. The field's `lock` turns to `lock_open` in `primary` over `short`. The sheet sinks into the bottom edge as a Sheet closes, over `medium` on `emphasized_accel`. From `short` on the clock flies from `type.clock` into the bar's clock over `long` on `emphasized_decel`, above the frame, its kicker fading over `short`, and lands on the bar's time on whole device pixels: its left edge, its width, its baseline and `type.body`'s size: the Light's letters spread as it lands by what the bar's Regular is wider, an even share after each, so each figure stands within a fifth of a logical pixel of the bar's. Once it has landed it hands over to the bar's own time over `micro`: the bar's Geist Regular fades in on top of the clock's Geist Light, which then goes, so the strokes thicken in place and nothing else moves. Meanwhile the frame's top edge grows from `space.s8` into the bar and the bar's pills fade in with it, on the same curve. From `medium` on the picture's blur and dim clear to the desktop's wallpaper and the windows settle in from `scale.behind`, over `long`. The motion plays on the still the lock captured as it locked; the session unlocks as it ends, and the real windows take the still's place. With reduce motion the awake lock fades to the desktop over `short`.",
  tracks: [
    { part: "the field's lock", on: ON, target: ".k-lock-shut", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the field's open lock", on: ON, target: ".k-lock-open", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
    { part: KICKER_PART, on: ON, target: ".k-lock-kicker", property: "opacity", from: 1, to: 0, duration: "short", cue: "short", curve: "standard" },
    ...intoTheBar(ON, "short"),
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "long", cue: "medium", curve: "standard" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "long", cue: "medium", curve: "standard" },
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: TOP, on: ON, target: ".k-lock", property: "--barred", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: BAR, on: ON, target: ".k-bar", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "instant", curve: "standard" },
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: held("awake"),
};

const TOUCHES: Move = {
  name: "Unlocking with the fingerprint",
  what: "A known finger, from rest: the cut-out and the kicker fade over `micro`, so the clock comes out from behind the city and rises above the frame before it reaches the frame's top edge, and the rest clock flies straight from `type.clock_rest` into the bar's clock over `long` on `emphasized_decel`, its digits going from the picture's tint to `on_surface`, landing and handing over to the bar's time as the password's unlock does. The frame closes in from past the screen's edges with its top edge growing into the bar, the bar's pills fading in, on the same curve, and from `short` on the windows settle in from `scale.behind` over `long`. With reduce motion the rest lock fades to the desktop over `short`.",
  tracks: [
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: KICKER_PART, on: ON, target: ".k-lock-kicker", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    ...intoTheBar(ON, undefined).map((track) => (track.property === "--above" ? { ...track, cue: "micro" as const } : track)),
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: TOP, on: ON, target: ".k-lock", property: "--barred", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: GLASS, on: ON, target: ".k-screen-glass", property: "opacity", from: 0, to: { opacity: "frame" }, duration: "short", curve: "standard" },
    { part: BAR, on: ON, target: ".k-bar", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "instant", curve: "standard" },
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: held("rest"),
};

const SHAKE: Move = {
  name: "A wrong password",
  what: "The field shakes its head: `space.s12` one way, `space.s8` back past its place, `space.s4` the first way, each over `micro` on `standard`, then home over `short` on `standard_decel`, a whole device pixel at a time; the field empties, turns Invalid, and the line under it says what went wrong in place of the reader's line. With reduce motion it stays still and only the line says so.",
  tracks: [
    { part: "the field's shake", on: ON, target: ".k-lock-field", property: "--jolt", from: 0, to: 3, duration: "micro", curve: "standard" },
    { part: "the field's shake", on: ON, target: ".k-lock-field", property: "--jolt", from: 3, to: -2, duration: "micro", cue: "micro", curve: "standard" },
    { part: "the field's shake", on: ON, target: ".k-lock-field", property: "--jolt", from: -2, to: 1, duration: "micro", cue: ["micro", "micro"], curve: "standard" },
    { part: "the field's shake", on: ON, target: ".k-lock-field", property: "--jolt", from: 1, to: 0, duration: "short", cue: ["micro", "micro", "micro"], curve: "standard_decel" },
  ],
  reduced: [{ part: "the field's shake", on: ON, target: ".k-lock-field", property: "--jolt", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => still({ phase: "awake", moment: "error" }),
};

type Shown = "rest" | "loading" | "error";

function reflow(): HTMLElement {
  return liveLock(PURPLE_HOUR, windowSize());
}

export default page({
  level: "templates",
  slug: "lock",
  title: "Lock",
  summary: "The lock screen: at rest the wallpaper and a clock the city stands in front of; awake, the frame closed in and the password's sheet up. Always in the wallpaper's own night scheme.",
  live: () => liveLock(),
  description: [
    "QML `Lock`, a template of the shell's alone: only the shell locks. Click into it and type, or press it: it wakes, and what you type lands in the field. Enter with a password shakes its head and says so; Esc, or 20 s with nothing done, puts it back to rest.",
    "At rest the lock is the wallpaper and its clock, and nothing else: no frame, no bar, no field. The digits are `type.clock_rest`, Geist Light at 384, about 280 tall, under a kicker line `space.s16` above their tops: where and which day in `accent`, `primary` (東京・月曜日), then ` · ` and the date in `body_strong`, `on_surface_variant`. The kicker and the digits are each centred on the clock's middle, on whole device pixels with the odd one after. The digits are a light tint of the picture behind them, so they read as part of it: the hue the picture shows most strongly under them (each pixel's OKLCH hue counted by its chroma squared, so the glow of a sky or a street's signs decide it, not its haze), at that chroma times 1.2, at lightness 0.84. They are large enough to read without a contrast of their own, so nothing shades them. Every screen of a setup shows rest, each with its own clock.",
    "Each wallpaper says where its clock stands: the middle of the digits' baseline as shares of the picture, so the clock keeps its place against the city however a screen crops the picture, kept `space.s32` inside the screen's edges. A wallpaper with a foreground cut-out (an alpha picture of the city, the same size as the wallpaper) has it drawn over the digits, so the towers stand in front of the clock: on Shinjuku the skyline covers the digits' feet, on Tokyo Tower the spire crosses the last digit. The place keeps the kicker clear of the cut-out. A picture of your own has no cut-out, and the clock stands in front of it.",
    "Behind the kicker, small text that keeps 4.5:1, lies a shade of `surface` tight to it: a capsule `space.s16` past it across and `space.s12` down, blurred by `space.s8` so it has no edge and no shape of its own, at the least opacity, in hundredths, that keeps its words at 4.5:1 over the brightest of the picture under them (its 98th percentile by luminance, read off the picture itself), raised by what the blur leaves at their corners so they hold it where it is weakest. A dark sky like Tokyo Tower's gets next to none, Shibuya's street of signs a lot. The digits have none, with a cut-out or without.",
    "Awake, on the screen with the pointer only, the shell's frame closes in round the picture, `surface` at `opacity.frame`, `space.s8` all round: the top edge is as thin as the others, and grows to the bar's height only as the lock gives way to the desktop. The picture blurs to `size.lock_blur` (8) and dims to `scrim` at `opacity.panel_scrim`. The clock lifts out from behind the cut-out, which fades, and settles at `type.clock` (80), centred, its kicker's top `space.s48` under the frame's top edge, its digits going from the picture's tint to `on_surface` as it lifts; the kicker's shade follows it, worked out the same way over the blurred, dimmed picture.",
    "The lock and the desktop are one surface. Super+L locks the session in its first frame and captures a still of the desktop then; the lock draws that still inside its own surface (the windows, and the bar's pills on the frame) and plays its locking on it: the windows go, and the bar's clock grows out of the bar into the rest clock. Unlocking plays the same way back, the clock flying into the bar's clock, and the session unlocks as it ends. A lid close or an idle suspend skips the motion and goes straight to rest.",
    "A Sheet rises from the middle of the bottom edge, curving into the frame through concave fillets of `radius.surface`, as the wallpaper picker does, `size.panel` (384) wide with a PanelPage's padding: `space.s16` at its top and foot, the container's 16 at its sides. It holds the md `Field` for the password, its lock symbol leading in the field's icon colour and lit with the keyboard in it; `space.s4` under it, as a field's note is, the line led by the `fingerprint` symbol in `primary`, Or touch the fingerprint reader; and `space.s16` under that the key hints, ↵ Unlock and Esc Rest. No avatar, no initial, no emoji: only KubaOS Symbols.",
    "The lock ignores light and dark. It always draws in the night scheme matugen derives from its wallpaper (the same `scheme-fidelity` dot-theme uses, in dark), and its derived roles from those: Shinjuku's purple hour gives a lavender primary, Tokyo Tower's blue hour a pale blue one, Shibuya's rain a teal one. So the lock is the same in either mode, and its kicker and its sheet keep their contrast over the night picture.",
  ],
  variants: [
    { name: "At rest, with a cut-out", for: "Shinjuku at purple hour: the skyline's towers stand in front of the digits' feet; the digits stand in the open sky in a pale pink drawn from the horizon behind them, and only the kicker has a soft shade behind it.", sample: () => still({ phase: "rest", wallpaper: PURPLE_HOUR }) },
    { name: "At rest, with a cut-out, a dark sky", for: "Tokyo Tower at blue hour: the spire crosses the last digit; the digits are a pale blue drawn from the sky, and the sky is dark enough that the kicker's shade is next to none.", sample: () => still({ phase: "rest", wallpaper: TOWER }) },
    { name: "At rest, without a cut-out", for: "Shibuya on a rainy night, as a picture of your own is: the clock in front, its digits a gold drawn from the lit storefronts behind them over the bare picture, and a soft shade behind the kicker alone, strong enough for 4.5:1 over the signs.", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
    { name: "Awake", for: "The frame closed in, the picture blurred and dimmed, the clock at the top and the password's sheet up from the bottom edge.", sample: () => still({ phase: "awake", wallpaper: PURPLE_HOUR }) },
    { name: "Awake, without a cut-out", for: "The same on Shibuya, in its own night scheme.", sample: () => still({ phase: "awake", wallpaper: SHIBUYA }) },
    { name: "The 12-hour clock", for: "Where the locale or Settings ask for 12 hours: the digits without a meridiem (9:47), which takes its own slot at the kicker's end (· PM), so the digits keep one size and the clock its place.", sample: () => twelveHour("rest") },
    { name: "The 12-hour clock, awake", for: "The same once the clock is at the top.", sample: () => twelveHour("awake") },
    { name: "The desktop's still", for: "What the lock draws as it locks and unlocks: the still of the desktop it captured as the session locked, the frame with the bar's height at the top, the bar's pills and its clock, and the windows.", sample: () => still({ phase: "desktop" }) },
    { name: "Two screens", for: "An external screen over a laptop's, the pointer on the laptop: only that screen wakes; the other stays at rest.", sample: twoScreens },
  ],
  sizes: doesntApply("a lock is as large as its screen; the clock keeps its type sizes on every screen, and the sheet is `size.panel` wide."),
  states: {
    shown: ["rest", "loading", "error"],
    absent: {
      lit: "the template isn't a control: the field is lit while the keyboard is in it, as its page says, so awake it is always lit.",
      pressed: "a press anywhere at rest wakes it; awake, a press outside the sheet does nothing but keep it awake.",
      disabled: "the lock is always ready for a key; while a password is checked the field says so (Loading).",
      selected: "it holds no choice.",
      empty: "an empty field is its rest state: the placeholder says Password.",
    },
    rows: nonEmpty([{ name: "Awake on Shinjuku", cell: (state: Shown) => still({ phase: "awake", moment: state }) }]),
    described: {
      rest: "Awake and waiting: the field empty and lit, its placeholder Password, the reader's line under it. The same by night and by day: the lock draws in the wallpaper's night scheme in either mode.",
      loading: "A password or a finger being checked: the field's symbol is the hourglass and its placeholder Checking…, the reader's line stays. Nothing moves to make room.",
      error: "A wrong password or an unknown finger: the field shakes, empties and turns Invalid, its tube the error's, and the line under it says what went wrong in `error` in place of the reader's line. The next key clears it.",
    },
  },
  focus: doesntApply("the template draws no line of its own; the field lights as its page says, by the night scheme's primary, or the error's after a miss."),
  content: {
    text: [
      { name: "The date", words: DATES, sample: (words: string) => still({ phase: "rest", kicker: kickerFor(words) }) },
      { name: "The date, awake", words: DATES, sample: (words: string) => still({ phase: "awake", kicker: kickerFor(words) }) },
      { name: "The time", words: TIMES, sample: (words: string) => still({ phase: "rest", time: words }) },
      { name: "What went wrong", words: LOCK_ERROR, sample: (words: string) => still({ phase: "awake", moment: "error", error: words }) },
    ],
    missingPicture: [
      { name: "No picture yet", for: "Before the wallpaper has arrived: the screen is `surface_container_lowest` and the clock stands a little above the middle, as a picture with no place of its own puts it; the picture fades in over `slow` once it has.", sample: () => still({ phase: "rest", wallpaper: { ...SHIBUYA, picture: "" } }) },
    ],
    counts: doesntApply("a lock holds one clock and one field; each screen of a setup draws its own (Variants, Two screens)."),
  },
  motion: {
    arrives: [LOCKS, LOCKS_BARE],
    leaves: [UNLOCKS, TOUCHES],
    between: [WAKES, RESTS, SHAKE],
  },
  keyboard: {
    keys: [
      { keys: ["Any key"], does: "At rest, on the screen with the pointer: wakes it. A key that types a character lands in the field as its first character." },
      { keys: ["Enter"], does: "Awake: checks what is typed; with nothing typed, nothing." },
      { keys: ["Esc"], does: "Awake: empties the field and goes back to rest. At rest: nothing." },
      { keys: ["Tab", "Shift+Tab"], does: "The field is the only stop: the keyboard stays in it." },
      { keys: ["Typing"], does: "Awake: goes into the field, whatever had the keyboard; each key keeps it awake for another `duration.lock_idle` (20 s)." },
    ],
    order: "The field alone. Nothing under the lock takes the keyboard while it is up, on any screen.",
    lands: "Waking: in the field, its caret striking on. Going to rest: on the lock itself, so the next key wakes it. Unlocking: back on what had the keyboard before it locked.",
  },
  measures: {
    samples: nonEmpty([
      { name: "At rest", for: "The rest clock's size, the kicker's gap above the digits, and the kicker's shade.", sample: () => still({ phase: "rest" }) },
      { name: "At rest, without a cut-out", for: "The kicker's shade over a street of signs, and the digits with none.", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
      { name: "Awake", for: "The frame, the clock at the top, and the sheet: its width, its padding and the gaps inside it.", sample: () => still({ phase: "awake" }) },
      { name: "The desktop's still", for: "The frame with the bar's height at the top, the bar's clock the clock flies to and from, and the windows.", sample: () => still({ phase: "desktop" }) },
    ]),
    measures: [
      { what: "The rest clock's digits.", token: "type-clock_rest-size" },
      { what: "The awake clock's digits.", token: "type-clock-size" },
      { what: "The kicker's line.", token: "type-accent-line" },
      { what: "Between the kicker and the digits' tops; how far the kicker's shade reaches past it across; inside the sheet at its top and foot; between the reader's line and the key hints.", token: "space-s16" },
      { what: "How far the kicker's shade reaches past it down.", token: "space-s12" },
      { what: "The least room between the rest clock and the screen's edges.", token: "space-s32" },
      { what: "The frame's top edge on the desktop, which unlocking grows it to; the bar's clock is centred in it.", token: "size-bar" },
      { what: "The bar's time, which the clock lands on and leaves from.", token: "type-body-size" },
      { what: "Between the frame's top and the awake clock's kicker.", token: "space-s48" },
      { what: "The awake frame's edges, all four; how far the kicker's shade is blurred.", token: "space-s8" },
      { what: "The sheet's width.", token: "size-panel" },
      { what: "The field.", token: "size-control_md" },
      { what: "Between the field and the reader's line.", token: "space-s4" },
      { what: "The fillets into the frame and the sheet's far corners.", token: "radius-surface" },
      { what: "How far the awake picture blurs.", token: "size-lock_blur" },
    ],
    reflow: {
      described: "The lock live at the toolbar's window size. The picture covers the screen, and the rest clock keeps its place on it, so on a narrower screen the city is cropped and the clock with it, held `space.s32` inside the edges; it keeps `type.clock_rest` at every size. Awake, the clock stays centred at the top and the sheet `size.panel` wide in the middle of the bottom edge.",
      sample: reflow,
    },
  },
});
