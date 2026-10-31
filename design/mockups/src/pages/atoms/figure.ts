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

/** Tenths of a second as a stopwatch shows them: m:ss.t, or h:mm:ss.t from an hour. */
function stopwatch(tenths: number): string {
  const seconds = Math.floor(tenths / 10);
  const pad = (n: number) => String(n).padStart(2, "0");
  const [h, m, s] = [Math.floor(seconds / 3600), Math.floor(seconds / 60) % 60, seconds % 60];
  return `${h > 0 ? `${h}:${pad(m)}` : m}:${pad(s)}.${tenths % 10}`;
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

const DESCRIBED =
  "A `Text` (`text`, `style`, `color`) in a box as wide as the same text with every digit a 0, Geist's widest figure; the text starts at the box's left edge, and its line is the style's. Only the digits are held: a colon, a point, a sign or a unit takes its own width, so the box grows only when the text's shape does (59:59 to 1:00:00).";

export default page({
  level: "atoms",
  slug: "figure",
  title: "Figure",
  summary: "A number that ticks (a time, a timer, a stopwatch) in a box that holds still as its digits change.",
  live: () => stack("s16", "start", beside(true), beside(false)),
  description: [
    "QML `Figure`: the clock popup's times, its timer and its stopwatch, on the QML specimen's \"Shell only\" page. `kubaos-ui` gets one with the first app that shows a ticking number.",
    "Geist's figures are proportional, so a ticking number in a plain `Text` changes width with its digits and jitters whatever sits beside it, as the second line above does. A `Figure` holds its box at its widest.",
    DESCRIBED,
    "A number that changes in place but doesn't tick (a slider's value) takes `label_number`, whose figures are tabular, instead.",
  ],
  variants: nonEmpty(
    SPECIMEN.map(([words, style]) => ({
      name: style,
      for: style === "hero_number" ? "The clock popup's time." : style === "headline" ? "A timer counting down." : "A stopwatch, to the tenth.",
      sample: () => figure(words, style),
    })),
  ),
  sizes: doesntApply("a figure takes its style's size: the variants are the styles the clock popup uses."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a figure takes no pointer and no focus.",
      pressed: "it takes no press: the timer's buttons beside it do.",
      disabled: "a stopped timer still shows its time, unchanged.",
      selected: "it holds no choice.",
      loading: "a time is known at once.",
      empty: "a figure always shows a number: a timer that isn't set shows 0:00.",
      error: "it has no value to be wrong.",
    },
    rows: nonEmpty(SPECIMEN.map(([words, style]) => ({ name: style, cell: () => figure(words, style) }))),
    described: {
      rest: `Always. ${DESCRIBED} By day the same role in daylight colours.`,
    },
  },
  focus: doesntApply("a figure isn't a control and isn't neon: it has no line and no glow."),
  content: {
    text: [{ name: "A stopwatch", words: TIME, sample: (words: string) => row("s8", "centre", figure(words, "headline"), text("elapsed", "body", "on_surface_variant")) }],
    missingPicture: doesntApply("a figure is words."),
    counts: [
      { name: "Minutes", for: "59:59: the box as wide as 00:00.", sample: () => figure("59:59", "headline") },
      { name: "An hour", for: "1:00:00: the text's shape changes, so the box grows to 0:00:00, once.", sample: () => figure("1:00:00", "headline") },
    ],
  },
  motion: {
    arrives: doesntApply("a figure comes with the surface it is on."),
    leaves: doesntApply("a figure goes with the surface it is on."),
    between: powerMoves(() => figure("21:47", "hero_number")),
  },
  keyboard: doesntApply("a figure takes no focus and no key."),
  measures: {
    samples: nonEmpty([
      { name: "A stopwatch", for: "Its box as wide as 0:00:00.0, the text at its left.", sample: () => figure("1:02:11.4", "body_strong") },
      { name: "The clock's time", for: "Its box as wide as 00:00.", sample: () => figure("21:47", "hero_number") },
    ]),
    measures: [
      { what: "The clock's time, its line.", token: "type-hero_number-line" },
      { what: "A timer's line.", token: "type-headline-line" },
      { what: "A stopwatch's line.", token: "type-body_strong-line" },
    ],
    reflow: doesntApply("a figure is as wide as its widest at every window size."),
  },
});
