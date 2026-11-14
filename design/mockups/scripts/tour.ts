// Renders the wallpaper picker's tour as numbered PNG frames, one every 1/60 s of the page's own
// time, for ffmpeg to join into a film (`README.md`, "The Gallery's tour").
//
//   node scripts/tour.ts OUT [--size 2560x1440] [--scale 1.5] [--from A] [--frames B] [--port 5183]
//
// OUT gets 00000.png, 00001.png, …, beats.txt, each beat's first frame and time, and trace-A.tsv,
// each frame's page time, the front picture's box, the step out and the room's light, to show the
// motion even. Only frames A to B (B left out) are written: the tour still plays from its start on
// the same clock, so a run in segments draws the frames one whole run would, and `--frames` alone is
// a trial. The live Gallery is opened on the whole viewport through the site's own Full screen
// control, at night, and driven by key presses alone: no pointer, so no hover and no tooltip. Vite
// serves the site on PORT for the run and stops with it.
//
// The page's time is its own: a clock installed before any of the page's scripts stands in for
// `performance.now`, `Date`, timers, animation frames and idle callbacks, and holds every animation
// paused, setting each to where it is at the page's time on every frame. Chromium's virtual time
// (`Emulation.setVirtualTimePolicy`) moves timers and animation frames but not the document
// timeline that transitions and the Web Animations run on, which keeps real time.

import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type CDPSession, type Page } from "playwright-core";
import { createServer } from "vite";
import { chromiumPath } from "./browser.ts";

const FPS = 60;
/** How long the picker rests between moves, once every animation has ended and no timer is near. */
const REST = 1.8;
/** The longest a move may take, in seconds: past it, something never stops. */
const LONGEST = 8;

/** A key pressed and what it shows, or a rest. */
interface Beat {
  readonly name: string;
  readonly key?: string;
  /** Seconds held after the move ends. */
  readonly rest: number;
}

const TOUR: readonly Beat[] = [
  { name: "the desktop at rest", rest: 2 },
  { name: "opening: the desktop steps back into the gallery, the lamp strikes", key: "Control+Meta+Space", rest: REST },
  { name: "the next picture", key: "ArrowRight", rest: REST },
  { name: "the one after", key: "ArrowRight", rest: REST },
  { name: "the floor above", key: "ArrowUp", rest: REST },
  { name: "back down", key: "ArrowDown", rest: REST },
  { name: "the Desktop view", key: "2", rest: REST },
  { name: "the Lock view", key: "3", rest: REST },
  { name: "the lock wakes", key: "w", rest: REST },
  { name: "the lock rests", key: "w", rest: REST },
  { name: "depth off", key: "d", rest: REST },
  { name: "depth on", key: "d", rest: REST },
  { name: "full screen", key: "f", rest: REST },
  { name: "back to the wall", key: "f", rest: REST },
  { name: "applied: the picture grows into the desktop", key: "Enter", rest: 2.5 },
];

/** The page's clock, as the tour drives it. */
interface TourClock {
  /** Moves the page's time on one frame and runs what falls due in it. */
  step(): Promise<void>;
  /** Whether nothing moves: no animation with an end still running, and no timer due soon. */
  quiet(): boolean;
}

declare global {
  interface Window {
    tourClock: TourClock;
  }
}

/**
 * Puts the page on a clock of its own that moves only on `step`, `frame` milliseconds a time. It
 * runs in the page before its scripts, so it takes nothing from this module's scope.
 */
function installClock(frame: number): void {
  const realFrame = window.requestAnimationFrame.bind(window);
  const RealDate = Date;
  const epoch = RealDate.now();
  let now = 0;
  let nextId = 1;
  const timers = new Map<number, { at: number; run: () => void; every?: number }>();
  let frames = new Map<number, FrameRequestCallback>();
  const idles = new Map<number, IdleRequestCallback>();

  // A macrotask's turn, so the microtasks a callback queued run before the next callback, as the
  // browser's own task loop does.
  const channel = new MessageChannel();
  const turns: (() => void)[] = [];
  channel.port1.onmessage = () => turns.shift()?.();
  const nextTurn = () =>
    new Promise<void>((done) => {
      turns.push(done);
      channel.port2.postMessage(null);
    });

  Object.defineProperty(performance, "now", { value: () => now });
  class VirtualDate extends RealDate {
    constructor(...args: unknown[]) {
      if (args.length === 0) super(epoch + now);
      // The arguments are passed on as given: a timestamp, a string, or the parts of a date.
      else super(...(args as [number]));
    }
    static override now(): number {
      return epoch + now;
    }
  }
  window.Date = VirtualDate as DateConstructor;

  const later = (handler: TimerHandler, timeout: number | undefined, args: unknown[], every: boolean): number => {
    if (typeof handler !== "function") throw new Error("the tour's clock runs no string timers");
    const id = nextId++;
    const wait = Math.max(0, timeout ?? 0);
    timers.set(id, { at: now + wait, run: () => handler(...args), ...(every ? { every: Math.max(1, wait) } : {}) });
    return id;
  };
  window.setTimeout = ((handler: TimerHandler, timeout?: number, ...args: unknown[]) => later(handler, timeout, args, false)) as typeof window.setTimeout;
  window.setInterval = ((handler: TimerHandler, timeout?: number, ...args: unknown[]) => later(handler, timeout, args, true)) as typeof window.setInterval;
  // The page's types allow Node's timer handles too; in the page an id is always a number.
  const clear = ((id?: number) => void timers.delete(id ?? 0)) as typeof window.clearTimeout;
  window.clearTimeout = window.clearInterval = clear;
  window.requestAnimationFrame = (callback) => {
    const id = nextId++;
    frames.set(id, callback);
    return id;
  };
  window.cancelAnimationFrame = (id) => void frames.delete(id);
  window.requestIdleCallback = (callback) => {
    const id = nextId++;
    idles.set(id, callback);
    return id;
  };
  window.cancelIdleCallback = (id) => void idles.delete(id);

  // Every animation is held paused and set to where it is at the page's time: `base` its time when
  // the clock took it, `from` the page's time then. One the page pauses itself stays where it put it.
  const taken = new WeakMap<Animation, { from: number; base: number }>();
  const held = new WeakSet<Animation>();
  // The animations made in script, until they end: `document.getAnimations` leaves out one on an
  // element outside the document, which would never reach its end and resolve `finished`.
  const made = new Set<Animation>();
  const proto = Animation.prototype;
  const pause = proto.pause;
  const play = proto.play;
  const time = Object.getOwnPropertyDescriptor(proto, "currentTime")!;
  const timeOf = (animation: Animation) => Number(time.get!.call(animation) ?? 0);
  const setTime = (animation: Animation, at: number) => time.set!.call(animation, at);
  const take = (animation: Animation) => {
    taken.set(animation, { from: now, base: timeOf(animation) });
    pause.call(animation);
  };
  proto.pause = function (this: Animation) {
    held.add(this);
    pause.call(this);
  };
  proto.play = function (this: Animation) {
    held.delete(this);
    play.call(this);
    take(this);
    made.add(this);
  };
  Object.defineProperty(proto, "currentTime", {
    get: time.get!,
    set(this: Animation, at: number | null) {
      setTime(this, at ?? 0);
      if (taken.has(this)) taken.set(this, { from: now, base: at ?? 0 });
    },
    configurable: true,
  });
  const animate = Element.prototype.animate;
  Element.prototype.animate = function (this: Element, ...args: Parameters<Element["animate"]>) {
    const animation = animate.apply(this, args);
    take(animation);
    made.add(animation);
    return animation;
  };

  const endOf = (animation: Animation) => Number(animation.effect?.getComputedTiming().endTime ?? Number.POSITIVE_INFINITY);

  /** Every animation running or held: the document's, and those made in script. */
  const animations = (): Set<Animation> => {
    for (const animation of made) if (animation.playState === "finished" || animation.playState === "idle") made.delete(animation);
    return new Set([...document.getAnimations(), ...made]);
  };

  /** Sets every animation to the page's time; true when it met one it hadn't taken yet. */
  const sync = (): boolean => {
    let met = false;
    for (const animation of animations()) {
      if (held.has(animation) || animation.playState === "finished" || animation.playState === "idle") continue;
      let at = taken.get(animation);
      if (!at) {
        // A transition or a CSS animation, started by a style the last turn set: it starts now.
        at = { from: now, base: 0 };
        taken.set(animation, at);
        met = true;
      }
      const rate = animation.playbackRate;
      const t = at.base + (now - at.from) * rate;
      if ((rate > 0 && t >= endOf(animation)) || (rate < 0 && t <= 0)) animation.finish();
      else {
        if (animation.playState !== "paused") pause.call(animation);
        setTime(animation, t);
      }
    }
    return met;
  };

  window.tourClock = {
    async step() {
      now += frame;
      for (;;) {
        let due: [number, { at: number; run: () => void; every?: number }] | undefined;
        for (const entry of timers) if (entry[1].at <= now && (!due || entry[1].at < due[1].at)) due = entry;
        if (!due) break;
        const [id, timer] = due;
        if (timer.every === undefined) timers.delete(id);
        else timer.at += timer.every;
        timer.run();
        await nextTurn();
      }
      const ran = frames;
      frames = new Map();
      for (const callback of ran.values()) callback(now);
      await nextTurn();
      const idle = idles.entries().next();
      if (!idle.done) {
        idles.delete(idle.value[0]);
        idle.value[1]({ didTimeout: false, timeRemaining: () => frame / 2 });
        await nextTurn();
      }
      // A real frame between passes lets the browser end what `finish` ended: the `finished`
      // promises resolve and what waits on them runs, at this frame's time.
      for (let pass = 0; pass < 6; pass++) {
        const met = sync();
        await new Promise<void>((done) => realFrame(() => done()));
        if (!met && pass > 0) break;
      }
      sync();
    },
    quiet() {
      const moving = [...animations()].some((each) => !held.has(each) && each.playState !== "finished" && each.playState !== "idle" && Number.isFinite(endOf(each)));
      const soon = [...timers.values()].some((timer) => timer.every === undefined && timer.at <= now + 3000);
      return !moving && !soon;
    },
  };
}

function usage(): never {
  process.stderr.write("usage: node scripts/tour.ts OUT [--size 2560x1440] [--scale 1.5] [--from A] [--frames B] [--port 5183]\n");
  process.exit(2);
}

/** Writes one frame of the page as it is now. */
async function shoot(cdp: CDPSession, file: string): Promise<void> {
  const shot = await cdp.send("Page.captureScreenshot", { format: "png", optimizeForSpeed: true });
  writeFileSync(file, Buffer.from(shot.data, "base64"));
}

/** Moves the page's time until `done` holds, at most `limit` frames. */
async function pump(page: Page, done: () => Promise<boolean>, limit: number): Promise<void> {
  for (let i = 0; i < limit; i++) {
    if (await done()) return;
    await page.evaluate(() => window.tourClock.step());
  }
  throw new Error("the page never got there");
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const out = args.shift();
  if (!out || out.startsWith("-")) usage();
  let [width, height] = [2560, 1440];
  let scale = "1.5";
  let from = 0;
  let limit = Number.POSITIVE_INFINITY;
  let port = 5183;
  while (args.length > 0) {
    const arg = args.shift()!;
    const value = args.shift() ?? usage();
    if (arg === "--size") [width, height] = value.split("x").map(Number) as [number, number];
    else if (arg === "--scale") scale = value;
    else if (arg === "--from") from = Number(value);
    else if (arg === "--frames") limit = Number(value);
    else if (arg === "--port") port = Number(value);
    else usage();
  }
  mkdirSync(out, { recursive: true });

  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const vite = await createServer({ root, logLevel: "error", server: { port, strictPort: true, hmr: false } });
  await vite.listen();
  // Software drawing alone: a GPU process on the desktop's own GPU has frozen the machine before.
  const browser = await chromium.launch({ executablePath: chromiumPath(), args: ["--disable-gpu"] });
  try {
    const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1, reducedMotion: "no-preference", colorScheme: "dark" });
    await context.addInitScript(installClock, 1000 / FPS);
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    const cdp = await context.newCDPSession(page);

    await page.goto(`http://localhost:${port}/index.html?mode=dark&scale=${scale}&text=short&reduced=off&speed=1&flicker=on#/shell/wallpaper`);
    // The site's modules load on real time, then draw the page on the page's.
    await pump(page, () => page.evaluate(() => "kubaos" in window), 600);
    let ready = false;
    void page.evaluate(() => window.kubaos.ready).then(() => (ready = true), (error: unknown) => errors.push(String(error)));
    await pump(page, async () => ready || errors.length > 0, 600);
    await page.evaluate(() => {
      const control = document.querySelector<HTMLButtonElement>(".page-full-screen:not([hidden])");
      if (!control) throw new Error("the stage has no Full screen control");
      control.click();
    });
    const shown = () => page.evaluate(() => document.activeElement?.matches(".site-full-screen .gl-screen") ?? false);
    await pump(page, shown, 60);
    await pump(page, () => page.evaluate(() => window.tourClock.quiet()), 600);

    const beats: string[] = [];
    const trace: string[] = ["frame\tms\tleft\twidth\tout\tlit"];
    let index = 0;
    const capture = async (): Promise<boolean> => {
      if (index >= limit) return false;
      await page.evaluate(() => window.tourClock.step());
      if (index < from) {
        index++;
        return true;
      }
      const row = await page.evaluate(() => {
        const screen = document.querySelector<HTMLElement>(".site-full-screen .gl-screen");
        const front = screen?.querySelector(".gl-art[data-on] .gl-frame")?.getBoundingClientRect();
        const style = screen ? getComputedStyle(screen) : undefined;
        return [performance.now().toFixed(3), front?.left.toFixed(2), front?.width.toFixed(2), style?.getPropertyValue("--out"), style?.getPropertyValue("--room-lit")].join("\t");
      });
      trace.push(`${index}\t${row}`);
      await shoot(cdp, path.join(out, `${String(index).padStart(5, "0")}.png`));
      index++;
      if (index % 60 === 0) process.stdout.write(`${index} frames\n`);
      return true;
    };
    for (const beat of TOUR) {
      beats.push(`${index}\t${(index / FPS).toFixed(2)}s\t${beat.name}`);
      if (beat.key) {
        await page.keyboard.press(beat.key);
        // The move plays out, then the rest: each move starts from a picker at rest.
        for (let i = 0; !(await page.evaluate(() => window.tourClock.quiet())); i++) {
          if (i === LONGEST * FPS) throw new Error(`${beat.name} still moves after ${LONGEST} s`);
          if (!(await capture())) break;
        }
      }
      for (let i = 0; i < Math.round(beat.rest * FPS); i++) if (!(await capture())) break;
      if (errors.length > 0) throw new Error(`the page failed: ${errors.join("; ")}`);
    }
    writeFileSync(path.join(out, "beats.txt"), `${beats.join("\n")}\n`);
    writeFileSync(path.join(out, `trace-${from}.tsv`), `${trace.join("\n")}\n`);
    process.stdout.write(`${index} frames, ${(index / FPS).toFixed(2)} s\n${beats.join("\n")}\n`);
  } finally {
    await browser.close();
    await vite.close();
  }
}

main().catch((error: unknown) => {
  process.stderr.write(`tour: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
