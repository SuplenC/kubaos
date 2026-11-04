// The lock screen: QML's `Lock` (`docs/design/components/lock.md`), a template of the shell's
// alone, drawn on a screen with its frame (`lib/sheet.ts`).
//
// At rest it is the wallpaper and its clock: the digits in `type.clock_rest` under a kicker line
// (where and which day, then the date), placed on the picture where the wallpaper says, and the
// picture's foreground cut-out, where it has one, drawn over the digits so the city stands in front
// of the clock. Behind the clock lies a soft shade of `surface`, as strong as this picture needs
// for its words to hold 4.5:1, and no stronger.
//
// Awake, the frame closes in round the picture, the picture blurs `size.lock_blur` and dims as a
// panel's desktop does, the clock lifts out from behind the cut-out to `type.clock` near the top,
// and a Sheet rises from the middle of the bottom edge holding the password's Field, the other way
// in, and the keys. Three numbers move it, each 0 at rest and 1 awake, so a move animates them and
// a frame held on its plot is exactly what plays: `--framed` (the frame), `--wake` (the blur) and
// `--lift` (the clock), with the sheet's own `--reveal` and the dim's `--panel-dim`.
//
// It always draws in the night scheme matugen derives from its wallpaper, whatever the mode.

import "./lock.css";
import { type NightScheme, paintNight, shadeFor } from "./lock-shade.ts";
import { field, fieldRoom, note } from "./field.ts";
import { icon } from "./icon.ts";
import { keyHints } from "./keycap.ts";
import { container, gap, row, stack } from "./layout.ts";
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

/** At rest, or awake with the password's sheet up. */
export type Phase = "rest" | "awake";

/** What the sheet shows: waiting for a password, checking one, or after a miss. */
export type Moment = "rest" | "loading" | "error";

/** The kicker over the digits: where and which day in `primary`, then the date. */
export interface Kicker {
  readonly place: string;
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

/** The password's field, as `moment` shows it: lit with the keyboard in it unless it is live. */
export function lockField(moment: Moment, live = false): HTMLElement {
  const busy = moment === "loading";
  return field({
    icon: busy ? "hourglass_top" : "lock",
    placeholder: busy ? "Checking…" : "Password",
    secret: true,
    ...(moment === "error" ? { state: "Invalid" as const } : {}),
    ...(live ? {} : { pinned: "lit" as const }),
  });
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

export function lock(options: LockOptions): Lock {
  const moment = options.moment ?? "rest";
  const wallpaper = options.wallpaper;
  const shown = screen({ width: options.width, height: options.height, wallpaper: wallpaper.picture });
  const element = shown.element;
  element.classList.add("k-lock");
  const colours = paintNight(element, wallpaper.night);
  element.toggleAttribute("data-awake", options.phase === "awake");
  element.toggleAttribute("data-open", options.phase === "awake");
  element.toggleAttribute("data-live", options.live ?? false);
  element.toggleAttribute("data-cutout", wallpaper.cutout !== undefined);

  const picture = element.querySelector<HTMLImageElement>(".k-screen-wallpaper")!;
  const blur = document.createElement("span");
  blur.className = "k-lock-blur";
  picture.after(blur);

  const time = options.time ?? TIME;
  const digits = text(time, "clock_rest");
  digits.classList.add("k-lock-digits");
  const kicker = kickerLine(options.kicker ?? KICKER);
  const clock = document.createElement("span");
  clock.className = "k-lock-clock";
  clock.style.setProperty("--digits-natural", String(digitsWidth(time)));
  clock.append(kicker, digits);
  const dim = element.querySelector(".k-screen-dim")!;
  dim.after(clock);

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
  // the screen crops the picture as `object-fit: cover` does, and the clock follows the city. Its
  // shade's strength is the picture's colours under it, read off the picture itself.
  const place = () => {
    const width = element.clientWidth;
    const height = element.clientHeight;
    if (width === 0) return;
    // Until the picture has arrived the clock is placed on the screen as if it were the picture.
    const shown = picture.naturalWidth > 0 ? { width: picture.naturalWidth, height: picture.naturalHeight } : { width, height };
    const px = pxOf(element);
    const token = (name: string) => length(name) * px;
    const scale = Math.max(width / shown.width, height / shown.height);
    const cover = { scale, left: (width - shown.width * scale) / 2, top: (height - shown.height * scale) / 2 };
    const cap = capHeight();
    const kickerHeight = kicker.offsetHeight;
    const kickerWidth = kicker.offsetWidth;
    const above = kickerHeight + token("space-s16");
    const natural = digitsWidth(time);
    clock.style.setProperty("--digits-natural", String(natural));
    const ratio = length("type-clock-size") / length("type-clock_rest-size");
    const rest = { width: Math.max(kickerWidth, Math.ceil(natural * px - 0.001)), height: above + cap * token("type-clock_rest-size") };
    const awake = { width: Math.max(kickerWidth, Math.ceil(natural * ratio * px - 0.001)), height: above + cap * token("type-clock-size") };
    const margin = token("space-s32");
    const half = rest.width / 2;
    const cx = Math.min(width - margin - half, Math.max(margin + half, cover.left + wallpaper.clock.x * shown.width * scale));
    const baseline = cover.top + wallpaper.clock.y * shown.height * scale;
    const restTop = Math.round(baseline - rest.height);
    const awakeTop = token("size-bar") + token("space-s48");
    clock.style.setProperty("--kicker-width", `${kickerWidth}px`);
    clock.style.setProperty("--rest-cx", `${cx}px`);
    clock.style.setProperty("--rest-top", `${restTop}px`);
    clock.style.setProperty("--shade-rest", String(shadeFor(picture, colours, { x: cx - half, y: restTop, width: rest.width, height: rest.height }, cover)));
    clock.style.setProperty("--shade-awake", String(shadeFor(picture, colours, { x: (width - awake.width) / 2, y: awakeTop, width: awake.width, height: awake.height }, cover, { blur: token("size-lock_blur"), dim: number("opacity-panel_scrim") })));
  };
  const watch = new ResizeObserver(place);
  watch.observe(element);
  watch.observe(kicker);
  picture.addEventListener("load", place);
  void document.fonts.ready.then(place);

  probe(element, { role: "component", name: "Lock", variant: variant(options.phase, wallpaper.cutout && "cutout"), fixed: { width: false, height: false, holds: true } });
  return { element, screen: shown, sheet: rising, content };
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
  const measured = meter.offsetHeight / length("type-clock_rest-size");
  meter.remove();
  if (measured > 0 && document.fonts.status === "loaded") cap = measured;
  return measured;
}
