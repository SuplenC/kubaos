// The Lock page: `docs/design/components/lock.md`. The lock screen at rest, the wallpaper and its
// clock with the city's cut-out in front of the digits, and awake, the frame closed in and the
// password's sheet up, always in the wallpaper's own night scheme.

import { doesntApply, type Move, nonEmpty, page, type Track } from "../../site/page.ts";
import type { Easing } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { DATES, kickerFor, LOCK_ERROR, liveLock, PURPLE_HOUR, SHIBUYA, still, TIMES, TOWER, twelveHour, twoScreens } from "./locks.ts";

/** A lock drawn still, wrapped so a move can reach its root: the frame's number is set there. */
function held(phase: "rest" | "awake", wallpaper = PURPLE_HOUR): () => HTMLElement {
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
const ON = "the lock on Shinjuku";
const BLURRED = ".k-lock-blur, .k-lock-cutout";

type Eased = Track<Easing>;

const WAKES: Move = {
  name: "Waking",
  what: "A key, a press or a touch of the reader, on the screen with the pointer: the frame closes in from past the screen's edges over `long` on `emphasized_decel`, its inner corners growing with it; the picture blurs to `size.lock_blur` on the same curve, and dims to `opacity.panel_scrim` over `medium` on `standard`, as a panel's desktop does. The cut-out fades over `short` on `standard`, so the clock comes out from behind the city, and the clock lifts over `long` on `emphasized_decel` from its rest place to `size.bar` and `space.s48` from the top, its digits going from `type.clock_rest` to `type.clock`, on whole device pixels. The sheet rises from the bottom edge as a Sheet opens, over `long` on `emphasized_decel`. With reduce motion the lock fades to awake over `short`: the frame, the sheet and the clock in their awake places fade in, the blur and the dim fade in, and the cut-out is gone at once.",
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

const LOCKS: Move = {
  name: "Locking",
  what: "The screen locks, by hand, by the idle timer or before sleep: the lock arrives at rest, every screen at once. Until the picture has arrived the screen is `surface_container_lowest`; the picture, the clock and the cut-out fade in over `scene` on `emphasized_decel`, one slow settled arrival, as a full-screen moment is looked at as it comes. Nothing moves. With reduce motion they fade in over `short`.",
  tracks: [
    { part: "the picture", on: ON, target: ".k-screen-wallpaper", property: "opacity", from: 0, to: 1, duration: "scene", curve: "emphasized_decel" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "scene", curve: "emphasized_decel" },
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 0, to: 1, duration: "scene", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the picture", on: ON, target: ".k-screen-wallpaper", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: CUTOUT, on: ON, target: ".k-lock-cutout", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
  ],
  back: "cuts",
  sample: held("rest"),
};

const UNLOCKS: Move = {
  name: "Unlocking",
  what: "The right password or a known finger: the lock gives way to the desktop it was over, which has the same wallpaper and the same frame. The sheet goes back into the bottom edge as a Sheet closes, over `medium` on `emphasized_accel`; the clock fades out and the blur clears on the same curve, and the dim lifts over `medium` on `standard`; the frame stays, and the desktop's bar and windows come up in it. `medium` and no longer: the session unlocks the moment it ends. With reduce motion the same fades over `short`.",
  tracks: [
    { part: SHEET, on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: "the sheet's fade", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: CLOCK, on: ON, target: ".k-lock-clock", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: BLUR, on: ON, target: BLURRED, property: "--wake", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: DIM, on: ON, target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: held("awake"),
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
    "At rest the lock is the wallpaper and its clock, and nothing else: no frame, no bar, no field. The digits are `type.clock_rest`, Geist Light at 384, about 280 tall, under a kicker line `space.s16` above their tops: where and which day in `accent`, `primary` (東京・月曜日), then ` · ` and the date in `body_strong`, `on_surface_variant`. The kicker and the digits are each centred on the clock's middle, on whole device pixels with the odd one after. Every screen of a setup shows rest, each with its own clock.",
    "Each wallpaper says where its clock stands: the middle of the digits' baseline as shares of the picture, so the clock keeps its place against the city however a screen crops the picture, kept `space.s32` inside the screen's edges. A wallpaper with a foreground cut-out (an alpha picture of the city, the same size as the wallpaper) has it drawn over the digits, so the towers stand in front of the clock: on Shinjuku the skyline covers the digits' feet, on Tokyo Tower the spire crosses the last digit. The place keeps the kicker clear of the cut-out. A picture of your own has no cut-out, and the clock stands in front of it.",
    "Behind the clock lies a soft shade: `surface`, solid `space.s64` past the clock and fading to nothing over the next `space.s64`, at the least opacity, in hundredths, that keeps every word of the clock at 4.5:1 over the brightest of the picture under it (its 98th percentile by luminance, read off the picture itself). A dark sky like Tokyo Tower's needs none and gets none; a street of signs like Shibuya's gets a strong one. The cut-out stands in front of the shade too, so only the sky darkens.",
    "Awake, on the screen with the pointer only, the shell's frame closes in round the picture, `surface` at `opacity.frame` with the bar's height at the top and `space.s8` down the sides and along the bottom, as the desktop draws it, so unlocking leaves it where it is. The picture blurs to `size.lock_blur` (8) and dims to `scrim` at `opacity.panel_scrim`. The clock lifts out from behind the cut-out, which fades, and settles at `type.clock` (80), centred, its kicker's top `space.s48` under the frame's top; its shade follows it.",
    "A Sheet rises from the middle of the bottom edge, curving into the frame through concave fillets of `radius.surface`, as the wallpaper picker does, `size.panel` (384) wide with a PanelPage's padding: `space.s16` at its top and foot, the container's 16 at its sides. It holds the md `Field` for the password, its lock symbol leading in the field's icon colour and lit with the keyboard in it; `space.s4` under it, as a field's note is, the line led by the `fingerprint` symbol in `primary`, Or touch the fingerprint reader; and `space.s16` under that the key hints, ↵ Unlock and Esc Rest. No avatar, no initial, no emoji: only KubaOS Symbols.",
    "The lock ignores light and dark. It always draws in the night scheme matugen derives from its wallpaper (the same `scheme-fidelity` dot-theme uses, in dark), and its derived roles from those: Shinjuku's purple hour gives a lavender primary, Tokyo Tower's blue hour a pale blue one, Shibuya's rain a teal one. So the lock is the same in either mode, and its words keep their contrast over the night picture.",
  ],
  variants: [
    { name: "At rest, with a cut-out", for: "Shinjuku at purple hour: the skyline's towers stand in front of the digits' feet, the shade only as strong as the sky needs.", sample: () => still({ phase: "rest", wallpaper: PURPLE_HOUR }) },
    { name: "At rest, with a cut-out, a dark sky", for: "Tokyo Tower at blue hour: the spire crosses the last digit; the sky is dark enough that the shade is none.", sample: () => still({ phase: "rest", wallpaper: TOWER }) },
    { name: "At rest, without a cut-out", for: "Shibuya on a rainy night, as a picture of your own is: the clock in front, its shade strong enough for 4.5:1 over the signs.", sample: () => still({ phase: "rest", wallpaper: SHIBUYA }) },
    { name: "Awake", for: "The frame closed in, the picture blurred and dimmed, the clock at the top and the password's sheet up from the bottom edge.", sample: () => still({ phase: "awake", wallpaper: PURPLE_HOUR }) },
    { name: "Awake, without a cut-out", for: "The same on Shibuya, in its own night scheme.", sample: () => still({ phase: "awake", wallpaper: SHIBUYA }) },
    { name: "The 12-hour clock", for: "Where the locale or Settings ask for 12 hours: the digits without a meridiem (9:47), which takes its own slot at the kicker's end (· PM), so the digits keep one size and the clock its place.", sample: () => twelveHour("rest") },
    { name: "The 12-hour clock, awake", for: "The same once the clock is at the top.", sample: () => twelveHour("awake") },
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
    arrives: [LOCKS],
    leaves: [UNLOCKS],
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
      { name: "At rest", for: "The rest clock's size, the kicker's gap above the digits, and the shade round them.", sample: () => still({ phase: "rest" }) },
      { name: "Awake", for: "The frame, the clock at the top, and the sheet: its width, its padding and the gaps inside it.", sample: () => still({ phase: "awake" }) },
    ]),
    measures: [
      { what: "The rest clock's digits.", token: "type-clock_rest-size" },
      { what: "The awake clock's digits.", token: "type-clock-size" },
      { what: "The kicker's line.", token: "type-accent-line" },
      { what: "Between the kicker and the digits' tops; inside the sheet at its top and foot; between the reader's line and the key hints.", token: "space-s16" },
      { what: "How far the shade is solid past the clock, and how far it fades after that.", token: "space-s64" },
      { what: "The least room between the rest clock and the screen's edges.", token: "space-s32" },
      { what: "The frame's top, the bar's height, from where the awake clock is placed.", token: "size-bar" },
      { what: "Between the frame's top and the awake clock's kicker.", token: "space-s48" },
      { what: "The frame's sides and bottom.", token: "space-s8" },
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
