// The panels the Sheet and PanelPage page shows: the system monitor, the notifications and the
// control center (the twins, in their buttons' order along the bar), and the clock, sized to its
// content on the top edge. Their bodies are stand-ins built from the library's parts; each panel's
// own organisms are its surface's pages. Each is drawn on a screen with its frame, over Kōbō's
// wallpaper, so the glass and its shadow read as they do on the desktop.

import "./panels.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { button } from "../../lib/button.ts";
import { calendar } from "../../lib/calendar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { fill, filling, row, stack } from "../../lib/layout.ts";
import { type ListRowSpec, list } from "../../lib/list.ts";
import { meter } from "../../lib/meter.ts";
import { type PanelPage, type PanelPageOptions, panelPage } from "../../lib/panel-page.ts";
import { paired, pill } from "../../lib/pill.ts";
import { icon } from "../../lib/icon.ts";
import { type Edge, hang, type Screen, screen, sheet, type Twins, twins } from "../../lib/sheet.ts";
import { text } from "../../lib/text.ts";

export const PUNS: Record<"system" | "notifications" | "control" | "clock", Words> = {
  system: { short: "Nothing's on fire. Yet.", long: "Nothing's on fire yet, but that browser is eating memory like it's paid by the gigabyte." },
  notifications: { short: "Things that wanted you. Rude.", long: "Everything that interrupted you while you were busy, kept here so you can ignore it properly." },
  control: { short: "Knobs for breaking things.", long: "Every knob, switch and slider you need to break something, all in one convenient place." },
  clock: { short: "Time flies. You don't.", long: "Time flies when you're compiling Rust, and it crawls when you're waiting on the linker." },
};

export const TITLES: Record<"system" | "notifications" | "control" | "clock", Words> = {
  system: { short: "System", long: "System monitor and every process eating your machine" },
  notifications: { short: "Notifications", long: "Notifications from every app that thinks it matters" },
  control: { short: "Control center", long: "Control center and every setting worth a shortcut" },
  clock: { short: "Monday, 2 November", long: "Monday, the second of November, two thousand and twenty-six" },
};

/** What a body shows: its content, nothing yet, still being found, or what broke. */
export type BodyState = "rest" | "empty" | "loading" | "error";

const PROCESSES: readonly ListRowSpec[] = [
  { title: "firefox", subtitle: "2.1 GB", detail: "18%" },
  { title: "rust-analyzer", subtitle: "1.4 GB", detail: "11%" },
  { title: "cargo", subtitle: "640 MB", detail: "9%" },
  { title: "qs", subtitle: "210 MB", detail: "2%" },
  { title: "Hyprland", subtitle: "180 MB", detail: "1%" },
  { title: "ghostty", subtitle: "120 MB", detail: "1%" },
];

const NOTES: readonly ListRowSpec[] = [
  { title: "Screenshot saved", subtitle: "Pictures · now", icon: "screenshot_region", tile: true },
  { title: "kubaos-0.4.iso downloaded", subtitle: "Firefox · 2 min", icon: "download", tile: true },
  { title: "Battery at 20%", subtitle: "Plug in soon · 5 min", icon: "battery_1_bar", tile: true },
  { title: "Updates installed", subtitle: "Restart to finish · 12 min", icon: "system_update_alt", tile: true },
  { title: "Backup to the NAS done", subtitle: "4 minutes · 1 h", icon: "storage", tile: true },
  { title: "WH-1000XM5 connected", subtitle: "Bluetooth · 2 h", icon: "headphones", tile: true },
  { title: "Stand-up in 10 minutes", subtitle: "Calendar · 3 h", icon: "event", tile: true },
  { title: "Disk nearly full", subtitle: "6 GB left · 5 h", icon: "hard_drive", tile: true },
];

const NETWORKS: readonly ListRowSpec[] = [
  { title: "Hoshi 5G", subtitle: "Connected · WPA3", icon: "wifi", tile: true, checked: true },
  { title: "Hoshi", subtitle: "Secured", icon: "wifi_lock", tile: true },
  { title: "Kissa Guest", subtitle: "Open", icon: "network_wifi_3_bar", tile: true },
  { title: "FRITZ!Box 7590", subtitle: "Secured", icon: "network_wifi_2_bar", tile: true },
  { title: "DIRECT-roku-112", subtitle: "Secured", icon: "network_wifi_1_bar", tile: true },
];

/** A body's state in place of its content. */
function stated(state: BodyState, content: () => HTMLElement, rows: readonly ListRowSpec[], empty: { icon: string; title: string; body: string }, broke: { message: string; hint: string }): HTMLElement {
  switch (state) {
    case "rest":
      return content();
    case "empty":
      return emptyState(empty);
    case "loading":
      return list(rows.slice(0, 4).map((each, index) => ({ ...each, loading: true, share: 0.5 + 0.1 * index })));
    case "error":
      return errorBlock(broke);
  }
}

/** A reading's line: its name, its bar, its value. */
function reading(name: string, share: number, value: string): HTMLElement {
  const line = row("s8", "centre", text(name, "body_strong"), fill(), text(value, "label_number", "on_surface_variant"));
  return stack("s8", "start", filling(line), filling(meter([{ share, color: "primary" }])));
}

export function systemBody(state: BodyState = "rest", count = PROCESSES.length): HTMLElement {
  return stated(
    state,
    () => stack("s16", "start", ...[reading("Processor", 0.34, "34%"), reading("Memory", 0.61, "9.8 of 16 GB"), reading("Disk", 0.82, "410 of 500 GB"), list(PROCESSES.slice(0, count))].map(filling)),
    PROCESSES,
    { icon: "memory", title: "Nothing to watch", body: "No processes worth showing. Suspicious." },
    { message: "kubaos-sysmon isn't running: the monitor needs it.", hint: "Start it again: systemctl --user start kubaos-sysmon." },
  );
}

export function notificationsBody(state: BodyState = "rest", count = NOTES.length): HTMLElement {
  return stated(
    state,
    () => list(NOTES.slice(0, count)),
    NOTES,
    { icon: "notifications", title: "Nothing new", body: "Silence. Enjoy it while it lasts." },
    { message: "The notification daemon stopped answering.", hint: "Restart the shell with Super+Shift+R." },
  );
}

export function controlBody(state: BodyState = "rest", count = NETWORKS.length): HTMLElement {
  return stated(
    state,
    () => list(NETWORKS.slice(0, count)),
    NETWORKS,
    { icon: "wifi_off", title: "No networks", body: "Nothing in range. Very off-grid of you." },
    { message: "Wi-Fi is off at the hardware switch.", hint: "Flip it on, then try again." },
  );
}

function clockBody(): HTMLElement {
  return calendar({ shown: { year: 2026, month: 11 }, today: { year: 2026, month: 11, day: 2 } });
}

/** A panel's icon button in its header band: a Text button, sm, pinned lit when `lit`. */
function action(name: string, label: string, lit = false): HTMLElement {
  return iconButton({ icon: name, label, kind: "Text", size: "Sm", ...(lit ? { pinned: "lit" as const } : {}) });
}

export interface PanelChoice {
  readonly state?: BodyState;
  /** How many of its rows the body holds. */
  readonly count?: number;
  readonly title?: string;
  readonly subtitle?: string;
  readonly down?: number;
  readonly fadePinned?: boolean;
  readonly footer?: boolean;
  /** The header's actions left out: a panel with none. */
  readonly bare?: boolean;
  /** Its first action pinned lit, as with the keyboard on it. */
  readonly lit?: boolean;
}

export function system(choice: PanelChoice = {}): PanelPage {
  return panelPage(options(choice, "system", [action("terminal", "Open btop", choice.lit)], systemBody(choice.state, choice.count)));
}

export function notifications(choice: PanelChoice = {}): PanelPage {
  const footer = choice.footer ?? true;
  return panelPage({
    ...options(choice, "notifications", [action("dark_mode", "Do not disturb", choice.lit), action("clear_all", "Clear all")], notificationsBody(choice.state, choice.count)),
    ...(footer && (choice.state ?? "rest") === "rest" ? { footer: [button({ label: "Mark all read", kind: "Tonal", size: "Sm" })] } : {}),
  });
}

export function controlCenter(choice: PanelChoice = {}): PanelPage {
  return panelPage({
    ...options(choice, "control", [action("wallpaper", "Wallpaper", choice.lit), action("power_settings_new", "Power")], controlBody(choice.state, choice.count)),
    tabs: { choices: [{ label: "Wi-Fi" }, { label: "Bluetooth" }, { label: "Output" }, { label: "Keyboard" }] },
  });
}

export function clock(choice: PanelChoice = {}): PanelPage {
  return panelPage({ ...options(choice, "clock", [action("settings", "Calendar settings")], clockBody()), sized: true });
}

function options(choice: PanelChoice, which: keyof typeof PUNS, actions: HTMLElement[], body: HTMLElement): PanelPageOptions {
  return {
    title: choice.title ?? shown(TITLES[which]),
    subtitle: choice.subtitle ?? shown(PUNS[which]),
    ...(choice.bare ? {} : { actions }),
    body,
    ...(choice.down !== undefined ? { down: choice.down } : {}),
    ...(choice.fadePinned ? { fadePinned: true } : {}),
  };
}

/** A symbol in a bar pill's ink, `icon.s16`, as the bar draws them. */
function glyph(name: string): SVGElement {
  return icon(name, "var(--icon-s16)", "var(--ink)");
}

/** Words in a bar pill's ink, `button_small`, as the bar sets them. */
function words(said: string): HTMLElement {
  const element = text(said, "button_small");
  element.style.color = "var(--ink)";
  return element;
}

/** The twins' buttons along the bar, in their order: the system monitor, the bell, the status. */
export function barButtons(): HTMLElement[] {
  return [
    pill({ content: [paired(glyph("memory"), words("34%"))], lone: false, interactive: true, tooltip: "System" }),
    pill({ content: [glyph("notifications")], lone: true, interactive: true, tooltip: "Notifications" }),
    pill({ content: [glyph("wifi"), glyph("volume_up"), paired(glyph("battery_full"), words("87%"))], lone: false, interactive: true, tooltip: "Control center" }),
  ];
}

/** A screen `width` by `height` (lengths' CSS) with its frame and the twins' buttons on the bar. */
export function desk(width: string, height: string): { screen: Screen; buttons: HTMLElement[] } {
  const buttons = barButtons();
  return { screen: screen({ width, height, wallpaper, bar: buttons }), buttons };
}

/** The room a panel sample is drawn in: the screen's corner, a panel wide and the frame round it. */
export const CORNER_WIDTH = "calc(var(--size-panel) + var(--space-s64) + var(--space-s8))";

/** `page` open on `edge` of a screen `width` by `height`, drawn still, `reveal` of the way open. */
export function onScreen(page: PanelPage | HTMLElement, edge: Edge = "right", width = CORNER_WIDTH, height = "var(--size-app_min_height)", reveal?: number): HTMLElement {
  const { screen: room } = desk(width, height);
  room.element.dataset.open = "";
  const element = page instanceof HTMLElement ? page : page.element;
  hang(room, sheet({ content: element, edge, ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}

/** The three twins on a screen, live: each button opens its panel, and another swaps it in. */
export function liveTwins(width: string, height: string): HTMLElement {
  const { screen: room, buttons } = desk(width, height);
  const pages = [system(), notifications(), controlCenter()];
  const firsts: (() => HTMLElement | null)[] = [
    () => pages[0]!.view,
    () => pages[1]!.view.querySelector<HTMLElement>(".k-list"),
    () => pages[2]!.tabs?.querySelector<HTMLElement>('[aria-selected="true"]') ?? null,
  ];
  for (const each of pages) each.view.tabIndex = -1;
  const set: Twins = twins(
    room,
    pages.map((each, side) => ({ content: each.element, side, land: () => landOn(firsts[side]!) })),
  );
  buttons.forEach((each, index) => each.addEventListener("click", () => set.toggle(index)));
  return room.element;
}

/**
 * The keyboard onto a panel's first thing to do once it is in place: the system monitor's body,
 * the notifications' list, the control center's picked tab. A pointer opened it, so nothing lights.
 */
function landOn(first: () => HTMLElement | null): void {
  requestAnimationFrame(() => first()?.focus({ preventScroll: true, focusVisible: false } as FocusOptions));
}

/** The clock on the top edge, sized to its content, on a screen as wide as it needs. */
export function clockOnScreen(): HTMLElement {
  return onScreen(clock(), "top", "calc(var(--size-panel) * 2)", "calc(var(--size-app_min_height) + var(--space-s64))");
}

/** Two twins in one sheet, the first on its way out over the second: a swap's sample. */
export function swapSample(): HTMLElement {
  const leaving = system().element;
  leaving.dataset.leaving = "";
  const arriving = notifications().element;
  const room = onScreen(arriving);
  room.querySelector(".k-sheet-rider")?.prepend(leaving);
  return room;
}
