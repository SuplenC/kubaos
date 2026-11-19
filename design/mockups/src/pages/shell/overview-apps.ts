// What each sample app looks like inside its window, drawn at the window's true size so the
// Overview's miniature is the real window scaled down, as the build's live window pictures will be.
// A DRAFT part of the Overview's page: the stand-ins for live pictures, not a library part.
//
// Text a reader would make out at a miniature's scale (a page's heading, a mail's subject, an
// album's title) is real text; body text, code and terminal output, which a miniature shrinks
// below a pixel's height, are drawn as lines of their colour and length, which stay crisp. KubaOS's
// own Settings is its mockup itself (`apps/settings/`).
//
// The apps' content follows the palette's roles, so a miniature takes the wallpaper's scheme with
// the rest of the desktop, as the apps themed from it do.

import shibuya from "../../assets/lock/shibuya-rainy-corner.jpg?url";
import shinjuku from "../../assets/lock/shinjuku-purple-hour.jpg?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";
import { icon } from "../../lib/icon.ts";
import { text, type TypeStyle } from "../../lib/text.ts";
import type { App, Win } from "./overview-model.ts";
import { bareWindow } from "../apps/settings/settings-window.ts";

function box(name: string, ...children: (Node | string)[]): HTMLElement {
  const element = document.createElement("span");
  element.className = name;
  element.append(...children);
  return element;
}

/** A line of text too small to read, in segments: each a share of the line's width and a colour role. */
type Segment = readonly [share: number, role: string];

function line(...segments: Segment[]): HTMLElement {
  const row = box("ova-line");
  for (const [share, role] of segments) {
    const bar = box("ova-bar");
    bar.style.setProperty("--share", String(share));
    bar.style.setProperty("--ink", role === "transparent" ? "transparent" : `var(--color-${role})`);
    row.append(bar);
  }
  return row;
}

/** An empty line: a gap of one line's height. */
const blank = () => box("ova-line");

/** A seeded stream of numbers in [0, 1), the same for the same window every time it is drawn. */
function stream(seed: string): () => number {
  let state = [...seed].reduce((sum, char) => (sum * 31 + char.charCodeAt(0)) >>> 0, 7);
  return () => {
    state = (state * 1664525 + 1013904223) >>> 0;
    return state / 2 ** 32;
  };
}

/** Code as an editor colours it: keywords, names, strings, comments, indented as a file is. */
function code(seed: string, count: number): HTMLElement[] {
  const next = stream(seed);
  const rows: HTMLElement[] = [];
  let depth = 0;
  for (let index = 0; index < count; index++) {
    const roll = next();
    if (roll < 0.08) {
      rows.push(blank());
      depth = Math.max(0, depth - 1);
      continue;
    }
    const indent: Segment = [depth * 0.04, "transparent"];
    if (roll < 0.18) rows.push(line(indent, [0.3 + next() * 0.3, "outline"]));
    else if (roll < 0.32) {
      rows.push(line(indent, [0.07, "tertiary"], [0.12 + next() * 0.12, "primary"], [0.03, "on_surface_variant"], [0.1 + next() * 0.15, "secondary"]));
      depth = Math.min(4, depth + 1);
    } else if (roll < 0.5) rows.push(line(indent, [0.05, "tertiary"], [0.1 + next() * 0.1, "on_surface"], [0.03, "on_surface_variant"], [0.15 + next() * 0.2, "success"]));
    else rows.push(line(indent, [0.08 + next() * 0.1, "on_surface"], [0.02, "on_surface_variant"], [0.1 + next() * 0.25, "cold"], [0.05 + next() * 0.1, "on_surface_variant"]));
  }
  return rows;
}

/** Plain lines of text, ragged at the end as prose is. */
function prose(seed: string, count: number, role = "on_surface_variant"): HTMLElement[] {
  const next = stream(seed);
  return Array.from({ length: count }, (_, index) => line([index === count - 1 ? 0.35 + next() * 0.3 : 0.85 + next() * 0.15, role]));
}

function words(content: string, style: TypeStyle, role = "on_surface"): HTMLElement {
  const made = text(content, style, role);
  made.classList.add("ova-words");
  return made;
}

function picture(source: string, name = "ova-picture"): HTMLElement {
  const image = document.createElement("img");
  image.className = name;
  image.src = source;
  image.alt = "";
  image.draggable = false;
  return image;
}

function symbol(name: string, role = "on_surface_variant", size = "var(--icon-s20)"): SVGElement {
  return icon(name, size, `var(--color-${role})`);
}

/** A round avatar or a cover, in a container role's colour. */
function dot(role: string, name = ""): HTMLElement {
  const made = box(`ova-dot ${name}`.trim());
  made.style.setProperty("--ink", `var(--color-${role})`);
  return made;
}

// ── Terminals ─────────────────────────────────────────────────────────────────────────────────

function prompt(path: string): HTMLElement {
  return line([0.02, "success"], [path.length * 0.012, "primary"], [0.08, "tertiary"]);
}

function mockups(win: Win): HTMLElement[] {
  return [
    prompt("~/Projects/kubaos"),
    line([0.04, "success"], [0.3, "on_surface"]),
    blank(),
    line([0.12, "success"], [0.08, "on_surface_variant"], [0.2, "outline"]),
    blank(),
    line([0.03, "success"], [0.08, "on_surface"], [0.34, "cold"]),
    line([0.03, "success"], [0.1, "on_surface_variant"], [0.22, "outline"]),
    blank(),
    ...Array.from({ length: 14 }, (_, i) => line([0.1, "outline"], [0.08, "cold"], [0.07, "success"], [0.2 + ((i * 7) % 5) * 0.05, "on_surface_variant"])),
    line([0.1, "outline"], [0.08, "cold"], [0.1, "warning"], [0.3, "on_surface_variant"]),
    ...prose(win.id, 3, "outline"),
  ];
}

function btop(win: Win): HTMLElement[] {
  const next = stream(win.id);
  const graph = box("ova-btop-graph");
  for (let i = 0; i < 48; i++) {
    const bar = box("ova-btop-bar");
    bar.style.setProperty("--level", String(0.15 + next() * 0.8));
    bar.style.setProperty("--ink", `var(--color-${i % 3 === 0 ? "tertiary" : "primary"})`);
    graph.append(bar);
  }
  const cpu = box("ova-btop-box", graph);
  const memory = box("ova-btop-box", ...Array.from({ length: 4 }, (_, i) => line([0.2, "on_surface_variant"], [0.3 + i * 0.12, ["success", "warning", "cold", "secondary"][i]!])));
  const processes = box("ova-btop-box ova-btop-wide", ...Array.from({ length: 16 }, (_, i) => line([0.08, "outline"], [0.22, i === 2 ? "primary" : "on_surface"], [0.1, "on_surface_variant"], [0.06, i < 3 ? "warning" : "success"])));
  return [box("ova-btop", cpu, memory, processes)];
}

function nvim(win: Win): HTMLElement[] {
  const numbers = box("ova-gutter", ...Array.from({ length: 40 }, () => line([1, "outline"])));
  const lines = box("ova-code", ...code(win.id, 40));
  const tabs = box("ova-nvim-tabs", line([0.18, "on_primary"]));
  const status = box("ova-nvim-status", box("ova-nvim-mode"), line([0.3, "on_surface_variant"], [0.4, "transparent"], [0.15, "on_surface_variant"]));
  return [tabs, box("ova-editor", numbers, lines), status];
}

function lazygit(win: Win): HTMLElement[] {
  const pane = (count: number, selected: number, seed: string) => {
    const next = stream(seed);
    return box("ova-pane", ...Array.from({ length: count }, (_, i) => {
      const made = line([0.05, i % 4 === 0 ? "warning" : "success"], [0.3 + next() * 0.45, "on_surface"]);
      if (i === selected) made.classList.add("ova-picked");
      return made;
    }));
  };
  const left = box("ova-stack", pane(5, 1, `${win.id}a`), pane(7, 0, `${win.id}b`), pane(9, 3, `${win.id}c`));
  const diff = box("ova-pane ova-grow", ...code(`${win.id}d`, 30).map((row, i) => {
    if (i % 5 === 2) row.classList.add("ova-added");
    if (i % 7 === 4) row.classList.add("ova-removed");
    return row;
  }));
  return [box("ova-lazygit", left, diff)];
}

function terminal(win: Win): HTMLElement {
  const content = win.shell === "btop" ? btop(win) : win.shell === "nvim" ? nvim(win) : win.shell === "git" ? lazygit(win) : mockups(win);
  const face = box("ova-face ova-terminal", ...content);
  face.dataset.shell = win.shell ?? "mockups";
  return face;
}

function yazi(win: Win): HTMLElement {
  const next = stream(win.id);
  const header = line([0.02, "success"], [0.3, "primary"]);
  const row = (folder: boolean, share: number, picked = false) => {
    const made = box("ova-yazi-row", folder ? symbol("folder", "primary", "var(--icon-s16)") : symbol("draft", "on_surface_variant", "var(--icon-s16)"), line([share, folder ? "primary" : "on_surface"]));
    made.toggleAttribute("data-picked", picked);
    return made;
  };
  const parent = box("ova-yazi-col", ...Array.from({ length: 9 }, (_, i) => row(true, 0.4 + next() * 0.4, i === 4)));
  const current = box("ova-yazi-col ova-yazi-main", ...Array.from({ length: 22 }, (_, i) => row(i < 5, 0.3 + next() * 0.5, i === 7)));
  const preview = box("ova-yazi-col ova-yazi-preview", picture(shibuya));
  const footer = line([0.06, "on_primary"], [0.2, "on_surface_variant"], [0.5, "transparent"], [0.1, "on_surface_variant"]);
  footer.classList.add("ova-yazi-status");
  return box("ova-face ova-terminal ova-yazi", header, box("ova-yazi-cols", parent, current, preview), footer);
}

// ── An editor ─────────────────────────────────────────────────────────────────────────────────

function vscodeFace(win: Win): HTMLElement {
  const activity = box("ova-activity", ...["draft", "search", "deployed_code", "code", "settings"].map((name, i) => symbol(name, i === 0 ? "on_surface" : "outline", "var(--icon-s24)")));
  const next = stream(`${win.id}tree`);
  const tree = box("ova-tree", line([0.5, "on_surface_variant"]), ...Array.from({ length: 26 }, (_, i) => {
    const depth = i < 3 ? 0 : i < 9 ? 1 : 2;
    const made = line([depth * 0.08, "transparent"], [0.08, i % 5 === 0 ? "primary" : "outline"], [0.3 + next() * 0.3, "on_surface_variant"]);
    if (i === 12) made.classList.add("ova-picked");
    return made;
  }));
  const tabs = box("ova-tabs", box("ova-tab ova-tab-on", line([0.8, "on_surface"])), box("ova-tab", line([0.7, "outline"])), box("ova-tab", line([0.6, "outline"])));
  const numbers = box("ova-gutter", ...Array.from({ length: 44 }, () => line([1, "outline"])));
  const editor = box("ova-editor", numbers, box("ova-code", ...code(win.id, 44)));
  const status = box("ova-status", line([0.12, "on_primary"], [0.6, "transparent"], [0.2, "on_primary"]));
  return box("ova-face ova-vscode", box("ova-vscode-main", activity, tree, box("ova-vscode-pane", tabs, editor)), status);
}

// ── Browsers ──────────────────────────────────────────────────────────────────────────────────

function chrome(tabs: number): HTMLElement {
  const strip = box("ova-tabstrip", ...Array.from({ length: tabs }, (_, i) => box(i === 0 ? "ova-btab ova-btab-on" : "ova-btab", dot(i === 0 ? "primary" : "outline", "ova-favicon"), line([0.7, i === 0 ? "on_surface" : "outline"]))));
  const address = box("ova-address", symbol("lock", "outline", "var(--icon-s16)"), line([0.4, "on_surface_variant"]));
  return box("ova-browser-top", strip, box("ova-toolbar", symbol("arrow_back", "outline"), symbol("arrow_forward", "outline"), symbol("restart_alt", "outline"), address));
}

function docsPage(win: Win): HTMLElement {
  const nav = box("ova-site-nav", dot("primary", "ova-logo"), line([0.08, "on_surface"], [0.08, "on_surface_variant"], [0.08, "on_surface_variant"], [0.08, "on_surface_variant"]));
  const side = box("ova-site-side", ...prose(`${win.id}s`, 14, "on_surface_variant").map((row, i) => (i === 3 && row.classList.add("ova-picked"), row)));
  const article = box("ova-site-article", words("Dispatchers", "display"), ...prose(`${win.id}a`, 4), picture(tower, "ova-figure"), ...prose(`${win.id}b`, 3), box("ova-snippet", ...code(`${win.id}c`, 6)), ...prose(`${win.id}d`, 5));
  return box("ova-site", nav, box("ova-site-body", side, article));
}

function pullsPage(win: Win): HTMLElement {
  const next = stream(win.id);
  const head = box("ova-pulls-head", words("Pull requests", "headline"), box("ova-pulls-count", line([0.6, "on_surface_variant"])));
  const rows = Array.from({ length: 12 }, (_, i) => box("ova-pull", dot(i % 4 === 3 ? "tertiary" : "success", "ova-pull-dot"), box("ova-message-words", line([0.4 + next() * 0.4, "on_surface"]), line([0.3 + next() * 0.2, "outline"])), dot("surface_container_highest", "ova-avatar")));
  return box("ova-site ova-pulls", head, box("ova-pull-list", ...rows));
}

function browser(win: Win): HTMLElement {
  const page = win.app === "chromium" ? pullsPage(win) : docsPage(win);
  const face = box("ova-face ova-browser", chrome(win.app === "chromium" ? 4 : 3), page);
  face.dataset.app = win.app;
  return face;
}

// ── Chat and mail ─────────────────────────────────────────────────────────────────────────────

function message(seed: string, role: string, count: number): HTMLElement {
  return box("ova-message", dot(role, "ova-avatar"), box("ova-message-words", line([0.15, role], [0.06, "outline"]), ...prose(seed, count)));
}

function discordFace(win: Win): HTMLElement {
  const servers = box("ova-servers", ...["primary", "secondary", "tertiary", "cold", "success"].map((role) => dot(role, "ova-server")));
  const next = stream(`${win.id}c`);
  const channels = box("ova-channels", line([0.6, "on_surface"]), ...Array.from({ length: 14 }, (_, i) => {
    const made = line([0.06, "outline"], [0.3 + next() * 0.4, i === 2 ? "on_surface" : "on_surface_variant"]);
    if (i === 2) made.classList.add("ova-picked");
    return made;
  }));
  const roles = ["primary", "tertiary", "secondary", "cold", "primary", "success"];
  const messages = box("ova-messages", ...roles.map((role, i) => message(`${win.id}${i}`, role, 1 + (i % 3))));
  const input = box("ova-input", line([0.3, "outline"]));
  return box("ova-face ova-chat", servers, channels, box("ova-conversation", box("ova-conversation-head", line([0.2, "on_surface"])), messages, input));
}

function telegramFace(win: Win): HTMLElement {
  const next = stream(`${win.id}l`);
  const chats = box("ova-chats", ...Array.from({ length: 11 }, (_, i) => {
    const made = box("ova-chat-row", dot(["primary", "secondary", "tertiary", "cold"][i % 4]!, "ova-avatar"), box("ova-message-words", line([0.4 + next() * 0.3, "on_surface"]), line([0.6 + next() * 0.3, "outline"])));
    made.toggleAttribute("data-picked", i === 0);
    return made;
  }));
  const bubbles = box("ova-bubbles", ...[0, 1, 0, 0, 1, 0, 1].map((mine, i) => {
    const made = box("ova-bubble", ...prose(`${win.id}${i}`, 1 + (i % 3), mine ? "on_primary_container" : "on_surface"));
    made.toggleAttribute("data-mine", mine === 1);
    return made;
  }));
  const input = box("ova-input", line([0.3, "outline"]));
  return box("ova-face ova-telegram", chats, box("ova-conversation", box("ova-conversation-head", line([0.2, "on_surface"])), bubbles, input));
}

function mail(win: Win): HTMLElement {
  const folders = box("ova-folders", ...["mail", "draft", "star", "delete"].map((name, i) => {
    const made = box("ova-folder", symbol(name, i === 0 ? "primary" : "outline", "var(--icon-s16)"), line([0.6, i === 0 ? "on_surface" : "on_surface_variant"]));
    made.toggleAttribute("data-picked", i === 0);
    return made;
  }));
  const next = stream(`${win.id}m`);
  const list = box("ova-mail-list", ...Array.from({ length: 12 }, (_, i) => {
    const made = box("ova-mail-row", box(i < 3 ? "ova-unread" : "ova-read"), box("ova-message-words", line([0.3 + next() * 0.2, i < 3 ? "on_surface" : "on_surface_variant"], [0.15, "transparent"], [0.1, "outline"]), line([0.5 + next() * 0.4, "outline"])));
    made.toggleAttribute("data-picked", i === 0);
    return made;
  }));
  const reading = box("ova-reading", words("Your order has shipped", "headline"), line([0.3, "on_surface_variant"], [0.2, "transparent"], [0.15, "outline"]), ...prose(`${win.id}r`, 6), blank(), ...prose(`${win.id}s`, 4));
  return box("ova-face ova-mail", folders, list, reading);
}

// ── Media ─────────────────────────────────────────────────────────────────────────────────────

/** A bar filled `share` of the way in a colour role: a game's life, a track's progress. */
function meter(role: string, share: number, name = "ova-meter"): HTMLElement {
  const made = box(name);
  made.style.setProperty("--ink", `var(--color-${role})`);
  made.style.setProperty("--share", String(share));
  return made;
}

function progress(share: number): HTMLElement {
  return box("ova-progress", meter("primary", share, "ova-progress-fill"));
}

function game(): HTMLElement {
  const hud = box("ova-hud", meter("error", 0.7), meter("cold", 0.45));
  const boons = box("ova-boons", ...["error", "cold", "warning", "tertiary"].map((role) => dot(role, "ova-boon")));
  return box("ova-face ova-game", picture(tower, "ova-scene"), hud, boons);
}

function video(win: Win): HTMLElement {
  const bar = box("ova-player", symbol("play_arrow", "on_surface", "var(--icon-s24)"), progress(0.38), line([1, "on_surface_variant"]));
  return box("ova-face ova-video", picture(win.id === "w-cut" ? shibuya : shinjuku, "ova-scene"), bar);
}

function cover(role: string): HTMLElement {
  return dot(role, "ova-cover");
}

function spotifyFace(win: Win): HTMLElement {
  const roles = ["primary_container", "secondary_container", "tertiary_container", "cold_container", "error_container", "surface_container_highest"];
  const library = box("ova-library", ...roles.concat(roles.slice(0, 3)).map((role) => box("ova-library-row", cover(role), box("ova-message-words", line([0.6, "on_surface"]), line([0.4, "outline"])))));
  const head = box("ova-playlist-head", cover("primary"), box("ova-playlist-words", line([0.15, "on_surface_variant"]), words("Discover Weekly", "display"), line([0.5, "on_surface_variant"])));
  const next = stream(`${win.id}t`);
  const tracks = box("ova-tracks", ...Array.from({ length: 10 }, (_, i) => box("ova-track", line([0.02, i === 2 ? "primary" : "outline"]), cover(roles[i % roles.length]!), box("ova-message-words", line([0.3 + next() * 0.3, i === 2 ? "primary" : "on_surface"]), line([0.2 + next() * 0.2, "outline"])), line([0.05, "outline"]))));
  const player = box("ova-now", cover("tertiary_container"), box("ova-message-words", line([0.5, "on_surface"]), line([0.3, "outline"])), progress(0.38));
  return box("ova-face ova-music", box("ova-music-main", library, box("ova-music-pane", head, tracks)), player);
}

function strawberryFace(win: Win): HTMLElement {
  const next = stream(`${win.id}c`);
  const collection = box("ova-collection", ...Array.from({ length: 18 }, (_, i) => line([i % 4 === 0 ? 0 : 0.06, "transparent"], [0.4 + next() * 0.4, i % 4 === 0 ? "on_surface" : "on_surface_variant"])));
  const table = box("ova-table", ...Array.from({ length: 16 }, (_, i) => {
    const made = line([0.04, "outline"], [0.3, i === 4 ? "primary" : "on_surface"], [0.04, "transparent"], [0.22, "on_surface_variant"], [0.04, "transparent"], [0.08, "outline"]);
    if (i === 4) made.classList.add("ova-picked");
    return made;
  }));
  const album = box("ova-album", cover("secondary_container"), words("Night Drive", "headline"), line([0.6, "on_surface_variant"]));
  const player = box("ova-now", symbol("play_arrow", "on_surface", "var(--icon-s24)"), progress(0.38), line([0.1, "on_surface_variant"]));
  return box("ova-face ova-music ova-strawberry", box("ova-music-main", collection, table, album), player);
}

// ── Settings ──────────────────────────────────────────────────────────────────────────────────

function settings(): HTMLElement {
  return box("ova-face ova-settings", bareWindow({}));
}

const FACES: Record<App, (win: Win) => HTMLElement> = {
  ghostty: terminal,
  yazi,
  vscode: vscodeFace,
  firefox: browser,
  chromium: browser,
  discord: discordFace,
  telegram: telegramFace,
  thunderbird: mail,
  steam: game,
  showtime: video,
  spotify: spotifyFace,
  strawberry: strawberryFace,
  settings,
};

/** What `win`'s app shows, filling its window. */
/**
 * `win`'s content: a picture of an app's window, not more of it. Its parts say nothing to the
 * geometry check, which holds the page's own parts, not what a window shows at a miniature's scale.
 */
export function face(win: Win): HTMLElement {
  const drawn = FACES[win.app](win);
  for (const each of [drawn, ...drawn.querySelectorAll<HTMLElement | SVGElement>("[data-probe], [data-scroll-probe]")]) {
    delete each.dataset.probe;
    delete each.dataset.scrollProbe;
  }
  return drawn;
}
