// The Audio panel as `config/quickshell/modules/audio/` is to draw it: a PanelPage on a Sheet at the
// frame's right edge, under the bar. Its body, from the top: the output's slider, which boosts to
// 150% holding at 100%, and the mic's, each one's icon muting it; the mic's live Meter; a Bluetooth
// headset's Calls and Music with the line under the one picked; then Output, Input and Options,
// stretched Tabs, over the picked tab's cards. Every part is the library's; the data is the shots'
// stand-ins'.

import "./audio.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { filling, row, stack } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { menu } from "../../lib/menu.ts";
import { liveMeter } from "../../lib/meter.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { select } from "../../lib/select.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { slider } from "../../lib/slider.ts";
import { setTabDisabled, tabs } from "../../lib/tabs.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { popover } from "../../lib/tooltip.ts";
import { APP_ICONS } from "../../site/app-icons.ts";
import { ALL, CHROME } from "../../site/settings.ts";
import { voice } from "../molecules/meter.ts";
import { mutePill, type MuteOptions, volumeIcon } from "./bar-audio-parts.ts";
import { bellPill, statusPill, sysmonPill } from "./bar-parts.ts";
import { CORNER, fadeIn, TALL } from "./control-center-parts.ts";

// The locale's percentage: a value is never put together from a number and a sign by hand.
const PERCENT = new Intl.NumberFormat("en-GB", { style: "percent", maximumFractionDigits: 0 });

/** The most the output's slider reaches: 150%, its value's 1. */
export const BOOST = 1.5;

/** The panel's header: its title, and its pun, which never moves. */
export const TITLE = "Audio";
export const PUN = "Loud and clear";

export const OUTPUTS = ["Built-in speakers", "WH-1000XM4", "LG ULTRAGEAR (HDMI)"] as const;
export const INPUTS = ["Built-in microphone", "WH-1000XM4"] as const;
/** A device whose name runs long, as a USB interface names itself. */
export const LONG_DEVICE = "Focusrite Scarlett 2i2 4th Gen USB Audio Interface Analog Stereo";

/** An app with a stream: its name, its icon, the device it plays to or records from, its level. */
export interface App {
  readonly name: string;
  readonly picture: string;
  readonly device: string;
  readonly level: number;
  readonly muted?: boolean;
}

const [KUBAOS, PLATE, DISC, STAR, WIDE] = APP_ICONS;

export const PLAYING: readonly App[] = [
  { name: "Spotify", picture: DISC!.picture, device: OUTPUTS[0], level: 0.7 },
  { name: "Firefox", picture: KUBAOS!.picture, device: OUTPUTS[0], level: 0.45 },
];

export const RECORDING: readonly App[] = [{ name: "Discord", picture: PLATE!.picture, device: INPUTS[0], level: 0.6 }];

/** Many apps playing at once, for the Apps card past its room. */
export const MANY: readonly App[] = [
  ...PLAYING,
  { name: "Discord", picture: PLATE!.picture, device: OUTPUTS[1], level: 0.8 },
  { name: "mpv", picture: STAR!.picture, device: OUTPUTS[0], level: 1 },
  { name: "Steam", picture: WIDE!.picture, device: OUTPUTS[2], level: 0.35, muted: true },
  { name: "Chromium", picture: KUBAOS!.picture, device: OUTPUTS[0], level: 0.55 },
];

/** An app whose name runs long. */
export const LONG_APP: App = { name: "Visual Studio Code — Insiders (Electron)", picture: KUBAOS!.picture, device: LONG_DEVICE, level: 0.5 };

export type Headset = "calls" | "music" | null;
export const TABS = ["Output", "Input", "Options"] as const;

export interface AudioOptions {
  /** A Bluetooth headset's mode, or none with no headset. */
  readonly headset?: Headset;
  /** The tab picked: Output, Input or Options. */
  readonly tab?: 0 | 1 | 2;
  /** The output's level, 0 to `BOOST`. */
  readonly output?: number;
  readonly outputMuted?: boolean;
  /** The mic's level, 0 to 1. */
  readonly input?: number;
  readonly inputMuted?: boolean;
  /** The meter, still: its level and its peak tick. */
  readonly meter?: { readonly level: number; readonly peak: number };
  readonly outputDevice?: string;
  readonly inputDevice?: string;
  /** Balance, 0 left to 1 right, the centre 0.5. */
  readonly balance?: number;
  readonly playing?: readonly App[];
  readonly recording?: readonly App[];
  readonly noise?: boolean;
  readonly echo?: boolean;
  readonly takeover?: boolean;
  /** Drawn live: the meter hears a voice, and the parts answer one another. */
  readonly live?: boolean;
  /** The output's slider pinned lit, as the keyboard lands on it as the panel opens. */
  readonly landed?: boolean;
  /** Scrolled this share of the way down, as a sample of it scrolled. */
  readonly down?: number;
  /** For a move's sample: the tab, and the headset's mode, a pick comes from. */
  readonly tabFrom?: number;
  readonly modeFrom?: "calls" | "music";
}

/** Balance as its words: the side and how far toward it, or Centre in the middle. */
export function sideOf(at: number): string {
  const far = Math.round(Math.abs(at - 0.5) * 200);
  return far === 0 ? "Centre" : `${at < 0.5 ? "Left" : "Right"} ${PERCENT.format(far / 100)}`;
}

/** The output's slider: compact, boosting to 150% and holding at 100%, its icon muting it. */
function outputSlider(options: AudioOptions): HTMLElement {
  const level = options.output ?? 0.62;
  return slider({
    icon: volumeIcon(Math.min(1, level)),
    size: "Compact",
    value: level / BOOST,
    sticky: 1 / BOOST,
    number: (at) => PERCENT.format(at * BOOST),
    widest: "150%",
    toggle: { icon: "volume_off", words: "Muted", label: "Mute", toggledLabel: "Unmute", on: options.outputMuted ?? false },
    ...(options.landed ? { pinned: "lit" as const } : {}),
  });
}

/** The mic's slider, its icon muting it, telling `muted` as it turns. */
function micSlider(level: number, muted: boolean, told?: (on: boolean) => void): HTMLElement {
  return slider({
    icon: "mic",
    size: "Compact",
    value: level,
    toggle: { icon: "mic_off", words: "Muted", label: "Mute the mic", toggledLabel: "Unmute the mic", on: muted, ...(told ? { changed: told } : {}) },
  });
}

/** What a headset's mode says under Calls and Music. */
export const LINES: Record<"calls" | "music", string> = {
  calls: "Your mic works; music sounds flatter while it's on.",
  music: "Music mode has no microphone: switch to Calls to talk.",
};

/** A card's label over what it holds, as a list's section label reads. */
function label(words: string): HTMLElement {
  return clamped(words, "label", "on_surface_variant");
}

/** A card of `parts`, `space.s8` apart under its label. */
function cardOf(name: string, ...parts: HTMLElement[]): HTMLElement {
  const made = card({ body: [stack("s8", "start", filling(label(name)), ...parts.map(filling))] });
  made.classList.add("audio-card");
  return made;
}

/**
 * An app's device as a menu: a Text sm button with its chevron, its words in line with the name
 * over it, opening a Menu of the devices under it; a pick moves the app there.
 */
function deviceMenu(device: string, devices: readonly string[]): HTMLElement {
  const room = document.createElement("span");
  room.className = "audio-device";
  const made = button({ label: device, kind: "Text", size: "Sm", disclosure: false });
  room.append(made);
  let open: HTMLElement | null = null;
  const close = () => {
    open?.remove();
    open = null;
    made.style.setProperty("--disclosed", "0");
    document.removeEventListener("pointerdown", outside, true);
  };
  const outside = (event: PointerEvent) => {
    if (!room.contains(event.target as Node)) close();
  };
  made.addEventListener("click", () => {
    if (open) return close();
    const picked = Math.max(0, devices.indexOf(device));
    const list_ = menu(
      devices.map((name, index) => ({ label: name, checked: index === picked })),
      {
        current: picked,
        onTake: (index) => {
          const words = made.querySelectorAll<HTMLElement>(".k-button-label");
          for (const each of words) each.textContent = devices[index]!;
          close();
          made.focus({ preventScroll: true });
        },
        onLeave: () => {
          close();
          made.focus({ preventScroll: true });
        },
      },
    );
    open = popover(2, list_);
    open.classList.add("audio-device-menu");
    room.append(open);
    made.style.setProperty("--disclosed", "1");
    document.addEventListener("pointerdown", outside, true);
    list_.focus({ preventScroll: true });
  });
  return room;
}

/** One app's row: its tile, its name over its device, then its slider across, its icon muting it.
    The name lines up with the tile's top and the device button's taller box hangs below it, so
    the name never rides above the tile. */
function appRow(app: App, devices: readonly string[], input: boolean): HTMLElement {
  const tile = iconTile({ glyph: input ? "mic" : "volume_up", size: "Md", picture: app.picture });
  const words = stack(0, "start", filling(clamped(app.name, "body_strong")), deviceMenu(app.device, devices));
  const head = row("s12", "start", tile, filling(words));
  const level = slider({
    icon: input ? "mic" : volumeIcon(app.level),
    size: "Compact",
    value: app.level,
    toggle: input
      ? { icon: "mic_off", words: "Muted", label: `Mute ${app.name}`, toggledLabel: `Unmute ${app.name}`, on: app.muted ?? false }
      : { icon: "volume_off", words: "Muted", label: `Mute ${app.name}`, toggledLabel: `Unmute ${app.name}`, on: app.muted ?? false },
  });
  const made = stack("s8", "start", filling(head), filling(level));
  made.classList.add("audio-app");
  return made;
}

/** What the Apps card says with no app: nothing playing, or nothing recording. */
export const NO_APPS = { output: "Nothing is playing", input: "No app is recording" } as const;

/** The Apps card: a row per app, `space.s16` apart, or its empty line. */
function appsCard(apps: readonly App[], devices: readonly string[], input: boolean): HTMLElement {
  const rows = apps.length === 0 ? [text(input ? NO_APPS.input : NO_APPS.output, "body", "on_surface_variant")] : apps.map((app) => appRow(app, devices, input));
  const column = stack("s16", "start", ...rows.map(filling));
  return cardOf("Apps", column);
}

/** The Output tab: the Device card (its Select, balance) and the Apps card. */
function outputTab(options: AudioOptions): HTMLElement {
  const device = options.outputDevice ?? OUTPUTS[0];
  const devices = OUTPUTS.includes(device as (typeof OUTPUTS)[number]) ? [...OUTPUTS] : [device, ...OUTPUTS];
  const pick = select({ options: devices, picked: devices.indexOf(device), icon: "speaker" });
  const knob = slider({ icon: "balance", size: "Compact", knob: true, value: options.balance ?? 0.5, sticky: 0.5, number: sideOf });
  return stack("s12", "start", cardOf("Device", pick, knob), filling(appsCard(options.playing ?? PLAYING, devices, false)));
}

/** The Input tab: the Device card (its Select) and the Apps card. */
function inputTab(options: AudioOptions): HTMLElement {
  const device = options.inputDevice ?? (options.headset ? INPUTS[1] : INPUTS[0]);
  const devices = INPUTS.includes(device as (typeof INPUTS)[number]) ? [...INPUTS] : [device, ...INPUTS];
  const pick = select({ options: devices, picked: devices.indexOf(device), icon: "mic" });
  const recording = (options.recording ?? RECORDING).map((app) => ({ ...app, device: options.headset && app.device === INPUTS[0] ? INPUTS[1] : app.device }));
  return stack("s12", "start", cardOf("Device", pick), filling(appsCard(recording, devices, true)));
}

/** The Input tab in Music mode: why, and the way back to Calls. */
function musicEmpty(toCalls?: () => void): HTMLElement {
  const back = button({ label: "Switch to Calls", kind: "Text", icon: "call" });
  if (toCalls) back.addEventListener("click", toCalls);
  const made = card({ body: [emptyState({ icon: "mic_off", title: "Music mode is on", body: "The headset has no microphone in Music mode. Switch to Calls to use it.", action: back })] });
  made.classList.add("audio-card");
  return made;
}

/** The Options tab: a switch row each. */
function optionsTab(options: AudioOptions): HTMLElement {
  return list([
    { title: "Noise suppression", subtitle: "Your mic, without the background", toggle: options.noise ?? true },
    { title: "Echo cancellation", subtitle: "For calls on the speakers", toggle: options.echo ?? false },
    { title: "New devices take over", subtitle: "A headset you connect becomes the default", toggle: options.takeover ?? true },
  ]);
}

/** The picked tab's content: Input in Music mode is its empty state. */
function tabContent(tab: number, options: AudioOptions, toCalls?: () => void): HTMLElement {
  if (tab === 1 && options.headset === "music") return musicEmpty(toCalls);
  return tab === 0 ? outputTab(options) : tab === 1 ? inputTab(options) : optionsTab(options);
}

/** The headset's mode, Calls or Music, and the line under the one picked. */
function modeOf(headset: "calls" | "music", onPick?: (mode: "calls" | "music") => void, from?: "calls" | "music"): { readonly well: HTMLElement; readonly line: HTMLElement } {
  const well = tabs({
    choices: [
      { label: "Calls", icon: "call" },
      { label: "Music", icon: "music_note" },
    ],
    picked: headset === "calls" ? 0 : 1,
    ...(from ? { from: from === "calls" ? 0 : 1 } : {}),
    ...(onPick ? { onPick: (index: number) => onPick(index === 0 ? "calls" : "music") } : {}),
  });
  return { well, line: modeLine(headset) };
}

/** The line under Calls and Music, for `mode`. */
function modeLine(mode: "calls" | "music"): HTMLElement {
  const line = filling(wrapped(LINES[mode], "small", "on_surface_variant"));
  line.classList.add("audio-line");
  return line;
}

export interface Panel {
  readonly element: HTMLElement;
  /** Where the keyboard lands as the panel opens: the output's slider. */
  readonly land: () => void;
}

/** The whole panel, as a PanelPage: the header, then the body that scrolls. */
export function audioPanel(options: AudioOptions = {}): Panel {
  let state: AudioOptions = { ...options };
  let tab: number = options.tab ?? 0;
  const live = options.live ?? false;
  const output = outputSlider(state);
  const inputMuted = () => (state.inputMuted ?? false) || state.headset === "music";
  const meterOf = liveMeter(state.meter?.level ?? 0.46, state.meter?.peak ?? 0.68);
  if (inputMuted()) meterOf.mute(true);
  let mic = micSlider(state.input ?? 0.8, inputMuted(), live ? (on) => meterOf.mute(on) : undefined);
  const room = document.createElement("span");
  room.className = "audio-tab";
  const picker = tabs({
    choices: TABS.map((name, index) => ({ label: name, ...(index === 1 && state.headset === "music" ? { disabled: true } : {}) })),
    picked: tab,
    ...(options.tabFrom !== undefined ? { from: options.tabFrom } : {}),
    onPick: (index) => {
      tab = index;
      show();
    },
  });

  function show(): void {
    room.replaceChildren(tabContent(tab, state, live ? () => headsetPicker?.querySelector<HTMLElement>(".k-tab")?.click() : undefined));
    if (live) fadeIn(room);
  }

  /** The headset's mode changes: the mic, its meter, the line, Input, and Input's content if it shows. */
  function switchMode(mode: "calls" | "music"): void {
    state = { ...state, headset: mode };
    const said = modeLine(mode);
    line?.replaceWith(said);
    line = said;
    const fresh = micSlider(state.input ?? 0.8, inputMuted(), (on) => meterOf.mute(on));
    mic.replaceWith(fresh);
    mic = fresh;
    meterOf.mute(inputMuted());
    setTabDisabled(picker, 1, mode === "music");
    if (tab === 1) show();
  }

  const headset = state.headset ?? null;
  const mode = headset ? modeOf(headset, live ? switchMode : undefined, options.modeFrom) : null;
  const headsetPicker = mode?.well ?? null;
  let line = mode?.line ?? null;
  show();

  const body = stack(
    "s16",
    "start",
    filling(stack("s8", "start", filling(output), filling(mic), filling(meterOf.bar))),
    ...(mode && line ? [filling(stack("s4", "start", filling(mode.well), filling(line)))] : []),
    filling(stack("s12", "start", filling(picker), filling(room))),
  );
  body.classList.add("audio-body");
  const page = panelPage({ title: TITLE, subtitle: PUN, body, ...(options.down !== undefined ? { down: options.down } : {}) });
  page.element.classList.add("audio-panel");

  if (live && CHROME && !ALL) {
    const says = voice();
    let frame = 0;
    const speak = (now: number) => {
      meterOf.feed(says(now) * (state.input ?? 0.8) * 1.25, now);
      frame = requestAnimationFrame(speak);
    };
    new IntersectionObserver(([entry]) => {
      cancelAnimationFrame(frame);
      if (entry?.isIntersecting && page.element.isConnected) frame = requestAnimationFrame(speak);
    }).observe(page.element);
  }
  return { element: page.element, land: () => output.focus({ focusVisible: true, preventScroll: true } as FocusOptions) };
}

/** A screen's height the panel runs past: the shortest the frame keeps for a panel. */
export const SHORT = "calc(var(--size-app_min_height) + var(--space-s64))";

/** The bar over the panel: the system monitor, the mute pill, the bell and status. */
export function audioBar(mute: MuteOptions = {}): Element[] {
  return [sysmonPill(), mutePill(mute), bellPill(), statusPill()];
}

/** The panel open on a screen's corner, under the bar, drawn still unless `reveal` says how far open. */
export function onScreen(panel: HTMLElement, height = TALL, reveal?: number, mute: MuteOptions = {}): HTMLElement {
  const room = screen({ width: CORNER, height, wallpaper, bar: audioBar(mute) });
  room.element.dataset.open = "";
  hang(room, sheet({ content: panel, edge: "right", ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}

/** The panel in `options` on a screen of `height`. */
export function shot(options: AudioOptions = {}, height?: string): HTMLElement {
  return onScreen(audioPanel(options).element, height);
}
