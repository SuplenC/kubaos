// The Lock page: `docs/design/components/lock.md` is its full reference. At rest, the wallpaper
// and its clock with the city in front of the digits; awake, the frame closed in and the
// password's sheet up; always in the wallpaper's own night scheme.

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
  what: "The frame slides in over `long`, the picture blurs and dims, the clock lifts, the sheet rises.",
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
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--clock-lift", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
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
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--clock-lift", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

const RESTS: Move = {
  name: "Going back to rest",
  what: "Esc, or 20 s idle: the sheet sinks, the frame slides out, the clock flies home.",
  tracks: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: FRAME, on: ON, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIM, on: ON, target: DIMMED, property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "long", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--clock-lift", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
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
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "--clock-lift", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
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
    { part: "the clock's depth", on, target: ".k-lock-clock", property: "--clock-above", from: 1, to: 0, duration: "instant", cue: ["micro", "long"], curve: "standard" },
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
    { part: "the clock's depth", on, target: ".k-lock-clock", property: "--clock-above", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: KICKER_PART, on, target: ".k-lock-kicker", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: FRAME, on, target: ".k-lock", property: "--framed", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: TOP, on, target: ".k-lock", property: "--barred", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
  ];
}

const LOCKS: Move = {
  name: "Locking",
  what: "Super+L: the windows go and the bar's clock grows into the rest clock behind the city.",
  tracks: locking(ON),
  reduced: lockingReduced(ON),
  back: "cuts",
  sample: held("desktop"),
};

const LOCKS_BARE: Move = {
  name: "Locking, without a cut-out",
  what: "The same on a picture with no depth: nothing cuts the clock as it flies.",
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
    timed({ part: "the clock's depth", on, target: ".k-lock-clock", property: "--clock-above", from: 0, to: 1, duration: "instant", curve: "standard" }, at()),
    timed({ part: CLOCK, on, target: ".k-lock-clock", property: "--dock", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: TOP, on, target: ".k-lock", property: "--barred", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }, at()),
    timed({ part: BAR_TIME, on, target: ".k-lock-bar-time", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" }, at("long")),
    timed({ part: "the clock's fade", on, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" }, at("long", "micro")),
  ];
}

const UNLOCKS: Move = {
  name: "Unlocking with the password",
  what: "The lock opens, the sheet sinks, and the clock flies into the bar's time: about 0.8 s.",
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
  what: "From rest: the clock rises out from behind the city and flies into the bar's clock.",
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
  what: "The field shakes its head, empties and turns Invalid; reduced motion keeps it still.",
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
  what: "It wakes as Waking does, the sheet showing the miss, the reader's mark shaking in `error`.",
  tracks: [
    ...waking(),
    { part: READER, on: ON, target: ".k-lock-reader", property: "--lock-reader-missed", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: 0, to: 1.5, duration: "micro", cue: "short", curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: 1.5, to: -1, duration: "micro", cue: ["short", "micro"], curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: -1, to: 0.5, duration: "micro", cue: ["short", "micro", "micro"], curve: "standard" },
    { part: READER_SHAKE, on: ON, target: ".k-lock-reader", property: "--jolt", from: 0.5, to: 0, duration: "short", cue: ["short", "micro", "micro", "micro"], curve: "standard_decel" },
  ],
  reduced: wakingReduced(),
  back: "cuts",
  sample: held("rest", PURPLE_HOUR, "unknown"),
};

const READS: Move = {
  name: "A finger on the reader",
  what: "While a finger is on the reader its mark fades to `cold`, and back as it lifts.",
  tracks: [{ part: READER, on: ON, target: ".k-lock-reader", property: "--lock-reader-reading", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: held("awake"),
};

const KNOWN_AWAKE: Move = {
  name: "Unlocking with the fingerprint, awake",
  what: "From awake after a miss: the mark turns to `lock_open`, then Unlocking with the password.",
  tracks: [
    { part: READER, on: ON, target: ".k-lock-reader", property: "--lock-reader-missed", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the reader's fingerprint", on: ON, target: ".k-lock-reader", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the reader's open lock", on: ON, target: ".k-lock-reader-open", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the placeholder", on: ON, target: ".k-field-placeholder", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    ...fromAwake(),
  ],
  reduced: fromAwakeReduced(),
  back: "cuts",
  sample: held("awake", PURPLE_HOUR, "unknown"),
};

type Shown = "rest" | "reading" | "loading" | "error";

/** A state of the sheet after a finger: its miss is an unknown finger, not a wrong password. */
const byFinger = (state: Shown) => (state === "error" ? "unknown" : state);

function reflow(): HTMLElement {
  return liveLock(PURPLE_HOUR, windowSize());
}

export default page({
  level: "templates",
  slug: "lock",
  title: "Lock",
  lede: "The lock screen: a clock the city stands in front of, then the frame and the password's sheet.",
  code: { unbuilt: "QML Lock: not built yet" },
  stage: {
    live: () => liveLock(),
    hint: "click it and type, hold the fingerprint, or Esc",
    daylight: doesntApply("it draws in its wallpaper's night scheme in either mode."),
  },
  variants: [
    { name: "Rest, a cut-out", for: "Shinjuku: towers before the digits' feet", sample: () => still({ phase: "rest", wallpaper: PURPLE_HOUR }) },
    { name: "Rest, a dark sky", for: "Tokyo Tower: the spire crosses a digit", sample: () => still({ phase: "rest", wallpaper: TOWER }) },
    { name: "Rest, no cut-out", for: "Shibuya, as a picture of your own", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
    { name: "Awake", for: "frame in, picture blurred, the sheet up", sample: () => still({ phase: "awake", wallpaper: PURPLE_HOUR }) },
    { name: "Awake, no cut-out", for: "Shibuya, in its own night scheme", sample: () => still({ phase: "awake", wallpaper: SHIBUYA }) },
    { name: "12-hour clock", for: "the meridiem at the kicker's end", sample: () => twelveHour("rest") },
    { name: "12-hour clock, awake", for: "the same, awake", sample: () => twelveHour("awake") },
    { name: "The desktop's still", for: "what locking and unlocking play on", sample: () => still({ phase: "desktop" }) },
    { name: "Two screens", for: "only the pointer's screen wakes", sample: twoScreens },
  ],
  sizes: doesntApply("as large as its screen; the sheet is `size.panel` wide."),
  states: {
    shown: ["rest", "reading", "loading", "error"],
    absent: {
      lit: "the field is lit while awake",
      pressed: "a press wakes it",
      disabled: "always ready for a key",
      selected: "it holds no choice",
      empty: "an empty field is rest",
    },
    rows: nonEmpty([
      { name: "A password", cell: (state: Shown) => still({ phase: "awake", moment: state }) },
      { name: "A finger", cell: (state: Shown) => still({ phase: "awake", moment: byFinger(state) }) },
    ]),
    looks: {
      rest: "awake, the field empty and lit",
      reading: "a finger on the reader: its mark in `cold`",
      loading: "an hourglass, Checking…",
      error: "the field shakes, or the reader's mark",
    },
  },
  focus: doesntApply("the field lights as its own page says."),
  content: {
    text: [
      { name: "The date", words: DATES, sample: (words: string) => still({ phase: "rest", kicker: kickerFor(words) }) },
      { name: "The date, awake", words: DATES, sample: (words: string) => still({ phase: "awake", kicker: kickerFor(words) }) },
      { name: "The time", words: TIMES, sample: (words: string) => still({ phase: "rest", time: words }) },
      { name: "A wrong password", words: LOCK_ERROR, sample: (words: string) => still({ phase: "awake", moment: "error", error: words }) },
      { name: "An unknown finger", words: FINGER_ERROR, sample: (words: string) => still({ phase: "awake", moment: "unknown", error: words }) },
    ],
    missingPicture: [
      { name: "No picture yet", for: "`surface_container_lowest`, then the picture fades in", sample: () => still({ phase: "rest", wallpaper: { ...SHIBUYA, picture: "" } }) },
    ],
    counts: doesntApply("one clock and one field a screen."),
  },
  motion: {
    arrives: [LOCKS, LOCKS_BARE],
    leaves: [UNLOCKS, TOUCHES, KNOWN_AWAKE],
    between: [WAKES, RESTS, SHAKE, READS, MISSES],
  },
  keyboard: {
    keys: [
      { keys: ["Any key"], does: "at rest: wakes it, typing into the field" },
      { keys: ["Enter"], does: "awake: checks what is typed" },
      { keys: ["Esc"], does: "awake: empties the field, back to rest" },
      { keys: ["Tab", "Shift+Tab"], does: "nowhere: the field is the only stop" },
      { keys: ["Typing"], does: "into the field, keeping it awake" },
    ],
    order: "The field alone; nothing under the lock takes the keyboard.",
    lands: "Waking, in the field; unlocking, back where it was before.",
  },
  measures: {
    samples: nonEmpty([
      { name: "At rest", for: "clock, kicker gap, halo", sample: () => still({ phase: "rest" }) },
      { name: "At rest, no cut-out", for: "the halo over signs", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
      { name: "Awake", for: "frame, awake clock, sheet", sample: () => still({ phase: "awake" }) },
      { name: "The desktop's still", for: "bar band, bar clock, windows", sample: () => still({ phase: "desktop" }) },
    ]),
    measures: [
      { part: "Rest digits", token: "type-clock_rest-size" },
      { part: "Awake clock's step", token: "scale-clock_awake" },
      { part: "Kicker line", token: "type-clock_date-line" },
      { part: "Kicker", token: "type-clock_date-size" },
      { part: "Halo, edge", token: "size-halo_edge" },
      { part: "Halo, round the word", token: "size-halo" },
      { part: "Kicker → digits; sheet top, foot", token: "space-s16" },
      { part: "Clock → screen edge", token: "space-s32" },
      { part: "Bar band", token: "size-bar" },
      { part: "Bar time", token: "type-body-size" },
      { part: "Awake frame", token: "space-s8" },
      { part: "Sheet width", token: "size-panel" },
      { part: "Field", token: "size-control_md" },
      { part: "Field marks' inset", token: "space-s12" },
      { part: "Fillets, sheet corners", token: "radius-surface" },
      { part: "Awake blur", token: "size-lock_blur" },
    ],
    reflow: {
      note: "The picture covers the screen; the clock keeps its place on it, `space.s32` inside the edges.",
      sample: reflow,
    },
  },
});
