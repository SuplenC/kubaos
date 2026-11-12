// The wallpaper picker, a gallery (`config/quickshell/modules/wallpaper/`, not built yet): every
// picture of a folder hung on a wall, a floor per folder, a neon lamp over each, the one in front of
// you lit. Each picture is drawn as what it would be: the picture alone, your desktop on it with
// its bar, frame and windows, or your lock with its clock between the plate and the city, each the
// Lock template at the screen's size scaled into its frame, in its own colours. The one in front of
// you steps out to fill the screen, and the live desktop is that picture stepped out: opening steps
// back into the wall, applying and cancelling step out again.
//
// Numbers move it: the screen's `--out` (the wall, 0, to the picture in front of you filling the
// screen, 1), each floor's and track's place, each picture's `--aside` (how far along the wall it is
// from the one in front of you), and each Lock's own (`lib/lock.ts`). Every move eases a number on
// the element that stays, so its last frame is what rests there.

import "./wallpaper.css";
import { button } from "../../lib/button.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { chip } from "../../lib/chip.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { row, stack } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { type Lock, lock, type Wallpaper } from "../../lib/lock.ts";
import { nested, role } from "../../lib/radius.ts";
import { type ShortcutGroup } from "../../lib/shortcuts.ts";
import { shortcutsButton, takeShortcutKeys } from "../../lib/shortcuts-button.ts";
import { switchControl } from "../../lib/switch.ts";
import { tabs } from "../../lib/tabs.ts";
import { text } from "../../lib/text.ts";
import { keyTooltip } from "../../lib/tooltip.ts";
import type { Duration, Easing, Track } from "../../site/page.ts";
import { bezier, css, cueOf, leads, milliseconds, strikeSteps } from "../../site/recipe.ts";
import { type ColourPath, oklch, type Palette, paint, paintBetween, palette, paletteOf } from "../../site/recolour.ts";
import { type Mode, settings } from "../../site/settings.ts";
import { barButtons } from "../templates/panels.ts";
import { fromAwake, fromAwakeReduced, locking, lockingReduced, resting, reducedRest, touching, touchingReduced, waking, wakingReduced } from "../templates/lock-moves.ts";
import { PURPLE_HOUR, SCREEN, SHIBUYA, TOWER } from "../templates/locks.ts";
import { wallpaper as kabukicho } from "./bar-parts.ts";
import sign from "../../../../../branding/photo/neon-sign-banner.jpg?url";
import hero from "../../../../../branding/png/kubaos-hero.jpg?url";
import { type ClockLook, clockEditor, defaultLook, type Editor, type EditorPose, lockClock } from "./wallpaper-clock.ts";
import { HERO_DAY, HERO_NIGHT, KABUKICHO_DAY, KABUKICHO_NIGHT, PURPLE_DAY, SHIBUYA_DAY, SIGN_DAY, SIGN_NIGHT, TOWER_DAY } from "./wallpaper-schemes.ts";

/** A picture on the wall: the Lock's wallpaper, its day scheme, and whether it moves. */
export interface Hung {
  readonly name: string;
  readonly wallpaper: Wallpaper;
  readonly day: Palette;
  readonly video?: boolean;
}


export const KABUKICHO: Hung = { name: "Kabukicho rain", wallpaper: { name: "Kabukicho rain", picture: kabukicho, night: KABUKICHO_NIGHT }, day: KABUKICHO_DAY };
export const PURPLE: Hung = { name: "Shinjuku purple hour", wallpaper: PURPLE_HOUR, day: PURPLE_DAY };
export const BLUE: Hung = { name: "Tokyo Tower blue hour", wallpaper: TOWER, day: TOWER_DAY };
export const SIGN: Hung = { name: "Neon sign", wallpaper: { name: "Neon sign", picture: sign, night: SIGN_NIGHT }, day: SIGN_DAY, video: true };
export const RAIN: Hung = { name: "Shibuya rainy corner", wallpaper: SHIBUYA, day: SHIBUYA_DAY };
export const HERO: Hung = { name: "kubaos-hero", wallpaper: { name: "kubaos-hero", picture: hero, night: HERO_NIGHT }, day: HERO_DAY };

export interface Folder {
  readonly name: string;
  readonly icon: string;
  readonly pictures: readonly Hung[];
}

/** KubaOS's own, the user's own, and the empty downloads: a floor each, the first the lowest. */
export const FOLDERS: readonly Folder[] = [
  { name: "KubaOS", icon: "wallpaper", pictures: [KABUKICHO, PURPLE, BLUE, SIGN, RAIN] },
  { name: "Mine", icon: "person", pictures: [HERO] },
  { name: "Downloads", icon: "download", pictures: [] },
];

/** The wallpaper applied: the desktop the gallery opens from. */
export const APPLIED = KABUKICHO;

/** Every key the picker takes, as its shortcuts popover lists them. */
export const PICKER_KEYS: readonly ShortcutGroup[] = [
  { name: "Browse", keys: [["← →", "The picture before or after"], ["↑ ↓", "The floor above or below"]] },
  { name: "Views", keys: [["1", "Picture"], ["2", "Desktop"], ["3", "Lock"], ["F", "Full screen and back"], ["W", "Wake the lock, or let it rest"]] },
  { name: "Clock", keys: [["E", "The clock's panel"], ["j k", "The setting below or above"], ["h l", "Less or more; Shift for a fine step"], ["R", "Standard place"], ["D", "In front, or back to its last step"]] },
  { name: "Depth", keys: [["D", "Depth on or off"]] },
  { name: "Apply and leave", keys: [["↵", "Apply"], ["Esc", "Cancel: the old wallpaper back"], ["?", "These shortcuts"]] },
];

export type View = "picture" | "desktop" | "lock";
export const VIEWS: readonly View[] = ["picture", "desktop", "lock"];
const VIEW_NAMES: Record<View, string> = { picture: "Picture", desktop: "Desktop", lock: "Lock" };

export interface GalleryOptions {
  /** Open on the wall; shut, it is the live desktop. */
  readonly open?: boolean;
  readonly view?: View;
  /** The picture in front of you filling the screen. */
  readonly full?: boolean;
  readonly folder?: number;
  /** The picture in front of you, by its index in the folder; the applied one's by default. */
  readonly at?: number;
  /** The lock in front of you awake. */
  readonly awake?: boolean;
  /** Pictures with depth turned off, by name. */
  readonly flat?: readonly string[];
  /** The windows' stills landed; false: only the desktop you opened from has them. */
  readonly stills?: boolean;
  /** The shortcuts open over their button. */
  readonly shortcuts?: boolean;
  /** A control's tooltip shown, by its name. */
  readonly tip?: string;
  /** The folders, if not the shots' three. */
  readonly folders?: readonly Folder[];
  /** The picture in front of you's name, at the toolbar's length. */
  readonly name?: string;
  /** Live: the keys, the pointer and every move. */
  readonly live?: boolean;
  /** The screen's size, if not the Lock's sample screen. */
  readonly size?: { readonly width: string; readonly height: string };
  /** The lamp lit with the signs off: a power change's dark beat, or by day. */
  readonly signsOff?: boolean;
  /** The wall alone, for a move that plays on the pictures: no bar, placard or floors' list. */
  readonly bare?: boolean;
  /** The clock's editor open on the lock in front of you, drawn in this pose; or a move's two. */
  readonly edit?: EditorPose | { readonly from: EditorPose | "wall"; readonly to: EditorPose | "wall" };
  /** Clocks set for pictures, by name, over their own. */
  readonly looks?: Readonly<Record<string, Partial<ClockLook>>>;
}

/** The mode the gallery is drawn in: its pane's. */
function modeOf(element: HTMLElement): Mode {
  const pane = element.parentElement?.closest<HTMLElement>("[data-mode]");
  return pane?.dataset.mode === "light" ? "light" : pane?.dataset.mode === "dark" ? "dark" : settings().mode;
}

const reducedFor = (element: Element) => settings().reduced || element.closest("[data-reduced]") !== null;
const ms = (duration: Duration) => milliseconds(duration) / settings().speed;
const curveOf = (easing: Easing) => `cubic-bezier(${bezier(easing).join(", ")})`;
const wait = (time: number) => new Promise<void>((done) => window.setTimeout(done, time));

/** A token's value as `element` resolves it, for a keyframe: `var()` is read, never passed on. */
function resolved(element: HTMLElement, value: number | string): string {
  if (typeof value === "number") return String(value);
  const name = /^var\((--[\w-]+)\)$/.exec(value)?.[1];
  return name ? getComputedStyle(element).getPropertyValue(name).trim() : value;
}

/**
 * Plays `tracks` on `root` and what it holds, each from its `from` to its `to` on its curve after its
 * cue, as the players play them. A property set on an element's own style is set to its `to` first,
 * so once the animations end, what rests is the end. Resolves when every track is done.
 */
export function play(root: HTMLElement, tracks: readonly Track[]): Promise<void> {
  const led = leads(tracks);
  const runs: Animation[] = [];
  for (const track of tracks) {
    const found = [...(root.matches(track.target) ? [root] : []), ...root.querySelectorAll<HTMLElement>(track.target)];
    for (const element of found) {
      const to = resolved(element, css(track.to));
      if (element.style.getPropertyValue(track.property) !== "") element.style.setProperty(track.property, to);
      const easing = curveOf(track.curve as Easing);
      runs.push(
        element.animate([{ [track.property]: resolved(element, css(track.from)) }, { [track.property]: to }], {
          duration: Math.max(1, ms(track.duration)),
          delay: cueOf(track) / settings().speed,
          easing,
          fill: led.has(track) ? "forwards" : "both",
        }),
      );
    }
  }
  return Promise.all(runs.map((run) => run.finished.catch(() => undefined))).then(() => {
    for (const run of runs) run.cancel();
  });
}

/** Eases `property` on `element` from where it is to `to`: set at once, the animation the way there. */
function ease(element: HTMLElement, property: string, to: string | number, duration: number, easing: Easing, delay = 0): Promise<void> {
  const from = getComputedStyle(element).getPropertyValue(property).trim() || (property === "opacity" ? getComputedStyle(element).opacity : "0");
  element.style.setProperty(property, String(to));
  if (duration <= 0) return wait(delay);
  return element
    .animate([{ [property]: from }, { [property]: String(to) }], { duration, delay, easing: curveOf(easing), fill: "backwards" })
    .finished.then(() => undefined)
    .catch(() => undefined);
}

/** Recolours `element` from the palette it is in to `to` along `path` over `recolour`. */
function recolour(element: HTMLElement, from: Palette | undefined, to: Palette, path: ColourPath): void {
  if (!from || reducedFor(element)) {
    paint(element, to);
    return;
  }
  paintBetween(element, path, from, to);
  const run = element.animate([{ "--recolour": "0" }, { "--recolour": "1" }], { duration: ms("recolour"), easing: curveOf("gather"), fill: "forwards" });
  void run.finished.then(() => {
    paint(element, to);
    run.cancel();
  });
}

// The moves between the views, on one picture's desktop. The one in front of you plays the Lock's
// own locking and unlocking; the others a simpler one, the clock fading where it stands.

const MAIN = "the picture in front of you";
const OTHERS = "the pictures either side";

/** `target` in `scope`. */
const at = (scope: string, target: string) => (scope === "" ? target : target.split(",").map((each) => `${scope} ${each.trim()}`).join(", "));

/** Picture to desktop: the frame and the bar slide in, the picture eases back to 1:1, the windows settle in. */
export function toDesktop(on: string, scope = "", pushed = true): Track<Easing>[] {
  return [
    { part: "the frame", on, target: at(scope, ".k-lock"), property: "--framed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the bar", on, target: at(scope, ".k-lock"), property: "--barred", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    ...(pushed ? [{ part: "the push", on, target: at(scope, ".k-lock"), property: "--push", from: 1, to: 0, duration: "long", curve: "emphasized_decel" } as Track<Easing>] : []),
    { part: "the windows", on, target: at(scope, ".k-lock-windows"), property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
    { part: "the windows' size", on, target: at(scope, ".k-lock-windows"), property: "scale", from: { scale: "behind" }, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
  ];
}

/** The same with reduce motion: the frame's glass, the bar and the windows fade in over `short`. */
export function toDesktopReduced(on: string, scope = ""): Track<Easing>[] {
  return [
    { part: "the frame", on, target: at(scope, ".k-lock"), property: "--framed", from: 1, to: 1, duration: "instant", curve: "standard" },
    { part: "the bar", on, target: at(scope, ".k-lock"), property: "--barred", from: 1, to: 1, duration: "instant", curve: "standard" },
    { part: "the frame's glass", on, target: at(scope, ".k-screen-glass"), property: "opacity", from: 0, to: { opacity: "frame" }, duration: "short", curve: "standard" },
    { part: "the bar's pills", on, target: at(scope, ".k-bar"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the windows' size", on, target: at(scope, ".k-lock-windows"), property: "scale", from: 1, to: 1, duration: "instant", curve: "standard" },
    { part: "the windows", on, target: at(scope, ".k-lock-windows"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ];
}

/** Desktop to picture: the windows step back, the frame and the bar slide out, the picture pushes in. */
export function toPicture(on: string, scope = "", pushed = true): Track<Easing>[] {
  return [
    { part: "the windows", on, target: at(scope, ".k-lock-windows"), property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the windows' size", on, target: at(scope, ".k-lock-windows"), property: "scale", from: 1, to: { scale: "behind" }, duration: "medium", curve: "emphasized_accel" },
    { part: "the frame", on, target: at(scope, ".k-lock"), property: "--framed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: "the bar", on, target: at(scope, ".k-lock"), property: "--barred", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    ...(pushed ? [{ part: "the push", on, target: at(scope, ".k-lock"), property: "--push", from: 0, to: 1, duration: "long", curve: "emphasized_decel" } as Track<Easing>] : []),
  ];
}

export function toPictureReduced(on: string, scope = ""): Track<Easing>[] {
  return [
    { part: "the windows", on, target: at(scope, ".k-lock-windows"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the frame's glass", on, target: at(scope, ".k-screen-glass"), property: "opacity", from: { opacity: "frame" }, to: 0, duration: "short", curve: "standard" },
    { part: "the bar's pills", on, target: at(scope, ".k-bar"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the frame", on, target: at(scope, ".k-lock"), property: "--framed", from: 0, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: "the bar", on, target: at(scope, ".k-lock"), property: "--barred", from: 0, to: 0, duration: "instant", cue: "short", curve: "standard" },
  ];
}

/** Picture to lock: the picture pushes in, if it isn't, and the clock fades in behind the city. */
export function pictureToLock(on: string, scope = "", pushed = true, reduced = false): Track<Easing>[] {
  const clock: Track<Easing> = reduced
    ? { part: "the clock", on, target: at(scope, ".k-lock-clock"), property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }
    : { part: "the clock", on, target: at(scope, ".k-lock-clock"), property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "standard" };
  return [...(pushed || reduced ? [] : [{ part: "the push", on, target: at(scope, ".k-lock"), property: "--push", from: 0, to: 1, duration: "long", curve: "emphasized_decel" } as Track<Easing>]), clock];
}

/** Lock to picture: the clock fades out, and the picture eases back if it isn't the one in front. */
export function lockToPicture(on: string, scope = "", pushed = true, reduced = false): Track<Easing>[] {
  return [
    { part: "the clock", on, target: at(scope, ".k-lock-clock"), property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    ...(pushed || reduced ? [] : [{ part: "the push", on, target: at(scope, ".k-lock"), property: "--push", from: 1, to: 0, duration: "long", curve: "emphasized_decel" } as Track<Easing>]),
  ];
}

/** The neighbours from desktop to lock: the frame and the bar slide out, the clock fades in. */
export function sideToLock(on: string, scope = "", reduced = false): Track<Easing>[] {
  return [...(reduced ? toPictureReduced(on, scope) : toPicture(on, scope)), ...pictureToLock(on, scope, true, reduced)];
}

/** The neighbours from lock to desktop: the clock fades out, the frame and the bar slide in. */
export function sideToDesktop(on: string, scope = "", reduced = false): Track<Easing>[] {
  return [...lockToPicture(on, scope, true, reduced), ...(reduced ? toDesktopReduced(on, scope) : toDesktop(on, scope))];
}

/** The tracks one picture plays going from view `from` to `to`. */
function viewTracks(from: View, to: View, main: boolean, reduced: boolean, awake: boolean): Track<Easing>[] {
  const on = main ? MAIN : OTHERS;
  if (from === "desktop" && to === "lock") return main ? (reduced ? lockingReduced(on) : locking(on)) : sideToLock(on, "", reduced);
  if (from === "lock" && to === "desktop") {
    if (!main) return sideToDesktop(on, "", reduced);
    if (awake) return reduced ? fromAwakeReduced(on) : fromAwake(on);
    return reduced ? touchingReduced(on) : touching(on);
  }
  if (from === "picture" && to === "desktop") return reduced ? toDesktopReduced(on) : toDesktop(on, "", main);
  if (from === "desktop" && to === "picture") return reduced ? toPictureReduced(on) : toPicture(on, "", main);
  if (from === "picture" && to === "lock") return pictureToLock(on, "", main, reduced);
  return lockToPicture(on, "", main, reduced);
}

const PHASES: Record<View, "picture" | "desktop" | "rest"> = { picture: "picture", desktop: "desktop", lock: "rest" };

/** One picture's desktop: the Lock template at the screen's size. */
interface Desk {
  readonly hung: Hung;
  readonly lock: Lock;
  readonly element: HTMLElement;
  view: View;
  palette: Palette | undefined;
}

/** The desktop's windows, stills of the user's: two side by side, each its name and a line. */
function stills(): HTMLElement {
  const room = document.createElement("span");
  room.className = "gl-stills";
  const corner = nested(role("surface"), "s8");
  // Where the user's windows happen to be: stand-ins, the terminal left, the files lower right.
  const places = [
    { title: "Terminal", line: "just mockups", left: "0%", top: "4%", width: "46%", height: "62%" },
    { title: "Files", line: "~/Pictures/Wallpapers", left: "56%", top: "30%", width: "40%", height: "56%" },
  ];
  for (const place of places) {
    const drawn = document.createElement("span");
    drawn.className = "k-lock-window gl-window";
    drawn.style.setProperty("--corner", corner.css);
    Object.assign(drawn.style, { left: place.left, top: place.top, width: place.width, height: place.height });
    drawn.append(text(place.title, "body_strong", "on_surface"), text(place.line, "small", "on_surface_variant"));
    room.append(drawn);
  }
  return room;
}

function makeDesk(hung: Hung, view: View, mode: Mode, size: { readonly width: string; readonly height: string }, wakes: boolean, look: ClockLook): Desk {
  // A picture of a desktop: its bar's pills still; only the lock in front can wake, so only it holds
  // the password's sheet until another comes to the front. Its clock is as set for the picture.
  const made = lock({ width: size.width, height: size.height, wallpaper: hung.wallpaper, phase: PHASES[view], bar: barButtons(false), windows: [stills()], sheet: wakes, clock: lockClock(look, hung.wallpaper, "wide") });
  const element = made.element;
  if (hung.video) {
    const loop = document.createElement("img");
    loop.className = "gl-loop";
    loop.src = hung.wallpaper.picture;
    loop.alt = "";
    element.querySelector(".k-screen-wallpaper")!.after(loop);
  }
  const desk: Desk = { hung, lock: made, element, view, palette: undefined };
  paintDesk(desk, mode, false);
  return desk;
}

/** A scheme's palette in `mode`, its derived roles worked out once. */
const palettes = new WeakMap<Palette, Map<Mode, Palette>>();
function paletteIn(scheme: Palette, mode: Mode): Palette {
  const known = palettes.get(scheme) ?? new Map<Mode, Palette>();
  palettes.set(scheme, known);
  const found = known.get(mode) ?? paletteOf(scheme, mode);
  known.set(mode, found);
  return found;
}

/** The palette a desktop is drawn in: its day scheme by day on the desktop, else its night. */
function paletteFor(hung: Hung, view: View, mode: Mode): { palette: Palette; mode: Mode } {
  const day = view === "desktop" && mode === "light";
  return { palette: paletteIn(day ? hung.day : hung.wallpaper.night, day ? "light" : "dark"), mode: day ? "light" : "dark" };
}

function paintDesk(desk: Desk, mode: Mode, moving: boolean): void {
  const next = paletteFor(desk.hung, desk.view, mode);
  if (desk.palette === next.palette) return;
  if (moving) recolour(desk.element, desk.palette, next.palette, "dawn");
  else paint(desk.element, next.palette);
  desk.element.dataset.mode = next.mode;
  desk.palette = next.palette;
}

/** Each role's OKLCH lightness in the tokens' own palette: the dark room's neutral, the same for
    every picture. */
const neutrals = new Map<Mode, Map<string, number>>();
function neutralLightness(mode: Mode): Map<string, number> {
  const known = neutrals.get(mode);
  if (known) return known;
  const found = new Map<string, number>();
  for (const [role, value] of Object.entries(palette("neon", mode))) if (/^#[0-9a-f]{6}$/i.test(value)) found.set(role, oklch(value).l);
  neutrals.set(mode, found);
  return found;
}

/** The room's palette: the picture in front of you's scheme in the page's mode. */
function roomPalette(hung: Hung | undefined, mode: Mode): Palette {
  const scheme = hung ? (mode === "light" ? hung.day : hung.wallpaper.night) : KABUKICHO_NIGHT;
  return paletteIn(scheme, mode);
}

/** The words over a picture: the folder, which of how many, in `label`, the 壁紙 accent first. */
function kicker(folder: Folder, index: number): HTMLElement {
  const count = folder.pictures.length;
  const which = count === 0 ? "empty" : `${String(index + 1).padStart(2, "0")} / ${String(count).padStart(2, "0")}`;
  return row("s8", "centre", text("壁紙", "accent", "cold"), text(`${folder.name} · ${which}`, "label_number", "on_surface_variant"));
}

/** The picture's chips: Current, a video's, and its depth. */
function chipsFor(hung: Hung, applied: Hung, flat: boolean, reduced: boolean): HTMLElement {
  const made: HTMLElement[] = [];
  if (hung === applied) made.push(chip({ label: "Current", kind: "Selected", icon: "check" }));
  if (hung.video) made.push(chip({ label: reduced ? "Video, still" : "Video", kind: "Neutral", icon: reduced ? "motion_photos_paused" : "motion_photos_on" }));
  if (hung.wallpaper.depth) made.push(chip({ label: flat ? "Depth off" : "Depth", kind: flat ? "Neutral" : "Cold", icon: "deployed_code" }));
  return row("s8", "centre", ...made);
}

/** Why the depth switch says what it says. */
function depthWords(hung: Hung, flat: boolean): string {
  if (!hung.wallpaper.depth) return "No cut-out: this picture is flat.";
  return flat ? "Off: one layer." : "The city stands off the sky.";
}

/** The gallery. */
export function gallery(options: GalleryOptions = {}): HTMLElement {
  const folders = options.folders ?? FOLDERS;
  const live = options.live ?? false;
  const screen = document.createElement("span");
  screen.className = "gl-screen";
  const size = options.size ?? SCREEN;
  screen.style.setProperty("--screen-width", size.width);
  screen.style.setProperty("--screen-height", size.height);
  screen.toggleAttribute("data-signs-off", options.signsOff ?? false);
  screen.tabIndex = live ? 0 : -1;

  const appliedFolder = Math.max(0, folders.findIndex((each) => each.pictures.includes(APPLIED)));
  const S = {
    open: options.open ?? true,
    view: (options.open ?? true) ? (options.view ?? "picture") : ("desktop" as View),
    full: !(options.open ?? true) || (options.full ?? false),
    folder: options.folder ?? appliedFolder,
    at: folders.map((folder, index) => (index === (options.folder ?? appliedFolder) && options.at !== undefined ? options.at : Math.max(0, folder.pictures.indexOf(APPLIED)))),
    awake: options.awake ?? false,
    flat: new Set(options.flat ?? []),
    applied: APPLIED,
    busy: false,
    stills: options.stills ?? true,
  };
  let mode: Mode = settings().mode;
  const front = () => folders[S.folder]!.pictures[S.at[S.folder]!];
  // Each picture's clock as set, by its name: its own until the clock's editor changes it.
  const looks = new Map<string, ClockLook>();
  const lookOf = (hung: Hung): ClockLook => {
    const known = looks.get(hung.name);
    if (known) return known;
    const made = { ...defaultLook(hung.wallpaper), ...structuredClone(options.looks?.[hung.name] ?? {}) };
    looks.set(hung.name, made);
    return made;
  };
  let editor: Editor | undefined;

  const room = document.createElement("span");
  room.className = "gl-room";
  const floors = document.createElement("span");
  floors.className = "gl-floors";
  room.append(floors);
  screen.append(room);

  // The pictures, a desktop each, and their places on the wall.
  const desks = new Map<HTMLElement, Desk>();
  const makers = new Map<HTMLElement, (wakes: boolean) => Desk>();
  const arts: HTMLElement[][] = [];
  folders.forEach((folder, level) => {
    const floor = document.createElement("span");
    floor.className = "gl-floor";
    floor.style.setProperty("--level", String(level));
    const track = document.createElement("span");
    track.className = "gl-track";
    floor.append(track);
    floors.append(floor);
    const here: HTMLElement[] = [];
    if (folder.pictures.length === 0) {
      const art = document.createElement("span");
      art.className = "gl-art";
      art.style.setProperty("--index", "0");
      const frame = document.createElement("span");
      frame.className = "gl-frame";
      frame.dataset.empty = "";
      frame.append(emptyState({ icon: "wallpaper", title: "Nothing here yet", body: `Drop pictures or videos in ~/Pictures/Wallpapers/${folder.name.toLowerCase()}.` }));
      const lamp = document.createElement("span");
      lamp.className = "gl-lamp gl-room-only";
      art.append(lamp, frame);
      track.append(art);
      here.push(art);
    }
    folder.pictures.forEach((hung, index) => {
      const art = document.createElement("span");
      art.className = "gl-art";
      art.style.setProperty("--index", String(index));
      const lamp = document.createElement("span");
      lamp.className = "gl-lamp gl-room-only";
      const frame = document.createElement("span");
      frame.className = "gl-frame";
      // A picture's desktop is built when it first shows: the one in front and those either side
      // at once, the rest while the gallery is idle, so the first frame waits on three, not all.
      makers.set(art, (wakes: boolean) => {
        const desk = makeDesk(index === 0 && options.name ? { ...hung, name: options.name } : hung, S.view, mode, size, wakes, lookOf(hung));
        if (S.flat.has(hung.name)) flatten(desk, true);
        frame.append(desk.element);
        return desk;
      });
      const placard = document.createElement("span");
      placard.className = "gl-placard gl-room-only";
      art.append(lamp, frame, placard);
      art.addEventListener("click", (event) => {
        if (!S.open || S.busy || editor?.editing()) return;
        // The password's sheet, up on an awake lock, keeps its own clicks.
        if (event.target instanceof Element && event.target.closest(".k-sheet-part")) return;
        // The picture in front of you steps out to the full screen, and a click on it there steps
        // back to the wall, as F does.
        if (level === S.folder && index === S.at[level]) void step(!S.full);
        else {
          S.folder = level;
          move(index - S.at[level]!);
        }
      });
      track.append(art);
      here.push(art);
    });
    arts.push(here);
  });
  const deskAt = () => desks.get(arts[S.folder]![S.at[S.folder]!]!);

  /** Builds `art`'s desktop if it isn't yet, in the look the gallery is in. */
  const ensure = (art: HTMLElement | undefined): void => {
    if (!art || desks.has(art)) return;
    const make = makers.get(art);
    if (!make) return;
    const desk = make(live || (options.awake ?? false));
    desks.set(art, desk);
    look(art, desk);
  };
  /** The picture in front of you and those either side, built. */
  const ensureNear = (): void => {
    const here = arts[S.folder]!;
    // With the clock's editor drawn open only the picture in front of you shows.
    if (options.edit) return ensure(here[S.at[S.folder]!]);
    for (let index = S.at[S.folder]! - 1; index <= S.at[S.folder]! + 1; index += 1) ensure(here[index]);
  };

  // The floors' list at the wall's start.
  const floorsPanel = document.createElement("span");
  floorsPanel.className = "gl-floors-panel gl-room-only";
  room.append(floorsPanel);

  // The bar, the shortcuts and the tooltips.
  const bar = document.createElement("span");
  bar.className = "gl-bar";
  const shortcutsMade = shortcutsButton(PICKER_KEYS, { shown: options.shortcuts ?? false });
  shortcutsMade.element.classList.add("gl-info");
  const tip = document.createElement("span");
  tip.className = "gl-tip";
  const controls = document.createElement("span");
  controls.className = "gl-controls";
  controls.append(bar, shortcutsMade.element);
  screen.append(controls, tip);

  let tipFor: HTMLElement | null = null;
  const showTip = (control: HTMLElement) => {
    if (shortcutsMade.open()) return;
    tipFor = control;
    tip.replaceChildren(keyTooltip(control.dataset.tip ?? "", control.dataset.keys ?? ""));
    const k = screen.getBoundingClientRect().width / screen.offsetWidth || 1;
    const box = control.getBoundingClientRect();
    const base = screen.getBoundingClientRect();
    const gap = Number.parseFloat(getComputedStyle(screen).getPropertyValue("--space-s8")) || 8;
    tip.style.left = `${(box.left - base.left) / k}px`;
    tip.style.top = `${(box.top - base.top) / k - gap}px`;
    tip.style.translate = "0 -100%";
    tip.dataset.shown = "";
  };
  const hideTip = (control: HTMLElement) => {
    if (tipFor !== control) return;
    tipFor = null;
    delete tip.dataset.shown;
  };
  /** Gives `control` its name and key on a tooltip, shown under the pointer or the keyboard. */
  const tipOn = (control: HTMLElement, name: string, key: string) => {
    control.dataset.tip = name;
    control.dataset.keys = key;
    control.addEventListener("pointerenter", () => showTip(control));
    control.addEventListener("pointerleave", () => hideTip(control));
    control.addEventListener("focus", () => control.matches(":focus-visible") && showTip(control));
    control.addEventListener("blur", () => hideTip(control));
    if (options.tip === name) requestAnimationFrame(() => requestAnimationFrame(() => showTip(control)));
  };

  // Painting. The room and its words take the front picture's colours as far as the room is lit
  // (`--room-lit`): every role keeps its hue, its chroma rising from none and its lightness from the
  // tokens' own palette's, so with the lamps off every picture's room is the same neutral room,
  // and a new picture's colours come in only as its lamp strikes, never through another hue.
  const paintRoom = () => {
    mode = modeOf(screen);
    screen.dataset.mode = mode;
    const to = roomPalette(front(), mode);
    const zero = neutralLightness(mode);
    for (const [role, value] of Object.entries(to)) {
      const l0 = zero.get(role);
      screen.style.setProperty(
        `--color-${role}`,
        l0 === undefined || !/^#[0-9a-f]{6}$/i.test(value) ? value : `oklch(from ${value} calc(l * var(--room-lit) + ${l0.toFixed(4)} * (1 - var(--room-lit))) calc(c * var(--room-lit)) h)`,
      );
    }
    screen.style.setProperty("--glow", to.primary ?? "");
  };
  /** Every lamp in its own picture's primary, lit or not: a lamp never changes colour. */
  const paintLamps = () => {
    arts.forEach((here, level) =>
      here.forEach((art, index) => {
        const hung = folders[level]!.pictures[index];
        if (hung) art.querySelector<HTMLElement>(".gl-lamp")?.style.setProperty("--lamp-tone", roomPalette(hung, modeOf(screen)).primary ?? "");
      }),
    );
  };

  // The light: lights off, walk, lights on. One number, `--room-lit`, on the screen: the lamp over
  // the picture in front of you burns with it, its light lies on the room with it, and the room's
  // colours come in with it.
  let litArt: HTMLElement | undefined;
  let lightRun: Animation | undefined;
  let strikeTimer = 0;
  const litNow = () => Number.parseFloat(getComputedStyle(screen).getPropertyValue("--room-lit")) || 0;
  /** The lamp goes off with a quick drop, over `short`, and the room goes dark and neutral. */
  const lightsOff = () => {
    window.clearTimeout(strikeTimer);
    const from = litNow();
    lightRun?.cancel();
    screen.style.setProperty("--room-lit", "0");
    if (from > 0) lightRun = screen.animate([{ "--room-lit": String(from) }, { "--room-lit": "0" }], { duration: ms("short"), easing: curveOf(reducedFor(screen) ? "standard" : "emphasized_accel") });
  };
  /** The lamp over the picture in front of you strikes on: the neon sign's flicker over `strike`,
      its steps as fades with Steady, and a fade over `short` with reduce motion. */
  const strike = () => {
    const art = arts[S.folder]?.[S.at[S.folder]!];
    if (!art || !front() || !S.open) return;
    lightRun?.cancel();
    litArt?.removeAttribute("data-lit");
    art.setAttribute("data-lit", "");
    litArt = art;
    paintRoom();
    screen.style.setProperty("--room-lit", "1");
    if (reducedFor(screen)) {
      lightRun = screen.animate([{ "--room-lit": "0" }, { "--room-lit": "1" }], { duration: ms("short"), easing: curveOf("standard") });
      return;
    }
    const steps = strikeSteps();
    const easing = settings().flicker ? "step-end" : "linear";
    lightRun = screen.animate(
      [{ "--room-lit": "0", offset: 0, easing }, ...steps.map((value, i) => ({ "--room-lit": String(value), offset: (i + 1) / (steps.length + 1), easing })), { "--room-lit": "1", offset: 1 }],
      { duration: ms("strike") },
    );
  };
  /** Strikes once the walk has landed; every step before then puts it off again, so only the lamp
      you stop at strikes, and the room stays dark while you move. */
  const strikeAfter = (delay: number) => {
    window.clearTimeout(strikeTimer);
    strikeTimer = window.setTimeout(strike, delay);
  };
  /** The room lit at once on the picture in front of you: a still, or a gallery drawn open. */
  const litAtOnce = () => {
    const art = arts[S.folder]?.[S.at[S.folder]!];
    litArt?.removeAttribute("data-lit");
    if (art && front()) {
      art.setAttribute("data-lit", "");
      litArt = art;
    }
    screen.style.setProperty("--room-lit", S.open && front() ? "1" : "0");
    paintRoom();
  };

  const bare = options.bare ?? false;
  const drawWall = () => {
    floors.style.setProperty("--floor", String(S.folder));
    arts.forEach((here, level) => {
      const track = here[0]?.parentElement;
      track?.style.setProperty("--at", String(S.at[level]));
      here.forEach((art, index) => {
        const on = level === S.folder && index === S.at[level];
        art.toggleAttribute("data-on", on);
        const desk = desks.get(art);
        if (!desk) return;
        desk.element.style.setProperty("--aside", String(Math.max(-1, Math.min(1, index - S.at[level]!))));
        if (on && !bare) drawPlacard(art, desk, level, index);
      });
    });
    if (bare) return;
    floorsPanel.replaceChildren(
      list(
        [...folders.keys()].reverse().map((level) => ({ title: folders[level]!.name, icon: folders[level]!.icon, detail: String(folders[level]!.pictures.length) })),
        { current: folders.length - 1 - S.folder, onTake: (index) => pickFolder(folders.length - 1 - index) },
      ),
    );
  };

  const drawPlacard = (art: HTMLElement, desk: Desk, level: number, index: number) => {
    const hung = desk.hung;
    const flat = S.flat.has(hung.name);
    const reduced = reducedFor(screen);
    const switched = switchControl({ on: Boolean(hung.wallpaper.depth) && !flat, disabled: !hung.wallpaper.depth });
    tipOn(switched, "Depth", "D");
    switched.addEventListener("click", (event) => {
      event.stopPropagation();
      void toggleDepth(false);
    });
    const words = stack("s4", "start", text("Depth", "body_strong"), text(depthWords(hung, flat), "small", "on_surface_variant"));
    words.classList.add("gl-depth-words");
    const placard = art.querySelector<HTMLElement>(".gl-placard")!;
    placard.replaceChildren(
      stack("s4", "start", kicker(folders[level]!, index), text(hung.name, "title"), chipsFor(hung, S.applied, flat, reduced)),
      row("s12", "centre", switched, words),
    );
  };

  const drawBar = () => {
    if (bare) return;
    if (editor?.editing()) {
      bar.replaceChildren(...editor.bar());
      bar.toggleAttribute("data-lifted", false);
      if (tipFor && !tipFor.isConnected) hideTip(tipFor);
      return;
    }
    const hung = front();
    const folder = folders[S.folder]!;
    const now = document.createElement("span");
    now.className = "gl-bar-now";
    const before = iconButton({ icon: "chevron_left", label: "The picture before", kind: "Text", size: "Sm", hint: false, disabled: S.at[S.folder] === 0 });
    const after = iconButton({ icon: "chevron_right", label: "The picture after", kind: "Text", size: "Sm", hint: false, disabled: S.at[S.folder]! >= folder.pictures.length - 1 });
    tipOn(before, "The picture before", "←");
    tipOn(after, "The picture after", "→");
    before.addEventListener("click", (event) => (event.stopPropagation(), move(-1)));
    after.addEventListener("click", (event) => (event.stopPropagation(), move(1)));
    const words: HTMLElement[] = hung ? [text(hung.name, "body_strong"), kicker(folder, S.at[S.folder]!)] : [text(folder.name, "body_strong")];
    if (hung?.wallpaper.depth) {
      const flat = S.flat.has(hung.name);
      const depth = chip({ label: flat ? "Depth off" : "Depth", kind: flat ? "Neutral" : "Cold", icon: "deployed_code", press: "press" });
      tipOn(depth, "Depth", "D");
      depth.addEventListener("click", (event) => (event.stopPropagation(), void toggleDepth(true)));
      words.push(depth);
    }
    now.append(before, ...words, after);
    const views = tabs({ choices: VIEWS.map((view) => ({ label: VIEW_NAMES[view] })), picked: VIEWS.indexOf(S.view), onPick: (index) => void setView(VIEWS[index]!) });
    views.querySelectorAll<HTMLElement>(".k-tab").forEach((tab, index) => tipOn(tab, VIEW_NAMES[VIEWS[index]!], String(index + 1)));
    // Both of the step's words, the one the bar isn't in shut to nothing.
    const stepper = document.createElement("span");
    stepper.className = "gl-step-pair";
    for (const [label, full] of [["Full screen", true], ["Back to the wall", false]] as const) {
      const made = button({ label, kind: "Text", size: "Sm", icon: "desktop_windows" });
      tipOn(made, label, "F");
      made.addEventListener("click", (event) => (event.stopPropagation(), void step(full)));
      const cell = document.createElement("span");
      cell.append(made);
      stepper.append(cell);
    }
    const actions = buttonBar({
      dismiss: "Cancel",
      actions: [...(S.view === "lock" ? [{ label: S.awake ? "Rest" : "Wake", kind: "Text" as const }, { label: "Clock", kind: "Text" as const, icon: "schedule" }] : []), { label: "Apply", kind: "Filled" }],
      size: "Sm",
    });
    tipOn(actions.dismiss!, "Cancel", "Esc");
    actions.dismiss!.addEventListener("click", (event) => (event.stopPropagation(), void cancel()));
    const apply = actions.actions.at(-1)!;
    tipOn(apply, "Apply", "↵");
    apply.addEventListener("click", (event) => (event.stopPropagation(), void applyIt()));
    if (S.view === "lock") {
      const waker = actions.actions[0]!;
      tipOn(waker, S.awake ? "Rest" : "Wake", "W");
      waker.addEventListener("click", (event) => (event.stopPropagation(), void wake(!S.awake)));
      const clock = actions.actions[1]!;
      tipOn(clock, "Clock", "E");
      clock.addEventListener("click", (event) => (event.stopPropagation(), void editor?.enter()));
    }
    const rule = () => {
      const made = document.createElement("span");
      made.className = "gl-bar-rule";
      return made;
    };
    // The full screen's lead is always there, shut to nothing on the wall (`data-full`).
    const lead = document.createElement("span");
    lead.className = "gl-bar-lead";
    const leadInner = document.createElement("span");
    leadInner.append(now, rule());
    lead.append(leadInner);
    bar.replaceChildren(lead, views, stepper, rule(), actions.element);
    bar.toggleAttribute("data-lifted", S.full && S.view === "lock" && S.awake);
    if (tipFor && !tipFor.isConnected) hideTip(tipFor);
  };

  /** The bar's full-screen look, at once or over the picture's step (`time`). */
  const barFull = (full: boolean, time = 0) => {
    screen.style.setProperty("--step-time", `${time}ms`);
    screen.style.setProperty("--step-curve", curveOf(reducedFor(screen) ? "standard" : "emphasized_decel"));
    screen.toggleAttribute("data-full", full);
  };

  const draw = () => {
    screen.toggleAttribute("data-open", S.open);
    screen.toggleAttribute("data-covering", !S.open);
    barFull(S.full);
    screen.toggleAttribute("data-stills", S.stills);
    screen.style.setProperty("--out", S.full ? "1" : "0");
    drawWall();
    drawBar();
  };

  // Each picture's look in the current view, at once: the phase, the push and its colours.
  const settle = () => {
    for (const [art, desk] of desks) look(art, desk);
    deskAt()?.element.toggleAttribute("data-live", !S.open);
  };
  function look(art: HTMLElement, desk: Desk): void {
    {
      const main = art === arts[S.folder]![S.at[S.folder]!];
      desk.view = S.view;
      desk.element.dataset.phase = main && S.view === "lock" && S.awake ? "awake" : PHASES[S.view];
      desk.element.toggleAttribute("data-open", main && S.view === "lock" && S.awake);
      for (const part of desk.element.querySelectorAll<HTMLElement>(".k-sheet-part")) part.style.setProperty("--reveal", main && S.view === "lock" && S.awake ? "1" : "0");
      if (S.view === "picture") desk.element.style.setProperty("--push", main && !reducedFor(screen) ? "1" : "0");
      else desk.element.style.removeProperty("--push");
      paintDesk(desk, mode, false);
      const level = arts.findIndex((here) => here.includes(art));
      desk.element.style.setProperty("--aside", String(Math.max(-1, Math.min(1, arts[level]!.indexOf(art) - S.at[level]!))));
    }
  }

  // The moves.

  /** Every picture from the view it is in to `to`: the one in front with the whole move. */
  const viewMoves = (to: View): Promise<void> => {
    const from = S.view;
    const reduced = reducedFor(screen);
    const runs: Promise<void>[] = [];
    for (const [art, desk] of desks) {
      const main = art === arts[S.folder]![S.at[S.folder]!];
      const tracks = viewTracks(from, to, main, reduced, main && S.awake);
      desk.view = to;
      desk.element.dataset.phase = PHASES[to];
      desk.element.toggleAttribute("data-open", false);
      if (to === "picture") desk.element.style.setProperty("--push", main && !reduced ? "1" : "0");
      else desk.element.style.removeProperty("--push");
      paintDesk(desk, mode, true);
      runs.push(play(desk.element, tracks));
    }
    S.awake = false;
    S.view = to;
    return Promise.all(runs).then(() => undefined);
  };

  async function setView(to: View): Promise<void> {
    if (!S.open || S.busy || to === S.view || !front()) return;
    S.busy = true;
    const done = viewMoves(to);
    drawBar();
    await done;
    S.busy = false;
  }

  /** Waking the lock in front of you, or letting it rest: the Lock's own moves. */
  async function wake(on: boolean): Promise<void> {
    if (!S.open || S.busy || S.view !== "lock" || on === S.awake) return;
    const desk = deskAt();
    if (!desk) return;
    S.busy = true;
    S.awake = on;
    const reduced = reducedFor(screen);
    desk.element.dataset.phase = on ? "awake" : "rest";
    desk.element.toggleAttribute("data-open", on);
    const tracks = on ? (reduced ? wakingReduced(MAIN) : waking(MAIN)) : reduced ? reducedRest(MAIN) : resting(MAIN);
    for (const part of desk.element.querySelectorAll<HTMLElement>(".k-sheet-part")) part.style.setProperty("--reveal", on ? "1" : "0");
    drawBar();
    await play(desk.element, tracks);
    S.busy = false;
  }

  /** Stepping out to the full screen and back: the room grows round the picture in front of you. */
  async function step(full: boolean): Promise<void> {
    if (!S.open || S.busy || full === S.full || !front()) return;
    S.busy = true;
    S.full = full;
    drawBar();
    const time = full ? ms("slow") : ms("long");
    // The bar changes with the picture's step, over its time and curve; with reduce motion a short fade.
    barFull(full, reducedFor(screen) ? ms("short") : time);
    await stepTo(full ? 1 : 0, time);
    S.busy = false;
  }

  /** `--out` to `to`; with reduce motion the screen fades out and back in round the change. */
  async function stepTo(to: number, duration: number): Promise<void> {
    if (reducedFor(screen)) {
      await ease(room, "opacity", 0, ms("short"), "standard");
      screen.style.setProperty("--out", String(to));
      await ease(room, "opacity", 1, ms("short"), "standard");
      room.style.removeProperty("opacity");
      return;
    }
    await ease(screen, "--out", to, duration, "emphasized_decel");
  }

  function move(by: number): void {
    const folder = folders[S.folder]!;
    if (!S.open || S.busy || folder.pictures.length === 0) return;
    const to = Math.min(Math.max(S.at[S.folder]! + by, 0), folder.pictures.length - 1);
    if (to === S.at[S.folder]) return;
    if (S.awake) {
      S.awake = false;
      settle();
    }
    const was = deskAt();
    S.at[S.folder] = to;
    ensureNear();
    lightsOff();
    landOn(was);
    editor?.swap();
    strikeAfter(reducedFor(screen) ? ms("short") : ms("long"));
  }

  function pickFolder(level: number): void {
    if (!S.open || S.busy || level === S.folder || level < 0 || level >= folders.length) return;
    if ((S.view !== "picture" || S.full) && folders[level]!.pictures.length === 0) return;
    const was = deskAt();
    S.folder = level;
    ensureNear();
    lightsOff();
    landOn(was);
    strikeAfter(reducedFor(screen) ? ms("short") : ms("long"));
  }

  /** A new picture in front of you: the wall slides, the one you leave sinks back, this one lifts. */
  function landOn(was: Desk | undefined): void {
    const now = deskAt();
    const reduced = reducedFor(screen);
    if (S.view === "picture" && !reduced) {
      if (was && was !== now) void ease(was.element, "--push", 0, ms("long"), "emphasized_decel");
      if (now) void ease(now.element, "--push", 1, ms("long"), "emphasized_decel");
    }
    drawWall();
    drawBar();
  }

  /** D: the city settles onto the sky or lifts off it; on a lock its clock fades out and back. */
  async function toggleDepth(fromKey: boolean): Promise<void> {
    const desk = deskAt();
    const hung = front();
    if (!S.open || S.busy || !desk || !hung?.wallpaper.depth) return;
    S.busy = true;
    const flat = !S.flat.has(hung.name);
    if (flat) S.flat.add(hung.name);
    else S.flat.delete(hung.name);
    if (fromKey) arts[S.folder]![S.at[S.folder]!]!.querySelector<HTMLButtonElement>(".k-switch")?.click();
    const words = arts[S.folder]![S.at[S.folder]!]!.querySelector(".gl-depth-words > :last-child");
    words?.replaceWith(text(depthWords(hung, flat), "small", "on_surface_variant"));
    drawBar();
    await flattenEased(desk, flat);
    drawWall();
    S.busy = false;
  }

  async function flattenEased(desk: Desk, flat: boolean): Promise<void> {
    const reduced = reducedFor(screen);
    const clock = desk.element.querySelector<HTMLElement>(".k-lock-clock")!;
    const lockShown = desk.view === "lock";
    if (lockShown) await ease(clock, "opacity", 0, ms("short"), "standard");
    desk.element.removeAttribute("data-flat");
    await ease(desk.element, "--depth", flat ? 0 : 1, reduced ? 0 : ms("long"), "emphasized_decel");
    desk.element.toggleAttribute("data-flat", flat);
    if (lockShown) {
      await ease(clock, "opacity", 1, ms("short"), "standard");
      clock.style.removeProperty("opacity");
    }
  }

  /** Opening: the live desktop steps back into its frame as the wall builds round it. */
  async function open(): Promise<void> {
    if (S.open || S.busy) return;
    S.busy = true;
    S.open = true;
    S.full = false;
    S.stills = false;
    // The room and the bar take the wall's look now; the step back into it is `--out`, which stays
    // where the live desktop has it until the move eases it.
    screen.toggleAttribute("data-open", true);
    screen.toggleAttribute("data-stills", false);
    // The controls are in their places from the first frame, covered by the desktop until its edge
    // passes over them; with reduce motion they fade in with the room.
    screen.toggleAttribute("data-covering", !reducedFor(screen));
    barFull(false);
    drawWall();
    drawBar();
    // The stills are asked for now and land while the gallery opens.
    window.setTimeout(() => screen.toggleAttribute("data-stills", (S.stills = true)), ms("medium") + ms("slow") + ms("short"));
    // The room starts dark; the lamp strikes as the desktop lands in its frame.
    lightRun?.cancel();
    screen.style.setProperty("--room-lit", "0");
    strikeAfter(reducedFor(screen) ? ms("short") : ms("slow"));
    const opened = viewMoves("picture");
    // The bar shows the view the wall opens in, Picture, from the first frame.
    drawBar();
    if (reducedFor(screen)) controls.animate([{ opacity: 0 }, { opacity: 0, offset: 0.5 }, { opacity: 1 }], { duration: 2 * ms("short"), easing: curveOf("standard") });
    await Promise.all([opened, stepTo(0, ms("slow"))]);
    screen.toggleAttribute("data-covering", false);
    S.busy = false;
  }

  /** Steps out to the desktop of the picture in front of you, and shuts. */
  async function close(): Promise<void> {
    // The lamp goes off as the picture grows back into the desktop; the bar and the shortcuts'
    // button fade, the bar keeping its words meanwhile.
    lightsOff();
    shortcutsMade.set(false);
    // On the full screen the controls lie over the picture, which can't cover them: they go with a
    // short fade, as they do with reduce motion; on the wall the growing picture covers them.
    if (S.full || reducedFor(screen)) await ease(controls, "opacity", 0, ms("short"), "standard");
    screen.toggleAttribute("data-covering", true);
    screen.toggleAttribute("data-open", false);
    const toDesk = S.view === "desktop" ? Promise.resolve() : viewMoves("desktop");
    const stepped = S.full ? Promise.resolve() : stepTo(1, ms("slow"));
    S.full = true;
    await Promise.all([toDesk, stepped]);
    S.open = false;
    shortcutsMade.set(false);
    settle();
    draw();
    controls.style.removeProperty("opacity");
  }

  /** Enter: the picture in front of you becomes the desktop. */
  async function applyIt(): Promise<void> {
    if (!S.open || S.busy || !front()) return;
    S.busy = true;
    S.applied = front()!;
    await close();
    S.busy = false;
  }

  /** Esc: back along the wall to the wallpaper you had, and its frame grows back into the desktop. */
  async function cancel(): Promise<void> {
    if (!S.open || S.busy) return;
    S.busy = true;
    const level = Math.max(0, folders.findIndex((each) => each.pictures.includes(S.applied)));
    const index = folders[level]!.pictures.indexOf(S.applied);
    if (level !== S.folder || index !== S.at[level]) {
      if (S.full) {
        S.full = false;
        drawBar();
        await stepTo(0, ms("long"));
      }
      const was = deskAt();
      lightsOff();
      S.folder = level;
      S.at[level] = index;
      ensureNear();
      landOn(was);
      await wait(ms("long"));
    }
    await close();
    S.busy = false;
  }

  // The keys: the shortcuts take ? and their Esc first, then the gallery's own, before any control.
  if (live) {
    takeShortcutKeys(screen, shortcutsMade);
    screen.addEventListener(
      "keydown",
      (event) => {
        const key = event.key;
        const own = event.target === screen;
        if (!S.open) {
          if (key === " " || key === "Enter") {
            event.preventDefault();
            void open();
          }
          return;
        }
        if (editor?.editing()) {
          const owner = editor.key(event);
          if (owner === "control") return;
          if (owner === "taken") {
            event.preventDefault();
            event.stopPropagation();
            return;
          }
        }
        const act = (): boolean => {
          if (key === "e" || key === "E") {
            if (S.view !== "lock" || S.busy || !front()) return false;
            void editor?.enter();
          } else if (key === "ArrowLeft" || key === "h") move(-1);
          else if (key === "ArrowRight" || key === "l") move(1);
          else if (key === "ArrowUp" || key === "k") pickFolder(S.folder + 1);
          else if (key === "ArrowDown" || key === "j") pickFolder(S.folder - 1);
          else if (key === "1" || key === "2" || key === "3") void setView(VIEWS[Number(key) - 1]!);
          else if (key === "f" || key === "F" || (key === " " && own)) void step(!S.full);
          else if (key === "d" || key === "D") void toggleDepth(true);
          else if (key === "w" || key === "W") void wake(!S.awake);
          else if (key === "Enter" && own) void applyIt();
          else if (key === "Escape") void cancel();
          else return false;
          return true;
        };
        if (act()) {
          event.preventDefault();
          event.stopPropagation();
        }
      },
      true,
    );
    screen.addEventListener("click", (event) => {
      if (!S.open && event.target instanceof Node && !bar.contains(event.target)) void open();
      // A click in the clock's panel leaves the keyboard on the control it pressed.
      if (event.target instanceof Element && event.target.closest(".ce-panel")) return;
      screen.focus({ preventScroll: true });
    });
  }

  // The clock's editor, on a live gallery or one drawn with it open.
  if (live || options.edit) {
    editor = clockEditor({
      screen,
      front: () => {
        const art = arts[S.folder]?.[S.at[S.folder]!];
        const desk = art ? desks.get(art) : undefined;
        if (!art || !desk) return undefined;
        return { lock: desk.lock, wallpaper: desk.hung.wallpaper, name: desk.hung.name, frame: art.querySelector<HTMLElement>(".gl-frame")! };
      },
      look: (shown) => lookOf(folders.flatMap((folder) => folder.pictures).find((hung) => hung.name === shown.name) ?? front()!),
      setLook: (shown, set) => looks.set(shown.name, set),
      wallFull: () => S.full,
      leaveWallFull: (time) => {
        S.full = false;
        barFull(false, time);
        void ease(screen, "--out", 0, time, reducedFor(screen) ? "standard" : "emphasized_decel");
      },
      awake: () => S.awake,
      wake,
      drawBar,
      tipOn,
      reduced: () => reducedFor(screen),
      live,
    });
  }

  // The first frame: the pictures that show, the room lit on the front one, or dark while shut.
  ensureNear();
  litAtOnce();
  paintLamps();
  settle();
  draw();
  requestAnimationFrame(() => {
    paintRoom();
    paintLamps();
    for (const desk of desks.values()) paintDesk(desk, modeOf(screen), false);
  });
  // A live gallery builds the rest of its wall while idle, one picture a time, so browsing never
  // waits on one.
  if (live) {
    const idle = (work: () => void) => requestIdleCallback(work);
    const rest = arts.flat().filter((art) => makers.has(art) && !desks.has(art));
    const next = () => {
      const art = rest.shift();
      if (!art) return;
      if (!S.busy) ensure(art);
      else rest.push(art);
      idle(next);
    };
    idle(next);
  }
  if (options.shortcuts) shortcutsMade.set(true);
  if (editor && options.edit) {
    const edit = options.edit;
    if ("from" in edit) editor.pose(edit.to, edit.from);
    else editor.pose(edit);
  }
  return screen;
}

/** Turns a picture's depth off at once: a sample of it flat. */
function flatten(desk: Desk, flat: boolean): void {
  desk.element.style.setProperty("--depth", flat ? "0" : "1");
  desk.element.toggleAttribute("data-flat", flat);
}
