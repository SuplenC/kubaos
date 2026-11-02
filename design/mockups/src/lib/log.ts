// The log behind an error or a progress bar: `kubaos-ui`'s `error::log`, `KubaOS.Components`'
// `Log` (`docs/design/components/log.md`). A sunken well (`lib/list.ts`'s scrolling well: the soft
// hairline, the content radius, its own thumb) on `surface_container_lowest` at `opacity.log`,
// `size.log` tall unless its place says otherwise, its lines in `mono`, newest at the bottom,
// `space.s16` in at the sides and `space.s12` above and below.
//
// Each line takes its level's colour: an error `error`, a warning `warning`, the start of a step
// `cold`, the rest `on_surface`, debug `on_surface_variant`. Lines wrap between words, or run on
// one line each and scroll across as well.
//
// It never jumps: at its end it follows the newest line; scrolled up, what is read holds still as
// lines arrive below it, and lines dropped from its top give their room back without moving it.
// End, or scrolling back down to the end, follows again. Its words select as text and copy.

import "./feedback.css";
import { scaleOf, scrolled } from "./list.ts";
import { stack } from "./layout.ts";
import { text, wrapped } from "./text.ts";
import { length } from "./tokens.ts";

export const LEVELS = ["error", "warn", "step", "info", "debug"] as const;
export type Level = (typeof LEVELS)[number];

/** A line's colour role by its level. */
export const LEVEL_COLOURS: Record<Level, string> = {
  error: "error",
  warn: "warning",
  step: "cold",
  info: "on_surface",
  debug: "on_surface_variant",
};

export interface LogLine {
  readonly level: Level;
  readonly text: string;
}

export interface LogOptions {
  /** Its height, a length's CSS: `size.log` unless given. */
  readonly height?: string;
  /** Each line on one line, the log scrolling across too, in place of wrapping. */
  readonly unwrapped?: boolean;
  /** The most lines it keeps; older ones leave from its top. */
  readonly most?: number;
  /** Scrolled this share of the way down, as a sample of it scrolled up. */
  readonly down?: number;
  /** The thumb pinned as under the pointer. */
  readonly thumbPinned?: boolean;
}

export interface Log {
  readonly box: HTMLElement;
  /** Adds `lines` at its end. */
  readonly append: (...lines: LogLine[]) => void;
  /** Whether it follows its newest line. */
  readonly following: () => boolean;
}

function lineOf(line: LogLine, unwrapped: boolean): HTMLElement {
  const colour = LEVEL_COLOURS[line.level];
  const element = unwrapped ? text(line.text, "mono", colour) : wrapped(line.text, "mono", colour);
  element.classList.add("k-log-line");
  element.dataset.level = line.level;
  return element;
}

/** A line's height in the page's pixels: `mono`'s line at the log's scale. */
function lineHeight(box: HTMLElement): number {
  return length("type-mono-line") * scaleOf(box);
}

export function log(lines: readonly LogLine[], options: LogOptions = {}): Log {
  const unwrapped = options.unwrapped ?? false;
  const column = stack(0, "start", ...lines.map((line) => lineOf(line, unwrapped)));
  column.classList.add("k-log-lines");
  const { box, pane, thumb } = scrolled("Log", column, { height: options.height ?? "var(--size-log)", ...(options.thumbPinned ? { thumbPinned: true } : {}) });
  box.classList.add("k-log");
  box.toggleAttribute("data-unwrapped", unwrapped);
  box.tabIndex = 0;
  box.setAttribute("role", "log");
  let follows = true;

  const atEnd = () => pane.scrollTop + pane.clientHeight >= pane.scrollHeight - 1;
  pane.addEventListener("scroll", () => (follows = atEnd()), { passive: true });

  function toEnd(): void {
    pane.scrollTop = pane.scrollHeight;
    follows = true;
    thumb();
  }

  function append(...added: LogLine[]): void {
    column.append(...added.map((line) => lineOf(line, unwrapped)));
    const most = options.most;
    if (most !== undefined) {
      let given = 0;
      while (column.children.length > most) {
        const first = column.firstElementChild as HTMLElement;
        given += first.offsetHeight;
        first.remove();
      }
      if (!follows) pane.scrollTop -= given;
    }
    if (follows) toEnd();
    else thumb();
  }

  function selectAll(): void {
    const range = document.createRange();
    range.selectNodeContents(column);
    const selection = window.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);
  }

  box.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key.toLowerCase() === "a") {
      event.preventDefault();
      selectAll();
      return;
    }
    const line = lineHeight(box);
    const page = pane.clientHeight - line;
    const by: Record<string, number> = { ArrowDown: line, j: line, ArrowUp: -line, k: -line, PageDown: page, PageUp: -page };
    if (event.key === "Home") pane.scrollTop = 0;
    else if (event.key === "End") toEnd();
    else if (by[event.key] !== undefined) pane.scrollTop += by[event.key]!;
    else return;
    event.preventDefault();
  });

  requestAnimationFrame(() => {
    if (options.down === undefined) toEnd();
    else {
      pane.scrollTop = Math.round((pane.scrollHeight - pane.clientHeight) * options.down);
      follows = atEnd();
      thumb();
    }
  });
  return { box, append, following: () => follows };
}

/**
 * `shown`, a log, in a room that opens from nothing to its height as `--log-open` goes from 0 to 1,
 * cutting it as it goes: the log opening under an error or a progress bar. `gap` (a spacing token's
 * name) is the room above the log, which opens and closes with it, so a shut log leaves no gap.
 */
export function logRoom(shown: HTMLElement, open: boolean, gap?: string): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-log-room";
  room.style.setProperty("--log-open", open ? "1" : "0");
  if (gap) room.style.setProperty("--log-gap", `var(--space-${gap})`);
  room.append(shown);
  return room;
}
