// Who a wheel scrolls. Over a live sample it scrolls the page, as over any other part of it,
// until the reader clicks into the sample: from then on the sample's own lists and scroll views
// take the wheel, as they do on a desktop, until Esc or a click outside it lets it go. A page of
// samples that scroll would otherwise catch the wheel at every one the pointer passes over.
//
// A wheel over a sample with no scroll view under the pointer goes to the page as the browser
// sends it, smoothly; one over a scroll view, which would take it or keep it, is turned to the
// page by hand.

/** The sample clicked into, which keeps the wheel; none at first. */
let engaged: HTMLElement | null = null;

function engage(sample: HTMLElement | null): void {
  if (sample === engaged) return;
  engaged?.removeAttribute("data-engaged");
  engaged = sample;
  engaged?.setAttribute("data-engaged", "");
}

/**
 * Whether `element` is a scroll view down its height: one that the wheel would scroll, or that
 * would keep it at its end (a list's `overscroll-behavior: contain`).
 */
function scrollsDown(element: Element): boolean {
  return /auto|scroll/.test(getComputedStyle(element).overflowY) && element.scrollHeight > element.clientHeight + 1;
}

/** The wheel's travel in CSS pixels, whatever unit the event counts it in. */
function pixels(event: WheelEvent): { dx: number; dy: number } {
  const unit = event.deltaMode === WheelEvent.DOM_DELTA_LINE ? Number.parseFloat(getComputedStyle(document.documentElement).lineHeight) || 1 : event.deltaMode === WheelEvent.DOM_DELTA_PAGE ? innerHeight : 1;
  return { dx: event.deltaX * unit, dy: event.deltaY * unit };
}

/** The outermost sample `target` is in: a state's cell on a stage is inside the stage's box. */
function outermostSample(target: Element): HTMLElement | null {
  let sample = target.closest<HTMLElement>("[data-sample]");
  for (let outer = sample?.parentElement?.closest<HTMLElement>("[data-sample]"); outer; outer = outer.parentElement?.closest<HTMLElement>("[data-sample]")) sample = outer;
  return sample;
}

/** Whether a scroll view between `target` and `sample`, itself included, would take the wheel. */
function overScrollView(target: Element, sample: HTMLElement): boolean {
  for (let at: Element | null = target; at && at !== sample.parentElement; at = at.parentElement) if (scrollsDown(at)) return true;
  return false;
}

/** The sample a wheel at `event` is the page's over, or null where it is another's. */
function pageWheelOver(event: WheelEvent): HTMLElement | null {
  if (event.ctrlKey || !(event.target instanceof Element)) return null;
  if (engaged?.contains(event.target)) return null;
  return outermostSample(event.target);
}

/**
 * Every wheel, passive, so the page scrolls on the compositor: over a sample it is kept from the
 * sample's own script, unless a scroll view under it would take it, which `onScrollViewWheel`
 * turns to the page.
 */
function onWheel(event: WheelEvent): void {
  const sample = pageWheelOver(event);
  if (!sample) return;
  const { dx, dy } = pixels(event);
  // A sample's own horizontal scroll still takes a sideways wheel: the page has none.
  if (Math.abs(dy) >= Math.abs(dx) && overScrollView(event.target as Element, sample)) return;
  // A sample's lists that scroll themselves in script never hear a wheel they weren't given.
  event.stopPropagation();
}

/**
 * A wheel reaching a scroll view inside a sample: turned to the page by hand. Only a scroll view
 * listens so, as a wheel a listener may cancel scrolls on the page's thread, not the compositor's.
 */
function onScrollViewWheel(event: WheelEvent): void {
  if (event.defaultPrevented) return;
  const sample = pageWheelOver(event);
  if (!sample) return;
  const { dx, dy } = pixels(event);
  event.stopPropagation();
  if (Math.abs(dy) < Math.abs(dx) || !overScrollView(event.target as Element, sample)) return;
  event.preventDefault();
  document.scrollingElement?.scrollBy({ top: dy, behavior: "instant" });
}

/** The boxes inside samples already looked at for a scroll view. */
const looked = new WeakSet<Element>();

/**
 * Gives each scroll view inside a sample the pointer comes onto its cancelling listener, before the
 * wheel it may turn: the pointer is over it before any wheel is.
 */
function onPointerOver(event: PointerEvent): void {
  if (!(event.target instanceof Element)) return;
  const sample = outermostSample(event.target);
  if (!sample) return;
  for (let at: Element | null = event.target; at && at !== sample.parentElement && !looked.has(at); at = at.parentElement) {
    looked.add(at);
    if (/auto|scroll/.test(getComputedStyle(at).overflowY)) at.addEventListener("wheel", onScrollViewWheel as EventListener, { capture: true, passive: false });
  }
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target instanceof Element ? event.target : null;
  engage(target?.closest<HTMLElement>("[data-sample]") ?? null);
}

function onKeyDown(event: KeyboardEvent): void {
  if (event.key === "Escape") engage(null);
}

/** Puts the wheel under the page's rule, once for the site's life. */
export function holdWheel(): void {
  addEventListener("wheel", onWheel, { capture: true, passive: true });
  addEventListener("pointerover", onPointerOver, { capture: true, passive: true });
  addEventListener("pointerdown", onPointerDown, { capture: true });
  addEventListener("keydown", onKeyDown, { capture: true });
}

/** Lets go of the sample clicked into: the page is drawn again, and it is gone. */
export function releaseWheel(): void {
  engage(null);
}
