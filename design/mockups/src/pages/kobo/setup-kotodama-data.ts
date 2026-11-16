// First boot's Kotodama card's stand-in data: the Items its stand-in menu finds, the search over
// them as Kotodama ranks (its tiers, typos, sums, a `>` command, the web's rows only when the real
// matches leave rows empty), the settings the card offers, and what the others answer in the race.
// The Items are the menu's own titles where it has them (`config/kotodama/kubaos.toml`,
// `crates/kotodama-kubaos/src/toggles.rs`); the rest are a stand-in laptop's apps and projects.

import type { KeyHint } from "../../lib/keycap.ts";
import type { Result } from "../../lib/list-detail.ts";
import type { ListRowSpec } from "../../lib/list.ts";
import type { Run } from "../../lib/text.ts";

/** Which of Kotodama's Extensions an Item comes from, as the card's switches name them. */
export type Finds = "apps" | "actions" | "projects" | "windows" | "clipboard" | "emoji" | "calculator";

/** One Item: its words, symbol, Extension and what running it says. */
export interface MenuItem {
  readonly id: string;
  readonly title: string;
  readonly subtitle: string;
  readonly icon: string;
  readonly finds: Finds;
  /** What the pane calls it: App, Toggle, Project. */
  readonly kind: string;
  readonly keywords: readonly string[];
  /** Its first Action's name, in the pane and the footer. */
  readonly verb: string;
  /** What the Toast says once it ran. */
  readonly said: string;
}

/** The stand-in laptop's Items, in the Daemon's order. */
export const MENU_ITEMS: readonly MenuItem[] = [
  { id: "firefox", title: "Firefox", subtitle: "Web browser", icon: "public", finds: "apps", kind: "App", keywords: ["browser", "web", "internet"], verb: "Open", said: "Firefox opens. For real once first boot is done." },
  { id: "files", title: "Files", subtitle: "Access and organise files", icon: "folder", finds: "actions", kind: "App", keywords: ["nautilus", "folder", "explorer"], verb: "Open", said: "Files opens. Your stuff, no cloud upsell." },
  { id: "ghostty", title: "Ghostty", subtitle: "Terminal", icon: "terminal", finds: "apps", kind: "App", keywords: ["terminal", "shell", "console"], verb: "Open", said: "A terminal. Now we're talking." },
  { id: "zed", title: "Zed", subtitle: "Fast editor, written in Rust", icon: "bolt", finds: "apps", kind: "App", keywords: ["editor", "code"], verb: "Open", said: "Zed opens. Go write some Rust." },
  { id: "spotify", title: "Spotify", subtitle: "Music", icon: "music", finds: "apps", kind: "App", keywords: ["music", "songs"], verb: "Open", said: "Spotify opens. Turn it up." },
  { id: "signal", title: "Signal", subtitle: "Chat", icon: "message_circle", finds: "apps", kind: "App", keywords: ["chat", "messages"], verb: "Open", said: "Signal opens." },
  { id: "wifi", title: "Wi-Fi", subtitle: "On · Home-5G. Enter turns it off", icon: "wifi", finds: "actions", kind: "Toggle", keywords: ["wireless", "network", "internet", "wlan"], verb: "Turn on or off", said: "Wi-Fi off. Bold move. Enter again turns it back on." },
  { id: "wifi-networks", title: "Wi-Fi networks", subtitle: "Pick a network, type its password", icon: "wifi_find", finds: "actions", kind: "Action", keywords: ["wireless", "network", "ssid"], verb: "Open", said: "The networks open in the control center." },
  { id: "bluetooth", title: "Bluetooth", subtitle: "Turn Bluetooth on or off", icon: "bluetooth", finds: "actions", kind: "Toggle", keywords: ["headphones", "pair"], verb: "Turn on or off", said: "Bluetooth on." },
  { id: "volume", title: "Set volume", subtitle: "Set the volume, from 0 to 100%", icon: "volume_up", finds: "actions", kind: "Toggle", keywords: ["sound", "audio", "loud"], verb: "Set", said: "Type the number after it: volume 40." },
  { id: "mute", title: "Mute audio", subtitle: "Mute or unmute the speakers", icon: "volume_off", finds: "actions", kind: "Toggle", keywords: ["sound", "silence", "quiet"], verb: "Turn on or off", said: "Muted. Peace." },
  { id: "brightness", title: "Set brightness", subtitle: "Set the screen's brightness, from 1 to 100%", icon: "brightness_high", finds: "actions", kind: "Toggle", keywords: ["screen", "display", "dim"], verb: "Set", said: "Type the number after it: brightness 60." },
  { id: "dnd", title: "Do not disturb: on", subtitle: "Notifications wait until you turn it off", icon: "notifications_off", finds: "actions", kind: "Action", keywords: ["focus", "quiet", "notifications"], verb: "Run", said: "Do not disturb. Nobody gets through." },
  { id: "wallpaper", title: "Next wallpaper", subtitle: "The next one in the set, its colours with it", icon: "wallpaper", finds: "actions", kind: "Action", keywords: ["background", "theme"], verb: "Run", said: "Next wallpaper. The colours follow it." },
  { id: "screenshot", title: "Screenshot region", subtitle: "Drag a box; it's copied and saved", icon: "screenshot_region", finds: "actions", kind: "Action", keywords: ["capture", "print", "snip"], verb: "Run", said: "Drag a box. Evidence secured." },
  { id: "update", title: "Update KubaOS", subtitle: "When you say so, never on its own", icon: "system_update_alt", finds: "actions", kind: "Action", keywords: ["upgrade", "packages"], verb: "Run", said: "Updating, because you asked." },
  { id: "power", title: "Power menu", subtitle: "Lock, suspend, log out, reboot or shut down", icon: "power_settings_new", finds: "actions", kind: "Action", keywords: ["shutdown", "reboot", "restart", "lock", "logout"], verb: "Open", said: "The power menu opens." },
  { id: "menu-settings", title: "Settings: Menu", subtitle: "The Command menu's own settings and your actions, in Files", icon: "settings", finds: "actions", kind: "Action", keywords: ["kotodama", "config", "preferences"], verb: "Open", said: "Its settings open: this card's, and the rest." },
  { id: "kubaos", title: "kubaos", subtitle: "~/Projects/kubaos", icon: "folder_open", finds: "projects", kind: "Project", keywords: ["repo", "code"], verb: "Open", said: "kubaos opens in its tmux session." },
  { id: "kotodama", title: "kotodama", subtitle: "~/Projects/kotodama", icon: "folder_open", finds: "projects", kind: "Project", keywords: ["repo", "code"], verb: "Open", said: "kotodama opens in its tmux session." },
  { id: "window", title: "Firefox: Issues · kubaos", subtitle: "Workspace 2", icon: "desktop_windows", finds: "windows", kind: "Window", keywords: ["window", "switch"], verb: "Go to", said: "There it is, on workspace 2." },
  { id: "bind", title: "Move window to workspace 3", subtitle: "Super + Shift + 3 does it too", icon: "keyboard_command_key", finds: "windows", kind: "Bind", keywords: ["bind", "shortcut"], verb: "Run", said: "Moved." },
  { id: "clipboard", title: "Clipboard history", subtitle: "What you copied, newest first", icon: "content_paste", finds: "clipboard", kind: "App", keywords: ["paste", "copy"], verb: "Open", said: "Your clipboard, newest first." },
  { id: "emoji", title: "Emoji", subtitle: "Search them by name; Enter types it", icon: "mood", finds: "emoji", kind: "App", keywords: ["smiley", "face"], verb: "Open", said: "The emoji, by name." },
];

/** What the card's settings say, read by the search and the menu. */
export interface MenuSettings {
  key: "super-space" | "alt-space" | "super";
  apps: boolean;
  slides: boolean;
  web: boolean;
  engine: number;
  typos: boolean;
  comma: boolean;
  readonly finds: Record<Exclude<Finds, "apps">, boolean>;
}

export function defaultSettings(): MenuSettings {
  return { key: "super-space", apps: true, slides: true, web: true, engine: 0, typos: true, comma: false, finds: { actions: true, projects: true, windows: true, clipboard: true, emoji: true, calculator: true } };
}

/** The keys that open it: Kotodama has no such setting yet; KubaOS binds Super + Space. */
export const OPENS_WITH = [
  { id: "super-space", keys: "Super+Space", title: "Super + Space", line: "The default" },
  { id: "alt-space", keys: "Alt+Space", title: "Alt + Space", line: "Keyboards without Super" },
  { id: "super", keys: "Super", title: "Super, tapped alone", line: "Like Start. Your call" },
] as const;

/** `[extensions.web-search] engines`, in Kotodama's order. */
export const ENGINES = ["DuckDuckGo", "Google", "Wikipedia"] as const;

/** A Result of the stand-in: what the list shows and the pane says, and what running it says. */
export interface Found {
  readonly result: Result;
  readonly said: string;
  readonly item?: MenuItem;
  readonly answer?: boolean;
}

const fold = (words: string) => words.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

function distance(a: string, b: string): number {
  const d = Array.from({ length: a.length + 1 }, (_, i) => [i, ...Array<number>(b.length).fill(0)]);
  for (let j = 1; j <= b.length; j++) d[0]![j] = j;
  for (let i = 1; i <= a.length; i++)
    for (let j = 1; j <= b.length; j++) {
      d[i]![j] = Math.min(d[i - 1]![j]! + 1, d[i]![j - 1]! + 1, d[i - 1]![j - 1]! + (a[i - 1] === b[j - 1] ? 0 : 1));
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) d[i]![j] = Math.min(d[i]![j]!, d[i - 2]![j - 2]! + 1);
    }
  return d[a.length]![b.length]!;
}

/**
 * How `item` matches `query`, Kotodama's tiers: 0 the whole title; 1 the initials, a word start or
 * a keyword, one tier; 2 inside a word; 3 scattered letters; 4 a near miss, one typo per four
 * letters, never in the first. Null when it doesn't.
 */
function match(item: MenuItem, query: string, typos: boolean): { tier: number; runs: Run[] } | null {
  const title = fold(item.title);
  if (title === query) return { tier: 0, runs: [{ start: 0, end: title.length }] };
  const words = [...title.matchAll(/[a-z0-9]+/g)].map((m) => ({ word: m[0], at: m.index }));
  const parts = query.split(/\s+/).filter(Boolean);
  if (parts.length === 1 && query.length > 1 && words.map((w) => w.word[0]).join("").startsWith(query)) return { tier: 1, runs: words.slice(0, query.length).map((w) => ({ start: w.at, end: w.at + 1 })) };
  let tier = 0;
  const runs: Run[] = [];
  for (const part of parts) {
    const start = words.find((w) => w.word.startsWith(part));
    if (start) {
      tier = Math.max(tier, 1);
      runs.push({ start: start.at, end: start.at + part.length });
      continue;
    }
    if (item.keywords.some((k) => fold(k).startsWith(part))) {
      tier = Math.max(tier, 1);
      continue;
    }
    // Punctuation inside a title is no word break to a person typing it: "wifi" is Wi-Fi.
    if (part.length > 1 && title.replace(/[^a-z0-9]/g, "").startsWith(part)) {
      tier = Math.max(tier, 1);
      const end = [...title].reduce((at, each, index) => (at.count < part.length ? { count: at.count + (/[a-z0-9]/.test(each) ? 1 : 0), end: index + 1 } : at), { count: 0, end: 0 }).end;
      runs.push({ start: 0, end });
      continue;
    }
    const inside = part.length > 1 ? title.indexOf(part) : -1;
    if (inside >= 0) {
      tier = Math.max(tier, 2);
      runs.push({ start: inside, end: inside + part.length });
      continue;
    }
    if (typos && part.length >= 4) {
      const near = [...words.map((w) => w.word), ...item.keywords.map(fold)].some((w) => w[0] === part[0] && distance(w.slice(0, part.length + 1), part) <= Math.floor(part.length / 4));
      if (near) {
        tier = 4;
        continue;
      }
    }
    return null;
  }
  return { tier, runs };
}

/** `count` as the locale writes it, with the decimal mark the settings pick. */
export function figure(count: number, comma: boolean): string {
  return count.toLocaleString(comma ? "de-DE" : "en-GB", { maximumFractionDigits: 6, useGrouping: false });
}

/** A sum as typed, worked out, or null when it isn't one. */
function sum(typed: string, comma: boolean): number | null {
  const expression = typed.replace(/[×x]/g, "*").replace(/÷/g, "/").replace(comma ? /,/g : /(?!)/g, ".");
  if (!/^[\d\s.+\-*/()%]+$/.test(expression) || !/\d/.test(expression) || !/[+\-*/%]/.test(expression.replace(/^\s*-/, ""))) return null;
  try {
    // The expression holds only digits, operators and brackets, checked above.
    const value = Function(`"use strict"; return (${expression});`)() as unknown;
    return typeof value === "number" && Number.isFinite(value) ? value : null;
  } catch {
    return null;
  }
}

function row(title: string, subtitle: string, icon: string, runs: readonly Run[] = []): ListRowSpec {
  return { title, subtitle, icon, tile: true, tall: true, clear: true, tight: true, ...(runs.length ? { matched: runs } : {}) };
}

function found(spec: ListRowSpec, kind: string, verb: string, said: string, more: Partial<Found> = {}): Found {
  const hints: readonly KeyHint[] = [
    ["↵", verb],
    ["⌃K", "Actions"],
  ];
  return { result: { row: spec, kind, actions: [{ title: verb, keys: "↵" }, { title: "Forget", keys: "⌃⇧⌫" }], hints }, said, ...more };
}

/** With nothing typed: the most used, then the apps' way in. */
const EMPTY = ["wifi", "firefox", "files", "ghostty", "kubaos", "screenshot", "power", "menu-settings", "update", "wallpaper", "zed", "spotify", "signal", "emoji"];

/** The Results for `typed`, best first, `rows` of them filled: the web's rows only fill what is left. */
export function search(typed: string, settings: MenuSettings, rows: number): Found[] {
  const shown = (item: MenuItem) => (item.finds === "apps" ? settings.apps : settings.finds[item.finds]);
  const query = fold(typed.trim());
  if (!query) return EMPTY.map((id) => MENU_ITEMS.find((item) => item.id === id)!).filter(shown).map((item) => found(row(item.title, item.subtitle, item.icon), item.kind, item.verb, item.said, { item }));
  if (query.startsWith(">")) {
    const command = typed.trim().slice(1).trim();
    return [found(row(command ? `Run “${command}”` : "Run a command", "In a floating terminal that stays open on its output", "terminal"), "Command", "Run", command ? `Ran “${command}”; the terminal stays open on its output.` : "A command after the >, then Enter.")];
  }
  const out: Found[] = [];
  const value = settings.finds.calculator ? sum(typed.trim(), settings.comma) : null;
  if (value !== null) {
    const answer = figure(value, settings.comma);
    out.push(found(row(answer, `${typed.trim()} · Enter copies it`, "calculate"), "Calculator", "Copy", `Copied ${answer}.`, { answer: true }));
  }
  const volume = /^vol(?:ume)?\s+(\d{1,3})$/.exec(query);
  if (volume && settings.finds.actions) {
    const level = Math.min(100, Number(volume[1]));
    out.push(found(row(`Set volume to ${level}%`, "Set the volume, from 0 to 100%", "volume_up", [{ start: 0, end: 10 }]), "Toggle", "Set", `Volume ${level}%.`));
  }
  const kinds: Record<string, number> = { App: 0, Project: 1 };
  const matches = MENU_ITEMS.filter(shown)
    .map((item) => ({ item, hit: match(item, query, settings.typos) }))
    .filter((each): each is { item: MenuItem; hit: { tier: number; runs: Run[] } } => each.hit !== null)
    .sort((a, b) => a.hit.tier - b.hit.tier || (kinds[a.item.kind] ?? 2) - (kinds[b.item.kind] ?? 2) || a.item.title.length - b.item.title.length);
  for (const { item, hit } of matches) out.push(found(row(item.title, hit.tier === 4 ? `${item.subtitle} · a typo away` : item.subtitle, item.icon, hit.runs), item.kind, item.verb, item.said, { item }));
  if (settings.web && out.length < rows) {
    const order = [ENGINES[settings.engine]!, ...ENGINES.filter((_, at) => at !== settings.engine)];
    for (const engine of order.slice(0, rows - out.length)) out.push(found(row(`Search ${engine} for “${typed.trim()}”`, "Below what is here, never instead of it", "travel_explore"), "Web search", "Open", `${engine} opens in Firefox. Your call, not mine.`));
  }
  return out;
}

// ---------------------------------------------------------------- The race --

/** The question the race asks all three at the same moment. */
export const QUESTION = "wifi";

/** When each of the others' answers lands, from the moment the key is let go, at speed 1. */
export const LANDS = {
  /** Start: the web's rows, its recommendations, the ad rising over them, the weather; then it stops. */
  start: { web: 1000, recommended: 2000, ad: 3000, weather: 3600, done: 5800 },
  /** Spotlight: a website and a definition, then a web search; its indexing never ends. */
  spotlight: { site: 1000, web: 2000, done: 2400 },
} as const;

/** The others' words, the question's where it goes. */
export const OTHERS = {
  start: {
    name: "Start",
    searching: "Searching the web for something on your computer",
    still: "Still searching the web for something on your computer…",
    stopped: (seconds: string) => `${seconds} · an ad`,
    roast: (q: string) => `Start, asked for “${q}”: the web, a game, an ad, the weather. What you wanted is under “Show 1 more”.`,
  },
  spotlight: {
    name: "Spotlight",
    indexing: "Indexing. About 2 hours left. It said that yesterday",
    still: "Still thinking…",
    stopped: (seconds: string) => `${seconds} · a website`,
    roast: "Spotlight found a website and a definition, and it's still indexing.",
  },
  kotodama: {
    name: "Kotodama",
    stopped: "3 ms",
    roast: "Kotodama: the thing itself, first row, before you let go of the key. Nothing left the machine.",
  },
} as const;

/** The ad Start slides over its answers: a cleaner nobody asked for, in every loud colour there is. */
export const AD = {
  tag: "AD",
  sponsor: "Sponsored · Recommended for you",
  headline: "Your PC is 97% SLOWER than your neighbour's!",
  product: "QuickBoost Ultra 365",
  maker: "by Hyperzap Labs · 2,000,000 PCs boosted",
  rating: "4.9 · “Fixed 4,812 problems”",
  action: "FIX MY PC NOW · FREE*",
  fine: "*Then €49.99 a month. Forever. Cancel by fax.",
} as const;
