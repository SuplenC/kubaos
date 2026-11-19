// Boot, from the boot menu to the session (`boot.ts` is its page): the screens GRUB and the boot
// splash draw, each a whole screen at its device size, the splash's sign in its exact centre on
// every one of them and everything else under it. GRUB's menu is the sign unlit with the entries
// under it; choosing one takes the menu away and leaves the sign; the splash switches it on a group
// at a time with the passphrase's field already under it; once the disk is open the lit sign
// fades straight into the lock's rest clock, and with the same password the lock opens at once.
//
// The screens are built from the library (ListRow, Field, Keycap, KeyHints) and the splash's own
// pictures (`neon-sign.ts`); the moves are data, and the live sample plays the same tracks.

import "./boot.css";
import type { Words } from "../../lib/content.ts";
import { field, fieldRoom } from "../../lib/field.ts";
import { inline, keycap, keyHints, type KeyHint } from "../../lib/keycap.ts";
import { listRow } from "../../lib/list.ts";
import { stack } from "../../lib/layout.ts";
import { clamped, text } from "../../lib/text.ts";
import { durationMs } from "../../lib/tokens.ts";
import type { Duration, Easing, NonEmpty, Track } from "../../site/page.ts";
import { struck } from "../../site/recipe.ts";
import { paint, paletteOf } from "../../site/recolour.ts";
import { settings } from "../../site/settings.ts";
import { still } from "../templates/locks.ts";
import { ON, touching, touchingReduced } from "../templates/lock-moves.ts";
import { halt, play, settle, unsettle } from "./live-tracks.ts";
import { GROUPS, groupOf, light, neonRoom } from "./neon-sign.ts";

type Eased = Track<Easing>;

// ─── What the screens say ───────────────────────────────────────────────────────────────────────

/** GRUB's entries, as `10_linux` names them from os-release. */
export const ENTRIES: readonly string[] = ["KubaOS Shinjuku", "Advanced options for KubaOS Shinjuku", "KubaOS snapshots", "UEFI Firmware Settings"];

/** A long entry: a recovery kernel's, as Advanced options lists it. */
export const LONG_ENTRY: Words = { short: "KubaOS Shinjuku", long: "KubaOS Shinjuku, with Linux 6.17.0-14-generic (recovery mode)" };

/** Many entries: Advanced options opened, each kernel twice. */
export const MANY: readonly string[] = [
  "KubaOS Shinjuku, with Linux 6.17.0-14-generic",
  "KubaOS Shinjuku, with Linux 6.17.0-14-generic (recovery mode)",
  "KubaOS Shinjuku, with Linux 6.17.0-12-generic",
  "KubaOS Shinjuku, with Linux 6.17.0-12-generic (recovery mode)",
  "KubaOS Shinjuku, with Linux 6.14.0-37-generic",
  "KubaOS Shinjuku, with Linux 6.14.0-37-generic (recovery mode)",
  "KubaOS Shinjuku, with Linux 6.8.0-90-generic",
  "KubaOS Shinjuku, with Linux 6.8.0-90-generic (recovery mode)",
];

/** The rows GRUB's menu has room for (`theme.txt.in`'s `boot_menu` height); more scroll in place. */
export const MENU_ROWS = 4;

/** GRUB's keys: `branding/boot.py`'s GRUB_HINTS. */
export const GRUB_HINTS: readonly KeyHint[] = [
  ["↑↓", "Pick"],
  ["Enter", "Boot"],
  ["E", "Edit the entry"],
  ["C", "GRUB shell"],
];

/** The countdown after a boot that didn't make it up (`GRUB_RECORDFAIL_TIMEOUT`). */
export const COUNTDOWN = 10;

export const PLACEHOLDER = "Disk passphrase";
/** The hint under the empty field, its key a keycap: `branding/boot.py`'s HINT. */
export const HINT: Words = { short: "The one you set when you installed KubaOS.", long: "The passphrase you chose for this disk when you installed KubaOS." };
export const WRONG = "Wrong passphrase. Try again.";
export const BUSY = "Unlocking…";

/** The passphrase the live sample opens with. */
export const PASSPHRASE = "kuba";

// ─── Screens ────────────────────────────────────────────────────────────────────────────────────

/**
 * A screen the sequence is drawn on, in logical pixels, and the scale the splash picks its
 * pictures at there: the sign's device size over it is the same on each.
 */
export interface Display {
  readonly name: string;
  readonly width: number;
  readonly height: number;
}

/** A 24-inch monitor at 1: the screen every still is drawn on. */
export const MONITOR: Display = { name: "Monitor, 1920 × 1080 at 1", width: 1920, height: 1080 };
/** The laptop's 2880 × 1800 panel at 2, where the splash takes its @2× pictures. */
export const LAPTOP: Display = { name: "Laptop, 2880 × 1800 at 2", width: 1440, height: 900 };

/** What a screen shows: GRUB's menu, the entry chosen, the splash, black, or GRUB's text screens. */
export type Scene = "menu" | "chosen" | "splash" | "black" | "shell" | "edit";

/** The passphrase's prompt under the sign. */
export type Prompt = "none" | "empty" | "typing" | "caps" | "wrong" | "busy";

export interface Drawn {
  readonly scene?: Scene;
  readonly display?: Display;
  /** How many of the sign's groups burn. */
  readonly lit?: number;
  readonly prompt?: Prompt;
  /** The characters typed. */
  readonly typed?: number;
  readonly entries?: readonly string[];
  /** The entry the keyboard is on. */
  readonly at?: number;
  /** Seconds left on the countdown; null once a key has stopped it. */
  readonly countdown?: number | null;
  /** The hint's words, if not its own. */
  readonly hint?: string;
  /** The lock at rest under the splash, for the splash to fade into. */
  readonly lock?: boolean;
}

export interface Screen {
  readonly element: HTMLElement;
  readonly menu: HTMLElement;
  readonly countdown: HTMLElement;
  readonly prompt: HTMLElement;
}

const px = (n: number) => `calc(${n} * var(--px))`;

/** GRUB's menu: a ListRow an entry, `space.s4` apart, the one the keyboard is on lit, four in view. */
export function drawMenu(menu: HTMLElement, entries: readonly string[], at: number): void {
  const first = Math.max(0, Math.min(at - MENU_ROWS + 1, entries.length - MENU_ROWS));
  menu.replaceChildren(...entries.slice(first, first + MENU_ROWS).map((title, i) => listRow({ title, named: true }, first + i === at, first + i === at ? "lit" : undefined)));
}

/** The line under the field: `small` words and keycaps, a keycap tall, so swapping one moves nothing. */
function noteOf(prompt: Prompt, hint: string): HTMLElement {
  const muted = (words: string) => text(words, "small", "on_surface_variant");
  const line =
    prompt === "caps"
      ? inline(keycap("Caps Lock"), muted("is on."))
      : prompt === "wrong"
        ? text(WRONG, "small", "error")
        : prompt === "busy"
          ? muted(BUSY)
          : inline(clamped(hint, "small", "on_surface_variant"), keycap("Enter"), muted("unlocks."));
  const box = document.createElement("span");
  box.className = "boot-note";
  box.append(line);
  return box;
}

/**
 * The passphrase's field and its note, `space.s8` apart: the library's Field, `size.password` wide,
 * the keyboard in it. Unlocking, it is read-only with an hourglass and no ring, its bullets muted;
 * a wrong passphrase empties it and turns it Invalid.
 */
export function drawPrompt(room: HTMLElement, prompt: Prompt, typed = 0, hint = HINT.short): void {
  room.dataset.prompt = prompt;
  if (prompt === "none") {
    room.replaceChildren();
    return;
  }
  const busy = prompt === "busy";
  const value = prompt === "wrong" || prompt === "empty" ? "" : "x".repeat(Math.max(typed, 1));
  const made = field({
    icon: busy ? "hourglass_top" : "lock",
    placeholder: PLACEHOLDER,
    secret: true,
    value,
    ...(busy ? { state: "Locked" as const } : { pinned: "lit" as const }),
    ...(prompt === "wrong" ? { state: "Invalid" as const } : {}),
  });
  const box = fieldRoom("var(--size-password)", made);
  box.classList.add("boot-field");
  room.replaceChildren(stack("s8", "centre", box, noteOf(prompt, hint)));
}

/**
 * GRUB's text screens, as today's theme draws them: the terminal's box, a hairline at the surface
 * radius, `8%` in and `84%` wide, in `mono`, with the menu's key hints under it.
 */
function terminal(scene: "shell" | "edit"): HTMLElement {
  const box = document.createElement("span");
  box.className = "boot-terminal";
  const line = (words: string, cls = "") => {
    const made = text(words, "mono", "on_surface");
    made.classList.add("boot-terminal-line");
    if (cls) made.classList.add(cls);
    return made;
  };
  if (scene === "shell") {
    box.append(
      line("GNU GRUB  version 2.12", "boot-terminal-head"),
      line("   Minimal BASH-like line editing is supported. For the first word, TAB lists possible command completions. Anywhere else TAB lists possible device or file completions. To enable less(1)-like paging, \"set pager=1\". ESC at any time exits."),
      line(""),
      line("grub> _"),
    );
  } else {
    box.append(
      line("GNU GRUB  version 2.12", "boot-terminal-head"),
      line("setparams 'KubaOS Shinjuku'"),
      line(""),
      line("        recordfail"),
      line("        load_video"),
      line("        gfxmode $linux_gfx_mode"),
      line("        insmod gzio"),
      line("        insmod part_gpt"),
      line("        insmod btrfs"),
      line("        search --no-floppy --fs-uuid --set=root 3f2a9c1e-5b7d-4e0a-9c42-8d1f6e2b7a90"),
      line("        linux /@/boot/vmlinuz-6.17.0-14-generic root=/dev/mapper/cryptroot ro rootflags=subvol=@ quiet splash $vt_handoff"),
      line("        initrd /@/boot/initrd.img-6.17.0-14-generic"),
      line(""),
      line("      Minimum Emacs-like screen editing is supported. TAB lists completions. Press Ctrl-x or F10 to boot, Ctrl-c or F2 for a command-line or ESC to discard edits and return to the GRUB menu."),
    );
  }
  return box;
}

/** One screen of the sequence, still, as `drawn` says. */
export function screen(drawn: Drawn = {}): Screen {
  const scene = drawn.scene ?? "menu";
  const display = drawn.display ?? MONITOR;
  const element = document.createElement("span");
  element.className = "boot-screen";
  element.dataset.scene = scene;
  element.style.setProperty("--boot-w", px(display.width));
  element.style.setProperty("--boot-h", px(display.height));
  if (drawn.lock) {
    const under = still({ phase: "rest", size: { width: px(display.width), height: px(display.height) } });
    under.classList.add("boot-lock");
    element.append(under);
  }
  // Every boot screen draws in the tokens' own dark palette, whatever the theme or the mode.
  const splash = document.createElement("span");
  splash.className = "boot-splash";
  splash.dataset.mode = "dark";
  paint(splash, paletteOf({}, "dark"));
  const menu = document.createElement("span");
  menu.className = "boot-menu";
  drawMenu(menu, drawn.entries ?? ENTRIES, drawn.at ?? 0);
  const prompt = document.createElement("span");
  prompt.className = "boot-prompt";
  drawPrompt(prompt, drawn.prompt ?? "none", drawn.typed ?? 0, drawn.hint);
  const countdown = text(`Booting in ${drawn.countdown ?? COUNTDOWN} s. Any key stops the clock.`, "small", "on_surface_variant");
  countdown.classList.add("boot-countdown");
  if (drawn.countdown === null) countdown.style.visibility = "hidden";
  const foot = document.createElement("span");
  foot.className = "boot-foot";
  foot.append(countdown, keyHints(GRUB_HINTS));
  const lit = scene === "menu" || scene === "chosen" ? 0 : (drawn.lit ?? GROUPS.length);
  const room = neonRoom(lit, menu, prompt);
  splash.append(room, foot);
  if (scene === "shell" || scene === "edit") {
    room.style.opacity = "0";
    menu.style.opacity = "0";
    countdown.style.visibility = "hidden";
    splash.append(terminal(scene));
  }
  // The splash draws no menu: the field stands right under the sign, not under the menu's room.
  if (scene === "splash" || scene === "shell" || scene === "edit") menu.replaceChildren();
  if (scene !== "menu") {
    menu.style.opacity = "0";
    if (scene !== "shell" && scene !== "edit") foot.style.opacity = "0";
  }
  if (scene === "black") room.style.opacity = "0";
  element.append(splash);
  return { element, menu, countdown, prompt };
}

/** Several screens side by side, `space.s32` apart, each named under it. */
export function screens(...each: readonly HTMLElement[]): HTMLElement {
  const box = document.createElement("span");
  box.className = "boot-screens";
  box.append(...each);
  return box;
}

/** A screen named under it, for a sample of several. */
export function named(display: Display, drawn: HTMLElement): HTMLElement {
  return stack("s12", "centre", drawn, text(display.name, "label", "on_surface_variant"));
}

// ─── Moves, as data ─────────────────────────────────────────────────────────────────────────────

export const SCREEN_PART = "the boot screen";
const MENU = ".boot-menu";
const FOOT = ".boot-foot";
const PROMPT = ".boot-prompt";
const FIELD = ".boot-field";
const SPLASH = ".boot-splash";

/** A cue of `phases`, or none for a part that starts with the move. */
function cue(phases: readonly Duration[]): Pick<Track, "cue"> {
  return phases.length === 0 ? {} : { cue: phases as unknown as NonEmpty<Duration> };
}

function track(part: string, target: string, property: string, from: Track["from"], to: Track["to"], duration: Duration, curve: Easing, ...at: Duration[]): Eased {
  return { part, on: SCREEN_PART, target, property, from, to, duration, curve, ...cue(at) };
}

/** `tracks` played after `phases`, each cue moved on by them. */
function after(tracks: readonly Eased[], ...phases: Duration[]): Eased[] {
  return tracks.map((each) => {
    const own: Duration[] = each.cue === undefined ? [] : typeof each.cue === "string" ? [each.cue] : [...each.cue];
    return { ...each, ...cue([...phases, ...own]) };
  });
}

/**
 * When each group strikes, as phases: the splash's STRIKE_AT (200, 533, 1000, 1333 and 1800 ms)
 * on the duration tokens, ク a `short` in, OS a `scene` and two `medium`s.
 */
export const STRIKES: Record<(typeof GROUPS)[number], readonly Duration[]> = {
  ku: ["short"],
  ba: ["short", "medium"],
  dakuten: ["long", "long"],
  kuba: ["long", "long", "medium"],
  os: ["scene", "medium", "medium"],
};

/** Choosing an entry: GRUB clears its menu, its countdown and its hints at once; the unlit sign stays. */
export function choosing(): NonEmpty<Eased> {
  return [track("the menu", MENU, "opacity", 1, 0, "instant", "standard"), track("the countdown and hints", FOOT, "opacity", 1, 0, "instant", "standard")];
}

/**
 * Starting: the splash takes over the unlit sign where it stands, the field already under it, and
 * the sign comes on a group at a time, each through `neon.strike` over `medium`.
 */
export function starting(encrypted: boolean): NonEmpty<Track> {
  return [
    ...(encrypted ? [track("the field", PROMPT, "opacity", 0, 1, "instant", "standard")] : []),
    ...GROUPS.map((group) => ({ part: `${group}'s tubes`, on: SCREEN_PART, target: groupOf(group), property: "opacity", from: 0, to: 1, duration: "medium" as const, curve: "neon.strike" as const, ...cue(STRIKES[group]) })),
  ] as unknown as NonEmpty<Track>;
}

/** With the flicker off: each group fades on over `short` where it would strike. */
export function startingUnflickered(encrypted: boolean): NonEmpty<Eased> {
  return starting(encrypted).map((each) => (struck(each.curve) ? { ...each, duration: "short" as const, curve: "standard" as const } : each)) as unknown as NonEmpty<Eased>;
}

/** With reduce motion: the whole sign fades on over `short`. */
export function startingReduced(encrypted: boolean): NonEmpty<Eased> {
  return [...(encrypted ? [track("the field", PROMPT, "opacity", 0, 1, "instant", "standard")] : []), ...GROUPS.map((group) => track(`${group}'s tubes`, groupOf(group), "opacity", 0, 1, "short", "standard"))] as unknown as NonEmpty<Eased>;
}

/** A wrong passphrase: the field shakes as the lock's does, `space.s12`, back `space.s8`, `space.s4`, home. */
export function shaking(): NonEmpty<Eased> {
  return [
    track("the field's shake", FIELD, "--jolt", 0, 3, "micro", "standard"),
    track("the field's shake", FIELD, "--jolt", 3, -2, "micro", "standard", "micro"),
    track("the field's shake", FIELD, "--jolt", -2, 1, "micro", "standard", "micro", "micro"),
    track("the field's shake", FIELD, "--jolt", 1, 0, "short", "standard_decel", "micro", "micro", "micro"),
  ];
}

/**
 * Unlocked: the field fades over `medium`, the lit sign holds a `long`, then the splash fades over
 * `slow` straight into the lock's rest clock under it, no black between. Without a field, the
 * hold starts once the sign is lit.
 */
export function intoTheLock(encrypted: boolean, reduced = false): NonEmpty<Eased> {
  if (reduced) {
    return [
      ...(encrypted ? [track("the field", PROMPT, "opacity", 1, 0, "short", "standard")] : []),
      track("the splash", SPLASH, "opacity", 1, 0, "short", "standard", ...(encrypted ? (["short"] as Duration[]) : []), "long"),
    ] as unknown as NonEmpty<Eased>;
  }
  return [
    ...(encrypted ? [track("the field", PROMPT, "opacity", 1, 0, "medium", "standard_accel")] : []),
    track("the splash", SPLASH, "opacity", 1, 0, "slow", "standard", ...(encrypted ? (["medium"] as Duration[]) : []), "long"),
  ] as unknown as NonEmpty<Eased>;
}

/** The same password: the lock plays its own unlock from rest at once, and the desktop is there. */
export function straightIn(reduced = false): NonEmpty<Eased> {
  const into = intoTheLock(true, reduced);
  const phases: Duration[] = reduced ? ["short", "long", "short"] : ["medium", "long", "slow"];
  return [...into, ...after(reduced ? touchingReduced(ON) : touching(ON), ...phases)] as unknown as NonEmpty<Eased>;
}

/** Restart: black on every screen while the machine is off, then GRUB draws the menu at once. */
export function restarting(): NonEmpty<Eased> {
  return [track("the sign", ".neon-room", "opacity", 0, 1, "instant", "standard", "scene"), track("the menu", MENU, "opacity", 0, 1, "instant", "standard", "scene"), track("the countdown and hints", FOOT, "opacity", 0, 1, "instant", "standard", "scene")];
}

/** `draw` drawn where `tracks` begin, wrapped so a move's player reaches its root. */
export function from(tracks: readonly Track[], draw: () => HTMLElement): () => HTMLElement {
  return () => {
    const box = document.createElement("span");
    box.className = "boot-held";
    const made = draw();
    unsettle(made, tracks);
    box.append(made);
    return box;
  };
}

// ─── Live ───────────────────────────────────────────────────────────────────────────────────────

export interface World {
  readonly encrypted: boolean;
  /** The session's password is the disk passphrase. */
  readonly same: boolean;
  readonly displays: readonly Display[];
}

const reducedAt = (element: Element) => settings().reduced || element.closest("[data-reduced]") !== null;
const flickerOff = (element: Element) => !settings().flicker || element.closest("[data-flicker]") === null;
const wait = (name: Duration) => new Promise<void>((done) => window.setTimeout(done, durationMs(name) / settings().speed));

/**
 * The sequence live, from the boot menu: ↑ ↓ pick, Enter boots, E and C open GRUB's text screens
 * and Esc leaves them. The splash takes the passphrase (`kuba` opens it; anything else is wrong),
 * then the lock; with another password a key stands in for typing it. On the desktop, a key
 * restarts. The clock is stopped, as after a key.
 */
export function liveBoot(world: World): HTMLElement {
  const root = document.createElement("span");
  root.className = "boot-live";
  root.tabIndex = 0;
  type Phase = "menu" | "shell" | "edit" | "starting" | "prompt" | "busy" | "lock" | "desktop" | "moving";
  let phase: Phase = "menu";
  let at = 0;
  let typed = "";
  let shown: Screen[] = [];
  const primary = () => shown[0]!;

  function draw(scene: Scene): void {
    halt(root);
    shown = world.displays.map((display) => screen({ scene, display, countdown: null, at, lock: true }));
    const drawn = shown.map((each) => each.element);
    root.replaceChildren(drawn.length === 1 ? drawn[0]! : screens(...drawn.map((each, i) => named(world.displays[i]!, each))));
  }

  function menu(): void {
    phase = "menu";
    draw("menu");
  }

  async function boot(): Promise<void> {
    phase = "moving";
    settle(root, choosing());
    // A stand-in for the kernel loading: the unlit sign alone, nothing moving.
    await wait("long");
    light(root, 0);
    for (const each of shown) each.menu.replaceChildren();
    if (world.encrypted) drawPrompt(primary().prompt, "empty");
    phase = world.encrypted ? "prompt" : "starting";
    typed = "";
    const reduced = reducedAt(root);
    await play(root, reduced ? startingReduced(world.encrypted) : flickerOff(root) ? startingUnflickered(world.encrypted) : starting(world.encrypted));
    if (!world.encrypted) await intoLock();
  }

  async function intoLock(): Promise<void> {
    phase = "moving";
    const reduced = reducedAt(root);
    if (world.encrypted && world.same) {
      await play(root, straightIn(reduced));
      phase = "desktop";
      return;
    }
    await play(root, intoTheLock(world.encrypted, reduced));
    phase = "lock";
  }

  async function enter(): Promise<void> {
    if (typed === "") return;
    phase = "busy";
    drawPrompt(primary().prompt, "busy", typed.length);
    await wait("slow");
    if (typed === PASSPHRASE) {
      await intoLock();
      return;
    }
    typed = "";
    drawPrompt(primary().prompt, "wrong");
    phase = "prompt";
    if (!reducedAt(root)) void play(root, shaking());
  }

  async function unlock(): Promise<void> {
    phase = "moving";
    await play(root, reducedAt(root) ? touchingReduced(ON) : touching(ON));
    phase = "desktop";
  }

  async function restart(): Promise<void> {
    phase = "moving";
    draw("black");
    await play(root, restarting());
    at = 0;
    phase = "menu";
  }

  root.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key.toLowerCase() === "x" && phase === "edit") {
      event.preventDefault();
      void boot();
      return;
    }
    if (event.metaKey || event.altKey || event.ctrlKey || event.key === "Tab") return;
    switch (phase) {
      case "menu": {
        event.preventDefault();
        const entries = ENTRIES.length;
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          at = Math.max(0, Math.min(entries - 1, at + (event.key === "ArrowDown" ? 1 : -1)));
          for (const each of shown) drawMenu(each.menu, ENTRIES, at);
        } else if (event.key === "Enter") void boot();
        else if (event.key.toLowerCase() === "e" || event.key.toLowerCase() === "c") {
          phase = event.key.toLowerCase() === "e" ? "edit" : "shell";
          draw(phase);
        }
        return;
      }
      case "shell":
      case "edit":
        if (event.key === "Escape" || event.key === "F10") {
          event.preventDefault();
          if (event.key === "F10" && phase === "edit") void boot();
          else menu();
        }
        return;
      case "prompt": {
        event.preventDefault();
        if (event.key === "Enter") {
          void enter();
          return;
        }
        if (event.key === "Backspace") typed = typed.slice(0, -1);
        else if (event.key.length === 1) typed += event.key;
        else if (event.key !== "CapsLock") return;
        const caps = event.getModifierState("CapsLock");
        drawPrompt(primary().prompt, typed === "" ? (caps ? "caps" : "empty") : caps ? "caps" : "typing", typed.length);
        return;
      }
      case "lock":
        event.preventDefault();
        void unlock();
        return;
      case "desktop":
        event.preventDefault();
        void restart();
        return;
      default:
        event.preventDefault();
    }
  });
  root.addEventListener("click", () => {
    root.focus({ preventScroll: true });
    if (phase === "lock") void unlock();
    else if (phase === "desktop") void restart();
  });
  menu();
  return root;
}
