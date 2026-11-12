// The bar's audio pills, as `config/quickshell/modules/bar/` is to draw them: the mute pill (the
// output's and the input's symbols, each a press target that mutes its side, the wheel over it its
// level, a right-click the Audio panel) and now playing (the active player's icon, which opens the
// players Menu under it, then previous, play or pause on the accent's fill, next). Each press target
// is a circle a chip tall in a pill `space.s4` in, `space.s4` apart, lit by its state layer as the
// tray's circles are; the pill itself takes no tube. The data is the shots' stand-ins'.

import "./bar.css";
import { badgedIcon } from "../../lib/badge.ts";
import { icon } from "../../lib/icon.ts";
import { type MenuEntry, menu } from "../../lib/menu.ts";
import { pill } from "../../lib/pill.ts";
import { popover } from "../../lib/tooltip.ts";
import { APP_ICONS } from "../../site/app-icons.ts";

/** A circle pinned in a state, as the specimens pin them. */
export type CirclePinned = "lit" | "pressed";

export interface CircleOptions {
  /** On the accent's fill, its content `on_primary`: play or pause. */
  readonly filled?: boolean;
  readonly pinned?: CirclePinned;
  /** What a press, or Enter from the keyboard on its menu, does. */
  readonly onPress?: () => void;
  /** Its accessible name: what a press does. */
  readonly label: string;
}

/** One press target in an audio pill: `content` centred in a circle a chip tall, with its state layer. */
export function pillCircle(content: Element, options: CircleOptions): HTMLElement {
  const item = document.createElement("span");
  item.className = "shell-pill-item";
  item.setAttribute("role", "button");
  item.setAttribute("aria-label", options.label);
  item.toggleAttribute("data-filled", options.filled ?? false);
  if (options.pinned) item.dataset.pinned = options.pinned;
  item.append(content);
  if (options.onPress) item.addEventListener("click", options.onPress);
  return item;
}

/** A symbol at the bar's 16 in a colour role. */
function glyph(name: string, color: string): SVGElement {
  return icon(name, "var(--icon-s16)", `var(--color-${color})`);
}

/** Circles in a pill, `space.s4` in and apart. */
export function circlesPill(circles: readonly HTMLElement[]): HTMLElement {
  const made = pill({ lone: false, end: "s4", gap: "s4", content: circles });
  made.classList.add("shell-audio-pill");
  return made;
}

// The mute pill.

/** One side of the mute pill: muted or not, and for the input whether an app is recording. */
export interface Side {
  readonly muted?: boolean;
  /** An app records from the mic: the microphone's privacy dot on its symbol's corner. */
  readonly recording?: boolean;
}

export interface MuteOptions {
  readonly output?: Side;
  readonly input?: Side;
  /** The output's level 0 to 1, which picks its symbol. */
  readonly level?: number;
  /** Given way in the narrowest bar: the mic's circle alone, the output left to its keys and the OSD. */
  readonly micOnly?: boolean;
  /** A circle pinned: which side, and how. */
  readonly pinned?: { readonly side: "output" | "input"; readonly state: CirclePinned };
  readonly onOutput?: () => void;
  readonly onInput?: () => void;
}

/** The output's symbol for `level`, as the OSD and the sliders pick it. */
export function volumeIcon(level: number, muted = false): string {
  return muted || level <= 0 ? "volume_off" : level < 0.34 ? "volume_mute" : level < 0.67 ? "volume_down" : "volume_up";
}

/**
 * The output's symbol, then the mic's: each `on_surface_variant`, or crossed in `error` while muted;
 * the mic carrying the microphone's privacy dot while an app records from it, unless it is muted.
 */
export function mutePill(options: MuteOptions = {}): HTMLElement {
  const output = options.output ?? {};
  const input = options.input ?? {};
  const level = options.level ?? 0.62;
  const outGlyph = glyph(volumeIcon(level, output.muted), output.muted ? "error" : "on_surface_variant");
  const dotted = (input.recording ?? false) && !(input.muted ?? false);
  const inGlyph = badgedIcon({ name: input.muted ? "mic_off" : "mic", color: input.muted ? "error" : "on_surface_variant", badged: dotted, meaning: "microphone" });
  const pinnedOf = (side: "output" | "input") => (options.pinned?.side === side ? { pinned: options.pinned.state } : {});
  const circles = [
    pillCircle(outGlyph, { label: output.muted ? "Unmute the output" : "Mute the output", ...pinnedOf("output"), ...(options.onOutput ? { onPress: options.onOutput } : {}) }),
    pillCircle(inGlyph, { label: input.muted ? "Unmute the mic" : "Mute the mic", ...pinnedOf("input"), ...(options.onInput ? { onPress: options.onInput } : {}) }),
  ];
  const made = circlesPill(options.micOnly ? circles.slice(1) : circles);
  made.classList.add("shell-mute-pill");
  return made;
}

// Now playing.

/** A player MPRIS knows: its app's name and icon. */
export interface Player {
  readonly app: string;
  readonly picture: string;
}

const [KUBAOS, PLATE, DISC, STAR] = APP_ICONS;

/** The shots' players: the app names are stand-ins, each on one of the IconTile's kinds of icon. */
export const PLAYERS: readonly Player[] = [
  { app: "Spotify", picture: DISC!.picture },
  { app: "Firefox", picture: KUBAOS!.picture },
  { app: "Discord", picture: PLATE!.picture },
];

/** A player whose app's name runs long, for the menu's longest line. */
export const LONG_PLAYER: Player = { app: "Strawberry Music Player (Flatpak)", picture: STAR!.picture };

/** Which circle of now playing is pinned: its icon, previous, play or pause, next. */
export type PlayingPart = "icon" | "previous" | "play" | "next";

export interface PlayingOptions {
  readonly players?: readonly Player[];
  /** The active player, by its index. */
  readonly active?: number;
  readonly playing?: boolean;
  /** Given way in a narrow bar: the icon and play or pause alone. */
  readonly compact?: boolean;
  readonly pinned?: { readonly part: PlayingPart; readonly state: CirclePinned };
  readonly onIcon?: () => void;
  readonly onPlay?: () => void;
}

/** An app's picture at the bar's 16, as the tray draws one; with none, a note's symbol in its place. */
function appPicture(player: Player): Element {
  if (!player.picture) return glyph("music_note", "on_surface_variant");
  const image = document.createElement("img");
  image.className = "shell-tray-picture";
  image.src = player.picture;
  image.alt = "";
  return image;
}

/** The active player's icon, then previous, play or pause on the accent's fill, next; none with no player. */
export function nowPlaying(options: PlayingOptions = {}): HTMLElement {
  const players = options.players ?? PLAYERS;
  const active = players[options.active ?? 0] ?? players[0]!;
  const playing = options.playing ?? true;
  const pinnedOf = (part: PlayingPart) => (options.pinned?.part === part ? { pinned: options.pinned.state } : {});
  const icon_ = pillCircle(appPicture(active), { label: `Players: ${active.app}`, ...pinnedOf("icon"), ...(options.onIcon ? { onPress: options.onIcon } : {}) });
  icon_.setAttribute("aria-haspopup", "menu");
  const play = pillCircle(glyph(playing ? "pause" : "play_arrow", "on_primary"), { label: playing ? "Pause" : "Play", filled: true, ...pinnedOf("play"), ...(options.onPlay ? { onPress: options.onPlay } : {}) });
  const made = circlesPill(
    options.compact
      ? [icon_, play]
      : [icon_, pillCircle(glyph("skip_previous", "on_surface_variant"), { label: "Previous", ...pinnedOf("previous") }), play, pillCircle(glyph("skip_next", "on_surface_variant"), { label: "Next", ...pinnedOf("next") })],
  );
  made.classList.add("shell-playing-pill");
  return made;
}

/** The players Menu's entries: every player with its icon, the active one checked, then Show <app>. */
export function playerEntries(players: readonly Player[], active: number): MenuEntry[] {
  const app = players[active]?.app ?? "";
  return [...players.map((player, index) => ({ label: player.app, ...(player.picture ? { picture: player.picture } : { icon: "music_note" }), checked: index === active })), "separator", { label: `Show ${app}`, icon: "app_window" }];
}

export interface PlayersMenuOptions {
  readonly players?: readonly Player[];
  readonly active?: number;
  /** The item the keyboard is on, by its index among the entries. */
  readonly current?: number;
  readonly onTake?: (index: number) => void;
  readonly onLeave?: () => void;
}

/** The players Menu on its popover. */
export function playersMenu(options: PlayersMenuOptions = {}): { readonly card: HTMLElement; readonly list: HTMLElement } {
  const players = options.players ?? PLAYERS;
  const list = menu(playerEntries(players, options.active ?? 0), {
    ...(options.current !== undefined ? { current: options.current } : {}),
    ...(options.onTake ? { onTake: options.onTake } : {}),
    ...(options.onLeave ? { onLeave: options.onLeave } : {}),
  });
  const card = popover(2, list);
  card.classList.add("shell-tray-menu");
  return { card, list };
}

/**
 * Now playing with its players Menu open `space.s12` under the icon's circle, its start on the
 * circle's, the icon pinned pressed and the keyboard on the item `current`: the active one.
 */
export function nowPlayingOpen(options: { players?: readonly Player[]; active?: number; current?: number; playing?: boolean } = {}): HTMLElement {
  const players = options.players ?? PLAYERS;
  const active = options.active ?? 0;
  const room = document.createElement("span");
  room.className = "shell-tray-room";
  const bar = nowPlaying({ players, active, playing: options.playing ?? true, pinned: { part: "icon", state: "lit" } });
  const { card } = playersMenu({ players, active, current: options.current ?? active });
  room.append(bar, card);
  requestAnimationFrame(() => {
    const circle = bar.querySelector<HTMLElement>(".shell-pill-item");
    if (circle) card.style.setProperty("--menu-x", `${(circle.getBoundingClientRect().left - room.getBoundingClientRect().left) * (room.offsetWidth / Math.max(1, room.getBoundingClientRect().width))}px`);
  });
  return room;
}
