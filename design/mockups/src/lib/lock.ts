// The lock screen: QML's `Lock` (`docs/design/components/lock.md`), a template of the shell's
// alone, drawn on a screen with its frame (`lib/sheet.ts`).
//
// At rest it is the wallpaper and its clock: the digits in `type.clock_rest` under a kicker line
// (where and which day, then the date), placed on the picture where the wallpaper says, and the
// picture's foreground cut-out, where it has one, drawn over the digits so the city stands in front
// of the clock. The digits are a light tint of the picture behind them, so they read as part of it.
// Behind the kicker a shade of `surface` tight to it, as strong as this picture needs for its words
// to hold 4.5:1 and no stronger.
//
// Awake, the frame closes in round the picture, `space.s8` all round, the picture blurs
// `size.lock_blur` and dims as a panel's desktop does, the clock lifts out from behind the cut-out
// to `type.clock` near the top, and a Sheet rises from the middle of the bottom edge holding the
// password's Field, the other way in, and the keys.
//
// The lock and the desktop are one surface: locking and unlocking play on a still of the desktop
// the lock captured as it locked, drawn inside the lock (its windows, and the bar's pills on the
// frame), and the clock flies between the bar's clock and its own place.
//
// Numbers move it, each a property a move animates, so a frame held on its plot is exactly what
// plays: `--framed` (the frame closing in), `--barred` (its top edge from `space.s8` to the bar's),
// `--wake` (the blur), `--lift` (the clock from rest to awake), `--dock` (the clock into the bar),
// the sheet's own `--reveal` and the dim's `--panel-dim`.
//
// It always draws in the night scheme matugen derives from its wallpaper, whatever the mode.

import "./lock.css";
import { backingCover, KICKER_WORDS, type NightScheme, paintNight, shadeFor, tintFor } from "./lock-shade.ts";
import { field, fieldRoom, note } from "./field.ts";
import { icon } from "./icon.ts";
import { keyHints } from "./keycap.ts";
import { container, gap, row, stack } from "./layout.ts";
import { pill } from "./pill.ts";
import { probe, variant } from "./probe.ts";
import { hang, type Screen, screen, type Sheet, sheet } from "./sheet.ts";
import { text } from "./text.ts";
import { length, number } from "./tokens.ts";

export type { NightScheme };

/** A wallpaper as the lock draws it. */
export interface Wallpaper {
  readonly name: string;
  readonly picture: string;
  /** Its foreground cut out, the same size as the picture: what stands in front of the clock. */
  readonly cutout?: string;
  /** The roles matugen derives from it in its night scheme. */
  readonly night: NightScheme;
  /**
   * Where the rest clock stands on it: the middle of its digits' baseline, as shares of the
   * picture's width and height, so it keeps its place against the city however the screen crops it.
   */
  readonly clock: { readonly x: number; readonly y: number };
}

/**
 * At rest, awake with the password's sheet up, or the desktop: the still of it the lock plays its
 * locking and unlocking on, the frame and the bar as the shell draws them and the clock in the bar.
 */
export type Phase = "rest" | "awake" | "desktop";

/** What the sheet shows: waiting for a password, checking one, or after a miss. */
export type Moment = "rest" | "loading" | "error";

/** The kicker over the digits: where and which day in `primary`, then the date. */
export interface Kicker {
  readonly place: string;
  readonly date: string;
}

/** The bar's clock pill's words: the weekday's kanji, the short date, and the time. */
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
  readonly kicker?: Kicker;
  /** What the line under the field says after a miss. */
  readonly error?: string;
  /** Drawn to be moved by a person, not held still: its numbers ease as its phase changes. */
  readonly live?: boolean;
  /** The pills on the bar's right, as the desktop's bar draws them. */
  readonly bar?: readonly Element[];
  /** The desktop's windows, tiled in the frame. */
  readonly windows?: readonly Element[];
}

export interface Lock {
  readonly element: HTMLElement;
  readonly screen: Screen;
  readonly sheet: Sheet;
  /** The sheet's content, whose field and line a live lock swaps. */
  readonly content: HTMLElement;
}

export const TIME = "21:47";
export const KICKER: Kicker = { place: "東京・月曜日", date: "Monday, 2 November" };
export const BAR_CLOCK: BarClock = { kanji: "月", date: "Mon 2 Nov" };

/**
 * The password's field, as `moment` shows it: lit with the keyboard in it unless it is live. Its
 * `lock` has `lock_open` in `primary` laid over it, unseen, for the right password to turn to.
 */
export function lockField(moment: Moment, live = false): HTMLElement {
  const busy = moment === "loading";
  const made = field({
    icon: busy ? "hourglass_top" : "lock",
    placeholder: busy ? "Checking…" : "Password",
    secret: true,
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
  return made;
}

/** The line under the field: the other way in, or what went wrong. */
export function lockLine(moment: Moment, error: string): HTMLElement {
  if (moment === "error") return note(error, true);
  return row("s8", "centre", icon("fingerprint", "var(--type-small-icon)", "var(--color-primary)"), text("Or touch the fingerprint reader", "small", "on_surface_variant"));
}

/** The sheet's content: the field, its line under it, and the keys, PanelPage's padding round them. */
function sheetContent(moment: Moment, error: string, live: boolean): HTMLElement {
  const room = fieldRoom("100%", lockField(moment, live));
  room.classList.add("k-lock-field");
  const body = stack(0, "start", room, gap("s4"), lockLine(moment, error), gap("s16"), keyHints([["↵", "Unlock"], ["Esc", "Rest"]]));
  body.classList.add("k-lock-body");
  const box = document.createElement("span");
  box.className = "k-lock-sheet";
  box.append(container(body));
  return box;
}

/** The kicker: the place and day in `primary`, then ` · ` and the date in `on_surface_variant`. */
function kickerLine(kicker: Kicker): HTMLElement {
  const line = row(0, "centre", text(kicker.place, "accent", "primary"), text(` · ${kicker.date}`, "body_strong", "on_surface_variant"));
  line.classList.add("k-lock-kicker");
  return line;
}

/**
 * The bar's clock, as `modules/bar/Clock.qml` draws it: a pill centred on the bar holding the
 * weekday's kanji in `accent`, `tertiary`, the short date in `body`, `on_surface_variant`, and
 * the time in `body_number`, `on_surface`.
 */
function barClock(time: string, words: BarClock): { pill: HTMLElement; time: HTMLElement } {
  const shown = text(time, "body_number", "on_surface");
  shown.classList.add("k-lock-bar-time");
  const made = pill({ content: [text(words.kanji, "accent", "tertiary"), text(words.date, "body", "on_surface_variant"), shown], lone: false });
  made.classList.add("k-lock-bar-clock");
  return { pill: made, time: shown };
}

export function lock(options: LockOptions): Lock {
  const moment = options.moment ?? "rest";
  const wallpaper = options.wallpaper;
  const time = options.time ?? TIME;
  const bar = barClock(time, BAR_CLOCK);
  const shown = screen({ width: options.width, height: options.height, wallpaper: wallpaper.picture, bar: [bar.pill, ...(options.bar ?? [])] });
  const element = shown.element;
  element.classList.add("k-lock");
  const colours = paintNight(element, wallpaper.night);
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
  dim.after(windows);

  const digits = text(time, "clock_rest");
  digits.classList.add("k-lock-digits");
  // The digits' colour is the CSS's: the picture's tint at rest, `on_surface` awake and in the bar.
  digits.style.removeProperty("color");
  const kicker = kickerLine(options.kicker ?? KICKER);
  const shade = document.createElement("span");
  shade.className = "k-lock-shade";
  const clock = document.createElement("span");
  clock.className = "k-lock-clock";
  clock.style.setProperty("--digits-natural", String(digitsWidth(time)));
  clock.style.setProperty("--cap-height", String(capHeight()));
  clock.append(shade, kicker, digits);
  windows.after(clock);

  let cutout: HTMLImageElement | null = null;
  if (wallpaper.cutout) {
    cutout = document.createElement("img");
    cutout.className = "k-lock-cutout";
    cutout.src = wallpaper.cutout;
    cutout.alt = "";
    clock.after(cutout);
  }

  const content = sheetContent(moment, options.error ?? "", options.live ?? false);
  const rising = sheet({ content, edge: "bottom", ...(options.live ? {} : { reveal: options.phase === "awake" ? 1 : 0 }) });
  if (options.live) {
    for (const part of rising.parts) {
      part.dataset.live = "";
      part.toggleAttribute("data-shown", options.phase === "awake");
    }
  }
  hang(shown, rising);

  // The rest clock's place is the picture's, which only the picture's size and the screen's know:
  // the screen crops the picture as `object-fit: cover` does, and the clock follows the city. The
  // digits' tint and the shade's strength are the picture's colours under the digits and the
  // kicker, read off the picture itself. The bar's clock is where the pill's layout puts its time,
  // which only layout knows.
  const place = () => {
    const width = element.clientWidth;
    const height = element.clientHeight;
    if (width === 0) return;
    // Until the picture has arrived the clock is placed on the screen as if it were the picture.
    const seen = picture.naturalWidth > 0 ? { width: picture.naturalWidth, height: picture.naturalHeight } : { width, height };
    const px = pxOf(element);
    const token = (name: string) => length(name) * px;
    const scale = Math.max(width / seen.width, height / seen.height);
    const cover = { scale, left: (width - seen.width * scale) / 2, top: (height - seen.height * scale) / 2 };
    const cap = capHeight();
    const kickerHeight = kicker.offsetHeight;
    const kickerWidth = kicker.offsetWidth;
    const natural = digitsWidth(time);
    const restDigits = Math.ceil(natural * px - 0.001);
    const margin = token("space-s32");
    const half = Math.max(kickerWidth, restDigits) / 2;
    const cx = Math.min(width - margin - half, Math.max(margin + half, cover.left + wallpaper.clock.x * seen.width * scale));
    const restBase = cover.top + wallpaper.clock.y * seen.height * scale;
    const restTop = Math.round(restBase - cap * token("type-clock_rest-size"));
    const above = kickerHeight + token("space-s16");
    const awakeTop = token("space-s8") + token("space-s48") + above;
    clock.style.setProperty("--digits-natural", String(natural));
    clock.style.setProperty("--cap-height", String(cap));
    clock.style.setProperty("--kicker-width", `${kickerWidth}px`);
    clock.style.setProperty("--kicker-height", `${kickerHeight}px`);
    clock.style.setProperty("--rest-cx", `${cx}px`);
    clock.style.setProperty("--rest-top", `${restTop}px`);
    placeBarTime(element, bar, clock);

    const restDigitsBox = { x: cx - restDigits / 2, y: restTop, width: restDigits, height: cap * token("type-clock_rest-size") };
    const tint = tintFor(picture, restDigitsBox, cover);
    if (tint) clock.style.setProperty("--digits-tint", tint);
    else clock.style.removeProperty("--digits-tint");

    const kickerBox = (top: number, middle: number) => ({ x: middle - kickerWidth / 2, y: top - above, width: kickerWidth, height: kickerHeight });
    const dimmed = { blur: token("size-lock_blur"), dim: number("opacity-panel_scrim") };
    // The shade is a box blurred past the kicker, so its opacity is raised by what the blur leaves
    // at the kicker's corners: the words hold 4.5:1 where it is weakest.
    const left = backingCover({ x: token("space-s16"), y: token("space-s12") }, token("space-s8"));
    const shade = (needed: number) => String(Math.min(1, Math.ceil((needed / left) * 100) / 100));
    clock.style.setProperty("--shade-rest", shade(shadeFor(picture, colours, KICKER_WORDS, kickerBox(restTop, cx), cover)));
    clock.style.setProperty("--shade-awake", shade(shadeFor(picture, colours, KICKER_WORDS, kickerBox(awakeTop, width / 2), cover, dimmed)));
  };
  const watch = new ResizeObserver(place);
  watch.observe(element);
  watch.observe(kicker);
  watch.observe(bar.pill);
  picture.addEventListener("load", place);
  void document.fonts.ready.then(place);

  probe(element, { role: "component", name: "Lock", variant: variant(options.phase, wallpaper.cutout && "cutout"), fixed: { width: false, height: false, holds: true } });
  return { element, screen: shown, sheet: rising, content };
}

/**
 * Where the bar's time is, for the clock to land on: its middle across the screen, and its
 * baseline under the pill's top, read off the pill's layout. The pill is centred in the frame's
 * top edge, so CSS places it from `--frame-top` as the edge grows.
 */
function placeBarTime(element: HTMLElement, bar: { pill: HTMLElement; time: HTMLElement }, clock: HTMLElement): void {
  bar.pill.style.setProperty("--pill-width", `${bar.pill.offsetWidth}px`);
  const screenBox = element.getBoundingClientRect();
  if (screenBox.width === 0) return;
  // The screen may be drawn zoomed to fit a stage: its layout's pixels, not the viewport's.
  const zoom = element.offsetWidth / screenBox.width;
  const timeBox = bar.time.getBoundingClientRect();
  const pillBox = bar.pill.getBoundingClientRect();
  const mark = document.createElement("span");
  mark.className = "k-lock-baseline";
  bar.time.append(mark);
  const baseline = mark.getBoundingClientRect().bottom;
  mark.remove();
  clock.style.setProperty("--bar-cx", `${((timeBox.left + timeBox.width / 2 - screenBox.left) * zoom).toFixed(3)}px`);
  clock.style.setProperty("--bar-base", `${((baseline - pillBox.top) * zoom).toFixed(3)}px`);
}

/** The CSS pixels a logical pixel is in `element`'s box: its scale. */
function pxOf(element: HTMLElement): number {
  return Number.parseFloat(getComputedStyle(element).getPropertyValue("--scale")) || 1;
}

/**
 * The digits' width in `type.clock_rest`, in logical pixels, read off the font as it lays them
 * out: the canvas `advance` measures with can't draw a font this large at its precision.
 */
function digitsWidth(time: string): number {
  const meter = document.createElement("span");
  meter.className = "k-lock-meter";
  meter.textContent = time;
  document.documentElement.append(meter);
  const width = meter.getBoundingClientRect().width;
  meter.remove();
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
