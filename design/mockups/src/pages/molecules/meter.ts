// The Meter page: `docs/design/components/meter.md`. A reading in parts on a progress bar's track,
// as the system monitor and the bar's pill show memory, disks and the battery; its colour steps to
// `warning` and `error` as a reading nears and passes its threshold.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { figure, tick } from "../../lib/figure.ts";
import { icon } from "../../lib/icon.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { type Level, level, meter, type MeterPart, setMeter, tint } from "../../lib/meter.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";

/** A system monitor card's room: the shell's panel less its padding and the card's. */
export const CARD_ROOM = "calc(var(--size-panel) - 4 * var(--space-s16))";

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
  return stack("s24", "start", within(CARD_ROOM, stack("s8", "start", head, bar)), more);
}

const RISES: Move = {
  name: "A new reading",
  what: "A sample comes: each part eases from where it ended to where it ends now over `short` on `linear`, as a progress bar follows its work, every part from the left edge so each keeps its rounded end. A part's colour changes at once when its reading crosses a step. Reduce motion caps it at `short`, so it plays the same.",
  tracks: [
    { part: "the used part", on: "memory", target: ".k-meter-part:last-child", property: "--reach", from: 0.42, to: 0.64, duration: "short", curve: "linear" },
    { part: "the cache after it", on: "memory", target: ".k-meter-part:first-child", property: "--reach", from: 0.6, to: 0.82, duration: "short", curve: "linear" },
  ],
  reduced: "same",
  back: "cuts",
  sample: () => within(CARD_ROOM, meter([{ share: 0.42, color: "primary" }, { share: 0.18, color: "tertiary" }])),
};

const DESCRIBED =
  "A progress bar's track and shape: `size.progress` (8) tall across its room at `radius.full`, the track `surface_container_highest` at `opacity.track`. Each part is a share and its colour, left to right (memory as used, then cache), drawn from the left edge to where it ends, rounded up to a device pixel, the later ones under the earlier, so every part keeps the bar's rounded ends. It has no glow: a reading isn't work going on.";

export default page<"rest" | "error" | "empty">({
  level: "molecules",
  slug: "meter",
  title: "Meter",
  summary: "A reading in parts on a progress bar's track: memory, a disk, the battery, stepping to warning and error near and past its limit.",
  live,
  description: [
    "Rust `meter::meter(look, parts)`, QML `Meter` (`parts`, each `{value, color}`). The system monitor's memory, GPU memory, disks and battery, and the bar's pill when its style is bars. Press Open another tab: memory fills, steps to `warning` within 10 of its threshold and to `error` past it, then frees up again.",
    DESCRIBED,
    "A reading takes its colour step from its threshold in `sysmon.toml`: fine, it is `primary`; within its margin of the threshold (10 for memory and the CPU's temperature, 5 for a disk and the battery's draw), `warning`; past it, `error`. Those are the fixed palette's text roles, the same on every wallpaper, held at 3:1 as marks on every surface (`[contrast]`), so a reading in trouble reads as trouble whatever the accent. The reading's own figure beside it takes the same step; a second part (cache) keeps `tertiary`. A battery under 15% is `error`.",
    "A new reading eases each part over `short` on `linear`; the colour changes at once. In the bar's pill it is a small control wide (`size.control_sm`), beside its 16 symbol, `space.s4` apart; in a system monitor card it fills the card, its reading's line above it and its words under it, `space.s8` apart.",
  ],
  variants: [
    { name: "In parts", for: "A reading that splits: memory as used, then cache in `tertiary`.", sample: () => memory(42, 18) },
    { name: "One part", for: "A disk as used.", sample: () => disk("System", 38) },
    { name: "In the pill", for: "The bar's pill, its style bars: each reading a small control wide beside its symbol.", sample: () => row("s12", "centre", inPill("memory", 23, 0), inPill("memory_alt", 84, 1), inPill("thermostat", 96, 2)) },
  ],
  sizes: [
    { name: "Its room", for: "A system monitor card: across the card.", sample: () => disk("System", 38) },
    { name: "control_sm", for: "The bar's pill: a small control wide, beside a 16 symbol.", sample: () => inPill("memory_alt", 46, 0) },
  ],
  states: {
    shown: ["rest", "error", "empty"],
    absent: {
      lit: "it takes neither the pointer nor the keyboard; a system monitor card's tooltip is the card's.",
      pressed: "it takes no press.",
      disabled: "a reading the machine lacks leaves its line out rather than drawing a meter.",
      selected: "it holds no choice.",
      loading: "before the first sample its line is a placeholder that the reading fades in over; the meter is drawn empty, so nothing moves.",
    },
    rows: [
      { name: "Memory", cell: (state) => (state === "empty" ? memory(0, 0) : state === "error" ? memory(93, 5) : memory(42, 18)) },
      { name: "A disk", cell: (state) => (state === "empty" ? disk("System", 0) : state === "error" ? disk("System", 97) : disk("System", 38)) },
    ],
    described: {
      rest: `Fine: ${DESCRIBED} Close to its threshold the part and its figure are \`warning\`. By day the same roles in daylight colours.`,
      error: "Past its threshold: the part and its figure in `error`, the fixed palette's red on every wallpaper; cache keeps `tertiary`.",
      empty: "Nothing used, or before the first sample: the track alone.",
    },
  },
  focus: doesntApply("a meter isn't a control and has no light: it takes no focus and nothing glows."),
  content: {
    text: [{ name: "The reading's name", words: DISK, sample: (words: string) => disk(words, 72) }],
    missingPicture: doesntApply("a meter is drawn."),
    counts: [
      { name: "Fine", for: "Under its threshold less its margin: `primary`.", sample: () => memory(42, 18) },
      { name: "Close", for: "Within its margin of its threshold: `warning`.", sample: () => memory(84, 10) },
      { name: "Over", for: "Past its threshold: `error`.", sample: () => memory(93, 5) },
      { name: "Full", for: "Every part together fills it; a share past the end is cut there.", sample: () => memory(80, 40) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its card, at the reading it has."),
    leaves: doesntApply("it goes with its card."),
    between: [RISES, ...powerMoves(() => memory(42, 18))],
  },
  keyboard: doesntApply("a meter takes no key; its reading's figure beside it is read out."),
  measures: {
    samples: nonEmpty([
      { name: "In a card", for: "Its height, the line above it and the words under it.", sample: () => memory(42, 18) },
      { name: "In the pill", for: "Its width beside its symbol.", sample: () => inPill("memory_alt", 46, 0) },
    ]),
    measures: [
      { what: "Its height.", token: "size-progress" },
      { what: "Its width in the bar's pill.", token: "size-control_sm" },
      { what: "The symbol beside it in the pill.", token: "icon-s16" },
      { what: "Between the symbol and it, in the pill.", token: "space-s4" },
      { what: "Between it and the lines above and under it, in a card.", token: "space-s8" },
      { what: "Between the reading's name and its figure.", token: "space-s12" },
    ],
    reflow: {
      described: "In a card it fills the card at every window size, its parts the same shares of it; in the pill it keeps its small control's width.",
      sample: () => within("100%", stack("s8", "start", named(shown(DISK), "72%", 0), meter([{ share: 0.72, color: "primary" }]), text("A meter as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
