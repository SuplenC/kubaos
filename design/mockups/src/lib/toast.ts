// The transient message: `kubaos-ui`'s `toast::toast`, `KubaOS.Components`' `Toast`
// (`docs/design/components/toast.md`). One `body` line saying what just happened, in the inverse
// colours (`on_surface` behind `surface`) so it reads over anything, an optional symbol
// (`body`'s 20) `space.s12` before it and the keys that undo it as key hints `space.s16` after it,
// `space.s12` inside at the control radius.
//
// Its room is the surface it confirms in: it shows at the room's bottom, centred on a whole device
// pixel, `space.s24` above the edge. It arrives over `short`, rising `space.s32` into place a whole
// device pixel at a time as it fades in (`emphasized_decel`), stays `duration.toast`, and leaves over
// `medium`, fading (`standard_accel`). One shows at a time: a new one takes the place of the one
// showing, which goes over `micro`. The pointer on it holds it; it never takes the keyboard.

import { icon } from "./icon.ts";
import { keyHints } from "./keycap.ts";
import { row } from "./layout.ts";
import { probe } from "./probe.ts";
import { raw } from "./tokens.ts";
import { wrapped } from "./text.ts";
import { settings } from "../site/settings.ts";

export interface ToastOptions {
  readonly words: string;
  readonly icon?: string;
  /** The keys that undo what it says, and what they do: `["Ctrl+Z", "Undo"]`. */
  readonly undo?: readonly [combination: string, does: string];
}

export function toast(options: ToastOptions): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-toast";
  box.setAttribute("role", "status");
  const words = wrapped(options.words, "body", "surface");
  words.classList.add("k-toast-words");
  const said = row("s12", "centre", ...(options.icon ? [icon(options.icon, "var(--type-body-icon)", "var(--color-surface)")] : []), words);
  said.classList.add("k-toast-said");
  const parts: Element[] = [said];
  if (options.undo) parts.push(keyHints([options.undo], { color: "surface" }));
  box.append(row("s16", "centre", ...parts));
  return probe(box, { role: "component", name: "Toast", variant: "", fixed: { width: false, height: false } });
}

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** Centres `shown` across its room on a whole device pixel, the odd one after it. */
function centre(room: HTMLElement, shown: HTMLElement): void {
  shown.style.left = `${Math.floor((room.clientWidth - shown.offsetWidth) / 2)}px`;
}

/**
 * The bottom of a surface where its toasts show, and `say`, which shows one there: it takes the place
 * of the one showing, stays `duration.toast` (held while the pointer is on it), then leaves.
 */
export function toastRoom(): { readonly room: HTMLElement; readonly say: (options: ToastOptions) => void } {
  const room = document.createElement("span");
  room.className = "k-toast-room";
  let current: HTMLElement | null = null;
  let timer = 0;

  /** Sends `shown` away over `leave`, then takes it out once it has faded. */
  function dismiss(shown: HTMLElement, leave: "micro" | "medium"): void {
    shown.dataset.leaving = leave;
    if (current === shown) current = null;
    shown.addEventListener("transitionend", (event) => {
      if (event.propertyName === "opacity") shown.remove();
    });
  }

  function stay(shown: HTMLElement): void {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => dismiss(shown, "medium"), ms("toast"));
  }

  function say(options: ToastOptions): void {
    if (current) dismiss(current, "micro");
    const shown = toast(options);
    room.append(shown);
    centre(room, shown);
    current = shown;
    shown.addEventListener("pointerenter", () => window.clearTimeout(timer));
    shown.addEventListener("pointerleave", () => stay(shown));
    stay(shown);
  }

  return { room, say };
}

/**
 * `shown`, toasts drawn still at the bottom of a surface `width` by `height` (lengths' CSS), each
 * centred across it on a whole device pixel: a sample of a toast where it shows.
 */
export function onSurface(width: string, height: string, ...shown: HTMLElement[]): HTMLElement {
  const surface = document.createElement("span");
  surface.className = "k-toast-surface";
  surface.style.setProperty("--room", width);
  surface.style.setProperty("--tall", height);
  for (const each of shown) each.classList.add("k-toast-still");
  surface.append(...shown);
  new ResizeObserver(() => {
    for (const each of shown) centre(surface, each);
  }).observe(surface);
  return surface;
}
