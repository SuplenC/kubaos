// The lock screens the Lock page shows: three of the shipped wallpapers (`assets/lock/CREDITS.md`),
// two with their foreground cut out and one without, each in the night scheme matugen derives from
// it, and the lock live on them.
//
// Each night scheme is matugen's output for the shipped picture, as `scripts/dot-theme` makes it:
// `matugen image FILE -m dark -t scheme-fidelity --source-color-index 0 --json hex --dry-run`, its
// dark colours taken for the roles that have a matugen source (`[color.source]` in the tokens). The
// fixed roles keep the tokens' values.

import purpleCutout from "../../assets/lock/shinjuku-purple-hour-cutout.webp?url";
import purple from "../../assets/lock/shinjuku-purple-hour.jpg?url";
import shibuya from "../../assets/lock/shibuya-rainy-corner.jpg?url";
import towerCutout from "../../assets/lock/tokyo-tower-blue-hour-cutout.webp?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";
import type { Words } from "../../lib/content.ts";
import { setValue } from "../../lib/field.ts";
import { KICKER, type Kicker, type Lock, lock, lockField, lockLine, type Moment, type Phase, TIME, type Wallpaper } from "../../lib/lock.ts";
import { stack } from "../../lib/layout.ts";
import { raw } from "../../lib/tokens.ts";
import { settings } from "../../site/settings.ts";

/**
 * A screen the lock is drawn on: a 16:10 panel, 1536 by 960 logical pixels, a 2560 by 1600 laptop
 * at 1.67 or a 1920 by 1200 one at 1.25. Like the toolbar's window sizes, it is a sample's room,
 * not a design value.
 */
export const SCREEN: { readonly width: string; readonly height: string } = { width: "calc(1536 * var(--px))", height: "calc(960 * var(--px))" };

export const PURPLE_HOUR: Wallpaper = {
  name: "Shinjuku, purple hour",
  picture: purple,
  cutout: purpleCutout,
  // The digits stand in the sky, their feet behind the towers along the skyline.
  clock: { x: 0.5, y: 0.512 },
  night: { primary: "#d7baff", on_primary: "#420d7f", primary_container: "#7b4fb9", on_primary_container: "#ffffff", secondary: "#d3bdee", on_secondary: "#392950", secondary_container: "#503f68", on_secondary_container: "#ebd9ff", tertiary: "#ebc160", on_tertiary: "#3f2e00", tertiary_container: "#816100", on_tertiary_container: "#ffffff", cold: "#ebc160", on_cold: "#3f2e00", cold_container: "#816100", on_cold_container: "#ffffff", surface: "#151219", on_surface: "#e8e0ea", surface_variant: "#4b4451", on_surface_variant: "#cdc3d3", surface_dim: "#151219", surface_bright: "#3c383f", surface_container_lowest: "#100d13", surface_container_low: "#1d1a21", surface_container: "#221e25", surface_container_high: "#2c2930", surface_container_highest: "#37333b", outline: "#968e9d", outline_variant: "#4b4451", shadow: "#000000", scrim: "#000000", inverse_surface: "#e8e0ea", inverse_on_surface: "#332f36", inverse_primary: "#7347b0" },
};

export const TOWER: Wallpaper = {
  name: "Tokyo Tower, blue hour",
  picture: tower,
  cutout: towerCutout,
  // The tower's spire crosses the last digit; the kicker stays clear of it.
  clock: { x: 0.445, y: 0.47 },
  night: { primary: "#9cccf5", on_primary: "#003450", primary_container: "#1b5174", on_primary_container: "#dbedff", secondary: "#b7c9da", on_secondary: "#213240", secondary_container: "#3a4b5a", on_secondary_container: "#d4e6f8", tertiary: "#f0be74", on_tertiary: "#442b00", tertiary_container: "#694604", on_tertiary_container: "#ffe7ca", cold: "#f0be74", on_cold: "#442b00", cold_container: "#694604", on_cold_container: "#ffe7ca", surface: "#111416", on_surface: "#e2e2e5", surface_variant: "#41474e", on_surface_variant: "#c1c7cf", surface_dim: "#111416", surface_bright: "#37393c", surface_container_lowest: "#0c0e11", surface_container_low: "#191c1e", surface_container: "#1e2022", surface_container_high: "#282a2d", surface_container_highest: "#333538", outline: "#8b9198", outline_variant: "#41474e", shadow: "#000000", scrim: "#000000", inverse_surface: "#e2e2e5", inverse_on_surface: "#2e3133", inverse_primary: "#316387" },
};

export const SHIBUYA: Wallpaper = {
  name: "Shibuya, a rainy corner",
  picture: shibuya,
  // No cut-out: the clock stands in front, a little above the middle, its shade under it.
  clock: { x: 0.5, y: 0.56 },
  night: { primary: "#5ae7dd", on_primary: "#003734", primary_container: "#33cbc1", on_primary_container: "#00312e", secondary: "#9fd0ca", on_secondary: "#003734", secondary_container: "#20514d", on_secondary_container: "#bceee8", tertiary: "#ffc7a4", on_tertiary: "#512400", tertiary_container: "#ffa05f", on_tertiary_container: "#481f00", cold: "#ffc7a4", on_cold: "#512400", cold_container: "#ffa05f", on_cold_container: "#481f00", surface: "#0e1514", on_surface: "#dde4e2", surface_variant: "#3c4948", on_surface_variant: "#bbcac7", surface_dim: "#0e1514", surface_bright: "#343a39", surface_container_lowest: "#090f0f", surface_container_low: "#161d1c", surface_container: "#1a2120", surface_container_high: "#252b2a", surface_container_highest: "#2f3635", outline: "#859491", outline_variant: "#3c4948", shadow: "#000000", scrim: "#000000", inverse_surface: "#dde4e2", inverse_on_surface: "#2b3231", inverse_primary: "#006a64" },
};

export const WALLPAPERS: readonly Wallpaper[] = [PURPLE_HOUR, TOWER, SHIBUYA];

export const LOCK_ERROR: Words = {
  short: "Wrong password. Fat fingers? Try again.",
  long: "Fingerprint not recognised. Wrong finger? Try again or type your password",
};

/** The date in the kicker, at its shortest and its longest in English. */
export const DATES: Words = { short: "Monday, 2 November", long: "Wednesday, 30 September" };

/** The time, at its narrowest and its widest. */
export const TIMES: Words = { short: "1:11", long: "23:58" };

const KANJI: Record<string, string> = { Sunday: "日", Monday: "月", Tuesday: "火", Wednesday: "水", Thursday: "木", Friday: "金", Saturday: "土" };

/** The kicker for `date`: its weekday in Japanese after the place, then the date. */
export function kickerFor(date: string): Kicker {
  const weekday = date.split(",")[0] ?? "";
  return { place: `東京・${KANJI[weekday] ?? "月"}曜日`, date };
}

export interface Drawn {
  readonly phase?: Phase;
  readonly moment?: Moment;
  readonly wallpaper?: Wallpaper;
  readonly time?: string;
  readonly kicker?: Kicker;
  readonly error?: string;
  readonly size?: { readonly width: string; readonly height: string };
}

/** A lock drawn still, on a screen of `SCREEN`'s size unless given. */
export function still(drawn: Drawn = {}): HTMLElement {
  return lockOf(drawn).element;
}

function lockOf(drawn: Drawn, live = false): Lock {
  const size = drawn.size ?? SCREEN;
  return lock({
    ...size,
    wallpaper: drawn.wallpaper ?? PURPLE_HOUR,
    phase: drawn.phase ?? "rest",
    moment: drawn.moment ?? "rest",
    time: drawn.time ?? TIME,
    kicker: drawn.kicker ?? KICKER,
    error: drawn.error ?? LOCK_ERROR.short,
    live,
  });
}

/** The 12-hour clock: the digits without a meridiem, which takes its own slot in the kicker. */
export function twelveHour(phase: Phase): HTMLElement {
  return still({ phase, time: "9:47", kicker: { place: KICKER.place, date: `${KICKER.date} · PM` } });
}

/** Two screens, one over the other: the one with the pointer awake, the other still at rest. */
export function twoScreens(): HTMLElement {
  return stack("s16", "centre", still({ phase: "rest" }), still({ phase: "awake" }));
}

/** A duration token in milliseconds, as the toolbar's speed divides it. */
function millis(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The wrong password's shake: `space.s12`, back `space.s8`, `space.s4`, each over `micro`, then home over `short`. */
function shakeFrames(): Keyframe[] {
  const total = 3 * millis("micro") + millis("short");
  const at = (n: number) => (n * millis("micro")) / total;
  return [
    { "--jolt": "0", offset: 0, easing: raw("easing-standard") },
    { "--jolt": "3", offset: at(1), easing: raw("easing-standard") },
    { "--jolt": "-2", offset: at(2), easing: raw("easing-standard") },
    { "--jolt": "1", offset: at(3), easing: raw("easing-standard_decel") },
    { "--jolt": "0", offset: 1 },
  ];
}

/**
 * The lock, live: a key, a press or the reader wakes it, a printable key landing in the field;
 * Enter with a password shakes the field and says so; Esc, or `lock_idle` with nothing done, puts
 * it back to rest.
 */
export function liveLock(wallpaper: Wallpaper = PURPLE_HOUR, size = SCREEN): HTMLElement {
  const made = lockOf({ wallpaper, size }, true);
  const element = made.element;
  element.tabIndex = 0;
  const room = made.content.querySelector<HTMLElement>(".k-lock-field")!;
  let idle = 0;
  const current = () => room.querySelector<HTMLElement>(".k-field")!;
  const typed = () => room.querySelector(".k-field-room")?.textContent ?? "";
  const awake = () => element.hasAttribute("data-awake");
  const lineOf = () => room.nextElementSibling?.nextElementSibling as HTMLElement | undefined;

  const fresh = (moment: Moment) => {
    const next = lockField(moment, true);
    room.replaceChildren(next);
    lineOf()?.replaceWith(lockLine(moment, LOCK_ERROR.short));
    return next;
  };
  const keepAwake = () => {
    window.clearTimeout(idle);
    idle = window.setTimeout(rest, millis("lock_idle"));
  };
  const setAwake = (on: boolean) => {
    element.toggleAttribute("data-awake", on);
    element.toggleAttribute("data-open", on);
    for (const part of made.sheet.parts) part.toggleAttribute("data-shown", on);
  };
  function wake(first?: string): void {
    setAwake(true);
    const input = current();
    if (first !== undefined) setValue(input, first);
    requestAnimationFrame(() => input.focus({ focusVisible: true, preventScroll: true } as FocusOptions));
    keepAwake();
  }
  function rest(): void {
    window.clearTimeout(idle);
    fresh("rest");
    setAwake(false);
    element.focus({ preventScroll: true });
  }

  element.addEventListener("keydown", (event) => {
    if (!awake()) {
      if (event.key === "Tab" || event.key === "Escape" || event.ctrlKey || event.metaKey || event.altKey) return;
      event.preventDefault();
      wake(event.key.length === 1 ? event.key : undefined);
      return;
    }
    keepAwake();
    if (event.key === "Escape") {
      event.preventDefault();
      rest();
      return;
    }
    if (event.key !== "Enter" || typed() === "") return;
    event.preventDefault();
    const missed = fresh("error");
    requestAnimationFrame(() => missed.focus({ focusVisible: true, preventScroll: true } as FocusOptions));
    if (!settings().reduced) room.animate(shakeFrames(), { duration: 3 * millis("micro") + millis("short") });
  });
  element.addEventListener("mousedown", () => {
    if (!awake()) wake();
    else keepAwake();
  });
  return element;
}
