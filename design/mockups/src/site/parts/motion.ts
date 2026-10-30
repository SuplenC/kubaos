// The motion player: each move live on its sample, looping (its first state held, the move, its
// last state held, then back), with pause, restart and slow motion, a scrubber to seek it frame by
// frame, and its recipe beside it: every part, property, value, timing, curve and keyframe, what
// reduced motion and the speed make of it, as tables and as JSON. The sample is animated with the
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
  strikeSteps,
  tokenOf,
  tracksOf,
} from "../recipe.ts";
import { CHROME, onChange } from "../settings.ts";
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

function table(heads: readonly string[], rows: readonly HTMLElement[]): HTMLElement {
  return el("table", "page-table", el("thead", "", el("tr", "", ...heads.map((h) => cell(h, true)))), el("tbody", "", ...rows));
}

/** Whether `t` is inside one of `spans`. */
function within(t: number, spans: readonly Span[]): boolean {
  return spans.some((span) => t >= span.start && t < span.end);
}

/** The table of `set`'s tracks, each row lit while its part moves. */
function tracksTable(tracks: readonly Track[], set: TrackSet, lits: Lit[]): HTMLElement {
  const rows = tracks.map((track) => {
    const row = el(
      "tr",
      "",
      cell(track.part),
      cell(track.on),
      cell(`\`${track.property}\``),
      cell(valueWords(track.from)),
      cell(valueWords(track.to)),
      cell(`\`duration.${track.duration}\` ${milliseconds(track.duration)} ms`),
      cell(track.cue ? `\`duration.${track.cue}\` ${milliseconds(track.cue)} ms after the start` : "at the start"),
      cell(curveWords(track)),
    );
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
function recipeOf(move: Move, lits: Lit[]): HTMLElement {
  const of = (set: TrackSet, words: string): Element[] => {
    const tracks = tracksOf(move, set);
    if (!tracks) return [];
    return [label(words, "body_strong", "on_surface"), tracksTable(tracks, set, lits), ...keyframeTables(tracks, set, lits)];
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

/** The marks on the scrubber: each part's start and end, and each keyframe. */
function marks(playing: Loop): { at: number; kind: "part" | "key"; words: string }[] {
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
  let playing = loop(move);
  let animations: Animation[] = [];
  let running = CHROME;
  let visible = false;
  let slow = false;
  let time = 0;
  let frame = 0;

  const lits: Lit[] = [];
  const recipePart = recipeOf(move, lits);
  const loopPart = el("div", "");
  let loopLits: Lit[] = [];

  const head = el("div", "page-scrubber-head");
  const marksBox = el("div", "page-scrubber-marks");
  const bar = el("div", "page-scrubber", el("div", "page-scrubber-rail"), marksBox, head);
  bar.tabIndex = 0;
  bar.setAttribute("role", "slider");
  bar.setAttribute("aria-label", `${move.name}: where the playhead is`);
  const readout = el("div", "page-scrubber-time");

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

  function show(t: number): void {
    const at = frameOf(t);
    head.style.setProperty("--at", String(t / playing.total));
    readout.textContent = `${Math.round(t)} ms · frame ${at + 1} of ${lastFrame() + 1} · a frame is ${shown(FRAME)} ms (60 Hz)`;
    bar.setAttribute("aria-valuemin", "0");
    bar.setAttribute("aria-valuemax", String(Math.round(playing.total)));
    bar.setAttribute("aria-valuenow", String(Math.round(t)));
    bar.setAttribute("aria-valuetext", `${Math.round(t)} ms, frame ${at + 1}`);
    for (const { row, lit } of [...lits, ...loopLits]) row.toggleAttribute("data-now", lit(t, playing));
  }

  /** Holds every animation at `t`. */
  function hold(t: number): void {
    time = t;
    for (const animation of animations) {
      animation.pause();
      animation.currentTime = t;
    }
    show(t);
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
      show(current());
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

  /** Builds the animations from the move's data, for the settings picked, and goes on at `t`. */
  function build(t: number): void {
    for (const animation of animations) animation.cancel();
    playing = loop(move);
    animations = playing.timed.flatMap((timed) => {
      const targets = [...stage.querySelectorAll<HTMLElement>(timed.track.target)];
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
    });
    loopLits = [];
    loopPart.replaceChildren(loopTable(playing, loopLits));
    marksBox.replaceChildren(
      ...marks(playing).map((mark) => {
        const element = el("span", "page-scrubber-mark");
        element.dataset.kind = mark.kind;
        element.title = mark.words;
        element.style.setProperty("--at", String(mark.at / playing.total));
        element.dataset.at = String(mark.at);
        return element;
      }),
    );
    go(Math.min(t, playing.total - 1e-3));
  }

  const playControl = control("Pause", null, () => {
    running = !running;
    playControl.textContent = running ? "Pause" : "Play";
    go(current());
  });
  if (!running) playControl.textContent = "Play";

  /** Stops the player at `t`, as the scrubber does. */
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

  const seekTo = (event: PointerEvent) => {
    const box = bar.getBoundingClientRect();
    pause(((event.clientX - box.left) / box.width) * playing.total);
  };
  bar.addEventListener("pointerdown", (event) => {
    bar.setPointerCapture(event.pointerId);
    // A mark pressed is its exact moment, wherever on its width the pointer is.
    const mark = event.target instanceof HTMLElement ? event.target.closest<HTMLElement>(".page-scrubber-mark") : null;
    if (mark?.dataset.at) pause(Number(mark.dataset.at));
    else seekTo(event);
  });
  bar.addEventListener("pointermove", (event) => {
    if (bar.hasPointerCapture(event.pointerId)) seekTo(event);
  });
  bar.addEventListener("keydown", (event) => {
    const by = event.shiftKey ? BIG_STEP : 1;
    const at = frameOf(current());
    const to: Record<string, number> = {
      ArrowLeft: Math.max(0, at - by) * FRAME,
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
  });
  build(0);

  const whole = el(
    "div",
    "page-player",
    el("div", "page-player-words", label(move.name, "body_strong", "on_surface"), prose(move.what)),
    stage,
    el("div", "page-player-controls", playControl, restart, slowControl, copy),
    bar,
    readout,
    prose("Drag or click the bar to hold a moment; ← and → step a frame, with Shift ten. The marks are each part's start and end and each keyframe.", "small", "on_surface_variant"),
    recipePart,
    loopPart,
  );
  // Only a player on screen runs: one scrolled away holds where it is.
  const watch = new IntersectionObserver(([entry]) => {
    visible = entry?.isIntersecting ?? false;
    go(current());
  });
  watch.observe(whole);
  players.add(() => {
    forget();
    watch.disconnect();
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
