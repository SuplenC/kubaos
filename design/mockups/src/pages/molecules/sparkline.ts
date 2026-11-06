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
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { CARD_ROOM } from "./meter.ts";

/** When every still sample's minute ends, so a sample draws the same each time. */
const END = 3_600_000;

const NAME: Words = { short: "CPU", long: "AMD Ryzen 7 PRO 7840U with Radeon 780M Graphics, all 16 threads" };

/** The stage's room: a column of text, so the minute reads across a wide graph. */
const COLUMN = "var(--size-column)";

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
  return within(COLUMN, stack("s8", "start", line, drawn.box));
}

/** Only the last `seconds` of a minute: a graph opened a moment ago. */
function since(seconds: number, shape: (second: number) => number): Sample[] {
  return minute(END, shape).slice(-seconds - 1);
}

export default page<"rest" | "error" | "empty">({
  level: "molecules",
  slug: "sparkline",
  title: "Sparkline",
  lede: "The last minute of a reading as a line, filled under it, newest at the right edge.",
  code: { rust: "meter::sparkline", qml: "Sparkline", unbuilt: "Rust time spacing: not built yet" },
  stage: { live, hint: "a sample comes each second", daylight: true },
  variants: [
    { name: "One reading", for: "a percent on a scale to 100", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
    { name: "Two readings", for: "upload over download in `tertiary`, fitted scale", sample: () => graph("Network", "7.4 MB/s", 0, minute(END, down), { second: minute(END, up), max: 0 }) },
    { name: "In the pill", for: "the bar's pill: a small control wide", sample: () => row("s12", "centre", inPill("memory", minute(END, busy), 0), inPill("thermostat", minute(END, hot), 1, 100)) },
  ],
  sizes: [
    { name: "size.sparkline", for: "a system monitor card, 48 tall", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
    { name: "Inline", for: "`small`'s line tall, a small control wide", sample: () => inPill("memory", minute(END, busy), 0) },
  ],
  states: {
    shown: ["rest", "error", "empty"],
    absent: {
      lit: "it takes neither pointer nor keyboard",
      pressed: "it takes no press",
      disabled: "a missing reading leaves its graph out",
      selected: "it holds no choice",
      loading: "draws from two samples, part way across",
    },
    rows: [
      { name: "CPU", cell: (state) => (state === "empty" ? graph("CPU", "–", 0, since(0, busy)) : state === "error" ? graph("CPU", "94 °C", 2, minute(END, (second) => 52 + busy(second) * 0.6)) : graph("CPU", "31%", 0, minute(END, busy))) },
    ],
    looks: {
      rest: "`primary`; within its margin, `warning`; no glow",
      error: "line, fill and figure in `error`",
      empty: "under two samples: nothing drawn, its room kept",
    },
  },
  focus: doesntApply("a sparkline isn't a control: no focus, no glow."),
  content: {
    text: [{ name: "The reading's name", words: NAME, sample: (words: string) => graph(words, "31%", 0, minute(END, busy)) }],
    missingPicture: doesntApply("a sparkline is drawn."),
    counts: [
      { name: "None or one", for: "too few for a line: its room alone", sample: () => graph("CPU", "–", 0, since(0, busy)) },
      { name: "A few seconds", for: "opened a moment ago: starts part way across", sample: () => graph("CPU", "31%", 0, since(12, busy)) },
      { name: "A minute", for: "the whole minute across", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
      { name: "Close", for: "within its margin: `warning`", sample: () => graph("Temperature", "84 °C", 1, minute(END, hot), { max: 100 }) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its card, drawn from its samples."),
    leaves: doesntApply("it goes with its card."),
    between: powerMoves(() => graph("CPU", "31%", 0, minute(END, busy))),
  },
  keyboard: doesntApply("a sparkline takes no key; its card's reading is read out."),
  measures: {
    samples: nonEmpty([
      { name: "In a card", for: "height, line above", sample: () => graph("CPU", "31%", 0, minute(END, busy)) },
      { name: "In the pill", for: "width and height beside its symbol", sample: () => inPill("memory", minute(END, busy), 0) },
    ]),
    measures: [
      { part: "Height in a card", token: "size-sparkline" },
      { part: "Height in the pill", token: "type-small-line" },
      { part: "Width in the pill", token: "size-control_sm" },
      { part: "Line", token: "stroke-thin" },
      { part: "Reading → graph", token: "space-s8" },
      { part: "Symbol → graph", token: "space-s4" },
    ],
    reflow: {
      note: "Fills a card at every size, the same minute spread over it; in the pill it keeps a small control's width.",
      sample: () => within("100%", stack("s8", "start", filling(row("s12", "centre", filling(clamped(shown(NAME), "label")), text("31%", "mono"))), sparkline({ samples: minute(END, busy) }).box)),
    },
  },
});
