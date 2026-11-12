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
