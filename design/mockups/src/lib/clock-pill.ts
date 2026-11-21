// The bar's clock, the ClockPill, a DRAFT part (`docs/design/components/bar.md`): the weekday's seal
// (its kanji on a disc of `cold`), the date plain, the time in a flat capsule of `primary`, and the
// weather plain like the date. The capsule is the screen's exact middle and the pill is laid out
// round it. Recording merged in, the pill and its capsule take the recording's colours and its dot
// leads. The bar draws it, and the Lock draws it in its bar for the rest clock to fly into.

import "./clock-pill.css";
import { type Shown, stateOptions } from "./control.ts";
import { icon } from "./icon.ts";
import { widestTime } from "./lock.ts";
import { paired, pill } from "./pill.ts";
import { text } from "./text.ts";

/** What the pill says: the weekday's kanji, the date full and short, the time, and the weather. */
export interface ClockPillWords {
  readonly kanji: string;
  readonly date: string;
  /** The date without its month, for a narrower bar: "Tue 27". */
  readonly day: string;
  readonly time: string;
  /** What its hint says: the date written out. */
  readonly whole: string;
  readonly weather?: { readonly symbol: string; readonly temperature: string };
}

export interface ClockPillOptions {
  readonly words: ClockPillWords;
  /** The weekday's seal. */
  readonly seal?: boolean;
  readonly date?: "full" | "day" | "none";
  /** The weather, where the words have it: none with no home city, or once a narrow bar sheds it. */
  readonly weather?: boolean;
  /** Recording merged in on a narrow bar: the recording's colours, its dot leading. */
  readonly recording?: boolean;
  /** Whether the time is a 12-hour clock's, for the slot it keeps. */
  readonly twelve?: boolean;
  /** A press target, lit by the pointer as the bar's pills are; false for a picture of one. */
  readonly interactive?: boolean;
  readonly state?: Shown;
}

/** The pill, and its time: the words the Lock's clock lands on. */
export interface ClockPill {
  readonly element: HTMLElement;
  readonly time: HTMLElement;
}

/**
 * The time in `body`, Geist's own proportional figures, centred in a slot as wide as the widest
 * time its format shows: the pill keeps its width as the minutes turn, and the bar's steps never
 * meet a wider time than they were laid out for.
 */
function slotted(time: string, color: string, twelve: boolean): HTMLElement {
  const shown = text(time, "body", color);
  shown.classList.add("k-clock-pill-time");
  shown.style.setProperty("--slot", String(widestTime("body", twelve)));
  return shown;
}

/** One of the parts round the capsule, which come and go round it: plain words at an end stand `space.s12` from the curve. */
function around<E extends Element>(element: E, plain: boolean): E {
  element.classList.add("k-clock-pill-around");
  if (plain) element.classList.add("k-clock-pill-plain");
  return element;
}

export function clockPill(options: ClockPillOptions): ClockPill {
  const words = options.words;
  const recording = options.recording ?? false;
  const ink = (role: string) => (recording ? "on_error_container" : role);
  const parts: Element[] = [];
  if (recording) {
    const dot = document.createElement("span");
    dot.className = "k-clock-pill-dot";
    parts.push(around(dot, true));
  }
  if (options.seal ?? true) {
    const seal = document.createElement("span");
    seal.className = "k-clock-pill-seal";
    seal.append(text(words.kanji, "accent", recording ? "error_container" : "on_cold"));
    parts.push(around(seal, false));
  }
  const date = options.date ?? "full";
  if (date !== "none") parts.push(around(text(date === "full" ? words.date : words.day, "body", ink("on_surface_variant")), true));
  const capsule = document.createElement("span");
  capsule.className = "k-clock-pill-capsule";
  const time = slotted(words.time, recording ? "on_error" : "on_primary", options.twelve ?? false);
  capsule.append(time);
  parts.push(capsule);
  if ((options.weather ?? true) && words.weather) {
    const weather = paired(icon(words.weather.symbol, "var(--icon-s16)", `var(--color-${ink("on_surface_variant")})`), text(words.weather.temperature, "body", ink("on_surface_variant")));
    weather.classList.add("k-clock-pill-weather");
    parts.push(around(weather, true));
  }
  const interactive = options.interactive ?? true;
  const element = pill({
    content: parts,
    lone: false,
    end: "s4",
    ...(recording ? { tone: "recording" as const } : {}),
    ...(interactive ? { interactive: true, tooltip: recording ? `Recording. ${words.whole}` : words.whole, ...stateOptions(options.state ?? "rest") } : {}),
  });
  element.classList.add("k-clock-pill");
  element.toggleAttribute("data-recording", recording);
  centreCapsule(element);
  return { element, time };
}

/** Where the capsule's middle sits in `pill`, from its left, in the viewport's pixels. */
export function capsuleExtents(pill: HTMLElement): { readonly before: number; readonly after: number } {
  const capsule = pill.querySelector<HTMLElement>(".k-clock-pill-capsule");
  const box = pill.getBoundingClientRect();
  if (!capsule || box.width === 0) return { before: box.width / 2, after: box.width / 2 };
  const own = capsule.getBoundingClientRect();
  const middle = own.left + own.width / 2 - box.left;
  return { before: middle, after: box.width - middle };
}

/**
 * Keeps the capsule on the screen's exact middle: its owner centres the pill's box, so the side of
 * the capsule with less beside it takes a margin of the difference, centred with the pill. A width
 * worked out from the layout, which the CSS can't read.
 */
function centreCapsule(pill: HTMLElement): void {
  new ResizeObserver(() => {
    const zoom = pill.getBoundingClientRect().width / Math.max(1, pill.offsetWidth) || 1;
    const { before, after } = capsuleExtents(pill);
    const spare = Math.round((before - after) / zoom);
    pill.style.setProperty("--capsule-before", `${Math.max(0, -spare)}px`);
    pill.style.setProperty("--capsule-after", `${Math.max(0, spare)}px`);
  }).observe(pill);
}
