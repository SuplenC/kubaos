// The bar's clock, the ClockPill, a DRAFT part (`docs/design/components/clock-pill.md`), flat on the
// glass like every other item: the weekday's seal (its kanji on a disc of `cold`), the date, the
// time, and the weather. It is two press targets: the time's half (seal, date, time) opens the
// calendar, the weather's the weather view, and the edge light lies under the half the pointer is
// on, or the half whose view is open. While its card is open the time sits in a filled capsule of
// `primary`, `size.bar_capsule` tall, coming in over `short` on `standard`; at rest it is plain.
// The capsule's place is the screen's exact middle and the pill is laid out round it. Recording
// merged in, the capsule stays filled in the recording's colours and its dot leads. The bar draws
// it, and the Lock draws it in its bar for the rest clock to fly into.

import "./clock-pill.css";
import { centreOnPixels, forgetCentring } from "./centre.ts";
import { pin, type Shown, stateOptions } from "./control.ts";
import { opens } from "./glass.ts";
import { icon } from "./icon.ts";
import { widestTime } from "./lock.ts";
import { paired, pill } from "./pill.ts";
import { probe, variant } from "./probe.ts";
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
  /** The pointer's state, on the time's half. */
  readonly state?: Shown;
  /** Its card open on the calendar (the time's half) or the weather: the capsule filled. */
  readonly open?: "clock" | "weather";
}

/** The pill, its time (the words the Lock's clock lands on), and its two halves, which a card stands under. */
export interface ClockPill {
  readonly element: HTMLElement;
  readonly time: HTMLElement;
  /** The time's half: the seal, the date and the time. */
  readonly when: HTMLElement;
  /** The weather's half, where the pill has one. */
  readonly weather: HTMLElement | null;
}

/** Fills the clock's capsule, its card open on `view`, or empties it: none. */
export function setClockOpen(pill: HTMLElement, view: "clock" | "weather" | null): void {
  if (view) pill.dataset.open = view;
  else delete pill.dataset.open;
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
  const time = slotted(words.time, recording ? "on_error" : "on_surface", options.twelve ?? false);
  // Its ink turns with the capsule's fill: the plain words' at rest, `on_primary` filled.
  if (!recording) time.style.color = "var(--clock-ink)";
  capsule.append(time);
  parts.push(capsule);
  const interactive = options.interactive ?? true;
  const when = document.createElement("span");
  when.className = "k-clock-pill-when";
  when.append(...parts);
  let weather: HTMLElement | null = null;
  if ((options.weather ?? true) && words.weather) {
    weather = paired(icon(words.weather.symbol, "var(--icon-s16)", `var(--color-${ink("on_surface_variant")})`), text(words.weather.temperature, "body", ink("on_surface_variant")));
    weather.classList.add("k-clock-pill-weather");
    around(weather, true);
  }
  if (interactive) {
    for (const half of [when, weather]) if (half) opens(half);
    pin(when, stateOptions(options.state ?? "rest").pinned);
  }
  const element = pill({
    content: weather ? [when, weather] : [when],
    lone: false,
    ...(interactive ? { interactive: true, tooltip: recording ? `Recording. ${words.whole}` : words.whole } : {}),
  });
  element.classList.add("k-clock-pill");
  element.toggleAttribute("data-recording", recording);
  if (options.open) element.dataset.open = options.open;
  // A component of its own, not one of the bar's pills: its parts are its own, and its variants
  // are the parts it shows.
  probe(element, { role: "component", name: "ClockPill", variant: variant((options.seal ?? true) && "seal", date !== "none" && `date ${date}`, parts.some((part) => part.classList.contains("k-clock-pill-weather")) && "weather", recording && "recording"), fixed: { width: false, height: true } });
  centreCapsule(element);
  return { element, time, when, weather };
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
 * How far the capsule's middle is off the pill's, as the margins that would put it in the middle of
 * a box its owner centres: the side of the capsule with less beside it takes the difference. A
 * width worked out from the layout, which the CSS can't read.
 */
function centreCapsule(pill: HTMLElement): void {
  new ResizeObserver(() => {
    const zoom = pill.getBoundingClientRect().width / Math.max(1, pill.offsetWidth) || 1;
    const { before, after } = capsuleExtents(pill);
    const spare = Math.round((before - after) / zoom);
    const was = [pill.style.getPropertyValue("--capsule-before"), pill.style.getPropertyValue("--capsule-after")];
    const now = [`${Math.max(0, -spare)}px`, `${Math.max(0, spare)}px`];
    if (was[0] === now[0] && was[1] === now[1]) return;
    pill.style.setProperty("--capsule-before", now[0]!);
    pill.style.setProperty("--capsule-after", now[1]!);
    // An owner that turns these into margins centres the pill again with them, not round the
    // margins it had before.
    forgetCentring(pill);
    if (pill.parentElement) centreOnPixels(pill.parentElement);
  }).observe(pill);
}
