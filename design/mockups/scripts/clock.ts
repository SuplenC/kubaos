// A clock of the page's own, for the scripts that capture a page's motion frame by frame
// (`scripts/tour.ts`, `scripts/shot.ts`). Installed before any of the page's scripts, it stands in
// for `performance.now`, `Date`, timers, animation frames and idle callbacks, and holds every
// animation paused, setting each to where it is at the page's time on every frame, so a frame is
// exactly its step after the last whatever it takes to draw. Chromium's virtual time
// (`Emulation.setVirtualTimePolicy`) moves timers and animation frames but not the document
// timeline that transitions and the Web Animations run on, which keeps real time.

/** The page's clock, as a script drives it. */
export interface PageClock {
  /** Moves the page's time on one frame and runs what falls due in it. */
  step(): Promise<void>;
  /** Whether nothing moves: no animation with an end still running, and no timer due soon. */
  quiet(): boolean;
}

declare global {
  interface Window {
    pageClock: PageClock;
  }
}

/**
 * Puts the page on a clock of its own that moves only on `step`, `frame` milliseconds a time. It
 * runs in the page before its scripts, so it takes nothing from this module's scope.
 */
export function installClock(frame: number): void {
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
    if (typeof handler !== "function") throw new Error("the page's clock runs no string timers");
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

  window.pageClock = {
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
