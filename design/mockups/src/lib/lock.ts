// The lock screen: QML's `Lock` (`docs/design/components/lock.md`), a template of the shell's
// alone, drawn on a screen with its frame (`lib/sheet.ts`).
//
// At rest it is the wallpaper and its clock: the digits in `type.clock_rest` under a kicker line
// (where and which day, then the date), placed on the picture where the wallpaper says, and the
// picture's foreground cut-out, where it has one, drawn over the digits so the city stands in front
// of the clock. The digits and the kicker are a light tint of the picture behind the digits, so
// they read as part of it, the kicker lifted off it by a halo of `surface` round each letter.
//
// Awake, the frame slides in round the picture, `space.s8` all round, the picture blurs
// `size.lock_blur` and dims as a panel's desktop does, the clock goes `scale.clock_awake` of the
// way along its flight to the bar's clock, still behind the cut-out, and a Sheet rises from the
// middle of the bottom edge holding the password's Field alone, the reader's mark at its end.
//
// The lock and the desktop are one surface: locking and unlocking play on a still of the desktop
// the lock captured as it locked, drawn inside the lock (its windows, and the bar's pills on the
// frame), and the clock flies between the bar's clock and its own place. As the lock comes up the
// picture pushes in from the desktop's 1:1 to `scale.push` and its cut-out further, to
// `scale.near`, so the city comes off the sky; the plate, the picture with the city painted out,
// fills in what the city uncovers. As the lock goes they ease back to 1:1.
//
// Numbers move it, each a property a move animates, so a frame held on its plot is exactly what
// plays: `--framed` (the frame sliding in), `--barred` (the bar sliding in over its top edge),
// `--wake` (the blur), `--clock-lift` (waking: the clock's first step along its flight to the bar),
// `--dock` (the rest of its flight, into the bar), `--push` (the picture and its city pushing in),
// `--clock-above` (the clock above the frame or behind the city), the sheet's own `--reveal` and
// the dim's `--panel-dim`.
//
// It always draws in the night scheme matugen derives from its wallpaper, whatever the mode.

import "./lock.css";
import { cityOf, plateOf } from "./depth-picture.ts";
import { type NightScheme, paintNight, tintFor } from "./lock-tint.ts";
import { field, fieldRoom } from "./field.ts";
import { icon } from "./icon.ts";
import { container, row } from "./layout.ts";
import { pill } from "./pill.ts";
import { probe, variant } from "./probe.ts";
import { hang, type Screen, screen, type Sheet, sheet } from "./sheet.ts";
import { advance, text, type TypeStyle } from "./text.ts";
import { length, number } from "./tokens.ts";

export type { NightScheme };

/**
 * A wallpaper's depth, each picture the wallpaper's size: its city cut out, and the plate behind
 * it, the city painted out a little past its outline and nothing else, so the plate over the
 * picture is the picture wherever the cut-out doesn't cover it fully.
 */
export interface Depth {
  /** The city cut out, as an alpha picture: only its alpha is drawn, as a mask on the picture. */
  readonly cutout: string;
  /** What is behind the city: the picture's sky and street, painted in where the city stood. */
  readonly plate: string;
}

/** A wallpaper as the lock draws it. */
export interface Wallpaper {
  readonly name: string;
  readonly picture: string;
  /**
   * Its city cut out and the plate behind it: what stands in front of the clock, and what the city
   * uncovers as it pushes in.
   */
  readonly depth?: Depth;
  /** The roles matugen derives from it in its night scheme. */
  readonly night: NightScheme;
  /**
   * Where the rest clock stands on it: the middle of its digits' baseline, as shares of the
   * picture's width and height, so it keeps its place against the city however the screen crops it.
   */
  readonly clock: { readonly x: number; readonly y: number };
  /**
   * Where it stands on a portrait screen, which crops the picture to a tall strip and makes the
   * clock as wide as the strip: higher, in the sky. The wide place when it has none.
   */
  readonly portrait?: { readonly x: number; readonly y: number };
}

/**
 * At rest, awake with the password's sheet up, or the desktop: the still of it the lock plays its
 * locking and unlocking on, the frame and the bar as the shell draws them and the clock in the bar.
 * Or the picture alone, pushed in with its city off the sky: the wallpaper picker's wall.
 */
export type Phase = "rest" | "awake" | "desktop" | "picture";

/**
 * What the sheet shows: waiting for a password or a finger, checking one, after a wrong password,
 * or after a finger the reader doesn't know.
 */
export type Moment = "rest" | "loading" | "error" | "unknown";

/** The kicker over the digits: where and which day, then the date. */
export interface Kicker {
  readonly place: string;
  readonly date: string;
}

/** The bar's clock pill's words: the weekday's kanji and the short date. */
export interface BarClock {
  readonly kanji: string;
  readonly date: string;
}

export interface LockOptions {
  readonly width: string;
  readonly height: string;
  readonly wallpaper: Wallpaper;
  readonly phase: Phase;
  readonly moment?: Moment;
  readonly time?: string;
  /** Whether the time is a 12-hour clock's (1:00 to 12:59) or a 24-hour one's (00:00 to 23:59). */
  readonly twelveHour?: boolean;
  readonly kicker?: Kicker;
  /** What the field's placeholder says after a miss: a wrong password, or an unknown finger. */
  readonly error?: string;
  /** Whether the machine has a fingerprint reader: its mark ends the field. */
  readonly reader?: boolean;
  /** Drawn to be moved by a person, not held still: its numbers ease as its phase changes. */
  readonly live?: boolean;
  /** The pills on the bar's right, as the desktop's bar draws them. */
  readonly bar?: readonly Element[];
  /** The desktop's windows, tiled in the frame. */
  readonly windows?: readonly Element[];
  /** The password's sheet; false for a lock that is only ever at rest: a picture of one. */
  readonly sheet?: boolean;
}

export interface Lock {
  readonly element: HTMLElement;
  readonly screen: Screen;
  readonly sheet: Sheet;
  /** The sheet's content, whose field a live lock swaps. */
  readonly content: HTMLElement;
}

export const TIME = "21:47";
export const KICKER: Kicker = { place: "東京・月曜日", date: "Monday, 2 November" };
export const BAR_CLOCK: BarClock = { kanji: "月", date: "Mon 2 Nov" };

/**
 * The password's field, as `moment` shows it: lit with the keyboard in it unless it is live. Its
 * `lock` has `lock_open` in `primary` laid over it, unseen, for the right password to turn to; with
 * a reader, `fingerprint` ends it in `primary`, or in `error` once a finger is unknown, with its own
 * `lock_open` laid over it for a known finger to turn to. After a miss its placeholder is `error`,
 * what went wrong, until something is typed.
 */
export function lockField(moment: Moment, live = false, error = "", reader = true): HTMLElement {
  const busy = moment === "loading";
  const missed = moment === "error" || moment === "unknown";
  const made = field({
    icon: busy ? "hourglass_top" : "lock",
    placeholder: busy ? "Checking…" : missed ? error : "Password",
    secret: true,
    ...(reader ? { trailing: { icon: "fingerprint", color: "primary" } } : {}),
    ...(moment === "error" ? { state: "Invalid" as const } : {}),
    ...(live ? {} : { pinned: "lit" as const }),
  });
  const shut = made.querySelector<HTMLElement>(".k-field-mark");
  if (shut && !busy) {
    shut.classList.add("k-lock-shut");
    const open = document.createElement("span");
    open.className = "k-field-mark k-lock-open";
    open.append(icon("lock_open", "var(--type-body_large-icon)", "var(--color-primary)"));
    shut.after(open);
  }
  const print = made.querySelector<HTMLElement>(".k-field-trailing");
  if (print && !busy) {
    print.classList.add("k-lock-reader");
    // The CSS mixes the mark's colour from `--lock-reader-missed`, so a miss can animate it.
    print.style.removeProperty("--trailing");
    print.toggleAttribute("data-missed", moment === "unknown");
    const open = document.createElement("span");
    open.className = "k-field-mark k-field-trailing k-lock-reader-open";
    open.append(icon("lock_open", "var(--type-body_large-icon)", "var(--color-primary)"));
    print.after(open);
  }
  return made;
}

/** The sheet's content: the field alone, PanelPage's padding round it. */
function sheetContent(moment: Moment, error: string, live: boolean, reader: boolean): HTMLElement {
  const room = fieldRoom("100%", lockField(moment, live, error, reader));
  room.classList.add("k-lock-field");
  const box = document.createElement("span");
  box.className = "k-lock-sheet";
  box.append(container(room));
  return box;
}

/**
 * The kicker: the place and day in `clock_place`, then ` · ` and the date in `clock_date`. Its
 * colours are the CSS's, as the digits' are: the picture's tint at rest, its roles in the bar.
 */
function kickerLine(kicker: Kicker): HTMLElement {
  const place = text(kicker.place, "clock_place");
  const date = text(` · ${kicker.date}`, "clock_date");
  for (const words of [place, date]) words.style.removeProperty("color");
  place.classList.add("k-lock-place");
  date.classList.add("k-lock-date");
  const line = row(0, "centre", place, date);
  line.classList.add("k-lock-kicker");
  return line;
}

/**
 * The bar's clock, as `modules/bar/Clock.qml` draws it: a pill centred on the bar holding the
 * weekday's kanji in `accent`, `tertiary`, the short date in `body`, `on_surface_variant`, and
 * the time in `body`, `on_surface`, centred in a slot as wide as the widest time its format shows,
 * so the pill keeps its width as the minutes turn.
 */
function barClock(time: string, twelve: boolean, words: BarClock): { pill: HTMLElement; time: HTMLElement } {
  const shown = text(time, "body", "on_surface");
  shown.classList.add("k-lock-bar-time");
  shown.style.setProperty("--slot", String(widestTime("body", twelve)));
  const made = pill({ content: [text(words.kanji, "accent", "tertiary"), text(words.date, "body", "on_surface_variant"), shown], lone: false });
  made.classList.add("k-lock-bar-clock");
  return { pill: made, time: shown };
}

export function lock(options: LockOptions): Lock {
  const moment = options.moment ?? "rest";
  const wallpaper = options.wallpaper;
  const time = options.time ?? TIME;
  const twelve = options.twelveHour ?? false;
  const bar = barClock(time, twelve, BAR_CLOCK);
  const shown = screen({ width: options.width, height: options.height, wallpaper: wallpaper.picture, bar: [bar.pill, ...(options.bar ?? [])] });
  const element = shown.element;
  element.classList.add("k-lock");
  paintNight(element, wallpaper.night);
  element.dataset.phase = options.phase;
  element.toggleAttribute("data-open", options.phase === "awake");
  element.toggleAttribute("data-live", options.live ?? false);

  const picture = element.querySelector<HTMLImageElement>(".k-screen-wallpaper")!;
  const blur = document.createElement("span");
  blur.className = "k-lock-blur";
  picture.after(blur);

  const windows = document.createElement("span");
  windows.className = "k-lock-windows";
  windows.append(...(options.windows ?? []));
  const dim = element.querySelector(".k-screen-dim")!;

  const digits = text(time, "clock_rest");
  digits.classList.add("k-lock-digits");
  // The digits' colour is the CSS's: the picture's tint at rest, `on_surface` in the bar.
  digits.style.removeProperty("color");
  const words = options.kicker ?? KICKER;
  const kicker = kickerLine(words);
  const clock = document.createElement("span");
  clock.className = "k-lock-clock";
  clock.style.setProperty("--digits-natural", String(digitsWidth(time)));
  clock.style.setProperty("--cap-height", String(capHeight()));
  clock.style.setProperty("--bar-natural", String(advance("body", time)));
  clock.style.setProperty("--letters", String([...time].length));
  const placeNatural = advance("clock_place", words.place);
  const dateNatural = advance("clock_date", ` · ${words.date}`);
  clock.style.setProperty("--place-natural", String(placeNatural));
  clock.style.setProperty("--date-natural", String(dateNatural));
  clock.append(kicker, digits);
  dim.after(clock);

  if (wallpaper.depth) {
    const plate = plateOf(wallpaper.depth.plate);
    plate.classList.add("k-lock-plate");
    picture.after(plate);
    const city = cityOf(wallpaper.picture, wallpaper.depth.cutout);
    city.classList.add("k-lock-cutout");
    city.firstElementChild!.classList.add("k-lock-city");
    clock.after(city);
    clock.style.setProperty("--city", `url("${wallpaper.depth.cutout}")`);
  }
  // The desktop's windows stand in front of the city: they are the still's top layer.
  (element.querySelector(".k-lock-cutout") ?? clock).after(windows);

  const content = options.sheet === false ? document.createElement("span") : sheetContent(moment, options.error ?? "", options.live ?? false, options.reader ?? true);
  const rising = sheet({ content, edge: "bottom", ...(options.live ? {} : { reveal: options.phase === "awake" ? 1 : 0 }) });
  if (options.live) {
    for (const part of rising.parts) {
      part.dataset.live = "";
      part.toggleAttribute("data-shown", options.phase === "awake");
    }
  }
  hang(shown, rising);

  // The rest clock's place is the picture's, which only the picture's size and the screen's know:
  // the screen crops the picture as `object-fit: cover` does, the push scales that round the middle
  // of the screen's bottom edge, and the clock follows the sky on the desktop's picture and the
  // pushed one: it stands in front of the sky and behind the city.
  // The clock's tint is the picture's colour under the digits at rest, read off the picture
  // itself. The bar's clock is where the pill's layout puts its time, which only layout knows.
  // Placing reads the layout and writes styles; done for every lock at once in phases (`placeAll`),
  // reads then writes, so a page of many locks is laid out a few times, not once a lock.
  function* place(): Generator<void, void, void> {
    const width = element.clientWidth;
    const height = element.clientHeight;
    const pillWidth = bar.pill.offsetWidth;
    if (width === 0) return;
    // Until the picture has arrived the clock is placed on the screen as if it were the picture.
    const seen = picture.naturalWidth > 0 ? { width: picture.naturalWidth, height: picture.naturalHeight } : { width, height };
    const px = pxOf(element);
    const token = (name: string) => length(name) * px;
    const scale = Math.max(width / seen.width, height / seen.height);
    const flat = { scale, left: (width - seen.width * scale) / 2, top: (height - seen.height * scale) / 2 };
    const pushed = pushedCover(flat, width, height, element.closest("[data-reduced]") ? 1 : number("scale-push"));
    const cap = capHeight();
    // The clock is the screen's share (`scale.lock_clock`, `scale.lock_clock_width`, and on a portrait
    // screen as large as makes the time it shows `scale.lock_clock_portrait` of the width, so it
    // changes size a little as the minutes turn), its kicker the clock's
    // (`scale.lock_date`, `scale.lock_date_gap`, the portrait pair), never under
    // `size.lock_date_least`: `type.clock_rest` and `type.clock_date` are the 1080 tall screen's.
    const portrait = height > width;
    const reference = token("type-clock_rest-size");
    // Each on whole pixels of the screen it is drawn on.
    const whole = (v: number) => Math.round(v / px) * px;
    const restSize = whole(
      portrait
        ? (number("scale-lock_clock_portrait") * width * length("type-clock_rest-size")) / digitsWidth(time)
        : Math.min(number("scale-lock_clock") * height, number("scale-lock_clock_width") * width),
    );
    const grown = restSize / reference;
    const kickerSize = whole(Math.max(token("size-lock_date_least"), number(portrait ? "scale-lock_date_portrait" : "scale-lock_date") * restSize));
    const kickerGrown = kickerSize / token("type-clock_date-size");
    const kickerGap = whole(number(portrait ? "scale-lock_date_gap_portrait" : "scale-lock_date_gap") * restSize);
    const kickerWidth = Math.ceil(placeNatural * kickerGrown * px - 0.001) + Math.ceil(dateNatural * kickerGrown * px - 0.001);
    const natural = digitsWidth(time) * grown;
    const restDigits = Math.ceil(natural * px - 0.001);
    const margin = token("space-s32");
    const half = Math.max(kickerWidth, restDigits) / 2;
    const spot = portrait ? (wallpaper.portrait ?? wallpaper.clock) : wallpaper.clock;
    const restOn = (cover: Cover) => ({
      cx: Math.min(width - margin - half, Math.max(margin + half, cover.left + spot.x * seen.width * cover.scale)),
      top: Math.round(cover.top + spot.y * seen.height * cover.scale - cap * restSize),
    });
    const restFlat = restOn(flat);
    const rest = restOn(pushed);
    clock.style.setProperty("--digits-natural", String(natural));
    clock.style.setProperty("--rest-size", `${restSize}px`);
    clock.style.setProperty("--kicker-size", `${kickerSize}px`);
    clock.style.setProperty("--kicker-line", `${(token("type-clock_date-line") * kickerGrown).toFixed(3)}px`);
    clock.style.setProperty("--kicker-gap", `${kickerGap}px`);
    clock.style.setProperty("--place-natural", String(placeNatural * kickerGrown));
    clock.style.setProperty("--date-natural", String(dateNatural * kickerGrown));
    clock.style.setProperty("--cap-height", String(cap));
    clock.style.setProperty("--bar-natural", String(advance("body", time)));
    clock.style.setProperty("--middle", `${width / 2}px`);
    clock.style.setProperty("--rest-cx-flat", `${restFlat.cx}px`);
    clock.style.setProperty("--rest-top-flat", `${restFlat.top}px`);
    clock.style.setProperty("--rest-cx-pushed", `${rest.cx}px`);
    clock.style.setProperty("--rest-top-pushed", `${rest.top}px`);
    clock.style.setProperty("--city-flat-left", `${flat.left}px`);
    clock.style.setProperty("--city-flat-top", `${flat.top}px`);
    clock.style.setProperty("--city-flat-width", `${seen.width * flat.scale}px`);
    clock.style.setProperty("--city-flat-height", `${seen.height * flat.scale}px`);
    bar.time.style.setProperty("--slot", String(widestTime("body", twelve)));
    bar.pill.style.setProperty("--pill-width", `${pillWidth}px`);
    const mark = document.createElement("span");
    mark.className = "k-lock-baseline";
    bar.time.append(mark);
    // The kicker stands on its baseline, which only the font knows: the date's, its Latin words.
    const kickerMark = document.createElement("span");
    kickerMark.className = "k-lock-baseline";
    const date = kicker.querySelector(".k-lock-date")!;
    date.append(kickerMark);
    yield;
    const measured = measureBarTime(element, bar, mark);
    const zoom = element.offsetWidth / (element.getBoundingClientRect().width || 1);
    const kickerBase = (kickerMark.getBoundingClientRect().bottom - kicker.getBoundingClientRect().top) * zoom;
    yield;
    mark.remove();
    kickerMark.remove();
    if (measured) {
      clock.style.setProperty("--bar-cx", measured.cx);
      clock.style.setProperty("--bar-base", measured.base);
    }
    if (kickerBase > 0) clock.style.setProperty("--kicker-base", `${kickerBase.toFixed(3)}px`);

    const restDigitsBox = { x: rest.cx - restDigits / 2, y: rest.top, width: restDigits, height: cap * restSize };
    const tint = tintFor(picture, restDigitsBox, pushed);
    if (tint) clock.style.setProperty("--digits-tint", tint);
    else clock.style.removeProperty("--digits-tint");
  }
  placers.set(element, place);
  placers.set(bar.pill, place);
  watch.observe(element);
  watch.observe(bar.pill);
  picture.addEventListener("load", () => later(place));
  void document.fonts.ready.then(() => later(place));

  probe(element, { role: "component", name: "Lock", variant: variant(options.phase, wallpaper.depth && "cutout"), fixed: { width: false, height: false, holds: true } });
  return { element, screen: shown, sheet: rising, content };
}

/**
 * Where the bar's time is, for the clock to land on: its middle across the screen, and its
 * baseline (`mark`'s bottom) under the pill's top, read off the pill's layout. The pill is centred
 * on the bar's band, so CSS places it from `--bar-shift` as the band slides.
 */
function measureBarTime(element: HTMLElement, bar: { pill: HTMLElement; time: HTMLElement }, mark: HTMLElement): { cx: string; base: string } | null {
  const screenBox = element.getBoundingClientRect();
  if (screenBox.width === 0) return null;
  // The screen may be drawn zoomed to fit a stage: its layout's pixels, not the viewport's.
  const zoom = element.offsetWidth / screenBox.width;
  const timeBox = bar.time.getBoundingClientRect();
  const pillBox = bar.pill.getBoundingClientRect();
  const baseline = mark.getBoundingClientRect().bottom;
  return { cx: `${((timeBox.left + timeBox.width / 2 - screenBox.left) * zoom).toFixed(3)}px`, base: `${((baseline - pillBox.top) * zoom).toFixed(3)}px` };
}

/** Each lock's placing, by the elements whose size moves it. */
const placers = new WeakMap<Element, () => Generator<void, void, void>>();

/** Places every lock in `jobs` together: each runs to its next phase before any runs further. */
function placeAll(jobs: Iterable<() => Generator<void, void, void>>): void {
  let running = [...new Set(jobs)].map((job) => job());
  while (running.length > 0) running = running.filter((step) => !step.next().done);
}

const watch = new ResizeObserver((entries) => placeAll(entries.flatMap((entry) => placers.get(entry.target) ?? [])));

const waiting = new Set<() => Generator<void, void, void>>();

/** Places `job`'s lock on the next frame, with every other waiting there. */
function later(job: () => Generator<void, void, void>): void {
  if (waiting.size === 0) {
    requestAnimationFrame(() => {
      const jobs = [...waiting];
      waiting.clear();
      placeAll(jobs);
    });
  }
  waiting.add(job);
}

/** Where `object-fit: cover` draws a picture on a screen: its scale and its top left corner. */
interface Cover {
  readonly scale: number;
  readonly left: number;
  readonly top: number;
}

/**
 * `cover` pushed in by `push` round the middle of the bottom edge of a screen `width` by `height`,
 * as the CSS scales it.
 */
function pushedCover(cover: Cover, width: number, height: number, push: number): Cover {
  return { scale: cover.scale * push, left: width / 2 + (cover.left - width / 2) * push, top: height + (cover.top - height) * push };
}

const widest = new Map<string, number>();

/**
 * The widest time a clock can show in `style`, in logical pixels: every time of its day measured on
 * the font, 00:00 to 23:59, or 1:00 to 12:59 for a 12-hour clock. The figures are proportional, so
 * no one digit is the widest in every place.
 */
export function widestTime(style: TypeStyle, twelve: boolean): number {
  const key = `${style}|${twelve ? "12" : "24"}`;
  const known = widest.get(key);
  if (known !== undefined) return known;
  const hours = twelve ? Array.from({ length: 12 }, (_, h) => String(h + 1)) : Array.from({ length: 24 }, (_, h) => String(h).padStart(2, "0"));
  let most = 0;
  for (const hour of hours) {
    for (let minute = 0; minute < 60; minute += 1) most = Math.max(most, advance(style, `${hour}:${String(minute).padStart(2, "0")}`));
  }
  if (document.fonts.status === "loaded") widest.set(key, most);
  return most;
}

/** The CSS pixels a logical pixel is in `element`'s box: its scale. */
function pxOf(element: HTMLElement): number {
  return Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || 1;
}

/**
 * The digits' width in `type.clock_rest`, in logical pixels, read off the font as it lays them
 * out: the canvas `advance` measures with can't draw a font this large at its precision.
 */
const digitWidths = new Map<string, number>();

function digitsWidth(time: string): number {
  const known = digitWidths.get(time);
  if (known !== undefined) return known;
  const meter = document.createElement("span");
  meter.className = "k-lock-meter";
  meter.textContent = time;
  document.documentElement.append(meter);
  const width = meter.getBoundingClientRect().width;
  meter.remove();
  if (document.fonts.status === "loaded") digitWidths.set(time, width);
  return width;
}

let cap: number | null = null;

/**
 * Geist Light's cap height as a share of its size: the digits' box is trimmed to it and the
 * baseline (`text-box`), so a digit's top is its box's, which only the font knows.
 */
function capHeight(): number {
  if (cap !== null) return cap;
  const meter = document.createElement("span");
  meter.className = "k-lock-meter";
  meter.textContent = "0";
  document.documentElement.append(meter);
  const measured = meter.getBoundingClientRect().height / length("type-clock_rest-size");
  meter.remove();
  if (measured > 0 && document.fonts.status === "loaded") cap = measured;
  return measured;
}
