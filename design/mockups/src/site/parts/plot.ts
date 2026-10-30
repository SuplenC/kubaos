// The motion player's plot: the whole loop on x, in milliseconds, and each track's progress from
// its `from` (0) to its `to` (1) on y, one curve per track drawn from its easing token's cubic
// Bézier or `neon.strike`'s steps, with a ball riding each curve at the playhead. Drawn as SVG in
// the page's device pixels, so its hairlines land on whole pixels at every scale.

import { length } from "../../lib/tokens.ts";
import { advance } from "../../lib/text.ts";
import { el, label } from "../dom.ts";
import type { Track } from "../page.ts";
import { bezier, type Loop, progress, reach, type Span, steps, strikeSteps, type Timed } from "../recipe.ts";
import { settings } from "../settings.ts";

const SVG = "http://www.w3.org/2000/svg";

/** The y axis's step: a quarter of the way from `from` to `to`. */
const Y_STEP = 0.25;

/** The x axis's steps to choose from, in milliseconds: the smallest that leaves its labels room. */
const X_STEPS = [50, 100, 200, 250, 500, 1000, 2000, 5000];

/** Each ball's diameter by the track's place: concentric, so tracks on one curve all show. */
const BALLS = ["space-s16", "space-s12", "space-s8", "space-s4"];

function svg<K extends keyof SVGElementTagNameMap>(tag: K, className: string, attributes: Record<string, string | number> = {}): SVGElementTagNameMap[K] {
  const element = document.createElementNS(SVG, tag);
  if (className) element.setAttribute("class", className);
  for (const [name, value] of Object.entries(attributes)) element.setAttribute(name, String(value));
  return element;
}

/** A number as the plot writes it: at most two decimals. */
function shown(n: number): string {
  return String(Math.round(n * 100) / 100);
}

/** A token's length in the page's device pixels, at the scale picked. */
function px(token: string): number {
  return length(token) * settings().scale;
}

/** A track's place among the tracks playing, as its curve, swatch and ball are told apart. */
function look(element: Element, index: number): void {
  element.setAttribute("data-track", String(index % BALLS.length));
}

/** A short line in the track's colour and dash: its key in the legend and in the recipe's rows. */
export function swatch(index: number): HTMLElement {
  const box = el("span", "page-plot-swatch");
  const drawing = svg("svg", "", { "aria-hidden": "true" });
  const line = svg("line", "page-plot-curve", { x1: 0, x2: "100%", y1: "50%", y2: "50%" });
  look(line, index);
  drawing.append(line);
  box.append(drawing);
  return box;
}

/**
 * The tracks of `playing` that run on one curve, by place: the same span, back and curve. Their
 * curves are drawn in turns along it, so each colour shows.
 */
export function together(playing: Loop): number[][] {
  const groups = new Map<string, number[]>();
  playing.timed.forEach((timed, i) => {
    const key = JSON.stringify([timed.forward, timed.back, timed.track.curve]);
    groups.set(key, [...(groups.get(key) ?? []), i]);
  });
  return [...groups.values()];
}

/** The legend under the plot: each track's swatch and its part, as the recipe's rows name it. */
export function legend(playing: Loop): HTMLElement {
  const name = (track: Track) => `${track.part} (${track.on})`;
  const shared = together(playing)
    .filter((group) => group.length > 1)
    .map((group) => {
      const parts = group.map((i) => `the ${playing.timed[i]!.track.part}`);
      const words = `${parts.slice(0, -1).join(", ")} and ${parts.at(-1)} run on one curve, drawn in their colours in turn.`;
      return words.charAt(0).toUpperCase() + words.slice(1);
    });
  return el(
    "div",
    "page-plot-legend",
    ...playing.timed.map((timed, i) => el("span", "page-plot-key", swatch(i), label(name(timed.track), "small", "on_surface"))),
    ...shared.map((words) => label(words, "small", "on_surface_variant")),
  );
}

/** A mark under the x axis: a part's start or end, or a keyframe. */
export interface Mark {
  readonly at: number;
  readonly kind: "part" | "key";
  readonly words: string;
}

/** The plot's geometry for one loop at one width: where a time and a progress are drawn. */
interface Frame {
  readonly x: (t: number) => number;
  readonly y: (v: number) => number;
  readonly left: number;
  readonly right: number;
  readonly top: number;
  readonly bottom: number;
}

/** The plot of `playing`'s tracks, and the playhead and balls moved to a time. */
export class Plot {
  readonly element: SVGSVGElement;
  private playing: Loop | null = null;
  private frame: Frame | null = null;
  private readonly head = svg("line", "page-plot-head");
  private balls: SVGCircleElement[] = [];
  /** The thin stroke the curves and playhead are drawn with, rounded to device pixels. */
  private thin = 1;

  constructor() {
    this.element = svg("svg", "page-plot");
  }

  /** The time at a pointer's `clientX`, on the loop. */
  timeAt(clientX: number): number {
    if (!this.playing || !this.frame) return 0;
    const box = this.element.getBoundingClientRect();
    const zoom = this.element.clientWidth / box.width || 1;
    const x = (clientX - box.left) * zoom;
    const { left, right } = this.frame;
    return ((x - left) / (right - left)) * this.playing.total;
  }

  /** Draws `playing`'s curves and `marks` at the plot's width, the playhead and balls at `t`. */
  draw(playing: Loop, marks: readonly Mark[], t: number): void {
    const width = this.element.parentElement?.clientWidth ?? 0;
    // Not laid out yet: drawn once it is, when its box gets a width.
    this.playing = width === 0 ? null : playing;
    if (width === 0) return;
    const hairline = 1;
    const thin = Math.max(1, Math.round(px("stroke-thin")));
    this.thin = thin;
    // How far the curves reach: past 1 for an overshooting curve, below 0 for one that dips.
    let low = 0;
    let high = 1;
    for (const { track, back } of playing.timed) {
      if (track.curve === "neon.strike") continue;
      const range = reach(track.curve);
      low = Math.min(low, range.low);
      high = Math.max(high, range.high);
      // Played back, the curve runs from 1 to 0: an overshoot past 1 dips below 0.
      if (back) {
        low = Math.min(low, 1 - range.high);
        high = Math.max(high, 1 - range.low);
      }
    }
    const bottomValue = Math.floor(low / Y_STEP) * Y_STEP;
    const topValue = Math.ceil(high / Y_STEP - 1e-9) * Y_STEP;
    const unit = px("size-motion_plot");
    const line = px("type-mono-line");
    const top = px("space-s8");
    const left = px("space-s64");
    const right = width - px("space-s8");
    const bottom = top + (topValue - bottomValue) * unit;
    const height = bottom + px("space-s32");
    const x = (t: number) => left + (t / playing.total) * (right - left);
    const y = (v: number) => top + (topValue - v) * unit;
    this.frame = { x, y, left, right, top, bottom };
    // A hairline's centre on a device pixel's centre, so it is one pixel wide, never two half ones.
    const crisp = (at: number, stroke: number) => Math.round(at - stroke / 2) + stroke / 2;

    this.element.setAttribute("width", String(width));
    this.element.setAttribute("height", String(height));
    this.element.setAttribute("viewBox", `0 0 ${width} ${height}`);
    // Where 0 ms and the loop's end, and a progress of 0 and 1, are drawn: for the player's test.
    for (const [name, value] of Object.entries({ x0: left, x1: right, total: playing.total, y0: y(0), y1: y(1) })) {
      this.element.dataset[name] = String(value);
    }
    const grid = svg("g", "page-plot-grid");
    const labels = svg("g", "page-plot-labels");
    for (let v = bottomValue; v <= topValue + 1e-9; v += Y_STEP) {
      const at = crisp(y(v), hairline);
      grid.append(svg("line", "", { x1: left, x2: right, y1: at, y2: at }));
      labels.append(svg("text", "page-plot-text", { x: left - px("space-s16"), y: y(v), "text-anchor": "end", "dominant-baseline": "central" }));
      labels.lastChild!.textContent = String(Math.round(v * 100) / 100).replace("-", "\u2212");
    }
    const room = px("space-s64");
    const step = X_STEPS.find((s) => (s / playing.total) * (right - left) >= room) ?? X_STEPS.at(-1)!;
    const below = bottom + px("space-s12") + line / 2;
    const wide = (words: string) => advance("mono", words) * settings().scale;
    // The loop's end is always named, at the plot's right edge.
    const end = `${shown(playing.total)} ms`;
    const endsAt = right - wide(end) - px("space-s8");
    labels.append(svg("text", "page-plot-text", { x: right, y: below, "text-anchor": "end", "dominant-baseline": "central" }));
    labels.lastChild!.textContent = end;
    for (let ms = 0; ms <= playing.total + 1e-9; ms += step) {
      const at = crisp(x(ms), hairline);
      grid.append(svg("line", "", { x1: at, x2: at, y1: top, y2: bottom }));
      const words = ms === 0 ? "0 ms" : String(ms);
      // A label that would run into the end's is left out; its grid line stays.
      if (ms > 0 && x(ms) + wide(words) / 2 > endsAt) continue;
      labels.append(svg("text", "page-plot-text", { x: x(ms), y: below, "text-anchor": ms === 0 ? "start" : "middle", "dominant-baseline": "central" }));
      labels.lastChild!.textContent = words;
    }

    const ticks = svg("g", "page-plot-marks");
    const sorted = marks.map((mark) => x(mark.at)).sort((a, b) => a - b);
    for (const mark of marks) {
      const at = crisp(x(mark.at), hairline);
      // A target reaches halfway to its neighbours at most, so two never overlap.
      const near = Math.min(...sorted.filter((other) => other !== x(mark.at)).map((other) => Math.abs(other - x(mark.at))));
      const reachOut = Math.min(px("space-s4"), near / 2);
      const tick = svg("g", "page-plot-mark", { "data-at": mark.at, "data-kind": mark.kind });
      const title = svg("title", "");
      title.textContent = mark.words;
      tick.append(
        title,
        svg("line", "", { x1: at, x2: at, y1: bottom, y2: bottom + px(mark.kind === "part" ? "space-s8" : "space-s4") }),
        // The mark's target is wider than its line, so a pointer finds it.
        svg("rect", "page-plot-hit", { x: x(mark.at) - reachOut, y: bottom, width: 2 * reachOut, height: px("space-s8") }),
      );
      ticks.append(tick);
    }

    const curves = svg("g", "");
    const turn = px("space-s8");
    const turns = new Map<number, { place: number; of: number }>();
    for (const group of together(playing)) group.forEach((i, place) => turns.set(i, { place, of: group.length }));
    const hits = playing.timed.map((timed, i) => {
      const d = path(timed, playing.total, (t) => crisp(x(t), thin), (v) => crisp(y(v), thin));
      const curve = svg("path", "page-plot-curve", { d, "stroke-width": thin, "data-pick": i });
      look(curve, i);
      const shared = turns.get(i)!;
      if (shared.of > 1) {
        // Tracks on one curve take turns along it, a dash each, so every colour shows.
        curve.style.strokeDasharray = `${turn} ${(shared.of - 1) * turn}`;
        curve.style.strokeDashoffset = String(-shared.place * turn);
      }
      curves.append(curve);
      // A wider, unpainted line along the curve, for the pointer to find it by.
      return svg("path", "page-plot-curve-hit", { d, "stroke-width": px("space-s12"), "data-pick": i });
    });
    this.head.setAttribute("y1", String(top));
    this.head.setAttribute("y2", String(bottom));
    this.head.setAttribute("stroke-width", String(thin));
    this.balls = playing.timed.map((_, i) => {
      const ball = svg("circle", "page-plot-ball", { r: px(BALLS[i % BALLS.length]!) / 2, "data-target": playing.timed[i]!.track.target });
      look(ball, i);
      return ball;
    });
    this.balls.forEach((ball, i) => ball.setAttribute("data-pick", String(i)));
    this.element.replaceChildren(grid, labels, ticks, curves, ...hits, this.head, ...this.balls);
    this.at(t);
  }

  /** Moves the playhead to `t`, and each track's ball onto its curve there. */
  at(t: number): void {
    if (!this.playing || !this.frame) return;
    const { x, y } = this.frame;
    const head = Math.round(x(t) - this.thin / 2) + this.thin / 2;
    this.head.setAttribute("x1", String(head));
    this.head.setAttribute("x2", String(head));
    this.playing.timed.forEach((timed, i) => {
      const ball = this.balls[i];
      if (!ball) return;
      const value = progress(timed, t);
      ball.setAttribute("cx", String(x(t)));
      ball.setAttribute("cy", String(y(value)));
      ball.dataset.progress = String(value);
    });
  }
}

/**
 * `timed`'s curve over the loop as path data: at 0, then its easing as the very cubic Bézier the
 * browser eases along, laid on the track's span (a Bézier stays one under an affine map), or
 * `neon.strike`'s steps, held, then back along the same curve, or cut back at the loop's end.
 * `x` and `y` put it on device pixels, so its straight runs are crisp; a ball is drawn unrounded,
 * a fraction of a pixel off them at most.
 */
function path(timed: Timed, total: number, x: (t: number) => number, y: (v: number) => number): string {
  const { track } = timed;
  const point = (t: number, v: number) => `${x(t)} ${y(v)}`;
  const parts = [`M ${point(0, 0)}`];
  const eased = (span: Span, from: number, to: number) => {
    if (track.curve === "neon.strike") return;
    const [x1, y1, x2, y2] = bezier(track.curve) as [number, number, number, number];
    const time = (share: number) => span.start + share * (span.end - span.start);
    const value = (share: number) => from + share * (to - from);
    parts.push(`L ${point(span.start, from)}`, `C ${point(time(x1), value(y1))} ${point(time(x2), value(y2))} ${point(span.end, to)}`);
  };
  if (track.curve === "neon.strike") {
    const values = strikeSteps();
    let before = 0;
    steps({ ...timed, back: null }).forEach((span, i) => {
      parts.push(`L ${point(span.start, before)}`, `L ${point(span.start, values[i]!)}`);
      before = values[i]!;
    });
    parts.push(`L ${point(timed.forward.end, before)}`, `L ${point(timed.forward.end, 1)}`);
  } else {
    eased(timed.forward, 0, 1);
  }
  if (timed.back) {
    eased(timed.back, 1, 0);
    parts.push(`L ${point(total, 0)}`);
  } else {
    // Cut back: the last state holds to the loop's end, and the loop starts again on the first.
    parts.push(`L ${point(total, 1)}`, `L ${point(total, 0)}`);
  }
  return parts.join(" ");
}
