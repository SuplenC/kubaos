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

function onWheel(event: WheelEvent): void {
  if (event.ctrlKey || !(event.target instanceof Element)) return;
  if (engaged?.contains(event.target)) return;
  // The outermost sample the wheel is over: a state's cell on a stage is inside the stage's box.
  let sample = event.target.closest<HTMLElement>("[data-sample]");
  if (!sample) return;
  for (let outer = sample.parentElement?.closest<HTMLElement>("[data-sample]"); outer; outer = outer.parentElement?.closest<HTMLElement>("[data-sample]")) sample = outer;
  // A sample's lists that scroll themselves in script never hear a wheel they weren't given.
  event.stopPropagation();
  const { dx, dy } = pixels(event);
  // A sample's own horizontal scroll still takes a sideways wheel: the page has none.
  if (Math.abs(dy) < Math.abs(dx)) return;
  for (let at: Element | null = event.target; at && at !== sample.parentElement; at = at.parentElement) {
    if (!scrollsDown(at)) continue;
    event.preventDefault();
    document.scrollingElement?.scrollBy({ top: dy, behavior: "instant" });
    return;
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
  addEventListener("wheel", onWheel, { capture: true, passive: false });
  addEventListener("pointerdown", onPointerDown, { capture: true });
  addEventListener("keydown", onKeyDown, { capture: true });
}

/** Lets go of the sample clicked into: the page is drawn again, and it is gone. */
export function releaseWheel(): void {
  engage(null);
}
