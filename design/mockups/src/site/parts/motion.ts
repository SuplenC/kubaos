// The motion player: each move live on its sample, looping (its first state held, the move, its
// last state held, then back), with pause, restart and slow motion, a plot of its tracks' curves
// over the loop to seek it on frame by frame, each part alone, and its recipe beside it: every
// part, property, value, timing, curve and keyframe, what reduced motion and the speed make of it,
// as tables and as JSON. The sample is animated with the
// Web Animations API from the move's own data (`site/recipe.ts`), so a frame seeked to is exactly
// the frame that plays, and the recipe shown is what plays.

import { number } from "../../lib/tokens.ts";
import { el, label, prose } from "../dom.ts";
import type { DoesntApply, Motion, Move, Track, Value } from "../page.ts";
import { applies } from "../page.ts";
import {
  between,
  bezier,
  css,
  FRAME,
  HOLD,
  keyframes,
  type Loop,
  loop,
  milliseconds,
  recipe,
  type TrackSet,
  type Span,
  steps,
  type Timed,
  strikeSteps,
  tokenOf,
  tracksOf,
} from "../recipe.ts";
import { CHROME, onChange } from "../settings.ts";
import { legend, type Mark, Plot, swatch } from "./plot.ts";
import { partOr, subheading } from "./section.ts";

/** How many times slower slow motion plays. */
const SLOW = 10;

/** How many frames Shift with an arrow moves. */
const BIG_STEP = 10;

/** How to stop each player on the page drawn: its settings listener, its observer, its frames. */
const players = new Set<() => void>();

/** Stops every player on the page, before the page is drawn again. */
export function stopPlayers(): void {
  for (const stop of players) stop();
  players.clear();
}

function control(words: string, pressed: boolean | null, onPress: () => void): HTMLButtonElement {
  const button = el("button", "site-control", words);
  button.type = "button";
  if (pressed !== null) button.setAttribute("aria-pressed", String(pressed));
  button.addEventListener("click", onPress);
  return button;
}

/** A number as the recipe writes it: at most two decimals. */
function shown(n: number): string {
  return String(Math.round(n * 100) / 100);
}

function valueWords(value: Value): string {
  const token = tokenOf(value);
  if (typeof value === "number") return shown(value);
  if ("opacity" in value) return `\`${token}\` ${shown(number(`opacity-${value.opacity}`))}`;
  return `\`${token}\``;
}

function curveWords(track: Track): string {
  if (track.curve === "neon.strike") return `\`neon.strike\`, ${strikeSteps().length} even steps (keyframes below)`;
  return `\`easing.${track.curve}\` cubic-bezier(${bezier(track.curve).join(", ")})`;
}

/** A table's cell of the page's words. */
function cell(words: string, header = false): HTMLElement {
  return el(header ? "th" : "td", "", header ? label(words, "label") : prose(words, "small"));
}

/** A row the playhead lights while `lit(t)` holds. */
interface Lit {
  readonly row: HTMLElement;
  readonly lit: (t: number, playing: Loop) => boolean;
}

/** A track's row in the recipe: its set and its place in it, which the player picks it by. */
interface Row {
  readonly row: HTMLElement;
  readonly set: TrackSet;
  readonly index: number;
}

function table(heads: readonly string[], rows: readonly HTMLElement[]): HTMLElement {
  return el("table", "page-table", el("thead", "", el("tr", "", ...heads.map((h) => cell(h, true)))), el("tbody", "", ...rows));
}

/** Whether `t` is inside one of `spans`. */
function within(t: number, spans: readonly Span[]): boolean {
  return spans.some((span) => t >= span.start && t < span.end);
}

/** The table of `set`'s tracks, each row lit while its part moves. */
function tracksTable(tracks: readonly Track[], set: TrackSet, lits: Lit[], picks: Row[]): HTMLElement {
  const rows = tracks.map((track, i) => {
    // The row's swatch is its curve's on the plot, when this set plays.
    const part = el("td", "", el("span", "page-plot-key", swatch(i), label(track.part, "small", "on_surface")));
    const row = el(
      "tr",
      "",
      part,
      cell(track.on),
      cell(`\`${track.property}\``),
      cell(valueWords(track.from)),
      cell(valueWords(track.to)),
      cell(`\`duration.${track.duration}\` ${milliseconds(track.duration)} ms`),
      cell(track.cue ? `\`duration.${track.cue}\` ${milliseconds(track.cue)} ms after the start` : "at the start"),
      cell(curveWords(track)),
    );
    picks.push({ row, set, index: i });
    lits.push({
      row,
      lit: (t, playing) => {
        const timed = playing.set === set ? playing.timed.find((each) => each.track === track) : undefined;
        return timed !== undefined && within(t, [timed.forward, ...(timed.back ? [timed.back] : [])]);
      },
    });
    return row;
  });
  return table(["Part", "On", "Property", "From", "To", "Duration", "Cue", "Curve"], rows);
}

/** The keyframes of every stepped track in `set`, those with the same timing in one table. */
function keyframeTables(tracks: readonly Track[], set: TrackSet, lits: Lit[]): HTMLElement[] {
  const groups = new Map<string, Track[]>();
  for (const track of tracks.filter((each) => each.curve === "neon.strike")) {
    const key = JSON.stringify([track.duration, track.cue, track.from, track.to]);
    groups.set(key, [...(groups.get(key) ?? []), track]);
  }
  return [...groups.values()].map((group) => {
    const first = group[0]!;
    const duration = milliseconds(first.duration);
    const values = strikeSteps();
    const rows = values.map((step, i) => {
      const row = el("tr", "", cell(`${shown((i * duration) / values.length)} ms`), cell(shown(between(first, step))));
      lits.push({
        row,
        lit: (t, playing) =>
          playing.set === set &&
          playing.timed.some((timed) => group.includes(timed.track) && steps(timed).some((span, j) => j % values.length === i && within(t, [span]))),
      });
      return row;
    });
    rows.push(el("tr", "", cell(`${duration} ms`), cell(`${shown(between(first, 1))}, held`)));
    const parts = group.map((track) => `the ${track.part} (${track.on})`).join(", ");
    return el("div", "page-recipe-part", prose(`\`neon.strike\`'s keyframes for ${parts}: each value holds until the next. Time is from the part's start, at speed 1.`, "small"), table(["Time", first.property], rows));
  });
}

/** The recipe of `move`: its tracks, their keyframes, reduced motion, the stutter off, speed. */
function recipeOf(move: Move, lits: Lit[], picks: Row[]): HTMLElement {
  const of = (set: TrackSet, words: string): Element[] => {
    const tracks = tracksOf(move, set);
    if (!tracks) return [];
    return [label(words, "body_strong", "on_surface"), tracksTable(tracks, set, lits, picks), ...keyframeTables(tracks, set, lits)];
  };
  return el(
    "div",
    "page-recipe",
    ...of("tracks", "The recipe"),
    ...(move.reduced === "same"
      ? [label("Reduced motion", "body_strong", "on_surface"), prose("The same: every duration is within the `short` that reduced motion caps durations at.", "small")]
      : of("reduced", "Reduced motion")),
    ...of("unstuttered", "With the focus stutter off"),
    label("Speed", "body_strong", "on_surface"),
    prose("The speed setting (0.5× to 2×) divides every duration and cue; the player's holds aren't motion, so they stay.", "small"),
  );
}

/** The player's loop, phase by phase, each row lit while it plays. */
function loopTable(playing: Loop, lits: Lit[]): HTMLElement {
  const rows = playing.phases.map((phase) => {
    const row = el("tr", "", cell(phase.name), cell(`${shown(phase.span.start)} ms`), cell(`${shown(phase.span.end)} ms`));
    lits.push({ row, lit: (t) => within(t, [phase.span]) });
    return row;
  });
  return el(
    "div",
    "page-recipe-part",
    label("The player's loop", "body_strong", "on_surface"),
    prose(`Not part of the move: each end held for \`duration.${HOLD}\`, ${milliseconds(HOLD)} ms, so each state can be seen, then it starts again.`, "small"),
    table(["Phase", "From", "To"], rows),
  );
}

/** The marks under the plot: each part's start and end, and each keyframe. */
function marks(playing: Loop): Mark[] {
  const found = new Map<number, { at: number; kind: "part" | "key"; words: string[] }>();
  const add = (at: number, kind: "part" | "key", words: string) => {
    const key = Math.round(at * 100);
    const mark = found.get(key) ?? { at, kind, words: [] };
    if (kind === "part") mark.kind = "part";
    mark.words.push(words);
    found.set(key, mark);
  };
  for (const timed of playing.timed) {
    for (const span of [timed.forward, ...(timed.back ? [timed.back] : [])]) {
      add(span.start, "part", `the ${timed.track.part} starts`);
      add(span.end, "part", `the ${timed.track.part} ends`);
    }
    if (timed.track.curve === "neon.strike") {
      for (const span of steps(timed)) add(span.start, "key", `the ${timed.track.part}'s keyframe`);
    }
  }
  return [...found.values()].map((mark) => ({ ...mark, words: `${shown(mark.at)} ms: ${[...new Set(mark.words)].join(", ")}` }));
}

function player(move: Move): HTMLElement {
  const sample = move.sample();
  const stage = el("div", "page-player-stage mock", sample);
  const whole = el("div", "page-player");
  let playing = loop(move);
  let animations: Animation[] = [];
  let running = CHROME;
  let visible = false;
  let slow = false;
  let time = 0;
  let frame = 0;

  const lits: Lit[] = [];
  const picks: Row[] = [];
  const recipePart = recipeOf(move, lits, picks);
  const loopPart = el("div", "");
  let loopLits: Lit[] = [];

  const plot = new Plot();
  const bar = el("div", "page-plot-box", plot.element);
  bar.tabIndex = 0;
  bar.setAttribute("role", "slider");
  bar.setAttribute("aria-label", `${move.name}: where the playhead is`);
  const key = el("div", "");
  const readout = el("div", "page-plot-time");
  const parts = el("div", "page-parts");
  const outlines = el("div", "page-player-outlines");
  stage.append(outlines);

  /**
   * Picks the playing track `i`, or none: its row, curve, legend key and stage light, and its
   * layers are outlined on the whole sample.
   */
  function pick(i: number | null): void {
    const track = i === null ? null : (playing.timed[i]?.track ?? null);
    whole.toggleAttribute("data-picking", track !== null);
    for (const item of whole.querySelectorAll("[data-pick]")) {
      const set = item.getAttribute("data-pick-set");
      item.toggleAttribute("data-picked", track !== null && item.getAttribute("data-pick") === String(i) && (set === null || set === playing.set));
    }
    outlines.replaceChildren();
    if (!track) return;
    const origin = stage.getBoundingClientRect();
    const zoom = stage.offsetWidth / origin.width || 1;
    for (const layer of stage.querySelectorAll(track.target)) {
      const box = layer.getBoundingClientRect();
      const outline = el("div", "page-player-outline");
      outline.style.left = `${(box.left - origin.left) * zoom}px`;
      outline.style.top = `${(box.top - origin.top) * zoom}px`;
      outline.style.width = `${box.width * zoom}px`;
      outline.style.height = `${box.height * zoom}px`;
      outlines.append(outline);
    }
  }

  /**
   * Makes `item` pick track `i` while the pointer is on it: of the tracks playing, or of `set`'s,
   * a recipe row's, only while that set plays.
   */
  function pickable(item: Element, i: number, set?: TrackSet): void {
    item.setAttribute("data-pick", String(i));
    if (set) item.setAttribute("data-pick-set", set);
    item.addEventListener("pointerenter", () => pick(set === undefined || set === playing.set ? i : null));
    item.addEventListener("pointerleave", () => pick(null));
  }

  const rate = () => (slow ? 1 / SLOW : 1);
  const now = () => Number(document.timeline.currentTime ?? 0);

  /** Where the loop is, in milliseconds. */
  function current(): number {
    const first = animations[0];
    if (!first) return time;
    return Number(first.currentTime ?? 0) % playing.total;
  }

  /** The loop's last frame: the one after it is the first again. */
  const lastFrame = () => Math.ceil(playing.total / FRAME - 1e-6) - 1;

  /** The frame `t` is in. */
  const frameOf = (t: number) => Math.min(Math.floor(t / FRAME + 1e-6), lastFrame());

  /** Shows `t` on the plot, the readout and the recipe; held, on the slider's value too. */
  function show(t: number, held: boolean): void {
    const at = frameOf(t);
    plot.at(t);
    readout.textContent = `${Math.round(t)} ms · frame ${at + 1} of ${lastFrame() + 1} · a frame is ${shown(FRAME)} ms (60 Hz)`;
    for (const { row, lit } of [...lits, ...loopLits]) row.toggleAttribute("data-now", lit(t, playing));
    // A value changing every frame is noise to a screen reader: it is told where a held loop is.
    if (!held) {
      bar.removeAttribute("aria-valuenow");
      bar.setAttribute("aria-valuetext", "playing");
      return;
    }
    bar.setAttribute("aria-valuemin", "0");
    bar.setAttribute("aria-valuemax", String(Math.round(playing.total)));
    bar.setAttribute("aria-valuenow", String(Math.round(t)));
    bar.setAttribute("aria-valuetext", `${Math.round(t)} ms, frame ${at + 1}`);
  }

  /** Holds every animation at `t`. */
  function hold(t: number): void {
    time = t;
    for (const animation of animations) {
      animation.pause();
      animation.currentTime = t;
    }
    show(t, true);
  }

  /** Plays every animation on from `t`, in step, on the document's clock. */
  function run(t: number): void {
    time = t;
    const start = now() - t / rate();
    for (const animation of animations) {
      animation.playbackRate = rate();
      animation.startTime = start;
    }
    cancelAnimationFrame(frame);
    const tick = () => {
      if (!stage.isConnected) return;
      show(current(), false);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  }

  /** Plays or holds, as the button and whether the player is on screen say. */
  function go(t: number): void {
    cancelAnimationFrame(frame);
    if (running && visible) run(t);
    else hold(t);
  }

  /** `timed`'s animations on its targets inside `root`, paused, from the move's own keyframes. */
  function animate(root: Element, timed: Timed): Animation[] {
    const targets = [...root.querySelectorAll<HTMLElement>(timed.track.target)];
    if (targets.length === 0) throw new Error(`${move.name}: no ${timed.track.target} in its sample`);
    if (!CHROME) {
      // A shot holds the first state. It is set from the move's data rather than animated, since
      // a screenshot cancels every animation that never ends.
      for (const target of targets) target.style.setProperty(timed.track.property, String(css(timed.track.from)));
      return [];
    }
    const frames = keyframes(timed, playing.total);
    return targets.map((target) => {
      const animation = target.animate(frames, { duration: playing.total, iterations: Number.POSITIVE_INFINITY, fill: "both" });
      animation.pause();
      return animation;
    });
  }

  /**
   * Each part alone, for a move of more than one: the sample drawn again with the other parts'
   * layers hidden and only the controls the part is on, its layer animated by the same keyframes
   * on the player's one clock, so the part and the whole never disagree.
   */
  function drawParts(): Animation[] {
    if (playing.timed.length < 2) {
      parts.replaceChildren();
      return [];
    }
    const added: Animation[] = [];
    parts.replaceChildren(
      ...playing.timed.map((timed, i) => {
        const alone = move.sample();
        const target = timed.track.target;
        for (const other of playing.timed) {
          for (const layer of alone.querySelectorAll<HTMLElement>(other.track.target)) {
            if (!layer.matches(target)) layer.style.visibility = "hidden";
          }
        }
        if (alone.hasAttribute("data-layout")) {
          for (const child of [...alone.children]) {
            if (!child.matches(target) && !child.querySelector(target)) child.remove();
          }
        }
        added.push(...animate(alone, timed));
        const box = el("div", "page-part-stage mock", alone);
        // A picture of the part, not more controls: no focus, no hover, nothing read out twice.
        box.inert = true;
        box.setAttribute("aria-hidden", "true");
        const part = el("div", "page-part-alone", el("span", "page-plot-key", swatch(i), label(`${timed.track.part} (${timed.track.on})`, "small", "on_surface")), box);
        pickable(part, i);
        return part;
      }),
    );
    // Their rings' glows are drawn by the page's settle, which follows every build.
    return added;
  }

  /** Builds the animations from the move's data, for the settings picked, and goes on at `t`. */
  function build(t: number): void {
    for (const animation of animations) animation.cancel();
    pick(null);
    playing = loop(move);
    animations = playing.timed.flatMap((timed) => animate(stage, timed));
    animations.push(...drawParts());
    loopLits = [];
    loopPart.replaceChildren(loopTable(playing, loopLits));
    key.replaceChildren(legend(playing));
    for (const [i, item] of [...key.querySelectorAll(".page-plot-key")].entries()) pickable(item, i);
    const at = Math.min(t, playing.total - 1e-3);
    plot.draw(playing, marks(playing), at);
    go(at);
  }

  const playControl = control("Pause", null, () => {
    running = !running;
    playControl.textContent = running ? "Pause" : "Play";
    go(current());
  });
  if (!running) playControl.textContent = "Play";

  /** Stops the player at `t`, as seeking on the plot does. */
  function pause(t: number): void {
    running = false;
    playControl.textContent = "Play";
    go(Math.max(0, Math.min(t, playing.total - 1e-3)));
  }

  const slowControl = control(`Slow ×${SLOW}`, false, () => {
    const t = current();
    slow = !slow;
    slowControl.setAttribute("aria-pressed", String(slow));
    go(t);
  });
  const restart = control("Restart", null, () => {
    running = true;
    playControl.textContent = "Pause";
    go(0);
  });
  const copy = control("Copy as JSON", null, () => {
    void navigator.clipboard.writeText(JSON.stringify(recipe(move), null, 2)).then(() => {
      copy.textContent = "Copied";
      setTimeout(() => {
        copy.textContent = "Copy as JSON";
      }, milliseconds(HOLD));
    });
  });

  const seekTo = (event: PointerEvent) => pause(plot.timeAt(event.clientX));
  bar.addEventListener("pointerdown", (event) => {
    if (!event.isPrimary || event.button !== 0) return;
    // A mark pressed is its exact moment, wherever on its target the pointer is, and stays it: a
    // press on a mark isn't a drag.
    const mark = event.target instanceof Element ? event.target.closest(".page-plot-mark") : null;
    const at = mark?.getAttribute("data-at");
    if (at) {
      pause(Number(at));
      return;
    }
    bar.setPointerCapture(event.pointerId);
    seekTo(event);
  });
  bar.addEventListener("pointermove", (event) => {
    if (bar.hasPointerCapture(event.pointerId)) seekTo(event);
  });
  bar.addEventListener("keydown", (event) => {
    const by = event.shiftKey ? BIG_STEP : 1;
    const t0 = current();
    const at = frameOf(t0);
    // Between two frames' starts, ← goes back to the start of the frame it is in first.
    const back = Math.abs(t0 - at * FRAME) > 1e-6 ? by - 1 : by;
    const to: Record<string, number> = {
      ArrowLeft: Math.max(0, at - back) * FRAME,
      ArrowRight: Math.min(lastFrame(), at + by) * FRAME,
      Home: 0,
      End: lastFrame() * FRAME,
    };
    const t = to[event.key];
    if (t === undefined) return;
    event.preventDefault();
    pause(t);
  });

  const forget = onChange((_, changed) => {
    if (changed.has("reduced") || changed.has("speed") || changed.has("stutter")) build(current());
    else if (changed.has("scale")) plot.draw(playing, marks(playing), current());
  });
  for (const { row, set, index } of picks) pickable(row, index, set);
  // The curves are drawn again with the plot, so the plot listens for them.
  plot.element.addEventListener("pointerover", (event) => {
    const curve = event.target instanceof Element ? event.target.closest(".page-plot-curve-hit") : null;
    pick(curve ? Number(curve.getAttribute("data-pick")) : null);
  });
  plot.element.addEventListener("pointerleave", () => pick(null));
  build(0);

  whole.append(
    el("div", "page-player-words", label(move.name, "body_strong", "on_surface"), prose(move.what)),
    stage,
    el("div", "page-player-controls", playControl, restart, slowControl, copy),
    bar,
    readout,
    key,
    parts,
    prose(
      "The loop across, in milliseconds, holds included; each part's progress from its `from` (0) to its `to` (1) up, along its curve, a ball on it at the playhead. Drag or click the plot to hold a moment; ← and → step a frame, with Shift ten. The ticks under it are each part's start and end, and each keyframe: click one to go to it. Under the legend, a move of more than one part plays each alone on the same clock, the sample with the other parts hidden; pointing at a part's row, curve or stage picks it everywhere, outlined on the whole.",
      "small",
      "on_surface_variant",
    ),
    recipePart,
    loopPart,
  );
  // The plot is as wide as the player: drawn again when that changes.
  let drawnWidth = 0;
  const resize = new ResizeObserver(() => {
    if (bar.clientWidth === drawnWidth) return;
    drawnWidth = bar.clientWidth;
    plot.draw(playing, marks(playing), current());
  });
  resize.observe(bar);
  // Only a player on screen runs: one scrolled away holds where it is.
  const watch = new IntersectionObserver(([entry]) => {
    visible = entry?.isIntersecting ?? false;
    go(current());
  });
  watch.observe(whole);
  players.add(() => {
    forget();
    watch.disconnect();
    resize.disconnect();
    cancelAnimationFrame(frame);
    for (const animation of animations) animation.cancel();
  });
  return whole;
}

function moves(list: readonly Move[]): HTMLElement {
  return el("div", "page-players", ...list.map(player));
}

export function motion(spec: Motion): HTMLElement {
  return el(
    "div",
    "page-part",
    subheading("Arriving"),
    partOr(spec.arrives, moves),
    subheading("Leaving"),
    partOr(spec.leaves, moves),
    subheading("Between states"),
    partOr(spec.between, moves),
  );
}

/** Every move's recipe on a page, as JSON data: what `dist/motion/PAGE.json` holds. */
export function recipes(spec: Motion): Record<string, unknown> {
  const of = (list: readonly Move[] | DoesntApply) => (applies(list) ? list.map(recipe) : list);
  return { arrives: of(spec.arrives), leaves: of(spec.leaves), between: of(spec.between) };
}
