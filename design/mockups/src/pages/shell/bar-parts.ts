// The bar's parts as `config/quickshell/modules/bar/` draws them, for the shell's pages: every pill
// with its content, the tray, the window title, and the whole bar along a screen's top. The data is
// the shots' stand-ins' (`test/shots/fakes`): Tuesday 27 October, 21:47, recording for 1:23.

import "./bar.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { badgedIcon } from "../../lib/badge.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { icon } from "../../lib/icon.ts";
import { meter } from "../../lib/meter.ts";
import { sparkline, minute } from "../../lib/meter.ts";
import { makePill, type Pill, type PillOptions, paired, pill } from "../../lib/pill.ts";
import { type Screen, screen } from "../../lib/sheet.ts";
import { widestTime } from "../../lib/lock.ts";
import { advance, clamped, text } from "../../lib/text.ts";
import { length } from "../../lib/tokens.ts";
import { APP_ICONS } from "../../site/app-icons.ts";
import { settings } from "../../site/settings.ts";
import { mutePill, nowPlaying } from "./bar-audio-parts.ts";
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

/**
 * A pinned countdown as the clock says it, `seconds` left: to the minute as the clock beside it
 * ("52 d 2 h", "3 h 12 min", "12 min"), and under a minute its seconds ("0:45"), never its name alone.
 */
export function countdownWords(name: string, seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${name} ${days} d ${hours} h`;
  if (hours > 0) return `${name} ${hours} h ${minutes} min`;
  if (minutes > 0) return `${name} ${minutes} min`;
  return `${name} 0:${String(seconds % 60).padStart(2, "0")}`;
}

export interface ClockOptions {
  readonly countdown?: string;
  readonly date?: string;
  readonly time?: string;
  readonly state?: Shown;
}

/**
 * The time in `body`, Geist's own proportional figures, centred in a slot as wide as the widest
 * time a 24-hour clock shows, as the lock's bar clock is: the pill keeps its width as the minutes
 * turn, and the bar's steps never meet a wider time than they were laid out for.
 */
export function clockTime(time: string = CLOCK.time, color = "on_surface"): HTMLElement {
  const shown = text(time, "body", color);
  shown.classList.add("shell-clock-time");
  shown.style.setProperty("--slot", String(widestTime("body", false)));
  return shown;
}

/** The weekday's kanji in `accent`, the date in `body`, the time in its slot, a countdown after. */
export function clockPill(options: ClockOptions = {}): HTMLElement {
  return control(clockContent(options), "Tuesday, 27 October 2026", options.state ?? "rest");
}

export function clockContent(options: ClockOptions = {}): Element[] {
  const parts: Element[] = [text(CLOCK.kanji, "accent", "tertiary"), text(options.date ?? CLOCK.date, "body", "on_surface_variant"), clockTime(options.time)];
  if (options.countdown) parts.push(text(options.countdown, "body", "tertiary"));
  return parts;
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
  return control(statusContent(options), "Home 5G  ·  Battery 64%", options.state ?? "rest");
}

// The bell.

/**
 * The bell, crossed out under Do not disturb, the accent's unread dot on its symbol's corner while
 * anything is unread: the popups carry no dot, since a popup is new by being one.
 */
export function bellPill(dnd = false, state: Shown = "rest", unread = false): HTMLElement {
  const name = dnd ? "notifications_off" : "notifications";
  return control([unread ? badgedIcon({ name, accent: true, badged: true }) : glyph(name)], "", state, { lone: true });
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

/** The live preview: its symbol alone, on the cold tint, the branch in its hint. */
export function previewPill(branch = BRANCH, state: Shown = "rest"): HTMLElement {
  return control([icon("code", "var(--icon-s16)", "var(--color-cold)")], `Live preview of feature/${branch}. Click to go back to the release`, state, { tone: "preview", lone: true });
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
  return control(sysmonContent(options), tip, options.state ?? "rest", { gap: "s8" });
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

// The window title.

export const TITLE = "kubaos — nvim config/quickshell/modules/bar/Workspaces.qml";

export function title(words = TITLE): HTMLElement {
  const box = document.createElement("span");
  box.className = "shell-title";
  box.append(clamped(words, "body_strong", "on_surface_variant"));
  return box;
}

// The ⋯: the tray's apps, and what folds in with them, behind one symbol.

export function morePill(state: Shown = "rest"): HTMLElement {
  return control([glyph("more")], "Tray apps, the update and the preview", state, { lone: true });
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
  const made = control([svg], `CPU ${Math.round(loads[0]!.share * 100)}%  ·  RAM ${Math.round((loads[1]?.share ?? 0) * 100)}%  ·  Temperature 61 °C`, state, { lone: true });
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
  return control(parts, "Home 5G  ·  Battery 64%", options.state ?? "rest", { lone: parts.length === 1 });
}

// The clock as it compacts.

export interface CompactClock {
  readonly kanji?: boolean;
  readonly date?: "full" | "day" | "none";
  readonly countdown?: boolean;
  /** Recording merged into it: the recording pill's look, its dot leading. */
  readonly recording?: boolean;
  readonly state?: Shown;
}

export function compactClock(options: CompactClock = {}): HTMLElement {
  const recording = options.recording ?? false;
  const ink = (role: string) => (recording ? "on_error_container" : role);
  const parts: Element[] = [];
  if (recording) {
    const dot = document.createElement("span");
    dot.className = "shell-dot";
    parts.push(dot);
  }
  if (options.kanji ?? true) parts.push(text(CLOCK.kanji, "accent", ink("tertiary")));
  const date = options.date ?? "full";
  if (date !== "none") parts.push(text(date === "full" ? CLOCK.date : "Tue 27", "body", ink("on_surface_variant")));
  parts.push(clockTime(CLOCK.time, ink("on_surface")));
  if (options.countdown) parts.push(text(CLOCK.countdown, "body", ink("tertiary")));
  return control(parts, recording ? "Recording. Tuesday, 27 October 2026" : "Tuesday, 27 October 2026", options.state ?? "rest", recording ? { tone: "recording" } : {});
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
  readonly countdown: boolean;
  readonly title: string;
  readonly readings: 2 | 3;
  readonly borrowed: boolean;
  /** An app has a player: now playing shows. */
  readonly playing: boolean;
}

/** Every pill at once, as the shots' stand-ins hold them. */
export const EVERYTHING: BarContent = { recording: true, keepAwake: true, proxy: true, layout: true, update: true, preview: true, tray: true, countdown: true, title: TITLE, readings: 3, borrowed: true, playing: true };
/** An ordinary day: the tray, the system monitor's two default readings, the bell, status. */
export const EVERYDAY: BarContent = { recording: false, keepAwake: false, proxy: false, layout: false, update: false, preview: false, tray: true, countdown: false, title: TITLE, readings: 2, borrowed: false, playing: false };

/** How far each part has given way. */
interface BarState {
  title: boolean;
  layout: boolean;
  countdown: boolean;
  idle: boolean;
  date: "full" | "day" | "none";
  recording: "time" | "dot";
  tray: "open" | "folded";
  update: boolean;
  preview: boolean;
  name: boolean;
  ghosts: boolean;
  sysmon: "numbers" | "rings";
  playing: "full" | "play" | "none";
  mute: "both" | "mic";
  shed: number;
}

const FULL: BarState = { title: true, layout: true, countdown: true, idle: true, date: "full", recording: "time", tray: "open", update: true, preview: true, name: true, ghosts: true, sysmon: "numbers", playing: "full", mute: "both", shed: 0 };

/** Where a rung's part sits: the left, the right, or the clock in the middle. */
type Side = "left" | "right" | "clock";

/** One step of giving way: the side it frees, what it says, and what it changes. */
export interface Rung {
  readonly side: Side;
  readonly says: string;
  readonly apply: (state: BarState) => void;
}

/**
 * The order the bar gives way in, Kuba's and its proposals: the title first, then the layout; what
 * only shows state before what opens something; the clock, the system monitor, the mute pill, status
 * and the bell always there, and the recording's dot never gone. It reaches down to 540; a phone's
 * width is a topic of its own. A rung is taken only while its side runs short of the clock's clear
 * space (a clock rung, while either side does), and nothing else goes while
 * the title still shows.
 */
export const RUNGS: readonly Rung[] = [
  { side: "left", says: "Title gone", apply: (s) => void (s.title = false) },
  { side: "left", says: "Keyboard layout goes", apply: (s) => void (s.layout = false) },
  { side: "clock", says: "Countdown goes", apply: (s) => void (s.countdown = false) },
  { side: "left", says: "Keep awake and Proxy go", apply: (s) => void (s.idle = false) },
  { side: "clock", says: "Date drops the month", apply: (s) => void (s.date = "day") },
  { side: "clock", says: "Date goes", apply: (s) => void (s.date = "none") },
  { side: "left", says: "Recording keeps its dot", apply: (s) => void (s.recording = "dot") },
  { side: "right", says: "Tray apps fold into ⋯", apply: (s) => void (s.tray = "folded") },
  { side: "left", says: "Update joins ⋯", apply: (s) => void (s.update = false) },
  { side: "left", says: "Preview joins ⋯", apply: (s) => void (s.preview = false) },
  { side: "left", says: "Group name goes", apply: (s) => void (s.name = false) },
  { side: "left", says: "Ghosts go", apply: (s) => void (s.ghosts = false) },
  { side: "right", says: "Now playing drops ⏮ ⏭", apply: (s) => void (s.playing = "play") },
  { side: "right", says: "System monitor turns to rings", apply: (s) => void (s.sysmon = "rings") },
  { side: "right", says: "Battery % goes", apply: (s) => void (s.shed = 1) },
  { side: "right", says: "Now playing goes", apply: (s) => void (s.playing = "none") },
  { side: "right", says: "Power mode goes", apply: (s) => void (s.shed = 2) },
  { side: "right", says: "Network goes", apply: (s) => void (s.shed = 3) },
  { side: "right", says: "Bluetooth goes", apply: (s) => void (s.shed = 4) },
  { side: "right", says: "Mute pill keeps the mic", apply: (s) => void (s.mute = "mic") },
];

function stateOf(taken: ReadonlySet<number>): BarState {
  const state = { ...FULL };
  RUNGS.forEach((rung, index) => taken.has(index) && rung.apply(state));
  return state;
}

/** The workspaces as the state shows them: a borrowed group's name and the ghosts while they last. */
function workspacesFor(content: BarContent, state: BarState, options: WorkspacesOptions = {}): HTMLElement {
  const own = state.ghosts ? SHOTS_GROUPS[0]! : { ...SHOTS_GROUPS[0]!, spaces: SHOTS_GROUPS[0]!.spaces.slice(0, 2) };
  const desk = SHOTS_GROUPS[1]!;
  const borrowed = { ...desk, ...(state.name ? {} : { name: "" }), spaces: state.ghosts ? desk.spaces : desk.spaces.slice(0, 1) };
  return workspaces({ groups: content.borrowed ? [own, borrowed] : [own], ...options });
}

interface Drawn {
  readonly row: HTMLElement;
  readonly left: HTMLElement;
  readonly clock: HTMLElement;
  readonly right: HTMLElement;
  readonly title: HTMLElement | null;
}

/** The bar's row in `state`: the left, the clock centred, the right. */
function draw(content: BarContent, state: BarState, shown: Shown, options: WorkspacesOptions = {}): Drawn {
  const left = document.createElement("span");
  left.className = "shell-bar-left";
  left.append(workspacesFor(content, state, options));
  if (content.preview && state.preview) left.append(previewPill(BRANCH, shown));
  if (content.recording) left.append(state.recording === "dot" ? recordingDot(shown) : recordingPill({ state: shown }));
  if (content.keepAwake && state.idle) left.append(idlePill({ state: shown }));
  if (content.proxy && state.idle) left.append(proxyPill(false, shown));
  if (content.layout && state.layout) left.append(keyboardPill("US", shown));
  if (content.update && state.update) left.append(updatePill(undefined, shown));
  const titled = content.title !== "" && state.title ? title(content.title) : null;
  if (titled) left.append(titled);
  const clock = compactClock({ date: state.date, countdown: content.countdown && state.countdown, state: shown });
  const right = document.createElement("span");
  right.className = "shell-bar-right";
  const folded = state.tray === "folded" || (content.update && !state.update) || (content.preview && !state.preview);
  right.append(content.tray && !folded ? trayPill() : morePill(shown));
  right.append(state.sysmon === "rings" ? rimPill(LOADS.slice(0, 3), shown) : sysmonPill({ readings: READINGS.slice(0, content.readings), state: shown }));
  // The audio pills' circles take the state layer in a pill's place: lit or pressed, the first's.
  const circle = shown === "lit" || shown === "pressed" ? shown : undefined;
  if (content.playing && state.playing !== "none") right.append(nowPlaying({ compact: state.playing === "play", ...(circle ? { pinned: { part: "play", state: circle } } : {}) }));
  right.append(mutePill({ ...(state.mute === "mic" ? { micOnly: true } : {}), ...(circle ? { pinned: { side: state.mute === "mic" ? "input" : "output", state: circle } } : {}) }), bellPill(false, shown), shedStatus({ shed: state.shed, state: shown }));
  const row = document.createElement("span");
  row.className = "shell-bar-row";
  row.append(left, clock, right);
  return { row, left, clock, right, title: titled };
}

/** The recording's dot alone, its time in its hint. */
function recordingDot(shown: Shown): HTMLElement {
  const dot = document.createElement("span");
  dot.className = "shell-dot";
  const made = control([dot], "Recording 1:23. Click to stop", shown, { tone: "recording", end: "s12" });
  return made;
}

/** A rung's state's fit at `width` logical pixels, measured where the page draws. */
interface Measured {
  readonly leftFits: boolean;
  readonly rightFits: boolean;
  readonly titleRoom: number;
}

function measure(width: number, content: BarContent, state: BarState): Measured {
  const host = document.createElement("span");
  host.style.cssText = "position:absolute;visibility:hidden;left:0;top:0";
  const strip = document.createElement("span");
  strip.className = "shell-bar-strip";
  strip.style.setProperty("--strip-width", `calc(${width} * var(--px))`);
  const drawn = draw(content, { ...state, title: false }, "rest");
  strip.append(drawn.row);
  host.append(strip);
  (document.getElementById("page") ?? document.body).append(host);
  const scale = Number(getComputedStyle(host).getPropertyValue("--scale")) || 1;
  const of = (element: Element) => element.getBoundingClientRect().width / scale;
  const gap = length("space-s8");
  // Each side's own pills and their gaps: its grid column shrinks with the bar, its pills don't.
  const need = (side: HTMLElement) => {
    const items = [...side.children];
    return items.reduce((sum, each) => sum + of(each), 0) + gap * (items.length - 1);
  };
  const leftNeed = need(drawn.left);
  const rightNeed = need(drawn.right);
  const clockWidth = of(drawn.clock);
  host.remove();
  const clear = length("space-s40");
  const half = (width - 2 * gap - clockWidth) / 2;
  return { leftFits: leftNeed + clear <= half, rightFits: rightNeed + clear <= half, titleRoom: half - clear - leftNeed - length("space-s12") };
}

/** The rungs the bar takes at `width` logical pixels to clear its centred clock, and its state. */
export function fitBar(width: number, content: BarContent): { readonly taken: ReadonlySet<number>; readonly state: BarState; readonly fits: boolean } {
  const taken = new Set<number>();
  let got = measure(width, content, stateOf(taken));
  const fits = () => got.leftFits && got.rightFits;
  for (let index = 1; index < RUNGS.length && !fits(); index++) {
    const side = RUNGS[index]!.side;
    const short = side === "left" ? !got.leftFits : side === "right" ? !got.rightFits : true;
    if (!short) continue;
    taken.add(index);
    got = measure(width, content, stateOf(taken));
  }
  // The title goes first: once anything else has given way, or under space.s48 × 2 of room.
  if (taken.size > 0 || got.titleRoom < 2 * length("space-s48")) taken.add(0);
  return { taken, state: stateOf(taken), fits: fits() };
}

export interface BarOptions {
  /** The bar's width in logical pixels. */
  readonly width: number;
  readonly content?: BarContent;
  readonly workspaces?: WorkspacesOptions;
  readonly state?: Shown;
}

/** The bar alone, `width` logical pixels wide, on the frame's glass, given way as far as it needs. */
export function barStrip(options: BarOptions): HTMLElement {
  const content = options.content ?? EVERYDAY;
  const { state } = fitBar(options.width, content);
  const strip = document.createElement("span");
  strip.className = "shell-bar-strip";
  strip.style.setProperty("--strip-width", `calc(${options.width} * var(--px))`);
  strip.append(draw(content, state, options.state ?? "rest", options.workspaces).row);
  return strip;
}

/** The bar on a screen `width` logical pixels by `height` (a length's CSS), its frame round the wallpaper. */
export function barScreen(width: number, height: string, options: Omit<BarOptions, "width"> = {}): Screen {
  const content = options.content ?? EVERYDAY;
  // Inside the frame's sides, as a strip of the screen's width is inside its own.
  const { state } = fitBar(width, content);
  return screen({ width: `calc(${width} * var(--px))`, height, wallpaper, bar: [draw(content, state, options.state ?? "rest", options.workspaces).row] });
}

/** A window size's width in logical pixels. */
export function logicalWidth(css: string): number {
  const calc = /calc\((\d+) \* var\(--px\)\)/.exec(css);
  if (calc) return Number(calc[1]);
  const token = /var\(--([a-z0-9_-]+)\)/.exec(css);
  return token ? length(token[1]!) : 0;
}
