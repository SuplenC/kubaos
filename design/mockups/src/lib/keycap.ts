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
// glowing at night as a lit control does, a `+` in `subheading` between them. It is still a picture:
// it takes no focus and nothing presses it.

import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe } from "./probe.ts";
import { advance, text } from "./text.ts";

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

/** The keys of `combination`, large and lit, a `+` between each two, `space.s12` apart. */
export function keyPrompt(combination: string): HTMLElement {
  const parts = keys(combination).flatMap((key, at) => {
    const cap = document.createElement("span");
    cap.className = "k-key-prompt-cap";
    cap.append(text(key, "button_large"));
    return at === 0 ? [cap] : [text("+", "subheading", "on_surface_variant"), cap];
  });
  const line = row("s12", "centre", ...parts);
  line.classList.add("k-key-prompt");
  return probe(line, { role: "component", name: "KeyPrompt", variant: "", fixed: { width: false, height: true } });
}

/** `parts` on one line, `space.s4` apart, centred down it: a keycap inline with words. */
export function inline(...parts: Element[]): HTMLElement {
  return row("s4", "centre", ...parts);
}
