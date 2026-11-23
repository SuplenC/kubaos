// The Settings window as its pages draw it: SidebarPage holding Settings' outline, in an app's
// window, still at a size or live and resizable; and where a sample lands on a setting.

import { type SidebarPageOptions, sidebarPage } from "../../../lib/sidebar-page.ts";
import { el } from "../../../site/dom.ts";
import { paintBetween, palette } from "../../../site/recolour.ts";
import type { Mode } from "../../../site/settings.ts";
import { resizable, still } from "../../templates/window.ts";
import { defaults, outline, sectionData, type Values } from "./settings-data.ts";
import { placeOf, type ResetPins } from "./settings-parts.ts";
import { showGlass } from "./sections/appearance.ts";

/** Where the window stands as it is drawn, and the values it holds. */
export interface WindowOptions extends Omit<SidebarPageOptions, "outline"> {
  readonly values?: Partial<Values>;
  /** A reset drawn lit or held: a sample's. */
  readonly resets?: ResetPins;
}

/**
 * What the live windows open with: a few settings off their defaults, so their resets show on
 * arrival. Power: two in Battery, one in Sleep & lock; Appearance: three.
 */
export function seeded(): Partial<Values> {
  return { chargeLimit: 0, lowAt: 2, lock: [2, 3], wallpaperMotion: 0, menuMotion: 1, cursorSize: 4 };
}

/** A window size, as lengths' CSS. */
export interface Size {
  readonly width: string;
  readonly height: string;
}

export const MINIMUM: Size = { width: "var(--size-app_min_width)", height: "var(--size-app_min_height)" };
export const DEFAULT: Size = { width: "var(--size-app_width)", height: "var(--size-app_height)" };
/** A laptop's screen and a 1080p one: the screens' sizes, not design values. */
export const LAPTOP: Size = { width: "calc(1280 * var(--px))", height: "calc(800 * var(--px))" };
export const FULL_HD: Size = { width: "calc(1920 * var(--px))", height: "calc(1080 * var(--px))" };

/** The values a window opens with: the defaults, as `options` change them. */
function valuesOf(options: WindowOptions): Values {
  return { ...defaults(), ...options.values };
}

/** The template holding Settings, its values wired to `frame` once it is in one. */
function drawn(options: WindowOptions, values: Values): HTMLElement {
  const { values: _, resets, ...rest } = options;
  // A sub-page drawn open holds the sidebar's bar on the sub-section it was opened from.
  const data = options.section === undefined || !values.opened ? null : sectionData(values, options.section);
  const page = data?.pages?.find((each) => each.id === values.opened);
  const heldSub = page && data ? data.subs.findIndex((each) => each.title === page.back) : -1;
  return sidebarPage({ outline: outline(values, resets), ...rest, ...(heldSub >= 0 ? { heldSub } : {}) });
}

/** The window's frame put where the values recolour it, in the mode they are drawn in. */
function framed(frame: HTMLElement, values: Values): void {
  values.window = frame;
  if (values.drawn === "light") frame.dataset.mode = "light";
  showGlass(values);
}

/** The Settings window, still, at `size`: a state's sample. */
export function settingsWindow(options: WindowOptions = {}, size: Size = DEFAULT): HTMLElement {
  const values = valuesOf(options);
  const frame = still(drawn(options, values), size.width, size.height);
  framed(frame, values);
  return frame;
}

/** The Settings window, live, in a frame that resizes by its corner from the toolbar's window size. */
export function liveWindow(options: WindowOptions = {}): HTMLElement {
  const values = valuesOf({ ...options, values: { ...seeded(), ...options.values } });
  const room = resizable(drawn({ focus: options.section === undefined, ...options }, values));
  framed(room.querySelector<HTMLElement>(".t-window")!, values);
  return room;
}

/** The template alone, for the reflow, which puts it in a window of the toolbar's size. */
export function bareWindow(options: WindowOptions = {}): HTMLElement {
  return drawn(options, valuesOf(options));
}

/**
 * The options that open the window on setting `id` of section `section`, as Home's search or the
 * Command menu lands on it: scrolled to its sub-section, the keyboard on its row, the row lit by the
 * landed highlight.
 */
export function landedOn(section: number, id: string, values: Partial<Values> = {}): WindowOptions {
  const place = placeOf(sectionData({ ...defaults(), ...values }, section), id);
  return { section, sub: place.sub, row: place.row, landed: place.row, keyboard: "page", values };
}

/** The keyboard on setting `id`'s row, the row lit as the keyboard lights it, with no landed highlight. */
export function litOn(section: number, id: string, values: Partial<Values> = {}): WindowOptions {
  const place = placeOf(sectionData({ ...defaults(), ...values }, section), id);
  return { section, sub: place.sub, row: place.row, keyboard: "page", values };
}

/** The window as a sequence's scene: drawn in `from`, painted to move along `path` to `to`. */
export function sceneOf(from: { theme: string; mode: Mode }, to: { theme: string; mode: Mode }, options: WindowOptions): () => Element {
  return () => {
    const frame = settingsWindow({ ...options, values: { ...options.values, drawn: from.mode } });
    const scene = el("div", "page-scene", frame);
    scene.dataset.mode = from.mode;
    delete frame.dataset.mode;
    paintBetween(scene, "dawn", palette(from.theme, from.mode), palette(to.theme, to.mode));
    return el("div", "page-scene-box", scene);
  };
}
