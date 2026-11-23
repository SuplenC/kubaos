// The media player: mpv, KubaOS's one player for video and audio, with a controller of KubaOS's
// own drawn over it. A DRAFT: its window is borderless, its controller shares the control
// center's media widget's layout, and a song plays in the same window as a film; it shows every
// state the player has. Its draft parts are in `media-player-parts.ts`.

import { label } from "../../site/dom.ts";
import { stageControls } from "../../site/parts/stage.ts";
import { doesntApply, type Move, nonEmpty, page, screen } from "../../site/page.ts";
import { CHROME, settings } from "../../site/settings.ts";
import { glassWindow } from "../../lib/glass.ts";
import { tabs } from "../../lib/tabs.ts";
import { row, stack } from "../../lib/layout.ts";
import { durationMs, raw } from "../../lib/tokens.ts";
import { barScreen } from "../shell/bar-parts.ts";
import { SCREEN_WIDTH, TALL } from "../shell/screen-parts.ts";
import {
  AUDIO_TRACKS,
  BARE_SONG,
  BROKEN,
  DESKTOP,
  FILM,
  LINES,
  LONG_TITLE,
  MESSAGES,
  type MediaFile,
  type Message,
  PLAYLIST,
  type PlayerState,
  player,
  RUNGS,
  SONG,
  SONGS,
  SPEEDS,
  STREAM,
  SUBTITLES,
  widget,
  type Colours,
} from "./media-player-parts.ts";
import { CHOICES, type VizChoice, vizName } from "./media-player-viz.ts";
import { bareWindow } from "./settings/settings-window.ts";
import { song as analysedSong } from "./media-player-song.ts";

// The states, as data.

const PLAYING: PlayerState = { file: FILM, playing: true, position: 0.34, buffered: 0.52, volume: 0.62, speed: 1, index: 0, count: 1, subtitle: null };
const SHOWN: PlayerState = { ...PLAYING, shown: true };
const PAUSED: PlayerState = { ...PLAYING, playing: false };
const IN_LIST: PlayerState = { ...SHOWN, index: 1, count: PLAYLIST.length };
const SONG_PLAYING: PlayerState = { ...PLAYING, file: SONG, position: 0.4 };
const BARE_PLAYING: PlayerState = { ...PLAYING, file: BARE_SONG, position: 0.12 };

/** The player tiled alone on the desktop, as Hyprland tiles it. A whole screen. */
const desk = screen((state: PlayerState): HTMLElement => {
  const room = barScreen(SCREEN_WIDTH, TALL);
  const slot = document.createElement("span");
  slot.className = "mp-tiled";
  slot.append(player(state).element);
  room.element.querySelector(".k-screen-dim")!.before(slot);
  // The desktop behind is one of the pictures the player already shows, so every sample shares
  // the few pictures the page decodes.
  room.element.querySelector<HTMLImageElement>(".k-screen-wallpaper")!.src = DESKTOP;
  return room.element;
});

/**
 * The player tiled beside Settings, `size.gap` apart and from the bar and the screen's edges:
 * Settings is the active window, its tube lit; the player keeps only the glass's ring.
 */
const beside = screen((state: PlayerState): HTMLElement => {
  const room = barScreen(SCREEN_WIDTH, TALL);
  const tiles = document.createElement("span");
  tiles.className = "mp-tiles";
  const active = glassWindow(document.createElement("span"), true);
  active.append(bareWindow());
  const shown = player(state).element;
  shown.removeAttribute("data-active");
  tiles.append(active, shown);
  room.element.querySelector(".k-screen-dim")!.before(tiles);
  room.element.querySelector<HTMLImageElement>(".k-screen-wallpaper")!.src = DESKTOP;
  return room.element;
});

/** The player covering the whole screen, the bar under it. */
const whole = screen((state: PlayerState): HTMLElement => {
  const room = document.createElement("span");
  room.className = "mp-whole";
  room.dataset.whole = "";
  room.style.setProperty("--screen-width", `calc(${SCREEN_WIDTH} * var(--px))`);
  room.style.setProperty("--screen-height", TALL);
  room.append(player({ ...state, full: true }).element);
  return room;
});

/** The player in a window `width` by `height` (lengths' CSS). */
function windowed(state: PlayerState, width: string, height: string): HTMLElement {
  const room = document.createElement("span");
  room.className = "mp-room";
  room.style.width = width;
  room.style.height = height;
  room.append(player(state).element);
  return room;
}

const DEFAULT_WIDTH = "var(--size-app_width)";
const DEFAULT_HEIGHT = "calc(var(--size-app_width) / (var(--aspect-wallpaper)))";
const LEAST_WIDTH = "var(--size-app_min_width)";
const LEAST_HEIGHT = "var(--size-app_min_height)";

/** The controller beside the control center's widget, whose layout it shares, in the desktop's mode. */
function shares(): HTMLElement {
  const state = IN_LIST;
  return row("s24", "start", windowed(state, DEFAULT_WIDTH, DEFAULT_HEIGHT), stack("s8", "start", label("The control center's widget, drafted", "label"), widget({ ...state, playing: true })));
}

// The stage: the player live, with what plays and a song's visualiser as switches over it.

const PLAYS: readonly { readonly label: string; readonly state: PlayerState }[] = [
  { label: "Film", state: PLAYING },
  { label: "Playlist", state: { ...PLAYING, index: 1, count: PLAYLIST.length, file: PLAYLIST[1]! } },
  { label: "Songs", state: { ...PLAYING, file: SONG, list: SONGS, index: 0, count: SONGS.length, position: 0, buffered: 1, playing: false, shown: true } },
  { label: "Bare song", state: { ...PLAYING, file: BARE_SONG, position: 0.12 } },
  { label: "Stream", state: { ...PLAYING, file: STREAM, buffering: 0.42, buffered: 0.36, position: 0.33 } },
  { label: "Broken", state: { ...PLAYING, file: BROKEN, error: true } },
  { label: "Nothing", state: { playing: false } },
];

/** The one sample on the page making sound, so a second one starting silences the first. */
let sounding: { readonly stop: () => void } | null = null;

/** A step of the volume from the wheel or a key, of 0 to 1; a wheel's notch as a browser counts it; the touchpad's travel to a step, px. */
const VOLUME_STEP = 0.02;
const WHEEL_NOTCH = 120;
const TOUCH_STEP = 24;

/** How loud mpv's volume of `value`, 0 to 1, sounds: its scale is cubic, as mpv's own is. */
const loudness = (value: number) => value * value * value;

/** A dropped file's tags, read from its first bytes where they are cheap to read: ID3v2 (mp3) and Vorbis comments (flac, ogg, opus). */
async function tagsOf(file: File): Promise<{ title?: string; by?: string }> {
  const bytes = new Uint8Array(await file.slice(0, 256 * 1024).arrayBuffer());
  const found: { title?: string; by?: string } = {};
  const utf8 = new TextDecoder();
  // ID3v2: frames of a four-letter id, a size, two flags bytes, then an encoding byte and the text.
  if (bytes[0] === 0x49 && bytes[1] === 0x44 && bytes[2] === 0x33) {
    const version = bytes[3]!;
    const syncsafe = (at: number) => ((bytes[at]! & 0x7f) << 21) | ((bytes[at + 1]! & 0x7f) << 14) | ((bytes[at + 2]! & 0x7f) << 7) | (bytes[at + 3]! & 0x7f);
    const end = Math.min(bytes.length, 10 + syncsafe(6));
    for (let at = 10; at + 10 < end; ) {
      const id = String.fromCharCode(...bytes.subarray(at, at + 4));
      const size = version >= 4 ? syncsafe(at + 4) : (bytes[at + 4]! << 24) | (bytes[at + 5]! << 16) | (bytes[at + 6]! << 8) | bytes[at + 7]!;
      if (!/^[A-Z0-9]{4}$/.test(id) || size <= 0) break;
      if (id === "TIT2" || id === "TPE1") {
        const body = bytes.subarray(at + 11, at + 10 + size);
        const encoding = bytes[at + 10];
        const words = (encoding === 1 || encoding === 2 ? new TextDecoder(encoding === 2 ? "utf-16be" : "utf-16") : encoding === 3 ? utf8 : new TextDecoder("latin1")).decode(body).replace(/\0+$/, "").trim();
        if (words) found[id === "TIT2" ? "title" : "by"] = words;
      }
      at += 10 + size;
    }
    return found;
  }
  // Vorbis comments: "TITLE=…" and "ARTIST=…", each after its length, four bytes little-endian.
  const text = new TextDecoder("latin1").decode(bytes);
  for (const [key, name] of [["title", "title"], ["artist", "by"]] as const) {
    const at = text.search(new RegExp(`${key}=`, "i"));
    if (at < 4) continue;
    const length = bytes[at - 4]! | (bytes[at - 3]! << 8) | (bytes[at - 2]! << 16) | (bytes[at - 1]! << 24);
    if (length <= key.length + 1 || length > 4096) continue;
    const words = utf8.decode(bytes.subarray(at + key.length + 1, at + length)).trim();
    if (words) found[name] = words;
  }
  return found;
}

/** How long a file of sound plays, by the browser's own reading of it: none if it can't play it. */
function playLength(url: string): Promise<number | null> {
  return new Promise((resolve) => {
    const probe = new Audio();
    probe.preload = "metadata";
    const done = (length: number | null) => {
      probe.removeAttribute("src");
      probe.load();
      resolve(length);
    };
    probe.addEventListener("loadedmetadata", () => done(Number.isFinite(probe.duration) && probe.duration > 0 ? probe.duration : null), { once: true });
    probe.addEventListener("error", () => done(null), { once: true });
    probe.src = url;
  });
}

/** A duration token in milliseconds under the speed setting. */
function ms(name: string): number {
  return durationMs(name) / settings().speed;
}

/** The live player, driven by the pointer and mpv's keys: every control and key acts. */
function live(): HTMLElement {
  let visual: VizChoice = "water";
  let colours: Colours = "wallpaper";
  let idle = 0;
  let said = 0;
  const room = document.createElement("span");
  room.className = "mp-live-room";
  /** The song's sound, made by the first play a person asks for, never before. */
  let audio: HTMLAudioElement | null = null;
  /** The file the sound was last set to: picking it again starts it again. */
  let sounded: MediaFile | undefined;
  /** The level the sound is ramping to, and the ramp's frame. */
  let heard = -1;
  let ramp = 0;

  /** A message from a key: down from the top edge, then back after `duration.scene`. */
  const say = (message: Message) => {
    set({ message });
    window.clearTimeout(said);
    said = window.setTimeout(() => {
      const going = root.querySelector<HTMLElement>(".mp-message");
      const done = () => set({ message: undefined });
      if (!going || settings().reduced) return done();
      going.animate([{ "--mp-up": "1" }, { "--mp-up": "0" }], { duration: ms("medium"), easing: raw("easing-emphasized_accel"), fill: "forwards" }).finished.then(done, done);
    }, ms("scene"));
  };

  const made = player(
    { ...PLAYING, preview: true, visual, colours },
    {
      volume: (value) => (say(MESSAGES.volume(value)), queueMicrotask(() => sync())),
      muted: (on) => (say(MESSAGES.volume(now().volume ?? 0, on)), queueMicrotask(() => sync())),
      take: (index) => playAt(index, true),
      live: true,
      time: () => {
        const file = now().file;
        if (!file?.source) return null;
        return audio && audio.dataset.source === file.source ? audio.currentTime : (now().position ?? 0) * file.length;
      },
    },
  );
  const root = made.element;
  root.tabIndex = 0;
  const now = made.state;
  room.append(root);

  /** Stops the sound and lets its file go: on leaving the page or the sample. */
  const silence = () => {
    if (!audio) return;
    audio.pause();
    audio.removeAttribute("src");
    audio.load();
    audio = null;
    sounding = null;
    heard = -1;
    cancelAnimationFrame(ramp);
  };
  /**
   * The sound follows the state: the file, play or pause, the volume, mute, the speed. `asked` says
   * a person's click or key led here, the only thing that may make the sound in the first place.
   */
  const sync = (asked = false) => {
    const state = now();
    const file = state.file;
    if (!audio) {
      if (!asked || !file?.source || !state.playing || state.error) return;
      sounding?.stop();
      audio = new Audio();
      audio.preload = "auto";
      audio.addEventListener("ended", () => {
        const at = now().index ?? 0;
        if (at + 1 < (now().count ?? 1)) playAt(at + 1);
        else set({ playing: false, position: 1 });
      });
      sounding = { stop: silence };
      // It starts at its level: only a change of it ramps.
      heard = state.muted ? 0 : loudness(state.volume ?? 1);
      audio.volume = heard;
    }
    const source = file?.source;
    if (!source || state.error) {
      sounded = file;
      return audio.pause();
    }
    if (audio.dataset.source !== source || sounded !== file) {
      // Another song: the old one stops before the new one loads, so the two are never heard at once.
      audio.pause();
      if (audio.dataset.source !== source) {
        audio.dataset.source = source;
        audio.src = source;
      }
      audio.currentTime = (state.position ?? 0) * file.length;
      sounded = file;
    }
    // The volume and mute only ever set the level, in place, ramped over `micro` so it never clicks;
    // the sound plays on.
    rampTo(state.muted ? 0 : loudness(state.volume ?? 1));
    audio.playbackRate = state.speed ?? 1;
    if (state.playing && !state.dragging && audio.paused) void audio.play().catch(() => undefined);
    else if ((!state.playing || state.dragging) && !audio.paused) audio.pause();
  };
  /** Eases the sound's level to `level` over `duration.micro`. */
  const rampTo = (level: number) => {
    const sound = audio;
    if (!sound || level === heard) return;
    heard = level;
    cancelAnimationFrame(ramp);
    const from = sound.volume;
    const start = performance.now();
    const length = durationMs("micro");
    // A frame's time is its start, which can lie before `start`: unclamped, the level would run
    // back past `from` and out of the range a sound's volume takes.
    const step = (now: number) => {
      const t = Math.max(0, Math.min(1, (now - start) / length));
      sound.volume = Math.max(0, Math.min(1, from + (level - from) * t));
      if (t < 1) ramp = requestAnimationFrame(step);
    };
    ramp = requestAnimationFrame(step);
  };
  /** Moves the sound to `position` of its song, when it is the one playing. */
  const seekSound = (position: number) => {
    const file = now().file;
    if (audio && file?.source && audio.dataset.source === file.source) audio.currentTime = Math.min(position, 0.999) * file.length;
  };
  const set = (patch: Parameters<typeof made.set>[0]) => {
    made.set(patch);
    sync();
  };
  window.addEventListener("pagehide", silence, { once: true });

  /** Whether the controller may go: playing, no menu or playlist open. */
  const free = () => now().playing === true && now().open === undefined && !now().sidebar;
  /**
   * The controller shows, and goes `duration.controller_rest` after the pointer last moved over the
   * picture; over the controller, a menu, the corner or the playlist it stays.
   */
  const wake = () => {
    if (!now().shown) set({ shown: true });
    window.clearTimeout(idle);
    idle = window.setTimeout(() => {
      const over = root.querySelector(".mp-controller:hover, .mp-popover:hover, .mp-corner:hover, .mp-sidebar:hover");
      if (!over && free()) set({ shown: false });
      else wake();
    }, durationMs("controller_rest"));
  };

  const lengthOf = () => now().file?.length ?? 1;
  const seekTo = (position: number) => {
    const at = Math.max(0, Math.min(1, position));
    set({ position: at, buffered: Math.max(at, now().buffered ?? 0) });
    seekSound(at);
  };
  const seekBy = (seconds: number) => {
    seekTo((now().position ?? 0) + seconds / lengthOf());
    say(MESSAGES.seek(now(), seconds > 0));
  };
  const volumeBy = (step: number) => {
    const volume = Math.max(0, Math.min(1, Math.round(((now().volume ?? 0) + step) * 100) / 100));
    set({ volume, muted: false });
    say(MESSAGES.volume(volume));
  };
  const mute = () => {
    const muted = !now().muted;
    set({ muted });
    say(MESSAGES.volume(now().volume ?? 0, muted));
  };
  const speedTo = (speed: number) => {
    const clamped = Math.max(0.25, Math.min(4, Math.round(speed * 100) / 100));
    set({ speed: clamped });
    say(MESSAGES.speed(clamped));
  };
  /** The playlist's file at `index`, from its start, playing; `asked` when a person picked it. */
  function playAt(index: number, asked = false): void {
    const count = now().count ?? 1;
    if (count < 2) return;
    const at = Math.max(0, Math.min(count - 1, index));
    if (at === now().index) return set({ sidebar: false });
    const file = (now().list ?? PLAYLIST)[at]!;
    made.set({ index: at, file, position: 0, buffered: file.source ? 1 : 0.1, playing: true, open: undefined });
    sync(asked);
  }
  const subtitleTo = (index: number | null) => {
    set({ subtitle: index, line: index === null ? undefined : LINES[index] ?? "", open: undefined });
    say(MESSAGES.subtitles(index));
  };
  const audioTo = (index: number) => {
    set({ audioTrack: index, open: undefined });
    say(MESSAGES.audio(index));
  };
  /**
   * Full screen on or off: the live room takes the whole screen, as mpv's window does, and the
   * player follows it. The screen's own way out (Esc, the browser's) brings the state back with it.
   */
  const fullShown = (on: boolean) => {
    set({ full: on, open: undefined });
    room.toggleAttribute("data-full", on);
    // The window is built again for it, the button pressed with it: mpv's keys stay with the player.
    root.focus({ preventScroll: true });
  };
  const full = (on: boolean) => {
    if (on && document.fullscreenElement !== room) void room.requestFullscreen?.().catch(() => undefined);
    else if (!on && document.fullscreenElement === room) void document.exitFullscreen().catch(() => undefined);
    fullShown(on);
  };
  const leftScreen = () => {
    if (!room.isConnected) return document.removeEventListener("fullscreenchange", leftScreen);
    const on = document.fullscreenElement === room;
    if (on !== (now().full ?? false)) fullShown(on);
  };
  document.addEventListener("fullscreenchange", leftScreen);
  /** Play or pause, blooming the state it enters in the picture's middle. */
  const toggle = () => {
    if (!now().file || now().error) return;
    const position = now().position ?? 0;
    made.set({ playing: !now().playing, ...(position >= 1 ? { position: 0 } : {}) });
    sync(true);
    made.bloom();
  };
  const closeAll = () => set({ open: undefined, sidebar: false });
  /** Another visualiser, `step` along the list from this one, Off among them; the switch follows. */
  let pickVisual: (choice: VizChoice) => void = () => undefined;
  const visualTo = (choice: VizChoice) => {
    visual = choice;
    set({ visual: choice, open: undefined });
    pickVisual(choice);
    if (now().file?.audio) say(MESSAGES.visual(choice));
  };
  const stepVisual = (step: number) => visualTo(CHOICES[(CHOICES.indexOf(visual) + step + CHOICES.length) % CHOICES.length]!);

  // Playing moves the time on in place, only on the live stage: a song's from its sound's own clock,
  // any other file's a second a second. Leaving the sample stops the sound.
  if (CHROME) {
    const TICK = 0.25;
    const tick = window.setInterval(() => {
      if (!root.isConnected) {
        silence();
        return window.clearInterval(tick);
      }
      const state = now();
      if (!state.playing || !state.file || state.error || state.buffering !== undefined || state.dragging) return;
      const file = state.file;
      if (file.source) {
        if (!audio || audio.dataset.source !== file.source) return;
        return made.set({ position: Math.min(1, audio.currentTime / file.length) });
      }
      const position = Math.min(1, (state.position ?? 0) + ((state.speed ?? 1) * TICK) / file.length);
      made.set({ position, buffered: Math.min(1, Math.max(position + 0.18, state.buffered ?? 0)) });
    }, TICK * 1000);
  }

  // The pointer.
  /** The element a click landed on, still in the page: the deepest of its path that is. */
  const landed = (event: Event): Element => {
    const path = event.composedPath().filter((each): each is Element => each instanceof Element);
    return path.find((each) => each.isConnected && root.contains(each)) ?? root;
  };
  const along = (event: PointerEvent): number | null => {
    const seek = root.querySelector<HTMLElement>(".mp-seek");
    if (!seek) return null;
    const box = seek.getBoundingClientRect();
    return Math.max(0, Math.min(1, (event.clientX - box.left) / Math.max(1, box.width)));
  };
  let dragging = false;
  /** A drag's end just now, whose click is not a click on the picture. */
  let dragged = 0;
  root.addEventListener("pointermove", (event) => {
    wake();
    const target = event.target as Element;
    if (dragging) {
      const at = along(event);
      if (at !== null) set({ position: at, hover: at });
    } else if (target.closest(".mp-seek")) {
      const at = along(event);
      if (at !== null && Math.abs(at - (now().hover ?? -1)) > 0.001) set({ hover: at });
    } else if (now().hover !== undefined) set({ hover: undefined });
  });
  // The pointer leaving the window takes the controller with it at once, as it goes in mpv.
  root.addEventListener("pointerleave", () => {
    if (dragging) return;
    if (now().hover !== undefined) set({ hover: undefined });
    window.clearTimeout(idle);
    if (now().shown && free()) set({ shown: false });
  });
  root.addEventListener("pointerdown", (event) => {
    const target = event.target as Element;
    if (!target.closest(".mp-seek")) return;
    dragging = true;
    root.setPointerCapture(event.pointerId);
    const at = along(event);
    if (at !== null) set({ dragging: true, position: at, hover: at });
  });
  const release = () => {
    if (!dragging) return;
    dragging = false;
    dragged = window.setTimeout(() => (dragged = 0));
    seekSound(now().position ?? 0);
    set({ dragging: false });
  };
  root.addEventListener("pointerup", release);
  root.addEventListener("pointercancel", release);
  // The wheel over the picture turns the volume; over the volume's slider, its own. The hand moving
  // up is louder on either device: a wheel turned away, two fingers pushed up the touchpad. The
  // touchpad scrolls naturally here, so its fingers going up arrive as a scroll down, and a browser
  // is not told which way round a device is: a wheel is known by its whole notches, and anything
  // finer is taken for the touchpad, whose travel adds up to a step every `TOUCH_STEP` px.
  let travelled = 0;
  root.addEventListener(
    "wheel",
    (event) => {
      if ((event.target as Element).closest(".mp-sidebar, .mp-popover, .k-slider")) return;
      event.preventDefault();
      if (event.deltaY === 0) return;
      const notches = (event as WheelEvent & { wheelDeltaY?: number }).wheelDeltaY ?? 0;
      if (event.deltaMode !== WheelEvent.DOM_DELTA_PIXEL || (notches !== 0 && notches % WHEEL_NOTCH === 0)) {
        travelled = 0;
        return volumeBy(event.deltaY < 0 ? VOLUME_STEP : -VOLUME_STEP);
      }
      travelled += event.deltaY;
      const steps = Math.trunc(travelled / TOUCH_STEP);
      if (steps === 0) return;
      travelled -= steps * TOUCH_STEP;
      volumeBy(steps * VOLUME_STEP);
    },
    { passive: false },
  );
  root.addEventListener("click", (event) => {
    if (dragged) return;
    // Where the click landed, as the page stood when it was dispatched: a control may redraw what
    // was pressed before this runs (the volume's icon swaps its symbol as it mutes), and a target
    // gone from the page would read as a click on the picture, which plays or pauses.
    const target = landed(event);
    const item = target.closest<HTMLElement>(".mp-popover .k-menu-item");
    if (item) {
      const open = now().open;
      const items = [...(item.closest(".mp-popover")?.querySelectorAll<HTMLElement>(".k-menu-item") ?? [])];
      const index = items.indexOf(item);
      const words = (item.textContent ?? "").trim();
      if (open === "tracks") index < AUDIO_TRACKS.length ? audioTo(index) : subtitleTo(index === AUDIO_TRACKS.length ? null : index - AUDIO_TRACKS.length - 1);
      else if (open === "speed") (speedTo(SPEEDS[index] ?? 1), set({ open: undefined }));
      else if (open === "visualiser") visualTo(CHOICES[index] ?? "off");
      else if (words.startsWith("Visualiser")) set({ open: "visualiser" });
      else if (words.startsWith("Speed")) set({ open: "speed" });
      else if (words.startsWith("Screenshot")) (set({ open: undefined }), say(MESSAGES.screenshot()));
      else if (words.startsWith("Previous")) playAt((now().index ?? 0) - 1, true);
      else if (words.startsWith("Next")) playAt((now().index ?? 0) + 1, true);
      else if (words.startsWith("Audio")) set({ open: "tracks" });
      else if (words.startsWith("Mute") || words.startsWith("Unmute")) (set({ open: undefined }), mute());
      else if (words.includes("ull screen")) full(!now().full);
      return;
    }
    if (target.closest(".mp-sidebar")) return;
    if (target.closest(".mp-length")) return set({ remaining: !now().remaining });
    const button = target.closest<HTMLElement>("button");
    if (button) {
      // The slider's own mute button: the slider tells the player itself.
      if (button.classList.contains("k-slider-icon-button")) return;
      const does = button.dataset.does ?? button.getAttribute("aria-label") ?? "";
      if (button.classList.contains("mp-play")) toggle();
      else if (does === "Previous") playAt((now().index ?? 0) - 1, true);
      else if (does === "Next") playAt((now().index ?? 0) + 1, true);
      else if (does === "Audio and subtitles") set({ open: now().open === "tracks" ? undefined : "tracks", sidebar: false });
      else if (does === "More") set({ open: now().open === "more" || now().open === "speed" ? undefined : "more", sidebar: false });
      else if (does === "Playlist") set({ sidebar: !now().sidebar, open: undefined });
      else if (does === "Full screen" || does === "Leave full screen") full(!now().full);
      else if (does === "Mute" || does === "Unmute") mute();
      return;
    }
    if (target.closest(".mp-controller, .mp-title-bar, .mp-corner")) return;
    if (now().open || now().sidebar) return closeAll();
    toggle();
  });
  root.addEventListener("dblclick", (event) => {
    if (landed(event).closest(".mp-controller, .mp-popover, .mp-sidebar, .mp-corner")) return;
    full(!now().full);
  });

  // mpv's own keys, as its default `input.conf` has them.
  root.addEventListener("keydown", (event) => {
    const key = event.key;
    const state = now();
    if (key === " " || key === "p") toggle();
    else if (key === "ArrowRight") seekBy(event.shiftKey ? 1 : 5);
    else if (key === "ArrowLeft") seekBy(event.shiftKey ? -1 : -5);
    else if (key === "ArrowUp") seekBy(60);
    else if (key === "ArrowDown") seekBy(-60);
    else if (key === "0" || key === "*") volumeBy(VOLUME_STEP);
    else if (key === "9" || key === "/") volumeBy(-VOLUME_STEP);
    else if (key === "m") mute();
    else if (key === "]") speedTo((state.speed ?? 1) * 1.1);
    else if (key === "[") speedTo((state.speed ?? 1) / 1.1);
    else if (key === "}") speedTo((state.speed ?? 1) * 2);
    else if (key === "{") speedTo((state.speed ?? 1) / 2);
    else if (key === "Backspace") speedTo(1);
    else if (key === "f" || key === "F") full(!state.full);
    else if (key === "Escape") {
      if (state.open || state.sidebar) closeAll();
      else full(false);
    } else if (key === "j") subtitleTo(state.subtitle === null || state.subtitle === undefined ? 0 : state.subtitle + 1 < SUBTITLES.length ? state.subtitle + 1 : null);
    else if (key === "n" || key === "N") stepVisual(key === "n" ? 1 : -1);
    else if (key === "v") subtitleTo(state.subtitle === null || state.subtitle === undefined ? 0 : null);
    else if (key === "#") audioTo(((state.audioTrack ?? 0) + 1) % AUDIO_TRACKS.length);
    else if (key === "s") say(MESSAGES.screenshot());
    else if (key === ">" || key === "Enter") playAt((state.index ?? 0) + 1, true);
    else if (key === "<") playAt((state.index ?? 0) - 1, true);
    else return;
    event.preventDefault();
    if (key !== "Escape") wake();
  });

  // A file of sound dropped on the player plays as the song now, through the same path as the
  // songs that come with the page: its sound, its analysis in the background, its visualiser. It
  // lives for the page's session only; a file the browser can't play leaves the song as it was.
  root.addEventListener("dragover", (event) => {
    if (!event.dataTransfer?.types.includes("Files")) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
  });
  root.addEventListener("drop", (event) => {
    const dropped = [...(event.dataTransfer?.files ?? [])].find((each) => each.type.startsWith("audio/") || /\.(mp3|flac|ogg|oga|opus|wav|m4a|aac)$/i.test(each.name));
    if (!dropped) return;
    event.preventDefault();
    void (async () => {
      const url = URL.createObjectURL(dropped);
      const [length, tags] = await Promise.all([playLength(url), tagsOf(dropped).catch(() => ({}) as { title?: string; by?: string })]);
      if (length === null) return URL.revokeObjectURL(url);
      const named = dropped.name.replace(/\.[^.]+$/, "").replace(/[_]+/g, " ").trim();
      const file: MediaFile = { title: tags.title ?? named, ...(tags.by ? { by: tags.by } : {}), length, audio: true, source: url };
      analysedSong(url, length);
      const state = now();
      const list = state.list && state.file?.audio ? state.list : [];
      const at = list.length > 0 ? (state.index ?? 0) + 1 : 0;
      const files = [...list.slice(0, at), file, ...list.slice(at)];
      made.set({ list: files, count: files.length, index: at, file, position: 0, buffered: 1, playing: true, error: undefined, buffering: undefined, open: undefined, sidebar: false });
      sync(true);
    })();
  });

  // What plays.
  const question = (title: string, choices: readonly string[], picked: number, pick: (index: number) => void) =>
    stack("s8", "start", label(title, "label"), tabs({ choices: choices.map((words) => ({ label: words })), picked, onPick: pick }));
  const options = document.createElement("span");
  options.className = "mp-options";
  options.append(
    question("Plays", PLAYS.map((each) => each.label), 0, (i) => {
      const next = PLAYS[i]!.state;
      set({ ...PLAYING, file: undefined, error: undefined, buffering: undefined, message: undefined, open: undefined, sidebar: false, line: undefined, remaining: false, ...next, preview: true, visual, colours, full: now().full ?? false, shown: true });
      root.focus({ preventScroll: true });
      // The songs with sound are analysed once, one after the other from the first, so a skip never waits.
      if (next.file?.source) void SONGS.reduce<Promise<unknown>>((before, each) => (each.source ? before.then(() => analysedSong(each.source!, each.length).ready).catch(() => undefined) : before), Promise.resolve());
    }),
  );
  // A song's visualiser, where its cover and words stand over it, and whose colours it burns in.
  const songs = document.createElement("span");
  songs.className = "mp-options";
  const visualSwitch = tabs({ choices: CHOICES.map((choice) => ({ label: vizName(choice) })), picked: CHOICES.indexOf(visual), onPick: (i) => CHOICES[i] !== visual && visualTo(CHOICES[i]!) });
  pickVisual = (choice) => visualSwitch.querySelectorAll<HTMLElement>("[role='tab'], .k-tab")[CHOICES.indexOf(choice)]?.click();
  songs.append(
    stack("s8", "start", label("A song's visualiser", "label"), visualSwitch),
    question("Its colours", ["Wallpaper", "Cover"], 0, (i) => set({ colours: (colours = i === 0 ? "wallpaper" : "cover") })),
  );
  const box = document.createElement("span");
  box.className = "mp-live";
  // What plays and the visualiser are the stage's own switches, drawn above the sample at the site's size.
  box.append(stageControls(options), stageControls(songs), room, label("Drag the corner to resize it; point at it, click, scroll, or use mpv's keys; drop a song of your own on it to try it", "mono"));
  return box;
}

// Motion.

const CONTROLLER = "the controller";

const APPEARS: Move = {
  name: "The controller arriving",
  what: "The pointer moves, or a pause: the glass rises `space.s16` as it fades in, its rows a `swap_lag` apart.",
  tracks: [
    { part: "the glass", on: CONTROLLER, target: ".mp-window", property: "--mp-shown", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the title row", on: CONTROLLER, target: ".mp-title-row", property: "--mp-in", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "the bar row", on: CONTROLLER, target: ".mp-bar-row", property: "--mp-in", from: 0, to: 1, duration: "short", cue: "swap_lag", curve: "emphasized_decel" },
    { part: "the buttons", on: CONTROLLER, target: ".mp-buttons", property: "--mp-in", from: 0, to: 1, duration: "short", cue: ["swap_lag", "swap_lag"], curve: "emphasized_decel" },
  ],
  reduced: [
    { part: "the glass", on: CONTROLLER, target: ".mp-window", property: "--mp-shown", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the rows", on: CONTROLLER, target: ".mp-title-row, .mp-bar-row, .mp-buttons", property: "--mp-in", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => windowed(PLAYING, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const LEAVES: Move = {
  name: "The controller leaving",
  what: "Playing: the pointer gone, or still `duration.controller_rest`; it sinks `space.s16` and fades over `medium`.",
  tracks: [{ part: "the glass", on: CONTROLLER, target: ".mp-window", property: "--mp-shown", from: 1, to: 0, duration: "medium", curve: "standard_accel" }],
  reduced: [{ part: "the glass", on: CONTROLLER, target: ".mp-window", property: "--mp-shown", from: 1, to: 0, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => windowed(SHOWN, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const BLOOMS: Move = {
  name: "Play or pause blooming",
  what: "A press on the picture or Space: its new symbol blooms in the middle to `scale.bloom` and fades.",
  tracks: [
    { part: "the disc", on: "the bloom", target: ".mp-bloom", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: "the disc", on: "the bloom", target: ".mp-bloom", property: "opacity", from: 1, to: 0, duration: "medium", cue: "micro", curve: "standard_accel" },
    { part: "its size", on: "the bloom", target: ".mp-bloom", property: "scale", from: 1, to: { scale: "bloom" }, duration: "long", curve: "standard_decel" },
  ],
  reduced: [
    { part: "the disc", on: "the bloom", target: ".mp-bloom", property: "opacity", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: "the disc", on: "the bloom", target: ".mp-bloom", property: "opacity", from: 1, to: 0, duration: "short", cue: "micro", curve: "standard" },
  ],
  back: "cuts",
  sample: () => windowed(PLAYING, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const MORPHS: Move = {
  name: "Play morphing to pause",
  what: "Its symbol's two halves part into the two bars over `short`; back the same way.",
  tracks: [{ part: "the symbol", on: "the play button", target: ".mp-play .k-morph", property: "--morph", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: [{ part: "the symbol", on: "the play button", target: ".mp-play .k-morph", property: "--morph", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "plays",
  sample: () => windowed(PAUSED, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const GROWS: Move = {
  name: "The bar lighting",
  what: "The pointer onto the bar: the tube grows to `size.progress` and its head lights over `short`.",
  tracks: [{ part: "the tube", on: "the seek bar", target: ".mp-seek", property: "--mp-grow", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => windowed(SHOWN, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const UNFOLDS: Move = {
  name: "The volume unfolding",
  what: "The pointer onto the speaker: the slider unfolds from it to `size.bar_label` over `medium`.",
  tracks: [{ part: "the slider", on: "the volume", target: ".mp-volume", property: "--mp-unfold", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "the slider", on: "the volume", target: ".mp-volume", property: "--mp-unfold", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "plays",
  sample: () => windowed(SHOWN, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const SLIDES: Move = {
  name: "The playlist sliding in",
  what: "Its corner button: the panel slides in from the end edge over `long`, the controller drawing in beside it; out over `medium`.",
  tracks: [{ part: "the panel", on: "the playlist", target: ".mp-window", property: "--mp-side", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the panel", on: "the playlist", target: ".mp-window", property: "--mp-side", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => windowed(IN_LIST, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const SAYS: Move = {
  name: "A message arriving",
  what: "A key: the OSD strip comes down from the top edge and fades in over `long`.",
  tracks: [{ part: "the strip", on: "a message", target: ".mp-message", property: "--mp-up", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the strip", on: "a message", target: ".mp-message", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => windowed({ ...PLAYING, message: MESSAGES.volume(0.64) }, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

const UNSAYS: Move = {
  name: "A message going",
  what: "`duration.scene` after the last key: it goes back up fading over `medium`.",
  tracks: [{ part: "the strip", on: "a message", target: ".mp-message", property: "--mp-up", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: "the strip", on: "a message", target: ".mp-message", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => windowed({ ...PLAYING, message: MESSAGES.volume(0.64) }, DEFAULT_WIDTH, DEFAULT_HEIGHT),
};

// The ladder.

/** The widths the ladder tries, in logical pixels: from the default down, a step at a time. */
const LADDER = Array.from({ length: 24 }, (_, i) => 1120 - i * 40);

/**
 * Every rung at the widest width that takes it: the windows are drawn at each width tried, and once
 * laid out only the first to reach each rung stays, saying what it gave way.
 */
function ladder(): HTMLElement {
  const box = document.createElement("span");
  box.className = "mp-ladder";
  const steps = LADDER.map((width) => {
    const shown = windowed({ ...IN_LIST, file: PLAYLIST[1]! }, `calc(${width} * var(--px))`, "calc(var(--size-thumbnail_height) + var(--space-s64) * 2)");
    const words = label(`${width}`, "mono");
    const step = document.createElement("span");
    step.className = "mp-ladder-step";
    step.style.width = `calc(${width} * var(--px))`;
    step.append(words, shown);
    box.append(step);
    return { width, step, shown, words };
  });
  let pruned = false;
  const prune = () => {
    if (pruned || !steps.every(({ shown }) => shown.querySelector<HTMLElement>(".mp-window")?.dataset.rung !== undefined)) return;
    pruned = true;
    let before = -1;
    for (const { width, step, shown, words } of steps) {
      const rung = Number(shown.querySelector<HTMLElement>(".mp-window")!.dataset.rung);
      if (rung === before) {
        step.remove();
        continue;
      }
      const taken = RUNGS.slice(Math.max(0, before), rung).map((each) => each.says);
      words.textContent = `${width} · ${rung === 0 ? "everything shows" : taken.join(" · ")}`;
      words.dataset.wrapped = "";
      before = rung;
    }
  };
  new ResizeObserver(() => requestAnimationFrame(prune)).observe(box);
  return box;
}

// The page.

type Shown = "rest" | "lit" | "pressed" | "open" | "disabled" | "loading" | "empty" | "error";

function cell(state: Shown): HTMLElement {
  const base: Record<Shown, PlayerState> = {
    rest: SHOWN,
    lit: { ...SHOWN, lit: "play", hover: 0.58 },
    pressed: { ...SHOWN, dragging: true, hover: 0.61, position: 0.61 },
    open: { ...SHOWN, open: "tracks" },
    disabled: { ...SHOWN, count: PLAYLIST.length, index: 0, file: PLAYLIST[0]! },
    loading: { ...SHOWN, file: STREAM, buffering: 0.42, buffered: 0.36, position: 0.33 },
    empty: { playing: false },
    error: { ...SHOWN, file: BROKEN, error: true },
  };
  return windowed(base[state], DEFAULT_WIDTH, DEFAULT_HEIGHT);
}

const titled = (file: MediaFile, title: string): MediaFile => ({ ...file, title });

/** What each visualiser is, a few words. */
const VIZ_FOR: Record<Exclude<VizChoice, "off">, string> = {
  water: "a tunnel of strings of light that carry the sound",
  silk: "sheets of light folding through each other, swelling on the bass",
  tube: "the waveform as a neon tube, its afterglow streaming outward",
  embers: "sparks in a current, thrown on the kick, ringing on a build",
};

export default page<Shown>({
  level: "apps",
  slug: "media-player",
  title: "Media player",
  lede: "mpv, KubaOS's one player for video and audio, with a controller of KubaOS's own drawn over it.",
  code: { unbuilt: "mpv Lua script: not built yet" },
  from: ["icon-button", "slider", "progress-bar", "rolling-number", "hint", "popover", "menu", "list", "error-block", "spinner"],
  stage: {
    live,
    hint: "point, click, scroll or use mpv's keys",
    daylight: doesntApply("it draws in the night scheme in either mode, as the lock does: it sits on a picture."),
  },
  variants: [
    { name: "Film", for: "the picture alone, its colours lighting the letterbox", sample: () => desk(PLAYING) },
    { name: "Controller shown", for: "the pointer moved: the glass, the lit bar, the buttons", sample: () => desk(SHOWN) },
    { name: "Full screen, hidden", for: "over the bar: the picture alone", sample: () => whole(PLAYING) },
    { name: "Full screen, shown", for: "the same controller, square corners; Esc leaves", sample: () => whole(SHOWN) },
    { name: "Beside Settings", for: "inactive: the ring alone; tiles `size.gap` apart", sample: () => beside(PLAYING) },
    { name: "Paused", for: "the controller stays while paused; Play's symbol", sample: () => desk(PAUSED) },
    { name: "Seeking, over the bar", for: "the tube grows, its head lit; time and chapter over it", sample: () => desk({ ...SHOWN, hover: 0.62 }) },
    { name: "Seeking, dragging", for: "held: the head brightens and follows the pointer", sample: () => desk({ ...SHOWN, dragging: true, hover: 0.47, position: 0.47 }) },
    { name: "Seeking, a frame preview", for: "a frame over the time, rising in", sample: () => desk({ ...SHOWN, hover: 0.62, preview: true }) },
    { name: "Volume, unfolded", for: "the speaker's slider unfolded under the pointer", sample: () => desk({ ...SHOWN, volume: 0.38, lit: "volume" }) },
    { name: "Muted", for: "the slider's toggle: its fill softens, Muted", sample: () => desk({ ...SHOWN, muted: true, lit: "volume" }) },
    { name: "Tracks", for: "audio and subtitles in one menu, by its button", sample: () => desk({ ...SHOWN, open: "tracks", subtitle: 1, line: LINES[1] }) },
    { name: "Subtitles off", for: "Off ticked; no line on the picture", sample: () => desk({ ...SHOWN, open: "tracks", subtitle: null }) },
    { name: "Subtitles on, hidden", for: "the line low; it lifts over the controller", sample: () => desk({ ...PLAYING, subtitle: 0, line: LINES[0] }) },
    { name: "Playlist", for: "several files: the corner's button, previous and next", sample: () => desk(IN_LIST) },
    { name: "Playlist, its panel", for: "the files on glass the window's height, the one playing ticked", sample: () => desk({ ...IN_LIST, sidebar: true }) },
    { name: "Playlist, the first file", for: "previous disabled at the start", sample: () => desk({ ...IN_LIST, index: 0, file: PLAYLIST[0]! }) },
    { name: "Speed", for: "⋯, then Speed: the presets, by ⋯", sample: () => desk({ ...SHOWN, open: "speed", speed: 1.25 }) },
    { name: "More", for: "⋯: speed and the screenshot, by its button", sample: () => desk({ ...SHOWN, open: "more" }) },
    { name: "The time left", for: "a press on the length: it counts down", sample: () => desk({ ...SHOWN, remaining: true }) },
    { name: "Message: seeking", for: "← →: the OSD strip, the place and the time", sample: () => desk({ ...PLAYING, message: MESSAGES.seek(PLAYING, true) }) },
    { name: "Message: volume", for: "9 0 and the wheel: as the shell's volume OSD", sample: () => desk({ ...PLAYING, message: MESSAGES.volume(0.64) }) },
    { name: "Message: speed", for: "[ ]: the speed, in words", sample: () => desk({ ...PLAYING, speed: 1.21, message: MESSAGES.speed(1.21) }) },
    { name: "Message: screenshot saved", for: "s: in words", sample: () => desk({ ...PLAYING, message: MESSAGES.screenshot() }) },
    { name: "Message over the controller", for: "a key while the controller shows", sample: () => desk({ ...SHOWN, message: MESSAGES.subtitles(1) }) },
    { name: "Song", for: "the cover with depth, its colours washing the window, the levels", sample: () => desk(SONG_PLAYING) },
    { name: "Song, paused", for: "the levels settled to rest", sample: () => desk({ ...SONG_PLAYING, playing: false }) },
    ...CHOICES.filter((choice) => choice !== "off").flatMap((choice) => [
      { name: `Visualiser: ${vizName(choice)}`, for: VIZ_FOR[choice], sample: () => desk({ ...SONG_PLAYING, visual: choice }) },
      { name: `${vizName(choice)}, paused`, for: "the song stopped: the light breathing slowly", sample: () => desk({ ...SONG_PLAYING, visual: choice, playing: false }) },
    ]),
    { name: "Visualiser, the controller shown", for: "the song's corner rides over the controller", sample: () => desk({ ...SONG_PLAYING, visual: "silk", shown: true }) },
    { name: "Visualiser in the cover's colours", for: "the light burning in the cover's colours", sample: () => desk({ ...SONG_PLAYING, visual: "water", colours: "cover" }) },
    { name: "Visualiser, a bare song", for: "an ambient song, slower, in the wallpaper's colours", sample: () => desk({ ...BARE_PLAYING, visual: "embers" }) },
    { name: "Song, the least window", for: "the cover smaller, never cropped", sample: () => windowed(SONG_PLAYING, LEAST_WIDTH, LEAST_HEIGHT) },
    { name: "Song, full screen", for: "the cover at `size.cover`, never blown up", sample: () => whole(SONG_PLAYING) },
    { name: "Bare song", for: "no cover: the music symbol on its tile", sample: () => desk(BARE_PLAYING) },
    { name: "Stream, buffering", for: "a stream waiting: the spinner and how much it has", sample: () => desk({ ...SHOWN, file: STREAM, buffering: 0.42, buffered: 0.36, position: 0.33 }) },
    { name: "Broken", for: "can't play: the ErrorBlock in its own red light", sample: () => desk({ ...SHOWN, file: BROKEN, error: true }) },
    { name: "Nothing", for: "opened with no file: a lit sign, what to do", sample: () => desk({ playing: false }) },
    { name: "The narrowing ladder", for: "what gives way as the window narrows, in order", sample: ladder },
    { name: "The control center's widget", for: "the controller's rows: times round the bar, the three centred", sample: shares },
  ],
  sizes: doesntApply("its window's size is Hyprland's; the controller's ladder is under Reflow."),
  states: {
    shown: ["rest", "lit", "pressed", "open", "disabled", "loading", "empty", "error"],
    absent: {
      selected: "a pick is a ticked menu item, under Open",
    },
    rows: nonEmpty([{ name: "A film", cell: (state: Shown) => cell(state) }]),
    looks: {
      rest: "the glass controller over the picture",
      lit: "Play's tube; the bar grown, its head lit",
      pressed: "the bar held: its head brightens",
      open: "a menu on a popover by its button",
      disabled: "previous on the playlist's first file",
      loading: "the spinner and Buffering with its share",
      empty: "idle: the lit sign and what to do",
      error: "the ErrorBlock in red light, no controller",
    },
  },
  focus: doesntApply("nothing takes keyboard focus: mpv's keys drive it, and its buttons light as IconButton's do."),
  content: {
    text: [
      { name: "The title", words: { short: FILM.title, long: LONG_TITLE }, sample: (words: string) => windowed({ ...SHOWN, file: titled(FILM, words) }, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
      { name: "The title, a narrow window", words: { short: FILM.title, long: LONG_TITLE }, sample: (words: string) => windowed({ ...SHOWN, file: titled(FILM, words) }, LEAST_WIDTH, DEFAULT_HEIGHT) },
    ],
    missingPicture: [
      { name: "Song, no cover", for: "the music symbol on a tile", sample: () => windowed(BARE_PLAYING, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
    ],
    counts: [
      { name: "One file", for: "previous and next disabled; no playlist button", sample: () => windowed(SHOWN, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
      { name: "Four files", for: "the corner's playlist button; 2 of 4 by the title", sample: () => windowed(IN_LIST, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
    ],
  },
  motion: { arrives: [APPEARS, SLIDES], leaves: [LEAVES], between: [BLOOMS, MORPHS, GROWS, UNFOLDS, SAYS, UNSAYS] },
  keyboard: {
    keys: [
      { keys: ["Space", "p"], does: "play or pause; the symbol blooms" },
      { keys: ["←", "→"], does: "seek 5 s; the seek message" },
      { keys: ["Shift+←", "Shift+→"], does: "seek 1 s, exactly" },
      { keys: ["↑", "↓"], does: "seek a minute" },
      { keys: ["9", "0", "Wheel"], does: "volume by 2, the hand going up louder; its message" },
      { keys: ["m"], does: "mute; the slider's toggle and the message" },
      { keys: ["[", "]"], does: "speed down and up by a tenth; the speed message" },
      { keys: ["{", "}"], does: "speed halved and doubled" },
      { keys: ["Backspace"], does: "speed back to 1×" },
      { keys: ["j"], does: "the next subtitle, then off" },
      { keys: ["v"], does: "subtitles on or off" },
      { keys: ["n", "N"], does: "a song's next or previous visualiser, Off among them" },
      { keys: ["#"], does: "the next audio track" },
      { keys: ["s"], does: "a screenshot; Screenshot saved" },
      { keys: ["<", ">", "Enter"], does: "the previous or next file" },
      { keys: ["f", "Double click"], does: "full screen on or off" },
      { keys: ["Esc"], does: "a menu or the playlist shut, or full screen left" },
      { keys: ["q"], does: "quits" },
    ],
    order: "Never a stop for Tab: mpv takes every key, and the controller none.",
    lands: "Nothing takes focus; the window keeps it as it opens and plays.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Controller shown", for: "the glass's inset and padding, rows, bar, buttons", sample: () => windowed(IN_LIST, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
      { name: "Song", for: "the cover's size, the words, the levels", sample: () => windowed(SONG_PLAYING, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
      { name: "A message", for: "the strip's width, padding and gaps", sample: () => windowed({ ...PLAYING, message: MESSAGES.volume(0.64) }, DEFAULT_WIDTH, DEFAULT_HEIGHT) },
    ]),
    measures: [
      { part: "Window's corner", token: "radius-content" },
      { part: "Active window's tube", token: "stroke-thin" },
      { part: "Glass's inset, padding", token: "space-s16" },
      { part: "Glass's corners", token: "radius-surface" },
      { part: "Glass's blur", token: "size-lock_blur" },
      { part: "Between its rows", token: "space-s8" },
      { part: "Buttons", token: "size-control_md" },
      { part: "Seek bar's room", token: "size-control_sm" },
      { part: "Tube at rest", token: "size-scrollbar" },
      { part: "Tube lit", token: "size-progress" },
      { part: "Its head", token: "size-slider_anchor" },
      { part: "Between chapters", token: "space-s4" },
      { part: "Volume unfolded", token: "size-bar_label" },
      { part: "Times to the bar", token: "space-s12" },
      { part: "Playlist panel's width", token: "size-panel" },
      { part: "Cover at its largest", token: "size-cover" },
      { part: "Bloom's disc", token: "size-control_lg" },
      { part: "Bloom's growth", token: "scale-bloom" },
      { part: "Message's width", token: "size-rail" },
    ],
    reflow: {
      note: "From 1120 down: the rung each width reaches, in the ladder's order; Play, the bar and ⋯ stay.",
      sample: ladder,
    },
  },
});
