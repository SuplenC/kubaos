// The Sparkline page: `docs/design/components/sparkline.md`. The last minute of a reading as a line,
// filled under it, newest at the right edge: the system monitor's graphs and the bar's pill.

import { shown, type Words } from "../../lib/content.ts";
import { figure, tick } from "../../lib/figure.ts";
import { icon } from "../../lib/icon.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { type Level, minute, type Sample, SECOND, sparkline, tint } from "../../lib/meter.ts";
import { clamped, text } from "../../lib/text.ts";
import { CHROME } from "../../site/settings.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { CARD_ROOM } from "./meter.ts";

/** When every still sample's minute ends, so a sample draws the same each time. */
const END = 3_600_000;

const NAME: Words = { short: "CPU", long: "AMD Ryzen 7 PRO 7840U with Radeon 780M Graphics, all 16 threads" };

/** A CPU's busy percent over a minute: a steady hum with a burst in it. */
const busy = (second: number) => 24 + 14 * Math.sin(second / 6) + 9 * Math.sin(second * 1.3) + (second % 60 > 38 && second % 60 < 46 ? 30 : 0);
/** A download's rate and the upload under it, in MB/s: fitted, so no percent. */
const down = (second: number) => Math.max(0, 6 + 5 * Math.sin(second / 4) + 3 * Math.sin(second * 2.1));
const up = (second: number) => Math.max(0, 1.2 + Math.sin(second / 3));
/** A temperature near its threshold, in °C, on a scale to the threshold and 10 more. */
const hot = (second: number) => 82 + 4 * Math.sin(second / 5) + 2 * Math.sin(second * 1.7);

/** A system monitor card's graph: its reading's line above it, `space.s8` apart. */
function graph(name: string, reading: string, step: Level, samples: readonly Sample[], options: { second?: readonly Sample[]; max?: number } = {}): HTMLElement {
  const line = filling(row("s12", "centre", filling(clamped(name, "label")), text(reading, "mono", tint(step, "on_surface"))));
  const drawn = sparkline({ samples, color: tint(step, "primary"), ...(options.second ? { second: options.second } : {}), ...(options.max !== undefined ? { max: options.max } : {}) });
  return within(CARD_ROOM, stack("s8", "start", line, drawn.box));
}

/** A reading in the bar's pill: its symbol and its sparkline, a small control wide and 16 tall. */
function inPill(glyph: string, samples: readonly Sample[], step: Level, max = 100): HTMLElement {
  const drawn = sparkline({ samples, inline: true, max, color: step > 0 ? tint(step, "primary") : "primary" });
  return row("s4", "centre", icon(glyph, "var(--icon-s16)", `var(--color-${tint(step, "on_surface_variant")})`), drawn.box);
}

/** The CPU's graph, a sample a second as the system monitor takes them while its panel is open. */
function live(): HTMLElement {
  const now = () => Date.now();
  const first = minute(CHROME ? now() : END, busy);
  const drawn = sparkline({ samples: first });
  const reading = figure("0%", "mono");
  const read = (samples: readonly Sample[]) => tick(reading, "mono", `${Math.round(samples.at(-1)?.[1] ?? 0).toLocaleString()}%`);
  read(first);
  if (CHROME) {
    const timer = window.setInterval(() => {
      if (!drawn.box.isConnected) {
        window.clearInterval(timer);
        return;
      }
      const samples = minute(now(), busy);
      drawn.update(samples);
      read(samples);
    }, SECOND);
  }
  const line = filling(row("s12", "centre", filling(clamped("CPU", "label")), reading));
  return within(CARD_ROOM, stack("s8", "start", line, drawn.box));
}

/** Only the last `seconds` of a minute: a graph opened a moment ago. */
function since(seconds: number, shape: (second: number) => number): Sample[] {
  return minute(END, shape).slice(-seconds - 1);
}

const DESCRIBED =
  "The last minute of a reading as a line in the thin stroke (`stroke.thin`, whole device pixels), round at its joins, filled under it with the line's colour at `opacity.tint`. Each sample is placed by its time over the last minute, the newest at the right edge, so a graph opened a moment ago starts part way across. A second reading over it, unfilled, is `tertiary`: upload over download. The scale tops out at 100, or fits the highest sample with 15% room above it, so the line never runs along the top edge.";

export default page<"rest" | "error" | "empty">({
  level: "molecules",
  slug: "sparkline",
  title: "Sparkline",
  summary: "The last minute of a reading as a line, filled under it, newest at the right edge: the system monitor's graphs.",
  live,
  description: [
    "Rust `meter::sparkline(samples)` with `second`, `max`, `colour` and `height`, drawn by `view`; QML `Sparkline` (`points` as `[t, value]` pairs, `second`, `max`, `span`, `color`, `revision`). The system monitor's CPU, temperature, GPU, network and power graphs, and the bar's pill when its style is sparklines. Here a sample comes each second, as the system monitor takes them while its panel is open.",
    DESCRIBED,
    "Its colour steps with its reading, as a meter's does: `primary` while it is fine, `warning` within its margin of its threshold, `error` past it, the fixed palette's text roles held at 3:1 as marks on every surface; the CPU's graph follows its temperature's step, the reading the card is about. As wide as its room and `size.sparkline` (48) tall in a system monitor card; in the bar's pill a small control wide and a line of small text tall, beside its 16 symbol.",
    "It draws when a sample comes, never per frame: nothing about it moves between samples, so it costs nothing while the reading holds still. It has no glow and takes no focus; its card's tooltip says the reading in words.",
  ],
  variants: [
    { name: "One reading", for: "The CPU, a percent on a scale to 100.", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
    { name: "Two readings", for: "The network: download filled, upload over it in `tertiary`, on a fitted scale.", sample: () => graph("Network", "7.4 MB/s", 0, minute(END, down), { second: minute(END, up), max: 0 }) },
    { name: "In the pill", for: "The bar's pill, its style sparklines: a small control wide beside its symbol.", sample: () => row("s12", "centre", inPill("memory", minute(END, busy), 0), inPill("thermostat", minute(END, hot), 1, 100)) },
  ],
  sizes: [
    { name: "size.sparkline", for: "A system monitor card: 48 tall across the card.", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
    { name: "Inline", for: "The bar's pill: `small`'s line tall, a small control wide.", sample: () => inPill("memory", minute(END, busy), 0) },
  ],
  states: {
    shown: ["rest", "error", "empty"],
    absent: {
      lit: "it takes neither the pointer nor the keyboard.",
      pressed: "it takes no press.",
      disabled: "a reading the machine lacks leaves its graph out.",
      selected: "it holds no choice.",
      loading: "it draws what it has from the first two samples, starting part way across; with one, nothing yet.",
    },
    rows: [
      { name: "CPU", cell: (state) => (state === "empty" ? graph("CPU", "–", 0, since(0, busy)) : state === "error" ? graph("CPU", "94 °C", 2, minute(END, (second) => 52 + busy(second) * 0.6)) : graph("CPU", "31%", 0, minute(END, busy))) },
    ],
    described: {
      rest: `Fine, in \`primary\`; close to its threshold, \`warning\`. ${DESCRIBED} By day the same roles in daylight colours.`,
      error: "Past its threshold: the line and its fill in `error`, the fixed palette's red on every wallpaper, and the reading's figure with it.",
      empty: "Fewer than two samples: nothing drawn, its room kept, so the card doesn't move when the line arrives.",
    },
  },
  focus: doesntApply("a sparkline isn't a control and has no light: it takes no focus and nothing glows."),
  content: {
    text: [{ name: "The reading's name", words: NAME, sample: (words: string) => graph(words, "31%", 0, minute(END, busy)) }],
    missingPicture: doesntApply("a sparkline is drawn."),
    counts: [
      { name: "None or one", for: "Too few to draw a line: its room alone.", sample: () => graph("CPU", "–", 0, since(0, busy)) },
      { name: "A few seconds", for: "A graph opened a moment ago: it starts part way across.", sample: () => graph("CPU", "31%", 0, since(12, busy)) },
      { name: "A minute", for: "The whole minute across.", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
      { name: "Close", for: "Within its margin of its threshold: `warning`.", sample: () => graph("Temperature", "84 °C", 1, minute(END, hot), { max: 100 }) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its card, drawn from the samples it has."),
    leaves: doesntApply("it goes with its card."),
    between: powerMoves(() => graph("CPU", "31%", 0, minute(END, busy))),
  },
  keyboard: doesntApply("a sparkline takes no key; its card's reading is read out."),
  measures: {
    samples: nonEmpty([
      { name: "In a card", for: "Its height and the line above it.", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
      { name: "In the pill", for: "Its width and height beside its symbol.", sample: () => inPill("memory", minute(END, busy), 0) },
    ]),
    measures: [
      { what: "Its height in a card.", token: "size-sparkline" },
      { what: "Its height in the bar's pill: `small`'s line.", token: "type-small-line" },
      { what: "Its width in the bar's pill.", token: "size-control_sm" },
      { what: "Its line, before it is rounded to whole device pixels.", token: "stroke-thin" },
      { what: "Between it and its reading's line above it.", token: "space-s8" },
      { what: "Between its symbol and it, in the pill.", token: "space-s4" },
    ],
    reflow: {
      described: "In a card it fills the card at every window size, the same minute spread over the width; in the pill it keeps its small control's width.",
      sample: () => within("100%", stack("s8", "start", filling(row("s12", "centre", filling(clamped(shown(NAME), "label")), text("31%", "mono"))), sparkline({ samples: minute(END, busy) }).box, text("A sparkline as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
