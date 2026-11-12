// The power menu, "Leaving the room": the desktop steps back as the frame closes in, a sheet rises
// from the middle of the frame's bottom edge holding what the action you're on does and five tiles,
// and that action plays out on the desktop itself before you commit. Lock and Suspend act at once;
// Log out, Restart and Shut down are held. Once Restart or Shut down is held to the end the room
// goes dark with the boot splash's sign already lit over it, and the sign goes out a group of tubes
// at a time, the last first, as the splash does on the way down (`kubaos.script`).
//
// The room is the Lock's desktop (`lib/lock.ts`): its picture, its frame, its bar and its windows,
// moved by the Lock's own numbers, and a library Sheet on the bottom edge. Every move here is data,
// tracks the page's players play, and the live room plays the same tracks from wherever the room
// stands, so what the recipe says and what you try never disagree.

import "./power-room.css";
import signBa from "../../../../../system/files/plymouth/kubaos/sign-ba@2x.png?url";
import signBase from "../../../../../system/files/plymouth/kubaos/sign-base@2x.png?url";
import signDakuten from "../../../../../system/files/plymouth/kubaos/sign-dakuten@2x.png?url";
import signKu from "../../../../../system/files/plymouth/kubaos/sign-ku@2x.png?url";
import signKuba from "../../../../../system/files/plymouth/kubaos/sign-kuba@2x.png?url";
import signOs from "../../../../../system/files/plymouth/kubaos/sign-os@2x.png?url";
import { banner } from "../../lib/banner.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import { type Pinned, pin } from "../../lib/control.ts";
import { focusRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { filling, row, stack } from "../../lib/layout.ts";
import { lock } from "../../lib/lock.ts";
import { paintNight } from "../../lib/lock-tint.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { nested, role } from "../../lib/radius.ts";
import { hang, sheet } from "../../lib/sheet.ts";
import { text } from "../../lib/text.ts";
import { keycap, keys } from "../../lib/keycap.ts";
import { floatingTooltip, keyTooltip, tooltip } from "../../lib/tooltip.ts";
import type { Duration, Easing, NonEmpty, Track } from "../../site/page.ts";
import { between, bezier, css, cueOf, milliseconds, stepsOf, struck } from "../../site/recipe.ts";
import { paint, paletteOf } from "../../site/recolour.ts";
import { settings } from "../../site/settings.ts";
import { barButtons } from "../templates/panels.ts";
import { PURPLE_HOUR, SCREEN } from "../templates/locks.ts";
import { BLURRED, DIMMED, ON, reducedRest, resting, touching, touchingReduced } from "../templates/lock-moves.ts";
import { PURPLE_DAY } from "./wallpaper-schemes.ts";

type Eased = Track<Easing>;

/** One of the five actions: its tile, its letter, and whether it is held to commit. */
export interface Action {
  readonly id: string;
  readonly name: string;
  readonly icon: string;
  readonly key: string;
  readonly held: boolean;
}

export const ACTIONS: readonly Action[] = [
  { id: "lock", name: "Lock", icon: "lock", key: "L", held: false },
  { id: "suspend", name: "Suspend", icon: "bedtime", key: "S", held: false },
  { id: "logout", name: "Log out", icon: "logout", key: "O", held: true },
  { id: "restart", name: "Restart", icon: "restart_alt", key: "R", held: true },
  { id: "shutdown", name: "Shut down", icon: "power_settings_new", key: "D", held: true },
];

export const LOCK = 0;
export const SUSPEND = 1;
export const LOGOUT = 2;
export const RESTART = 3;
export const SHUTDOWN = 4;

/** What the machine is in the middle of: work an app won't let go of, an update waiting. */
export interface Situation {
  readonly unsaved: boolean;
  readonly update: boolean;
}

export const CALM: Situation = { unsaved: false, update: true };

/** A window of the desktop: its name and a line, and, for one that won't quit, why. */
interface Shown {
  readonly title: string;
  readonly line: string;
  readonly glyph: string;
  readonly blocks?: { readonly app: string; readonly why: string; readonly chip: string };
}

const WINDOWS: readonly Shown[] = [
  { title: "kitty — ~/kubaos", line: "cargo build --release", glyph: "terminal", blocks: { app: "kitty", why: "Still running cargo build.", chip: "cargo build is running" } },
  { title: "Zed — main.rs", line: "let menu = PowerMenu::new();", glyph: "code", blocks: { app: "Zed", why: "Changes to main.rs aren't saved.", chip: "main.rs isn't saved" } },
  { title: "Files — Pictures", line: "wallpapers, shots, notes.md", glyph: "folder" },
];

/** The windows that won't quit, in the order the blocker lists them. */
const BLOCKING = [1, 0] as const;

/** The tile's name in its tooltip, held or not. */
export function tipTitle(index: number): string {
  const action = ACTIONS[index]!;
  return action.held ? `${action.name}, held` : action.name;
}

/** What a tooltip adds past the name: Restart and Shut down can't be stopped once the sign shows. */
export const NO_UNDO = "No undo once the sign shows.";

/**
 * The tile's tooltip: its name and letter on one line, or for Restart and Shut down the name, the
 * words on what can't be undone, and the letter under them.
 */
function tipOf(index: number): HTMLElement {
  const action = ACTIONS[index]!;
  return index >= RESTART ? tooltip(tipTitle(index), NO_UNDO, row("s4", "centre", ...keys(action.key).map(keycap))) : keyTooltip(tipTitle(index), action.key);
}

/** The line over the tiles: what the action does, in a sentence. */
export function lead(index: number, situation: Situation): string {
  return [
    "Locks the screen. Everything keeps running behind it.",
    "Sleeps until you're back. The battery barely notices.",
    "Closes everything you have open. KubaOS stays up.",
    situation.update ? "Off, update, back on. Takes about 40 s." : "Off and back on. The oldest fix there is.",
    "Closes everything and turns the lights off.",
  ][index]!;
}

/** The line while it is held: once the sign is up, it can't be stopped. */
export const HOLDING: readonly string[] = ["", "", "Keep holding to log out.", "Keep holding. Once the sign's up, it's restarting.", "Keep holding. Once the sign's up, it's going down."];

/** The line once a held action meets apps that won't quit. */
export const WAITING = "Waiting on 2 apps. They won't quit on their own.";

/** A chip of a fact in `warning`, which the library's chip has no kind for. */
function warned(label: string, glyph: string): HTMLElement {
  const made = chip({ label, icon: glyph });
  // A Cold chip's look in `warning`: its tint under its words and symbol in the tone.
  made.style.setProperty("--fill", "rgb(from var(--color-warning) r g b / var(--opacity-tint))");
  made.style.setProperty("--label", "var(--color-warning)");
  return made;
}

/** The facts beside the line, as chips. */
function facts(index: number, situation: Situation): HTMLElement[] {
  const fact = (glyph: string, label: string) => chip({ label, icon: glyph });
  const blocked = situation.unsaved && index >= LOGOUT ? warned("2 apps aren't done: Zed, kitty", "warning") : null;
  const lists: (HTMLElement | null)[][] = [
    [fact("schedule", "Up 3 d 4 h"), fact("battery_full", "87%")],
    [fact("battery_full", "87%, about 9 days asleep")],
    [blocked ?? fact("desktop_windows", "3 windows close")],
    [situation.update ? fact("system_update_alt", "Installs Shinjuku 0.1.3") : fact("schedule", "Up 3 d 4 h"), blocked],
    [blocked, fact("schedule", "Up 3 d 4 h"), situation.update ? fact("system_update_alt", "0.1.3 waits for a restart") : null],
  ];
  return lists[index]!.filter((each): each is HTMLElement => each !== null);
}

// ─── Moves, as data ─────────────────────────────────────────────────────────────────────────────

export const ROOM = "the room";
const R = ".power-room";
const SHEET = ".power-sheet";
const WINDOWS_BOX = ".k-lock-windows";
const CLOCK = ".k-lock-clock";
const BAR_TIME = ".k-lock-bar-time";
const KICKER = ".k-lock-kicker";
const BAR = ".k-bar";
const FRAME = ".k-frame";
const SPLASH = ".power-splash";
const BLACK = ".power-black";
const ART = ".power-sign";
const GLASS = ".power-sign-base";
const windowAt = (i: number) => `.power-window[data-at="${i}"]`;
const tileOf = (i: number) => `.power-tile[data-action="${ACTIONS[i]!.id}"]`;

/** The tubes' groups in the order they switch on (`kubaos.script`'s GROUP); they go out the other way. */
export const GROUPS = ["ku", "ba", "dakuten", "kuba", "os"] as const;

/** `count` of `step`, a cue of whole phases. */
function times(count: number, step: Duration): Duration[] {
  return Array.from({ length: count }, () => step);
}

/** A cue of `phases`, or none for a part that starts with the move. */
function cue(...phases: Duration[]): Pick<Track, "cue"> {
  return phases.length === 0 ? {} : { cue: phases as unknown as NonEmpty<Duration> };
}

function track(part: string, target: string, property: string, from: Track["from"], to: Track["to"], duration: Duration, curve: Easing, ...at: Duration[]): Eased {
  return { part, on: ROOM, target, property, from, to, duration, curve, ...cue(...at) };
}

/** Opening: the frame closes in, the picture pushes in, the windows step back and the sheet rises. */
export function opening(): NonEmpty<Eased> {
  return [
    track("the frame closing in", R, "--power-closein", 0, 1, "long", "emphasized_decel"),
    track("the push", R, "--push", 0, 1, "long", "emphasized_decel"),
    track("the windows stepping back", WINDOWS_BOX, "scale", 1, { scale: "behind" }, "long", "emphasized_decel"),
    track("the sheet", SHEET, "--reveal", 0, 1, "long", "emphasized_decel"),
  ];
}

export function openingReduced(): NonEmpty<Eased> {
  return [track("the sheet", SHEET, "--reveal", 0, 1, "instant", "standard"), track("the sheet's fade", SHEET, "opacity", 0, 1, "short", "standard")];
}

/** Closing: the sheet sinks and the frame opens out, and the desktop comes forward again. */
export function closing(): NonEmpty<Eased> {
  return [
    track("the sheet", SHEET, "--reveal", 1, 0, "medium", "emphasized_accel"),
    track("the frame closing in", R, "--power-closein", 1, 0, "medium", "emphasized_accel"),
    track("the push", R, "--push", 1, 0, "long", "emphasized_decel"),
    track("the windows stepping back", WINDOWS_BOX, "scale", { scale: "behind" }, 1, "long", "emphasized_decel"),
  ];
}

export function closingReduced(): NonEmpty<Eased> {
  return [track("the sheet's fade", SHEET, "opacity", 1, 0, "short", "standard"), track("the sheet", SHEET, "--reveal", 1, 0, "instant", "standard", "short")];
}

/** How dark the room is while `index` is looked at: black over it, as `opacity` of it. */
function dusk(index: number): Track["to"] {
  return index === SUSPEND ? { opacity: "scrim" } : index === SHUTDOWN ? { opacity: "power_dusk" } : 0;
}

/**
 * The desktop as `index` would leave it, from the desktop at rest round the open sheet. Lock: the
 * lock as it is awake, the windows gone, the bar up, the clock lifted out of it and the picture
 * blurred and dimmed. Suspend: the lights go down. Log out: the windows leave one after another,
 * those that won't quit staying with their warning tube lit. Restart: the lights cycle (`cycling`).
 * Shut down: the windows and the bar go and the lights go down, the sheet's tubes last.
 */
export function previewing(index: number, situation: Situation = CALM, reduced = false): NonEmpty<Eased> {
  const lock = index === LOCK;
  const curve: Easing = lock ? "emphasized_decel" : "standard";
  const out: Eased[] = [];
  if (!reduced) out.push(track("the bar's band", R, "--barred", 1, lock || index === SHUTDOWN ? 0 : 1, "long", "emphasized_decel"));
  out.push(
    track("the picture's blur", BLURRED, "--wake", 0, lock ? 1 : 0, reduced ? "short" : "long", reduced ? "standard" : curve),
    track("the picture's dim", DIMMED, "--panel-dim", 0, lock ? 1 : 0, reduced ? "short" : "long", reduced ? "standard" : curve),
  );
  if (lock) {
    out.push(
      track("the clock's lift", CLOCK, "--clock-lift", 0, 1, "instant", "standard"),
      ...(reduced
        ? [track("the clock's flight", CLOCK, "--dock", 1, 0, "instant", "standard"), track("the clock's fade", CLOCK, "opacity", 0, 1, "short", "standard")]
        : [track("the clock's fade", CLOCK, "opacity", 0, 1, "instant", "standard"), track("the clock's flight", CLOCK, "--dock", 1, 0, "long", "emphasized_decel")]),
      track("the bar's time", BAR_TIME, "opacity", 1, 0, "micro", "standard"),
    );
  }
  out.push(
    track("the lights", R, "--power-dark", 0, dusk(index), index === SHUTDOWN && !reduced ? "slow" : reduced ? "short" : "long", "standard", ...(index === SHUTDOWN && !reduced ? (["short"] as Duration[]) : [])),
    track("the bar", BAR, "opacity", 1, index === SHUTDOWN || (lock && reduced) ? 0 : 1, reduced ? "short" : "medium", "standard"),
  );
  WINDOWS.forEach((shown, i) => {
    const stays = index >= LOGOUT && situation.unsaved && shown.blocks !== undefined;
    const gone = (lock || index === LOGOUT || index === SHUTDOWN) && !stays;
    const lag = reduced || !gone ? [] : times(i, "micro");
    out.push(track(`window ${i + 1}`, windowAt(i), "opacity", 1, gone ? 0 : 1, reduced ? "short" : gone ? "medium" : "long", reduced ? "standard" : gone ? "emphasized_accel" : "standard", ...lag));
    if (!reduced) out.push(track(`window ${i + 1}'s drop`, windowAt(i), "--power-gone", 0, gone && index === LOGOUT ? 1 : 0, gone ? "medium" : "long", gone ? "emphasized_accel" : "emphasized_decel", ...lag));
    if (shown.blocks) out.push(track(`window ${i + 1}'s warning`, windowAt(i), "--power-refuse", 0, stays ? 1 : 0, "short", "standard"));
  });
  return out as unknown as NonEmpty<Eased>;
}

/**
 * Leaving a look at an action for the desktop round the open sheet, as `previewing` plays any
 * other: what the live room plays as the keyboard moves on.
 */
export function unpreviewing(reduced = false): Eased[] {
  return [
    ...(reduced
      ? [track("the clock's fade", CLOCK, "opacity", 1, 0, "short", "standard"), track("the clock's flight", CLOCK, "--dock", 0, 1, "instant", "standard", "short"), track("the bar's time", BAR_TIME, "opacity", 0, 1, "short", "standard", "short")]
      : [
          track("the clock's flight", CLOCK, "--dock", 0, 1, "long", "emphasized_decel"),
          track("the clock's fade", CLOCK, "opacity", 1, 0, "instant", "standard", "long"),
          track("the bar's time", BAR_TIME, "opacity", 0, 1, "instant", "standard", "long"),
        ]),
  ];
}

/**
 * Restart, looked at: the lights cycle, out and back on, over and over, every two `scene`s and a
 * `long`. One cycle: the frame, then the bar a `micro` after, fade down with the room, hold a
 * `long`, and come back up.
 */
export function cycling(): NonEmpty<Eased> {
  return [
    track("the frame", FRAME, "opacity", 1, { opacity: "power_cycle_glass" }, "medium", "standard"),
    track("the frame", FRAME, "opacity", { opacity: "power_cycle_glass" }, 1, "medium", "standard", "long", "medium"),
    track("the bar", BAR, "opacity", 1, { opacity: "power_cycle_glass" }, "medium", "standard", "micro"),
    track("the bar", BAR, "opacity", { opacity: "power_cycle_glass" }, 1, "medium", "standard", "long", "medium", "micro"),
    track("the lights", R, "--power-dark", 0, { opacity: "power_cycle" }, "medium", "standard"),
    track("the lights", R, "--power-dark", { opacity: "power_cycle" }, 0, "medium", "standard", "long", "medium"),
  ];
}

/** How often Restart's cycle starts again, in milliseconds at speed 1. */
export function cycleEvery(): number {
  return 2 * milliseconds("scene") + milliseconds("long");
}

/** What holding `index` darkens the room to: most of the way, the last of it coming with the sign. */
function heldTo(index: number): Track["to"] | null {
  return index === SHUTDOWN ? { opacity: "power_held" } : index === RESTART ? { opacity: "power_restart_held" } : null;
}

/**
 * Holding a held action: its tile runs white-hot and its tint fills it from the start over `hold`,
 * and the room darkens with the fill. Not a move: reduce motion keeps it.
 */
export function holding(index: number): NonEmpty<Eased> {
  const to = heldTo(index);
  return [
    track("the tile's press", tileOf(index), "--press", 0, 1, "micro", "standard"),
    track("the tile's fill", tileOf(index), "--power-held", 0, 1, "hold", "linear"),
    ...(to === null ? [] : [track("the lights", R, "--power-dark", dusk(index), to, "hold", "linear")]),
  ];
}

/** Letting go before the fill is full: it drains over `short`, and the room comes back to the look. */
export function lettingGo(index: number): NonEmpty<Eased> {
  const to = heldTo(index);
  return [
    track("the tile's press", tileOf(index), "--press", 1, 0, "micro", "standard"),
    track("the tile's fill", tileOf(index), "--power-held", 0.5, 0, "short", "standard"),
    ...(to === null ? [] : [track("the lights", R, "--power-dark", 0.5, dusk(index), index === SHUTDOWN ? "slow" : "long", "standard")]),
  ];
}

/** The room's own parts going dark at once over `duration` on `curve`, the sheet with them. */
function darkening(duration: Duration, curve: Easing, from: Track["from"]): Eased[] {
  return [
    ...WINDOWS.map((_, i) => track(`window ${i + 1}`, windowAt(i), "opacity", 1, 0, duration, curve)),
    ...WINDOWS.flatMap((shown, i) => (shown.blocks ? [track(`window ${i + 1}'s warning`, windowAt(i), "--power-refuse", 0, 0, "instant", "standard")] : [])),
    track("the sheet's fade", SHEET, "opacity", 1, 0, duration, curve),
    track("the lights", R, "--power-dark", from, 1, duration, "standard"),
    track("the bar", BAR, "opacity", 1, 0, duration, "standard"),
    track("the frame", FRAME, "opacity", 1, 0, duration, "standard"),
  ];
}

/**
 * Restart or Shut down, held to the end: the room goes dark over `long` as the boot splash's sign
 * comes up over it already lit, on the same duration and curve, so the dark never arrives empty;
 * the black under the sign takes over, unseen. The lit sign holds a `slow`, then a group goes out
 * every `short`, OS first and ク last, each through `neon.strike` played backwards over `medium`
 * (`kubaos.script`'s OFF_HOLD and OFF_STEP); once ク is out the dead glass fades over `medium`.
 */
export function goingDown(index: number): NonEmpty<Track> {
  const from = heldTo(index) ?? 0;
  return [
    ...darkening("long", "standard", from),
    track("the splash", SPLASH, "opacity", 0, 1, "long", "standard"),
    track("the splash's black", BLACK, "opacity", 0, 1, "instant", "standard", "long"),
    ...[...GROUPS].reverse().map((group, i) => ({ part: `${group}'s tubes`, on: ROOM, target: `.power-sign-${group}`, property: "opacity", from: 1, to: 0, duration: "medium" as const, curve: "neon.strike.reversed" as const, cue: ["long", "slow", ...times(i + 1, "short")] as unknown as NonEmpty<Duration> })),
    track("the dead glass", GLASS, "opacity", 1, 0, "medium", "emphasized_accel", "long", "slow", ...times(GROUPS.length, "short"), "medium"),
  ] as unknown as NonEmpty<Track>;
}

/** With the flicker off: the same, each group fading out over `short` where it would strike. */
export function goingDownUnflickered(index: number): NonEmpty<Eased> {
  return goingDown(index).map((each) => (struck(each.curve) ? { ...each, duration: "short" as const, curve: "standard" as const } : each)) as unknown as NonEmpty<Eased>;
}

/** With reduce motion: the room and the sign crossfade over `short`; the sign holds, then fades over `short`. */
export function goingDownReduced(index: number): NonEmpty<Eased> {
  const from = heldTo(index) ?? 0;
  return [
    ...darkening("short", "standard", from),
    track("the splash", SPLASH, "opacity", 0, 1, "short", "standard"),
    track("the splash's black", BLACK, "opacity", 0, 1, "instant", "standard", "short"),
    track("the sign", ART, "opacity", 1, 0, "short", "standard", "short", "slow"),
  ] as unknown as NonEmpty<Eased>;
}

/** Log out, held to the end: the windows, the sheet, the frame and the bar go as the room goes dark. */
export function loggingOut(reduced = false): NonEmpty<Eased> {
  return [...darkening(reduced ? "short" : "medium", reduced ? "standard" : "emphasized_accel", 0), track("the black", BLACK, "opacity", 0, 1, "instant", "standard", reduced ? "short" : "medium"), track("the splash", SPLASH, "opacity", 0, 1, "instant", "standard", reduced ? "short" : "medium"), track("the sign", ART, "opacity", 1, 0, "instant", "standard")] as unknown as NonEmpty<Eased>;
}

/** Suspend: the sheet fades and the lights go down over `slow`, to black. */
export function suspending(reduced = false): NonEmpty<Eased> {
  const d: Duration = reduced ? "short" : "slow";
  return [
    track("the sheet's fade", SHEET, "opacity", 1, 0, d, "standard"),
    track("the lights", R, "--power-dark", { opacity: "scrim" }, 1, d, "standard"),
    track("the black", BLACK, "opacity", 0, 1, "instant", "standard", d),
    track("the splash", SPLASH, "opacity", 0, 1, "instant", "standard", d),
    track("the sign", ART, "opacity", 1, 0, "instant", "standard"),
  ];
}

/**
 * Lock, from its look: the sheet sinks with the frame, the blur and the dim clear, and the clock
 * goes the rest of the way home, behind the city, its kicker coming in: the Lock's going back to rest.
 */
export function locking(reduced = false): NonEmpty<Eased> {
  if (reduced) return [...reducedRest(ON), track("the frame closing in", R, "--power-closein", 1, 0, "instant", "standard", "short")] as unknown as NonEmpty<Eased>;
  return [
    ...resting(ON),
    track("the frame closing in", R, "--power-closein", 1, 0, "medium", "emphasized_accel"),
    track("the kicker", KICKER, "opacity", 0, 1, "short", "standard", "medium"),
    track("the clock's depth", CLOCK, "--clock-above", 1, 0, "instant", "standard", "long"),
  ];
}

/** The apps that won't quit, coming into the sheet over the line: up `space.s24` and in, over `long`. */
export function blocking(reduced = false): NonEmpty<Eased> {
  if (reduced) return [track("the blocker", ".power-blocker", "opacity", 0, 1, "short", "standard")];
  return [track("the blocker", ".power-blocker", "opacity", 0, 1, "long", "emphasized_decel"), track("the blocker's rise", ".power-blocker", "--power-rise", 1, 0, "long", "emphasized_decel")];
}

// ─── The room ───────────────────────────────────────────────────────────────────────────────────

/**
 * The scale of the screen the sample's room stands for: `SCREEN` is a 1920 by 1200 laptop at 1.25.
 * The boot splash draws in device pixels, at 1 on a screen that short, so its sign and the field's
 * room under it are drawn at their device size, this much smaller in logical pixels.
 */
const SPLASH_SCALE = 1.25;

/** After a committed action: the lit sign over the black, `out` groups gone dark, or black alone. */
export type Dark = { readonly sign: number } | "black";

export interface RoomOptions {
  readonly situation?: Situation;
  /** The sheet up. */
  readonly open?: boolean;
  /** The tile the keyboard is on; -1 for none. */
  readonly at?: number;
  /** Its action played on the desktop: false as the menu opens, before the keyboard moves. */
  readonly played?: boolean;
  /** The tile pinned in a state, whatever the pointer does. */
  readonly pinned?: Pinned;
  /** How far a hold on the tile is, 0 to 1. */
  readonly held?: number;
  /** The apps that won't quit, up in the sheet. */
  readonly blocked?: boolean;
  /** Its tooltip shown. */
  readonly tip?: boolean;
  /** The line over the tiles, if not the action's own. */
  readonly words?: string;
  /** The tiles' names, if not the actions' own. */
  readonly names?: readonly string[];
  /** After a committed Restart or Shut down, or the black after any. */
  readonly dark?: Dark;
  /** The room at its rest after locking: the lock at rest. */
  readonly locked?: boolean;
  readonly live?: boolean;
}

interface Built {
  readonly element: HTMLElement;
  readonly tiles: readonly HTMLButtonElement[];
  readonly say: HTMLElement;
  readonly slot: HTMLElement;
  readonly blocker: HTMLElement;
  readonly blockerButtons: readonly HTMLButtonElement[];
  readonly quit: HTMLButtonElement;
  readonly timed: HTMLElement;
  readonly sheetParts: readonly HTMLElement[];
}

/** The line and its facts, for `index`. */
function sayFor(say: HTMLElement, index: number, situation: Situation, words?: string): void {
  const line = text(words ?? (index >= 0 ? lead(index, situation) : ""), "body_strong", "on_surface");
  line.classList.add("power-lead");
  const chips = row("s8", "centre", ...(index >= 0 ? facts(index, situation) : []));
  chips.classList.add("power-facts");
  say.replaceChildren(line, chips);
}

function windowOf(shown: Shown, index: number): HTMLElement {
  const corner = nested(role("surface"), "s8");
  const drawn = document.createElement("span");
  drawn.className = "k-lock-window power-window";
  drawn.dataset.at = String(index);
  drawn.style.setProperty("--corner", corner.css);
  drawn.append(row("s8", "centre", icon(shown.glyph, "var(--type-small-icon)", "var(--color-on_surface_variant)"), text(shown.title, "body_strong", "on_surface")), text(shown.line, "small", "on_surface_variant"));
  if (shown.blocks) {
    const why = warned(shown.blocks.chip, "warning");
    why.classList.add("power-why");
    drawn.append(why);
    const ring = focusRing(drawn, "surface", "warning", false);
    ring.style.setProperty("--corner", corner.css);
  }
  return drawn;
}

function tileOfAction(action: Action, name: string): HTMLButtonElement {
  const made = document.createElement("button");
  made.type = "button";
  made.className = "power-tile k-control";
  made.dataset.action = action.id;
  made.dataset.hot = "";
  const glyph = icon(action.icon, "var(--icon-s24)", "var(--power-glyph)");
  made.append(glyph, text(name, "button", "on_surface"));
  focusRing(made, "control", "accent", false).dataset.pressable = "";
  return made;
}

function signOf(): HTMLElement {
  const box = document.createElement("span");
  box.className = "power-sign";
  const layer = (src: string, cls: string) => {
    const img = document.createElement("img");
    img.src = src;
    img.alt = "";
    img.draggable = false;
    img.className = cls;
    return img;
  };
  const glass = layer(signBase, "power-sign-base");
  // The splash's picture is drawn at its 1× size, its @2× giving the pixels: its natural size, halved.
  const size = () => {
    if (glass.naturalWidth === 0) return;
    box.style.setProperty("--sign-device", String(glass.naturalWidth / 2));
    box.style.setProperty("--sign-ratio", `${glass.naturalWidth} / ${glass.naturalHeight}`);
  };
  glass.addEventListener("load", size);
  size();
  const src: Record<(typeof GROUPS)[number], string> = { ku: signKu, ba: signBa, dakuten: signDakuten, kuba: signKuba, os: signOs };
  box.append(glass, ...GROUPS.map((group) => layer(src[group], `power-sign-g power-sign-${group}`)));
  return box;
}

function build(options: RoomOptions): Built {
  const situation = options.situation ?? CALM;
  const names = options.names ?? ACTIONS.map((each) => each.name);
  const made = lock({ ...SCREEN, wallpaper: PURPLE_HOUR, phase: "desktop", bar: barButtons(false), windows: WINDOWS.map(windowOf), sheet: false });
  const element = made.element;
  element.classList.add("power-room");
  element.style.setProperty("--power-splash-scale", String(SPLASH_SCALE));

  const tiles = ACTIONS.map((action, i) => tileOfAction(action, names[i]!));
  const anchors = tiles.map((tile, i) => floatingTooltip(tile, tipOf(i), { side: "above", held: i >= RESTART ? "end" : "start", shown: options.tip === true && options.at === i }));
  const say = stack("s8", "start");
  say.classList.add("power-say");
  sayFor(say, options.at ?? -1, situation, options.words);

  // The blocker: the apps that won't quit, each with the way to it, and the Danger banner whose held
  // button quits them anyway, its timed bar along its foot.
  const shows: HTMLButtonElement[] = [];
  const rows = BLOCKING.map((at) => {
    const blocks = WINDOWS[at]!.blocks!;
    const show = button({ label: "Show", kind: "Text", size: "Sm" });
    shows.push(show);
    const named = stack(0, "start", text(blocks.app, "body_strong", "on_surface"), text(blocks.why, "small", "on_surface_variant"));
    const line = row("s12", "centre", iconTile({ glyph: WINDOWS[at]!.glyph, size: "Md" }), filling(named), show);
    line.classList.add("power-who");
    return line;
  });
  const who = banner({ tone: "Warning", icon: "warning", title: "2 apps won't quit", body: "", actions: stack("s4", "start", ...rows.map(filling)) });
  const bar = buttonBar({ actions: [{ label: "Quit anyway" }], tone: "Danger", size: "Sm" });
  const quit = bar.actions[0]!;
  const timed = progressBar({ fraction: 0, timed: true, danger: true });
  const red = banner({ tone: "Danger", icon: "warning", title: "Quit them anyway", body: "Zed's changes to main.rs are lost, and cargo build stops.", actions: bar.element, foot: timed });
  const blocker = stack("s8", "start", filling(who), filling(red));
  blocker.classList.add("power-blocker");
  const slot = document.createElement("span");
  slot.className = "power-slot";
  if (options.blocked) slot.append(blocker);

  const tilesRow = row("s8", "start", ...anchors);
  tilesRow.classList.add("power-tiles");
  const content = stack("s16", "start", slot, say, tilesRow);
  content.classList.add("power-content");
  const rising = sheet({ content, edge: "bottom", reveal: 0 });
  for (const part of rising.parts) part.classList.add("power-sheet");
  hang(made.screen, rising);

  const splash = document.createElement("span");
  splash.className = "power-splash";
  const black = document.createElement("span");
  black.className = "power-black";
  const room = document.createElement("span");
  room.className = "power-sign-room";
  splash.append(black, signOf(), room);
  element.append(splash);

  return { element, tiles, say, slot, blocker, blockerButtons: [...shows, quit], quit, timed, sheetParts: rising.parts };
}

/** Every number the room moves, as it stands on the desktop at rest with the menu shut. */
const REST: readonly (readonly [string, string, number | string])[] = [
  [R, "--framed", 1],
  [R, "--barred", 1],
  [R, "--push", 0],
  [R, "--power-closein", 0],
  [R, "--power-dark", 0],
  [BLURRED, "--wake", 0],
  [DIMMED, "--panel-dim", 0],
  [CLOCK, "--clock-lift", 0],
  [CLOCK, "--dock", 1],
  [CLOCK, "--clock-above", 1],
  [CLOCK, "opacity", 0],
  [BAR_TIME, "opacity", 1],
  [KICKER, "opacity", 0],
  [WINDOWS_BOX, "opacity", 1],
  [WINDOWS_BOX, "scale", 1],
  [".power-window", "opacity", 1],
  [".power-window", "--power-gone", 0],
  [".power-window", "--power-refuse", 0],
  [BAR, "opacity", 1],
  [FRAME, "opacity", 1],
  [SHEET, "--reveal", 0],
  [SHEET, "opacity", 1],
  [".power-tile", "--power-held", 0],
  [SPLASH, "opacity", 0],
  [BLACK, "opacity", 0],
  [ART, "opacity", 1],
  [GLASS, "opacity", 1],
  [".power-sign-g", "opacity", 1],
  [".power-blocker", "opacity", 1],
  [".power-blocker", "--power-rise", 0],
];

function targets(root: HTMLElement, selector: string): HTMLElement[] {
  return [...(root.matches(selector) ? [root] : []), ...root.querySelectorAll<HTMLElement>(selector)];
}

function rest(root: HTMLElement): void {
  for (const animation of root.getAnimations({ subtree: true })) if (!(animation instanceof CSSTransition)) animation.cancel();
  for (const [selector, property, value] of REST) for (const each of targets(root, selector)) each.style.setProperty(property, String(value));
}

/** `tracks` at their ends, at once: a still of where a move leaves the room. */
function settle(root: HTMLElement, tracks: readonly Track[]): void {
  const ordered = [...tracks].sort((a, b) => cueOf(a) + milliseconds(a.duration) - (cueOf(b) + milliseconds(b.duration)));
  for (const each of ordered) for (const element of targets(root, each.target)) element.style.setProperty(each.property, String(css(each.to)));
}

const reducedAt = (element: Element) => settings().reduced || element.closest("[data-reduced]") !== null;
const flickerOff = (element: Element) => !settings().flicker || element.closest("[data-flicker]") === null;
const ms = (duration: Duration) => milliseconds(duration) / settings().speed;
const curveOf = (easing: Easing) => `cubic-bezier(${bezier(easing).join(", ")})`;

/** Each element's running animations, by property: a new one takes over from where the old one stands. */
const running = new WeakMap<Element, Map<string, Animation>>();

/** The value `property` stands at on `element` now, mid-animation or not. */
function now(element: HTMLElement, property: string): string {
  const style = getComputedStyle(element);
  const value = property === "opacity" ? style.opacity : style.getPropertyValue(property).trim();
  return value === "" || value === "none" ? (property === "scale" ? "1" : "0") : value;
}

/** Eases `property` on `element` from where it stands to `to`: set at once, the animation the way there. */
function tween(element: HTMLElement, each: Track, delay: number): Promise<void> {
  const map = running.get(element) ?? new Map<string, Animation>();
  running.set(element, map);
  const key = each.property;
  const from = now(element, key);
  const to = String(css(each.to));
  map.get(key)?.cancel();
  element.style.setProperty(key, to);
  const frames: Keyframe[] = struck(each.curve)
    ? [...stepsOf(each.curve).map((step, i, all) => ({ offset: i / all.length, easing: "step-end", [key]: String(between(each, step)) })), { offset: 1, [key]: to }]
    : [{ [key]: from }, { [key]: to }];
  const animation = element.animate(frames, { duration: Math.max(1, ms(each.duration)), delay, easing: struck(each.curve) ? "linear" : curveOf(each.curve as Easing), fill: "backwards" });
  map.set(key, animation);
  return animation.finished.then(
    () => undefined,
    () => undefined,
  );
}

/**
 * Plays `tracks` on `root` from wherever each part stands, each to its `to` on its curve after its
 * cue: the live room's way of playing the page's moves. Tracks on one part and property play in cue
 * order, each from where the one before left it. A stepped track flickers through `neon.strike`'s
 * steps. A pressed tile is the state model's to draw, so its press isn't played. Resolves once every
 * track is done.
 */
function play(root: HTMLElement, tracks: readonly Track[]): Promise<void> {
  const chains = new Map<HTMLElement, Map<string, Track[]>>();
  for (const each of [...tracks].sort((a, b) => cueOf(a) - cueOf(b))) {
    if (each.property === "--press") continue;
    for (const element of targets(root, each.target)) {
      const byProperty = chains.get(element) ?? new Map<string, Track[]>();
      chains.set(element, byProperty);
      byProperty.set(each.property, [...(byProperty.get(each.property) ?? []), each]);
    }
  }
  const started = performance.now();
  const runs: Promise<void>[] = [];
  for (const [element, byProperty] of chains) {
    for (const list of byProperty.values()) {
      runs.push(
        (async () => {
          for (const each of list) {
            const at = cueOf(each) / settings().speed - (performance.now() - started);
            await tween(element, each, Math.max(0, at));
          }
        })(),
      );
    }
  }
  return Promise.all(runs).then(() => undefined);
}

const wait = (time: number) => new Promise<void>((done) => window.setTimeout(done, time));

/**
 * The mode the room is drawn in: its pane's, by day in the wallpaper's day scheme; locked, the
 * lock's night scheme whatever the mode, as the Lock always is.
 */
function paintFor(element: HTMLElement, locked = false): void {
  const pane = element.parentElement?.closest<HTMLElement>("[data-mode]");
  const mode = pane?.dataset.mode ?? settings().mode;
  if (locked || mode !== "light") {
    paintNight(element, PURPLE_HOUR.night);
    return;
  }
  paint(element, paletteOf(PURPLE_DAY, "light"));
  element.dataset.mode = "light";
}

/** The room, drawn still as `options` says, or live. */
export function room(options: RoomOptions = {}): HTMLElement {
  const built = build(options);
  const { element } = built;
  const situation = options.situation ?? CALM;
  rest(element);
  requestAnimationFrame(() => paintFor(element, options.locked === true));
  if (options.live) {
    liven(built, situation);
    return element;
  }
  const at = options.at ?? -1;
  if (options.open) {
    settle(element, opening());
    for (const part of built.sheetParts) part.setAttribute("data-power-settled", "");
    if (at >= 0) {
      const tile = built.tiles[at]!;
      tile.dataset.on = "";
      pin(tile, options.pinned);
      if (options.played !== false) settle(element, previewing(at, situation));
      if (at === RESTART && options.played !== false) settle(element, [0, 2, 4].map((i) => cycling()[i]!));
      if (options.held !== undefined) {
        tile.dataset.keyed = "";
        tile.style.setProperty("--power-held", String(options.held));
        const to = heldTo(at);
        if (to !== null) {
          const from = Number(css(dusk(at)));
          element.style.setProperty("--power-dark", String(from + (Number(css(to)) - from) * options.held));
        }
        if (options.words === undefined) sayFor(built.say, at, situation, HOLDING[at]);
      }
      if (options.blocked && options.words === undefined) sayFor(built.say, at, situation, WAITING);
    }
  }
  if (options.dark !== undefined) {
    settle(element, opening());
    settle(element, darkening("long", "standard", 0));
    element.querySelector<HTMLElement>(SPLASH)!.style.opacity = "1";
    element.querySelector<HTMLElement>(BLACK)!.style.opacity = "1";
    if (options.dark === "black") element.querySelector<HTMLElement>(ART)!.style.opacity = "0";
    else {
      const out = options.dark.sign;
      [...GROUPS].reverse().forEach((group, i) => (element.querySelector<HTMLElement>(`.power-sign-${group}`)!.style.opacity = i < out ? "0" : "1"));
      if (out > GROUPS.length) element.querySelector<HTMLElement>(GLASS)!.style.opacity = "0";
    }
  }
  if (options.locked) {
    settle(element, opening());
    settle(element, previewing(LOCK, situation));
    settle(element, locking());
  }
  return element;
}

/**
 * The room live: shut at first, the desktop at rest. A press on the desktop, or Super+Esc, opens
 * the menu; the arrows and Tab move along the tiles round, the pointer too, each playing
 * its action on the desktop; Enter or Space, or the tile's letter, acts, held for Log out, Restart
 * and Shut down. Esc closes it. Once it has gone dark, or locked, a key or a press brings the
 * desktop back.
 */
function liven(built: Built, situation: Situation): void {
  const { element, tiles } = built;
  element.tabIndex = 0;
  type Phase = "desk" | "open" | "locked" | "dark";
  let phase: Phase = "desk";
  let at = -1;
  let shown = false;
  let heldAt = -1;
  let busy = false;
  let cycle = 0;
  let blocked = false;
  const reduced = () => reducedAt(element);
  const pick = <T>(full: T, calm: T) => (reduced() ? calm : full);
  const parts = built.sheetParts;
  const settleSheet = () => {
    for (const part of parts) part.removeAttribute("data-power-settled");
    requestAnimationFrame(() => requestAnimationFrame(() => parts.forEach((part) => part.setAttribute("data-power-settled", ""))));
  };

  function stopCycle(): void {
    window.clearInterval(cycle);
    cycle = 0;
  }

  function preview(index: number, was = index): void {
    stopCycle();
    const wasLock = shown && was === LOCK;
    void play(element, [...(wasLock && index !== LOCK ? unpreviewing(reduced()) : []), ...previewing(index, situation, reduced())]);
    if (index === RESTART && !reduced()) {
      const once = () => void play(element, cycling());
      once();
      cycle = window.setInterval(once, cycleEvery() / settings().speed);
    }
  }

  function go(index: number, focus = true, play = true): void {
    if (index === at && (shown || !play)) return;
    const was = at;
    at = index;
    tiles.forEach((tile, i) => tile.toggleAttribute("data-on", i === index));
    if (focus) tiles[index]!.focus({ preventScroll: true });
    sayFor(built.say, index, situation);
    if (play) {
      preview(index, was);
      shown = true;
    }
  }

  async function open(): Promise<void> {
    if (phase !== "desk" || busy) return;
    phase = "open";
    busy = true;
    settleSheet();
    shown = false;
    at = -1;
    go(0, false, false);
    await play(element, pick(opening(), openingReduced()));
    busy = false;
    if (phase === "open" && at === 0) tiles[0]!.focus({ preventScroll: true });
  }

  async function close(): Promise<void> {
    if (phase !== "open") return;
    stopCycle();
    hideBlocker();
    phase = "desk";
    for (const part of parts) part.removeAttribute("data-power-settled");
    const back = shown ? [...(at === LOCK ? unpreviewing(reduced()) : []), ...previewing(-1, situation, reduced())] : [];
    shown = false;
    tiles.forEach((tile) => tile.removeAttribute("data-on"));
    at = -1;
    await play(element, [...back, ...pick(closing(), closingReduced())]);
    element.focus({ preventScroll: true });
  }

  function showBlocker(index: number): void {
    blocked = true;
    preview(index);
    sayFor(built.say, index, situation, WAITING);
    built.blocker.style.opacity = "0";
    built.blocker.style.setProperty("--power-rise", reduced() ? "0" : "1");
    built.slot.append(built.blocker);
    void play(element, blocking(reduced()));
    window.setTimeout(() => blocked && built.blockerButtons[0]!.focus({ focusVisible: true, preventScroll: true } as FocusOptions), reduced() ? 0 : ms("medium"));
  }

  function hideBlocker(): void {
    if (!blocked) return;
    blocked = false;
    built.blocker.remove();
    setProgress(built.timed, 0);
  }

  function goDark(): void {
    phase = "dark";
    tiles.forEach((tile) => tile.removeAttribute("data-on"));
  }

  async function proceed(index: number): Promise<void> {
    busy = true;
    stopCycle();
    hideBlocker();
    goDark();
    if (index >= RESTART) await play(element, reduced() ? goingDownReduced(index) : flickerOff(element) ? goingDownUnflickered(index) : goingDown(index));
    else await play(element, loggingOut(reduced()));
    busy = false;
  }

  function commit(index: number): void {
    if (situation.unsaved && index >= LOGOUT) showBlocker(index);
    else void proceed(index);
  }

  async function act(index: number): Promise<void> {
    if (busy) return;
    busy = true;
    if (index === LOCK) {
      if (at !== LOCK || !shown) {
        go(LOCK);
        await wait(ms("long"));
      }
      phase = "locked";
      paintFor(element, true);
      tiles.forEach((tile) => tile.removeAttribute("data-on"));
      await play(element, locking(reduced()));
    } else {
      goDark();
      await play(element, suspending(reduced()));
    }
    busy = false;
  }

  /** Back to the desktop from the dark: the desktop at rest under the black, which lifts. */
  async function reveal(): Promise<void> {
    busy = true;
    const splash = element.querySelector<HTMLElement>(SPLASH)!;
    rest(element);
    splash.style.opacity = "1";
    element.querySelector<HTMLElement>(BLACK)!.style.opacity = "1";
    element.querySelector<HTMLElement>(ART)!.style.opacity = "0";
    const box = element.querySelector<HTMLElement>(WINDOWS_BOX)!;
    box.style.setProperty("opacity", "0");
    if (!reduced()) box.style.setProperty("scale", String(css({ scale: "behind" })));
    tiles.forEach((tile) => tile.removeAttribute("data-on"));
    at = -1;
    await play(element, [
      track("the windows stepping in", WINDOWS_BOX, "scale", { scale: "behind" }, 1, reduced() ? "instant" : "long", "emphasized_decel", "short"),
      track("the windows", WINDOWS_BOX, "opacity", 0, 1, reduced() ? "short" : "long", "emphasized_decel", "short"),
      track("the dark", SPLASH, "opacity", 1, 0, reduced() ? "short" : "slow", "standard"),
    ]);
    rest(element);
    phase = "desk";
    busy = false;
    element.focus({ preventScroll: true });
  }

  async function unlock(): Promise<void> {
    busy = true;
    // The windows went one by one as the lock came; they come back together, from behind.
    for (const each of targets(element, ".power-window")) {
      each.style.setProperty("opacity", "1");
      each.style.setProperty("--power-gone", "0");
    }
    const box = element.querySelector<HTMLElement>(WINDOWS_BOX)!;
    box.style.setProperty("opacity", "0");
    box.style.setProperty("scale", String(css({ scale: "behind" })));
    element.style.setProperty("--power-dark", "0");
    paintFor(element);
    await play(element, pick(touching(ON), touchingReduced(ON)));
    rest(element);
    phase = "desk";
    busy = false;
    element.focus({ preventScroll: true });
  }

  // Holding a tile: its fill and the room's dark run on `hold`; let go early and they drain back.
  let holdRun: Promise<void> | null = null;
  let holdStart = 0;
  function startHold(index: number): void {
    if (heldAt >= 0 || busy || blocked) return;
    heldAt = index;
    stopCycle();
    const tile = tiles[index]!;
    tile.dataset.keyed = "";
    sayFor(built.say, index, situation, HOLDING[index]);
    holdStart = performance.now();
    const run = play(element, holding(index));
    holdRun = run;
    void run.then(() => {
      if (holdRun !== run || heldAt !== index) return;
      if (performance.now() - holdStart < ms("hold") - 1) return;
      heldAt = -1;
      tile.removeAttribute("data-keyed");
      tile.style.setProperty("--power-held", "0");
      commit(index);
    });
  }
  function stopHold(): void {
    if (heldAt < 0) return;
    const index = heldAt;
    heldAt = -1;
    holdRun = null;
    tiles[index]!.removeAttribute("data-keyed");
    sayFor(built.say, index, situation);
    void play(element, [track("the tile's fill", tileOf(index), "--power-held", 0, 0, "short", "standard"), track("the tile's press", tileOf(index), "--press", 1, 0, "micro", "standard")]);
    preview(index);
  }

  tiles.forEach((tile, i) => {
    tile.addEventListener("pointerenter", () => {
      if (phase === "open" && heldAt < 0 && !blocked && !busy) go(i, false);
    });
    tile.addEventListener("pointerdown", (event) => {
      if (phase !== "open" || blocked) return;
      event.preventDefault();
      event.stopPropagation();
      if (i !== at) go(i, false);
      if (ACTIONS[i]!.held) startHold(i);
    });
    tile.addEventListener("pointerup", () => stopHold());
    tile.addEventListener("pointerleave", () => stopHold());
    tile.addEventListener("click", (event) => {
      event.stopPropagation();
      if (phase === "open" && !ACTIONS[i]!.held) void act(i);
    });
  });

  // The blocker: Show goes to the app (the menu closes); Quit anyway is held over `hold`.
  built.blockerButtons.slice(0, -1).forEach((show) =>
    show.addEventListener("click", (event) => {
      event.stopPropagation();
      void close();
    }),
  );
  let quitFrame = 0;
  let quitStart = 0;
  const quitTick = (time: number) => {
    const fraction = Math.min(1, (time - quitStart) / ms("hold"));
    setProgress(built.timed, fraction);
    if (fraction < 1) quitFrame = requestAnimationFrame(quitTick);
    else {
      quitFrame = 0;
      void proceed(at);
    }
  };
  const quitBegin = () => {
    if (quitFrame) return;
    quitStart = performance.now();
    quitFrame = requestAnimationFrame(quitTick);
  };
  const quitEnd = () => {
    if (!quitFrame) return;
    cancelAnimationFrame(quitFrame);
    quitFrame = 0;
    setProgress(built.timed, 0);
  };
  built.quit.addEventListener("pointerdown", (event) => {
    event.stopPropagation();
    quitBegin();
  });
  for (const name of ["pointerup", "pointerleave", "blur"]) built.quit.addEventListener(name, quitEnd);
  built.quit.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    event.stopPropagation();
    if (!event.repeat) quitBegin();
  });
  built.quit.addEventListener("keyup", (event) => {
    if (event.key === "Enter" || event.key === " ") quitEnd();
  });

  element.addEventListener("keydown", (event) => {
    if (phase === "dark") {
      event.preventDefault();
      if (!busy) void reveal();
      return;
    }
    if (phase === "locked") {
      event.preventDefault();
      if (!busy) void unlock();
      return;
    }
    if (phase === "desk") {
      if (event.key === "Escape" && event.metaKey) {
        event.preventDefault();
        void open();
      }
      return;
    }
    if (blocked) {
      const buttons = built.blockerButtons;
      const here = buttons.indexOf(document.activeElement as HTMLButtonElement);
      const next = ["ArrowDown", "ArrowRight", "j", "l"].includes(event.key) || (event.key === "Tab" && !event.shiftKey);
      const back = ["ArrowUp", "ArrowLeft", "k", "h"].includes(event.key) || (event.key === "Tab" && event.shiftKey);
      if (event.key === "Escape") {
        event.preventDefault();
        hideBlocker();
        sayFor(built.say, at, situation);
        preview(at);
        tiles[at]!.focus({ preventScroll: true });
      } else if (next || back) {
        event.preventDefault();
        buttons[(here + (next ? 1 : -1) + buttons.length) % buttons.length]!.focus();
      }
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      if (event.metaKey || !busy) void close();
      return;
    }
    // The letters are the actions' own (L is Lock), so only the arrows and Tab move.
    const by = event.key === "ArrowRight" || (event.key === "Tab" && !event.shiftKey) ? 1 : event.key === "ArrowLeft" || (event.key === "Tab" && event.shiftKey) ? -1 : 0;
    if (by !== 0) {
      event.preventDefault();
      if (heldAt < 0 && !busy) go((Math.max(at, 0) + by + ACTIONS.length) % ACTIONS.length);
      return;
    }
    const lettered = ACTIONS.findIndex((each) => each.key.toLowerCase() === event.key.toLowerCase());
    if (lettered >= 0 || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (event.repeat || busy) return;
      const index = lettered >= 0 ? lettered : at;
      go(index);
      if (ACTIONS[index]!.held) startHold(index);
      else void act(index);
    }
  });
  element.addEventListener("keyup", (event) => {
    const lettered = ACTIONS.findIndex((each) => each.key.toLowerCase() === event.key.toLowerCase());
    if (lettered >= 0 || event.key === "Enter" || event.key === " ") stopHold();
  });
  element.addEventListener("click", (event) => {
    if (busy) return;
    if (phase === "dark") void reveal();
    else if (phase === "locked") void unlock();
    else if (phase === "desk") void open();
    else if (!built.sheetParts[1]!.contains(event.target as Node)) void close();
  });
}
