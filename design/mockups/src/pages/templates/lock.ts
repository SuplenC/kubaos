// The Lock page: `docs/design/components/lock.md`. The lock screen at rest, the wallpaper and its
// clock with the city's cut-out in front of the digits, and awake, the frame closed in and the
// password's sheet up, always in the wallpaper's own night scheme.

import { doesntApply, type Move, nonEmpty, page, type Track } from "../../site/page.ts";
import type { Duration, Easing } from "../../site/page.ts";
import type { Moment } from "../../lib/lock.ts";
import { windowSize } from "../../site/settings.ts";
import { DATES, FINGER_ERROR, kickerFor, LOCK_ERROR, liveLock, PURPLE_HOUR, SHIBUYA, still, TIMES, TOWER, twelveHour, twoScreens } from "./locks.ts";

/**
 * A lock drawn still, wrapped so a move can reach its root: the frame's number is set there. Its
 * sheet shows `moment`, seen or not.
 */
function held(phase: "rest" | "awake" | "desktop", wallpaper = PURPLE_HOUR, moment: Moment = "rest"): () => HTMLElement {
  return () => {
    const box = document.createElement("span");
    box.className = "k-lock-held";
    box.append(still({ phase, wallpaper, moment }));
    return box;
  };
}

const FRAME = "the frame";
const BLUR = "the picture's blur";
const DIM = "the dim";
const CLOCK = "the clock";
const SHEET = "the sheet";
const KICKER_PART = "the kicker";
const WINDOWS = "the windows";
const BAR = "the bar's pills";
const BAR_TIME = "the bar's time";
const TOP = "the frame's top edge";
const GLASS = "the frame's glass";
const PUSH = "the push";
const ON = "the lock on Shinjuku";
const ON_SHIBUYA = "the lock on Shibuya";
const BLURRED = ".k-lock-blur, .k-lock-cutout";
const DIMMED = ".k-screen-dim, .k-lock-cutout";

type Eased = Track<Easing>;

const WAKES: Move = {
  name: "Waking",
  what: "A key or a press on the screen with the pointer, or a finger the reader doesn't know, which wakes it with its miss shown (A finger not recognised): the frame slides in from past the screen's edges to `space.s8` all round over `long` on `emphasized_decel`, the locking's slide reversed: each edge comes in from its own side, by its thickness and the fillet its inner corners curve through, `radius.surface` all the way, and its shadow strengthens as it comes. The picture blurs to `size.lock_blur` on the same curve, and dims to `opacity.panel_scrim` over `medium` on `standard`, as a panel's desktop does. The city blurs and dims with the picture and stays in front, and over `long` on `emphasized_decel` the clock takes the first `scale.clock_awake` (0.2) of its flight to the bar's clock, measured along the path: its size, its place and its colour all move that far from the rest clock's toward the bar's time together, on whole device pixels, so it stays big, a little higher and a little smaller, above the dim and still behind the city, and keeps most of the picture's tint. Its kicker keeps its rest sizes, 20 on its 24 line. The sheet rises from the bottom edge as a Sheet opens, over `long` on `emphasized_decel`, riding on the edge as it slides in. The picture and its city keep their push under the blur. With reduce motion the lock fades to awake over `short`: the frame, the sheet and the clock at its awake step fade in, and the blur and the dim fade in over the picture and its city.",
  tracks: waking(),
  reduced: wakingReduced(),
  back: "cuts",
  sample: held("rest"),
};

/** Waking from rest: the frame slides in, the picture blurs and dims, the clock lifts and the sheet rises. */
function waking(): [Eased, ...Eased[]] {
  return [
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ];
}

/** Waking with reduce motion: the awake lock fades in over `short`. */
function wakingReduced(): [Eased, ...Eased[]] {
  return [
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the glass", on: ON, target: ".k-screen-glass", property: "opacity", from: 0, to: { opacity: "frame" }, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

const RESTS: Move = {
  name: "Going back to rest",
  what: "Esc, or `duration.lock_idle` (20 s) with no key, press or touch: the sheet goes back into the bottom edge as a Sheet closes and the frame slides back out past the screen's edges, both over `medium` on `emphasized_accel`; the dim lifts over `medium` on `standard` and the blur clears over `long` on `standard`. The clock goes back along its flight to its rest place, size and tint over `long` on `emphasized_decel`, the city sharpening and brightening with the picture. The field empties. With reduce motion the awake lock fades out over `short` and the rest clock fades in behind the city over the next `short`.",
  tracks: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "long", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
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
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--lift", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ];
}

/**
 * Super+L: the desktop's still leaves, its frame sliding off the screen with the bar's pills, and the
 * bar's time hands over to the lock's clock where it stands, which then grows out of the bar to the
 * rest clock behind the city, as the picture and its city push in.
 */
function locking(on: string): [Eased, ...Eased[]] {
  const landed: Eased[] = [
    { part: KICKER_PART, on, target: ".k-lock-kicker", property: "opacity", from: 0, to: 1, duration: "short", cue: ["micro", "medium"], curve: "standard" },
    { part: "the clock's depth", on, target: ".k-lock-clock", property: "--above", from: 1, to: 0, duration: "instant", cue: ["micro", "long"], curve: "standard" },
  ];
  return [
    { part: WINDOWS, on, target: ".k-lock-windows", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the windows' size", on, target: ".k-lock-windows", property: "scale", from: 1, to: { scale: "behind" }, duration: "medium", curve: "emphasized_accel" },
    { part: FRAME, on, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: TOP, on, target: ".k-lock", property: "--barred", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: BAR_TIME, on, target: ".k-lock-bar-time", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: CLOCK, on, target: ".k-lock-clock", property: "--dock", from: 1, to: 0, duration: "long", cue: "micro", curve: "emphasized_decel" },
    { part: PUSH, on, target: ".k-lock", property: "--push", from: 0, to: 1, duration: "long", cue: "micro", curve: "emphasized_decel" },
    ...landed,
  ];
}

/** Super+L with reduce motion: the desktop's still fades out as the rest clock fades in behind the city, over `short`. */
function lockingReduced(on: string): [Eased, ...Eased[]] {
  return [
    { part: WINDOWS, on, target: ".k-lock-windows", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BAR, on, target: ".k-bar", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: GLASS, on, target: ".k-screen-glass", property: "opacity", from: { opacity: "frame" }, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on, target: ".k-lock-clock", property: "--dock", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's depth", on, target: ".k-lock-clock", property: "--above", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: KICKER_PART, on, target: ".k-lock-kicker", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: FRAME, on, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: TOP, on, target: ".k-lock", property: "--barred", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
  ];
}

const LOCKS: Move = {
  name: "Locking",
  what: "Super+L. The session locks in the first frame; what follows plays on a still of the desktop the lock captured then, drawn inside the lock's own surface, so the lock and the desktop are one surface. The city is on screen from that first frame: it is the picture itself drawn through the cut-out's alpha, so the first frame is the desktop's picture exactly. The windows fade and shrink to `scale.behind` round the middle of the screen over `medium` on `emphasized_accel`, and on the same `medium` and curve the frame slides off the screen, as the picture pushes in: each edge goes out past its own side by its thickness and the fillet its inner corners curve through, `radius.surface` all the way, so the frame leaves fast, as if the screen zoomed in past it. The top edge goes up at the bar's full 40, carrying the bar's pills, never thinning; the sides go left and right and the bottom goes down; its shadow weakens as it goes, so none is left at rest. The bar's clock grows out of the bar: its time, Geist Regular, hands over to the lock's clock, Geist Light, where it stands over `micro`, the strokes thinning in place, and its pill, the weekday and the date round an empty slot, rides up with the top edge, so the clock leaves it as it leaves the screen; from `micro` on the clock grows from the bar's time in `type.body` to `type.clock_rest` at its rest place over `long` on `emphasized_decel`, on whole device pixels, its digits taking the picture's tint, and the kicker fades in above it over `short` from `micro` and `medium` on. From `micro` on too, with the clock's flight, over `long` on `emphasized_decel`, the picture pushes in from the desktop's 1:1 to `scale.push` (1.025) and its city in front of it further, to `scale.near` (1.05), both round the middle of the screen's bottom edge, so the city comes off the sky; the plate behind the city comes in with the push and fills what the city uncovers, a band along its towers' sides. The clock flies above the frame and the windows, yet behind the city: the city's alpha is taken out of its digits wherever the city stands in front of them, so it never crosses the city. Once it has landed it drops behind the frame, between the picture and its city, which changes nothing on screen. A lid close or an idle suspend skips all of it and goes straight to rest. With reduce motion nothing pushes or slides: the desktop, its frame and its bar fade out as the rest clock and its kicker fade in behind the city, over `short`.",
  tracks: locking(ON),
  reduced: lockingReduced(ON),
  back: "cuts",
  sample: held("desktop"),
};

const LOCKS_BARE: Move = {
  name: "Locking, without a cut-out",
  what: "The same on a picture with no depth, Shibuya's or one of your own: the picture pushes in alone, nothing cuts the clock as it flies, and once it has landed it drops behind nothing: the tinted clock stands on the bare picture, only its kicker's letters haloed.",
  tracks: locking(ON_SHIBUYA),
  reduced: lockingReduced(ON_SHIBUYA),
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
    timed({ part: BAR_TIME, on, target: ".k-lock-bar-time", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" }, at("long")),
    timed({ part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" }, at("long", "micro")),
  ];
}

const UNLOCKS: Move = {
  name: "Unlocking with the password",
  what: "The right password, from awake: about 0.8 s in all. The field's `lock` turns to `lock_open` in `primary` over `short`. The sheet sinks into the bottom edge as a Sheet closes, over `medium` on `emphasized_accel`. From `short` on the clock rises above the frame, still cut by the city, and flies the rest of its flight from where waking left it into the bar's clock over `long` on `emphasized_decel`, its size, its place and its colour going on from the awake step with no jump, its kicker fading over `short`, and lands on the bar's time on whole device pixels: its left edge, its width, its baseline and `type.body`'s size: the Light's letters spread as it lands by what the bar's Regular is wider, an even share after each, so each figure stands within a fifth of a logical pixel of the bar's. Once it has landed it hands over to the bar's own time over `micro`: the bar's Geist Regular fades in on top of the clock's Geist Light, which then goes, so the strokes thicken in place and nothing else moves. Meanwhile the bar slides down from past the screen's top edge, by its 40 and the fillet, carrying its pills, on the same curve: over the frame's thin top edge first, then pushing it down from `space.s8` to the bar's 40, its inner corners keeping `radius.surface`; the sides and the bottom edge are in already and stay. The clock's pill arrives round its empty slot and the clock lands in it as both settle. From `medium` on the picture's blur and dim clear from the picture and its city and the windows settle in from `scale.behind`, over `long`, the picture easing back from `scale.push` and its city from `scale.near` to the desktop's 1:1 with them on `emphasized_decel`, the plate going with the push, so the last frame is the desktop's picture. The kicker leaves at its rest 20, which it keeps awake. The motion plays on the still the lock captured as it locked; the session unlocks as it ends, and the real windows take the still's place. With reduce motion the awake lock fades to the desktop over `short`.",
  tracks: [
    { part: "the field's lock", on: ON, target: ".k-lock-shut", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the field's open lock", on: ON, target: ".k-lock-open", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    ...fromAwake(),
  ],
  reduced: fromAwakeReduced(),
  back: "cuts",
  sample: held("awake"),
};

/**
 * Unlocking from awake once the field has opened, over its first `short`: the sheet sinks, the clock
 * flies into the bar as the bar slides in, and the picture eases back to the desktop's.
 */
function fromAwake(): Eased[] {
  return [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", cue: "short", curve: "emphasized_accel" },
    { part: KICKER_PART, on: ON, target: ".k-lock-kicker", property: "opacity", from: 1, to: 0, duration: "short", cue: "short", curve: "standard" },
    ...intoTheBar(ON, "short"),
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "long", cue: "medium", curve: "standard" },
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 1, to: 0, duration: "long", cue: "medium", curve: "standard" },
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "medium", curve: "emphasized_decel" },
    { part: PUSH, on: ON, target: ".k-lock", property: "--push", from: 1, to: 0, duration: "long", cue: "medium", curve: "emphasized_decel" },
  ];
}

/** Unlocking from awake with reduce motion: the awake lock fades to the desktop over `short`. */
function fromAwakeReduced(): [Eased, ...Eased[]] {
  return [
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: TOP, on: ON, target: ".k-lock", property: "--barred", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: BAR, on: ON, target: ".k-bar", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "instant", curve: "standard" },
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

const TOUCHES: Move = {
  name: "Unlocking with the fingerprint",
  what: "A known finger, from rest: the kicker fades over `micro`, and the clock rises above the frame at once, behind the city still: the city's alpha is taken out of its digits wherever it stands in front of them, so the clock comes up out from behind the city, never through it. The rest clock flies straight from `type.clock_rest` into the bar's clock over `long` on `emphasized_decel`, its digits going from the picture's tint to `on_surface`, landing and handing over to the bar's time as the password's unlock does. The frame slides in from past the screen's edges, the locking's slide reversed: its top edge arrives at the bar's full 40 carrying the bar's pills, the sides and the bottom with it, on the same curve, so the clock lands in its pill as both settle, and from `short` on the windows settle in from `scale.behind` over `long`, the picture, its plate and its city easing back to the desktop's 1:1 with them on `emphasized_decel`. The kicker leaves at its rest 20. With reduce motion the rest lock fades to the desktop over `short`.",
  tracks: [
    { part: KICKER_PART, on: ON, target: ".k-lock-kicker", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    ...intoTheBar(ON, undefined),
    { part: WINDOWS, on: ON, target: ".k-lock-windows", property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: "the windows' size", on: ON, target: ".k-lock-windows", property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: PUSH, on: ON, target: ".k-lock", property: "--push", from: 1, to: 0, duration: "long", cue: "short", curve: "emphasized_decel" },
  ],
  reduced: [
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
  what: "The field shakes its head: `space.s12` one way, `space.s8` back past its place, `space.s4` the first way, each over `micro` on `standard`, then home over `short` on `standard_decel`, a whole device pixel at a time; the field empties, turns Invalid, its tube the error's, and its placeholder says what went wrong until the next key. With reduce motion it stays still and only the field says so.",
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

const READER = "the reader's mark";
const READER_SHAKE = "the reader's mark's shake";

const MISSES: Move = {
  name: "A finger not recognised",
  what: "A finger the reader doesn't know, from rest. A touch wakes the lock, so it wakes exactly as Waking does: the frame slides in, the picture blurs and dims, the clock lifts and the sheet rises, all on Waking's own durations and curves. The sheet arrives already showing the miss: its placeholder says what went wrong (Finger not recognised), and from `short` on the reader's `fingerprint` eases from `primary` to `error` over `short` on `standard`, as the sheet settles. With it, from `short` on, the mark alone shakes its head, half the wrong password's jolt: `space.s4` and a half one way, `space.s4` back past its place, half a `space.s4` the first way, each over `micro` on `standard`, then home over `short` on `standard_decel`, a whole device pixel at a time. The field stays as it was: no shake, no Invalid tube, since the password wasn't wrong. With reduce motion the lock fades to awake over `short` as Waking does, the mark already in `error`, and nothing shakes.",
  tracks: [
    ...waking(),
    { part: READER, on: ON, target: ".k-lock-reader .k-icon", property: "color", from: { color: "primary" }, to: { color: "error" }, duration: "short", cue: "short", curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: 0, to: 1.5, duration: "micro", cue: "short", curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: 1.5, to: -1, duration: "micro", cue: ["short", "micro"], curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: -1, to: 0.5, duration: "micro", cue: ["short", "micro", "micro"], curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: 0.5, to: 0, duration: "short", cue: ["short", "micro", "micro", "micro"], curve: "standard_decel" },
  ],
  reduced: wakingReduced(),
  back: "cuts",
  sample: held("rest", PURPLE_HOUR, "unknown"),
};

const KNOWN_AWAKE: Move = {
  name: "Unlocking with the fingerprint, awake",
  what: "A known finger, from awake after a finger the reader didn't know: the reader's mark is in `error` and the placeholder says Finger not recognised. Over `short` on `standard` the mark eases back to `primary` and turns into `lock_open`, as the field's `lock` turns after the right password: the `fingerprint` fades as the `lock_open` in `primary` laid over it fades in. The placeholder clears over the same `short`. The rest is Unlocking with the password, track for track: from `short` on the sheet sinks into the bottom edge, the clock rises above the frame and flies into the bar's clock as the bar slides in over the frame's thin top edge, and from `medium` on the picture's blur and dim clear, the windows settle in and the picture and its city ease back from the push to the desktop's 1:1. With reduce motion the awake lock fades to the desktop over `short`.",
  tracks: [
    { part: READER, on: ON, target: ".k-lock-reader .k-icon", property: "color", from: { color: "error" }, to: { color: "primary" }, duration: "short", curve: "standard" },
    { part: "the reader's fingerprint", on: ON, target: ".k-lock-reader", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the reader's open lock", on: ON, target: ".k-lock-reader-open", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the placeholder", on: ON, target: ".k-field-placeholder", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    ...fromAwake(),
  ],
  reduced: fromAwakeReduced(),
  back: "cuts",
  sample: held("awake", PURPLE_HOUR, "unknown"),
};

type Shown = "rest" | "loading" | "error";

/** A state of the sheet after a finger: its miss is an unknown finger, not a wrong password. */
const byFinger = (state: Shown) => (state === "error" ? "unknown" : state);

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
    "QML `Lock`, a template of the shell's alone: only the shell locks. Click into it and type, or press it: it wakes, and what you type lands in the field. Enter with a password shakes its head and says so in its placeholder; Esc, or 20 s with nothing done, puts it back to rest.",
    "At rest the lock is the wallpaper and its clock, and nothing else: no frame, no bar, no field. The digits are `type.clock_rest`, Geist Light at 384, about 280 tall, under a kicker line `space.s16` above their tops: where and which day in `clock_place` (東京・月曜日, M PLUS 1), then ` · ` and the date in `clock_date` (Geist), both 20 on a 24 line, medium. The kicker and the digits are each centred on the clock's middle, on whole device pixels with the odd one after. The digits are a light tint of the picture behind them, so they read as part of it: the hue the picture shows most strongly under them (each pixel's OKLCH hue counted by its chroma squared, so the glow of a sky or a street's signs decide it, not its haze), at that chroma times 1.2, at lightness 0.84. They are large enough to read without a contrast of their own, so nothing shades them. The kicker takes the same colour, its date at `opacity.clock_date` (0.85) of it, so where and which day lead the line. Every screen of a setup shows rest, each with its own clock.",
    "Each wallpaper says where its clock stands: the middle of the digits' baseline as shares of the picture, so the clock keeps its place against the city however a screen crops the picture, kept `space.s32` inside the screen's edges. A wallpaper with depth has its city cut out (an alpha picture, the same size as the wallpaper) and a plate, the picture with the city painted out a little past its outline. The lock draws the picture and its plate, then the clock, then the city: the picture itself through the cut-out's alpha, so the towers stand in front of the clock: on Shinjuku the skyline covers the digits' feet, on Tokyo Tower the spire crosses the last digit. The place keeps the kicker clear of the city. At rest the picture and its plate stand pushed in to `scale.push` (1.025) and the city further, to `scale.near` (1.05), round the middle of the screen's bottom edge, so the city stands off the sky, and it holds still; the clock's place moves with the picture. A picture of your own has no depth: it pushes in alone, and the clock stands in front of it.",
    "Nothing stands behind the kicker: no capsule, no band. Round each letter lies a halo of `surface`, two text shadows, a tight one blurred by `size.halo_edge` (1) at `opacity.halo_edge` (0.50) that holds the strokes' edges and a soft one blurred by `size.halo` (6) at `opacity.halo` (0.35) round the word. It follows the glyphs, so no shape shows, and it is the same on every wallpaper: the clock matching its picture matters more than the kicker's contrast. On Shinjuku's pink sky the line holds about 2.5:1 round its letters, on Tokyo Tower's blue hour 4.4 to 5.4:1, and on Shibuya the place 12:1 over a dark awning while the date crosses lit signs. The digits have no halo, with a cut-out or without.",
    "Awake, on the screen with the pointer only, the shell's frame slides in round the picture, `surface` at `opacity.frame`, `space.s8` all round: the top edge is as thin as the others, and the bar slides down over it only as the lock gives way to the desktop. The picture blurs to `size.lock_blur` (8) and dims to `scrim` at `opacity.panel_scrim`. The city blurs and dims with the picture and stays where it is, in front of the clock. The clock has no awake size or place of its own: waking takes it `scale.clock_awake` (0.2) of the way along its flight to the bar's clock, measured along the path, its size, its place and its colour together, so it stays big, a little higher and smaller than at rest and drawn toward the middle, above the dim, and keeps most of the picture's tint, a fifth of the way to `on_surface`. Its kicker keeps its rest sizes, 20 on its 24 line, a fifth of the way from the tint to its own roles (where and which day toward `primary`, the date toward `on_surface_variant`); its halo stays. Unlocking flies the rest of the way from there.",
    "The lock and the desktop are one surface. Super+L locks the session in its first frame and captures a still of the desktop then; the lock draws that still inside its own surface (the windows, and the bar's pills on the frame) and plays its locking on it: the windows go, and the bar's clock grows out of the bar into the rest clock. Unlocking plays the same way back, the clock flying into the bar's clock, and the session unlocks as it ends. A lid close or an idle suspend skips the motion and goes straight to rest.",
    "A Sheet rises from the middle of the bottom edge, curving into the frame through concave fillets of `radius.surface`, as the wallpaper picker does, `size.panel` (384) wide with a PanelPage's padding: `space.s16` at its top and foot, the container's 16 at its sides. It holds the md `Field` for the password and nothing else, so the sheet is the field and its padding: the field's lock symbol leads it in the field's icon colour, the `fingerprint` symbol ends it in `primary`, the same size and as far in from its end as the lock is from its start, and it is lit with the keyboard in it. On a machine without a fingerprint reader the field has no trailing mark. A finger the reader doesn't know wakes the lock from rest, the sheet arriving with the miss shown: the mark in `error` and the placeholder Finger not recognised. A known finger unlocks: from rest the clock goes straight into the bar; awake, the mark turns into `lock_open` in `primary` and the lock unlocks as with the password. Enter and Esc need no hints: the field is the only thing on the sheet. No avatar, no initial, no emoji: only KubaOS Symbols.",
    "The lock ignores light and dark. It always draws in the night scheme matugen derives from its wallpaper (the same `scheme-fidelity` dot-theme uses, in dark), and its derived roles from those: Shinjuku's purple hour gives a lavender primary, Tokyo Tower's blue hour a pale blue one, Shibuya's rain a teal one. So the lock is the same in either mode, and its sheet and its awake kicker keep their contrast over the night picture.",
  ],
  variants: [
    { name: "At rest, with a cut-out", for: "Shinjuku at purple hour: the skyline's towers stand in front of the digits' feet; the digits stand in the open sky in a pale pink drawn from the horizon behind them, and the kicker in the same pink stands off the sky on the halo round its letters alone.", sample: () => still({ phase: "rest", wallpaper: PURPLE_HOUR }) },
    { name: "At rest, with a cut-out, a dark sky", for: "Tokyo Tower at blue hour: the spire crosses the last digit; the digits are a pale blue drawn from the sky, and the kicker in the same blue reads plainly on the dark sky.", sample: () => still({ phase: "rest", wallpaper: TOWER }) },
    { name: "At rest, without a cut-out", for: "Shibuya on a rainy night, as a picture of your own is: the clock in front, its digits a gold drawn from the lit storefronts behind them over the bare picture, and the kicker in the same gold, its halo lifting the date where it crosses the signs.", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
    { name: "Awake", for: "The frame closed in, the picture blurred and dimmed, the clock a fifth of the way along its flight to the bar, still behind the city, and the password's sheet up from the bottom edge.", sample: () => still({ phase: "awake", wallpaper: PURPLE_HOUR }) },
    { name: "Awake, without a cut-out", for: "The same on Shibuya, in its own night scheme.", sample: () => still({ phase: "awake", wallpaper: SHIBUYA }) },
    { name: "The 12-hour clock", for: "Where the locale or Settings ask for 12 hours: the digits without a meridiem (9:47), which takes its own slot at the kicker's end (· PM), so the digits keep one size and the clock its place.", sample: () => twelveHour("rest") },
    { name: "The 12-hour clock, awake", for: "The same at the clock's awake step.", sample: () => twelveHour("awake") },
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
    rows: nonEmpty([
      { name: "A password", cell: (state: Shown) => still({ phase: "awake", moment: state }) },
      { name: "A finger", cell: (state: Shown) => still({ phase: "awake", moment: byFinger(state) }) },
    ]),
    described: {
      rest: "Awake and waiting: the field empty and lit, its placeholder Password, the reader's `fingerprint` at its end in `primary`. The same by night and by day: the lock draws in the wallpaper's night scheme in either mode.",
      loading: "A password or a finger being checked: the field's symbol is the hourglass and its placeholder Checking…; the reader's mark stays `primary`. Nothing moves to make room.",
      error: "A wrong password: the field shakes, empties and turns Invalid, its tube the error's, and its placeholder says what went wrong; the reader's mark stays `primary`. An unknown finger: the field is the password's, its placeholder says what went wrong, and the reader's mark turns `error` and shakes alone, so the miss is told where the finger went; the field doesn't shake, since it wasn't wrong. At rest, an unknown finger wakes the lock, which arrives with the miss already shown. The next key, or the next touch, clears either; a known finger then turns the mark into `lock_open` and unlocks as the password does.",
    },
  },
  focus: doesntApply("the template draws no line of its own; the field lights as its page says, by the night scheme's primary, or the error's after a miss."),
  content: {
    text: [
      { name: "The date", words: DATES, sample: (words: string) => still({ phase: "rest", kicker: kickerFor(words) }) },
      { name: "The date, awake", words: DATES, sample: (words: string) => still({ phase: "awake", kicker: kickerFor(words) }) },
      { name: "The time", words: TIMES, sample: (words: string) => still({ phase: "rest", time: words }) },
      { name: "A wrong password", words: LOCK_ERROR, sample: (words: string) => still({ phase: "awake", moment: "error", error: words }) },
      { name: "An unknown finger", words: FINGER_ERROR, sample: (words: string) => still({ phase: "awake", moment: "unknown", error: words }) },
    ],
    missingPicture: [
      { name: "No picture yet", for: "Before the wallpaper has arrived: the screen is `surface_container_lowest` and the clock stands a little above the middle, as a picture with no place of its own puts it; the picture fades in over `slow` once it has.", sample: () => still({ phase: "rest", wallpaper: { ...SHIBUYA, picture: "" } }) },
    ],
    counts: doesntApply("a lock holds one clock and one field; each screen of a setup draws its own (Variants, Two screens)."),
  },
  motion: {
    arrives: [LOCKS, LOCKS_BARE],
    leaves: [UNLOCKS, TOUCHES, KNOWN_AWAKE],
    between: [WAKES, RESTS, SHAKE, MISSES],
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
      { name: "At rest", for: "The rest clock's size, the kicker's gap above the digits, and the kicker's halo.", sample: () => still({ phase: "rest" }) },
      { name: "At rest, without a cut-out", for: "The kicker's halo over a street of signs, and the digits with none.", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
      { name: "Awake", for: "The frame, the clock at its awake step, and the sheet: its width, and its padding round the field.", sample: () => still({ phase: "awake" }) },
      { name: "The desktop's still", for: "The frame with the bar's height at the top, the bar's clock the clock flies to and from, and the windows.", sample: () => still({ phase: "desktop" }) },
    ]),
    measures: [
      { what: "The rest clock's digits.", token: "type-clock_rest-size" },
      { what: "How far along its flight to the bar's time the awake clock stands: its size, its place and its colour.", token: "scale-clock_awake" },
      { what: "The kicker's line.", token: "type-clock_date-line" },
      { what: "The kicker, at rest and awake.", token: "type-clock_date-size" },
      { what: "How far the kicker's halo is blurred at its letters' edges.", token: "size-halo_edge" },
      { what: "How far the kicker's halo is blurred round the word.", token: "size-halo" },
      { what: "Between the kicker and the digits' tops; inside the sheet at its top and foot.", token: "space-s16" },
      { what: "The least room between the rest clock and the screen's edges.", token: "space-s32" },
      { what: "The bar's band, the frame's top edge on the desktop, which slides off and back with its pills; the bar's clock is centred in it.", token: "size-bar" },
      { what: "The bar's time, which the clock lands on and leaves from.", token: "type-body-size" },
      { what: "The awake frame's edges, all four.", token: "space-s8" },
      { what: "The sheet's width.", token: "size-panel" },
      { what: "The field.", token: "size-control_md" },
      { what: "The field's lock and the reader's mark, each from its end.", token: "space-s12" },
      { what: "The fillets into the frame and the sheet's far corners.", token: "radius-surface" },
      { what: "How far the awake picture blurs.", token: "size-lock_blur" },
    ],
    reflow: {
      described: "The lock live at the toolbar's window size. The picture covers the screen, and the rest clock keeps its place on it, so on a narrower screen the city is cropped and the clock with it, held `space.s32` inside the edges; it keeps `type.clock_rest` at every size. Awake, the clock stands a fifth of the way from its rest place to the bar's time, which is centred on the screen, and the sheet `size.panel` wide in the middle of the bottom edge.",
      sample: reflow,
    },
  },
});
