// The lock clock's editor, in the wallpaper picker's lock view (`pages/shell/wallpaper-parts.ts`):
// Clock (E) opens a panel beside the picture in front of you, which moves over in one move and
// shows its lock at rest, unpushed. The panel sets the clock for this picture and each screen
// shape: its place and size, how many of the picture's layers stand in front of it, its colour,
// its glow and its date. On the picture the clock has a box with four corners: dragged it moves,
// a corner or the wheel sizes it, and while it moves the guides show and hold it. At full screen
// (F) the panel is a drawer on the left or right edge, or floats where it is carried.
//
// Every move eases one number from where things stand, and the CSS draws each frame from it:
// `--ce-morph` on the stage mixes the picture's frame, its screen's shape (so its crop), its push
// and its clock between where they were (A) and where they go (B); `--ce-slide` on the panel mixes
// its place the same way; `--ce-dim` on the screen takes the wall away round the picture. A move
// that starts mid-move starts from where the last one has got to, on `standard`, so a turn has no
// kick. Only a drag runs a frame loop of its own.
//
// Everything is stored as shares of the screen (`ClockLook`); pixels are worked out where drawn.

import "./wallpaper-clock.css";
import { button } from "../../lib/button.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { icon } from "../../lib/icon.ts";
import { type ClockAt, type ClockDate, type ClockPlace, clockFigures, drawClock, inkEdges, type Lock, type LockClock, TIME, type Wallpaper } from "../../lib/lock.ts";
import { TINT_LIGHTNESS } from "../../lib/lock-tint.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { setSlider, slider } from "../../lib/slider.ts";
import { switchControl } from "../../lib/switch.ts";
import { tabs } from "../../lib/tabs.ts";
import { text, wrapped } from "../../lib/text.ts";
import { length, number } from "../../lib/tokens.ts";
import type { Duration, Easing } from "../../site/page.ts";
import { bezier, milliseconds } from "../../site/recipe.ts";
import { oklch } from "../../site/recolour.ts";
import { settings } from "../../site/settings.ts";
import { type ClockColour, colourPicker, customColour } from "./wallpaper-colour.ts";
import { stepsOf } from "./wallpaper-layers.ts";

export type Shape = "wide" | "portrait";
export type GlowMode = "off" | "dark" | "light" | "auto";

/** The clock as set for one picture: place and date per screen shape, the rest for both. */
export interface ClockLook {
  /** The clock shown on each shape: on unless set off. Off, no clock and no date. */
  shown: Partial<Record<Shape, boolean>>;
  /** The date over it on each shape: on unless set off. */
  dated: Partial<Record<Shape, boolean>>;
  place: Partial<Record<Shape, ClockPlace>>;
  date: Partial<Record<Shape, ClockDate>>;
  colour: ClockColour;
  glow: { mode: GlowMode; strength: number };
  /** How many of the picture's layers stand in front of the clock: 0, in front of them all. */
  depth: number;
  /** The last step but in front, which D goes back to. */
  last?: number;
}

/** A glow's strength before one is set, and the least it goes to: off is its own choice. */
const GLOW = { first: 0.5, least: 0.1 } as const;

/** The look a picture's clock has before it is set: the lock's own, behind the shipped cut-out. */
export function defaultLook(wallpaper: Wallpaper): ClockLook {
  return { shown: {}, dated: {}, place: {}, date: {}, colour: { mode: "auto" }, glow: { mode: "off", strength: GLOW.first }, depth: stepsOf(wallpaper).cut ?? 0 };
}

/** The look's lightness, for Auto's glow: the opposite of the clock's own. */
function lightness(look: ClockLook, wallpaper: Wallpaper): number {
  const colour = look.colour;
  if (colour.mode === "role") return oklch(wallpaper.night[colour.role] ?? "#ffffff").l;
  if (colour.mode === "custom") return colour.l;
  return TINT_LIGHTNESS;
}

/** The lock's clock for `look` on a screen of `shape`. */
export function lockClock(look: ClockLook, wallpaper: Wallpaper, shape: Shape): LockClock {
  const steps = stepsOf(wallpaper).steps;
  const colour = look.colour.mode === "role" ? `var(--color-${look.colour.role})` : look.colour.mode === "custom" ? customColour(look.colour.h, look.colour.l) : undefined;
  const dark = look.glow.mode === "dark" || (look.glow.mode === "auto" && lightness(look, wallpaper) >= GLOW_TURN);
  const place = look.place[shape];
  const date = look.date[shape];
  return {
    ...(place ? { place } : {}),
    ...(date ? { date } : {}),
    ...(colour ? { colour } : {}),
    ...(look.glow.mode === "off" ? {} : { glow: { colour: dark ? "var(--color-shadow)" : "var(--color-on_surface)", strength: look.glow.strength } }),
    layers: look.depth > 0 ? (steps[Math.min(look.depth, steps.length) - 1] ?? []) : [],
    ...(look.shown[shape] === false ? { showClock: false } : {}),
    ...(look.dated[shape] === false ? { showDate: false } : {}),
  };
}

/** The OKLCH lightness from which Auto's glow is dark under a light clock, light under a dark one. */
const GLOW_TURN = 0.6;

/**
 * What goes from the wall while editing, each easing its own `--ce-dim`: the number doesn't
 * inherit, so a frame of the fade restyles these alone, not every picture's lock inside them.
 */
export const DIMMED = ".gl-frame, .gl-lamp, .gl-placard, .gl-floors-panel";

/** The keyboard's and the wheel's steps for the clock's place and size, as shares: coarse and fine. */
const STEP = 0.01;
const FINE = 0.001;
/** A pull rips the drawer off only when it is this many times its travel along the edge: within about 27° of straight out. */
const PULL_STRAIGHT = 2;

/** The picture in front of you, as the gallery hands it over. */
export interface Front {
  readonly lock: Lock;
  readonly wallpaper: Wallpaper;
  readonly name: string;
  /** Its frame on the wall, which the editor takes the lock out of and gives it back to. */
  readonly frame: HTMLElement;
}

/** What the gallery lends the editor. */
export interface EditorHost {
  readonly screen: HTMLElement;
  front(): Front | undefined;
  look(front: Front): ClockLook;
  setLook(front: Front, look: ClockLook): void;
  /** The gallery's own full screen, which the editor leaves as it opens. */
  wallFull(): boolean;
  leaveWallFull(time: number): void;
  awake(): boolean;
  wake(on: boolean): Promise<void>;
  drawBar(): void;
  tipOn(control: HTMLElement, name: string, key: string): void;
  reduced(): boolean;
  /** Whether a person drives it: a sample drawn in a pose takes no pointer. */
  readonly live: boolean;
}

/** Where the panel stands: its card's left edge and top, and how far it is a drawer (0 to 1). */
interface Spot {
  cx: number;
  y: number;
  d: number;
}

/** The picture's frame on the screen, its scale and how round its corners are (0 to 1). */
interface Frame {
  readonly x: number;
  readonly y: number;
  readonly k: number;
  readonly round: number;
}

/** What the stage draws: the frame, the screen shape inside it, the push and the clock. */
interface Geom {
  readonly frame: Frame;
  readonly dw: number;
  readonly dh: number;
  readonly push: number;
  readonly at: ClockAt;
  readonly tint: string;
}

/** A pose the editor is drawn in at once, for a sample: as it stands, or a move's start. */
export interface EditorPose {
  readonly full?: boolean;
  readonly tab?: Shape;
  readonly drawer?: "closed" | "open" | "float";
  readonly side?: "left" | "right";
  /** The clock's box held, its guides shown. */
  readonly dragging?: boolean;
  /** The clock's box under the pointer. */
  readonly lit?: boolean;
  /** The look set, over the picture's own. */
  readonly look?: Partial<ClockLook>;
}

export interface Editor {
  editing(): boolean;
  enter(): Promise<void>;
  leave(keep: boolean): Promise<void>;
  /** A key while editing: `taken` it, `control` the focused control's, `wall` the gallery's. */
  key(event: KeyboardEvent): "taken" | "control" | "wall";
  /** The bar's parts while editing. */
  bar(): HTMLElement[];
  /** The picture in front of you changed while editing: edit the new one. */
  swap(): void;
  /** Draws the editor in a pose at once, or ready to move between two, for a sample. */
  pose(to: EditorPose | "wall", from?: EditorPose | "wall"): void;
}

export function clockEditor(host: EditorHost): Editor {
  const screen = host.screen;
  const stage = document.createElement("span");
  stage.className = "ce-stage";
  const frame = document.createElement("span");
  frame.className = "ce-frame";
  stage.append(frame);
  const panel = document.createElement("span");
  panel.className = "ce-panel";
  const body = document.createElement("span");
  body.className = "ce-body";
  panel.append(body);
  screen.append(stage, panel);
  new ResizeObserver(() => panel.style.setProperty("--ce-ph", `${panel.offsetHeight}px`)).observe(panel);

  // ─── The state. ───
  let editing = false;
  let full = false;
  let tab: Shape = "wide";
  let staged: Front | undefined;
  let before: ClockLook | undefined;
  const fs: { mode: "dock" | "float"; side: "left" | "right"; dockY: number | null; fx: number; fy: number } = { mode: "dock", side: "right", dockY: null, fx: 0, fy: 0 };
  let open = false;
  let ovSide: "left" | "right" = "right";
  let snapped: { x: number | null; y: number | "std" | null } = { x: null, y: null };
  let sliders: { x: HTMLElement; y: HTMLElement; s: HTMLElement } | undefined;
  let syncing = false;

  // ─── Measures, in the screen's CSS pixels. ───
  const px = () => Number.parseFloat(getComputedStyle(screen).getPropertyValue("--scale")) || 1;
  const tok = (name: string) => length(name) * px();
  const SW = () => screen.offsetWidth;
  const SH = () => screen.offsetHeight;
  const sizeOf = (shape: Shape) => (shape === "wide" ? { W: SW(), H: SH() } : { W: SH(), H: SW() });
  const reduced = () => host.reduced();
  const ms = (duration: Duration) => milliseconds(duration) / settings().speed;
  const curve = (easing: Easing) => `cubic-bezier(${bezier(easing).join(", ")})`;
  const curveFn = (easing: Easing) => {
    const [x1, y1, x2, y2] = bezier(easing) as [number, number, number, number];
    const bx = (t: number) => 3 * (1 - t) ** 2 * t * x1 + 3 * (1 - t) * t * t * x2 + t ** 3;
    const by = (t: number) => 3 * (1 - t) ** 2 * t * y1 + 3 * (1 - t) * t * t * y2 + t ** 3;
    return (x: number) => {
      let lo = 0;
      let hi = 1;
      for (let i = 0; i < 30; i += 1) {
        const mid = (lo + hi) / 2;
        if (bx(mid) < x) lo = mid;
        else hi = mid;
      }
      return by((lo + hi) / 2);
    };
  };
  const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
  const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

  const look = () => host.look(staged!);
  const wallpaper = () => staged!.wallpaper;

  // ─── The clock's place and its limits, per screen shape. ───

  /** The standard place and size on `shape`: what the lock does with nothing set. */
  function standard(shape: Shape = tab): ClockPlace {
    const { W, H } = sizeOf(shape);
    const at = staged!.lock.clockAt({}, W, H);
    return { x: at.cx / W, y: (at.top + clockFigures().cap * at.size) / H, s: at.size / Math.min(W, H) };
  }

  /** The date as set on `shape`, or the tokens' share. */
  function dateOf(shape: Shape = tab): ClockDate {
    const set = look().date[shape];
    if (set) return set;
    const portrait = shape === "portrait";
    return { size: number(portrait ? "scale-lock_date_portrait" : "scale-lock_date"), gap: number(portrait ? "scale-lock_date_gap_portrait" : "scale-lock_date_gap"), align: "centre" };
  }

  /** How far the digits' baseline may stand down a screen `H` tall so that awake the clock stays `space.s16` clear of the password's sheet. */
  function awakeBase(H: number): number {
    const a = number("scale-clock_awake");
    const sheet = tok("size-control_md") + 2 * tok("space-s16");
    const barBase = (tok("size-bar") + tok("type-body-size") * clockFigures().cap) / 2;
    return (H - sheet - tok("space-s16") - a * barBase) / (1 - a);
  }

  /**
   * The room the clock may stand in on `shape`: its size within `scale.clock_least` and
   * `scale.clock_most` of the shorter side, its widest time and its date `space.s32` inside the
   * screen, and its digits clear of the password's sheet awake.
   */
  function limits(shape: Shape, s: number) {
    const { W, H } = sizeOf(shape);
    const m = Math.min(W, H);
    const { cap, widest } = clockFigures();
    const k = dateOf(shape);
    const inset = tok("space-s32");
    const sMax = Math.min(number("scale-clock_most"), (W - 2 * inset) / widest / m, (H - 2 * inset) / m / (cap * 1.5));
    const sMin = number("scale-clock_least");
    const size = clamp(s, sMin, sMax);
    const f = size * m;
    const half = (widest * f) / 2;
    // With no date the clock's own cap height is all that stands over its baseline.
    const dated = look().dated[shape] !== false;
    const kick = dated ? Math.max(k.size * f, tok("size-lock_date_least")) * cap + k.gap * f : 0;
    return {
      sMin,
      sMax,
      s: size,
      xMin: (inset + half) / W,
      xMax: 1 - (inset + half) / W,
      yMin: (inset + kick + cap * f) / H,
      yMax: Math.min(1 - inset / H, awakeBase(H) / H),
    };
  }

  const fit = (p: ClockPlace, shape: Shape = tab): ClockPlace => {
    const L = limits(shape, p.s);
    return { x: clamp(p.x, L.xMin, L.xMax), y: clamp(p.y, L.yMin, L.yMax), s: L.s };
  };
  const current = () => fit(look().place[tab] ?? standard(), tab);

  // ─── The stage: the picture's frame, its shape, its push and its clock. ───

  /** The front frame where the wall has it, `--out` of the way to filling the screen. */
  function wallFrame(): Frame {
    const share = number("scale-gallery_picture");
    const out = Number.parseFloat(getComputedStyle(screen).getPropertyValue("--out")) || 0;
    const zoom = 1 + (1 / share - 1) * out;
    const top = SH() * number("scale-gallery_top");
    const middle = top + (SH() * share) / 2;
    const cy = middle + (SH() / 2 - middle) * out;
    const k = share * zoom;
    return { x: SW() / 2 - (SW() * k) / 2, y: cy - (SH() * k) / 2, k, round: 1 - out };
  }

  /** The frame beside the panel, or filling the screen. */
  function editFrame(shape: Shape, whole: boolean): Frame {
    const { W, H } = sizeOf(shape);
    if (whole) {
      const k = Math.min(SW() / W, SH() / H);
      const fills = Math.abs(W * k - SW()) < 0.5 && Math.abs(H * k - SH()) < 0.5;
      return { x: (SW() - W * k) / 2, y: (SH() - H * k) / 2, k, round: fills ? 0 : 1 };
    }
    const m = tok("space-s32");
    const room = SW() - tok("size-panel") - 2 * m;
    // The room above the bar: the wall's own, as far down as its pictures' top edge is from the top.
    const tall = SH() * (1 - number("scale-gallery_top")) - 2 * m;
    const k = Math.min((room - m) / W, (shape === "wide" ? number("scale-gallery_picture") * SH() : tall) / H);
    return { x: m + (room - W * k) / 2, y: m + (tall - H * k) / 2, k, round: 1 };
  }

  /** What the stage shows in the state the editor is in. */
  function target(): Geom {
    const shape: Shape = editing ? tab : "wide";
    const { W, H } = sizeOf(shape);
    const clock = lockClock(look(), wallpaper(), shape);
    const at = staged!.lock.clockAt(clock, W, H);
    const tint = clock.colour ?? staged!.lock.tintAt(at, W, H) ?? "var(--color-on_surface)";
    return { frame: editing ? editFrame(shape, full) : wallFrame(), dw: W, dh: H, push: editing || reduced() ? 0 : 1, at, tint };
  }

  let gA: Geom | undefined;
  let gB: Geom | undefined;
  let morphRun: Animation | undefined;
  const morphT = () => (morphRun?.playState === "running" || morphRun?.playState === "finished" ? Number.parseFloat(getComputedStyle(stage).getPropertyValue("--ce-morph")) : Number.parseFloat(stage.style.getPropertyValue("--ce-morph")) || 1);
  const mixAt = (a: ClockAt, b: ClockAt, t: number): ClockAt => {
    const out: Record<string, number> = {};
    for (const key of Object.keys(b) as (keyof ClockAt)[]) out[key] = lerp(a[key], b[key], t);
    return out as unknown as ClockAt;
  };
  /** The stage as drawn this moment, mid-move included. */
  function drawn(): Geom {
    if (!gA || !gB) return target();
    const t = morphT();
    if (t >= 1) return gB;
    const f = (key: keyof Frame) => lerp(gA!.frame[key], gB!.frame[key], t);
    return {
      frame: { x: f("x"), y: f("y"), k: f("k"), round: f("round") },
      dw: lerp(gA.dw, gB.dw, t),
      dh: lerp(gA.dh, gB.dh, t),
      push: lerp(gA.push, gB.push, t),
      at: mixAt(gA.at, gB.at, t),
      tint: gA.tint === gB.tint ? gB.tint : `color-mix(in oklab, ${gB.tint} ${(t * 100).toFixed(2)}%, ${gA.tint})`,
    };
  }

  /** Writes the stage between `gA` and `gB`, as far as `--ce-morph` says. */
  function drawStage(): void {
    if (!gA || !gB || !staged) return;
    const a = gA;
    const b = gB;
    const mix = (from: number, to: number, unit: string) => (Math.abs(from - to) < 1e-4 ? `${to}${unit}` : `calc(${from}${unit} + ${to - from}${unit} * var(--ce-morph))`);
    stage.style.setProperty("--ce-x", mix(a.frame.x, b.frame.x, "px"));
    stage.style.setProperty("--ce-y", mix(a.frame.y, b.frame.y, "px"));
    stage.style.setProperty("--ce-k", mix(a.frame.k, b.frame.k, ""));
    stage.style.setProperty("--ce-round", mix(a.frame.round, b.frame.round, ""));
    stage.style.setProperty("--ce-dw", mix(a.dw, b.dw, "px"));
    stage.style.setProperty("--ce-dh", mix(a.dh, b.dh, "px"));
    const lock = staged.lock.element;
    lock.style.setProperty("--push", mix(a.push, b.push, ""));
    const clock = lock.querySelector<HTMLElement>(".k-lock-clock")!;
    drawClock(clock, b.at, a.at, "--ce-morph");
    clock.style.setProperty("--digits-tint", a.tint === b.tint ? b.tint : `color-mix(in oklab, ${b.tint} calc(var(--ce-morph) * 100%), ${a.tint})`);
    drawGuides();
  }

  /**
   * Eases the stage from what it shows now to what `change` makes it show, in one move: over
   * `long` on `emphasized_decel`, or on `standard` turning a move under way round. With reduce
   * motion the stage fades out and back in round the change, over `short` each way.
   */
  async function morph(change: () => void, duration: Duration = "long"): Promise<boolean> {
    const turning = morphRun?.playState === "running";
    const from = drawn();
    change();
    const to = target();
    staged!.lock.setClock(lockClock(look(), wallpaper(), editing ? tab : "wide"));
    morphRun?.cancel();
    if (reduced()) {
      const fade = stage.animate([{ opacity: 1 }, { opacity: 0 }], { duration: ms("short"), easing: curve("standard") });
      morphRun = fade;
      const done = await fade.finished.then(
        () => true,
        () => false,
      );
      if (!done) return false;
      gA = gB = to;
      stage.style.setProperty("--ce-morph", "1");
      drawStage();
      const back = stage.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard") });
      morphRun = back;
      return back.finished.then(
        () => true,
        () => false,
      );
    }
    gA = from;
    gB = to;
    // Held at the start until the move has run: no frame can draw its end before its first.
    stage.style.setProperty("--ce-morph", "0");
    drawStage();
    settleLayout();
    const run = stage.animate([{ "--ce-morph": "0" }, { "--ce-morph": "1" }], { duration: ms(duration), easing: curve(turning ? "standard" : "emphasized_decel"), fill: "forwards" });
    morphRun = run;
    const done = await run.finished.then(
      () => true,
      () => false,
    );
    if (done && morphRun === run) {
      gA = gB;
      stage.style.setProperty("--ce-morph", "1");
      drawStage();
      run.cancel();
    }
    return done;
  }

  /**
   * Lays out what a move's start changed now, before its clock starts: a move is timed from its
   * first frame, so a slow first frame would draw it already far along, a jump.
   */
  function settleLayout(): void {
    void screen.getBoundingClientRect();
  }

  /** Draws the stage as the state says at once: a change made while dragging, or a sample. */
  function settleStage(): void {
    morphRun?.cancel();
    gA = gB = target();
    stage.style.setProperty("--ce-morph", "1");
    drawStage();
  }

  /** The clock's place as set, the stage following at once and the panel's rows with it. */
  function setPlace(p: ClockPlace): void {
    look().place[tab] = fit(p);
    staged!.lock.setClock(lockClock(look(), wallpaper(), tab));
    settleStage();
    if (sliders && !syncing) syncSliders();
  }

  function syncSliders(): void {
    if (!sliders) return;
    const p = current();
    const L = limits(tab, p.s);
    setSlider(sliders.x, p.x);
    setSlider(sliders.y, p.y);
    setSlider(sliders.s, (p.s - L.sMin) / Math.max(1e-6, L.sMax - L.sMin));
  }

  /** Takes the front picture's lock out of its frame onto the stage, held for the editor to draw. */
  function stageFront(front: Front): void {
    staged = front;
    const lock = front.lock.element;
    lock.dataset.ceWidth = lock.style.getPropertyValue("--screen-width");
    lock.dataset.ceHeight = lock.style.getPropertyValue("--screen-height");
    front.lock.hold(true);
    frame.append(lock);
    lock.style.setProperty("--screen-width", "var(--ce-dw)");
    lock.style.setProperty("--screen-height", "var(--ce-dh)");
    front.frame.closest(".gl-art")?.toggleAttribute("data-staged", true);
    const clock = lock.querySelector<HTMLElement>(".k-lock-clock")!;
    clock.append(boxOf());
    lock.append(guides);
    stage.toggleAttribute("data-shown", true);
  }

  /** Gives the lock back to its frame, drawn by itself again with its look. */
  function unstage(): void {
    const front = staged;
    if (!front) return;
    const lock = front.lock.element;
    lock.style.setProperty("--screen-width", lock.dataset.ceWidth ?? "");
    lock.style.setProperty("--screen-height", lock.dataset.ceHeight ?? "");
    delete lock.dataset.ceWidth;
    delete lock.dataset.ceHeight;
    lock.style.removeProperty("--push");
    lock.querySelector(".ce-box")?.remove();
    guides.remove();
    front.frame.append(lock);
    front.frame.closest(".gl-art")?.toggleAttribute("data-staged", false);
    front.lock.setClock(lockClock(host.look(front), front.wallpaper, "wide"));
    front.lock.hold(false);
    stage.toggleAttribute("data-shown", false);
    staged = undefined;
  }

  // ─── The clock's box, its corners, and the guides. ───

  const guides = document.createElement("span");
  guides.className = "ce-guides";
  const GUIDES = ["v50", "v33", "v67", "h50", "h33", "h67", "hstd"] as const;
  const safe = document.createElement("span");
  safe.className = "ce-safe";
  guides.append(safe, ...GUIDES.map((name) => {
    const line = document.createElement("span");
    line.className = `ce-guide ce-${name}`;
    line.dataset.guide = name;
    return line;
  }));

  /** The guides where this shape puts them: the standard baseline and the room the clock may move in. */
  function drawGuides(): void {
    if (!staged || !editing) return;
    const { H } = sizeOf(tab);
    const L = limits(tab, current().s);
    guides.style.setProperty("--ce-std", String(standard().y));
    guides.style.setProperty("--ce-floor", `${H - L.yMax * H}px`);
    for (const line of guides.querySelectorAll<HTMLElement>(".ce-guide")) {
      const name = line.dataset.guide!;
      const lit = (snapped.x !== null && name === `v${Math.round(snapped.x * 100)}`) || (snapped.y === "std" ? name === "hstd" : snapped.y !== null && name === `h${Math.round(snapped.y * 100)}`);
      line.toggleAttribute("data-lit", lit);
    }
  }

  function boxOf(): HTMLElement {
    const box = document.createElement("span");
    box.className = "ce-box";
    for (const corner of ["nw", "ne", "sw", "se"]) {
      const handle = document.createElement("span");
      handle.className = "ce-handle";
      handle.dataset.corner = corner;
      box.append(handle);
    }
    box.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      const corner = (event.target as HTMLElement).dataset.corner;
      startDrag(event, corner ?? "move");
    });
    box.addEventListener(
      "wheel",
      (event) => {
        event.preventDefault();
        const p = current();
        setPlace({ ...p, s: p.s - Math.sign(event.deltaY) * (event.shiftKey ? FINE : STEP) });
      },
      { passive: false },
    );
    return box;
  }

  /** The pointer in shares of the screen shape, through the frame. */
  function shareAt(event: PointerEvent): { x: number; y: number } {
    const box = staged!.lock.element.getBoundingClientRect();
    return { x: (event.clientX - box.left) / box.width, y: (event.clientY - box.top) / box.height };
  }

  /** The box moves the clock; a corner sizes it about the corner across; the guides pull it, unless Alt is held. */
  function startDrag(event: PointerEvent, mode: string): void {
    if (!host.live || !editing || !staged) return;
    event.preventDefault();
    event.stopPropagation();
    const p0 = current();
    const at0 = shareAt(event);
    const box = staged.lock.element.querySelector(".ce-box")!.getBoundingClientRect();
    const desk = staged.lock.element.getBoundingClientRect();
    const anchor = {
      x: ((mode.includes("w") ? box.right : box.left) - desk.left) / desk.width,
      y: ((mode.includes("n") ? box.bottom : box.top) - desk.top) / desk.height,
    };
    const h0 = Math.abs(at0.y - anchor.y);
    const snap = number("scale-clock_snap");
    screen.toggleAttribute("data-ce-dragging", true);
    const move = (moved: PointerEvent) => {
      const at = shareAt(moved);
      snapped = { x: null, y: null };
      if (mode === "move") {
        let x = p0.x + at.x - at0.x;
        let y = p0.y + at.y - at0.y;
        if (!moved.altKey) {
          for (const g of [0.5, 1 / 3, 2 / 3]) if (Math.abs(x - g) < snap) [x, snapped.x] = [g, g];
          for (const g of [0.5, 1 / 3, 2 / 3]) if (Math.abs(y - g) < snap) [y, snapped.y] = [g, g];
          const std = standard().y;
          if (Math.abs(y - std) < snap) [y, snapped.y] = [std, "std"];
        }
        setPlace({ x, y, s: p0.s });
      } else {
        const L = limits(tab, p0.s);
        const s = clamp(p0.s * Math.max(0.1, Math.abs(at.y - anchor.y) / h0), L.sMin, L.sMax);
        const r = s / p0.s;
        setPlace({ x: anchor.x + (p0.x - anchor.x) * r, y: anchor.y + (p0.y - anchor.y) * r, s });
      }
    };
    const up = () => {
      snapped = { x: null, y: null };
      screen.toggleAttribute("data-ce-dragging", false);
      removeEventListener("pointermove", move);
      removeEventListener("pointerup", up);
      drawGuides();
    };
    addEventListener("pointermove", move);
    addEventListener("pointerup", up);
    drawGuides();
  }

  // ─── The panel. ───

  // The locale's percentages: a value is never put together from a number and a sign by hand.
  const SHARE = new Intl.NumberFormat("en-GB", { style: "percent", minimumFractionDigits: 1, maximumFractionDigits: 1 });
  const WHOLE = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 0 });

  /** A row: its label in the panel's one column of labels, its control in the other. */
  function row(name: string, label: string, control: HTMLElement, sub = false): HTMLElement {
    const made = document.createElement("span");
    made.className = "ce-row";
    made.toggleAttribute("data-sub", sub);
    made.dataset.row = name;
    made.append(text(label, "label", "on_surface_variant"), control);
    return made;
  }
  /** A setting with sub-settings: one well round the parent row and its sub-rows. */
  function well(...rows: HTMLElement[]): HTMLElement {
    const made = document.createElement("span");
    made.className = "ce-well";
    made.append(...rows);
    return made;
  }
  function heading(words: string): HTMLElement {
    const made = text(words, "label", "on_surface_variant");
    made.classList.add("ce-heading");
    return made;
  }
  /** A range slider, `value` mapped from [`least`, `most`] to the slider's 0 to 1. */
  function ranged(options: { icon: string; value: number; least: number; most: number; fine?: number; words: (value: number) => string; disabled?: boolean; changed: (value: number) => void }): HTMLElement {
    const span = Math.max(1e-6, options.most - options.least);
    return slider({
      icon: options.icon,
      size: "Compact",
      value: (options.value - options.least) / span,
      number: (v) => options.words(options.least + v * span),
      widest: options.words(options.most),
      ...(options.fine !== undefined ? { fine: options.fine / span } : {}),
      ...(options.disabled ? { disabled: true } : {}),
      changed: (v) => options.changed(options.least + v * span),
    });
  }

  /** Redraws the panel for a change made in it, focus kept on its row. */
  function refresh(): void {
    const focused = (document.activeElement as HTMLElement | null)?.closest<HTMLElement>(".ce-row")?.dataset.row;
    fill();
    if (focused) requestAnimationFrame(() => focusRow(focused));
  }

  /** The look changed: the lock takes it and the stage draws it at once. */
  function restyle(): void {
    staged!.lock.setClock(lockClock(look(), wallpaper(), tab));
    settleStage();
  }

  function fill(): void {
    if (!staged) return;
    const front = staged;
    const set = look();
    const p = current();
    const L = limits(tab, p.s);
    // Off, every row that sets the clock or its date stays where it is, disabled, so nothing moves.
    const shown = set.shown[tab] !== false;
    const dated = shown && set.dated[tab] !== false;
    const toggle = (on: boolean, flip: (on: boolean) => void, disabled = false) => {
      const made = switchControl({ on, disabled });
      made.addEventListener("click", (event) => {
        event.stopPropagation();
        flip(made.getAttribute("aria-checked") === "true");
        restyle();
        refresh();
      });
      return made;
    };
    const clockSwitch = toggle(shown, (on) => (on ? delete set.shown[tab] : (set.shown[tab] = false)));
    const dateSwitch = toggle(dated, (on) => (on ? delete set.dated[tab] : (set.dated[tab] = false)), !shown);
    const place = (which: "x" | "y" | "s", value: number, least: number, most: number, glyph: string) =>
      ranged({
        icon: glyph,
        value,
        least,
        most,
        fine: FINE,
        disabled: !shown,
        words: (v) => SHARE.format(v),
        changed: (v) => {
          syncing = true;
          setPlace({ ...current(), [which]: v });
          syncing = false;
        },
      });
    sliders = { x: place("x", p.x, 0, 1, "arrow_forward"), y: place("y", p.y, 0, 1, "arrow_downward"), s: place("s", p.s, L.sMin, L.sMax, "schedule") };

    // Wide or portrait: the frame, the picture's crop and the clock morph from one to the other.
    const screens = tabs({
      fill: true,
      choices: [{ label: "Wide screen" }, { label: "Portrait screen" }],
      picked: tab === "wide" ? 0 : 1,
      onPick: (index) => {
        const to: Shape = index === 0 ? "wide" : "portrait";
        if (to === tab) return;
        void morph(() => (tab = to));
        refresh();
      },
    });

    // Depth reads as how far forward the clock stands: lowest behind every layer but the farthest,
    // each step up in front of one more, highest in front of everything. Stored as layers in front.
    const n = stepsOf(front.wallpaper).steps.length;
    const flat = front.lock.element.hasAttribute("data-flat");
    const names = Array.from({ length: n + 1 }, (_, f) => (f === n ? "In front" : `${f + 1} of ${n + 1}`));
    const depth =
      n > 0 && !flat && shown
        ? slider({
            icon: "deployed_code",
            size: "Compact",
            stops: n + 1,
            index: n - Math.min(set.depth, n),
            names,
            // The clock passes behind one layer after another under the pointer.
            live: true,
            changed: (f) => {
              set.depth = n - f;
              if (set.depth > 0) set.last = set.depth;
              restyle();
            },
          })
        : slider({ icon: "deployed_code", size: "Compact", stops: Math.max(2, n + 1), index: n > 0 && !flat ? n - Math.min(set.depth, n) : Math.max(1, n), names: n > 0 && !flat ? names : ["", "In front"], disabled: true });
    const depthNote = n === 0 ? wrapped("This picture has no depth layers: the clock stands in front.", "small", "on_surface_variant") : flat ? wrapped("Depth is off for this picture: the clock stands in front.", "small", "on_surface_variant") : null;

    // Colour: Auto, the scheme's roles, or Custom with its Hue and Tone.
    const night = getComputedStyle(front.lock.element);
    const picker = colourPicker(
      set.colour,
      (role) => night.getPropertyValue(`--color-${role}`).trim(),
      (colour) => {
        set.colour = colour;
        restyle();
        refresh();
      },
      (h, l) => {
        set.colour = { mode: "custom", h, l };
        restyle();
        picker.choice.querySelector<HTMLElement>(".ce-swatch[data-custom]")?.style.setProperty("--swatch", customColour(h, l));
      },
      !shown,
    );

    // Glow: Off, Dark, Light or Auto (the one opposing the clock's colour), and its strength.
    const glow = tabs({
      fill: true,
      choices: [{ label: "Off" }, { label: "Dark" }, { label: "Light" }, { label: "Auto" }],
      picked: ["off", "dark", "light", "auto"].indexOf(set.glow.mode),
      disabled: !shown,
      onPick: (index) => {
        set.glow.mode = (["off", "dark", "light", "auto"] as const)[index]!;
        restyle();
        refresh();
      },
    });
    const strength = ranged({
      icon: "light_mode",
      value: set.glow.strength,
      least: GLOW.least,
      most: 1,
      words: (v) => WHOLE.format(v),
      disabled: !shown || set.glow.mode === "off",
      changed: (v) => {
        set.glow.strength = v;
        restyle();
      },
    });

    // The date: its edge on the digits' ink, its size and its gap, as shares of the clock's size.
    // Its size never under `size.lock_date_least` nor wider than the digits' ink, and neither its
    // size nor its gap takes it past the top of the allowed area: there the slider stops.
    const date = dateOf();
    const setDate = (patch: Partial<ClockDate>) => (set.date[tab] = { ...dateOf(), ...patch });
    const align = tabs({
      fill: true,
      choices: [{ label: "Left" }, { label: "Centre" }, { label: "Right" }],
      picked: ["left", "centre", "right"].indexOf(date.align),
      disabled: !dated,
      onPick: (index) => {
        void morph(() => setDate({ align: (["left", "centre", "right"] as const)[index]! }));
        refresh();
      },
    });
    const room = () => {
      const q = current();
      const { W, H } = sizeOf(tab);
      const font = q.s * Math.min(W, H);
      const at = front.lock.clockAt(lockClock(set, front.wallpaper, tab), W, H);
      const ink = clockFigures();
      const edges = inkEdges(TIME);
      const inkWidth = at.digits * px() - (edges.left + edges.right) * font;
      const perSize = ((at.place + at.date) * px()) / Math.max(1, at.kickerSize);
      return { font, least: tok("size-lock_date_least") / font, widest: inkWidth / perSize / font, above: q.y * H - ink.cap * font - tok("space-s32") };
    };
    const r = room();
    const sizeMost = Math.min(r.widest, number("scale-clock_date_most"));
    const sizeLeast = Math.min(r.least, sizeMost);
    const gapLeast = number("scale-clock_gap_least");
    const gapMost = number("scale-clock_gap_most");
    const cap = clockFigures().cap;
    let dateSize: HTMLElement;
    let dateGap: HTMLElement;
    dateSize = ranged({
      icon: "event",
      value: clamp(date.size, sizeLeast, sizeMost),
      least: sizeLeast,
      most: sizeMost,
      fine: FINE,
      disabled: !dated,
      words: (v) => SHARE.format(v),
      changed: (v) => {
        const q = room();
        const most = (q.above - dateOf().gap * q.font) / (q.font * cap);
        const c = clamp(v, sizeLeast, Math.min(sizeMost, most));
        setDate({ size: c });
        if (c !== v) setSlider(dateSize, (c - sizeLeast) / Math.max(1e-6, sizeMost - sizeLeast));
        restyle();
      },
    });
    dateGap = ranged({
      icon: "swap_vert",
      value: clamp(date.gap, gapLeast, gapMost),
      least: gapLeast,
      most: gapMost,
      fine: FINE,
      disabled: !dated,
      words: (v) => SHARE.format(v),
      changed: (v) => {
        const q = room();
        const most = (q.above - Math.max(dateOf().size * q.font, tok("size-lock_date_least")) * cap) / q.font;
        const c = clamp(v, gapLeast, Math.min(gapMost, most));
        setDate({ gap: c });
        if (c !== v) setSlider(dateGap, (c - gapLeast) / Math.max(1e-6, gapMost - gapLeast));
        restyle();
      },
    });

    const standardPlace = button({ label: "Standard place", kind: "Text", size: "Sm", icon: "restart_alt" });
    host.tipOn(standardPlace, "Standard place", "R");
    standardPlace.addEventListener("click", (event) => (event.stopPropagation(), resetAll()));

    const content = document.createElement("span");
    content.className = "ce-rows";
    const depthGroup = document.createElement("span");
    depthGroup.className = "ce-group";
    depthGroup.append(row("depth", "Depth", depth), ...(depthNote ? [depthNote] : []));
    const placeGroup = document.createElement("span");
    placeGroup.className = "ce-group";
    placeGroup.append(heading("Place"), row("x", "Across", sliders.x), row("y", "Down", sliders.y), row("s", "Size", sliders.s));
    const lookGroup = document.createElement("span");
    lookGroup.className = "ce-group";
    lookGroup.append(
      heading("Look"),
      well(row("colour", "Colour", picker.choice), row("hue", "Hue", picker.hue, true), row("tone", "Tone", picker.tone, true)),
      well(row("glow", "Glow", glow), row("strength", "Strength", strength, true)),
      well(row("date", "Date", dateSwitch), row("align", "Align", align, true), row("dsize", "Size", dateSize, true), row("dgap", "Gap", dateGap, true)),
    );
    content.append(row("tabs", "Screen", screens), row("clock", "Clock", clockSwitch), placeGroup, depthGroup, lookGroup);
    const made = panelPage({
      title: "Clock",
      subtitle: `${front.name} · each screen its own`,
      body: content,
      footer: [text("Place, depth, look and date", "small", "on_surface_variant"), standardPlace],
    });
    body.replaceChildren(made.element);
  }

  /** The rows the keys walk: every row whose control takes them now. */
  const rows = () => [...panel.querySelectorAll<HTMLElement>(".ce-row")].filter((each) => focusable(each) !== null);
  const focusable = (each: HTMLElement) =>
    each.querySelector<HTMLElement>(".k-slider:not([aria-disabled='true']), .ce-hue:not([aria-disabled='true']), .k-tabs:not([aria-disabled='true']) .k-tab[aria-selected='true'], .ce-swatches:not([aria-disabled='true']), .k-switch:not(:disabled)");
  function focusRow(name: string): void {
    const found = rows().find((each) => each.dataset.row === name) ?? rows()[0];
    if (found) focusable(found)?.focus({ preventScroll: true });
  }
  function moveRow(by: number): void {
    const all = rows();
    const now = all.findIndex((each) => each.contains(document.activeElement));
    const next = all[clamp(now + by, 0, all.length - 1)];
    if (next) focusable(next)?.focus({ preventScroll: true });
  }

  function resetAll(): void {
    void morph(() => {
      const set = look();
      delete set.place[tab];
      delete set.date[tab];
      set.colour = { mode: "auto" };
      set.glow = { mode: "off", strength: GLOW.first };
      set.depth = stepsOf(wallpaper()).cut ?? 0;
      delete set.shown[tab];
      delete set.dated[tab];
    });
    refresh();
  }

  /** D: in front, or back to the last step but in front. */
  function toggleDepth(): void {
    const set = look();
    const n = stepsOf(wallpaper()).steps.length;
    if (n === 0 || staged!.lock.element.hasAttribute("data-flat") || set.shown[tab] === false) return;
    if (set.depth > 0) {
      set.last = set.depth;
      set.depth = 0;
    } else set.depth = set.last ?? stepsOf(wallpaper()).cut ?? n;
    restyle();
    refresh();
  }

  // ─── The panel's place: beside the frame, or at full screen a drawer or floating. ───

  let pA: Spot = { cx: 0, y: 0, d: 0 };
  let pB: Spot = { cx: 0, y: 0, d: 0 };
  let slideRun: Animation | undefined;
  const panelW = () => tok("size-panel");
  const short = () => Math.min(SW(), SH());
  const STICKY = () => number("scale-dock_sticky") * short();
  const PULL = () => Math.max(number("scale-dock_pull") * short(), tok("space-s16"));
  const RUBBER = () => number("scale-dock_rubber");
  const STRIP = () => tok("space-s12");
  const NEAR = () => tok("space-s4");
  const slideT = () => (slideRun?.playState === "running" || slideRun?.playState === "finished" ? Number.parseFloat(getComputedStyle(panel).getPropertyValue("--ce-slide")) : Number.parseFloat(panel.style.getPropertyValue("--ce-slide")) || 1);
  const spotNow = (): Spot => {
    const t = slideT();
    return { cx: lerp(pA.cx, pB.cx, t), y: lerp(pA.y, pB.y, t), d: lerp(pA.d, pB.d, t) };
  };
  const framedAt = (): Spot => ({ cx: SW() - tok("space-s24") - panelW(), y: tok("space-s24"), d: 0 });
  const offAt = (): Spot => ({ cx: SW(), y: tok("space-s24"), d: 0 });
  const openX = (side: "left" | "right") => (side === "right" ? SW() - panelW() : 0);
  const closedX = (side: "left" | "right") => (side === "right" ? SW() - STRIP() : -panelW() + STRIP());
  const dockLimits = () => ({ y0: 0, y1: Math.max(0, SH() - panel.offsetHeight) });
  const floatLimits = () => ({ y0: tok("space-s32"), y1: Math.max(tok("space-s32"), SH() - tok("space-s32") - panel.offsetHeight) });
  const clampDockY = (y: number) => clamp(y, dockLimits().y0, dockLimits().y1);
  const clampFloatY = (y: number) => clamp(y, floatLimits().y0, floatLimits().y1);

  function drawPanel(): void {
    const mix = (from: number, to: number, unit: string) => (Math.abs(from - to) < 1e-4 ? `${to}${unit}` : `calc(${from}${unit} + ${to - from}${unit} * var(--ce-slide))`);
    panel.style.setProperty("--ce-px", mix(pA.cx, pB.cx, "px"));
    panel.style.setProperty("--ce-py", mix(pA.y, pB.y, "px"));
    panel.style.setProperty("--ce-pd", mix(pA.d, pB.d, ""));
    panel.style.setProperty("--ce-left", ovSide === "left" ? "1" : "0");
  }

  /** Puts the panel at `spot` at once. */
  function placePanel(spot: Spot): void {
    slideRun?.cancel();
    pA = pB = { ...spot };
    panel.style.setProperty("--ce-slide", "1");
    drawPanel();
  }

  /**
   * Eases the panel from where it is to `to`, over `long` on `emphasized_decel`, or on `standard`
   * turning a move under way round; with reduce motion it fades out and back in round the change.
   */
  async function tweenTo(to: Partial<Spot>): Promise<boolean> {
    const turning = slideRun?.playState === "running";
    const from = spotNow();
    slideRun?.cancel();
    if (reduced()) {
      const fade = panel.animate([{ opacity: 1 }, { opacity: 0 }], { duration: ms("short"), easing: curve("standard") });
      slideRun = fade;
      if (!(await fade.finished.then(() => true, () => false))) return false;
      placePanel({ ...from, ...to });
      const back = panel.animate([{ opacity: 0 }, { opacity: 1 }], { duration: ms("short"), easing: curve("standard") });
      slideRun = back;
      return back.finished.then(() => true, () => false);
    }
    pA = from;
    pB = { ...from, ...to };
    panel.style.setProperty("--ce-slide", "0");
    drawPanel();
    settleLayout();
    const run = panel.animate([{ "--ce-slide": "0" }, { "--ce-slide": "1" }], { duration: ms("long"), easing: curve(turning ? "standard" : "emphasized_decel"), fill: "forwards" });
    slideRun = run;
    const done = await run.finished.then(() => true, () => false);
    if (done && slideRun === run) {
      pA = pB;
      panel.style.setProperty("--ce-slide", "1");
      drawPanel();
      run.cancel();
    }
    return done;
  }

  const atFull = () => editing && full;
  /** The pointer in the screen's CSS pixels. */
  const toScreen = (event: PointerEvent | MouseEvent) => {
    const box = screen.getBoundingClientRect();
    const k = box.width / Math.max(1, screen.offsetWidth);
    return { x: (event.clientX - box.left) / k, y: (event.clientY - box.top) / k };
  };
  type Point = { x: number; y: number };
  const within = (pt: Point, r: { x0: number; x1: number; y0: number; y1: number }) => pt.x >= r.x0 && pt.x <= r.x1 && pt.y >= r.y0 && pt.y <= r.y1;
  /** What opens the docked panel: the screen's last `space.s32` on its side, beside it and `space.s32` past it. */
  function inTrigger(pt: Point): boolean {
    if (fs.mode !== "dock") return false;
    const y = spotNow().y;
    const h = panel.offsetHeight;
    const m = tok("space-s32");
    return within(pt, { x0: fs.side === "right" ? SW() - m : -1e4, x1: fs.side === "right" ? 1e4 : m, y0: y - m, y1: y + h + m });
  }
  /** What keeps it open: the open drawer and `space.s16` round it, out to its edge, and the trigger. */
  function inKeep(pt: Point): boolean {
    if (fs.mode !== "dock") return false;
    const y = spotNow().y;
    const h = panel.offsetHeight;
    const m = tok("space-s16");
    const span = fs.side === "right" ? { x0: SW() - panelW() - m, x1: 1e4 } : { x0: -1e4, x1: panelW() + m };
    if (onBar(pt)) return false;
    return within(pt, { ...span, y0: y - m, y1: y + h + m }) || inTrigger(pt);
  }
  /** Whether the pointer is on a part of the bar that shows, not under the panel. */
  function onBar(pt: Point): boolean {
    const box = screen.getBoundingClientRect();
    const k = box.width / Math.max(1, screen.offsetWidth);
    const c = { x: box.left + pt.x * k, y: box.top + pt.y * k };
    const p = panel.getBoundingClientRect();
    if (c.x >= p.left && c.x <= p.right && c.y >= p.top && c.y <= p.bottom) return false;
    return [...screen.querySelectorAll(".gl-bar, .gl-info")].some((each) => {
      const b = each.getBoundingClientRect();
      return c.x >= b.left && c.x <= b.right && c.y >= b.top && c.y <= b.bottom;
    });
  }

  let pointer: Point | undefined;
  let closeTimer = 0;
  let keysUp = false;
  let keyTimer = 0;
  let downOnTrigger = false;
  /** Opens or closes the docked drawer. */
  function peek(on: boolean): void {
    if (!atFull() || fs.mode !== "dock" || drag || open === on) return;
    open = on;
    void tweenTo({ cx: on ? openX(fs.side) : closedX(fs.side) });
  }
  /** Closing waits `duration.short` after the pointer leaves, and never starts mid-opening. */
  function wantClose(): void {
    if (!closeTimer) closeTimer = window.setTimeout(tryClose, ms("short"));
  }
  function cancelClose(): void {
    window.clearTimeout(closeTimer);
    closeTimer = 0;
  }
  function tryClose(): void {
    closeTimer = 0;
    if (!atFull() || fs.mode !== "dock" || drag || !open) return;
    if (slideRun?.playState === "running") {
      closeTimer = window.setTimeout(tryClose, ms("micro"));
      return;
    }
    if (keysUp || panel.querySelector(".k-slider[data-dragging], .ce-hue[data-held]") || (pointer && inKeep(pointer))) return;
    peek(false);
  }

  // The drag: anywhere on the panel at full screen but its controls.
  let drag: { mode: "dock" | "float"; p0: Point; y0: number; gx: number; gy: number; anchor: Point; caught: { t0: number; from: Spot } | null; inSticky: "left" | "right" | null } | null = null;
  let dragFrame = 0;
  const CONTROLS = ".k-control, .k-slider, .k-tabs, .ce-swatches, .ce-hue, button, [role='slider']";

  panel.addEventListener("pointerdown", (event) => {
    if (!host.live || !atFull() || event.button !== 0 || (event.target as Element).closest(CONTROLS)) return;
    // Only a drawer at rest open is grabbed: one still opening slides under the pointer.
    if (fs.mode === "dock" && (!open || slideRun?.playState === "running")) return;
    event.preventDefault();
    cancelClose();
    const pt = toScreen(event);
    const st = spotNow();
    placePanel(st);
    drag = { mode: fs.mode, p0: pt, y0: st.y, gx: pt.x - st.cx, gy: pt.y - st.y, anchor: pt, caught: null, inSticky: null };
    pointer = pt;
    panel.toggleAttribute("data-dragging", true);
    addEventListener("pointerup", dragUp, { once: true });
    dragFrame = requestAnimationFrame(dragTick);
  });

  function dragTick(now: number): void {
    if (!drag) return;
    dragStep(now);
    dragFrame = requestAnimationFrame(dragTick);
  }

  /** One frame of the drag: where the pointer asks the panel to be, eased in over `duration.medium` after a rip or a catch. */
  function dragStep(now: number): void {
    const g = drag!;
    const pt = pointer!;
    const P = panelW();
    let want: Spot;
    if (g.mode === "dock") {
      const out = Math.max(0, fs.side === "right" ? g.p0.x - pt.x : pt.x - g.p0.x);
      const along = Math.abs(pt.y - g.anchor.y);
      const y = clampDockY(g.y0 + pt.y - g.p0.y);
      if (out > PULL() && out > PULL_STRAIGHT * along) {
        // Rips free: floats from here, under the pointer as it was grabbed.
        g.mode = "float";
        fs.mode = "float";
        open = false;
        g.gx = clamp(g.gx, tok("space-s16"), P - tok("space-s16"));
        g.caught = { t0: now, from: { ...pB } };
        return dragStep(now);
      }
      const pull = out * RUBBER();
      want = { cx: openX(fs.side) + (fs.side === "right" ? -pull : pull), y, d: 1 };
    } else {
      const icx = pt.x - g.gx;
      const y = clampFloatY(pt.y - g.gy);
      const dR = SW() - (icx + P);
      const dL = icx;
      const side = dR < dL ? "right" : "left";
      const dist = Math.min(dR, dL);
      // The push into the sticky step: how far past its start the pointer has carried the panel.
      const push = STICKY() - dist;
      if (push > PULL()) {
        // Through: it sticks to the border, the drawer at this height, still under the pointer.
        const dy = clampDockY(y);
        fs.mode = "dock";
        fs.side = side;
        fs.dockY = dy;
        ovSide = side;
        open = true;
        g.mode = "dock";
        g.p0 = pt;
        g.y0 = dy;
        g.anchor = pt;
        g.inSticky = null;
        g.caught = { t0: now, from: { ...pB } };
        return dragStep(now);
      }
      let cx: number;
      if (push > 0) {
        // The step: it holds back, moving `scale.dock_rubber` of the push further on, its gap still open.
        const gap = STICKY() - push * RUBBER();
        cx = side === "right" ? SW() - P - gap : gap;
        ovSide = side;
      } else cx = clamp(icx, tok("space-s32"), SW() - tok("space-s32") - P);
      g.inSticky = dist < STICKY() ? side : null;
      want = { cx, y, d: 0 };
    }
    if (g.caught) {
      const t = clamp((now - g.caught.t0) / ms("medium"), 0, 1);
      const e = reduced() ? 1 : curveFn("standard")(t);
      want = { cx: lerp(g.caught.from.cx, want.cx, e), y: lerp(g.caught.from.y, want.y, e), d: lerp(g.caught.from.d, want.d, e) };
      if (t >= 1) g.caught = null;
    }
    pA = pB = want;
    drawPanel();
  }

  function dragUp(): void {
    const g = drag;
    if (!g) return;
    drag = null;
    cancelAnimationFrame(dragFrame);
    panel.toggleAttribute("data-dragging", false);
    const st = spotNow();
    if (g.mode === "dock") {
      fs.dockY = clampDockY(st.y);
      void tweenTo({ cx: openX(fs.side), y: fs.dockY, d: 1 });
    } else {
      // Let go before pushing through: it stays floating where it shows (the sticky step's gap is
      // past the `space.s32` inset, so this is already inside it).
      const cx = g.inSticky ? st.cx : clamp(st.cx, tok("space-s32"), SW() - tok("space-s32") - panelW());
      fs.fx = cx;
      fs.fy = clampFloatY(st.y);
      void tweenTo({ cx, y: fs.fy, d: 0 });
    }
  }

  const onPointerMove = (event: PointerEvent) => {
    if (!host.live || !atFull()) return;
    pointer = toScreen(event);
    if (drag) {
      if (drag.mode === "dock") {
        const out = Math.max(0, fs.side === "right" ? drag.p0.x - pointer.x : pointer.x - drag.p0.x);
        // The pull is measured from where it began: a pull mostly along the edge is a slide.
        if (out < NEAR()) drag.anchor = pointer;
      }
      return;
    }
    if (fs.mode !== "dock") return;
    const held = panel.querySelector(".k-slider[data-dragging], .ce-hue[data-held]") !== null;
    if (event.buttons && !held) return;
    if (!open) {
      if (inTrigger(pointer)) {
        cancelClose();
        peek(true);
      }
      return;
    }
    if (held || inKeep(pointer)) cancelClose();
    else wantClose();
  };
  // A press that starts elsewhere (dragging the clock to the edge) is no click on it.
  const onPointerDown = (event: PointerEvent) => (downOnTrigger = host.live && atFull() && inTrigger(toScreen(event)));
  const onClick = (event: MouseEvent) => {
    if (atFull() && downOnTrigger && !open && inTrigger(toScreen(event))) {
      cancelClose();
      peek(true);
    }
  };
  /** The page's pointer, heard only while editing: the drawer's edge zone lies outside the panel. */
  function listen(on: boolean): void {
    const way = on ? addEventListener : removeEventListener;
    way("pointermove", onPointerMove);
    way("pointerdown", onPointerDown, true);
    way("click", onClick, true);
  }

  // The keyboard: the panel's own keys bring the docked drawer out; it closes `duration.hold` after
  // the last of them, unless the pointer is over it. A floating panel is always open.
  const PANEL_KEYS = new Set(["j", "k", "h", "l", "J", "K", "H", "L", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Tab", "Home", "End"]);
  function revealForKeys(event: KeyboardEvent): void {
    if (!atFull() || fs.mode !== "dock" || event.ctrlKey || event.metaKey || event.altKey || !PANEL_KEYS.has(event.key)) return;
    cancelClose();
    peek(true);
    keysUp = true;
    window.clearTimeout(keyTimer);
    keyTimer = window.setTimeout(() => {
      keysUp = false;
      if (atFull() && fs.mode === "dock" && !(pointer && inKeep(pointer))) peek(false);
    }, ms("hold"));
  }

  // ─── Full screen, entering and leaving. ───

  /** Full screen while configuring: the frame fills the screen and the panel goes to where it was last there. */
  function setFull(on: boolean): void {
    if (!editing || on === full) return;
    void morph(() => (full = on));
    screen.toggleAttribute("data-ce-full", on);
    screen.style.setProperty("--step-time", `${reduced() ? ms("short") : ms("long")}ms`);
    drag = null;
    if (on) {
      ovSide = fs.side;
      panel.toggleAttribute("data-edge", true);
      fs.dockY ??= tok("space-s32");
      if (fs.mode === "dock") {
        open = false;
        void tweenTo({ cx: closedX(fs.side), y: clampDockY(fs.dockY), d: 1 });
      } else void tweenTo({ cx: fs.fx, y: clampFloatY(fs.fy), d: 0 });
    } else {
      cancelClose();
      void tweenTo(framedAt()).then((done) => done && !full && panel.toggleAttribute("data-edge", false));
    }
  }

  /** Opens the panel: the frame moves over beside it and the lock rests unpushed, in one move. */
  async function enter(): Promise<void> {
    const front = host.front();
    if (editing || !front) return;
    if (host.awake()) await host.wake(false);
    before = structuredClone(host.look(front));
    listen(true);
    if (host.wallFull()) host.leaveWallFull(reduced() ? ms("short") : ms("long"));
    gA = undefined;
    gB = undefined;
    stageFront(front);
    // The first frame is the wall's own: the stage stands where the frame is before it moves.
    tab = "wide";
    settleStage();
    editing = true;
    full = false;
    screen.toggleAttribute("data-editing", true);
    screen.toggleAttribute("data-ce-full", false);
    dimWall(true);
    fill();
    panel.toggleAttribute("data-shown", true);
    panel.toggleAttribute("data-edge", false);
    placePanel(offAt());
    host.drawBar();
    void tweenTo(framedAt());
    requestAnimationFrame(() => panel.querySelector<HTMLElement>(".k-slider")?.focus({ preventScroll: true }));
    await morph(() => {});
  }

  /** Closes it: the same move back, the panel sliding out, the look kept or the old one back. */
  async function leave(keep: boolean): Promise<void> {
    if (!editing || !staged) return;
    const front = staged;
    if (!keep && before) host.setLook(front, before);
    listen(false);
    cancelClose();
    drag = null;
    const wasFull = full;
    editing = false;
    sliders = undefined;
    screen.toggleAttribute("data-editing", false);
    screen.toggleAttribute("data-ce-full", false);
    dimWall(false);
    host.drawBar();
    screen.focus({ preventScroll: true });
    if (wasFull && panel.hasAttribute("data-edge")) {
      const st = spotNow();
      const out = st.cx + panelW() / 2 < SW() / 2 ? -panelW() - tok("space-s32") * 3 : SW() + tok("space-s32") * 3;
      void tweenTo({ cx: out, d: 0 }).then((done) => done && !editing && hidePanel());
    } else void tweenTo(offAt()).then((done) => done && !editing && hidePanel());
    full = false;
    const done = await morph(() => {});
    if (done && !editing) unstage();
  }

  function hidePanel(): void {
    panel.toggleAttribute("data-shown", false);
    panel.toggleAttribute("data-edge", false);
    body.replaceChildren();
  }

  let dimRuns: Animation[] = [];
  /** The wall round the picture goes while editing, and comes back after. */
  function dimWall(on: boolean): void {
    const parts = [...screen.querySelectorAll<HTMLElement>(DIMMED)];
    const from = Number.parseFloat(getComputedStyle(parts[0] ?? screen).getPropertyValue("--ce-dim")) || 0;
    for (const run of dimRuns) run.cancel();
    dimRuns = parts.map((part) => {
      part.style.setProperty("--ce-dim", on ? "1" : "0");
      return part.animate([{ "--ce-dim": String(from) }, { "--ce-dim": on ? "1" : "0" }], { duration: reduced() ? ms("short") : ms("long"), easing: curve("standard") });
    });
  }

  // ─── The keys. ───
  function key(event: KeyboardEvent): "taken" | "control" | "wall" {
    revealForKeys(event);
    const k = event.key;
    const inPanel = panel.contains(event.target as Node);
    if (k === "j" || k === "ArrowDown") return moveRow(1), "taken";
    if (k === "k" || k === "ArrowUp") return moveRow(-1), "taken";
    if (k === "r" || k === "R") return resetAll(), "taken";
    if (k === "d" || k === "D") return toggleDepth(), "taken";
    if (k === "f" || k === "F") return setFull(!full), "taken";
    // Space and Enter press a focused switch, as they press a button; Enter elsewhere is Done.
    if (k === "Enter" && (event.target as Element).closest?.(".k-switch")) return "control";
    if (k === "Enter") return void leave(true), "taken";
    // Esc first closes the docked drawer at full screen, then cancels.
    if (k === "Escape") {
      if (atFull() && fs.mode === "dock" && open) {
        cancelClose();
        peek(false);
      } else void leave(false);
      return "taken";
    }
    if (inPanel && ["h", "l", "H", "L", "ArrowLeft", "ArrowRight", "Home", "End", " "].includes(k)) return "control";
    if (k === "ArrowLeft" || k === "ArrowRight" || k === "w" || k === "W") return "wall";
    if (k === "Tab" || k === "?") return "control";
    return "taken";
  }

  /** The bar while editing: what it edits, Full screen and back, Cancel and Done. */
  function bar(): HTMLElement[] {
    const title = document.createElement("span");
    title.className = "ce-bar-title";
    title.append(icon("schedule", "var(--type-label-icon)", "var(--color-on_surface)"), text("Clock", "label"));
    const stepper = document.createElement("span");
    stepper.className = "gl-step-pair ce-step";
    for (const [label, whole] of [["Full screen", true], ["Back to the wall", false]] as const) {
      const made = button({ label, kind: "Text", size: "Sm", icon: "desktop_windows" });
      host.tipOn(made, label, "F");
      made.addEventListener("click", (event) => (event.stopPropagation(), setFull(whole)));
      const cell = document.createElement("span");
      cell.append(made);
      stepper.append(cell);
    }
    const rule = document.createElement("span");
    rule.className = "gl-bar-rule";
    const actions = buttonBar({ dismiss: "Cancel", actions: [{ label: "Done", kind: "Filled" }], size: "Sm" });
    host.tipOn(actions.dismiss!, "Cancel", "Esc");
    actions.dismiss!.addEventListener("click", (event) => (event.stopPropagation(), void leave(false)));
    const done = actions.actions[0]!;
    host.tipOn(done, "Done", "↵");
    done.addEventListener("click", (event) => (event.stopPropagation(), void leave(true)));
    return [title, stepper, rule, actions.element];
  }

  /** The picture in front of you changed while editing: the stage edits the new one, at once. */
  function swap(): void {
    if (!editing) return;
    const front = host.front();
    if (!front || front === staged) return;
    unstage();
    before = structuredClone(host.look(front));
    stageFront(front);
    settleStage();
    fill();
  }

  /**
   * Draws the editor in `to` at once; with `from`, drawn at `to` with each move's number ready to
   * take it from `from`, for a move's sample: the player eases `--ce-morph` and `--ce-slide`.
   */
  function pose(to: EditorPose | "wall", from?: EditorPose | "wall"): void {
    const front = host.front();
    if (!front) return;
    if (!staged) stageFront(front);
    const apply = (shown: EditorPose | "wall") => {
      if (shown === "wall") {
        editing = false;
        full = false;
        tab = "wide";
      } else {
        if (shown.look) host.setLook(front, { ...host.look(front), ...structuredClone(shown.look) });
        editing = true;
        full = shown.full ?? false;
        tab = shown.tab ?? "wide";
        fs.side = shown.side ?? "right";
        fs.mode = shown.drawer === "float" ? "float" : "dock";
        fs.dockY = tok("space-s32");
        open = shown.drawer === "open";
        ovSide = fs.side;
      }
      staged!.lock.setClock(lockClock(look(), wallpaper(), editing ? tab : "wide"));
    };
    const spot = (shown: EditorPose | "wall"): Spot => {
      if (shown === "wall") return offAt();
      if (!full) return framedAt();
      if (fs.mode === "float") return { cx: (SW() - panelW()) / 2, y: clampFloatY(tok("space-s64")), d: 0 };
      return { cx: open ? openX(fs.side) : closedX(fs.side), y: fs.dockY ?? tok("space-s32"), d: 1 };
    };
    // The parts a player moves are there at once; where they stand waits for the gallery to be
    // laid out and its fonts to be in, since everything is placed from the screen's size.
    const shown = to === "wall" ? from : to;
    panel.toggleAttribute("data-shown", true);
    const laid = new ResizeObserver(() => {
      if (screen.offsetWidth === 0) return;
      laid.disconnect();
      void document.fonts.ready.then(() => posed(shown));
    });
    laid.observe(screen);
    const posed = (shown: EditorPose | "wall" | undefined) => {
      if (shown && shown !== "wall") apply(shown);
      fill();
      host.drawBar();
      if (from) {
        apply(from);
        gA = target();
        pA = spot(from);
      }
      apply(to);
      gB = target();
      pB = spot(to);
      if (!from) {
        gA = gB;
        pA = pB;
      }
      stage.style.setProperty("--ce-morph", "1");
      panel.style.setProperty("--ce-slide", "1");
      drawStage();
      drawPanel();
      const edge = to === "wall" ? from !== undefined && from !== "wall" && (from.full ?? false) : full;
      screen.toggleAttribute("data-editing", editing);
      screen.toggleAttribute("data-ce-full", full);
      panel.toggleAttribute("data-edge", edge);
      for (const part of screen.querySelectorAll<HTMLElement>(DIMMED)) part.style.setProperty("--ce-dim", editing ? "1" : "0");
      if (to !== "wall" && to.dragging) {
        screen.toggleAttribute("data-ce-dragging", true);
        snapped = { x: 0.5, y: "std" };
        drawGuides();
      }
      staged?.lock.element.querySelector(".ce-box")?.toggleAttribute("data-lit", to !== "wall" && (to.lit ?? false));
    };
  }

  return { editing: () => editing, enter, leave, key, bar, swap, pose };
}
