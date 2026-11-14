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
import { across } from "./across.ts";

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

/**
 * Every duration up to a scene racing from one start, the fill at the speed picked and capped by
 * reduce motion. Longer ones are holds, not moves, and a bar of theirs would run off any pane.
 */
function race(): HTMLElement {
  return el(
    "div",
    "f-race",
    ...durations()
      .filter((duration) => milliseconds(duration) <= milliseconds("scene"))
      .flatMap((duration) => {
      const track = el("div", "f-race-track", el("div", "f-race-fill"));
      track.style.setProperty("--d", `var(--duration-${duration})`);
      track.style.setProperty("--motion", `var(--motion-${duration})`);
      return [token(`duration.${duration}`, `${milliseconds(duration)} ms`), track];
      }),
  );
}

function durationTable(): HTMLElement {
  return across(table(
    ["Duration", "ms", "For"],
    durations().map((duration) => [token(`duration.${duration}`), label(String(milliseconds(duration)), "mono", "on_surface"), prose(durationUse(duration), "body", "on_surface")]),
  ));
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
  return across(table(
    ["Curve", "Cubic Bézier", "Drawn", "Played over", "For"],
    curves().map((curve) => [
      token(`easing.${curve}`),
      label(bezier(curve).join(", "), "mono", "on_surface"),
      curveDrawing(curve),
      token(`duration.${PLAYED_OVER[curve]}`),
      prose(curveUse(curve), "body", "on_surface"),
    ]),
  ));
}

/** What moves on which duration and curve, by what it is (`docs/DESIGN.md`, "Motion"). */
const BY_WHAT: readonly { readonly what: string; readonly duration: Duration | null; readonly curve: Easing | "neon.strike" }[] = [
  { what: "A press, a colour change under the pointer", duration: "micro", curve: "standard" },
  { what: "Lighting up, back to rest, focus moving, a knob, a check", duration: "short", curve: "standard" },
  { what: "A slider settling on its step", duration: "short", curve: "standard_decel" },
  { what: "A slider's anchor dot popping in", duration: "short", curve: "pop" },
  { what: "A list reordering, a row opening its room", duration: "medium", curve: "standard_decel" },
  { what: "A toast fading out, a row giving its room back", duration: "medium", curve: "standard_accel" },
  { what: "A panel or dialog arriving, a page sliding in", duration: "long", curve: "emphasized_decel" },
  { what: "A panel or dialog leaving", duration: "medium", curve: "emphasized_accel" },
  { what: "The wallpaper carousel's step", duration: "long", curve: "spring" },
  { what: "A power change's lights, never focus or hover", duration: "medium", curve: "neon.strike" },
  { what: "Every colour role in a theme change or a switch", duration: "recolour", curve: "gather" },
  { what: "Progress that tracks a real quantity, a spinner", duration: null, curve: "linear" },
];

function byWhat(): HTMLElement {
  return across(table(
    ["What", "Duration", "Curve"],
    BY_WHAT.map((each) => [
      prose(each.what, "body", "on_surface"),
      each.duration ? token(`duration.${each.duration}`, `${milliseconds(each.duration)} ms`) : label("as long as the quantity takes", "body", "on_surface_variant"),
      token(each.curve === "neon.strike" ? "neon.strike" : `easing.${each.curve}`),
    ]),
  ));
}

/** A number of milliseconds as the page writes it: at most one decimal. */
function ms(n: number): string {
  return `${Math.round(n * 10) / 10}`;
}

/** Every duration at every speed, the speed picked lit. */
function speeds(): HTMLElement {
  const picked = settings().speed;
  return across(table(
    ["Duration", ...SPEEDS.map((speed) => `${speed}×${speed === picked ? " (picked)" : ""}`)],
    durations().map((duration) => [
      token(`duration.${duration}`),
      ...SPEEDS.map((speed) => label(ms(milliseconds(duration) / speed), "mono", speed === picked ? "primary" : "on_surface")),
    ]),
  ));
}

/** Every duration with reduce motion on: capped at `short`. */
function reduced(): HTMLElement {
  const cap = milliseconds("short");
  return across(table(
    ["Duration", "ms", "With reduce motion"],
    durations().map((duration) => {
      const d = milliseconds(duration);
      return [token(`duration.${duration}`), label(String(d), "mono", "on_surface"), label(String(Math.min(d, cap)), "mono", d > cap ? "primary" : "on_surface")];
    }),
  ));
}

/** What never animates, in words. */
const NEVER: readonly string[] = [
  "Typed text and the caret: `instant`",
  "Layout around focus: the tube and its glow take no room",
  "A window's size or place",
  "Anything while the user drags it",
];

/** A ball on a rail, which a curve's player moves from one end to the other. */
function rail(): HTMLElement {
  return el("div", "f-rail", el("div", "f-ball"));
}

/** A curve's player: the ball from one end of the rail to the other over its duration. */
function curveMove(curve: Easing): Move {
  const duration = PLAYED_OVER[curve];
  const track = (over: Duration) => ({ part: "ball", on: "the rail", target: ".f-ball", property: "--ridden", from: 0, to: 1, duration: over, curve });
  const capped = milliseconds(duration) > milliseconds("short");
  return {
    name: `easing.${curve}`,
    what: `The ball crosses on \`easing.${curve}\` over \`duration.${duration}\`, ${milliseconds(duration)} ms${capped ? "; `short` with reduce motion" : ""}.`,
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

/** The race beside every curve, drawn: the family at a glance. */
function atAGlance(): HTMLElement {
  const drawn = curves().map((curve) => {
    const drawing = curveDrawing(curve);
    drawing.style.setProperty("width", "var(--size-row_two_line)");
    drawing.style.setProperty("height", "var(--size-row_two_line)");
    return el("div", "f-swatch-cell", token(curve), drawing);
  });
  // Beside the race where a panel's width is left, under it where it isn't.
  const beside = el("div", "f-row", ...drawn);
  beside.style.flex = "1 1 0";
  beside.style.minWidth = "var(--size-panel)";
  return el("div", "f-row", race(), beside);
}

export default foundation({
  level: "foundations",
  slug: "motion",
  title: "Motion",
  lede: "Every animation takes a duration and a curve by what it is; the speed setting divides them all, reduce motion caps them.",
  stage: { live: atAGlance, hint: "pick a speed in the top bar, or Reduced", daylight: true },
  tokens: [
    { name: "Every duration", note: "Each duration's milliseconds, and what it is for.", sample: durationTable },
    { name: "Every curve", note: "Each drawn, with room past 1 where `pop` and `spring` overshoot; each plays under Motion.", sample: curveTable },
  ],
  rules: [
    { name: "By what it is, not feel", note: "What moves, on which duration and curve; arriving is slower than leaving.", sample: byWhat },
    { name: "The speed divides every duration", note: "The top bar's speed is lit; cues and beats divide too, so a sequence keeps its shape.", sample: speeds },
    { name: "Reduce motion caps at short", note: "Nothing is longer than `short`; a theme change or a switch swaps its colours at once.", sample: reduced },
    {
      name: "What never animates",
      note: "These are never moves; everything else runs at the display's full refresh rate.",
      sample: () => el("div", "page-part", ...NEVER.map((line) => prose(line))),
    },
  ],
  modes: doesntApply("a duration and a curve are the same in light and dark."),
  themes: doesntApply("a duration and a curve are the same in every theme; the theme change plays under Motion."),
  scales: doesntApply("a duration and a curve are the same at every scale."),
  content: doesntApply("a duration and a curve hold no words."),
  motion: {
    arrives: doesntApply("each curve plays under Between states; a component's page plays its own arriving."),
    leaves: doesntApply("each curve plays under Between states; a component's page plays its own leaving."),
    between: nonEmpty<Move>([
      ...curves().map(curveMove),
      themeMove("neon", "rose"),
      themeMove("lagoon", "moss"),
      toLight({ from: paletteName("neon", "dark"), to: paletteName("neon", "light") }, scene({ theme: "neon", mode: "dark" }, { theme: "neon", mode: "light" }, "dawn")),
      toDark({ from: paletteName("neon", "light"), to: paletteName("neon", "dark") }, scene({ theme: "neon", mode: "light" }, { theme: "neon", mode: "dark" }, "dawn")),
    ]),
  },
  checks: [
    { check: "kubaos-tokens", holds: "generates every duration and curve, so a move names a token" },
    { check: "kubaos-design lint", holds: "fails a duration or a curve written out in UI code" },
    { check: "The players' check", holds: "`just mockups-check` holds each player's ball to what the browser plays" },
    { check: "Not enforced yet", holds: "libraries ignore speed and reduce motion; frame times go unmeasured" },
  ],
});
