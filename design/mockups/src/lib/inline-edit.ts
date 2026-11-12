// A row's text that is edited where it is: `InlineEdit`, new, in neither library yet. A double
// click on it, or F2 on its focused row, turns it editable in a box of its own: `surface_container_highest`
// under it and a `primary` underline the thin stroke tall along its foot, all of it selected. The box
// reaches `space.s4` past the text at either side and is the text's line tall, so the text doesn't
// move and the row keeps its height. Enter saves; Esc, or a click anywhere else, cancels. A value
// it can't take (a name left empty, a count that isn't one) shakes the box once, `space.s4` either
// way, and the old value stays.

import "./inline-edit.css";
import { probe } from "./probe.ts";
import { text, type TypeStyle } from "./text.ts";
import { raw } from "./tokens.ts";
import { settings } from "../site/settings.ts";

export interface InlineEditOptions {
  readonly value: string;
  readonly style: TypeStyle;
  readonly color?: string;
  /** What it shows at rest for `value`: the row's clamped title, a rolling number. */
  readonly rest: (value: string) => HTMLElement;
  /** Its text's place in its box: a name from the start, a count to the end. */
  readonly align?: "start" | "end";
  /** Only figures may be typed: a count. */
  readonly numeric?: boolean;
  /** The value typed as it is saved, or null for one it can't take. */
  readonly check: (typed: string) => string | null;
  readonly onSave?: (value: string) => void;
  /** Drawn editing, all of it selected: a still sample. */
  readonly editing?: boolean;
  /** Drawn mid-shake, `--shake` from -1 to 1: a still sample, and the player's. */
  readonly shake?: number;
}

export interface InlineEdit {
  readonly element: HTMLElement;
  /** Turns it editable, all of it selected: F2 on its row. */
  readonly edit: () => void;
  readonly value: () => string;
}

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The text a still sample shows editing: the value over the selection's highlight. */
function selectedText(value: string, style: TypeStyle, color: string): HTMLElement {
  const words = text(value, style, color);
  words.classList.add("k-inline-selected");
  return words;
}

export function inlineEdit(options: InlineEditOptions): InlineEdit {
  const color = options.color ?? "on_surface";
  const element = document.createElement("span");
  element.className = "k-inline-edit";
  element.dataset.align = options.align ?? "start";
  element.style.setProperty("--t-line", `var(--type-${options.style}-line)`);
  const box = document.createElement("span");
  box.className = "k-inline-box";
  box.setAttribute("aria-hidden", "true");
  element.append(box);
  let value = options.value;
  let shown = options.rest(value);
  element.append(shown);

  function rest(): void {
    delete element.dataset.editing;
    const fresh = options.rest(value);
    element.querySelector(".k-inline-input, .k-inline-selected")?.remove();
    shown.remove();
    shown = fresh;
    element.append(shown);
  }

  function shake(): void {
    if (settings().reduced) return;
    // Once each way and back to where it was: `micro` a leg, three legs, on `standard`.
    element.animate([{ "--shake": 0 }, { "--shake": -1 }, { "--shake": 1 }, { "--shake": 0 }], { duration: 3 * ms("micro"), easing: raw("easing-standard") });
  }

  function edit(): void {
    if (element.hasAttribute("data-editing")) return;
    element.dataset.editing = "";
    const input = document.createElement("input");
    input.className = "k-inline-input";
    input.value = value;
    input.style.setProperty("--t-family", `var(--type-${options.style}-family)`);
    input.style.setProperty("--t-size", `var(--type-${options.style}-size)`);
    input.style.setProperty("--t-weight", `var(--type-${options.style}-weight)`);
    input.style.color = `var(--color-${color})`;
    if (options.numeric) input.inputMode = "numeric";
    shown.remove();
    element.append(input);
    input.focus({ preventScroll: true });
    input.select();
    let done = false;
    const end = (save: boolean) => {
      if (done) return;
      if (save) {
        const taken = options.check(input.value);
        if (taken === null) {
          done = true;
          shake();
          setTimeout(rest, settings().reduced ? 0 : 3 * ms("micro"));
          return;
        }
        value = taken;
        options.onSave?.(value);
      }
      done = true;
      rest();
    };
    input.addEventListener("keydown", (event) => {
      event.stopPropagation();
      if (event.key === "Enter") end(true);
      else if (event.key === "Escape") end(false);
      else return;
      event.preventDefault();
    });
    input.addEventListener("blur", () => end(false));
  }

  element.addEventListener("dblclick", (event) => {
    event.stopPropagation();
    edit();
  });

  if (options.editing) {
    element.dataset.editing = "";
    shown.remove();
    element.append(selectedText(value, options.style, color));
  }
  if (options.shake !== undefined) element.style.setProperty("--shake", String(options.shake));

  probe(element, { role: "component", name: "InlineEdit", variant: options.numeric ? "number" : "", fixed: { width: false, height: true } });
  return { element, edit, value: () => value };
}
