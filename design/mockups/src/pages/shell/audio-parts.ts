// The Audio panel as `config/quickshell/modules/audio/` is to draw it: a PanelPage on a Sheet at the
// frame's right edge, under the bar. Its body, from the top: the output's slider, which boosts to
// 150% holding at 100%, and the mic's, each one's icon muting it; the mic's live Meter; the Device
// card, the default device for both sides, with a Bluetooth headset's Calls and Music, each saying
// in its tooltip what it means, and its codec; then Output, Input and Options, stretched Tabs, over the
// picked tab's cards, where each side, then each app, may take another device. The header's pun
// follows the panel's state, the first of `SUBTITLES`' cases that holds. Every part is the
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
import { clamped, text } from "../../lib/text.ts";
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

/** The panel's header's title; its pun, the subtitle, follows the panel's state (`SUBTITLES`). */
export const TITLE = "Audio";

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

/** A machine with no built-in mic: its speakers stand alone as a default. */
export const NO_BUILT_IN_MIC: readonly Device[] = DEVICES.filter((each) => each !== BUILT_IN_MIC);

/**
 * One choice of the default device, the top Device card's: what plays, and the mic that comes with
 * it, which is the device itself when it records, the built-in mic for the built-in set, or none.
 */
export interface DefaultChoice {
  readonly name: string;
  readonly link: Link;
  readonly output: Device;
  readonly input: Device | null;
}

/** The built-in set's name: a machine's built-in speakers and built-in mic as one default. */
export const BUILT_IN_SET = "Built-in";

/** `device` as a default on its own: its own mic if it records, none if it doesn't. */
function alone(device: Device): DefaultChoice {
  return { name: device.name, link: device.link, output: device, input: device.records ? device : null };
}

/**
 * The default's choices on `machine`: every device that plays, in order. Built-in speakers with no
 * mic of their own pair with the machine's built-in mic, when it has one, as one choice, the built-in
 * set: it makes the speakers the default output and the mic the default input. Each side's own
 * Select and each app's keep the two apart.
 */
export function defaultChoices(machine: readonly Device[]): DefaultChoice[] {
  const builtInMic = machine.find((each) => each.link === "builtin" && each.records && !each.plays);
  return machine
    .filter((each) => each.plays)
    .map((each) =>
      each.link === "builtin" && !each.records && builtInMic ? { name: BUILT_IN_SET, link: each.link, output: each, input: builtInMic } : alone(each),
    );
}

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
  /** Every device the machine has: the shots' laptop's when none is given. */
  readonly machine?: readonly Device[];
  /**
   * The default device's output: the laptop's speakers when none is given, which with the built-in
   * mic is the built-in set.
   */
  readonly device?: Device;
  /** A Bluetooth default device's mode: Calls when none is given. */
  readonly mode?: Mode;
  /** The mode whose tooltip is drawn shown over its choice, as under the pointer. */
  readonly modeTip?: Mode;
  /** The codec picked for the mode: the best when none is given. */
  readonly codec?: string;
  /** The device's tooltip shown, as under the pointer. */
  readonly tooltip?: boolean;
  /** The output's own device over the default, or none while it follows it. */
  readonly outputDevice?: Device | null;
  /** The mic's own device over the default, or none while it follows it. */
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
  /** The apps recording: none when none is given, as a sample about a recording app adds one. */
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
  /** The top Device card's Select drawn open, its choices listed. */
  readonly deviceOpen?: boolean;
  /** The Codec Select drawn open, its descriptions under the codecs. */
  readonly codecOpen?: boolean;
  /** For a move's sample: the tab, and the headset's mode, a pick comes from. */
  readonly tabFrom?: number;
  readonly modeFrom?: Mode;
}

/** The default device and each side's, worked out from `options`. */
interface Devices {
  readonly machine: readonly Device[];
  /** Whether any device plays: with none, the default and the output's Select say so. */
  readonly outputs: boolean;
  /** The default's choice, as the top Device card's Select names it. */
  readonly choice: DefaultChoice;
  /** The default output. */
  readonly default: Device;
  /** The default input: the choice's mic, or none when the default has no mic. */
  readonly defaultInput: Device | null;
  /** The output's override, if any. */
  readonly output: Device | null;
  /** The mic's override, if any. */
  readonly input: Device | null;
  /** The mode, for a Bluetooth default; none for any other device. */
  readonly mode: Mode | null;
}

function devicesOf(options: AudioOptions): Devices {
  const machine = options.machine ?? DEVICES;
  const chosen = options.device ?? SPEAKERS;
  const choice = defaultChoices(machine).find((each) => each.output === chosen) ?? alone(chosen);
  const outputs = machine.some((each) => each.plays);
  return {
    machine,
    outputs,
    choice,
    default: chosen,
    // With nothing to play, the mic is the machine's own, there being no default to bring one.
    defaultInput: outputs ? choice.input : (machine.find((each) => each.records) ?? null),
    output: options.outputDevice ?? null,
    input: options.inputDevice ?? null,
    mode: chosen.link === "bluetooth" ? (options.mode ?? "calls") : null,
  };
}

/** What the subtitle reads of the panel's state. */
export interface Heard {
  /** Whether any device on the machine plays. */
  readonly outputs: boolean;
  readonly outputMuted: boolean;
  /** The mic muted by hand; a headset in Music mode is its own case. */
  readonly micMuted: boolean;
  /** The first app listed as recording, and as playing. */
  readonly recording: App | undefined;
  readonly playing: App | undefined;
  /** The default's name, as its Select names it, and whether it brings a mic. */
  readonly device: string;
  readonly deviceMic: boolean;
  readonly mode: Mode | null;
  /** The output's level, 0 to `BOOST`. */
  readonly level: number;
  /** Whether the output, the mic or an app is on a device of its own, off its side's. */
  readonly moved: boolean;
  /** Balance, 0 left to 1 right; centred as `sideOf` rounds it. */
  readonly balance: number;
  readonly noise: boolean;
}

/** One case of the subtitle: what it says of the state, when it holds, and the line. */
export interface Subtitle {
  readonly when: string;
  readonly holds: (heard: Heard) => boolean;
  readonly line: (heard: Heard) => string;
}

/** Whether `at` is off centre, as its words would say: Left or Right, not Centre. */
function offCentre(at: number): boolean {
  return Math.round(Math.abs(at - 0.5) * 200) !== 0;
}

/**
 * The panel's subtitle, by its state: the first case that holds wins, so the order is the rule. The
 * last always holds, so the line is never empty and nothing under it moves.
 */
export const SUBTITLES: readonly Subtitle[] = [
  { when: "No output device at all", holds: (h) => !h.outputs, line: () => "Nowhere to send the noise" },
  { when: "Output and mic both muted", holds: (h) => h.outputMuted && h.micMuted, line: () => "Full monk mode" },
  { when: "Output muted", holds: (h) => h.outputMuted, line: () => "Shh, the speakers are napping" },
  { when: "Mic muted while an app records", holds: (h) => h.micMuted && h.recording !== undefined, line: (h) => `${h.recording!.name} sees lips, hears nothing` },
  { when: "An app is recording", holds: (h) => h.recording !== undefined, line: (h) => `${h.recording!.name} is all ears` },
  { when: "Mic muted", holds: (h) => h.micMuted, line: () => "Your secrets are safe" },
  { when: "The default device has no mic", holds: (h) => !h.deviceMic, line: (h) => `${h.device} can't hear a thing` },
  { when: "Headset in Music mode", holds: (h) => h.mode === "music", line: () => "All ears, no mouth" },
  { when: "Headset in Calls mode", holds: (h) => h.mode === "calls", line: () => "Ring ring, ready to talk" },
  { when: "Boosted past 100%", holds: (h) => h.level > 1, line: () => "Neighbours have been notified" },
  { when: "Volume at 0", holds: (h) => h.level === 0, line: () => "Technically on" },
  { when: "Something off the default", holds: (h) => h.moved, line: () => "One ear here, one mouth there" },
  { when: "Balance off centre", holds: (h) => offCentre(h.balance), line: (h) => `Leaning ${h.balance < 0.5 ? "left" : "right"} a little` },
  { when: "Something is playing", holds: (h) => h.playing !== undefined, line: (h) => `${h.playing!.name} has the floor` },
  { when: "Noise suppression on", holds: (h) => h.noise, line: () => "The dog is now on mute" },
  { when: "Nothing playing, all fine", holds: () => true, line: () => "Loud and clear" },
];

/** What the subtitle reads of `options`. */
function heard(options: AudioOptions, devices: Devices): Heard {
  return {
    outputs: devices.outputs,
    outputMuted: options.outputMuted ?? false,
    micMuted: options.inputMuted ?? false,
    recording: (options.recording ?? [])[0],
    playing: (options.playing ?? PLAYING)[0],
    device: devices.choice.name,
    deviceMic: devices.defaultInput !== null,
    mode: devices.mode,
    level: options.output ?? LEVEL,
    moved:
      (devices.output !== null && devices.output !== devices.default) ||
      (devices.input !== null && devices.input !== devices.defaultInput) ||
      (options.playing ?? PLAYING).some((app) => app.device !== undefined && app.device !== (devices.output ?? devices.default)) ||
      (options.recording ?? []).some((app) => app.device !== undefined && app.device !== (devices.input ?? devices.defaultInput)),
    balance: options.balance ?? 0.5,
    noise: options.noise ?? true,
  };
}

/** The case of `SUBTITLES` that picks the subtitle for `options`: the first that holds. */
export function subtitleCase(options: AudioOptions): Subtitle {
  const state = heard(options, devicesOf(options));
  return SUBTITLES.find((each) => each.holds(state))!;
}

/** The panel's subtitle for `options`. */
export function subtitleOf(options: AudioOptions): string {
  return subtitleCase(options).line(heard(options, devicesOf(options)));
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

/** The output's level when none is given. */
const LEVEL = 0.62;

/** The output's slider: compact, boosting to 150% and holding at 100%, its icon muting it. */
function outputSlider(options: AudioOptions, told?: (change: AudioOptions) => void): HTMLElement {
  const level = options.output ?? LEVEL;
  return slider({
    icon: volumeIcon(Math.min(1, level)),
    size: "Compact",
    value: level / BOOST,
    sticky: 1 / BOOST,
    number: (at) => PERCENT.format(at * BOOST),
    widest: "150%",
    toggle: { icon: "volume_off", words: "Muted", label: "Mute", toggledLabel: "Unmute", on: options.outputMuted ?? false, ...(told ? { changed: (on: boolean) => told({ outputMuted: on }) } : {}) },
    ...(told ? { changed: (value: number) => told({ output: value * BOOST }) } : {}),
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

/** What a headset's mode means, each tooltip's body under the mode's name. */
export const TIPS: Record<Mode, string> = {
  calls: "Your mic works; music sounds flatter.",
  music: "The best sound; the mic is off.",
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

/** What each codec is good for, under its name in the Codec Select's list. */
export const CODEC_WORDS: Readonly<Record<string, string>> = {
  LDAC: "The best sound; drops out sooner far from the PC",
  AAC: "Very good sound, steady",
  SBC: "Plain sound; every headset has it",
  mSBC: "Clearer voice on calls",
  CVSD: "Basic voice; every headset has it",
};

/**
 * The symbol at the end of a default's row that brings no mic. Only the top Device card's Select
 * shows it, where a pick sets the mic too; a side's and an app's choose where sound goes alone.
 */
const NO_MIC = "mic_off";

/** A device's name and symbol in a Select's list. */
function names(devices: readonly Device[]): { readonly options: string[]; readonly icons: string[] } {
  return {
    options: devices.map((each) => each.name),
    icons: devices.map((each) => LINK_ICONS[each.link]),
  };
}

/** What a device's tooltip names: its link, and a headset's codec. */
export function linkOf(device: Device, codec?: string): string {
  return codec ? `${LINK_WORDS[device.link]} · ${codec}` : LINK_WORDS[device.link];
}

/**
 * The top Device card's Select: the default's choices, the built-in set as one, the default
 * picked, its tooltip over it. A choice with no mic ends in the crossed-out mic.
 */
function defaultPicker(devices: Devices, codec: string | undefined, shown: boolean, open = false): HTMLElement {
  if (!devices.outputs) return select({ options: [], placeholder: NO_OUTPUT });
  const choices = defaultChoices(devices.machine);
  const listed = choices.some((each) => each.output === devices.default) ? choices : [devices.choice, ...choices];
  const pick = select({
    options: listed.map((each) => each.name),
    icons: listed.map((each) => LINK_ICONS[each.link]),
    trailing: listed.map((each) => (each.input ? undefined : NO_MIC)),
    picked: listed.findIndex((each) => each.output === devices.default),
    ...(shown ? { pinned: "lit" as const } : {}),
    ...(open ? { open: true } : {}),
  });
  const anchor = floatingTooltip(pick, tooltip(linkOf(devices.default, codec), ""), { side: "above", shown });
  return anchor;
}

/** What the default's Select and the output's say on a machine with nothing that plays. */
const NO_OUTPUT = "No output device";

/** A Bluetooth headset's codecs in its mode, each with what it is good for, `codec` picked. */
function codecPicker(device: Device, mode: Mode, codec: string | undefined, open = false): HTMLElement {
  const codecs = device.codecs?.[mode] ?? [];
  const pick = select({
    options: codecs,
    descriptions: codecs.map((each) => CODEC_WORDS[each] ?? ""),
    picked: Math.max(0, codecs.indexOf(codec ?? codecs[0] ?? "")),
    ...(open ? { open: true } : {}),
  });
  return stack("s8", "start", filling(label("Codec")), filling(pick));
}

/**
 * A side's Device card's Select: "Default · <device>" first, with the default's symbol, then every
 * other device of that side, the built-in speakers and mic apart. A default that has nothing for the
 * side leaves it unpicked, saying so.
 */
function sidePicker(devices: Devices, input: boolean, told?: (change: AudioOptions) => void): HTMLElement {
  const fits = (each: Device) => (input ? each.records : each.plays);
  const chosen = input ? devices.input : devices.output;
  const following = input ? devices.defaultInput : devices.outputs ? devices.default : null;
  const others = devices.machine.filter((each) => fits(each) && each !== following);
  const all = chosen && !others.includes(chosen) && chosen !== following ? [chosen, ...others] : others;
  /** A pick moves the side to its device, or back to the default. */
  const moved = (device: Device | null) => told?.(input ? { inputDevice: device } : { outputDevice: device });
  if (!following) {
    return select({
      ...names(all),
      picked: chosen ? all.indexOf(chosen) : -1,
      placeholder: input ? `No mic on ${devices.choice.name}` : NO_OUTPUT,
      ...(told ? { changed: (index: number) => moved(all[index] ?? null) } : {}),
    });
  }
  const listed = names(all);
  return select({
    options: [`${FOLLOWS} · ${following.name}`, ...listed.options],
    icons: [LINK_ICONS[following.link], ...listed.icons],
    picked: chosen && chosen !== following ? 1 + all.indexOf(chosen) : 0,
    ...(told ? { changed: (index: number) => moved(index === 0 ? null : (all[index - 1] ?? null)) } : {}),
  });
}

/**
 * An app's device, an inline Select under its name: "Default" while it follows its side, then
 * every device of that side; a pick moves the app there.
 */
function appPicker(app: App, side: readonly Device[], following: Device, state?: "lit" | "open", moved?: (device: Device | undefined) => void): HTMLElement {
  const all = app.device && !side.includes(app.device) ? [app.device, ...side] : side;
  const listed = names(all);
  return select({
    look: "inline",
    options: [FOLLOWS, ...listed.options],
    icons: [LINK_ICONS[following.link], ...listed.icons],
    picked: app.device ? 1 + all.indexOf(app.device) : 0,
    ...(state === "lit" ? { pinned: "lit" as const } : state === "open" ? { open: true, up: true } : {}),
    ...(moved ? { changed: (index: number) => moved(index === 0 ? undefined : all[index - 1]) } : {}),
  });
}

/** `app` moved to `device`, or following its side's again with none. */
function movedTo(app: App, device: Device | undefined): App {
  const { device: _left, ...rest } = app;
  return device ? { ...rest, device } : rest;
}

/** One app's row: its tile, its name over its device centred on it, then its slider across, its icon muting it. */
function appRow(app: App, side: readonly Device[], following: Device, input: boolean, state?: "lit" | "open", moved?: (device: Device | undefined) => void): HTMLElement {
  const tile = iconTile({ glyph: input ? "mic" : "volume_up", size: "Md", picture: app.picture });
  const words = stack(0, "start", filling(clamped(app.name, "body_strong")), appPicker(app, side, following, state, moved));
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
function appsCard(apps: readonly App[], devices: Devices, input: boolean, state?: "lit" | "open", told?: (change: AudioOptions) => void): HTMLElement {
  const side = devices.machine.filter((each) => (input ? each.records : each.plays));
  const following = (input ? (devices.input ?? devices.defaultInput) : devices.output) ?? devices.default;
  // The apps as they are moved, so each pick keeps the ones before it.
  let now = apps;
  const mover = (at: number) => (device: Device | undefined) => {
    now = now.map((each, index) => (index === at ? movedTo(each, device) : each));
    told?.(input ? { recording: now } : { playing: now });
  };
  const rows =
    apps.length === 0
      ? [text(input ? NO_APPS.input : NO_APPS.output, "body", "on_surface_variant")]
      : apps.map((app, index) => appRow(app, side, following, input, index === 0 ? state : undefined, told ? mover(index) : undefined));
  const column = stack("s16", "start", ...rows.map(filling));
  return cardOf("Apps", column);
}

/** The Output tab: its Device card (the output's own device, balance) and the Apps card. */
function outputTab(options: AudioOptions, devices: Devices, told?: (change: AudioOptions) => void): HTMLElement {
  const knob = slider({ icon: "balance", size: "Compact", knob: true, value: options.balance ?? 0.5, sticky: 0.5, number: sideOf, ...(told ? { changed: (balance: number) => told({ balance }) } : {}) });
  return stack("s12", "start", cardOf("Device", sidePicker(devices, false, told), knob), filling(appsCard(options.playing ?? PLAYING, devices, false, options.appPicker, told)));
}

/** The Input tab: its Device card (the mic's own device) and the Apps card. */
function inputTab(options: AudioOptions, devices: Devices, told?: (change: AudioOptions) => void): HTMLElement {
  return stack("s12", "start", cardOf("Device", sidePicker(devices, true, told)), filling(appsCard(options.recording ?? [], devices, true, options.appPicker, told)));
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
function optionsTab(options: AudioOptions, told?: (change: AudioOptions) => void): HTMLElement {
  const made = list([
    { title: "Noise suppression", subtitle: "Your mic, without the background", toggle: options.noise ?? true },
    { title: "Echo cancellation", subtitle: "For calls on the speakers", toggle: options.echo ?? false },
    { title: "New devices take over", subtitle: "A headset you connect becomes the default", toggle: options.takeover ?? true },
  ]);
  if (told) {
    const keys = ["noise", "echo", "takeover"] as const;
    // A row's own click turns its switch first, so it reads as turned here.
    made.querySelectorAll<HTMLElement>(".k-list-row").forEach((row, index) =>
      row.addEventListener("click", () => told({ [keys[index]!]: row.getAttribute("aria-checked") === "true" })),
    );
  }
  return made;
}

/** The picked tab's content: Input, while the mic is a headset's own in Music mode, is its empty state. */
function tabContent(tab: number, options: AudioOptions, devices: Devices, toCalls?: () => void, told?: (change: AudioOptions) => void): HTMLElement {
  if (tab === 1 && micInMusic(devices)) return musicEmpty(toCalls);
  return tab === 0 ? outputTab(options, devices, told) : tab === 1 ? inputTab(options, devices, told) : optionsTab(options, told);
}

/**
 * Calls and Music, stretched with their symbols, each saying what it means in its tooltip; disabled,
 * with nothing picked and no tooltips, for a device that isn't Bluetooth.
 */
function modeTabs(mode: Mode | null, onPick?: (mode: Mode) => void, from?: Mode, tip?: Mode): HTMLElement {
  const tips = mode !== null;
  return tabs({
    choices: [
      { label: "Calls", icon: "call", ...(tips ? { tip: TIPS.calls } : {}) },
      { label: "Music", icon: "music_note", ...(tips ? { tip: TIPS.music } : {}) },
    ],
    picked: mode === null ? null : mode === "music" ? 1 : 0,
    ...(mode === null ? { disabled: true } : {}),
    ...(tips && tip ? { tipShown: tip === "calls" ? 0 : 1 } : {}),
    ...(from ? { from: from === "calls" ? 0 : 1 } : {}),
    ...(onPick ? { onPick: (index: number) => onPick(index === 0 ? "calls" : "music") } : {}),
  });
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
  /** The panel's state changes by hand, live: the subtitle follows it. */
  const told = live ? (change: AudioOptions) => {
    state = { ...state, ...change };
    devices = devicesOf(state);
    page.subtitle(subtitleOf(state));
  } : undefined;
  const output = outputSlider(state, told);
  const inputMuted = () => (state.inputMuted ?? false) || micInMusic(devices);
  const meterOf = liveMeter(state.meter?.level ?? 0.46, state.meter?.peak ?? 0.68);
  if (inputMuted()) meterOf.mute(true);
  /** The mic's slider turned: its meter, and the subtitle, which a headset in Music mode leaves be. */
  const micTurned = (on: boolean) => {
    meterOf.mute(on);
    if (!micInMusic(devices)) told?.({ inputMuted: on });
  };
  let mic = micSlider(state.input ?? 0.8, inputMuted(), live ? micTurned : undefined);
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

  const modes = modeTabs(devices.mode, live ? switchMode : undefined, options.modeFrom, options.modeTip);
  let codec = devices.mode ? codecPicker(devices.default, devices.mode, state.codec, options.codecOpen) : null;
  let device = defaultPicker(devices, devices.mode ? (state.codec ?? devices.default.codecs?.[devices.mode][0]) : undefined, state.tooltip ?? false, options.deviceOpen);

  function show(): void {
    room.replaceChildren(tabContent(tab, state, devices, live ? () => modes.querySelector<HTMLElement>(".k-tab")?.click() : undefined, told));
    if (live) fadeIn(room);
  }

  /** The headset's mode changes: the mic, its meter, the codecs, Input, and Input's content if it shows.
      Each new part fills the width as the one it replaces did, or it shrinks to its content. */
  function switchMode(mode: Mode): void {
    // The codec picked was the other mode's: the new mode starts on its best.
    const { codec: _left, ...rest } = state;
    state = { ...rest, mode };
    devices = devicesOf(state);
    const named = filling(defaultPicker(devices, devices.default.codecs?.[mode][0], false));
    device.replaceWith(named);
    device = named;
    const codecs = filling(codecPicker(devices.default, mode, undefined));
    codec?.replaceWith(codecs);
    codec = codecs;
    const fresh = filling(micSlider(state.input ?? 0.8, inputMuted(), micTurned));
    mic.replaceWith(fresh);
    mic = fresh;
    meterOf.mute(inputMuted());
    setTabDisabled(picker, 1, micInMusic(devices));
    if (tab === 1) show();
    page.subtitle(subtitleOf(state));
  }

  show();

  const top = cardOf("Device", device, modes, ...(codec ? [codec] : []));
  const body = stack(
    "s16",
    "start",
    filling(stack("s8", "start", filling(output), filling(mic), filling(meterOf.bar))),
    filling(top),
    filling(stack("s12", "start", filling(picker), filling(room))),
  );
  body.classList.add("audio-body");
  const page = panelPage({ title: TITLE, subtitle: subtitleOf(state), body, ...(options.down !== undefined ? { down: options.down } : {}) });
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
