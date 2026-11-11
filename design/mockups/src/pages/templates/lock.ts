// The Lock page: `docs/design/components/lock.md` is its full reference. At rest, the wallpaper
// and its clock with the city in front of the digits; awake, the frame closed in and the
// password's sheet up; always in the wallpaper's own night scheme.

import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { fromAwake, fromAwakeReduced, locking, lockingReduced, ON, reducedRest, resting, touching, touchingReduced, waking, wakingReduced } from "./lock-moves.ts";
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

const ON_SHIBUYA = "the lock on Shibuya";


const WAKES: Move = {
  name: "Waking",
  what: "The frame slides in over `long`, the picture blurs and dims, the clock lifts, the sheet rises.",
  tracks: waking(),
  reduced: wakingReduced(),
  back: "cuts",
  sample: held("rest"),
};



const RESTS: Move = {
  name: "Going back to rest",
  what: "Esc, or 20 s idle: the sheet sinks, the frame slides out, the clock flies home.",
  tracks: resting(ON),
  reduced: reducedRest(),
  back: "cuts",
  sample: held("awake"),
};




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



const TOUCHES: Move = {
  name: "Unlocking with the fingerprint",
  what: "From rest: the clock rises out from behind the city and flies into the bar's clock.",
  tracks: touching(ON),
  reduced: touchingReduced(ON),
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
  lede: "The lock screen: a clock the city stands in front of, then the frame and the password's sheet.",
  code: { unbuilt: "QML Lock: not built yet" },
  stage: {
    live: () => liveLock(),
    hint: "click it and type, or Esc",
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
    shown: ["rest", "loading", "error"],
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
    between: [WAKES, RESTS, SHAKE, MISSES],
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
