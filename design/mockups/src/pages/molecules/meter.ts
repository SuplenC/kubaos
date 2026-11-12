// The Meter page: `docs/design/components/meter.md`. A reading in parts on a progress bar's track,
// as the system monitor and the bar's pill show memory, disks and the battery; its colour steps to
// `warning` and `error` as a reading nears and passes its threshold. The live variant is the audio
// panel's mic level, fed here by a simulated voice.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { figure, tick } from "../../lib/figure.ts";
import { icon } from "../../lib/icon.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { type Level, level, liveMeter, meter, type MeterPart, setMeter, tint } from "../../lib/meter.ts";
import { slider } from "../../lib/slider.ts";
import { CHROME } from "../../site/settings.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** A system monitor card's room: the shell's panel less its padding and the card's. */
export const CARD_ROOM = "calc(var(--size-panel) - 4 * var(--space-s16))";

/** The stage's room: a column of text, so the steps read across a wide bar. */
const COLUMN = "var(--size-column)";

/** A shell panel's room for its rows: the panel less its padding either side, as the audio panel's. */
const PANEL = "calc(var(--size-panel) - 2 * var(--space-s16))";

/** Memory's threshold and how close counts as close, in percent, as the system monitor's. */
const RAM = { threshold: 90, margin: 10 };

const DISK: Words = { short: "Data", long: "Samsung 990 Pro 2 TB, mounted at /run/media/kuba/projects" };

/** A reading's line, as a system monitor card shows it: its name, then the reading in its step. */
function named(name: string, reading: string, step: Level): HTMLElement {
  return filling(row("s12", "centre", filling(clamped(name, "label")), text(reading, "mono", tint(step, "on_surface"))));
}

/** Memory at `used` percent with `cache` percent more, as a card's line, its meter and its words. */
function memory(used: number, cache: number): HTMLElement {
  const step = level(used, RAM.threshold, RAM.margin);
  const parts: MeterPart[] = [
    { share: used / 100, color: tint(step, "primary") },
    { share: cache / 100, color: "tertiary" },
  ];
  return within(CARD_ROOM, stack("s8", "start", named("Memory", `${used.toLocaleString()}%`, step), meter(parts), wrapped(`${((used / 100) * 32).toLocaleString(undefined, { maximumFractionDigits: 1 })} GB used · ${((cache / 100) * 32).toLocaleString(undefined, { maximumFractionDigits: 1 })} GB cache of 32 GB`, "small", "on_surface_variant")));
}

/** A disk `percent` full, its step against the disk's narrower margin. */
function disk(name: string, percent: number): HTMLElement {
  const step = level(percent, 95, 5);
  return within(CARD_ROOM, stack("s8", "start", named(name, `${percent.toLocaleString()}%`, step), meter([{ share: percent / 100, color: tint(step, "primary") }])));
}

/** A reading in the bar's pill: its symbol and a meter a small control wide, in its step's tone. */
function inPill(glyph: string, percent: number, step: Level): HTMLElement {
  const bar = meter([{ share: percent / 100, color: step > 0 ? tint(step, "primary") : "primary" }]);
  bar.dataset.compact = "";
  return row("s4", "centre", icon(glyph, "var(--icon-s16)", `var(--color-${tint(step, "on_surface_variant")})`), bar);
}

/** Memory filling up as it is used: each press takes more, past the threshold and back to fine. */
function live(): HTMLElement {
  const readings = [42, 64, 82, 88, 93, 97, 51];
  let at = 0;
  const bar = meter([
    { share: readings[at]! / 100, color: "primary" },
    { share: 0.18, color: "tertiary" },
  ]);
  const reading = figure("42%", "mono");
  const draw = () => {
    const used = readings[at]!;
    const step = level(used, RAM.threshold, RAM.margin);
    setMeter(bar, [
      { share: used / 100, color: tint(step, "primary") },
      { share: Math.min(0.18, 1 - used / 100), color: "tertiary" },
    ]);
    tick(reading, "mono", `${used.toLocaleString()}%`);
    (reading.firstElementChild as HTMLElement).style.color = `var(--color-${tint(step, "on_surface")})`;
  };
  draw();
  const head = filling(row("s12", "centre", filling(clamped("Memory", "label")), reading));
  const more = button({ label: "Open another tab", kind: "Tonal", size: "Sm", icon: "add" });
  more.addEventListener("click", () => {
    at = (at + 1) % readings.length;
    draw();
  });
  return stack("s24", "start", within(COLUMN, stack("s8", "start", head, bar)), more);
}

/**
 * A voice as a mic hears it, the level at a time in milliseconds: words of a few syllables, each
 * rising and falling, short gaps between syllables and longer pauses between words, over a little
 * room noise. Seeded, so it says the same each time the page is drawn.
 */
export function voice(): (now: number) => number {
  let seed = 0x5eed;
  const random = () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let mixed = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    mixed = (mixed + Math.imul(mixed ^ (mixed >>> 7), 61 | mixed)) ^ mixed;
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296;
  };
  const between = (low: number, high: number) => low + (high - low) * random();
  // The sound being made now: a syllable at its loudness, or silence, from `start` to `end`.
  let start = 0;
  let end = 0;
  let loud = 0;
  let left = 0;
  return (now) => {
    while (now >= end) {
      start = end;
      if (loud > 0) {
        loud = 0;
        end = start + (left > 0 ? between(30, 90) : between(250, 900));
      } else {
        if (left === 0) left = Math.round(between(2, 5));
        left -= 1;
        loud = between(0.35, 0.85);
        end = start + between(90, 260);
      }
    }
    const room = between(0.02, 0.05);
    if (loud === 0) return room;
    const through = (now - start) / (end - start);
    return Math.max(room, loud * Math.sin(Math.PI * through) ** 0.6 * between(0.85, 1));
  };
}

/**
 * The audio panel's mic: its compact slider, whose icon mutes it, over its live meter, fed by a
 * voice while it is on screen. A shot draws one still frame of it.
 */
function mic(): HTMLElement {
  const gain = 0.8;
  const meterOf = liveMeter(CHROME ? 0 : 0.46, CHROME ? 0 : 0.68);
  let louder = gain;
  const control = slider({
    icon: "mic",
    size: "Compact",
    value: gain,
    toggle: { icon: "mic_off", words: "Muted", label: "Mute", toggledLabel: "Unmute", changed: (on) => meterOf.mute(on) },
    changed: (value) => {
      louder = value;
    },
  });
  const box = within(PANEL, stack("s8", "start", control, meterOf.bar));
  if (!CHROME) return box;
  const says = voice();
  let frame = 0;
  const speak = (now: number) => {
    meterOf.feed(says(now) * (louder / gain), now);
    frame = requestAnimationFrame(speak);
  };
  new IntersectionObserver(([entry]) => {
    cancelAnimationFrame(frame);
    if (entry?.isIntersecting && box.isConnected) frame = requestAnimationFrame(speak);
  }).observe(box);
  return box;
}

/** A live meter in a panel's room, still at `level` with its tick at `peak`. */
function still(level: number, peak: number): HTMLElement {
  return within(PANEL, liveMeter(level, peak).bar);
}

/** A muted mic's live meter: the track alone. */
function silent(): HTMLElement {
  const muted = liveMeter();
  muted.mute(true);
  return within(PANEL, muted.bar);
}

const PEAK: Move = {
  name: "A live level, its peak falling",
  what: "The level jumps with no easing; the tick holds for `slow`, then falls the whole bar per `scene`, on `linear`.",
  tracks: [
    { part: "the level", on: "a live meter", target: ".k-meter-part", property: "--reach", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the peak tick", on: "a live meter", target: ".k-meter", property: "--meter-peak", from: 1, to: 0, duration: "scene", cue: "slow", curve: "linear" },
  ],
  reduced: "same",
  back: "cuts",
  sample: () => still(1, 1),
};

const RISES: Move = {
  name: "A new reading",
  what: "Each part eases to its new reach over `short` on `linear`; a part's colour changes at once.",
  tracks: [
    { part: "the used part", on: "memory", target: ".k-meter-part:last-child", property: "--reach", from: 0.42, to: 0.64, duration: "short", curve: "linear" },
    { part: "the cache after it", on: "memory", target: ".k-meter-part:first-child", property: "--reach", from: 0.6, to: 0.82, duration: "short", curve: "linear" },
  ],
  reduced: "same",
  back: "cuts",
  sample: () => within(CARD_ROOM, meter([{ share: 0.42, color: "primary" }, { share: 0.18, color: "tertiary" }])),
};

export default page<"rest" | "error" | "empty">({
  level: "molecules",
  slug: "meter",
  title: "Meter",
  lede: "A reading in parts on a progress bar's track, stepping to `warning` and `error` near and past its limit.",
  code: { rust: "meter::meter", qml: "Meter", unbuilt: "Rust easing, live: not built yet" },
  stage: { live: () => stack("s32", "start", live(), mic()), hint: "press Open another tab; mute the mic", daylight: true },
  variants: [
    { name: "In parts", for: "a reading that splits: used, then cache in `tertiary`", sample: () => memory(42, 18) },
    { name: "One part", for: "a disk as used", sample: () => disk("System", 38) },
    { name: "In the pill", for: "the bar's pill: a small control wide beside its symbol", sample: () => row("s12", "centre", inPill("memory", 23, 0), inPill("memory_alt", 84, 1), inPill("thermostat", 96, 2)) },
    { name: "Live", for: "a mic's level as it comes: no easing, a peak tick", sample: () => still(0.46, 0.68) },
  ],
  sizes: [
    { name: "Its room", for: "a system monitor card: across the card", sample: () => disk("System", 38) },
    { name: "control_sm", for: "the bar's pill: beside a 16 symbol", sample: () => inPill("memory_alt", 46, 0) },
  ],
  states: {
    shown: ["rest", "error", "empty"],
    absent: {
      lit: "it takes neither pointer nor keyboard",
      pressed: "it takes no press",
      disabled: "a missing reading leaves its line out",
      selected: "it holds no choice",
      loading: "drawn empty before the first sample, so nothing moves",
    },
    rows: [
      { name: "Memory", cell: (state) => (state === "empty" ? memory(0, 0) : state === "error" ? memory(93, 5) : memory(42, 18)) },
      { name: "A disk", cell: (state) => (state === "empty" ? disk("System", 0) : state === "error" ? disk("System", 97) : disk("System", 38)) },
      { name: "A live mic", cell: (state) => (state === "empty" ? silent() : still(0.46, 0.68)) },
    ],
    looks: {
      rest: "`primary`, cache `tertiary`; within its margin, `warning`",
      error: "part and figure in `error`; cache keeps `tertiary`; live has none",
      empty: "the track alone; a muted mic's, no tick",
    },
  },
  focus: doesntApply("a meter isn't a control: no focus, no glow."),
  content: {
    text: [{ name: "The reading's name", words: DISK, sample: (words: string) => disk(words, 72) }],
    missingPicture: doesntApply("a meter is drawn."),
    counts: [
      { name: "Fine", for: "under its threshold less margin: `primary`", sample: () => memory(42, 18) },
      { name: "Close", for: "within its margin: `warning`", sample: () => memory(84, 10) },
      { name: "Over", for: "past its threshold: `error`", sample: () => memory(93, 5) },
      { name: "Full", for: "parts fill it; a share past the end is cut", sample: () => memory(80, 40) },
      { name: "Live, quiet", for: "room noise, the tick near the start", sample: () => still(0.04, 0.12) },
      { name: "Live, at full", for: "the tick cut by the round end", sample: () => still(0.9, 1) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its card, at the reading it has."),
    leaves: doesntApply("it goes with its card."),
    between: [RISES, PEAK, ...powerMoves(() => memory(42, 18))],
  },
  keyboard: doesntApply("a meter takes no key; its figure beside it is read out."),
  measures: {
    samples: nonEmpty([
      { name: "In a card", for: "height, line above, words under", sample: () => memory(42, 18) },
      { name: "In the pill", for: "width beside its symbol", sample: () => inPill("memory_alt", 46, 0) },
      { name: "Live, under the mic", for: "the peak tick; slider → meter", sample: () => mic() },
    ]),
    measures: [
      { part: "Height", token: "size-progress" },
      { part: "Width in the pill", token: "size-control_sm" },
      { part: "Symbol in the pill", token: "icon-s16" },
      { part: "Symbol → meter, in the pill", token: "space-s4" },
      { part: "Meter → lines, in a card", token: "space-s8" },
      { part: "Name → figure", token: "space-s12" },
      { part: "Peak tick", token: "stroke-thin" },
      { part: "Mic slider → live meter", token: "space-s8" },
    ],
    reflow: {
      note: "Fills a card at every size, its parts the same shares; in the pill it keeps a small control's width.",
      sample: () => within("100%", stack("s8", "start", named(shown(DISK), "72%", 0), meter([{ share: 0.72, color: "primary" }]))),
    },
  },
});
