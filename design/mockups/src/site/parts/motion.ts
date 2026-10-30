// The motion player: each move live on its sample, replayable and slowed down, with its duration
// and curve named, their values beside them, and what reduced motion makes of it.

import { number, raw } from "../../lib/tokens.ts";
import type { Easing, Motion, Move } from "../page.ts";
import { el, label, prose } from "../dom.ts";
import { partOr, subheading } from "./section.ts";

/** How many times slower slow motion plays. */
const SLOW = 10;

function control(words: string, pressed: boolean | null, onPress: () => void): HTMLButtonElement {
  const button = el("button", "site-control", words);
  button.type = "button";
  if (pressed !== null) button.setAttribute("aria-pressed", String(pressed));
  button.addEventListener("click", onPress);
  return button;
}

/** A duration token in milliseconds, as the tokens write it; the browser hands it back in seconds. */
function milliseconds(duration: Move["duration"]): number {
  const value = raw(`duration-${duration}`);
  const amount = Number.parseFloat(value);
  return Math.round(value.endsWith("ms") ? amount : amount * 1000);
}

/** A curve token's four numbers, as the tokens write them. */
function curve(easing: Easing): string {
  return raw(`easing-${easing}`).replace(/^cubic-bezier\((.*)\)$/, "$1").replace(/(^|[ ,-])\./g, "$10.");
}

function player(move: Move): HTMLElement {
  const { element, play, rest } = move.sample();
  const stage = el("div", "page-player-stage mock", element);
  let slow = false;
  const slowControl = control(`Slow ×${SLOW}`, false, () => {
    slow = !slow;
    stage.style.setProperty("--slow", slow ? String(SLOW) : "1");
    slowControl.setAttribute("aria-pressed", String(slow));
  });
  const how = move.curve === "neon.strike" ? `neon.strike, ${number("neon-strike-steps")} even steps` : `easing.${move.curve}, ${curve(move.curve)}`;
  const tokens = `duration.${move.duration}, ${milliseconds(move.duration)} ms · ${how}`;
  return el(
    "div",
    "page-player",
    stage,
    el(
      "div",
      "page-player-words",
      label(move.name, "body_strong", "on_surface"),
      prose(move.what),
      el("code", "page-code page-tokens", tokens),
      prose(`Reduced motion: ${move.reduced}`, "small", "on_surface_variant"),
      el("div", "page-player-controls", control("Replay", null, play), control("Rest", null, rest), slowControl),
    ),
  );
}

function moves(list: readonly Move[]): HTMLElement {
  return el("div", "page-players", ...list.map(player));
}

export function motion(spec: Motion): HTMLElement {
  return el(
    "div",
    "page-part",
    subheading("Arriving"),
    partOr(spec.arrives, moves),
    subheading("Leaving"),
    partOr(spec.leaves, moves),
    subheading("Between states"),
    partOr(spec.between, moves),
  );
}
