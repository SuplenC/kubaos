// One line of text, its label and its note: `kubaos-ui`'s `field::field`, `field::labelled` and
// `field::note`, `KubaOS.Components`' `Field`, `Labelled` and `Note` (`docs/design/components/
// field.md`). A field is `size.control_md` tall at the control radius, a hairline inside its edge,
// its value in `body_large` `space.s12` in, or past a leading `body_large` icon and `space.s8`
// more, and it fills the room it is given.
//
// It is a control in the state model (`.k-control` in `lib.css`): hovered or with the keyboard in
// it, it is lit, the tube over its hairline in its tone (the accent, or the error's on an invalid
// field) with the glow outside. With the keyboard in it, it also shows the caret, or the selection.
// A read-only field lights and selects but never edits; a disabled one takes no input and never
// lights.
//
// The caret is a core line, steady, never blinking. It strikes on as the keyboard comes in, and at
// night a faint glow in the tone shows round it while you type, then fades once the keys stop.
//
// The field owns its value and its caret here, as the app does in both libraries: keys and the
// pointer edit them, and the field draws them again, scrolled sideways to keep the caret in view.

import { settings } from "../site/settings.ts";
import { pin, type Pinned } from "./control.ts";
import { focusRing, glowMask } from "./focus.ts";
import { icon } from "./icon.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { advance, text, wrapped } from "./text.ts";
import { length, number, raw } from "./tokens.ts";

/** How a field shows its value: editable, wrong, read-only, or not usable now. */
export const FIELD_STATES = ["Normal", "Invalid", "Locked", "Disabled"] as const;
export type FieldState = (typeof FIELD_STATES)[number];

/** Where the caret is and what is selected, in characters: `anchor` at `pos` selects nothing. */
export interface Caret {
  readonly pos: number;
  readonly anchor: number;
}

export interface FieldOptions {
  readonly value?: string;
  /** A hint shown while the field is empty, never a value that looks typed. */
  readonly placeholder?: string;
  /** A symbol before the text. */
  readonly icon?: string;
  /** One bullet per character instead of the value: a password. */
  readonly secret?: boolean;
  readonly state?: FieldState;
  /** A state the field is pinned in: lit, with the keyboard in it unless `pointer`, or signs off. */
  readonly pinned?: Pinned;
  /** Pinned lit under the pointer rather than with the keyboard in it: no caret. */
  readonly pointer?: boolean;
  /** The caret, while the keyboard is in the field; at the end of the value unless given. */
  readonly caret?: Caret;
}

/** `words`' characters, so a position counts what a person sees, not UTF-16 units. */
const chars = (words: string) => [...words];

/** `words`' first `n` characters' advance in the value's type, in logical pixels. */
function upTo(words: string, n: number): number {
  return n <= 0 ? 0 : advance("body_large", chars(words).slice(0, n).join(""));
}

/** The character position nearest `x` logical pixels into `words`. */
function nearest(words: string, x: number): number {
  const count = chars(words).length;
  for (let n = 1; n <= count; n++) {
    const before = upTo(words, n - 1);
    const after = upTo(words, n);
    if (x < (before + after) / 2) return n - 1;
  }
  return count;
}

/** The word around `pos`: from the start of its run of letters to its end. */
function wordAt(words: string, pos: number): Caret {
  const list = chars(words);
  const inWord = (at: number) => at >= 0 && at < list.length && /\S/.test(list[at]!);
  let start = pos;
  while (inWord(start - 1)) start--;
  let end = pos;
  while (inWord(end)) end++;
  return { pos: end, anchor: start };
}

/** The position a word to the left or the right of `pos`, as Ctrl with an arrow moves. */
function wordStep(words: string, pos: number, by: -1 | 1): number {
  const list = chars(words);
  const space = (at: number) => /\s/.test(list[at] ?? " ");
  let at = pos;
  if (by < 0) {
    while (at > 0 && space(at - 1)) at--;
    while (at > 0 && !space(at - 1)) at--;
  } else {
    while (at < list.length && space(at)) at++;
    while (at < list.length && !space(at)) at++;
  }
  return at;
}

/** The duration token `name` in milliseconds under the speed setting. */
function motion(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The scale `part` is drawn at: the page's, or that of a box drawn at a scale of its own. */
function scaleOf(part: HTMLElement): number {
  return Number.parseFloat(getComputedStyle(part).getPropertyValue("--scale")) || settings().scale;
}

// The caret's glow is a mask round its own rounded line, worked out per device pixel as a ring's
// is, so it is drawn again whenever the caret's size changes: shown, or at another scale.
const caretSized = new ResizeObserver((entries) => {
  for (const entry of entries) drawCaretGlow(entry.target as HTMLElement);
});

/** Draws the glow round `caret` for its size now: `caret.glow` at its edge, out to `caret.near`. */
function drawCaretGlow(caret: HTMLElement): void {
  const glowing = caret.firstElementChild as HTMLElement | null;
  const width = caret.offsetWidth;
  const height = caret.offsetHeight;
  if (!glowing || width === 0 || height === 0) return;
  const scale = scaleOf(caret);
  const reach = Math.max(0, Math.ceil(length("caret-near") * scale - 1e-3));
  const key = `${width}x${height}+${reach}@${scale}`;
  if (glowing.dataset.drawn === key) return;
  glowing.dataset.drawn = key;
  glowing.style.inset = `${-reach}px`;
  const near = length("caret-near");
  const peak = number("caret-glow");
  const at = (d: number) => (d / scale >= near ? 0 : peak * (1 - d / scale / near) ** 2);
  glowing.style.maskImage = glowMask(width, height, width / 2, reach, at);
}

/**
 * The caret, with its glow inside it so a strike moves both: one element for the field's life, so
 * a glow fading carries on through every key.
 */
function caretMark(): HTMLElement {
  const mark = document.createElement("span");
  mark.className = "k-field-caret";
  const glowing = document.createElement("span");
  glowing.className = "k-field-caret-glow";
  // Nothing shows until the mask is drawn for the caret's size.
  glowing.style.maskImage = "linear-gradient(transparent, transparent)";
  mark.append(glowing);
  caretSized.observe(mark);
  return mark;
}

/**
 * The glow round `caret` coming in at once and holding through `held` milliseconds, then fading
 * over `caret_fade`: from its glow now, so a key mid-fade brings it back without a jump. With
 * reduced motion it shows and hides at once.
 */
function glowFor(caret: HTMLElement, held: number): void {
  const glowing = caret.firstElementChild as HTMLElement | null;
  if (!glowing) return;
  const now = Number.parseFloat(getComputedStyle(glowing).opacity) || 0;
  for (const running of glowing.getAnimations()) running.cancel();
  if (settings().reduced) {
    glowing.animate([{ opacity: 1 }, { opacity: 1 }], { duration: held });
    return;
  }
  const rise = motion("caret_glow");
  const fade = motion("caret_fade");
  const total = rise + held + fade;
  const curve = raw("easing-standard");
  glowing.animate(
    [
      { offset: 0, opacity: now, easing: curve },
      { offset: rise / total, opacity: 1, easing: "linear" },
      { offset: (rise + held) / total, opacity: 1, easing: curve },
      { offset: 1, opacity: 0 },
    ],
    { duration: total },
  );
}

/**
 * `caret` striking on as the keyboard comes in: through `neon.strike`'s even steps over `medium`,
 * a fade over `short` with the flicker off, at once with reduced motion. Returns how long it takes.
 */
function strike(caret: HTMLElement): number {
  for (const running of caret.getAnimations()) running.cancel();
  if (settings().reduced) return 0;
  if (!settings().flicker) {
    const duration = motion("short");
    caret.animate([{ opacity: 0 }, { opacity: 1 }], { duration, easing: raw("easing-standard") });
    return duration;
  }
  const count = number("neon-strike-steps");
  const steps = Array.from({ length: count }, (_, i) => ({ offset: i / count, opacity: number(`neon-strike-${i}`), easing: "step-end" }));
  const duration = motion("medium");
  caret.animate([...steps, { offset: 1, opacity: number(`neon-strike-${count - 1}`) }], { duration });
  return duration;
}

/** A field, live: hover lights it, a press puts the caret where it lands, and keys edit it. */
export function field(options: FieldOptions = {}): HTMLElement {
  const state = options.state ?? "Normal";
  const inert = state === "Disabled";
  const locked = state === "Locked";
  const element = document.createElement("span");
  element.className = "k-field";
  element.dataset.state = state;
  element.setAttribute("role", "textbox");
  element.style.setProperty("--invalid", state === "Invalid" ? "1" : "0");
  if (state === "Disabled") element.setAttribute("aria-disabled", "true");
  if (state === "Locked") element.setAttribute("aria-readonly", "true");
  if (options.icon) {
    element.dataset.icon = "";
    const mark = document.createElement("span");
    mark.className = "k-field-mark";
    mark.append(icon(options.icon, "var(--type-body_large-icon)", "var(--mark)"));
    element.append(mark);
  }
  const clip = document.createElement("span");
  clip.className = "k-field-clip";
  const room = document.createElement("span");
  room.className = "k-field-room";
  clip.append(room);
  element.append(clip);

  let value = options.value ?? "";
  let secret = options.secret ?? false;
  const length = () => chars(value).length;
  let caret: Caret = options.caret ?? { pos: length(), anchor: length() };
  const shownText = () => (secret ? "•".repeat(length()) : value);
  const caretShown = caretMark();
  element.append(caretShown);

  const draw = () => {
    const shown = shownText();
    const [from, to] = [Math.min(caret.pos, caret.anchor), Math.max(caret.pos, caret.anchor)];
    element.toggleAttribute("data-selecting", from !== to);
    element.style.setProperty("--caret-at", String(upTo(shown, caret.pos)));
    if (shown === "") {
      const placeholder = text(options.placeholder ?? "", "body_large");
      placeholder.style.color = "var(--placeholder)";
      placeholder.classList.add("k-field-placeholder");
      room.replaceChildren(...(options.placeholder ? [placeholder] : []));
      return;
    }
    const line = document.createElement("span");
    line.className = "k-field-line";
    const selection = document.createElement("span");
    selection.className = "k-field-selection";
    selection.style.setProperty("--from", String(upTo(shown, from)));
    selection.style.setProperty("--to", String(upTo(shown, to)));
    const words = text(shown, "body_large");
    words.style.color = "var(--value)";
    line.append(selection, words);
    room.replaceChildren(line);
  };

  const move = (pos: number, extend: boolean) => {
    const at = Math.max(0, Math.min(length(), pos));
    caret = { pos: at, anchor: extend ? caret.anchor : at };
    draw();
  };

  const replace = (inserted: string, from: number, to: number) => {
    const list = chars(value);
    value = [...list.slice(0, from), ...chars(inserted), ...list.slice(to)].join("");
    const at = from + chars(inserted).length;
    caret = { pos: at, anchor: at };
    draw();
    glowFor(caretShown, motion("caret_hold"));
  };

  const copy = (from: number, to: number) => {
    if (from === to || secret) return;
    void navigator.clipboard?.writeText(chars(value).slice(from, to).join("")).catch(() => undefined);
  };

  const edit = (event: KeyboardEvent) => {
    const [from, to] = [Math.min(caret.pos, caret.anchor), Math.max(caret.pos, caret.anchor)];
    const word = event.ctrlKey || event.altKey;
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      const by = event.key === "ArrowLeft" ? -1 : 1;
      const collapse = from !== to && !event.shiftKey;
      const pos = collapse ? (by < 0 ? from : to) : word ? wordStep(value, caret.pos, by) : caret.pos + by;
      move(pos, event.shiftKey);
    } else if (event.key === "Home" || event.key === "End") {
      move(event.key === "Home" ? 0 : length(), event.shiftKey);
    } else if (event.ctrlKey && event.key.toLowerCase() === "a") {
      caret = { pos: length(), anchor: 0 };
      draw();
    } else if (event.ctrlKey && event.key.toLowerCase() === "c") {
      copy(from, to);
    } else if (locked) {
      return;
    } else if (event.key === "Backspace" || event.key === "Delete") {
      if (from !== to) replace("", from, to);
      else if (event.key === "Backspace") replace("", word ? wordStep(value, from, -1) : Math.max(0, from - 1), from);
      else replace("", from, word ? wordStep(value, from, 1) : Math.min(length(), from + 1));
    } else if (event.key.length === 1 && !event.ctrlKey && !event.metaKey) {
      replace(event.key, from, to);
    } else {
      return;
    }
    event.preventDefault();
  };

  if (!inert) {
    element.classList.add("k-control");
    element.tabIndex = 0;
    pin(element, options.pinned);
    if (options.pinned && !options.pointer) element.dataset.keyboard = "";
    const ring = focusRing(element, "control", "accent", false);
    ring.dataset.pressable = "";
    // The tone fades from the accent to the error's as the field turns invalid, so the tube, its
    // glass and its glow follow `--invalid`.
    const tone = (accent: string, error: string) =>
      `color-mix(in srgb, var(--color-${accent}), var(--color-${error}) calc(var(--invalid) * 100%))`;
    ring.style.setProperty("--tube-lit", tone("primary", "error"));
    ring.style.setProperty("--core-lit", tone("primary_core", "error_core"));
    ring.style.setProperty("--tube-unlit", tone("primary_glass", "error_glass"));
    ring.style.setProperty("--core-unlit", tone("primary_unlit", "error_unlit"));
    if (!options.pinned) {
      element.addEventListener("keydown", edit);
      // A read-only field's caret strikes on but never glows: nothing is typed into it.
      element.addEventListener("focus", () => {
        const struck = strike(caretShown);
        if (!locked) glowFor(caretShown, struck + motion("caret_hold"));
      });
      element.addEventListener("mousedown", (event) => {
        // The pointer in logical pixels from the text's start: the page is zoomed, so its client
        // pixels are the room's drawn width over its laid-out one, and a logical pixel is a scale.
        const box = room.getBoundingClientRect();
        const start = room.querySelector(".k-field-line")?.getBoundingClientRect().left ?? box.left;
        const ratio = box.width / room.offsetWidth || 1;
        const at = (x: number) => nearest(shownText(), (x - start) / ratio / settings().scale);
        const pos = at(event.clientX);
        element.focus();
        event.preventDefault();
        if (event.detail === 2) caret = wordAt(value, pos);
        else if (event.detail >= 3) caret = { pos: length(), anchor: 0 };
        else {
          move(pos, event.shiftKey);
          const drag = (moved: MouseEvent) => move(at(moved.clientX), true);
          const stop = () => {
            window.removeEventListener("mousemove", drag);
            window.removeEventListener("mouseup", stop);
          };
          window.addEventListener("mousemove", drag);
          window.addEventListener("mouseup", stop);
          return;
        }
        draw();
      });
    }
  }
  // A Show or Hide action beside the field's label turns its bullets into the value and back.
  element.addEventListener("secret", (event) => {
    secret = (event as CustomEvent<boolean>).detail;
    draw();
  });
  draw();
  return probe(element, { role: "component", name: "Field", variant: options.icon ? "icon" : "", fixed: FIXED_HEIGHT });
}

/** Shows `field`'s value as bullets, or as itself. */
export function setSecret(field: HTMLElement, secret: boolean): void {
  field.dispatchEvent(new CustomEvent("secret", { detail: secret }));
}

/** The line under a field, in `small`: muted, or in the error colour when the value is wrong. */
export function note(words: string, invalid = false): HTMLElement {
  const line = wrapped(words, "small", invalid ? "error" : "on_surface_variant");
  line.classList.add("k-note");
  return probe(line, { role: "component", name: "Note", variant: "", fixed: { width: false, height: false } });
}

export interface LabelledOptions {
  /** A small action at the label's right: a Show toggle, a small button. */
  readonly action?: HTMLElement;
  readonly note?: string;
  readonly invalid?: boolean;
}

/**
 * `control` under `label`, in `label` type, with an optional action at the label's right and a
 * note under it. The label's row is `size.control_sm` tall whether or not it has an action, so
 * fields side by side line up.
 */
export function labelled(label: string, control: HTMLElement, options: LabelledOptions = {}): HTMLElement {
  const fill = probe(document.createElement("span"), { role: "spacer", fill: true });
  fill.className = "k-fill";
  const head = row(0, "centre", text(label, "label", "on_surface_variant"), fill, ...(options.action ? [options.action] : []));
  head.classList.add("k-labelled-head");
  const parts = [head, control, ...(options.note ? [note(options.note, options.invalid ?? false)] : [])];
  const column = document.createElement("span");
  column.className = "k-labelled";
  column.append(...parts);
  return probe(column, { role: "component", name: "Labelled", variant: "", fixed: { width: false, height: false } });
}

/** `parts` in a room `width` wide (a length's custom property): a field fills the room it gets. */
export function fieldRoom(width: string, ...parts: Element[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-field-box";
  box.style.setProperty("--room", width);
  box.append(...parts);
  return box;
}
