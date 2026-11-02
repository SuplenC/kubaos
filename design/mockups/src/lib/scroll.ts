// A page's or a panel's content taller than its room: `kubaos-ui`'s `layout::scroll`,
// `KubaOS.Components`' `Scroll` and `HeaderFade` (`docs/design/components/scroll.md`).
//
// A scroll has no edge of its own, so it reaches past its box into the room around it: as far as a
// glow reaches (`focus.far`) at its sides, and above and below up to the part of the panel there,
// with its content that far in. The content lines up with what is around it, a resting control at
// its edge keeps its whole glow, content scrolled past its box shows in that room and stops at the
// part beside it, and the panel's own edge cuts whatever passes it. Content that runs to its end
// leaves `space.s16` there for the thumb: `size.scrollbar` wide on the box's end, a pill in
// `outline_variant` (`outline` under the pointer and while dragged), along the whole room it shows,
// only while the content is taller than its room.
//
// A header fade lies over the gap between a panel's header and the scroll under it: while the
// content is scrolled from its top, the gap stays clear in `surface` and the content fades into it
// over the next `space.s16`, rather than being cut flat under the header's words. It spans the
// scroll's box but the thumb's width at its end, and fades in and out over `micro`.

import "./lists.css";
import { probe } from "./probe.ts";

export interface ScrollOptions {
  /** Its height, a length's CSS. */
  readonly height: string;
  /**
   * The room above it it reaches into, up to the part there: a spacing token's name, or 0 where
   * its box's edge is the window's, which would cut a thumb reaching past it.
   */
  readonly above?: string | 0;
  readonly below?: string | 0;
  /**
   * No `space.s16` kept at the content's end for the thumb: content that keeps a padding of its
   * own at its end (a page's, a sidebar's), which the thumb floats in.
   */
  readonly noGutter?: boolean;
  /** Scrolled this share of the way down, as a sample of it scrolled. */
  readonly down?: number;
  /** The thumb pinned as under the pointer. */
  readonly thumbPinned?: boolean;
  /** A header fade over the gap above it, that gap (a spacing token's name) tall before it fades. */
  readonly fade?: string;
  /** The header fade pinned shown, as while scrolled. */
  readonly fadePinned?: boolean;
}

/** `content` scrolling in a box `options.height` tall, reaching past it. Returns the box and what scrolls. */
export function scroll(content: HTMLElement, options: ScrollOptions): { box: HTMLElement; view: HTMLElement } {
  const box = document.createElement("span");
  box.className = "k-scroll";
  box.style.setProperty("--height", options.height);
  const reach = (room: string | 0 | undefined) => (room === 0 ? "0px" : room ? `var(--space-${room})` : "var(--focus-far)");
  box.style.setProperty("--above", reach(options.above));
  box.style.setProperty("--below", reach(options.below));
  box.toggleAttribute("data-no-gutter", options.noGutter ?? false);
  const view = document.createElement("span");
  view.className = "k-scroll-view";
  const body = document.createElement("span");
  body.className = "k-scroll-body";
  body.append(content);
  view.append(body);
  const thumb = document.createElement("span");
  thumb.className = "k-scroll-thumb";
  thumb.toggleAttribute("data-pinned", options.thumbPinned ?? false);
  box.append(view, thumb);
  if (options.fade) {
    const fade = headerFade(options.fade);
    fade.toggleAttribute("data-pinned", options.fadePinned ?? false);
    box.append(fade);
  }

  function place(): void {
    const overflow = view.scrollHeight - view.clientHeight;
    thumb.toggleAttribute("data-shown", overflow > 1);
    box.toggleAttribute("data-scrolled", view.scrollTop >= 1);
    if (overflow <= 1) return;
    const tall = Math.round((view.clientHeight * view.clientHeight) / view.scrollHeight);
    thumb.style.setProperty("--top", `${Math.round(((view.clientHeight - tall) * view.scrollTop) / overflow)}px`);
    thumb.style.setProperty("--tall", `${tall}px`);
  }

  view.addEventListener("scroll", place, { passive: true });
  new ResizeObserver(place).observe(body);
  thumb.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    thumb.setPointerCapture(event.pointerId);
    thumb.dataset.dragged = "";
    const ratio = box.offsetHeight / Math.max(1, box.getBoundingClientRect().height);
    const from = event.clientY;
    const start = view.scrollTop;
    const follow = (moved: PointerEvent) => {
      const room = view.clientHeight - thumb.offsetHeight;
      view.scrollTop = start + (((moved.clientY - from) * ratio) / Math.max(1, room)) * (view.scrollHeight - view.clientHeight);
    };
    const end = () => {
      delete thumb.dataset.dragged;
      thumb.removeEventListener("pointermove", follow);
    };
    thumb.addEventListener("pointermove", follow);
    thumb.addEventListener("pointerup", end, { once: true });
    thumb.addEventListener("pointercancel", end, { once: true });
  });
  requestAnimationFrame(() => {
    if (options.down !== undefined) view.scrollTop = Math.round((view.scrollHeight - view.clientHeight) * options.down);
    place();
  });
  probe(box, { role: "component", name: "Scroll", variant: "", fixed: { width: false, height: false } });
  return { box, view };
}

/**
 * The fade over the gap between a header and the scroll under it, `gap` (a spacing token's name)
 * clear then `space.s16` fading, shown while `scrolled` is marked on the scroll's box.
 */
export function headerFade(gap = "s16"): HTMLElement {
  const fade = document.createElement("span");
  fade.className = "k-header-fade";
  fade.style.setProperty("--gap", `var(--space-${gap})`);
  return probe(fade, { role: "component", name: "HeaderFade", variant: "", fixed: { width: false, height: true } });
}
