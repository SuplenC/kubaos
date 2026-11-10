// The control center's parts as the build draws them (`config/quickshell/modules/controlcenter/`),
// from the library alone, with the shots' stand-in data (`test/shots/fakes/qs/services`): the
// header, the quick toggles, the capture card, the sliders, the Wi-Fi and Bluetooth wells with
// their connection form, the outputs and the keyboard layouts. The control center's pages build
// their samples from these; each part is drawn at its build's sizes, by token.

import "./control-center.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import type { Pinned } from "../../lib/control.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { field } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { keyHints } from "../../lib/keycap.ts";
import { container, fill, filling, gap, grid, header, row, stack } from "../../lib/layout.ts";
import { type ListRowSpec, listRow, type RowPinned, scrolled } from "../../lib/list.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { slider } from "../../lib/slider.ts";
import { switchControl } from "../../lib/switch.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { tile, type TileOptions } from "../../lib/tile.ts";
import { tooltip } from "../../lib/tooltip.ts";
import { barButtons } from "../templates/panels.ts";

// The header: the shots' frozen clock, Tuesday 27 October 2026, 21:47, and the battery.
export const ACCENT = "火曜日";
export const TIME = "21:47";
export const DATE = "Tuesday, 27 October · 64% 3h 12m left";

export const TAB_NAMES = ["Wi-Fi", "Bluetooth", "Output", "Keyboard"] as const;
export type TabName = (typeof TAB_NAMES)[number];

/** The header's row: the weekday's kanji over the time and the date, the two Tonal buttons at its end. */
export function ccHeader(options: { date?: string; lit?: Pinned } = {}): HTMLElement {
  const words = header({ accent: ACCENT, title: TIME, subtitle: options.date ?? DATE });
  const wall = iconButton({ icon: "wallpaper", label: "Wallpaper", kind: "Tonal", ...(options.lit ? { pinned: options.lit } : {}) });
  const power = iconButton({ icon: "power_settings_new", label: "Power", kind: "Tonal" });
  return row("s8", "centre", filling(words), wall, power);
}

// The quick toggles.

export interface Toggles {
  readonly wifi?: Partial<TileOptions>;
  readonly bluetooth?: Partial<TileOptions>;
  readonly dnd?: Partial<TileOptions>;
  readonly motion?: Partial<TileOptions>;
  readonly keyboard?: Partial<TileOptions>;
  readonly proxy?: Partial<TileOptions> | false;
  /** The Proxy tile's detail open under the tiles. */
  readonly detail?: ProxyDetail | false;
}

export const TILES: Record<"wifi" | "bluetooth" | "dnd" | "motion" | "keyboard" | "proxy", TileOptions> = {
  wifi: { glyph: "signal_wifi_4_bar", title: "Wi-Fi", subtitle: "Shinjuku-5G", on: true },
  bluetooth: { glyph: "bluetooth_connected", title: "Bluetooth", subtitle: "2 connected", on: true },
  dnd: { glyph: "notifications", title: "Do not disturb", subtitle: "Off" },
  motion: { glyph: "motion_photos_auto", title: "Wallpaper motion", subtitle: "On AC, paused", on: true },
  keyboard: { glyph: "keyboard", title: "Keyboard", subtitle: "US · 1 of 2", on: true, detail: () => undefined },
  proxy: { glyph: "vpn_lock", title: "Proxy", subtitle: "vpn.example.org:51820", on: true, detail: () => undefined },
};

/** The tiles, two to a row `space.s8` apart; the keyboard tile across a row of its own with no Proxy tile. */
export function toggles(options: Toggles = {}): HTMLElement {
  const make = (name: keyof typeof TILES, patch?: Partial<TileOptions>) => tile({ ...TILES[name], ...patch });
  const proxy = options.proxy === false ? null : make("proxy", options.proxy);
  const paired = [make("wifi", options.wifi), make("bluetooth", options.bluetooth), make("dnd", options.dnd), make("motion", options.motion)];
  const keyboard = make("keyboard", options.keyboard);
  const cells = proxy ? [...paired, keyboard, proxy] : paired;
  const parts: HTMLElement[] = [filling(grid(2, "s8", cells))];
  if (!proxy) parts.push(filling(keyboard));
  if (proxy && options.detail) parts.push(filling(proxyDetail(options.detail)));
  return stack("s8", "start", ...parts);
}

// The Proxy tile's detail.

export interface ProxyDetail {
  readonly openvpn?: boolean;
  readonly local?: boolean;
  readonly state?: string;
  /** The live rows left out: the detail as it first opens, before the first reading. */
  readonly steady?: boolean;
  readonly server?: string;
}

/** A label on the left and its value, in `mono`, on the right. */
function valueRow(label: string, value: string): HTMLElement {
  return row("s12", "centre", text(label, "small", "on_surface_variant"), fill(), clamped(value, "mono"));
}

/** The detail: the state, the VPN, its server, the local network's switch, then the live rows. */
export function proxyDetail(options: ProxyDetail = {}): HTMLElement {
  const type = options.openvpn ? "OpenVPN" : "WireGuard";
  const steady = [
    valueRow("State", options.state ?? "On"),
    valueRow("VPN", `${type}, full tunnel, kill switch`),
    valueRow("Server", options.server ?? (options.openvpn ? "vpn.example.org:1194" : "vpn.example.org:51820")),
  ];
  const local = row("s12", "centre", text("Allow local network", "small", "on_surface_variant"), fill(), switchControl({ on: options.local ?? true }));
  const live = options.steady
    ? []
    : [valueRow("Last received", "12 s ago"), ...(options.openvpn ? [] : [valueRow("Last handshake", "32 s ago")]), valueRow("Moved", "48.2 MB in · 3.1 MB out")];
  return card({ body: [stack("s4", "start", ...[...steady, local, ...live].map(filling))] });
}

// The capture card.

export type Capture = "idle" | "recording" | "replay";

/** Capture: its two lines, then the screenshot, annotate, record and replay buttons, Tonal and sm. */
export function capture(state: Capture = "idle", elapsed = "0:42", pinned?: Pinned): HTMLElement {
  const recording = state === "recording";
  const status = recording ? clamped(`Recording · ${elapsed}`, "mono", "error") : clamped(state === "replay" ? "Replay buffer on" : "Screenshot or record", "small", state === "replay" ? "error" : "on_surface_variant");
  const lines = stack(0, "start", clamped("Capture", "body_strong"), status);
  let first = true;
  const sm = (glyph: string, label: string, kind: "Tonal" | "Danger" | "Filled" = "Tonal") => {
    const pin = first && pinned ? { pinned } : {};
    first = false;
    return iconButton({ icon: glyph, label, kind, size: "Sm", ...pin });
  };
  const buttons = [
    sm("screenshot_region", "Screenshot region"),
    sm("draw", "Screenshot and annotate"),
    recording ? sm("stop", "Stop recording", "Danger") : sm("screen_record", "Record region"),
    ...(recording ? [] : [sm("desktop_windows", "Record screen")]),
    sm("replay_30", state === "replay" ? "Replay buffer off" : "Replay buffer: keep the last 30 s", state === "replay" ? "Filled" : "Tonal"),
  ];
  return card({ body: [row("s4", "centre", filling(lines), ...buttons)] });
}

// The sliders.

export interface Sliders {
  readonly volume?: number;
  readonly muted?: boolean;
  readonly brightness?: number | false;
  readonly power?: number;
  readonly pinned?: Pinned;
}

export const POWER_STOPS = ["Super Saver", "Power Saver", "Balanced", "Performance", "Max Performance"] as const;
export const POWER_ICONS = ["bedtime", "eco", "balance", "bolt", "rocket_launch"] as const;

function volumeIcon(volume: number): string {
  return volume <= 0 ? "volume_off" : volume < 0.34 ? "volume_mute" : volume < 0.67 ? "volume_down" : "volume_up";
}

/** Volume, brightness and the power stops, compact, `space.s8` apart. */
export function sliders(options: Sliders = {}): HTMLElement {
  const volume = options.volume ?? 0.62;
  const power = options.power ?? 2;
  const pinned = options.pinned ? { pinned: options.pinned } : {};
  const parts = [
    slider({ icon: volumeIcon(volume), size: "Compact", value: volume, toggle: { icon: "volume_off", words: "Muted", label: "Mute", toggledLabel: "Unmute", on: options.muted ?? false }, ...pinned }),
    ...(options.brightness === false ? [] : [slider({ icon: "brightness_medium", size: "Compact", value: options.brightness ?? 0.7 })]),
    slider({ icon: POWER_ICONS[power]!, size: "Compact", stops: POWER_STOPS.length, index: power, names: POWER_STOPS }),
  ];
  return stack("s8", "start", ...parts.map(filling));
}

// The radio wells.

/** A row of a radio tab's well, as `RadioTab.qml` lists them. */
export type WellRow =
  | ({ readonly kind: "row" } & ListRowSpec)
  | { readonly kind: "label"; readonly title: string }
  | { readonly kind: "skeleton"; readonly share: number }
  | { readonly kind: "note"; readonly title: string; readonly failed?: boolean }
  | { readonly kind: "disclosure"; readonly title: string; readonly open: boolean };

export interface WellOptions {
  /** The well's height, a length's CSS. */
  readonly height?: string;
  /** The row the keyboard is on, by index among the rows. */
  readonly current?: number;
  readonly pinned?: RowPinned;
  readonly focused?: boolean;
}

function wellRow(each: WellRow, index: number, options: WellOptions): HTMLElement {
  switch (each.kind) {
    case "row": {
      const { kind: _kind, ...spec } = each;
      const current = index === options.current;
      return listRow(spec as ListRowSpec, current, current ? options.pinned : undefined);
    }
    case "label": {
      const label = text(each.title, "label", "on_surface_variant");
      label.classList.add("cc-label");
      label.toggleAttribute("data-after", index > 0);
      return label;
    }
    case "skeleton":
      return listRow({ title: "", icon: "bluetooth", tile: true, loading: true, share: each.share });
    case "note": {
      const line = document.createElement("span");
      line.className = "cc-line";
      line.append(clamped(each.title, "body", each.failed ? "error" : "on_surface_variant"));
      return line;
    }
    case "disclosure": {
      const line = document.createElement("span");
      line.className = "cc-line";
      const current = index === options.current;
      const toggle = button({ label: each.open ? "Hide" : "Show", kind: "Text", size: "Sm", icon: "expand_more", trailing: true, ...(current && options.pinned ? { pinned: "lit" as const } : {}) });
      line.append(clamped(each.title, "body", "on_surface_variant"), toggle);
      return line;
    }
  }
}

/** The rows `space.s4` apart in a scrolling well, as the radio tabs' `LiveList` holds them. */
export function radioWell(rows: readonly WellRow[], options: WellOptions = {}): HTMLElement {
  const column = stack("s4", "start", ...rows.map((each, index) => filling(wellRow(each, index, options))));
  column.classList.add("k-list-rows");
  const { box } = scrolled("ScrollingList", column, { height: options.height ?? "100%" });
  if (options.focused) box.querySelector<HTMLElement>(":scope > .k-ring")?.setAttribute("data-pinned", "lit");
  return box;
}

/** The well holding something else in its place: the connection form, or an empty state. */
export function wellOf(content: HTMLElement, height = "100%"): HTMLElement {
  return scrolled("ScrollingList", content, { height }).box;
}

// Wi-Fi.

const lock = "lock";

export const WIFI_KNOWN: readonly ListRowSpec[] = [
  { title: "Shinjuku-5G", subtitle: "Connected · 5 GHz", subtitleIcon: lock, icon: "signal_wifi_4_bar", tile: true, accent: true, checked: true },
  { title: "Golden Gai", subtitle: "Saved", subtitleIcon: lock, icon: "network_wifi_3_bar", tile: true, checked: false },
];

export const WIFI_NEARBY: readonly ListRowSpec[] = [
  { title: "7SPOT", subtitle: "Open", icon: "network_wifi_2_bar", tile: true, checked: false },
  { title: "Omoide Yokocho", subtitle: "Secured", subtitleIcon: lock, icon: "network_wifi_2_bar", tile: true, checked: false },
  { title: "Kabukicho Free Wi-Fi", subtitle: "Open", icon: "network_wifi_1_bar", tile: true, checked: false },
];

export type WifiState = "rest" | "scanning" | "none" | "off" | "no-adapter";

const asRow = (spec: ListRowSpec): WellRow => ({ kind: "row", ...spec });

/** The Wi-Fi tab's rows: Known, then Nearby, or a scan's skeletons, or a note. */
export function wifiRows(state: WifiState = "rest", nearby: readonly ListRowSpec[] = WIFI_NEARBY, known: readonly ListRowSpec[] = WIFI_KNOWN): WellRow[] {
  const out: WellRow[] = [];
  const shownKnown = state === "scanning" ? [] : known;
  if (shownKnown.length > 0) out.push({ kind: "label", title: "Known" }, ...shownKnown.map(asRow));
  out.push({ kind: "label", title: "Nearby" });
  if (state === "scanning") out.push(...[0.7, 0.45, 0.7].map((share) => ({ kind: "skeleton" as const, share })));
  else if (state === "none" || nearby.length === 0) out.push({ kind: "note", title: shownKnown.length === 0 ? "No networks in range" : "No other networks in range" });
  else out.push(...nearby.map(asRow));
  return out;
}

// Bluetooth.

export const BT_KNOWN: readonly ListRowSpec[] = [
  { title: "MX Master 3S", subtitle: "Connected · 45%", subtitleIcon: "battery_1_bar", icon: "mouse", tile: true, accent: true, checked: true },
  { title: "WH-1000XM4", subtitle: "Connected · 80%", subtitleIcon: "battery_3_bar", icon: "headphones", tile: true, accent: true, checked: true },
  { title: "Keychron K3", subtitle: "Not connected", icon: "keyboard", tile: true, checked: false },
  { title: "Pixel 9", subtitle: "Not connected", icon: "smartphone", tile: true, checked: false },
];

export const BT_NEARBY: readonly ListRowSpec[] = [
  { title: "JBL Flip 6", subtitle: "Very close", icon: "speaker", tile: true, checked: false },
  { title: "MX Keys Mini", subtitle: "Nearby", icon: "keyboard", tile: true, checked: false },
  { title: "Galaxy Watch", subtitle: "Far", icon: "bluetooth", tile: true, checked: false },
];

export const BT_UNNAMED: readonly ListRowSpec[] = ["4C:2F:9A:01:BE:77", "D8:3A:DD:5E:10:C2", "F1:07:63:A4:29:8B"].map((address) => ({ title: address, subtitle: "Not paired", icon: "bluetooth", tile: true, checked: false }));

export type BtState = "rest" | "scanning" | "none" | "failed";

/** The Bluetooth tab's rows: Known, Nearby, then the line over the drawer of unnamed devices. */
export function btRows(state: BtState = "rest", options: { unnamed?: "closed" | "open" | "none"; nearby?: readonly ListRowSpec[]; known?: readonly ListRowSpec[] } = {}): WellRow[] {
  const known = options.known ?? BT_KNOWN;
  const nearby = options.nearby ?? BT_NEARBY;
  const out: WellRow[] = [];
  if (known.length > 0) out.push({ kind: "label", title: "Known" }, ...known.map(asRow));
  out.push({ kind: "label", title: "Nearby" });
  if (state === "scanning") out.push(...[0.7, 0.45].map((share) => ({ kind: "skeleton" as const, share })));
  else if (state === "failed") out.push({ kind: "note", title: "Discovery failed: the adapter is busy", failed: true });
  else if (state === "none" || nearby.length === 0) out.push({ kind: "note", title: "Nothing in pairing mode nearby" });
  else out.push(...nearby.map(asRow));
  const unnamed = options.unnamed ?? "closed";
  if (unnamed !== "none") {
    out.push({ kind: "disclosure", title: `${BT_UNNAMED.length} unnamed devices`, open: unnamed === "open" });
    if (unnamed === "open") out.push(...BT_UNNAMED.map(asRow));
  }
  return out;
}

// The connection form.

export interface Form {
  readonly icon: string;
  readonly title: string;
  readonly subtitle: string;
  readonly subtitleIcon?: string;
  /** What it asks for: a field, a code shown to type on the device, or nothing. */
  readonly ask?: "field" | "code";
  readonly placeholder?: string;
  readonly secret?: boolean;
  readonly value?: string;
  readonly code?: string;
  readonly status?: string;
  readonly failed?: boolean;
  readonly action: string;
  readonly ready?: boolean;
  /** Where the keyboard is: the field, Cancel or the action. */
  readonly focus?: "field" | "cancel" | "action";
}

export const WIFI_FORM: Form = { icon: "network_wifi_2_bar", title: "Omoide Yokocho", subtitle: "Secured · WPA2", subtitleIcon: lock, ask: "field", placeholder: "Password", secret: true, action: "Connect", ready: false, focus: "field" };
export const BT_FORM: Form = { icon: "speaker", title: "JBL Flip 6", subtitle: "Not paired · Very close", action: "Pair", focus: "action" };

/** The form: the item on a large tile, its name and line, what it asks, the status, Cancel and the action. */
export function connectForm(form: Form): HTMLElement {
  const tileOf = iconTile({ glyph: form.icon, size: "Lg", accent: true });
  const name = clamped(form.title, "headline");
  const subtitle = row("s4", "centre", ...(form.subtitleIcon ? [icon(form.subtitleIcon, "var(--type-small-icon)", "var(--color-on_surface_variant)")] : []), clamped(form.subtitle, "small", "on_surface_variant"));
  const asks: HTMLElement[] = [];
  if (form.ask === "field") {
    const pinned = form.focus === "field" ? { pinned: "lit" as const } : {};
    asks.push(gap("s16"), filling(field({ icon: form.secret ? "key" : "passkey", secret: form.secret ?? false, placeholder: form.placeholder ?? "", value: form.value ?? "", state: form.failed ? "Invalid" : "Normal", ...pinned })));
  } else if (form.ask === "code") asks.push(gap("s16"), text(form.code ?? "", "headline", "primary"));
  const status = clamped(form.status ?? "", "small", form.failed ? "error" : "on_surface_variant");
  status.classList.add("cc-status");
  const cancel = button({ label: "Cancel", kind: "Text", ...(form.focus === "cancel" ? { pinned: "lit" as const } : {}) });
  const act = button({ label: form.action, kind: "Filled", disabled: form.ready === false, ...(form.focus === "action" && form.ready !== false ? { pinned: "lit" as const } : {}) });
  const actions = row("s8", "centre", cancel, fill(), act);
  const column = stack(0, "centre", tileOf, gap("s12"), name, gap("s4"), subtitle, ...asks, gap("s8"), filling(status), gap("s16"), filling(actions));
  const room = document.createElement("span");
  room.className = "cc-form";
  room.append(column);
  return room;
}

// The radios turned off.

export function wifiOff(adapter = true): HTMLElement {
  return emptyState(adapter ? { icon: "wifi_off", title: "Wi-Fi is off", body: "The Wi-Fi tile turns it on." } : { icon: "wifi_off", title: "No Wi-Fi adapter", body: "Nothing to connect with on this machine." });
}

export function btOff(kind: "off" | "blocked" | "none" = "off"): HTMLElement {
  if (kind === "none") return emptyState({ icon: "bluetooth_disabled", title: "No Bluetooth adapter", body: "Nothing to pair with on this machine." });
  if (kind === "blocked") return emptyState({ icon: "bluetooth_disabled", title: "Bluetooth is blocked", body: "Blocked by rfkill (airplane mode). Unblock it: rfkill unblock bluetooth" });
  return emptyState({ icon: "bluetooth_disabled", title: "Bluetooth is off", body: "Turn it on to find devices.", action: button({ label: "Turn on", kind: "Tonal" }) });
}

// Output.

export const OUTPUTS: readonly ListRowSpec[] = [
  { title: "Built-in Audio Analog Stereo", subtitle: "Default output", icon: "speaker", checked: true },
  { title: "WH-1000XM4", icon: "speaker_group", checked: false },
];

/** The outputs in their well, the default checked, `space.s12` above the first and below the last. */
export function outputs(rows: readonly ListRowSpec[] = OUTPUTS, options: WellOptions = {}): HTMLElement {
  const column = stack("s4", "start", ...rows.map((spec, index) => filling(listRow(spec, index === options.current, index === options.current ? options.pinned : undefined))));
  column.classList.add("k-list-rows");
  const { box } = scrolled("ScrollingList", column, { height: options.height ?? "100%" });
  return box;
}

// Keyboard.

export const LAYOUTS: readonly { readonly name: string; readonly spelling: string }[] = [
  { name: "English (US)", spelling: "us" },
  { name: "German", spelling: "de" },
];

export interface KeyboardOptions {
  readonly layouts?: readonly { readonly name: string; readonly spelling: string }[];
  /** The layout typed with. */
  readonly active?: number;
  /** The focused row, the toolbar's. */
  readonly selected?: number;
  readonly accents?: boolean;
  /** The Mac accents tooltip shown. */
  readonly explaining?: boolean;
  /** A row lifted and dragged, and the place it would land. */
  readonly drag?: { readonly from: number; readonly to: number };
  readonly problem?: string;
}

/** The accents chip, with its tooltip over it when `explaining`. */
function accentsChip(on: boolean, explaining: boolean): HTMLElement {
  const made = chip({ label: "Mac accents", icon: "keyboard_option_key", kind: on ? "Selected" : "Neutral", press: "press" });
  if (!explaining) return made;
  const hints = stack("s8", "start", keyHints([["Left Alt+E", "then"], ["E", "types é"]]), keyHints([["Left Alt+U", "then"], ["O", "types ö"]]));
  const tip = tooltip("Mac accents on Left Alt", `${on ? "On" : "Off"}. Type accents as a Mac does: hold Left Alt with the accent's key, then type the letter.`, hints);
  return stack("s8", "start", tip, made);
}

/** The layouts' well: the rows, the default first on its star, and the toolbar at its foot. */
export function keyboardTab(options: KeyboardOptions = {}): HTMLElement {
  const layouts = options.layouts ?? LAYOUTS;
  const active = options.active ?? 0;
  const selected = options.selected ?? -1;
  const rows: HTMLElement[] = [];
  layouts.forEach((layout, index) => {
    const made = listRow({ title: layout.name, subtitle: layout.spelling, icon: index === 0 ? "star_filled" : "keyboard", tile: true, accent: index === 0, checked: index === active }, index === selected || options.drag?.from === index);
    if (options.drag?.from === index) made.classList.add("cc-lifted");
    if (options.drag && options.drag.to === index && options.drag.to < options.drag.from) {
      const line = document.createElement("span");
      line.className = "cc-drop";
      rows.push(line);
    }
    rows.push(filling(made));
  });
  const toolbar = row(
    "s4",
    "centre",
    accentsChip(options.accents ?? true, options.explaining ?? false),
    fill(),
    iconButton({ icon: "arrow_upward", label: "Move up", kind: "Text", size: "Sm", disabled: !(selected > 0) }),
    iconButton({ icon: "arrow_downward", label: "Move down", kind: "Text", size: "Sm", disabled: !(selected >= 0 && selected < layouts.length - 1) }),
    iconButton({ icon: "add", label: "Add a language", kind: "Tonal", size: "Sm", disabled: layouts.length >= 4 }),
    iconButton({ icon: "close", label: "Remove", kind: "Text", size: "Sm", disabled: !(selected >= 0 && layouts.length > 1) }),
  );
  const grow = document.createElement("span");
  grow.className = "cc-grow";
  const column = document.createElement("span");
  column.className = "cc-keys";
  column.append(...rows, grow, filling(toolbar));
  const box = wellOf(column);
  if (!options.problem) return box;
  return stack("s12", "start", filling(box), clamped(options.problem, "body", "error"));
}

export const MATCHES: readonly ListRowSpec[] = [
  { title: "Polish", subtitle: "pl", icon: "keyboard", tile: true, checked: false },
  { title: "Polish (legacy, with dead keys)", subtitle: "pl(legacy)", icon: "keyboard", tile: true, checked: false },
  { title: "Polish (programmer Dvorak)", subtitle: "pl(dvp)", icon: "keyboard", tile: true, checked: false },
];

/** "Add a language" in the well: Back and its title, the search, and its matches. */
export function addView(options: { query?: string; matches?: readonly ListRowSpec[]; empty?: "searching" | "none" | "offline" } = {}): HTMLElement {
  const head = row("s8", "centre", iconButton({ icon: "arrow_back", label: "Back", kind: "Text", size: "Sm" }), clamped("Add a language", "body_strong"));
  const search = field({ icon: "search", placeholder: "Search languages and layouts", value: options.query ?? "pol", pinned: "lit" });
  let below: HTMLElement;
  if (options.empty) {
    const said = {
      searching: { icon: "search", title: "Searching…", body: "The layouts show up here." },
      none: { icon: "search_off", title: "No match", body: `No layout matches “${options.query ?? "xyzzy"}”.` },
      offline: { icon: "error", title: "Nothing to search", body: "The Command menu's search (kotodamad) isn't running." },
    }[options.empty];
    below = emptyState(said);
  } else {
    const matches = options.matches ?? MATCHES;
    below = stack("s4", "start", ...matches.map((spec, index) => filling(listRow(spec, index === 0))));
  }
  const column = document.createElement("span");
  column.className = "cc-keys";
  column.append(filling(head), filling(search), filling(below));
  return wellOf(column);
}

// The panel.

export interface PanelOptions {
  readonly tab?: number;
  /** What the tab's room holds: the tab's own sample, made for it. */
  readonly content?: () => HTMLElement;
  readonly toggles?: Toggles;
  readonly capture?: Capture;
  readonly sliders?: Sliders;
  readonly date?: string;
  readonly headerLit?: Pinned;
  readonly tabsPinned?: Pinned;
  /** The tab a pick comes from, for a move's sample. */
  readonly tabsFrom?: number;
  readonly onPick?: (index: number, room: HTMLElement) => void;
}

/** A tab's default sample. */
export function tabContent(index: number): HTMLElement {
  switch (index) {
    case 0:
      return radioWell(wifiRows());
    case 1:
      return radioWell(btRows());
    case 2:
      return outputs();
    default:
      return keyboardTab();
  }
}

/** The whole panel: the sections in a container, then the tabs and the tab's well in what is left. */
export function controlCenter(options: PanelOptions = {}): HTMLElement {
  const picked = options.tab ?? 0;
  const top = stack("s16", "start", ...[ccHeader({ ...(options.date ? { date: options.date } : {}), ...(options.headerLit ? { lit: options.headerLit } : {}) }), toggles(options.toggles), capture(options.capture), sliders(options.sliders)].map(filling));
  const topBox = container(top);
  topBox.classList.add("cc-top");
  const room = document.createElement("span");
  room.className = "cc-tab";
  room.append((options.content ?? (() => tabContent(picked)))());
  const bar = tabs({
    choices: TAB_NAMES.map((label) => ({ label })),
    picked,
    ...(options.tabsPinned ? { pinned: options.tabsPinned } : {}),
    ...(options.tabsFrom !== undefined ? { from: options.tabsFrom } : {}),
    onPick: (index) => {
      room.replaceChildren(tabContent(index));
      options.onPick?.(index, room);
    },
  });
  const column = document.createElement("span");
  column.className = "cc-tabs-column";
  column.append(filling(bar), room);
  const tabsBox = container(column);
  tabsBox.classList.add("cc-tabs");
  const panel = document.createElement("span");
  panel.className = "cc-panel";
  panel.append(topBox, tabsBox);
  return panel;
}

/** A tab's sample alone, the panel's container wide and `height` tall. */
export function alone(content: HTMLElement, height = "calc(6 * var(--size-row_two_line) + 9 * var(--space-s4))"): HTMLElement {
  const box = document.createElement("span");
  box.className = "cc-alone";
  box.style.setProperty("--cc-height", height);
  box.append(content);
  return box;
}

/** The screen's corner a panel is drawn in: the panel and the frame round it. */
export const CORNER = "calc(var(--size-panel) + var(--space-s64) + var(--space-s8))";
/** The shots' screen height, 1000 logical pixels: `size.app_height` and the rest. */
export const TALL = "calc(var(--size-app_min_height) * 2 + var(--space-s40))";

/** The panel open on the right edge of a screen corner, drawn still. */
export function onScreen(panel: HTMLElement, height = TALL, reveal?: number): HTMLElement {
  const room = screen({ width: CORNER, height, wallpaper, bar: barButtons() });
  room.element.dataset.open = "";
  hang(room, sheet({ content: panel, edge: "right", ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}

/** The well as the form takes the list's place: both in it, the two cross-fading. */
export function swapping(rows: readonly WellRow[], form: Form, formShown: boolean, height = "100%"): HTMLElement {
  const column = stack("s4", "start", ...rows.map((each, index) => filling(wellRow(each, index, {}))));
  column.classList.add("k-list-rows", "cc-list");
  const over = connectForm(form);
  over.classList.add("cc-overlay");
  column.style.opacity = formShown ? "0" : "1";
  over.style.opacity = formShown ? "1" : "0";
  const holder = document.createElement("span");
  holder.className = "cc-swap";
  holder.append(column, over);
  return scrolled("ScrollingList", holder, { height }).box;
}
