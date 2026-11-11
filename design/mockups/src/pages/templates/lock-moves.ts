// The Lock's moves as data (`docs/design/components/lock.md`): waking, going back to rest, locking
// and unlocking, the tracks the Lock page's players play and the wallpaper picker plays on the
// lock in front of you. `scope` narrows every target to one lock among several: the selector of
// the box holding it.

import type { Duration, Easing, Track } from "../../site/page.ts";

type Eased = Track<Easing>;

export const FRAME = "the frame";
export const BLUR = "the picture's blur";
export const DIM = "the dim";
export const CLOCK = "the clock";
export const SHEET = "the sheet";
export const KICKER_PART = "the kicker";
export const WINDOWS = "the windows";
export const BAR = "the bar's pills";
export const BAR_TIME = "the bar's time";
export const TOP = "the frame's top edge";
export const GLASS = "the frame's glass";
export const PUSH = "the push";
export const ON = "the lock on Shinjuku";
export const BLURRED = ".k-lock-blur, .k-lock-cutout";
export const DIMMED = ".k-screen-dim, .k-lock-cutout";

/** `target` inside the box `scope` selects, each of its selectors; `target` itself without one. */
export function within(scope: string, target: string): string {
  return scope === "" ? target : target.split(",").map((each) => `${scope} ${each.trim()}`).join(", ");
}

/** Waking from rest: the frame slides in, the picture blurs and dims, the clock lifts and the sheet rises. */
export function waking(on: string = ON, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: BLUR, on, target: within(scope, BLURRED), property: "--wake", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: DIM, on, target: within(scope, DIMMED), property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
    { part: CLOCK, on, target: within(scope, ".k-lock-clock"), property: "--clock-lift", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: SHEET, on, target: within(scope, ".k-sheet-part"), property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ];
}

/** Waking with reduce motion: the awake lock fades in over `short`. */
export function wakingReduced(on: string = ON, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the glass", on, target: within(scope, ".k-screen-glass"), property: "opacity", from: 0, to: { opacity: "frame" }, duration: "short", curve: "standard" },
    { part: BLUR, on, target: within(scope, BLURRED), property: "--wake", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: DIM, on, target: within(scope, DIMMED), property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: CLOCK, on, target: within(scope, ".k-lock-clock"), property: "--clock-lift", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: SHEET, on, target: within(scope, ".k-sheet-part"), property: "--reveal", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the sheet's fade", on, target: within(scope, ".k-sheet-part"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

export function reducedRest(on: string = ON, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: "the sheet's fade", on, target: within(scope, ".k-sheet-part"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the glass", on, target: within(scope, ".k-screen-glass"), property: "opacity", from: { opacity: "frame" }, to: 0, duration: "short", curve: "standard" },
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: DIM, on, target: within(scope, DIMMED), property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on, target: within(scope, BLURRED), property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on, target: within(scope, ".k-lock-clock"), property: "--clock-lift", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ];
}

/**
 * Super+L: the desktop's still leaves, its frame sliding off the screen with the bar's pills, and the
 * bar's time hands over to the lock's clock where it stands, which then grows out of the bar to the
 * rest clock behind the city, as the picture and its city push in.
 */
export function locking(on: string, scope = ""): [Eased, ...Eased[]] {
  const landed: Eased[] = [
    { part: KICKER_PART, on, target: within(scope, ".k-lock-kicker"), property: "opacity", from: 0, to: 1, duration: "short", cue: ["micro", "medium"], curve: "standard" },
    { part: "the clock's depth", on, target: within(scope, ".k-lock-clock"), property: "--clock-above", from: 1, to: 0, duration: "instant", cue: ["micro", "long"], curve: "standard" },
  ];
  return [
    { part: WINDOWS, on, target: within(scope, ".k-lock-windows"), property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the windows' size", on, target: within(scope, ".k-lock-windows"), property: "scale", from: 1, to: { scale: "behind" }, duration: "medium", curve: "emphasized_accel" },
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: TOP, on, target: within(scope, ".k-lock"), property: "--barred", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: BAR_TIME, on, target: within(scope, ".k-lock-bar-time"), property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: CLOCK, on, target: within(scope, ".k-lock-clock"), property: "--dock", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: PUSH, on, target: within(scope, ".k-lock"), property: "--push", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    ...landed,
  ];
}

/** Super+L with reduce motion: the desktop's still fades out as the rest clock fades in behind the city, over `short`. */
export function lockingReduced(on: string, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: WINDOWS, on, target: within(scope, ".k-lock-windows"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BAR, on, target: within(scope, ".k-bar"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: GLASS, on, target: within(scope, ".k-screen-glass"), property: "opacity", from: { opacity: "frame" }, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on, target: within(scope, ".k-lock-clock"), property: "--dock", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's depth", on, target: within(scope, ".k-lock-clock"), property: "--clock-above", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: KICKER_PART, on, target: within(scope, ".k-lock-kicker"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: TOP, on, target: within(scope, ".k-lock"), property: "--barred", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
  ];
}

/** The clock's flight into the bar and the desktop coming up round it, from `cue` on. */
export function intoTheBar(on: string, cue: Eased["cue"], scope = ""): Eased[] {
  const at = (...more: Duration[]): Eased["cue"] => {
    const before: Duration[] = cue === undefined ? [] : typeof cue === "string" ? [cue] : [...cue];
    const all = [...before, ...more];
    return all.length === 0 ? undefined : (all as [Duration, ...Duration[]]);
  };
  const timed = (track: Omit<Eased, "cue">, when: Eased["cue"]): Eased => (when === undefined ? track : { ...track, cue: when });
  return [
    timed({ part: "the clock's depth", on, target: within(scope, ".k-lock-clock"), property: "--clock-above", from: 0, to: 1, duration: "instant", curve: "standard" }, at()),
    timed({ part: CLOCK, on, target: within(scope, ".k-lock-clock"), property: "--dock", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: TOP, on, target: within(scope, ".k-lock"), property: "--barred", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: BAR_TIME, on, target: within(scope, ".k-lock-bar-time"), property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" }, at("long")),
    timed({ part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" }, at("long", "micro")),
  ];
}

/**
 * Unlocking from awake once the field has opened, over its first `short`: the sheet sinks, the clock
 * flies into the bar as the bar slides in, and the picture eases back to the desktop's.
 */
export function fromAwake(on: string = ON, scope = ""): Eased[] {
  return [
    { part: SHEET, on, target: within(scope, ".k-sheet-part"), property: "--reveal", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
    { part: KICKER_PART, on, target: within(scope, ".k-lock-kicker"), property: "opacity", from: 1, to: 0, duration: "short", cue: "short", curve: "standard" },
    ...intoTheBar(on, "short", scope),
    { part: BLUR, on, target: within(scope, BLURRED), property: "--wake", from: 1, to: 0, duration: "long", cue: "medium", curve: "standard" },
    { part: DIM, on, target: within(scope, DIMMED), property: "--panel-dim", from: 1, to: 0, duration: "long", cue: "medium", curve: "standard" },
    { part: WINDOWS, on, target: within(scope, ".k-lock-windows"), property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the windows' size", on, target: within(scope, ".k-lock-windows"), property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: PUSH, on, target: within(scope, ".k-lock"), property: "--push", from: 1, to: 0, duration: "long", cue: "medium", curve: "emphasized_decel" },
  ];
}

/** Unlocking from awake with reduce motion: the awake lock fades to the desktop over `short`. */
export function fromAwakeReduced(on: string = ON, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: "the sheet's fade", on, target: within(scope, ".k-sheet-part"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on, target: within(scope, BLURRED), property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: DIM, on, target: within(scope, DIMMED), property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: TOP, on, target: within(scope, ".k-lock"), property: "--barred", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: BAR, on, target: within(scope, ".k-bar"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the windows' size", on, target: within(scope, ".k-lock-windows"), property: "scale", from: { scale: "behind" }, to: 1, duration: "instant", curve: "standard" },
    { part: WINDOWS, on, target: within(scope, ".k-lock-windows"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

/** Unlocking with a known finger from rest: the clock rises out from behind the city into the bar. */
export function touching(on: string, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: KICKER_PART, on, target: within(scope, ".k-lock-kicker"), property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    ...intoTheBar(on, undefined, scope),
    { part: WINDOWS, on, target: within(scope, ".k-lock-windows"), property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: "the windows' size", on, target: within(scope, ".k-lock-windows"), property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: PUSH, on, target: within(scope, ".k-lock"), property: "--push", from: 1, to: 0, duration: "long", cue: "short", curve: "emphasized_decel" },
  ];
}

/** The same with reduce motion: the clock fades out as the desktop fades in. */
export function touchingReduced(on: string, scope = ""): [Eased, ...Eased[]] {
  return [
    { part: "the clock's fade", on, target: within(scope, ".k-lock-clock"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: FRAME, on, target: within(scope, ".k-lock"), property: "--framed", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: TOP, on, target: within(scope, ".k-lock"), property: "--barred", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: GLASS, on, target: within(scope, ".k-screen-glass"), property: "opacity", from: 0, to: { opacity: "frame" }, duration: "short", curve: "standard" },
    { part: BAR, on, target: within(scope, ".k-bar"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the windows' size", on, target: within(scope, ".k-lock-windows"), property: "scale", from: { scale: "behind" }, to: 1, duration: "instant", curve: "standard" },
    { part: WINDOWS, on, target: within(scope, ".k-lock-windows"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

