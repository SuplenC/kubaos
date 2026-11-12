// The lock screen: QML's `Lock` (`docs/design/components/lock.md`), a template of the shell's
// alone, drawn on a screen with its frame (`lib/sheet.ts`).
//
// At rest it is the wallpaper and its clock: the digits in `type.clock_rest` under a kicker line
// (where and which day, then the date), centred on the screen with their baseline `scale.lock_clock_base`
// of its height down (the same place on every wallpaper), and the picture's foreground cut-out, where it has one, drawn over the digits so the city stands in front
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
}

/**
 * A part of the picture that can stand in front of the clock, as an alpha picture the picture's
 * size: the city's cut-out, which pushes in to `scale.near` with the city, or a band of the
 * picture's own depth, which stays with the picture at `scale.push`.
 */
export interface Layer {
  readonly mask: string;
  /** Drawn by the city layer, ahead of the picture, rather than by the picture itself. */
  readonly near: boolean;
}

/** Where the rest clock stands, in shares of the screen it is drawn on. */
export interface ClockPlace {
  /** The digits' middle across the screen's width. */
  readonly x: number;
  /** The digits' baseline down the screen's height. */
  readonly y: number;
  /** The clock's type size, a share of the screen's shorter side. */
  readonly s: number;
}

export type DateAlign = "left" | "centre" | "right";

/** The kicker over the digits, in shares of the clock's size, and its edge on the digits' ink. */
export interface ClockDate {
  readonly size: number;
  readonly gap: number;
  readonly align: DateAlign;
}

/** A soft glow under the digits and the kicker: its colour and how strong it is, 0 to 1. */
export interface ClockGlow {
  readonly colour: string;
  readonly strength: number;
}

/**
 * The rest clock as the wallpaper picker sets it, each part the lock's own when left out: the
 * standard place, the tokens' kicker, the picture's tint, no glow, and the city's cut-out in front.
 */
export interface LockClock {
  readonly place?: ClockPlace;
  readonly date?: ClockDate;
  /** A CSS colour for the digits and the kicker, in place of the picture's tint. */
  readonly colour?: string;
  readonly glow?: ClockGlow;
  /** What stands in front of the clock, nearest first; none, and it stands in front of it all. */
  readonly layers?: readonly Layer[];
  /** False: no clock and no date at rest, the picture alone; the bar's time comes and goes with the bar. */
  readonly showClock?: boolean;
  /** False: the digits with no date over them. */
  readonly showDate?: boolean;
}

/**
 * The rest clock drawn on a screen, in that screen's CSS pixels (the natural widths in logical
 * pixels, as the CSS reads them): what a move of the clock eases between.
 */
export interface ClockAt {
  readonly cx: number;
  /** The digits' cap top. */
  readonly top: number;
  readonly size: number;
  readonly kickerSize: number;
  readonly kickerLine: number;
  readonly kickerGap: number;
  /** The kicker's edge on the digits' ink: 0 its left on their left, 0.5 its middle, 1 its right. */
  readonly align: number;
  readonly digits: number;
  readonly place: number;
  readonly date: number;
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
  /**
   * The rest clock as set for this picture: with one, the kicker stands on the digits' ink and
   * the clock in front of the city, cut by each layer in front of it.
   */
  readonly clock?: LockClock;
}

export interface Lock {
  readonly element: HTMLElement;
  readonly screen: Screen;
  readonly sheet: Sheet;
  /** The sheet's content, whose field a live lock swaps. */
  readonly content: HTMLElement;
  /** Sets the rest clock, and places it again. */
  setClock(clock: LockClock): void;
  /**
   * Held, the lock leaves its clock where its owner draws it (`drawClock`) rather than placing it
   * itself; it still reads the bar's time and the kicker's baseline as it is resized.
   */
  hold(held: boolean): void;
  /** Where the rest clock stands with `clock` on this lock drawn `width` by `height` CSS pixels. */
  clockAt(clock: LockClock, width: number, height: number): ClockAt;
  /** The picture's tint under the digits of `at`, on a screen `width` by `height`: null until it has arrived. */
  tintAt(at: ClockAt, width: number, height: number): string | null;
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
  // Their colours, sizes and lines are the CSS's: they scale with the clock.
  for (const words of [place, date]) for (const property of ["color", "--t-size", "--t-line"]) words.style.removeProperty(property);
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

  // The rest clock stands in the screen's place, the same on every wallpaper, in front of the sky
  // and behind the city; the city's cut-out clips the digits wherever it happens to cross them.
  // A picture's own clock (`options.clock`) stands where it was set instead, and in front of the
  // city, cut by every layer set in front of it. The clock's tint is the picture's colour under
  // the digits at rest, read off the picture itself. The bar's clock is where the pill's layout
  // puts its time, which only layout knows. Placing reads the layout and writes styles; done for
  // every lock at once in phases (`placeAll`), reads then writes, so a page of many locks is laid
  // out a few times, not once a lock.
  let own: LockClock | undefined = options.clock;
  let held = false;
  if (own) clockOn(element, clock, own);

  /** How the screen crops the picture, flat and pushed in, on a screen `width` by `height`. */
  const covers = (width: number, height: number) => {
    // Until the picture has arrived the clock is placed on the screen as if it were the picture.
    const seen = picture.naturalWidth > 0 ? { width: picture.naturalWidth, height: picture.naturalHeight } : { width, height };
    const scale = Math.max(width / seen.width, height / seen.height);
    const flat = { scale, left: (width - seen.width * scale) / 2, top: (height - seen.height * scale) / 2 };
    return { seen, flat, pushed: pushedCover(flat, width, height, element.closest("[data-reduced]") ? 1 : number("scale-push")) };
  };

  /** Where the clock stands with `set` on a screen `width` by `height`, in its CSS pixels. */
  const at = (set: LockClock | undefined, width: number, height: number): ClockAt => {
    const px = pxOf(element);
    const token = (name: string) => length(name) * px;
    const portrait = height > width;
    const reference = token("type-clock_rest-size");
    const margin = token("space-s32");
    // Each on whole pixels of the screen it is drawn on.
    const whole = (v: number) => Math.floor(v / px + 0.5) * px;
    const cap = capHeight();
    // The clock is the screen's share (`scale.lock_clock`, `scale.lock_clock_width`; on a portrait
    // screen sized from the width, below), its kicker the clock's (`scale.lock_date`,
    // `scale.lock_date_gap`, the portrait pair), never under `size.lock_date_least`:
    // `type.clock_rest` and `type.clock_date` are the 1080 tall screen's. On a portrait screen the
    // clock's size is set once from the width: the widest time it can show fills the width less
    // `space.s32` either side, so no time ever leaves the screen and the size never changes as the
    // minutes turn; a narrower time is simply drawn narrower, as on a wide one.
    const standard = portrait
      ? Math.floor(((width - 2 * margin) * length("type-clock_rest-size")) / widestRest(twelve) / px) * px
      : whole(Math.min(number("scale-lock_clock") * height, number("scale-lock_clock_width") * width));
    // A clock set by hand keeps its exact share, so a drag or a move between screens is smooth.
    const size = set?.place ? set.place.s * Math.min(width, height) : standard;
    const grown = size / reference;
    const least = token("size-lock_date_least");
    let kickerSize: number;
    let kickerGap: number;
    if (set?.date) {
      kickerSize = Math.max(least, set.date.size * size);
      kickerGap = set.date.gap * size;
    } else {
      // The kicker, the clock's share, shrunk to fit the width less `space.s32` either side should
      // a long date need it, and never under `size.lock_date_least`.
      const wanted = Math.max(least, number(portrait ? "scale-lock_date_portrait" : "scale-lock_date") * size);
      const wantedWidth = ((placeNatural + dateNatural) * wanted * px) / token("type-clock_date-size");
      kickerSize = whole(Math.max(least, wantedWidth > width - 2 * margin ? (wanted * (width - 2 * margin)) / wantedWidth : wanted));
      kickerGap = whole(number(portrait ? "scale-lock_date_gap_portrait" : "scale-lock_date_gap") * size);
    }
    const kickerGrown = kickerSize / token("type-clock_date-size");
    // One place on every wallpaper, the screen's, not the picture's: centred, its digits' baseline
    // `scale.lock_clock_base` of the screen's height down (`scale.lock_clock_base_portrait` on a
    // portrait screen); or the place set for this picture.
    const base = set?.place ? set.place.y * height : number(portrait ? "scale-lock_clock_base_portrait" : "scale-lock_clock_base") * height;
    const cx = set?.place ? set.place.x * width : width / 2;
    return {
      cx,
      top: set?.place ? base - cap * size : Math.round(base - cap * size),
      size,
      kickerSize,
      kickerLine: token("type-clock_date-line") * kickerGrown,
      kickerGap,
      align: ALIGN[set?.date?.align ?? "centre"],
      digits: digitsWidth(time) * grown,
      place: placeNatural * kickerGrown,
      date: dateNatural * kickerGrown,
    };
  };

  /** The picture's tint under the digits of `where`, as they stand at rest: pushed in. */
  const tintAt = (where: ClockAt, width: number, height: number): string | null => {
    const digitsBox = Math.ceil(where.digits * pxOf(element) - 0.001);
    return tintFor(picture, { x: where.cx - digitsBox / 2, y: where.top, width: digitsBox, height: capHeight() * where.size }, covers(width, height).pushed);
  };

  function* place(): Generator<void, void, void> {
    const width = element.clientWidth;
    const height = element.clientHeight;
    const pillWidth = bar.pill.offsetWidth;
    if (width === 0) return;
    const { seen, flat } = covers(width, height);
    clock.style.setProperty("--cap-height", String(capHeight()));
    clock.style.setProperty("--bar-natural", String(advance("body", time)));
    if (own) {
      element.style.setProperty("--picture-width", String(seen.width));
      element.style.setProperty("--picture-height", String(seen.height));
    } else {
      clock.style.setProperty("--city-flat-left", `${flat.left}px`);
      clock.style.setProperty("--city-flat-top", `${flat.top}px`);
      clock.style.setProperty("--city-flat-width", `${seen.width * flat.scale}px`);
      clock.style.setProperty("--city-flat-height", `${seen.height * flat.scale}px`);
    }
    const where = at(own, width, height);
    if (!held) drawClock(clock, where);
    if (own) inkOn(clock, time);
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
    const kickerBox = kicker.getBoundingClientRect();
    // Its baseline's depth below its top as a share of its size, so it holds at whatever size the
    // kicker is drawn (the size in the screen's pixels, the box as the viewport draws it).
    const kickerFont = Number.parseFloat(getComputedStyle(date).fontSize) * (kickerBox.width / ((kicker as HTMLElement).offsetWidth || kickerBox.width || 1));
    // From inside its padding: a clock with layers pads its kicker to give its mask room.
    const kickerPad = Number.parseFloat(getComputedStyle(kicker).paddingTop) * (kickerBox.height / ((kicker as HTMLElement).offsetHeight || kickerBox.height || 1));
    const kickerBase = kickerFont > 0 ? (kickerMark.getBoundingClientRect().bottom - kickerBox.top - kickerPad) / kickerFont : 0;
    yield;
    mark.remove();
    kickerMark.remove();
    if (measured) {
      clock.style.setProperty("--bar-cx", measured.cx);
      clock.style.setProperty("--bar-base", measured.base);
    }
    if (kickerBase > 0) clock.style.setProperty("--kicker-base-share", kickerBase.toFixed(5));
    if (held) return;
    const tint = own?.colour ?? tintAt(where, width, height);
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
  return {
    element,
    screen: shown,
    sheet: rising,
    content,
    setClock(set: LockClock): void {
      own = set;
      clockOn(element, clock, set);
      // Held, its owner draws the clock: nothing is measured, so a move it starts has no slow frame.
      if (!held) later(place);
    },
    hold(on: boolean): void {
      held = on;
      if (!on) later(place);
    },
    clockAt: (set, width, height) => at(set, width, height),
    tintAt,
  };
}

const ALIGN: Record<DateAlign, number> = { left: 0, centre: 0.5, right: 1 };

/**
 * Draws the rest clock at `to`, or on the way there from `from` as far as the number property `by`
 * stands (0 at `from`, 1 at `to`), so a move eases one number and the CSS draws every frame.
 */
export function drawClock(clock: HTMLElement, to: ClockAt, from?: ClockAt, by?: string): void {
  const mix = (a: number, b: number, unit: string) => (from === undefined || by === undefined || Math.abs(a - b) < 1e-4 ? `${b}${unit}` : `calc(${a}${unit} + ${b - a}${unit} * var(${by}))`);
  const was = from ?? to;
  clock.style.setProperty("--middle", mix(was.cx, to.cx, "px"));
  clock.style.setProperty("--rest-top-at", mix(was.top, to.top, "px"));
  clock.style.setProperty("--rest-size", mix(was.size, to.size, "px"));
  clock.style.setProperty("--kicker-size", mix(was.kickerSize, to.kickerSize, "px"));
  clock.style.setProperty("--kicker-line", mix(was.kickerLine, to.kickerLine, "px"));
  clock.style.setProperty("--kicker-gap", mix(was.kickerGap, to.kickerGap, "px"));
  clock.style.setProperty("--kicker-align", mix(was.align, to.align, ""));
  clock.style.setProperty("--digits-natural", mix(was.digits, to.digits, ""));
  clock.style.setProperty("--place-natural", mix(was.place, to.place, ""));
  clock.style.setProperty("--date-natural", mix(was.date, to.date, ""));
}

/**
 * Marks `element` as drawing a set clock and lays its layers, colour and glow on `clock`: the
 * digits and the kicker each take every layer in front of the clock out of themselves, laid where
 * its pixels are drawn, the city's ahead with the city, the picture's own with the picture.
 */
function clockOn(element: HTMLElement, clock: HTMLElement, set: LockClock): void {
  element.toggleAttribute("data-clocked", true);
  element.toggleAttribute("data-clockless", set.showClock === false);
  element.toggleAttribute("data-undated", set.showDate === false);
  const layers = set.layers ?? [];
  element.toggleAttribute("data-layered", layers.length > 0);
  for (const [part, x, y] of [
    [".k-lock-digits", "var(--clock-left)", "var(--clock-top)"],
    [".k-lock-kicker", "var(--clock-left) - var(--kicker-left)", "var(--clock-top) - var(--kicker-top)"],
  ] as const) {
    const each = clock.querySelector<HTMLElement>(part)!;
    if (layers.length === 0) {
      for (const property of ["mask-image", "mask-size", "mask-position", "mask-composite", "mask-repeat"]) each.style.removeProperty(property);
      continue;
    }
    const scale = (layer: Layer) => (layer.near ? "var(--neared)" : "var(--pushed)");
    const lead = (layer: Layer) => (layer.near ? " + var(--city-lead)" : "");
    // Each layer is taken out of the solid one under it in turn (`exclude`): the layers of a step
    // don't overlap, the cut-out being taken out of the bands round it, so each only removes.
    // Chromium doesn't take a union of several layers out of a solid one listed first.
    each.style.setProperty("mask-image", [...layers.map((layer) => `url("${layer.mask}")`), "linear-gradient(var(--color-surface), var(--color-surface))"].join(", "));
    each.style.setProperty("mask-composite", [...layers.map(() => "exclude"), "add"].join(", "));
    each.style.setProperty("mask-repeat", [...layers.map(() => "no-repeat"), "repeat"].join(", "));
    each.style.setProperty(
      "mask-size",
      [...layers.map((layer) => `calc(var(--city-flat-width) * ${scale(layer)} * var(--layer-cut)) calc(var(--city-flat-height) * ${scale(layer)} * var(--layer-cut))`), "100% 100%"].join(", "),
    );
    each.style.setProperty(
      "mask-position",
      [
        ...layers.map(
          (layer) =>
            `calc(var(--screen-middle) + (var(--city-flat-left) - var(--screen-middle)) * ${scale(layer)}${lead(layer)} - ${x}) calc(var(--screen-height) + (var(--city-flat-top) - var(--screen-height)) * ${scale(layer)} - ${y})`,
        ),
        "0 0",
      ].join(", "),
    );
  }
  element.toggleAttribute("data-glow", set.glow !== undefined);
  if (set.glow) {
    clock.style.setProperty("--glow-colour", set.glow.colour);
    clock.style.setProperty("--glow-strength", String(set.glow.strength));
  }
}

/** The digits' ink edges on `clock`, as shares of the type size, for the kicker to stand on. */
function inkOn(clock: HTMLElement, time: string): void {
  const ink = inkEdges(time);
  clock.style.setProperty("--ink-left", String(ink.left));
  clock.style.setProperty("--ink-right", String(ink.right));
}

/**
 * Where the ink of `time` in `type.clock_rest` starts and ends inside its box, as shares of the
 * type size: the figures' side bearings, which only the font knows, read off a canvas laid out as
 * the digits are (no kerning).
 */
export function inkEdges(time: string): { left: number; right: number } {
  const meter = document.createElement("span");
  meter.className = "k-lock-meter";
  document.documentElement.append(meter);
  // Read while it is laid out: a computed style is live, and empty once the meter is gone.
  const style = getComputedStyle(meter);
  const size = Number.parseFloat(style.fontSize);
  const font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  meter.remove();
  const context = document.createElement("canvas").getContext("2d");
  if (!context || !size) return { left: 0, right: 0 };
  context.font = font;
  context.fontKerning = "none";
  const measured = context.measureText(time);
  // Not kept: until the canvas draws the clock's own face it measures a stand-in's, and nothing
  // says when it does. One string is measured each time the lock is placed.
  return { left: -measured.actualBoundingBoxLeft / size, right: (measured.width - measured.actualBoundingBoxRight) / size };
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

const widestRests = new Map<boolean, number>();

/**
 * The widest time the rest clock can show, in logical pixels at `type.clock_rest`, read off the font
 * figure by figure: the widest hour, the colon and the widest minute. The figures are proportional,
 * so no one digit is the widest in every place.
 */
function widestRest(twelve: boolean): number {
  const known = widestRests.get(twelve);
  if (known !== undefined) return known;
  const figure = new Map<string, number>([...":0123456789"].map((c) => [c, digitsWidth(c)]));
  const widthOf = (words: string) => [...words].reduce((sum, c) => sum + figure.get(c)!, 0);
  const hours = twelve ? Array.from({ length: 12 }, (_, h) => String(h + 1)) : Array.from({ length: 24 }, (_, h) => String(h).padStart(2, "0"));
  const minutes = Array.from({ length: 60 }, (_, m) => String(m).padStart(2, "0"));
  const most = Math.max(...hours.map(widthOf)) + figure.get(":")! + Math.max(...minutes.map(widthOf));
  if (document.fonts.status === "loaded") widestRests.set(twelve, most);
  return most;
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

/**
 * The rest clock's figures as shares of its type size, for an owner that limits where it may
 * stand: Geist Light's cap height, and the widest time it can show.
 */
export function clockFigures(twelve = false): { readonly cap: number; readonly widest: number } {
  return { cap: capHeight(), widest: widestRest(twelve) / length("type-clock_rest-size") };
}
