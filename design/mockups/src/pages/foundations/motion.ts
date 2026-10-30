// The Motion foundation: every duration and curve, live, the spring included, each curve on a
// player with its plot; the speed setting and reduce motion; and the theme change and the dark ⇄
// light switch, from the same recipes the Button page plays (`docs/DESIGN.md`, "Motion").

import { button, KINDS } from "../../lib/button.ts";
import { row } from "../../lib/layout.ts";
import { names, words } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { type Duration, doesntApply, type Easing, foundation, type Move, nonEmpty } from "../../site/page.ts";
import { table, token } from "../../site/parts/swatches.ts";
import { bezier, eased, milliseconds, reach } from "../../site/recipe.ts";
import { type ColourPath, paintBetween, palette, paletteName, themeName } from "../../site/recolour.ts";
import { themeChange, toDark, toLight } from "../../site/sequences.ts";
import { type Mode, SPEEDS, settings } from "../../site/settings.ts";

const SVG = "http://www.w3.org/2000/svg";

/** Every duration token, by name, in the order the tokens declare them. */
function durations(): Duration[] {
  return names("duration-")
    .filter((name) => !name.endsWith("-use"))
    .map((name) => name.slice("duration-".length) as Duration)
    .sort((a, b) => milliseconds(a) - milliseconds(b));
}

/** Every curve token, by name, in `docs/DESIGN.md`'s order; one it doesn't list yet comes last. */
function curves(): Easing[] {
  const order = Object.keys(PLAYED_OVER);
  const rank = (curve: string) => (order.includes(curve) ? order.indexOf(curve) : order.length);
  return names("easing-")
    .filter((name) => !name.endsWith("-use"))
    .map((name) => name.slice("easing-".length) as Easing)
    .sort((a, b) => rank(a) - rank(b));
}

/** What a duration token is for, as the tokens say it. */
const durationUse = (duration: Duration) => words(`duration-${duration}-use`);
const curveUse = (curve: Easing) => words(`easing-${curve}-use`);

/**
 * The duration each curve's player plays it over: the one it is used with (`docs/DESIGN.md`,
 * "Motion"), so the curve is seen at the pace it moves things.
 */
const PLAYED_OVER: Record<Easing, Duration> = {
  standard: "short",
  standard_decel: "medium",
  standard_accel: "medium",
  emphasized_decel: "long",
  emphasized_accel: "medium",
  linear: "slow",
  pop: "short",
  spring: "long",
  gather: "recolour",
};

/** Every duration racing from one start, the fill at the speed picked and capped by reduce motion. */
function race(): HTMLElement {
  return el(
    "div",
    "f-race",
    ...durations().flatMap((duration) => {
      const track = el("div", "f-race-track", el("div", "f-race-fill"));
      track.style.setProperty("--d", `var(--duration-${duration})`);
      track.style.setProperty("--motion", `var(--motion-${duration})`);
      return [token(`duration.${duration}`, `${milliseconds(duration)} ms`), track];
    }),
  );
}

function durationTable(): HTMLElement {
  return table(
    ["Duration", "ms", "For"],
    durations().map((duration) => [token(`duration.${duration}`), label(String(milliseconds(duration)), "mono", "on_surface"), prose(durationUse(duration), "body", "on_surface")]),
  );
}

/** `curve` drawn small: its progress over its duration, with the room it overshoots into. */
function curveDrawing(curve: Easing): SVGSVGElement {
  const { low, high } = reach(curve);
  const bottom = Math.min(0, low);
  const top = Math.max(1, high);
  const y = (v: number) => 100 - ((v - bottom) / (top - bottom)) * 100;
  const drawing = document.createElementNS(SVG, "svg");
  drawing.setAttribute("class", "f-curve");
  // A margin round the unit square, so the line's ends at 0 and 1 aren't cut by the box.
  drawing.setAttribute("viewBox", "-4 -4 108 108");
  drawing.setAttribute("preserveAspectRatio", "none");
  drawing.setAttribute("aria-hidden", "true");
  for (const v of [0, 1]) {
    const line = document.createElementNS(SVG, "line");
    line.setAttribute("x1", "0");
    line.setAttribute("x2", "100");
    line.setAttribute("y1", String(y(v)));
    line.setAttribute("y2", String(y(v)));
    line.setAttribute("vector-effect", "non-scaling-stroke");
    drawing.append(line);
  }
  const points = Array.from({ length: 101 }, (_, i) => `${i === 0 ? "M" : "L"}${i} ${y(eased(curve, i / 100))}`);
  const path = document.createElementNS(SVG, "path");
  path.setAttribute("d", points.join(" "));
  path.setAttribute("vector-effect", "non-scaling-stroke");
  drawing.append(path);
  return drawing;
}

function curveTable(): HTMLElement {
  return table(
    ["Curve", "Cubic Bézier", "Drawn", "Played over", "For"],
    curves().map((curve) => [
      token(`easing.${curve}`),
      label(bezier(curve).join(", "), "mono", "on_surface"),
      curveDrawing(curve),
      token(`duration.${PLAYED_OVER[curve]}`),
      prose(curveUse(curve), "body", "on_surface"),
    ]),
  );
}

/** What moves on which duration and curve, by what it is (`docs/DESIGN.md`, "Motion"). */
const BY_WHAT: readonly { readonly what: string; readonly duration: Duration | null; readonly curve: Easing | "neon.strike" }[] = [
  { what: "A control pressed, a colour change under the pointer", duration: "micro", curve: "standard" },
  { what: "A control lighting up or going back to rest, hovered or focused; focus moving, between controls or windows; a switch's knob, a check mark", duration: "short", curve: "standard" },
  { what: "A slider settling on its step", duration: "short", curve: "standard_decel" },
  { what: "A slider's anchor dot popping in", duration: "short", curve: "pop" },
  { what: "A list reordering, a live list's row opening its room", duration: "medium", curve: "standard_decel" },
  { what: "A toast fading out, a live list's row giving its room back", duration: "medium", curve: "standard_accel" },
  { what: "A panel or dialog arriving, a page sliding in, the log opening", duration: "long", curve: "emphasized_decel" },
  { what: "A panel or dialog leaving", duration: "medium", curve: "emphasized_accel" },
  { what: "The wallpaper carousel's step", duration: "long", curve: "spring" },
  { what: "A power change's lights: a surface or window appearing, a definite off, the theme change and dark ⇄ light. Never focus or hover", duration: "medium", curve: "neon.strike" },
  { what: "Every colour role in a theme change or the dark ⇄ light switch", duration: "recolour", curve: "gather" },
  { what: "Progress that tracks a real quantity, a spinner", duration: null, curve: "linear" },
];

function byWhat(): HTMLElement {
  return table(
    ["What", "Duration", "Curve"],
    BY_WHAT.map((each) => [
      prose(each.what, "body", "on_surface"),
      each.duration ? token(`duration.${each.duration}`, `${milliseconds(each.duration)} ms`) : label("as long as the quantity takes", "body", "on_surface_variant"),
      token(each.curve === "neon.strike" ? "neon.strike" : `easing.${each.curve}`),
    ]),
  );
}

/** A number of milliseconds as the page writes it: at most one decimal. */
function ms(n: number): string {
  return `${Math.round(n * 10) / 10}`;
}

/** Every duration at every speed, the speed picked lit. */
function speeds(): HTMLElement {
  const picked = settings().speed;
  return table(
    ["Duration", ...SPEEDS.map((speed) => `${speed}×${speed === picked ? " (picked)" : ""}`)],
    durations().map((duration) => [
      token(`duration.${duration}`),
      ...SPEEDS.map((speed) => label(ms(milliseconds(duration) / speed), "mono", speed === picked ? "primary" : "on_surface")),
    ]),
  );
}

/** Every duration with reduce motion on: capped at `short`. */
function reduced(): HTMLElement {
  const cap = milliseconds("short");
  return table(
    ["Duration", "ms", "With reduce motion"],
    durations().map((duration) => {
      const d = milliseconds(duration);
      return [token(`duration.${duration}`), label(String(d), "mono", "on_surface"), label(String(Math.min(d, cap)), "mono", d > cap ? "primary" : "on_surface")];
    }),
  );
}

/** What never animates, in words. */
const NEVER: readonly string[] = [
  "Typed text and the caret: `instant`.",
  "Layout around focus: the tube and its glow take no room.",
  "A window's size or place.",
  "Anything while the user drags it.",
];

/** A ball on a rail, which a curve's player moves from one end to the other. */
function rail(): HTMLElement {
  return el("div", "f-rail", el("div", "f-ball"));
}

/** A curve's player: the ball from one end of the rail to the other over its duration. */
function curveMove(curve: Easing): Move {
  const duration = PLAYED_OVER[curve];
  const track = (over: Duration) => ({ part: "ball", on: "the rail", target: ".f-ball", property: "--along", from: 0, to: 1, duration: over, curve });
  const capped = milliseconds(duration) > milliseconds("short");
  return {
    name: `easing.${curve}`,
    what: `${curveUse(curve)} The ball crosses its rail on \`easing.${curve}\`, cubic-bezier(${bezier(curve).join(", ")}), over \`duration.${duration}\`, ${milliseconds(duration)} ms${capped ? "; with reduce motion, over `short`, as every duration is capped" : ""}.`,
    tracks: [track(duration)],
    reduced: capped ? [track("short")] : "same",
    back: "cuts",
    sample: rail,
  };
}

/** A palette: a theme in a mode. */
interface Painted {
  readonly theme: string;
  readonly mode: Mode;
}

/** The scene a sequence plays on: every kind of button lit, on its way between two palettes. */
function scene(from: Painted | string, to: Painted | string, path: ColourPath): () => Element {
  return () => {
    const mode = settings().mode;
    const [a, b] = [from, to].map((each) => (typeof each === "string" ? { theme: each, mode } : each)) as [Painted, Painted];
    const painted = el(
      "div",
      "page-scene",
      label("Every light and every colour, on one clock", "body_strong", "on_surface"),
      row("s16", "start", ...KINDS.map((kind) => button({ label: kind, kind, size: "Md", pinned: "lit" }))),
      rail(),
    );
    painted.dataset.mode = a.mode;
    paintBetween(painted, path, palette(a.theme, a.mode), palette(b.theme, b.mode));
    return el("div", "page-scene-box", painted);
  };
}

function themeMove(from: string, to: string): Move {
  const between = { from: themeName(from), to: themeName(to) };
  return themeChange(`Theme change: ${between.from} to ${between.to}`, between, scene(from, to, "through grey"));
}

export default foundation({
  level: "foundations",
  slug: "motion",
  title: "Motion",
  summary: "Every animation takes a duration and a curve by what it is, never by feel; the speed setting divides them all and reduce motion caps them.",
  live: race,
  description: [
    "Beautiful and fast. Every move takes a duration token and a curve token by what it is: a state layer `micro` on `standard`, a panel arriving `long` on `emphasized_decel` and leaving `medium` on `emphasized_accel`. Beside it, every duration races from one start, each bar its length at speed 1, `space.s16` for every 100 ms, filling at the speed the toolbar picks.",
    "Arriving is slower than leaving: you look at what arrives, and what leaves is already done with. A surface a key summons to act and go is a step quicker, since it opens and closes many times an hour.",
    "The speed setting, 0.5× to 2×, divides every duration, cue and beat. Reduce motion caps every duration at `short` and turns moves into fades, or into nothing; the theme change and the dark ⇄ light switch swap their colours at once. Both are the toolbar's, so every player on every page plays what they make of its move.",
    "The theme change and the dark ⇄ light switch are one sequence each, on one clock for every light and colour on every screen, played below from the same recipes the Button page and the toolbar play. The neon's flicker, `neon.strike`, is on the Focus page: it plays only for power changes.",
    "What never animates: typed text and the caret, layout around focus, a window's size or place, and anything while the user drags it. Every animation runs at the display's full refresh rate; one that drops a frame is a bug, measured with the frame times, not judged by eye.",
  ],
  tokens: [
    { name: "Every duration", described: "Each duration token, its milliseconds and what it is for, as the tokens say it.", sample: durationTable },
    {
      name: "Every curve",
      described: "Each curve token, its cubic Bézier, drawn small (its progress up over its duration across, with room past 1 where `pop` and `spring` overshoot), the duration its player plays it over, and what it is for. Each plays on its own player under Motion.",
      sample: curveTable,
    },
  ],
  rules: [
    { name: "By what it is, never by feel", described: "What moves, and the duration and curve it takes.", sample: byWhat },
    {
      name: "The speed divides every duration",
      described: "Every duration at each speed, the toolbar's lit. A cue and a closing beat are durations too, so a whole sequence keeps its shape at any speed. The race in the description follows the speed picked.",
      sample: speeds,
    },
    {
      name: "Reduce motion caps every duration at short",
      described: "Every duration with reduce motion on: nothing is longer than `short`, so a long move becomes a short fade, and the curves' players play over `short`. The theme change and the switch swap their colours at once instead, with nothing going dark. Turn it on in the toolbar and the race beside the description and every player follow.",
      sample: reduced,
    },
    {
      name: "What never animates",
      described: "Some changes are never moves, at any speed.",
      sample: () => el("div", "page-part", ...NEVER.map((line) => prose(line))),
    },
  ],
  modes: doesntApply("a duration and a curve are the same in light and dark; the moves that differ by day, where the neon is off, play their light mode's tracks when the toolbar picks light."),
  themes: doesntApply("a duration and a curve are the same in every theme; the theme change between two of them plays under Motion."),
  scales: doesntApply("a duration and a curve are the same at every scale; the plots under each player are drawn in the page's device pixels."),
  content: doesntApply("a duration and a curve hold no words."),
  motion: {
    arrives: doesntApply("a curve's player under Between states plays each curve on its own; what arrives on which is in the rules above, and each component's page plays its own."),
    leaves: doesntApply("as arriving: each curve plays under Between states, and each component's page plays its own leaving."),
    between: nonEmpty<Move>([
      ...curves().map(curveMove),
      themeMove("neon", "rose"),
      themeMove("lagoon", "moss"),
      toLight({ from: paletteName("neon", "dark"), to: paletteName("neon", "light") }, scene({ theme: "neon", mode: "dark" }, { theme: "neon", mode: "light" }, "dawn")),
      toDark({ from: paletteName("neon", "light"), to: paletteName("neon", "dark") }, scene({ theme: "neon", mode: "light" }, { theme: "neon", mode: "dark" }, "dawn")),
    ]),
  },
  checks: [
    { check: "kubaos-tokens", holds: "Generates every duration and curve for every toolkit from `design/tokens.toml`, so a move names a token, never a number." },
    { check: "kubaos-design lint", holds: "Fails on a duration or a curve written out in QML or Rust UI code." },
    { check: "The players' check", holds: "`just mockups-check` seeks every player on every page and holds each ball to the progress the browser plays, with its own tracks, reduced motion's, light mode's, the flicker off's and at speed 2." },
    { check: "Not enforced yet", holds: "Neither library reads the speed setting or reduce motion yet, nothing measures frame times, and nothing moves on `spring` or `gather` outside the mockups." },
  ],
});
