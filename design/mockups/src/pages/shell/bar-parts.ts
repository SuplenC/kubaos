// The bar's parts for the shell's pages: every item with its content, the tray, the window title,
// and the whole bar along a screen's top. The items lie flat on the glass in a `size.bar_row` row,
// in groups parted by a hairline (`divider`): at the start the Command menu button, the workspaces,
// what you're doing and the title; the clock in the middle; at the end what opens a card. The data
// is the shots' stand-ins' (`test/shots/fakes`): Tuesday 27 October, 21:47, recording for 1:23.

import "./bar.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { badgedIcon } from "../../lib/badge.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { icon } from "../../lib/icon.ts";
import { meter } from "../../lib/meter.ts";
import { sparkline, minute } from "../../lib/meter.ts";
import { makePill, type Pill, type PillOptions, paired, pill } from "../../lib/pill.ts";
import { edgeRail, glass } from "../../lib/glass.ts";
import { type BarLayout, type Screen, screen } from "../../lib/sheet.ts";
import { declare } from "../../lib/probe.ts";
import { advance, clamped, text } from "../../lib/text.ts";
import { length } from "../../lib/tokens.ts";
import { APP_ICONS } from "../../site/app-icons.ts";
import { settings } from "../../site/settings.ts";
import { capsuleExtents, clockPill as libraryClockPill, type ClockPillWords } from "../../lib/clock-pill.ts";
import { mutePill, nowPlaying, type Song } from "./bar-audio-parts.ts";
import { commandButton, type CommandState } from "./bar-command.ts";
import { degrees, HOME, NOW_HOUR, symbolOf } from "./weather-data.ts";
import { countersPill } from "./counters-parts.ts";
import { workspaces, type WorkspacesOptions, SHOTS_GROUPS } from "./workspaces.ts";

export { wallpaper };

/** A symbol at the bar's 16 in a colour role. */
export function glyph(name: string, color = "on_surface_variant"): SVGElement {
  return icon(name, "var(--icon-s16)", `var(--color-${color})`);
}

type Interactive = Omit<PillOptions, "content" | "lone" | "interactive" | "tooltip" | "pinned">;

/** An interactive pill in `state`, saying `tooltip` while the pointer is on it. */
function control(content: Element[], tooltip: string, state: Shown, options: Interactive & { lone?: boolean } = {}): HTMLElement {
  return pill({ lone: false, ...options, content, interactive: true, tooltip, ...stateOptions(state) });
}

/** The hairline between two of the bar's groups: `size.bar_divider` tall, `on_surface` at `opacity.bar_divider`. */
export function divider(): HTMLElement {
  const line = document.createElement("span");
  line.className = "shell-bar-divider";
  return line;
}

/** A group of the bar's items, `space.s8` apart. */
export function group(...items: Element[]): HTMLElement {
  const made = document.createElement("span");
  made.className = "shell-bar-group";
  made.append(...items);
  return made;
}

/** `groups` in a row, a hairline between each two that hold anything. */
export function grouped(groups: readonly (Element | null)[]): Element[] {
  const held = groups.filter((each): each is Element => each !== null && (!each.classList.contains("shell-bar-group") || each.childElementCount > 0));
  return held.flatMap((each, index) => (index === 0 ? [each] : [divider(), each]));
}

/**
 * Items on a short strip of glass as wide as they are, the Floating layout's look: a sample of the
 * bar's items with their edge light, which needs the glass's lower edge to lie on.
 */
export function onGlass(...items: Element[]): HTMLElement {
  const strip = glass(document.createElement("span"));
  strip.className += " shell-bar-piece";
  strip.append(...items);
  edgeRail(strip);
  return strip;
}

// The clock.

/** The clock pill's words at the shots' moment: Tuesday 27 October, 21:47, 12° in Warsaw. */
export const CLOCK: ClockPillWords = {
  kanji: "火",
  date: "Tue 27 Oct",
  day: "Tue 27",
  time: "21:47",
  whole: "Tuesday, 27 October 2026",
  weather: { symbol: symbolOf(HOME.weather, NOW_HOUR), temperature: degrees(HOME.temperature) },
};

/** What the clock pill shows, as far as a narrow bar has shed it. */
export interface ClockOptions {
  readonly seal?: boolean;
  readonly date?: "full" | "day" | "none";
  /** Home's weather: none with no home city, or once a narrow bar sheds it. */
  readonly weather?: boolean;
  readonly recording?: boolean;
  readonly state?: Shown;
  /** Its card open on the calendar or the weather: its capsule filled. */
  readonly open?: "clock" | "weather";
}

/** The bar's clock pill (`lib/clock-pill.ts`) with the shots' words. */
export function clockPill(options: ClockOptions = {}): HTMLElement {
  return libraryClockPill({ words: CLOCK, ...options }).element;
}

// The status pill.

export interface StatusOptions {
  readonly power?: string;
  readonly net?: string;
  readonly bluetooth?: string | null;
  /** The battery, or none on a desktop: its share, low or charging. */
  readonly battery?: { readonly percent: number; readonly low?: boolean; readonly charging?: boolean } | null;
  readonly state?: Shown;
}

export function statusContent(options: StatusOptions = {}): Element[] {
  const parts: Element[] = [glyph(options.power ?? "balance"), glyph(options.net ?? "signal_wifi_4_bar")];
  const bluetooth = options.bluetooth === undefined ? "bluetooth" : options.bluetooth;
  if (bluetooth) parts.push(glyph(bluetooth));
  const battery = options.battery === undefined ? { percent: 64 } : options.battery;
  if (battery) {
    const tone = battery.low ? "error" : battery.charging ? "primary" : "on_surface_variant";
    const name = battery.charging ? "battery_charging_full" : `battery_${Math.min(3, Math.floor(battery.percent / 25))}_bar`;
    parts.push(paired(glyph(name, tone), text(`${battery.percent}%`, "mono", "on_surface")));
  }
  return parts;
}

export function statusPill(options: StatusOptions = {}): HTMLElement {
  return control(statusContent(options), "Home 5G  ·  Battery 64%", options.state ?? "rest", { opens: true });
}

// The bell.

/**
 * The bell, crossed out under Do not disturb, the accent's unread dot on its symbol's corner while
 * anything is unread: the popups carry no dot, since a popup is new by being one.
 */
export function bellPill(dnd = false, state: Shown = "rest", unread = false): HTMLElement {
  const name = dnd ? "notifications_off" : "notifications";
  return control([unread ? badgedIcon({ name, accent: true, badged: true }) : glyph(name)], "", state, { lone: true, opens: true });
}

/** The bell with its hint: only with nothing unread. */
export function quietBell(state: Shown = "rest"): HTMLElement {
  return control([glyph("notifications")], "No new notifications", state, { lone: true, opens: true });
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
  if (options.scheduled) parts.push(paired(glyph(options.scheduled.icon, "tertiary"), text(options.scheduled.left, "mono", "on_surface")));
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
    { lone: true, opens: true },
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

/** The live preview: its symbol alone in `cold`, flat, the branch in its hint. */
export function previewPill(branch = BRANCH, state: Shown = "rest"): HTMLElement {
  return control([icon("code", "var(--icon-s16)", "var(--color-cold)")], `Live preview of feature/${branch}. Click to go back to the release`, state, { tone: "preview", lone: true });
}

// On air: the screen shared and the camera in use, each a pill of its own.

/** What a share takes: a whole screen, one window, or a region drawn on a screen. */
export type ShareKind = "screen" | "window" | "region";

/** Each kind's symbol, the same in the picker's tabs and on the pill. */
export const SHARE_ICONS: Record<ShareKind, string> = { screen: "desktop_windows", window: "app_window", region: "screenshot_region" };

/** A share running: what it takes, named as the picker names it, the app it goes to, and since when. */
export interface Share {
  readonly kind: ShareKind;
  readonly what: string;
  readonly who: string;
  /** The site in the app, where the app names one. */
  readonly site?: string;
  readonly since?: string;
}

/** A camera in use: the app that has it, the camera, and since when. */
export interface CameraUse {
  readonly who: string;
  readonly device?: string;
  readonly since?: string;
}

/** What the on-air pills' hints say, one message each, the names as placeholders. */
export const ON_AIR_WORDS = {
  sharing: (what: string, who: string) => `Sharing ${what} · ${who}`,
  camera: (who: string) => `Camera · ${who}`,
} as const;

// The locale's list joins the hint's messages: "Sharing the left screen · Chromium, Sharing … · Discord".
const LISTED = new Intl.ListFormat("en-GB", { style: "narrow", type: "unit" });
// The locale's number says the count.
const COUNT = new Intl.NumberFormat("en-GB");

/** A list of one or more: one share or camera, or several. */
type OneOrMore<T> = T | readonly T[];
const all = <T>(each: OneOrMore<T>): readonly T[] => (Array.isArray(each) ? (each as readonly T[]) : [each as T]);

/** Which on-air pill: the screen shared, or the camera in use. */
export type OnAirTone = "sharing" | "camera";

const ON_AIR_INK: Record<OnAirTone, string> = { sharing: "on_sharing_container", camera: "on_error_container" };

/**
 * An on-air pill's content: its symbol, then past one the count, `space.s4` after it as a value
 * after its symbol. The symbol is the same element either way, so as the count comes and goes the
 * Pill keeps the symbol solid and only the count fades, in its own room.
 */
export function onAirContent(tone: OnAirTone, symbol: string, count: number): Element[] {
  const ink = ON_AIR_INK[tone];
  const glyph = icon(symbol, "var(--icon-s16)", `var(--color-${ink})`);
  return count > 1 ? [glyph, text(COUNT.format(count), "label_number", ink)] : [glyph];
}

/**
 * An on-air pill's options: on its meaning's container, `space.s8` at each end whether or not a
 * count is in it, so the count arriving never moves the pill's ends; the gap `space.s4`.
 */
export function onAirOptions(tone: OnAirTone): Omit<PillOptions, "content"> {
  return { tone, lone: false, end: "s8", gap: "s4" };
}

function onAirPill(tone: OnAirTone, content: Element[], tip: string, state: Shown): HTMLElement {
  const made = pill({ ...onAirOptions(tone), content, interactive: true, opens: true, tooltip: tip, ...stateOptions(state) });
  made.classList.add("shell-on-air");
  made.dataset.onAir = tone;
  made.setAttribute("aria-haspopup", "dialog");
  return made;
}

/** The sharing pill's content: the kind's symbol, the screen's where shares of several kinds run, and the count past one. */
export function sharingContent(shares: OneOrMore<Share>): Element[] {
  const list = all(shares);
  const kinds = new Set(list.map((each) => each.kind));
  return onAirContent("sharing", SHARE_ICONS[kinds.size === 1 ? list[0]!.kind : "screen"], list.length);
}

/** The camera pill's content: the camera's symbol, and the count past one. */
export function cameraContent(cameras: OneOrMore<CameraUse>): Element[] {
  return onAirContent("camera", "video", all(cameras).length);
}

/**
 * The sharing pill: on `sharing_container`, the kind's symbol in `on_sharing_container`, and the
 * count past one. Its hint says what and with whom, for each; a press opens its panel.
 */
export function sharingPill(shares: OneOrMore<Share>, state: Shown = "rest"): HTMLElement {
  const tip = LISTED.format(all(shares).map((each) => ON_AIR_WORDS.sharing(each.what, each.who)));
  return onAirPill("sharing", sharingContent(shares), tip, state);
}

/**
 * The camera pill: the camera's symbol on `error_container` in `on_error_container`, red as the
 * screen recording's pill is, and the count past one. Its hint names the app, for each camera.
 */
export function cameraPill(cameras: OneOrMore<CameraUse>, state: Shown = "rest"): HTMLElement {
  return onAirPill("camera", cameraContent(cameras), LISTED.format(all(cameras).map((each) => ON_AIR_WORDS.camera(each.who))), state);
}

/** The on-air pills in their order: the shares, then the cameras. */
export function onAirPills(on: { readonly sharing?: OneOrMore<Share>; readonly camera?: OneOrMore<CameraUse> }, state: Shown = "rest"): HTMLElement[] {
  return [...(on.sharing ? [sharingPill(on.sharing, state)] : []), ...(on.camera ? [cameraPill(on.camera, state)] : [])];
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
  return [paired(dot, text(options.replay ? "Replay 30 s" : (options.elapsed ?? "1:23"), "mono", "on_error_container"))];
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

/** A reading's words, in a box as wide as "100%" at least, the room it doesn't use after them. */
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
  if (options.offering) return [paired(glyph("visibility_off"), text("Hide", "label", "on_surface"))];
  const style = options.style ?? "numbers";
  const parts: Element[] = (options.readings ?? READINGS).map((reading, index) => {
    const level = reading.level ?? 0;
    const ink = level > 0 ? tintOf(level) : "primary";
    let value: Element;
    if (style === "bars" && !reading.unscaled) {
      const bar = meter([{ share: reading.fill, color: ink }]);
      bar.dataset.compact = "";
      value = bar;
    } else if (style === "sparklines") {
      value = sparkline({ samples: minute(600_000, readingShape(index)), color: ink, inline: true }).box;
    } else {
      value = readingWords(reading);
    }
    return paired(glyph(reading.icon, tintOf(level)), value);
  });
  if (options.fullDisk) parts.push(glyph("hard_drive", "error"));
  return parts;
}

export function sysmonPill(options: SysmonOptions = {}): HTMLElement {
  const tip = options.offering ? "" : "CPU 37%  ·  61 °C  ·  RAM 11.2 GB of 15.6 GB";
  return control(sysmonContent(options), tip, options.state ?? "rest", { gap: "s8", opens: true });
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

/** One app's press target, a chip square, the picture at the bar's 16 in its middle. */
export function trayItem(app: TrayApp, pinned?: "hover" | "pressed"): HTMLElement {
  const item = document.createElement("span");
  item.className = "shell-tray-item";
  item.dataset.hoverable = "";
  // Each app opens its own menu: the edge light lies under it alone.
  item.dataset.opens = "";
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

export function trayPill(apps: readonly TrayApp[] = TRAY, pinned?: { index: number; state: "hover" | "pressed" }): HTMLElement {
  return pill({ lone: false, end: "s4", gap: 0, content: apps.map((app, index) => trayItem(app, pinned?.index === index ? pinned.state : undefined)) });
}

// The window title.

export const TITLE = "kubaos — nvim config/quickshell/modules/bar/Workspaces.qml";

export function title(words = TITLE): HTMLElement {
  const box = document.createElement("span");
  box.className = "shell-title";
  box.append(declare(clamped(words, "body_strong", "on_surface_variant"), { variable: true }));
  return box;
}

// The ⋯: the tray's apps, and what folds in with them, behind one symbol.

export function morePill(state: Shown = "rest"): HTMLElement {
  return control([glyph("more")], "Tray apps, the update and the preview", state, { lone: true, opens: true });
}

// The system monitor, compact: its readings as rings in one circle, the rim CPU.

/** A compact reading: its share 0–1 and its step against its limit. */
export interface Load {
  readonly share: number;
  readonly level?: 0 | 1 | 2;
}

export const LOADS: readonly Load[] = READINGS.map((reading) => ({ share: reading.fill }));

/**
 * CPU, RAM and temperature as rings edge to edge in the pill's circle, CPU the rim, RAM inside it,
 * temperature innermost: each ring `space.s4` less a device hairline thick, their middles
 * `space.s4` apart, so a hairline parts them. One palette for every ring, its order saying which is
 * which: `primary` at ease, `warning` near its limit, `error` past it, over its track at
 * `opacity.spinner_track`.
 */
export function rimPill(loads: readonly Load[] = LOADS, state: Shown = "rest"): HTMLElement {
  const scale = settings().scale;
  const side = length("size-control_sm");
  const step = length("space-s4");
  const stroke = step - 1 / scale;
  const middle = side / 2;
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", `0 0 ${side} ${side}`);
  svg.classList.add("shell-rim");
  svg.innerHTML = loads
    .map((load, index) => {
      const tone = load.level === 2 ? "error" : load.level === 1 ? "warning" : "primary";
      const radius = middle - stroke / 2 - step * index;
      const ring = `cx="${middle}" cy="${middle}" r="${radius}" fill="none" stroke-width="${stroke}"`;
      return `<circle ${ring} class="shell-rim-track" style="--tone: var(--color-${tone})"/>` +
        (load.share > 0 ? `<circle ${ring} class="shell-rim-arc" style="--tone: var(--color-${tone})" pathLength="100" stroke-dasharray="${(load.share * 100).toFixed(2)} 100" transform="rotate(-90 ${middle} ${middle})"/>` : "");
    })
    .join("");
  const made = control([svg], `CPU ${Math.round(loads[0]!.share * 100)}%  ·  RAM ${Math.round((loads[1]?.share ?? 0) * 100)}%  ·  Temperature 61 °C`, state, { lone: true, opens: true });
  made.classList.add("shell-rim-pill");
  return made;
}

// The status pill as it sheds.

export interface ShedOptions {
  /** How far it has shed: 0 all, 1 the battery's share gone, then a symbol from the left each, 4 the battery alone. */
  readonly shed?: number;
  readonly state?: Shown;
}

export function shedStatus(options: ShedOptions = {}): HTMLElement {
  const shed = options.shed ?? 0;
  const parts: Element[] = [];
  parts.push(...["balance", "signal_wifi_4_bar", "bluetooth"].slice(Math.max(0, shed - 1)).map((name) => glyph(name)));
  parts.push(shed === 0 ? paired(glyph("battery_2_bar"), text("64%", "mono", "on_surface")) : glyph("battery_2_bar"));
  return control(parts, "Home 5G  ·  Battery 64%", options.state ?? "rest", { lone: parts.length === 1, opens: true });
}

// The whole bar: what it holds, and the order it gives way in as it narrows.

/** What the bar holds at full width. */
export interface BarContent {
  readonly recording: boolean;
  readonly keepAwake: boolean;
  readonly proxy: boolean;
  readonly layout: boolean;
  readonly update: boolean;
  readonly preview: boolean;
  readonly tray: boolean;
  readonly title: string;
  readonly readings: 2 | 3;
  readonly borrowed: boolean;
  /** A player plays or is paused: now playing shows. */
  readonly playing: boolean;
  /** What plays, while something does. */
  readonly song?: Song;
  /** Shares run: the sharing pill shows, first on the right, never given way. */
  readonly sharing?: Share | readonly Share[];
  /** Apps have a camera: the camera pill shows after the sharing pill, never given way. */
  readonly camera?: CameraUse | readonly CameraUse[];
  /** No home city set: the clock shows no weather. */
  readonly noHome?: boolean;
  /** The counters are on: their pill shows at the start of the end, with the tray. */
  readonly counters?: boolean;
}

/** Every pill at once, as the shots' stand-ins hold them. */
export const EVERYTHING: BarContent = { recording: true, keepAwake: true, proxy: true, layout: true, update: true, preview: true, tray: true, title: TITLE, readings: 3, borrowed: true, playing: true, counters: true };
/** An ordinary day: the tray, the system monitor's two default readings, the bell, status. */
export const EVERYDAY: BarContent = { recording: false, keepAwake: false, proxy: false, layout: false, update: false, preview: false, tray: true, title: TITLE, readings: 2, borrowed: false, playing: false };

/** How far each part has given way. */
interface BarState {
  title: boolean;
  /** The quiet pills (the tray, the update, the preview, Keep awake, Proxy, the layout, the counters) folded into ⋯. */
  quiet: boolean;
  weather: boolean;
  date: "full" | "day" | "none";
  seal: boolean;
  recording: "time" | "clock";
  name: boolean;
  sysmon: "numbers" | "rings";
  playing: "wide" | "folded" | "none";
  mute: "both" | "mic";
  shed: number;
}

const FULL: BarState = { title: true, quiet: false, weather: true, date: "full", seal: true, recording: "time", name: true, sysmon: "numbers", playing: "wide", mute: "both", shed: 0 };

/** Where a rung's part sits: the start, the end, the clock in the middle, or both sides. */
type Side = "left" | "right" | "clock" | "both";

/** One step of giving way: the side it frees, what it says, and what it changes. */
export interface Rung {
  readonly side: Side;
  readonly says: string;
  /** What it says when a narrower step takes it back: now playing opens again once there is room. */
  readonly back?: string;
  readonly apply: (state: BarState) => void;
}

/**
 * The order the bar gives way in: first the quiet pills fold into ⋯, where they are one click
 * away, so the words (the title, the date, now playing) and every control that shows state stay
 * as long as they can. Then a borrowed group's name, now playing's words, the title, the system
 * monitor's numbers, the weather and the date; the clock, the mute pill, status and the bell
 * always there, and the recording's dot never gone. A rung is taken only while its side runs short
 * of the clock's clear space (a clock rung, or one on both sides, while either side does). The
 * title is not a rung of its own: it shrinks with its room and goes once that is too little.
 */
export const RUNGS: readonly Rung[] = [
  { side: "both", says: "Quiet pills fold into ⋯", apply: (s) => void (s.quiet = true) },
  { side: "left", says: "Group name goes", apply: (s) => void (s.name = false) },
  { side: "left", says: "Now playing folds to its controls", back: "Now playing wide again", apply: (s) => void (s.playing = "folded") },
  { side: "left", says: "Title gone", apply: (s) => void (s.title = false) },
  { side: "right", says: "System monitor turns to rings", apply: (s) => void (s.sysmon = "rings") },
  { side: "clock", says: "Weather goes", apply: (s) => void (s.weather = false) },
  { side: "clock", says: "Date drops the month", apply: (s) => void (s.date = "day") },
  { side: "clock", says: "Date goes", apply: (s) => void (s.date = "none") },
  { side: "left", says: "Recording joins the clock", apply: (s) => void (s.recording = "clock") },
  { side: "left", says: "Now playing goes", apply: (s) => void (s.playing = "none") },
  { side: "right", says: "Battery % goes", apply: (s) => void (s.shed = 1) },
  { side: "right", says: "Power mode goes", apply: (s) => void (s.shed = 2) },
  { side: "right", says: "Network goes", apply: (s) => void (s.shed = 3) },
  { side: "right", says: "Bluetooth goes", apply: (s) => void (s.shed = 4) },
  { side: "right", says: "Mute pill keeps the mic", apply: (s) => void (s.mute = "mic") },
  { side: "clock", says: "Seal goes", apply: (s) => void (s.seal = false) },
];

/** Now playing's words folding, and the title going: `fitBar` weighs these two against each other. */
const FOLD = RUNGS.findIndex((rung) => rung.says.startsWith("Now playing folds"));
const TITLE_GONE = RUNGS.findIndex((rung) => rung.says === "Title gone");
/** Now playing gone: no words to give back. */
const GONE = RUNGS.findIndex((rung) => rung.says === "Now playing goes");

function stateOf(taken: ReadonlySet<number>): BarState {
  const state = { ...FULL };
  RUNGS.forEach((rung, index) => taken.has(index) && rung.apply(state));
  return state;
}

/** The workspaces as the state shows them: a borrowed group's name while it lasts. */
function workspacesFor(content: BarContent, state: BarState, options: WorkspacesOptions = {}): HTMLElement {
  const own = SHOTS_GROUPS[0]!;
  const desk = SHOTS_GROUPS[1]!;
  const borrowed = { ...desk, ...(state.name ? {} : { name: "" }) };
  return workspaces({ groups: content.borrowed ? [own, borrowed] : [own], ...options });
}

/** How a bar is drawn beyond its content: the pointer's state, the workspaces', the Command menu button's. */
interface DrawOptions {
  readonly shown?: Shown;
  readonly workspaces?: WorkspacesOptions;
  readonly command?: CommandState;
}

interface Drawn {
  readonly row: HTMLElement;
  readonly left: HTMLElement;
  readonly clock: HTMLElement;
  readonly right: HTMLElement;
  readonly title: HTMLElement | null;
}

/**
 * The bar's row in `state`: the start (the Command menu button, the workspaces, what you're doing,
 * the title, now playing), the clock centred, the end (what is on air and the tray, the system
 * monitor, the mute pill, the bell, status), each group parted from the next by a hairline.
 */
function draw(content: BarContent, state: BarState, options: DrawOptions = {}): Drawn {
  const shown = options.shown ?? "rest";
  const left = document.createElement("span");
  left.className = "shell-bar-left";
  const command = commandButton({ state: options.command ?? "rest", live: true });
  command.dataset.ownIsland = "";
  const doing = group();
  if (content.preview && !state.quiet) doing.append(previewPill(BRANCH, shown));
  if (content.recording && state.recording === "time") doing.append(recordingPill({ state: shown }));
  if (content.keepAwake && !state.quiet) doing.append(idlePill({ state: shown }));
  if (content.proxy && !state.quiet) doing.append(proxyPill(false, shown));
  if (content.layout && !state.quiet) doing.append(keyboardPill("US", shown));
  if (content.update && !state.quiet) doing.append(updatePill(undefined, shown));
  const titled = content.title !== "" && state.title ? title(content.title) : null;
  // The audio pills' targets press in a pill's place: hovered or pressed, the first's.
  const circle = shown === "hovered" ? "hover" : shown === "pressed" ? shown : undefined;
  const playing = content.playing && state.playing !== "none" ? nowPlaying({ folded: state.playing === "folded", ...(content.song ? { song: content.song } : {}), ...(circle ? { pinned: { part: "play", state: circle } } : {}) }) : null;
  left.append(...grouped([command, workspacesFor(content, state, options.workspaces), doing, titled]), ...(playing ? [playing] : []));
  const clock = clockPill({ seal: state.seal, date: state.date, weather: !content.noHome && state.weather, recording: content.recording && state.recording === "clock", state: shown });
  const right = document.createElement("span");
  right.className = "shell-bar-right";
  const quiet = state.quiet && (content.tray || content.update || content.preview || content.keepAwake || content.proxy || content.layout || (content.counters ?? false));
  // First on the end, nearest the clock: the side is laid from its end, so a pill arriving here
  // moves no other pill, and it lands in the room the eye already crosses to read the time.
  const apps = group(...onAirPills(content, shown));
  if (content.counters && !state.quiet) apps.append(countersPill(shown));
  apps.append(content.tray && !quiet ? trayPill() : morePill(shown));
  const sysmon = state.sysmon === "rings" ? rimPill(LOADS.slice(0, 3), shown) : sysmonPill({ readings: READINGS.slice(0, content.readings), state: shown });
  const mute = mutePill({ ...(state.mute === "mic" ? { micOnly: true } : {}), ...(circle ? { pinned: { side: state.mute === "mic" ? "input" : "output", state: circle } } : {}) });
  right.append(...grouped([apps, sysmon, mute, bellPill(false, shown), shedStatus({ shed: state.shed, state: shown })]));
  const row = document.createElement("span");
  row.className = "shell-bar-row";
  row.append(left, clock, right);
  return { row, left, clock, right, title: titled };
}

/** A rung's state's fit on a bar `width` logical pixels wide in `layout`, measured where the page draws. */
interface Measured {
  /** Now playing ends the left, `space.s8` from the clock rather than its clear space. */
  readonly playing: boolean;
  readonly leftFits: boolean;
  readonly rightFits: boolean;
  readonly titleRoom: number;
}

/**
 * The room the bar's row has inside its glass, from its middle to each end, in logical pixels: the
 * screen less the bar's edges in `layout`. In Islands each side also gives up the ends of the
 * clock's island and its own, and the gap between them, and the start the Command menu button's
 * island and the gap after it.
 */
function halves(width: number, layout: BarLayout): { left: number; right: number } {
  const gap = Math.round(length("size-gap"));
  const pad = length("space-s4");
  const half = layout === "bar" ? width / 2 - gap : width / 2 - gap - pad;
  if (layout !== "islands") return { left: half, right: half };
  const parted = 2 * pad + gap;
  return { left: half - parted - (length("size-bar_strip") - pad + gap), right: half - parted };
}

function measure(width: number, content: BarContent, state: BarState, layout: BarLayout): Measured {
  const host = document.createElement("span");
  host.style.cssText = "position:absolute;visibility:hidden;left:0;top:0";
  const strip = document.createElement("span");
  strip.className = "shell-bar-strip";
  strip.style.setProperty("--strip-width", `calc(${width} * var(--px))`);
  const drawn = draw(content, { ...state, title: false });
  strip.append(drawn.row);
  host.append(strip);
  (document.getElementById("page") ?? document.body).append(host);
  const scale = Number(getComputedStyle(host).getPropertyValue("--scale")) || 1;
  // An item's width and the margins round it: a hairline's are the room either side of it. Now
  // playing's auto margin is the room it is pushed across, not room it needs.
  const outer = (element: Element) => {
    const style = getComputedStyle(element);
    const margins = element.classList.contains("shell-playing-pill") ? 0 : Number.parseFloat(style.marginLeft) + Number.parseFloat(style.marginRight);
    return (element.getBoundingClientRect().width + margins) / scale;
  };
  // Each side's own items: its grid column shrinks with the bar, its items don't.
  const need = (side: HTMLElement) => [...side.children].reduce((sum, each) => sum + outer(each), 0);
  const leftNeed = need(drawn.left);
  const rightNeed = need(drawn.right);
  // The capsule is the screen's middle: each side has the room from there to its end of the pill.
  const capsule = capsuleExtents(drawn.clock);
  const before = capsule.before / scale;
  const after = capsule.after / scale;
  const playing = drawn.left.querySelector(".shell-playing-pill") !== null;
  host.remove();
  const gap = length("space-s8");
  const clear = length("space-s40");
  const leftClear = playing ? gap : clear;
  const room = halves(width, layout);
  const leftHalf = room.left - before;
  const rightHalf = room.right - after;
  return { playing, leftFits: leftNeed + leftClear <= leftHalf, rightFits: rightNeed + clear <= rightHalf, titleRoom: leftHalf - leftClear - leftNeed - length("space-s12") };
}

/**
 * Whether now playing's words give way for the title: the left runs short of the clock's clear
 * space, or the title would have less room than its own width or the song's room, whichever is
 * less, or than the least it shows in. The title is never cut, or gone, to make room for the song.
 */
function folds(content: BarContent, got: Measured): boolean {
  if (!content.playing) return false;
  if (!got.leftFits) return true;
  const wants = Math.max(Math.min(advance("body_strong", content.title), length("size-now_playing")), TITLE_LEAST());
  return content.title !== "" && got.titleRoom < wants;
}

/** The least room the title shows in: under it, it goes. */
const TITLE_LEAST = () => 2 * length("space-s48");

/**
 * Gives now playing its words back where they no longer cost the title anything: the title gone,
 * or still as roomy as `folds` asks with the pill wide, and the left still clear of the clock.
 */
function unfold(width: number, content: BarContent, taken: Set<number>, layout: BarLayout): void {
  if (taken.has(GONE)) return;
  const wide = new Set([...taken].filter((index) => index !== FOLD));
  const got = measure(width, content, stateOf(wide), layout);
  if (!got.leftFits) return;
  if (!wide.has(TITLE_GONE) && folds(content, got)) return;
  taken.delete(FOLD);
}

/**
 * The rungs the bar takes on a screen `width` logical pixels wide in `layout`, measured against
 * the row's real room inside its glass, to clear its centred clock, and its state.
 */
export function fitBar(width: number, content: BarContent, layout: BarLayout = "bar"): { readonly taken: ReadonlySet<number>; readonly state: BarState; readonly fits: boolean } {
  const taken = new Set<number>();
  let got = measure(width, content, stateOf(taken), layout);
  const fits = () => got.leftFits && got.rightFits;
  for (let index = 0; index < RUNGS.length; index++) {
    if (index === TITLE_GONE) continue;
    const side = RUNGS[index]!.side;
    const short = side === "left" ? !got.leftFits : side === "right" ? !got.rightFits : !fits();
    if (!short && !(index === FOLD && folds(content, got))) continue;
    taken.add(index);
    got = measure(width, content, stateOf(taken), layout);
  }
  if (got.titleRoom < TITLE_LEAST()) taken.add(TITLE_GONE);
  if (taken.has(FOLD)) unfold(width, content, taken, layout);
  return { taken, state: stateOf(taken), fits: fits() };
}

export interface BarOptions {
  /** The bar's width in logical pixels. */
  readonly width: number;
  readonly content?: BarContent;
  readonly workspaces?: WorkspacesOptions;
  readonly state?: Shown;
  readonly command?: CommandState;
}

/**
 * The bar alone on a screen `width` logical pixels wide, as the Floating layout's strip on the
 * glass, given way as far as it needs, its items answering the pointer with the edge light.
 */
export function barStrip(options: BarOptions): HTMLElement {
  const content = options.content ?? EVERYDAY;
  const { state } = fitBar(options.width, content, "floating");
  const strip = glass(document.createElement("span"));
  strip.classList.add("shell-bar-strip");
  strip.style.setProperty("--strip-width", `calc(${options.width} * var(--px))`);
  strip.append(draw(content, state, drawOptions(options)).row);
  edgeRail(strip);
  return strip;
}

function drawOptions(options: Omit<BarOptions, "width">): DrawOptions {
  return { ...(options.state ? { shown: options.state } : {}), ...(options.workspaces ? { workspaces: options.workspaces } : {}), ...(options.command ? { command: options.command } : {}) };
}

/** How the bar lies on a screen: one of the layouts, the setting's default unless given. */
export interface ScreenBarOptions extends Omit<BarOptions, "width"> {
  readonly layout?: BarLayout;
}

/** The bar on a screen `width` logical pixels by `height` (a length's CSS), over the wallpaper. */
export function barScreen(width: number, height: string, options: ScreenBarOptions = {}): Screen {
  const content = options.content ?? EVERYDAY;
  const { state } = fitBar(width, content, options.layout ?? "bar");
  const row = draw(content, state, drawOptions(options)).row;
  return screen({ width: `calc(${width} * var(--px))`, height, wallpaper, bar: [row], ...(options.layout ? { layout: options.layout } : {}) });
}

/** A window size's width in logical pixels. */
export function logicalWidth(css: string): number {
  const calc = /calc\((\d+) \* var\(--px\)\)/.exec(css);
  if (calc) return Number(calc[1]);
  const token = /var\(--([a-z0-9_-]+)\)/.exec(css);
  return token ? length(token[1]!) : 0;
}
