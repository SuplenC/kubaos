// Playing a page's moves on a live sample: the same tracks its players play, each part eased from
// wherever it stands now, so what the recipe says and what you try never disagree. Shared by the
// Power menu's room and the Boot page's screens.

import type { Easing, Track } from "../../site/page.ts";
import { between, bezier, css, cueOf, milliseconds, stepsOf, struck } from "../../site/recipe.ts";
import { settings } from "../../site/settings.ts";

/** `root` and every element under it `selector` matches. */
export function targets(root: HTMLElement, selector: string): HTMLElement[] {
  return [...(root.matches(selector) ? [root] : []), ...root.querySelectorAll<HTMLElement>(selector)];
}

/** `tracks` at their ends, at once: a still of where a move leaves the sample. */
export function settle(root: HTMLElement, tracks: readonly Track[]): void {
  const ordered = [...tracks].sort((a, b) => cueOf(a) + milliseconds(a.duration) - (cueOf(b) + milliseconds(b.duration)));
  for (const each of ordered) for (const element of targets(root, each.target)) element.style.setProperty(each.property, String(css(each.to)));
}

/** `tracks` at their starts, at once, each part's first track winning: where a move begins. */
export function unsettle(root: HTMLElement, tracks: readonly Track[]): void {
  const ordered = [...tracks].sort((a, b) => cueOf(b) - cueOf(a));
  for (const each of ordered) for (const element of targets(root, each.target)) element.style.setProperty(each.property, String(css(each.from)));
}

const ms = (time: number) => time / settings().speed;
const curveOf = (easing: Easing) => `cubic-bezier(${bezier(easing).join(", ")})`;

/** Each element's running animations, by property: a new one takes over from where the old one stands. */
const running = new WeakMap<Element, Map<string, Animation>>();

/** The value `property` stands at on `element` now, mid-animation or not. */
function now(element: HTMLElement, property: string): string {
  const style = getComputedStyle(element);
  const value = property === "opacity" ? style.opacity : style.getPropertyValue(property).trim();
  return value === "" || value === "none" ? (property === "scale" ? "1" : "0") : value;
}

/** Eases `property` on `element` from where it stands to `to`: set at once, the animation the way there. */
function tween(element: HTMLElement, each: Track, delay: number): Promise<void> {
  const map = running.get(element) ?? new Map<string, Animation>();
  running.set(element, map);
  const key = each.property;
  const from = now(element, key);
  const to = String(css(each.to));
  map.get(key)?.cancel();
  element.style.setProperty(key, to);
  const length = Math.max(1, ms(milliseconds(each.duration)));
  if (struck(each.curve)) {
    // A strike's first step is already part lit, so it can't stand in for the wait before its cue
    // as a backwards fill would make it: the wait is the animation's own first stretch, held at
    // where the tube stood.
    const total = delay + length;
    const frames: Keyframe[] = [
      { offset: 0, easing: "step-end", [key]: from },
      ...stepsOf(each.curve).map((step, i, all) => ({ offset: (delay + (i * length) / all.length) / total, easing: "step-end", [key]: String(between(each, step)) })),
      { offset: 1, [key]: to },
    ];
    const animation = element.animate(frames, { duration: total, easing: "linear" });
    map.set(key, animation);
    return animation.finished.then(
      () => undefined,
      () => undefined,
    );
  }
  const frames: Keyframe[] = [{ [key]: from }, { [key]: to }];
  const animation = element.animate(frames, { duration: length, delay, easing: curveOf(each.curve as Easing), fill: "backwards" });
  map.set(key, animation);
  return animation.finished.then(
    () => undefined,
    () => undefined,
  );
}

/** Stops every animation `play` started on `root`'s parts, each left where it was headed. */
export function halt(root: HTMLElement): void {
  for (const animation of root.getAnimations({ subtree: true })) if (!(animation instanceof CSSTransition)) animation.cancel();
}

/**
 * Plays `tracks` on `root` from wherever each part stands, each to its `to` on its curve after its
 * cue. Tracks on one part and property play in cue order, each from where the one before left it.
 * A stepped track flickers through `neon.strike`'s steps. A control's press is its state model's
 * to draw, so it isn't played. Resolves once every track is done.
 */
export function play(root: HTMLElement, tracks: readonly Track[]): Promise<void> {
  const chains = new Map<HTMLElement, Map<string, Track[]>>();
  for (const each of [...tracks].sort((a, b) => cueOf(a) - cueOf(b))) {
    if (each.property === "--press") continue;
    for (const element of targets(root, each.target)) {
      const byProperty = chains.get(element) ?? new Map<string, Track[]>();
      chains.set(element, byProperty);
      byProperty.set(each.property, [...(byProperty.get(each.property) ?? []), each]);
    }
  }
  const started = performance.now();
  const runs: Promise<void>[] = [];
  for (const [element, byProperty] of chains) {
    for (const list of byProperty.values()) {
      runs.push(
        (async () => {
          for (const each of list) {
            const at = ms(cueOf(each)) - (performance.now() - started);
            await tween(element, each, Math.max(0, at));
          }
        })(),
      );
    }
  }
  return Promise.all(runs).then(() => undefined);
}
