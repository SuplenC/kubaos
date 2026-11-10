// The bar's parts as `config/quickshell/modules/bar/` draws them, for the shell's pages: every pill
// with its content, the tray, the window title, and the whole bar along a screen's top. The data is
// the shots' stand-ins' (`test/shots/fakes`): Tuesday 27 October, 21:47, recording for 1:23.

import "./bar.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { badgedIcon } from "../../lib/badge.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { icon } from "../../lib/icon.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { row } from "../../lib/layout.ts";
import { meter } from "../../lib/meter.ts";
import { sparkline, minute } from "../../lib/meter.ts";
import { makePill, type Pill, type PillOptions, pill } from "../../lib/pill.ts";
import { type Screen, screen } from "../../lib/sheet.ts";
import { advance, clamped, text } from "../../lib/text.ts";
import { APP_ICONS } from "../../site/app-icons.ts";
import { workspaces, type WorkspacesOptions, SHOTS_GROUPS } from "./workspaces.ts";

export { wallpaper };

/** A symbol at the bar's 16 in a colour role. */
export function glyph(name: string, color = "on_surface_variant"): SVGElement {
  return icon(name, "var(--icon-s16)", `var(--color-${color})`);
}

type Interactive = Omit<PillOptions, "content" | "lone" | "interactive" | "tooltip" | "pinned">;

/** An interactive pill in `state`, saying `tooltip` while it is lit. */
function control(content: Element[], tooltip: string, state: Shown, options: Interactive & { lone?: boolean } = {}): HTMLElement {
  return pill({ lone: false, ...options, content, interactive: true, tooltip, ...stateOptions(state) });
}

// The clock.

export const CLOCK = { kanji: "火", date: "Tue 27 Oct", time: "21:47", countdown: "Vacation 52 d 2 h" } as const;

export interface ClockOptions {
  readonly countdown?: string;
  readonly date?: string;
  readonly time?: string;
  readonly state?: Shown;
}

/** The weekday's kanji in `accent`, the date in `body`, the time in `mono_large`, a countdown after. */
export function clockPill(options: ClockOptions = {}): HTMLElement {
  return control(clockContent(options), "Tuesday, 27 October 2026", options.state ?? "rest");
}

export function clockContent(options: ClockOptions = {}): Element[] {
  const parts: Element[] = [text(CLOCK.kanji, "accent", "tertiary"), text(options.date ?? CLOCK.date, "body", "on_surface_variant"), text(options.time ?? CLOCK.time, "mono_large", "on_surface")];
  if (options.countdown) parts.push(text(options.countdown, "body", "tertiary"));
  return parts;
}

// The status pill.

export interface StatusOptions {
  readonly power?: string;
  readonly net?: string;
  readonly bluetooth?: string | null;
  readonly audio?: string;
  /** The battery, or none on a desktop: its share, low or charging. */
  readonly battery?: { readonly percent: number; readonly low?: boolean; readonly charging?: boolean } | null;
  readonly state?: Shown;
}

export function statusContent(options: StatusOptions = {}): Element[] {
  const parts: Element[] = [glyph(options.power ?? "balance"), glyph(options.net ?? "signal_wifi_4_bar")];
  const bluetooth = options.bluetooth === undefined ? "bluetooth" : options.bluetooth;
  if (bluetooth) parts.push(glyph(bluetooth));
  parts.push(glyph(options.audio ?? "volume_up"));
  const battery = options.battery === undefined ? { percent: 64 } : options.battery;
  if (battery) {
    const tone = battery.low ? "error" : battery.charging ? "primary" : "on_surface_variant";
    const name = battery.charging ? "battery_charging_full" : `battery_${Math.min(3, Math.floor(battery.percent / 25))}_bar`;
    parts.push(row("s4", "centre", glyph(name, tone), text(`${battery.percent}%`, "mono", "on_surface")));
  }
  return parts;
}

export function statusPill(options: StatusOptions = {}): HTMLElement {
  return control(statusContent(options), "Home 5G  ·  Volume 60%  ·  Battery 64%", options.state ?? "rest");
}

// The bell.

export function bellPill(dnd = false, state: Shown = "rest"): HTMLElement {
  return control([glyph(dnd ? "notifications_off" : "notifications")], "", state, { lone: true });
}

/** The bell with its hint: only with nothing unread. */
export function quietBell(state: Shown = "rest"): HTMLElement {
  return control([glyph("notifications")], "No new notifications", state, { lone: true });
}

// The keyboard layout.

export function keyboardPill(code = "US", state: Shown = "rest"): HTMLElement {
  return control([text(code, "button_small", "on_surface")], `Typing with ${code}. Click or SUPER+SHIFT+Space for the next layout`, state);
}

// Keep awake and a scheduled action.

export interface IdleOptions {
  readonly keepAwake?: boolean;
  /** A scheduled action: its symbol and the time left. */
  readonly scheduled?: { readonly icon: string; readonly left: string };
  readonly state?: Shown;
}

export function idleContent(options: IdleOptions): Element[] {
  const parts: Element[] = [];
  if (options.keepAwake ?? true) parts.push(glyph("coffee", "tertiary"));
  if (options.scheduled) parts.push(glyph(options.scheduled.icon, "tertiary"), text(options.scheduled.left, "mono", "on_surface"));
  return parts;
}

export function idleOptions(options: IdleOptions): Interactive {
  return { end: options.scheduled ? "s12" : "s8", gap: "s4" };
}

export function idlePill(options: IdleOptions = {}): HTMLElement {
  const tip = options.scheduled ? "Suspend at 23:00. Click to cancel it" : "Keeping awake until you turn it off. Click to stop keeping awake";
  return control(idleContent(options), tip, options.state ?? "rest", idleOptions(options));
}

// The Proxy.

export function proxyPill(problem = false, state: Shown = "rest"): HTMLElement {
  return control(
    [badgedIcon({ name: problem ? "key" : "vpn_lock", color: "tertiary", badged: problem })],
    problem ? "Proxy: the tunnel dropped. Click for the control center" : "Proxy on: socks5 to vps.example.net. Click for the control center",
    state,
    { lone: true },
  );
}

// The update marker.

export function updatePill(count?: number, state: Shown = "rest"): HTMLElement {
  return control(
    [badgedIcon({ name: "system_update_alt", color: "tertiary", badged: true, ...(count === undefined ? {} : { count }) })],
    count === undefined ? "KubaOS 0.5 is ready. Click and I'll run kubaos update" : `${count} security updates are waiting`,
    state,
    { lone: true },
  );
}

// The live preview.

export const BRANCH = "issue-228-live-preview";

export function previewPill(branch = BRANCH, state: Shown = "rest"): HTMLElement {
  const name = clamped(branch, "label", "cold");
  name.classList.add("shell-preview-name");
  const made = control([icon("code", "var(--type-label-icon)", "var(--color-cold)"), name], `Live preview of feature/${branch}. Click to go back to the release`, state, { tone: "preview", gap: "s4" });
  made.classList.add("shell-preview");
  return made;
}

// Recording.

export interface RecordingOptions {
  readonly replay?: boolean;
  readonly elapsed?: string;
  readonly state?: Shown;
}

export function recordingContent(options: RecordingOptions = {}): Element[] {
  const dot = document.createElement("span");
  dot.className = "shell-dot";
  dot.toggleAttribute("data-hollow", options.replay ?? false);
  return [dot, text(options.replay ? "Replay 30 s" : (options.elapsed ?? "1:23"), "mono", "on_error_container")];
}

export function recordingPill(options: RecordingOptions = {}): HTMLElement {
  const tip = options.replay ? "Replay buffer on: SUPER+SHIFT+P saves the last 30 s. Click to turn off" : "Recording. Click to stop";
  return control(recordingContent(options), tip, options.state ?? "rest", { tone: "recording" });
}

export function recordingMade(options: RecordingOptions = {}): Pill {
  return makePill({ content: recordingContent(options), lone: false, tone: "recording", interactive: true, tooltip: "Recording. Click to stop" });
}

// The system monitor.

/** A reading: its symbol, its words, its share of its scale, and its step against its limit. */
export interface Reading {
  readonly icon: string;
  readonly text: string;
  readonly fill: number;
  readonly level?: 0 | 1 | 2;
  /** No scale of its own: a rate, drawn as numbers even among bars. */
  readonly unscaled?: boolean;
}

export const READINGS: readonly Reading[] = [
  { icon: "memory", text: "37%", fill: 0.37 },
  { icon: "memory_alt", text: "71%", fill: 0.71 },
  { icon: "thermostat", text: "61°", fill: 0.61 },
];

export type SysmonStyle = "numbers" | "bars" | "sparklines";

export interface SysmonOptions {
  readonly readings?: readonly Reading[];
  readonly style?: SysmonStyle;
  /** A disk nearly full: a red drive at the end. */
  readonly fullDisk?: boolean;
  /** Right-clicked: Hide in place of the readings. */
  readonly offering?: boolean;
  readonly state?: Shown;
}

function tintOf(level: number): string {
  return level === 2 ? "error" : level === 1 ? "warning" : "on_surface_variant";
}

/** A reading's words, right-aligned in a box as wide as "100%" at least. */
function readingWords(reading: Reading): HTMLElement {
  const box = document.createElement("span");
  box.className = "shell-reading";
  box.style.setProperty("--widest", String(advance("mono", "100%")));
  box.append(text(reading.text, "mono", (reading.level ?? 0) > 0 ? tintOf(reading.level ?? 0) : "on_surface"));
  return box;
}

function readingShape(seed: number): (second: number) => number {
  return (second) => 40 + 25 * Math.sin(second / (3 + seed)) + 10 * Math.sin(second * 1.7 + seed);
}

export function sysmonContent(options: SysmonOptions = {}): Element[] {
  if (options.offering) return [row("s4", "centre", glyph("visibility_off"), text("Hide", "label", "on_surface"))];
  const style = options.style ?? "numbers";
  const parts: Element[] = (options.readings ?? READINGS).map((reading, index) => {
    const level = reading.level ?? 0;
    const ink = level > 0 ? tintOf(level) : "primary";
    const shown: Element[] = [glyph(reading.icon, tintOf(level))];
    if (style === "bars" && !reading.unscaled) {
      const bar = meter([{ share: reading.fill, color: ink }]);
      bar.dataset.compact = "";
      shown.push(bar);
    } else if (style === "sparklines") {
      shown.push(sparkline({ samples: minute(600_000, readingShape(index)), color: ink, inline: true }).box);
    } else {
      shown.push(readingWords(reading));
    }
    return row("s4", "centre", ...shown);
  });
  if (options.fullDisk) parts.push(glyph("hard_drive", "error"));
  return parts;
}

export function sysmonPill(options: SysmonOptions = {}): HTMLElement {
  const tip = options.offering ? "" : "CPU 37%  ·  61 °C  ·  RAM 11.2 GB of 15.6 GB";
  return control(sysmonContent(options), tip, options.state ?? "rest", { gap: "s12" });
}

// The tray.

/** An app's picture in the tray: its own colours, or a grey one lifted to the content colour. */
export interface TrayApp {
  readonly title: string;
  readonly picture: string;
  /** Grey and under 3:1 on the bar: drawn in `on_surface` in its shape. */
  readonly faint?: boolean;
  /** No picture at all. */
  readonly missing?: boolean;
}

const [KUBAOS, , DISC, STAR] = APP_ICONS;

export const TRAY: readonly TrayApp[] = [
  { title: "Volume", picture: DISC!.picture, faint: true },
  { title: "Network", picture: STAR!.picture, faint: true },
  { title: "Files", picture: KUBAOS!.picture },
];

/** One app's circle, a chip tall, the picture at the bar's 16 in its middle. */
export function trayItem(app: TrayApp, pinned?: "lit" | "pressed"): HTMLElement {
  const item = document.createElement("span");
  item.className = "shell-tray-item";
  item.setAttribute("role", "button");
  if (pinned) item.dataset.pinned = pinned;
  let picture: HTMLElement;
  if (app.missing) {
    picture = document.createElement("span");
    picture.className = "shell-tray-picture";
  } else if (app.faint) {
    picture = document.createElement("span");
    picture.className = "shell-tray-picture";
    picture.dataset.lifted = "";
    picture.style.setProperty("--picture", `url("${app.picture}")`);
  } else {
    const image = document.createElement("img");
    image.className = "shell-tray-picture";
    image.src = app.picture;
    image.alt = "";
    picture = image;
  }
  item.append(picture);
  return item;
}

export function trayPill(apps: readonly TrayApp[] = TRAY, pinned?: { index: number; state: "lit" | "pressed" }): HTMLElement {
  return pill({ lone: false, end: "s4", gap: 0, content: apps.map((app, index) => trayItem(app, pinned?.index === index ? pinned.state : undefined)) });
}

// The power button.

export function powerButton(state: Shown = "rest"): HTMLElement {
  return iconButton({ icon: "power_settings_new", label: "Power", kind: "Tonal", size: "Sm", ...stateOptions(state) });
}

// The window title.

export const TITLE = "kubaos — nvim config/quickshell/modules/bar/Workspaces.qml";

export function title(words = TITLE): HTMLElement {
  const box = document.createElement("span");
  box.className = "shell-title";
  box.append(clamped(words, "body_strong", "on_surface_variant"));
  return box;
}

// The whole bar.

export interface BarOptions {
  readonly workspaces?: WorkspacesOptions;
  readonly preview?: boolean;
  readonly title?: string;
  readonly clock?: ClockOptions;
  /** The right group, left to right; the everyday one when left out. */
  readonly right?: readonly Element[];
  readonly state?: Shown;
}

/** The right group as the shots' stand-ins fill it: everything on. */
export function everyRight(state: Shown = "rest"): Element[] {
  return [recordingPill({ state }), idlePill({ state }), proxyPill(false, state), keyboardPill("US", state), updatePill(undefined, state), trayPill(), sysmonPill({ state }), bellPill(false, state), statusPill({ state }), powerButton(state)];
}

/**
 * The right group on an ordinary day: the tray, the system monitor with its default readings (CPU
 * and RAM), the bell, status, power.
 */
export function everydayRight(state: Shown = "rest"): Element[] {
  return [trayPill(), sysmonPill({ readings: READINGS.slice(0, 2), state }), bellPill(false, state), statusPill({ state }), powerButton(state)];
}

/** The bar's row: the left group, the clock centred, the right group. */
export function barRow(options: BarOptions = {}): HTMLElement {
  const state = options.state ?? "rest";
  const left = document.createElement("span");
  left.className = "shell-bar-left";
  left.append(workspaces({ groups: SHOTS_GROUPS, ...options.workspaces }));
  if (options.preview) left.append(previewPill(BRANCH, state));
  if (options.title !== "") left.append(title(options.title));
  const right = document.createElement("span");
  right.className = "shell-bar-right";
  right.append(...(options.right ?? everydayRight(state)));
  const bar = document.createElement("span");
  bar.className = "shell-bar-row";
  bar.append(left, clockPill({ ...options.clock, state }), right);
  return bar;
}

/** The bar on a screen `width` by `height`, its frame round the wallpaper. */
export function barScreen(width: string, height: string, options: BarOptions = {}): Screen {
  return screen({ width, height, wallpaper, bar: [barRow(options)] });
}

/** The bar alone, `width` wide, on the frame's glass. */
export function barStrip(width: string, options: BarOptions = {}): HTMLElement {
  const strip = document.createElement("span");
  strip.className = "shell-bar-strip";
  strip.style.setProperty("--strip-width", width);
  strip.append(barRow(options));
  return strip;
}
