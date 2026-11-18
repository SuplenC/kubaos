// Keys drawn as keys, and keys with what each does: `kubaos-ui`'s `keycap::keycap` and
// `keycap::key_hints`, `KubaOS.Components`' `Keycap` and `KeyHints` (`docs/design/components/
// keycap.md`). A keycap is `size.keycap` tall, as a line of body text, the `keycap` type in
// `on_surface_variant` on the highest container, a hairline edge in `outline_variant` with a second
// at its foot, the small radius, `space.s4` either side of the key and never narrower than tall.
//
// A keycap is a picture of a key, not a control: it has no states of its own and takes no focus.
// On a lit menu item its key and its edge take the item's light, in `primary`.
//
// A KeyPrompt is the one place keys are drawn to be pressed now: the keys a surface asks for, large,
// each `size.control_lg` tall in `button_large` on the high container, in the primary's tube and
// glowing at night as a lit control does, a `+` in `subheading` between them. It takes no focus,
// but it answers its keys: pressed, each key's fill lifts a step and its glow swells to
// `focus.pressed_glow` times its own, as a pressed control's does.

import { milliseconds } from "../site/recipe.ts";
import { settings } from "../site/settings.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { advance, centreFor, text, type TypeStyle, wrapped } from "./text.ts";
import { cubic } from "./tokens.ts";

/** `key`, as it is written on the keyboard (`Esc`, `K`, `↵`), on a keycap. */
export function keycap(key: string): HTMLElement {
  const cap = document.createElement("span");
  cap.className = "k-keycap";
  cap.style.setProperty("--natural", String(advance("keycap", key)));
  const words = text(key, "keycap");
  words.style.color = "var(--key-ink)";
  const label = row(0, "start", words);
  label.classList.add("k-keycap-label");
  cap.append(label);
  return probe(cap, { role: "component", name: "Keycap", variant: "", fixed: FIXED_HEIGHT });
}

/**
 * The symbols that are a key each even when written together: the modifiers' and the arrows'
 * glyphs, Return, Backspace and Delete.
 */
const GLYPHS = "⌃⇧⌥⌘↑↓←→↵⌫⌦";

/**
 * The keys of a combination, one keycap each: `Ctrl+K` is `Ctrl` and `K`, `⌃K` is `⌃` and `K`,
 * `← →` is `←` and `→`, and `Left Alt` is one key; a `+` alone is the plus key.
 */
export function keys(combination: string): string[] {
  if (combination === "+") return ["+"];
  const found: string[] = [];
  const push = (word: string) => {
    const trimmed = word.trim();
    if (trimmed !== "") found.push(trimmed);
  };
  for (const part of combination.split("+")) {
    let word = "";
    for (const c of part) {
      if (GLYPHS.includes(c)) {
        push(word);
        push(c);
        word = "";
      } else {
        word += c;
      }
    }
    push(word);
  }
  return found;
}

/** A combination and what it does; no words for a key that says enough alone. */
export type KeyHint = readonly [combination: string, does: string];

export interface KeyHintsOptions {
  /** The colour role of what each does. */
  readonly color?: string;
  /** The widest the hints may take (a length's CSS): a hint past it is left out, with every one after it. */
  readonly room?: string;
}

/**
 * Key hints side by side, `space.s16` apart: each combination's keys as keycaps `space.s4` apart,
 * then what it does in `caption`, `space.s8` after them.
 */
export function keyHints(hints: readonly KeyHint[], options: KeyHintsOptions = {}): HTMLElement {
  const parts = hints.map(([combination, does]) => {
    const caps = row("s4", "centre", ...keys(combination).map(keycap));
    return row("s8", "centre", caps, ...(does === "" ? [] : [text(does, "caption", options.color ?? "on_surface_variant")]));
  });
  const line = row("s16", "centre", ...parts);
  line.classList.add("k-key-hints");
  if (options.room) {
    line.dataset.room = "";
    line.style.setProperty("--room", options.room);
  }
  return probe(line, { role: "component", name: "KeyHints", variant: "", fixed: { width: false, height: false } });
}

export interface KeyPromptOptions {
  /** Held in a state for a sample: its keys pressed. */
  readonly pinned?: "pressed";
}

/** The keys of `combination`, large and lit, a `+` between each two, `space.s12` apart. */
export function keyPrompt(combination: string, options: KeyPromptOptions = {}): HTMLElement {
  const parts = keys(combination).flatMap((key, at) => {
    const cap = document.createElement("span");
    cap.className = "k-key-prompt-cap";
    cap.append(text(key, "button_large"));
    return at === 0 ? [cap] : [text("+", "subheading", "on_surface_variant"), cap];
  });
  const line = row("s12", "centre", ...parts);
  line.classList.add("k-key-prompt");
  if (options.pinned) line.dataset.pinned = options.pinned;
  return probe(line, { role: "component", name: "KeyPrompt", variant: "", fixed: { width: false, height: true } });
}

/**
 * Plays a KeyPrompt's keys being pressed, as its keys are: `--press` up to 1 over `micro`, then
 * back over `short`. It moves nothing, so reduce motion keeps it.
 */
export function pressKeys(prompt: HTMLElement): void {
  const speed = settings().speed;
  const down = milliseconds("micro") / speed;
  const total = down + milliseconds("short") / speed;
  for (const cap of prompt.querySelectorAll<HTMLElement>(".k-key-prompt-cap")) {
    cap.animate([{ "--press": 0 }, { "--press": 1, offset: down / total }, { "--press": 0 }], { duration: total, easing: `cubic-bezier(${cubic("easing-standard").join(", ")})` });
  }
}

/** `parts` on one line, `space.s4` apart, centred down it: a keycap inline with words. */
export function inline(...parts: Element[]): HTMLElement {
  return row("s4", "centre", ...parts);
}

/**
 * A sentence that names keys as it goes, wrapping: `words` with each `[Key]` drawn as a keycap where
 * it stands, in the line (`Hold [Super] and drag`). A sentence with keys in it sets its lines a
 * keycap and `space.s4` apart, so every line is as tall whether it holds a key or not, and two keys
 * on lines one over the other never touch; one without is plain wrapped text. Its words are a text
 * each, as wide as the text rounds them, so every word and key starts on a whole device pixel, as
 * the libraries lay a line of runs; the line breaks between them.
 */
export function sentence(words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  const parts = words.split(/\[([^\]]+)\]/);
  if (parts.length === 1) return wrapped(words, style, color);
  // A part holding its runs, each a text or a key measured alone, as the libraries hold a line of
  // runs: what it holds is its words, so two sentences are never the same parts.
  const element = probe(document.createElement("span"), { role: "component", name: "Sentence", variant: "", fixed: { width: false, height: false, holds: true } });
  element.className = "k-said";
  parts.forEach((part, at) => {
    if (at % 2 === 1) element.append(keycap(part));
    else for (const word of part.match(/\S+\s*|\s+/g) ?? []) element.append(saidWord(word, style, color, words));
  });
  return element;
}

/** One word of a keyed sentence, with the space after it, set on the whole sentence's mass so every word shares one baseline. */
function saidWord(word: string, style: TypeStyle, color: string, whole: string): HTMLElement {
  const made = text(word, style, color);
  centreFor(made, style, whole.replaceAll(/[[\]]/g, ""));
  made.classList.add("k-said-word");
  return made;
}
