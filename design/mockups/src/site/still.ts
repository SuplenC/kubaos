// Work the page puts off until the reader is still: no wheel, scroll, pointer or key for a moment,
// then an idle moment. Building a sample takes tens of milliseconds, longer than a frame; done
// while the page scrolls or the pointer moves it would drop frames the reader sees, done while
// they read it drops none they notice.

/** How long the reader is still before put-off work runs, in milliseconds. */
const STILL = 200;

let lastInput = 0;
for (const type of ["wheel", "scroll", "pointermove", "pointerdown", "keydown"]) {
  addEventListener(type, () => (lastInput = performance.now()), { capture: true, passive: true });
}

/** Runs `work` in an idle moment once the reader has been still for `STILL`; returns how to cancel it. */
export function whenStill(work: () => void): () => void {
  let timer = 0;
  let idle = 0;
  const check = (): void => {
    const wait = lastInput + STILL - performance.now();
    if (wait > 0) {
      timer = window.setTimeout(check, wait);
      return;
    }
    // A move playing on the page (a page arriving, a scene handing over) is the reader watching:
    // building now would drop its frames.
    if (moving()) {
      timer = window.setTimeout(check, STILL);
      return;
    }
    idle = requestIdleCallback(() => {
      idle = 0;
      // The reader moved while the page waited for an idle moment.
      if (performance.now() - lastInput < STILL) check();
      else work();
    });
  };
  check();
  return () => {
    clearTimeout(timer);
    if (idle) cancelIdleCallback(idle);
  };
}

/** Whether a move with an end is playing on the page; the players' endless loops aren't moves. */
function moving(): boolean {
  return document.getAnimations().some((animation) => animation.playState === "running" && Number.isFinite(Number(animation.effect?.getComputedTiming().endTime)));
}

/** How long after the wheel last turned the page the page counts as still: a few frames. */
const SETTLED = 120;

let wheeled = Number.NEGATIVE_INFINITY;
addEventListener("wheel", () => (wheeled = performance.now()), { capture: true, passive: true });

let afterWheel = 0;

/**
 * Runs `work` once the wheel has stopped moving the page, in place of any work asked for before
 * it, at once when it isn't moving: what a part or a row under a resting pointer lights is read
 * only once the page under it has stopped.
 */
export function whenWheelStops(work: () => void): void {
  clearTimeout(afterWheel);
  afterWheel = window.setTimeout(work, Math.max(0, SETTLED - (performance.now() - wheeled)));
}

/** Forgets the work `whenWheelStops` was asked for. */
export function forgetWheelWork(): void {
  clearTimeout(afterWheel);
}
