// The Figure page: `docs/design/components/figure.md`. A number that ticks, in a box as wide as the
// same text with every digit a 0, so what sits beside it holds still as its digits change. QML
// only, the clock popup's; it isn't a control and has no states of its own.

import type { Words } from "../../lib/content.ts";
import { figure, tick } from "../../lib/figure.ts";
import { row, stack } from "../../lib/layout.ts";
import { advance, text, type TypeStyle } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { CHROME } from "../../site/settings.ts";

const SPECIMEN: readonly (readonly [string, TypeStyle])[] = [
  ["21:47", "hero_number"],
  ["4:59", "headline"],
  ["1:02:11.4", "body_strong"],
];

const TIME: Words = { short: "0:09", long: "1:02:11.4" };

// The locale's digits and separators: a time is never padded by hand. `en-GB` is the mockups'
// locale, so a shot reads the same on every machine.
const WHOLE = new Intl.NumberFormat("en-GB", { minimumIntegerDigits: 2, useGrouping: false });
const TENTHS = new Intl.NumberFormat("en-GB", { minimumIntegerDigits: 2, minimumFractionDigits: 1, maximumFractionDigits: 1, useGrouping: false });
const LEADING = new Intl.NumberFormat("en-GB", { useGrouping: false });

/**
 * Tenths of a second as a stopwatch shows them, m:ss.t, or h:mm:ss.t from an hour: each part
 * through the locale's number formatting, as `Intl.DurationFormat`'s digital style lays them out.
 */
function stopwatch(tenths: number): string {
  const seconds = Math.floor(tenths / 10);
  const [h, m] = [Math.floor(seconds / 3600), Math.floor(seconds / 60) % 60];
  const s = TENTHS.format((tenths % 600) / 10);
  return h > 0 ? `${LEADING.format(h)}:${WHOLE.format(m)}:${s}` : `${LEADING.format(m)}:${s}`;
}

/**
 * A stopwatch from `from` tenths, ticking in real time while it is on the page; a shot holds it at
 * `from`: in a figure when `held`, or in a plain text for the comparison.
 */
function ticking(from: number, style: TypeStyle, held: boolean): HTMLElement {
  const box = held ? figure(stopwatch(from), style) : text(stopwatch(from), style);
  if (!CHROME) return box;
  const start = performance.now();
  const timer = setInterval(() => {
    if (!box.isConnected) {
      clearInterval(timer);
      return;
    }
    const words = stopwatch(from + Math.floor((performance.now() - start) / 100));
    if (held) {
      tick(box, style, words);
    } else {
      box.textContent = words;
      box.style.setProperty("--natural", String(advance(style, words)));
    }
  }, 100);
  return box;
}

/** A stopwatch with the word after it: the word holds still beside a figure, and jitters beside a text. */
function beside(held: boolean): HTMLElement {
  return row("s8", "centre", ticking(3, "headline", held), text(held ? "held by a Figure" : "a plain Text", "body", "on_surface_variant"));
}

export default page({
  level: "atoms",
  slug: "figure",
  title: "Figure",
  lede: "A number that ticks, in a box as wide as its widest, so what sits beside it holds still.",
  code: { qml: "Figure", unbuilt: "Rust: not built yet" },
  stage: {
    live: () => row("s64", "centre", figure("21:47", "hero_number"), stack("s16", "start", beside(true), beside(false))),
    hint: "watch the words beside each stopwatch",
    daylight: true,
  },
  variants: nonEmpty(
    SPECIMEN.map(([words, style]) => ({
      name: style,
      for: style === "hero_number" ? "the clock popup's time" : style === "headline" ? "a timer counting down" : "a stopwatch, to the tenth",
      sample: () => figure(words, style),
    })),
  ),
  sizes: doesntApply("its style's size: the variants are the clock popup's styles."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "no pointer and no focus",
      pressed: "the timer's buttons take the press",
      disabled: "a stopped timer still shows its time",
      selected: "it holds no choice",
      loading: "a time is known at once",
      empty: "a timer that isn't set shows 0:00",
      error: "it has no value to be wrong",
    },
    rows: nonEmpty(SPECIMEN.map(([words, style]) => ({ name: style, cell: () => figure(words, style) }))),
    looks: {
      rest: "a `Text` in a box as wide as every digit a 0",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow."),
  content: {
    text: [{ name: "A stopwatch", words: TIME, sample: (words: string) => row("s8", "centre", figure(words, "headline"), text("elapsed", "body", "on_surface_variant")) }],
    missingPicture: doesntApply("a figure is words."),
    counts: [
      { name: "Minutes", for: "59:59, as wide as 00:00", sample: () => figure("59:59", "headline") },
      { name: "An hour", for: "1:00:00: the shape changes, so it grows, once", sample: () => figure("1:00:00", "headline") },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface."),
    leaves: doesntApply("it goes with its surface."),
    between: powerMoves(() => figure("21:47", "hero_number")),
  },
  keyboard: doesntApply("no focus and no key."),
  measures: {
    samples: nonEmpty([
      { name: "A stopwatch", for: "as wide as 0:00:00.0", sample: () => figure("1:02:11.4", "body_strong") },
      { name: "The clock's time", for: "as wide as 00:00", sample: () => figure("21:47", "hero_number") },
    ]),
    measures: [
      { part: "The clock's line", token: "type-hero_number-line" },
      { part: "A timer's line", token: "type-headline-line" },
      { part: "A stopwatch's line", token: "type-body_strong-line" },
    ],
    reflow: doesntApply("as wide as its widest at every window size."),
  },
});
