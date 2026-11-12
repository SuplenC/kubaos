// Picking the lock clock's colour, in the wallpaper picker's clock editor: Auto (the picture's own
// tint under the digits), one of the wallpaper scheme's roles as a swatch, or Custom, a colour of
// its own picked by its hue and its tone. The library has no colour picker: this one is the
// page's own, drawn from library parts where it can be (the Auto chip, the Tone slider), with its
// own swatches and hue track where the library has none.

import { chip } from "../../lib/chip.ts";
import { focusRing } from "../../lib/focus.ts";
import { slider } from "../../lib/slider.ts";
import { text } from "../../lib/text.ts";

/** A role the clock can take: they follow the wallpaper's scheme. */
export type ClockRole = "primary" | "secondary" | "cold" | "on_surface";

export const ROLES: readonly (readonly [ClockRole, string])[] = [
  ["primary", "Primary"],
  ["secondary", "Secondary"],
  ["cold", "Cold"],
  ["on_surface", "Text"],
];

export type ClockColour = { readonly mode: "auto" } | { readonly mode: "role"; readonly role: ClockRole } | { readonly mode: "custom"; readonly h: number; readonly l: number };

/**
 * A custom colour's chroma, the same at every hue and tone: strong enough to read as a colour, low
 * enough that every hue stays inside sRGB at the light tones a clock takes.
 */
export const CUSTOM_CHROMA = 0.12;
/** A custom colour's first pick, before one is made: a light blue, a clock's usual tone. */
export const CUSTOM_FIRST = { h: 250, l: 0.85 } as const;
/**
 * The tone and chroma the hue track and the Custom swatch draw every hue at: one light tone, so
 * the hues read as the same weight round the wheel.
 */
const HUE_TRACK = { tone: 0.75, chroma: 0.13 } as const;

/** Sets the hue track's tone and chroma on `element`, for its CSS to draw the wheel with. */
function wheel(element: HTMLElement): void {
  element.style.setProperty("--ce-hue-tone", String(HUE_TRACK.tone));
  element.style.setProperty("--ce-hue-chroma", String(HUE_TRACK.chroma));
}

/** The tones a custom colour can take, OKLCH lightness: from a deep one to nearly white. */
export const TONES = { least: 0.3, most: 0.97 } as const;

/** A custom colour as CSS. */
export function customColour(h: number, l: number): string {
  return `oklch(${l.toFixed(3)} ${CUSTOM_CHROMA} ${h.toFixed(1)})`;
}

// The locale's numbers: a degree and a percentage are never put together by hand.
const DEGREES = new Intl.NumberFormat("en-GB", { style: "unit", unit: "degree", unitDisplay: "narrow", maximumFractionDigits: 0 });
const PERCENT = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 0 });

/**
 * The hue track: the hues at one light tone round the wheel, a ring marking the hue picked, its
 * value in `label_number` at the far end in `surface` so it reads on every hue. The pointer and the
 * keys move it as they move a Slider: ← and → (h and l) a twentieth, with Shift a degree.
 */
function hueTrack(value: number, disabled: boolean, changed: (hue: number) => void): HTMLElement {
  const root = document.createElement("div");
  root.className = "ce-hue";
  wheel(root);
  root.setAttribute("role", "slider");
  root.setAttribute("aria-label", "Hue");
  root.setAttribute("aria-valuemin", "0");
  root.setAttribute("aria-valuemax", "359");
  root.tabIndex = disabled ? -1 : 0;
  if (disabled) root.setAttribute("aria-disabled", "true");
  const knob = document.createElement("span");
  knob.className = "ce-hue-knob";
  let words = text("", "label_number", "surface");
  root.append(knob, words);
  focusRing(root, "full", "accent", false);
  let hue = value;
  const draw = () => {
    root.style.setProperty("--hue-at", String(hue / 359));
    root.setAttribute("aria-valuenow", String(Math.round(hue)));
    // A text's box is its words' width, so new words are a new text.
    const said = text(DEGREES.format(Math.round(hue)), "label_number");
    // Its colour is the CSS's: dark on the light hues in either mode.
    said.style.removeProperty("color");
    said.classList.add("ce-hue-value");
    words.replaceWith(said);
    words = said;
  };
  const set = (to: number) => {
    hue = Math.max(0, Math.min(359, Math.round(to)));
    draw();
    changed(hue);
  };
  const at = (event: PointerEvent) => {
    const box = root.getBoundingClientRect();
    const side = box.height;
    return ((event.clientX - box.left - side / 2) / Math.max(1, box.width - side)) * 359;
  };
  if (!disabled) {
    root.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      event.stopPropagation();
      root.setPointerCapture(event.pointerId);
      root.dataset.held = "";
      set(at(event));
    });
    root.addEventListener("pointermove", (event) => root.hasAttribute("data-held") && set(at(event)));
    const release = () => delete root.dataset.held;
    root.addEventListener("pointerup", release);
    root.addEventListener("pointercancel", release);
    root.addEventListener("keydown", (event) => {
      const by = event.key === "ArrowLeft" || event.key === "h" ? -1 : event.key === "ArrowRight" || event.key === "l" ? 1 : 0;
      if (by !== 0) set(hue + by * (event.shiftKey ? 1 : 359 / 20));
      else if (event.key === "Home" || event.key === "End") set(event.key === "Home" ? 0 : 359);
      else return;
      event.preventDefault();
    });
  }
  draw();
  return root;
}

/** A swatch: a colour as a round button, a ring of `primary` round it while it is the one picked. */
function swatch(colour: string, name: string, picked: boolean, custom: boolean): HTMLButtonElement {
  const made = document.createElement("button");
  made.type = "button";
  made.className = "ce-swatch";
  made.toggleAttribute("data-custom", custom);
  made.style.setProperty("--swatch", colour);
  made.setAttribute("aria-pressed", String(picked));
  made.setAttribute("aria-label", name);
  made.title = name;
  made.tabIndex = -1;
  return made;
}

export interface ColourPicker {
  /** The colour row's control: Auto and the swatches, one stop for Tab, ← and → picking. */
  readonly choice: HTMLElement;
  readonly hue: HTMLElement;
  readonly tone: HTMLElement;
}

/**
 * The picker for `colour`, its roles' swatches in `roles`' colours (the lock's night scheme, which
 * the clock is drawn in whatever the mode): `pick` hears a new choice (Auto, a role, Custom) and `tuned` a custom
 * colour's hue or tone as it moves. Hue and Tone are live only while Custom is picked.
 */
export function colourPicker(colour: ClockColour, roles: (role: ClockRole) => string, pick: (colour: ClockColour) => void, tuned: (h: number, l: number) => void, disabled = false): ColourPicker {
  const own = colour.mode === "custom" ? colour : CUSTOM_FIRST;
  let h: number = own.h;
  let l: number = own.l;
  const choice = document.createElement("span");
  choice.className = "ce-swatches";
  wheel(choice);
  choice.setAttribute("role", "radiogroup");
  choice.setAttribute("aria-label", "Colour");
  choice.tabIndex = disabled ? -1 : 0;
  if (disabled) choice.setAttribute("aria-disabled", "true");
  focusRing(choice, "full", "accent", false);
  const auto = chip({ label: "Auto", kind: colour.mode === "auto" ? "Selected" : "Neutral", press: "press", tabStop: false, disabled });
  auto.setAttribute("aria-pressed", String(colour.mode === "auto"));
  const picks: { readonly element: HTMLElement; readonly colour: () => ClockColour }[] = [{ element: auto, colour: () => ({ mode: "auto" }) }];
  for (const [role, name] of ROLES) picks.push({ element: swatch(roles(role), name, colour.mode === "role" && colour.role === role, false), colour: () => ({ mode: "role", role }) });
  picks.push({ element: swatch(colour.mode === "custom" ? customColour(h, l) : "", "Custom", colour.mode === "custom", true), colour: () => ({ mode: "custom", h, l }) });
  choice.append(...picks.map((each) => each.element));
  for (const each of picks) {
    if (disabled) break;
    each.element.addEventListener("click", (event) => {
      event.stopPropagation();
      pick(each.colour());
    });
  }
  choice.addEventListener("keydown", (event) => {
    if (disabled) return;
    const by = event.key === "ArrowLeft" || event.key === "h" ? -1 : event.key === "ArrowRight" || event.key === "l" ? 1 : 0;
    if (by === 0) return;
    event.preventDefault();
    const now = picks.findIndex((each) => each.element.getAttribute("aria-pressed") === "true");
    const next = picks[Math.max(0, Math.min(picks.length - 1, now + by))];
    if (next && next !== picks[now]) pick(next.colour());
  });
  const live = colour.mode === "custom" && !disabled;
  const hue = hueTrack(h, !live, (to) => {
    h = to;
    tuned(h, l);
  });
  const tone = slider({
    icon: "brightness_medium",
    size: "Compact",
    value: (l - TONES.least) / (TONES.most - TONES.least),
    number: (v) => PERCENT.format(TONES.least + v * (TONES.most - TONES.least)),
    fine: 0.01 / (TONES.most - TONES.least),
    disabled: !live,
    changed: (v) => {
      l = TONES.least + v * (TONES.most - TONES.least);
      tuned(h, l);
    },
  });
  return { choice, hue, tone };
}
