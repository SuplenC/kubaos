// The Task: one thing to try, ticked once it is done (`docs/design/components/task.md`). A card
// `size.row_two_line` tall at the least, `space.s12` in above and below and `space.s16` at its sides:
// a round tick `size.check` wide, then its title in `body_strong` (any words to type in it set in
// `mono_large`, in `primary`) over a line in `small`. To do, the tick is an empty ring in `outline`
// on the container's fill; done, the card lights as a picked card does, the tube in `primary` with
// its near glow at night and the accent's tint, the tick filled `primary_fill` with its check. Its
// look is `feedback.css`.
//
// First boot's Kotodama card lists the three things to try in it: the step's own work, nothing
// required.

import "./feedback.css";
import { icon } from "./icon.ts";
import { filling, gap, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { text } from "./text.ts";
import { settings } from "../site/settings.ts";
import { cubic } from "./tokens.ts";
import { milliseconds } from "../site/recipe.ts";

export interface TaskOptions {
  /** The title's words before what to type, what to type, and the words after it. */
  readonly before: string;
  readonly typed?: string;
  readonly after?: string;
  /** The line under the title. */
  readonly line: string;
  readonly done?: boolean;
  /** What a press on it does: it takes one when set. */
  readonly onPress?: () => void;
}

export interface Task {
  readonly element: HTMLElement;
  /** Marks it done; its tick pops in over `medium` on `pop`, at once with reduced motion. */
  readonly tick: () => void;
}

export function task(options: TaskOptions): Task {
  const mark = document.createElement("span");
  mark.className = "k-task-tick";
  mark.append(icon("check", "var(--type-small-icon)", "currentColor"));
  const title = row(0, "centre", text(options.before, "body_strong"), ...(options.typed ? [gap("s4"), text(options.typed, "mono_large", "primary")] : []), ...(options.after ? [text(options.after, "body_strong")] : []));
  const element = row("s12", "centre", mark, filling(stack(0, "start", filling(title), text(options.line, "small", "on_surface_variant"))));
  element.classList.add("k-task");
  element.toggleAttribute("data-done", options.done ?? false);
  if (options.onPress) {
    element.dataset.pressable = "";
    element.addEventListener("click", options.onPress);
  }
  probe(element, { role: "component", name: "Task", variant: options.done ? "done" : "", fixed: { width: false, height: false } });
  return {
    element,
    tick: () => {
      if (element.hasAttribute("data-done")) return;
      element.setAttribute("data-done", "");
      if (settings().reduced) return;
      const duration = milliseconds("medium") / settings().speed;
      mark.animate([{ scale: 0.4 }, { scale: 1 }], { duration, easing: `cubic-bezier(${cubic("easing-pop").join(", ")})` });
    },
  };
}
