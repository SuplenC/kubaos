// A group in the bar: `KubaOS.Components`' `Pill` (`docs/design/components/pill.md`), the shell's
// alone. A pill `size.control_sm` tall at the full radius on `surface_container`, its content a row
// `space.s8` apart centred on device pixels, `space.s12` at each end between the curve and the
// content (`space.s8` round a lone symbol). A pill of its own tone takes a fill and an ink together.
//
// An interactive pill takes the pointer and is a control in the state model (`.k-control` in
// `lib.css`), lit only by the pointer: it is never a stop for Tab, so a panel's keyboard focus never
// wanders into the bar. Its name shows in a hint under it while it is lit. When its content changes
// width it eases to the new width over `medium`, cutting its content at its own edge meanwhile.

import { pin, type Pinned } from "./control.ts";
import { focusRing, type Tone } from "./focus.ts";
import { withHint } from "./hint.ts";
import { row } from "./layout.ts";
import { probe } from "./probe.ts";
import { cubic } from "./tokens.ts";

/** A pill's colours: the bar's own, or a tone's fill and ink together. */
export type PillTone = "neutral" | "recording" | "preview";

const FILLS: Record<PillTone, { readonly fill: string; readonly ink: string; readonly tone: Tone }> = {
  neutral: { fill: "var(--color-surface_container)", ink: "var(--color-on_surface)", tone: "accent" },
  recording: { fill: "var(--color-error_container)", ink: "var(--color-on_error_container)", tone: "error" },
  preview: { fill: "rgb(from var(--color-cold) r g b / var(--opacity-tint))", ink: "var(--color-cold)", tone: "cold" },
};

export interface PillOptions {
  readonly content: readonly Element[];
  /** A lone symbol: `space.s8` at each end instead of `space.s12`. */
  readonly lone: boolean;
  readonly tone?: PillTone;
  /** Takes the pointer: lit under it, pressed, and its name in a hint. */
  readonly interactive?: boolean;
  /** Its name, in a hint under it while it is lit. */
  readonly tooltip?: string;
  readonly pinned?: Pinned;
}

export interface Pill {
  readonly element: HTMLElement;
  /** Puts `content` in it, easing its width from the old content's to the new's. */
  readonly change: (content: readonly Element[]) => void;
}

/** A pill as an element, for a sample that never changes its content. */
export function pill(options: PillOptions): HTMLElement {
  return makePill(options).element;
}

export function makePill(options: PillOptions): Pill {
  const tone = FILLS[options.tone ?? "neutral"];
  const element = document.createElement("span");
  element.className = "k-pill";
  element.toggleAttribute("data-lone", options.lone);
  element.style.setProperty("--fill", tone.fill);
  element.style.setProperty("--ink", tone.ink);
  const clip = document.createElement("span");
  clip.className = "k-pill-clip";
  const line = row("s8", "centre", ...options.content);
  clip.append(line);
  element.append(clip);
  if (options.interactive) {
    element.classList.add("k-control");
    element.dataset.hot = "";
    element.setAttribute("role", "button");
    pin(element, options.pinned);
    focusRing(element, "full", tone.tone, false).dataset.pressable = "";
    if (options.tooltip) withHint(element, options.tooltip);
  }

  function change(content: readonly Element[]): void {
    const from = element.offsetWidth;
    line.replaceChildren(...content);
    element.style.removeProperty("--from");
    element.style.removeProperty("--to");
    const to = element.offsetWidth;
    element.style.setProperty("--from", `${from}px`);
    element.style.setProperty("--to", `${to}px`);
    element.animate([{ "--grown": 0 }, { "--grown": 1 }], { duration: motion("medium"), easing: `cubic-bezier(${cubic("easing-standard_decel").join(", ")})` });
  }

  probe(element, { role: "component", name: "Pill", variant: "", fixed: { width: false, height: true } });
  return { element, change };
}

/** A duration token in milliseconds, as the toolbar's speed and reduce motion scale it. */
function motion(name: string): number {
  const probe = document.createElement("span");
  probe.style.transitionDuration = `var(--motion-${name})`;
  document.body.append(probe);
  const ms = Number.parseFloat(getComputedStyle(probe).transitionDuration) * 1000;
  probe.remove();
  return ms;
}
