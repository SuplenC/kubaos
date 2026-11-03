// A titled part that opens to show more: `kubaos-ui`'s `collapsible::collapsible`,
// `KubaOS.Components`' `Collapsible` (`docs/design/components/collapsible.md`). The header is a
// control as tall as a row (`size.row`) at the control radius, `space.s12` inside: a chevron at its
// start, pointing right while it is shut and down while it is open, then its title in
// `body_strong`, and at its end an optional summary in `label`, there open or shut. Open, what it
// holds sits `space.s8` under the header, in line with the title, and an unlit rail of neon runs
// down its start side under the chevron's centre.
//
// The header is a control in the state model (`.k-control` in `lib.css`) with nothing filled under
// its line, so lit it draws the tube and pressed the tube runs white-hot. The room under it eases
// from nothing to what it holds over `medium` on `standard_decel` and back over `short` on
// `standard_accel`, the chevron turning and the rail growing with it; with reduce motion it opens
// at once, what it holds fades in and the chevron jumps.
//
// The rail is a control too, for the pointer alone: under it the rail lights, a hint beside it
// follows the pointer, and a press closes the part, scrolling the header back into view where it
// has scrolled out. It is no stop for Tab; the keyboard has the header.

import { type Pinned, pin } from "./control.ts";
import { focusRing } from "./focus.ts";
import { withHint } from "./hint.ts";
import { icon } from "./icon.ts";
import { filling, row } from "./layout.ts";
import { probe, variant } from "./probe.ts";
import { clamped } from "./text.ts";

export interface CollapsibleOptions {
  readonly title: string;
  /** What it shows open. */
  readonly content: Element;
  /** A line at the header's end saying what it holds, open or shut: "12 · 4.1 GB". */
  readonly summary?: string;
  readonly open?: boolean;
  /** No toggle: the header dims and takes no press and no stop for Tab, and the rail none either. */
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
  /** The rail pinned lit or pressed, its hint shown: a sample of it under the pointer. */
  readonly railPinned?: Pinned;
}

let made = 0;

export function collapsible(options: CollapsibleOptions): HTMLElement {
  const id = `k-collapsible-${made++}`;
  let open = options.open ?? false;
  const disabled = options.disabled ?? false;

  const part = document.createElement("span");
  part.className = "k-collapsible";

  const header = document.createElement("button");
  header.className = "k-collapsible-header k-control";
  header.type = "button";
  header.dataset.hot = "";
  header.disabled = disabled;
  header.setAttribute("aria-controls", id);
  pin(header, options.pinned);
  const chevron = icon("expand_more", "var(--type-body_strong-icon)", "var(--mark)");
  chevron.classList.add("k-collapsible-chevron");
  const title = clamped(options.title, "body_strong");
  title.style.color = "var(--title)";
  const line = row("s12", "centre", chevron, filling(title));
  if (options.summary !== undefined) {
    const summary = clamped(options.summary, "label");
    summary.classList.add("k-collapsible-summary");
    summary.style.color = "var(--mark)";
    line.append(summary);
  }
  header.append(line);
  focusRing(header, "control", "accent", false).dataset.pressable = "";

  const body = document.createElement("span");
  body.className = "k-collapsible-body";
  body.id = id;
  body.setAttribute("role", "region");
  const content = document.createElement("span");
  content.className = "k-collapsible-content";
  content.append(options.content, rail());
  body.append(content);
  part.append(header, body);

  /** The rail down what it holds: its line, and a strip round it that takes the pointer. */
  function rail(): HTMLElement {
    const strip = document.createElement("span");
    strip.className = "k-collapsible-rail k-control";
    strip.setAttribute("aria-hidden", "true");
    if (disabled) strip.setAttribute("aria-disabled", "true");
    pin(strip, options.railPinned);
    const drawn = document.createElement("span");
    drawn.className = "k-collapsible-line";
    strip.append(drawn);
    withHint(strip, `Close ${options.title}`);
    // The hint follows the pointer down the rail. The page's box is zoomed to the display's ratio,
    // so the pointer's height in the viewport's pixels is brought back to the strip's own by its
    // height over its layout height.
    strip.addEventListener("pointermove", (event) => {
      const box = strip.getBoundingClientRect();
      const ratio = strip.offsetHeight / Math.max(1, box.height);
      strip.style.setProperty("--at", `${(event.clientY - box.top) * ratio}px`);
    });
    strip.addEventListener("click", () => {
      if (disabled || !open) return;
      toggle();
      bringBack();
    });
    return strip;
  }

  /**
   * Scrolls the header back into its scroll view where closing from the rail left it out of sight:
   * that view alone, never the page round it, as `scrollIntoView` would.
   */
  function bringBack(): void {
    let view = part.parentElement;
    while (view && !/auto|scroll/.test(getComputedStyle(view).overflowY)) view = view.parentElement;
    if (!view) return;
    const box = view.getBoundingClientRect();
    const at = header.getBoundingClientRect();
    // The view's pixels are the page's, which its zoom shrinks in the viewport's.
    const zoom = view.clientHeight / box.height || 1;
    const by = at.top < box.top ? at.top - box.top : at.bottom > box.bottom ? at.bottom - box.bottom : 0;
    if (by === 0) return;
    const reduced = part.closest("[data-reduced]") !== null;
    view.scrollBy({ top: by * zoom, behavior: reduced ? "instant" : "smooth" });
  }

  // The room eases between nothing and what it holds, which only layout knows: its height in the
  // part's own CSS pixels, kept as it changes.
  new ResizeObserver(() => part.style.setProperty("--content-height", `${content.offsetHeight}px`)).observe(content);

  /** Shows `open`: the header's state, the room's, and whether what it holds takes the keyboard. */
  function show(): void {
    part.toggleAttribute("data-open", open);
    header.setAttribute("aria-expanded", String(open));
    body.inert = !open;
    probe(part, { role: "component", name: "Collapsible", variant: variant(open && "open"), fixed: { width: false, height: false, holds: true } });
  }

  /** Lets what it holds reach past the room once it is fully open, so a lit control's glow isn't cut. */
  function settle(): void {
    part.toggleAttribute("data-settled", open);
  }

  function toggle(): void {
    open = !open;
    part.removeAttribute("data-settled");
    show();
    // With nothing to play (reduced motion) the room is where it ends at once.
    if (getComputedStyle(part).transitionDuration.split(",").every((each) => Number.parseFloat(each) === 0)) settle();
  }

  part.addEventListener("transitionend", (event) => {
    if (event.target === part && event.propertyName === "--open") settle();
  });
  header.addEventListener("click", toggle);
  // Enter and Space press it once per press, never on key repeat.
  header.addEventListener("keydown", (event) => {
    if ((event.key === "Enter" || event.key === " ") && event.repeat) event.preventDefault();
  });

  show();
  settle();
  return part;
}
