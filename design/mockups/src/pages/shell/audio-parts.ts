// The Audio panel as `config/quickshell/modules/audio/` is to draw it: a PanelPage on a Sheet at the
// frame's right edge, under the bar. Its body, from the top: the output's slider, which boosts to
// 150% holding at 100%, and the mic's, each one's icon muting it; the mic's live Meter; the Device
// card, the default device for both sides, with a Bluetooth headset's Calls and Music, the line
// under the one picked and its codec; then Output, Input and Options, stretched Tabs, over the
// picked tab's cards, where each side, then each app, may take another device. Every part is the
// library's; the data is the shots' stand-ins'.

import "./audio.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { filling, row, stack } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { liveMeter } from "../../lib/meter.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { select } from "../../lib/select.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { slider } from "../../lib/slider.ts";
import { setTabDisabled, tabs } from "../../lib/tabs.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { floatingTooltip, tooltip } from "../../lib/tooltip.ts";
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

/** How a device connects, which its symbol says. */
export type Link = "bluetooth" | "usb" | "jack" | "hdmi" | "builtin";

/** Each link's symbol: built-in speakers and the built-in mic alike are the laptop. */
export const LINK_ICONS: Record<Link, string> = {
  bluetooth: "bluetooth",
  usb: "usb",
  jack: "headphones",
  hdmi: "device_tv",
  builtin: "device_laptop",
};

/** Each link as the device's tooltip names it. */
const LINK_WORDS: Record<Link, string> = {
  bluetooth: "Bluetooth",
  usb: "USB",
  jack: "Headphone jack",
  hdmi: "HDMI",
  builtin: "Built-in",
};

/** A headset's mode: Calls (its mic works) or Music (no mic). */
export type Mode = "calls" | "music";

/** A device: its name, how it connects, whether it plays and records, and a headset's codecs. */
export interface Device {
  readonly name: string;
  readonly link: Link;
  readonly plays: boolean;
  readonly records: boolean;
  /** What a Bluetooth headset supports in each mode, best first. */
  readonly codecs?: Readonly<Record<Mode, readonly string[]>>;
}

export const HEADSET: Device = { name: "WH-1000XM4", link: "bluetooth", plays: true, records: true, codecs: { music: ["LDAC", "AAC", "SBC"], calls: ["mSBC", "CVSD"] } };
export const USB_HEADSET: Device = { name: "Arctis Nova Pro Wireless", link: "usb", plays: true, records: true };
export const SPEAKERS: Device = { name: "Built-in speakers", link: "builtin", plays: true, records: false };
export const BUILT_IN_MIC: Device = { name: "Built-in microphone", link: "builtin", plays: false, records: true };
export const SCREEN: Device = { name: "LG ULTRAGEAR", link: "hdmi", plays: true, records: false };
export const JACK: Device = { name: "Headphones", link: "jack", plays: true, records: false };
/** A device whose name runs long, as a USB interface names itself. */
export const LONG_DEVICE: Device = { name: "Focusrite Scarlett 2i2 4th Gen USB Audio Interface Analog Stereo", link: "usb", plays: true, records: true };

/** Every device the shots' laptop has, in the order the Selects list them. */
export const DEVICES: readonly Device[] = [SPEAKERS, BUILT_IN_MIC, HEADSET, USB_HEADSET, JACK, SCREEN];

/** What an app's device picker says while the app follows its side's device. */
export const FOLLOWS = "Default";

/** An app with a stream: its name, its icon, the device it plays to or records from, its level. */
export interface App {
  readonly name: string;
  readonly picture: string;
  /** The device it is moved to, or none while it follows its side's. */
  readonly device?: Device;
  readonly level: number;
  readonly muted?: boolean;
}

const [KUBAOS, PLATE, DISC, STAR, WIDE] = APP_ICONS;

export const PLAYING: readonly App[] = [
  { name: "Spotify", picture: DISC!.picture, level: 0.7 },
  { name: "Firefox", picture: KUBAOS!.picture, device: SPEAKERS, level: 0.45 },
];

export const RECORDING: readonly App[] = [{ name: "Discord", picture: PLATE!.picture, level: 0.6 }];

/** Many apps playing at once, for the Apps card past its room. */
export const MANY: readonly App[] = [
  ...PLAYING,
  { name: "Discord", picture: PLATE!.picture, device: HEADSET, level: 0.8 },
  { name: "mpv", picture: STAR!.picture, level: 1 },
  { name: "Steam", picture: WIDE!.picture, device: SCREEN, level: 0.35, muted: true },
  { name: "Chromium", picture: KUBAOS!.picture, level: 0.55 },
];

/** An app whose name runs long. */
export const LONG_APP: App = { name: "Visual Studio Code — Insiders (Electron)", picture: KUBAOS!.picture, device: LONG_DEVICE, level: 0.5 };

export const TABS = ["Output", "Input", "Options"] as const;

export interface AudioOptions {
  /** The default device, both sides': the laptop's speakers when none is given. */
  readonly device?: Device;
  /** A Bluetooth default device's mode: Calls when none is given. */
  readonly mode?: Mode;
  /** The codec picked for the mode: the best when none is given. */
  readonly codec?: string;
  /** The device's tooltip shown, as under the pointer. */
  readonly tooltip?: boolean;
  /** The output's own device over the default, or none while it follows it. */
  readonly outputDevice?: Device;
  /**
   * The mic's own device over the default, or none while it follows it. With no device given at
   * all, the laptop's speakers are the default and the built-in mic its override.
   */
  readonly inputDevice?: Device | null;
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
  /** An app's device picker pinned in a state, or drawn open (upward, as near the panel's foot): the first app's. */
  readonly appPicker?: "lit" | "open";
  /** Scrolled this share of the way down, as a sample of it scrolled. */
  readonly down?: number;
  /** For a move's sample: the tab, and the headset's mode, a pick comes from. */
  readonly tabFrom?: number;
  readonly modeFrom?: Mode;
}

/** The default device and each side's, worked out from `options`. */
interface Devices {
  readonly default: Device;
  /** The output's override, if any. */
  readonly output: Device | null;
  /** The mic's override, if any. */
  readonly input: Device | null;
  /** The mode, for a Bluetooth default; none for any other device. */
  readonly mode: Mode | null;
}

function devicesOf(options: AudioOptions): Devices {
  const fallback = options.device === undefined;
  const chosen = options.device ?? SPEAKERS;
  const input = options.inputDevice === undefined ? (fallback ? BUILT_IN_MIC : null) : options.inputDevice;
  return { default: chosen, output: options.outputDevice ?? null, input, mode: chosen.link === "bluetooth" ? (options.mode ?? "calls") : null };
}

/** Whether the mic is a headset's own in Music mode, which has none: Input is then off. */
function micInMusic(devices: Devices): boolean {
  return devices.mode === "music" && (devices.input === null || devices.input === devices.default);
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
export const LINES: Record<Mode, string> = {
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

/** A device's name and symbol in a Select's list. */
function names(devices: readonly Device[]): { readonly options: string[]; readonly icons: string[] } {
  return { options: devices.map((each) => each.name), icons: devices.map((each) => LINK_ICONS[each.link]) };
}

/** What a device's tooltip names: its link, and a headset's codec. */
export function linkOf(device: Device, codec?: string): string {
  return codec ? `${LINK_WORDS[device.link]} · ${codec}` : LINK_WORDS[device.link];
}

/** The top Device card's Select: every device that plays, the default picked, its tooltip over it. */
function defaultPicker(devices: Devices, codec: string | undefined, shown: boolean): HTMLElement {
  const playing = DEVICES.filter((each) => each.plays);
  const listed = playing.includes(devices.default) ? playing : [devices.default, ...playing];
  const pick = select({ ...names(listed), picked: listed.indexOf(devices.default), ...(shown ? { pinned: "lit" as const } : {}) });
  const anchor = floatingTooltip(pick, tooltip(linkOf(devices.default, codec), ""), { side: "above", shown });
  return anchor;
}

/** A Bluetooth headset's codecs in its mode, `codec` picked. */
function codecPicker(device: Device, mode: Mode, codec: string | undefined): HTMLElement {
  const codecs = device.codecs?.[mode] ?? [];
  return stack("s8", "start", filling(label("Codec")), filling(select({ options: codecs, picked: Math.max(0, codecs.indexOf(codec ?? codecs[0] ?? "")) })));
}

/**
 * A side's Device card's Select: "Default · <device>" first, with the default's symbol, then every
 * other device of that side. A default that has nothing for the side leaves it unpicked, saying so.
 */
function sidePicker(devices: Devices, input: boolean): HTMLElement {
  const fits = (each: Device) => (input ? each.records : each.plays);
  const chosen = input ? devices.input : devices.output;
  const others = DEVICES.filter((each) => fits(each) && each !== devices.default);
  const all = chosen && !others.includes(chosen) && chosen !== devices.default ? [chosen, ...others] : others;
  if (!fits(devices.default)) {
    return select({ ...names(all), picked: chosen ? all.indexOf(chosen) : -1, placeholder: `No mic on ${devices.default.name}` });
  }
  const listed = names(all);
  return select({
    options: [`${FOLLOWS} · ${devices.default.name}`, ...listed.options],
    icons: [LINK_ICONS[devices.default.link], ...listed.icons],
    picked: chosen && chosen !== devices.default ? 1 + all.indexOf(chosen) : 0,
  });
}

/**
 * An app's device, an inline Select under its name: "Default" while it follows its side, then
 * every device of that side; a pick moves the app there.
 */
function appPicker(app: App, side: readonly Device[], following: Device, state?: "lit" | "open"): HTMLElement {
  const all = app.device && !side.includes(app.device) ? [app.device, ...side] : side;
  const listed = names(all);
  return select({
    look: "inline",
    options: [FOLLOWS, ...listed.options],
    icons: [LINK_ICONS[following.link], ...listed.icons],
    picked: app.device ? 1 + all.indexOf(app.device) : 0,
    ...(state === "lit" ? { pinned: "lit" as const } : state === "open" ? { open: true, up: true } : {}),
  });
}

/** One app's row: its tile, its name over its device centred on it, then its slider across, its icon muting it. */
function appRow(app: App, side: readonly Device[], following: Device, input: boolean, state?: "lit" | "open"): HTMLElement {
  const tile = iconTile({ glyph: input ? "mic" : "volume_up", size: "Md", picture: app.picture });
  const words = stack(0, "start", filling(clamped(app.name, "body_strong")), appPicker(app, side, following, state));
  const head = row("s12", "centre", tile, filling(words));
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
function appsCard(apps: readonly App[], devices: Devices, input: boolean, state?: "lit" | "open"): HTMLElement {
  const side = DEVICES.filter((each) => (input ? each.records : each.plays));
  const following = (input ? devices.input : devices.output) ?? devices.default;
  const rows =
    apps.length === 0
      ? [text(input ? NO_APPS.input : NO_APPS.output, "body", "on_surface_variant")]
      : apps.map((app, index) => appRow(app, side, following, input, index === 0 ? state : undefined));
  const column = stack("s16", "start", ...rows.map(filling));
  return cardOf("Apps", column);
}

/** The Output tab: its Device card (the output's own device, balance) and the Apps card. */
function outputTab(options: AudioOptions, devices: Devices): HTMLElement {
  const knob = slider({ icon: "balance", size: "Compact", knob: true, value: options.balance ?? 0.5, sticky: 0.5, number: sideOf });
  return stack("s12", "start", cardOf("Device", sidePicker(devices, false), knob), filling(appsCard(options.playing ?? PLAYING, devices, false, options.appPicker)));
}

/** The Input tab: its Device card (the mic's own device) and the Apps card. */
function inputTab(options: AudioOptions, devices: Devices): HTMLElement {
  return stack("s12", "start", cardOf("Device", sidePicker(devices, true)), filling(appsCard(options.recording ?? RECORDING, devices, true, options.appPicker)));
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

/** The picked tab's content: Input, while the mic is a headset's own in Music mode, is its empty state. */
function tabContent(tab: number, options: AudioOptions, devices: Devices, toCalls?: () => void): HTMLElement {
  if (tab === 1 && micInMusic(devices)) return musicEmpty(toCalls);
  return tab === 0 ? outputTab(options, devices) : tab === 1 ? inputTab(options, devices) : optionsTab(options);
}

/** Calls and Music, stretched with their symbols; disabled, with nothing picked, for a device that isn't Bluetooth. */
function modeTabs(mode: Mode | null, onPick?: (mode: Mode) => void, from?: Mode): HTMLElement {
  return tabs({
    choices: [
      { label: "Calls", icon: "call" },
      { label: "Music", icon: "music_note" },
    ],
    picked: mode === null ? null : mode === "music" ? 1 : 0,
    ...(mode === null ? { disabled: true } : {}),
    ...(from ? { from: from === "calls" ? 0 : 1 } : {}),
    ...(onPick ? { onPick: (index: number) => onPick(index === 0 ? "calls" : "music") } : {}),
  });
}

/** The line under Calls and Music, for `mode`. */
function modeLine(mode: Mode): HTMLElement {
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
  let devices = devicesOf(state);
  let tab: number = options.tab ?? 0;
  const live = options.live ?? false;
  const output = outputSlider(state);
  const inputMuted = () => (state.inputMuted ?? false) || micInMusic(devices);
  const meterOf = liveMeter(state.meter?.level ?? 0.46, state.meter?.peak ?? 0.68);
  if (inputMuted()) meterOf.mute(true);
  let mic = micSlider(state.input ?? 0.8, inputMuted(), live ? (on) => meterOf.mute(on) : undefined);
  const room = document.createElement("span");
  room.className = "audio-tab";
  const picker = tabs({
    choices: TABS.map((name, index) => ({ label: name, ...(index === 1 && micInMusic(devices) ? { disabled: true } : {}) })),
    picked: tab,
    ...(options.tabFrom !== undefined ? { from: options.tabFrom } : {}),
    onPick: (index) => {
      tab = index;
      show();
    },
  });

  const modes = modeTabs(devices.mode, live ? switchMode : undefined, options.modeFrom);
  let line = devices.mode ? modeLine(devices.mode) : null;
  let codec = devices.mode ? codecPicker(devices.default, devices.mode, state.codec) : null;
  let device = defaultPicker(devices, devices.mode ? (state.codec ?? devices.default.codecs?.[devices.mode][0]) : undefined, state.tooltip ?? false);

  function show(): void {
    room.replaceChildren(tabContent(tab, state, devices, live ? () => modes.querySelector<HTMLElement>(".k-tab")?.click() : undefined));
    if (live) fadeIn(room);
  }

  /** The headset's mode changes: the mic, its meter, the line, the codecs, Input, and Input's content if it shows.
      Each new part fills the width as the one it replaces did, or it shrinks to its content. */
  function switchMode(mode: Mode): void {
    // The codec picked was the other mode's: the new mode starts on its best.
    const { codec: _left, ...rest } = state;
    state = { ...rest, mode };
    devices = devicesOf(state);
    const named = filling(defaultPicker(devices, devices.default.codecs?.[mode][0], false));
    device.replaceWith(named);
    device = named;
    const said = filling(modeLine(mode));
    line?.replaceWith(said);
    line = said;
    const codecs = filling(codecPicker(devices.default, mode, undefined));
    codec?.replaceWith(codecs);
    codec = codecs;
    const fresh = filling(micSlider(state.input ?? 0.8, inputMuted(), (on) => meterOf.mute(on)));
    mic.replaceWith(fresh);
    mic = fresh;
    meterOf.mute(inputMuted());
    setTabDisabled(picker, 1, micInMusic(devices));
    if (tab === 1) show();
  }

  show();

  const top = cardOf(
    "Device",
    device,
    line ? stack("s4", "start", filling(modes), filling(line)) : modes,
    ...(codec ? [codec] : []),
  );
  const body = stack(
    "s16",
    "start",
    filling(stack("s8", "start", filling(output), filling(mic), filling(meterOf.bar))),
    filling(top),
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
