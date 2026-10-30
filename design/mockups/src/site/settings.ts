// The toolbar's settings: what every page is shown at. Each one is read from the URL first, so a
// link or a screenshot names its state, then from what was last picked, then its default.
//
// Mode, scale and motion apply through CSS, and a theme or mode picked plays the theme change or
// the dark ⇄ light switch across the page (`site/change.ts`): the page's box carries them as attributes and custom
// properties, and every length, colour and duration follows at once. Text length changes the words
// on the page, so it draws the page again; nothing ever reloads.

/**
 * Whether the toolbar and the navigation show. `?chrome=off` draws the page alone, as the shots
 * draw it, and holds its motion players still on their first state, so a shot is the same every
 * time.
 */
export const CHROME = new URLSearchParams(location.search).get("chrome") !== "off";

import { THEMES } from "./themes.ts";

export type Mode = "dark" | "light";
export type TextLength = "short" | "long" | "translated";

/** The scales a display may take (`rules.scales`): every length lands on whole device pixels. */
export const SCALES = [1, 1.25, 1.5, 2] as const;
export type Scale = (typeof SCALES)[number];

/** The speed setting's range (`kubaos_ui::motion::Motion::speed`). */
export const SPEEDS = [0.5, 1, 1.5, 2] as const;
export type Speed = (typeof SPEEDS)[number];

/** A window size to show a template or a screen in, by the size tokens it comes from. */
export interface WindowSize {
  readonly name: string;
  readonly width: string;
  readonly height: string;
}

/**
 * The window sizes: an app's minimum and its default (`docs/DESIGN.md`, "Space and shape"), and
 * two screens a window is tiled or full on, a laptop's and a 1080p one, whose sizes are the
 * screens', not design values.
 */
export const WINDOWS: readonly WindowSize[] = [
  { name: "Minimum", width: "var(--size-app_min_width)", height: "var(--size-app_min_height)" },
  { name: "Default", width: "var(--size-app_width)", height: "var(--size-app_height)" },
  { name: "Laptop", width: "calc(1280 * var(--px))", height: "calc(800 * var(--px))" },
  { name: "Full HD", width: "calc(1920 * var(--px))", height: "calc(1080 * var(--px))" },
];

export interface Settings {
  mode: Mode;
  /** The theme every page is painted in, by its slug (`site/themes.ts`). */
  theme: string;
  scale: Scale;
  reduced: boolean;
  speed: Speed;
  flicker: boolean;
  text: TextLength;
  window: string;
}

const DEFAULTS: Settings = {
  mode: "dark",
  theme: THEMES[0]!.slug,
  scale: 1,
  reduced: false,
  speed: 1,
  flicker: true,
  text: "short",
  window: "Default",
};

const STORED = "kubaos-mockups";

type Listener = (settings: Settings, changed: ReadonlySet<keyof Settings>) => void;

function parsed(raw: Record<string, string | null | undefined>): Partial<Settings> {
  const found: Partial<Settings> = {};
  if (raw.mode === "dark" || raw.mode === "light") found.mode = raw.mode;
  if (THEMES.some((theme) => theme.slug === raw.theme) && raw.theme) found.theme = raw.theme;
  const scale = SCALES.find((s) => String(s) === raw.scale);
  if (scale !== undefined) found.scale = scale;
  if (raw.reduced === "on" || raw.reduced === "off") found.reduced = raw.reduced === "on";
  const speed = SPEEDS.find((s) => String(s) === raw.speed);
  if (speed !== undefined) found.speed = speed;
  if (raw.flicker === "on" || raw.flicker === "off") found.flicker = raw.flicker === "on";
  if (raw.text === "short" || raw.text === "long" || raw.text === "translated") found.text = raw.text;
  if (WINDOWS.some((w) => w.name === raw.window) && raw.window) found.window = raw.window;
  return found;
}

function stored(): Partial<Settings> {
  try {
    return parsed(JSON.parse(localStorage.getItem(STORED) ?? "{}") as Record<string, string>);
  } catch {
    return {};
  }
}

function fromUrl(): Partial<Settings> {
  const query = new URLSearchParams(location.search);
  return parsed(Object.fromEntries(query.entries()));
}

function serialised(settings: Settings): Record<string, string> {
  return {
    mode: settings.mode,
    theme: settings.theme,
    scale: String(settings.scale),
    reduced: settings.reduced ? "on" : "off",
    speed: String(settings.speed),
    flicker: settings.flicker ? "on" : "off",
    text: settings.text,
    window: settings.window,
  };
}

let current: Settings = { ...DEFAULTS, ...stored(), ...fromUrl() };
const listeners: Listener[] = [];

export function settings(): Settings {
  return current;
}

/** Changes `patch`'s settings, keeps them for the next visit and in the URL, and tells listeners. */
export function change(patch: Partial<Settings>): void {
  const next = { ...current, ...patch };
  const changed = new Set(
    (Object.keys(patch) as (keyof Settings)[]).filter((key) => next[key] !== current[key]),
  );
  if (changed.size === 0) return;
  current = next;
  const values = serialised(current);
  localStorage.setItem(STORED, JSON.stringify(values));
  const url = new URL(location.href);
  for (const [key, value] of Object.entries(values)) url.searchParams.set(key, value);
  history.replaceState(null, "", url);
  // A copy: a listener may stop listening while it is called.
  for (const listener of [...listeners]) listener(current, changed);
}

/** Calls `listener` on every change, until the function it returns is called. */
export function onChange(listener: Listener): () => void {
  listeners.push(listener);
  return () => {
    const at = listeners.indexOf(listener);
    if (at >= 0) listeners.splice(at, 1);
  };
}

/**
 * Puts the settings that CSS applies on the page's box: its mode, its scale, and the motion
 * settings every duration reads (`motion.css`).
 */
export function apply(page: HTMLElement): void {
  document.documentElement.dataset.mode = current.mode;
  page.dataset.mode = current.mode;
  page.style.setProperty("--scale", String(current.scale));
  page.style.setProperty("--speed", String(current.speed));
  page.toggleAttribute("data-reduced", current.reduced);
  page.toggleAttribute("data-flicker", current.flicker && !current.reduced);
}

/** The window size picked, by its name. */
export function windowSize(): WindowSize {
  return WINDOWS.find((w) => w.name === current.window) ?? WINDOWS[1]!;
}
