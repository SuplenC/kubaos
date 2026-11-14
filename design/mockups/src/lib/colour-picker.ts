// A colour picked for something drawn over a picture: Auto (whatever its user works out for
// itself), one of the scheme's roles as a swatch, or Custom, a colour of its own picked by its hue
// and its tone (`docs/design/components/colour-picker.md`). `.k-colour-*` in `lib.css` draws it.
//
// It is three controls, each the control of one row of its user's form: the choice (Auto and the
// swatches in a row, one stop for Tab, ← and → picking), the hue track and the Tone slider. Hue and
// Tone are live only while Custom is picked. Auto is a `Chip`, Tone a compact `Slider`; the swatches
// and the hue track are the picker's own.

import { chip } from "./chip.ts";
import { pin, type Pinned } from "./control.ts";
import { focusRing } from "./focus.ts";
import { probe, variant } from "./probe.ts";
import { slider } from "./slider.ts";
import { text } from "./text.ts";

/** What is picked: Auto, a role of the scheme, or a custom colour by its hue and its OKLCH tone. */
export type PickedColour<R extends string = string> = { readonly mode: "auto" } | { readonly mode: "role"; readonly role: R } | { readonly mode: "custom"; readonly h: number; readonly l: number };

/** A role offered as a swatch: its name, said and shown as its tooltip, and its colour as CSS. */
export interface Swatch<R extends string = string> {
  readonly role: R;
  readonly name: string;
  readonly colour: string;
}

export interface ColourPickerOptions<R extends string = string> {
  readonly picked: PickedColour<R>;
  /** The roles offered, in order, after Auto. */
  readonly swatches: readonly Swatch<R>[];
  /** Auto offered first: whatever the picker's user works out for itself. */
  readonly auto?: boolean;
  /** Custom offered last, with its Hue and Tone. */
  readonly custom?: boolean;
  /** What the choice reads out as. */
  readonly label?: string;
  readonly disabled?: boolean;
  /**
   * The choice pinned in a state: lit or lit with the signs off, as the keyboard on it lights it;
   * pressed, the Auto chip held under the pointer.
   */
  readonly pinned?: Pinned;
  /** Hears a new choice: Auto, a role, or Custom at its hue and tone. */
  readonly onPick?: (colour: PickedColour<R>) => void;
  /** Hears a custom colour's hue or tone as it moves. */
  readonly onTune?: (h: number, l: number) => void;
}

export interface ColourPicker {
  /** The choice: Auto and the swatches, one stop for Tab, ← and → picking. */
  readonly choice: HTMLElement;
  /** The hue track, when Custom is offered. */
  readonly hue: HTMLElement | null;
  /** The Tone slider, when Custom is offered. */
  readonly tone: HTMLElement | null;
}

/**
 * A custom colour's chroma, the same at every hue and tone: strong enough to read as a colour, low
 * enough that every hue stays inside sRGB at the light tones a colour over a picture takes.
 */
export const CUSTOM_CHROMA = 0.12;
/** A custom colour's first pick, before one is made: a light blue, at a light tone. */
export const CUSTOM_FIRST = { h: 250, l: 0.85 } as const;
/**
 * The tone and chroma the hue track and the Custom swatch draw every hue at: one light tone, so
 * the hues read as the same weight round the wheel.
 */
const HUE_TRACK = { tone: 0.75, chroma: 0.13 } as const;

/** The tones a custom colour can take, OKLCH lightness: from a deep one to nearly white. */
export const TONES = { least: 0.3, most: 0.97 } as const;

/** A custom colour as CSS. */
export function customColour(h: number, l: number): string {
  return `oklch(${l.toFixed(3)} ${CUSTOM_CHROMA} ${h.toFixed(1)})`;
}

/** Sets the hue track's tone and chroma on `element`, for its CSS to draw the wheel with. */
function wheel(element: HTMLElement): void {
  element.style.setProperty("--k-colour-hue-tone", String(HUE_TRACK.tone));
  element.style.setProperty("--k-colour-hue-chroma", String(HUE_TRACK.chroma));
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
  root.className = "k-colour-hue";
  wheel(root);
  root.setAttribute("role", "slider");
  root.setAttribute("aria-label", "Hue");
  root.setAttribute("aria-valuemin", "0");
  root.setAttribute("aria-valuemax", "359");
  root.tabIndex = disabled ? -1 : 0;
  if (disabled) root.setAttribute("aria-disabled", "true");
  const knob = document.createElement("span");
  knob.className = "k-colour-hue-knob";
  let words = text("", "label_number", "surface");
  root.append(knob, words);
  focusRing(root, "full", "accent", false);
  let hue = value;
  const draw = () => {
    root.style.setProperty("--k-colour-hue-at", String(hue / 359));
    root.setAttribute("aria-valuenow", String(Math.round(hue)));
    // A text's box is its words' width, so new words are a new text.
    const said = text(DEGREES.format(Math.round(hue)), "label_number");
    // Its colour is the CSS's: dark on the light hues in either mode.
    said.style.removeProperty("color");
    said.classList.add("k-colour-hue-value");
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
  made.className = "k-colour-swatch";
  made.toggleAttribute("data-custom", custom);
  made.style.setProperty("--k-colour-swatch", colour);
  made.setAttribute("aria-pressed", String(picked));
  made.setAttribute("aria-label", name);
  made.title = name;
  made.tabIndex = -1;
  return made;
}

/** The picker for `options.picked`, its three controls for its user to lay out in its rows. */
export function colourPicker<R extends string>(options: ColourPickerOptions<R>): ColourPicker {
  const { picked } = options;
  const disabled = options.disabled ?? false;
  const offersAuto = options.auto ?? true;
  const offersCustom = options.custom ?? true;
  const own = picked.mode === "custom" ? picked : CUSTOM_FIRST;
  let h: number = own.h;
  let l: number = own.l;
  const choice = document.createElement("span");
  choice.className = "k-colour";
  wheel(choice);
  choice.setAttribute("role", "radiogroup");
  choice.setAttribute("aria-label", options.label ?? "Colour");
  choice.tabIndex = disabled ? -1 : 0;
  if (disabled) choice.setAttribute("aria-disabled", "true");
  if (options.pinned !== "pressed") pin(choice, options.pinned);
  focusRing(choice, "full", "accent", false);
  const picks: { readonly element: HTMLElement; readonly colour: () => PickedColour<R> }[] = [];
  if (offersAuto) {
    const auto = chip({
      label: "Auto",
      kind: picked.mode === "auto" ? "Selected" : "Neutral",
      press: "press",
      tabStop: false,
      disabled,
      ...(options.pinned === "pressed" ? { pinned: "pressed" as const } : {}),
    });
    auto.setAttribute("aria-pressed", String(picked.mode === "auto"));
    picks.push({ element: auto, colour: () => ({ mode: "auto" }) });
  }
  for (const { role, name, colour } of options.swatches) {
    picks.push({ element: swatch(colour, name, picked.mode === "role" && picked.role === role, false), colour: () => ({ mode: "role", role }) });
  }
  const customSwatch = offersCustom ? swatch(picked.mode === "custom" ? customColour(h, l) : "", "Custom", picked.mode === "custom", true) : null;
  if (customSwatch) picks.push({ element: customSwatch, colour: () => ({ mode: "custom", h, l }) });
  choice.append(...picks.map((each) => each.element));
  for (const each of picks) {
    if (disabled) break;
    each.element.addEventListener("click", (event) => {
      event.stopPropagation();
      options.onPick?.(each.colour());
    });
  }
  choice.addEventListener("keydown", (event) => {
    if (disabled) return;
    const by = event.key === "ArrowLeft" || event.key === "h" ? -1 : event.key === "ArrowRight" || event.key === "l" ? 1 : 0;
    if (by === 0) return;
    event.preventDefault();
    const now = picks.findIndex((each) => each.element.getAttribute("aria-pressed") === "true");
    const next = picks[Math.max(0, Math.min(picks.length - 1, now + by))];
    if (next && next !== picks[now]) options.onPick?.(next.colour());
  });
  probe(choice, { role: "component", name: "ColourPicker", variant: variant(offersAuto && "auto", offersCustom && "custom"), fixed: { width: false, height: false } });
  if (!offersCustom) return { choice, hue: null, tone: null };

  /** A custom colour moved: its swatch shows it, and the user hears it. */
  const tuned = () => {
    options.onTune?.(h, l);
    customSwatch?.style.setProperty("--k-colour-swatch", customColour(h, l));
  };
  const live = picked.mode === "custom" && !disabled;
  const hue = hueTrack(h, !live, (to) => {
    h = to;
    tuned();
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
      tuned();
    },
  });
  return { choice, hue, tone };
}
