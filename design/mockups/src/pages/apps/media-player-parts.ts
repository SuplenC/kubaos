// The media player's window and its controller: mpv drawing the video, KubaOS's controller (an mpv
// Lua script, drawn with mpv's own ASS vector overlays) drawn over it. A DRAFT: every part here
// that the library lacks is a draft part, named so, built from the library's parts and the tokens.
//
// The player draws in the night scheme in either mode, as the lock does: it sits on a picture. Its
// controller is the library's glass, `surface` at `opacity.glass` over the picture blurred by
// `size.lock_blur`, raised with a rim and a shadow; the picture's own colours light the window
// round it (`ambience`).
//
// The window is built once for what it plays and how it is laid out (`build`); everything a moment
// changes (the time, the pointer over the bar, the volume, play and pause, a menu, a message, the
// playlist's sidebar) changes in place, so nothing flickers, starts its motion over or drops the
// pointer's hover.

import { glass, glassWindow } from "../../lib/glass.ts";
import "./media-player.css";
import { centreOnPixels } from "../../lib/centre.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { hint, withHint } from "../../lib/hint.ts";
import { icon, setMorph } from "../../lib/icon.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { containerPadding, fill, filling, realign, row, stack } from "../../lib/layout.ts";
import { listWith, type ListRowSpec } from "../../lib/list.ts";
import { scroll } from "../../lib/scroll.ts";
import { menu, type MenuEntry } from "../../lib/menu.ts";
import { progressBar } from "../../lib/progress.ts";
import { rollingNumber, type Rolling } from "../../lib/rolling-number.ts";
import { setSlider, setSliderToggled, slider } from "../../lib/slider.ts";
import { spinner } from "../../lib/spinner.ts";
import { advance, clamped, text, type TypeStyle, wrapped } from "../../lib/text.ts";
import { anchorPopover, popover } from "../../lib/tooltip.ts";
import { declare, probe } from "../../lib/probe.ts";
import { durationMs, length, raw } from "../../lib/tokens.ts";
import { ALL, CHROME, settings } from "../../site/settings.ts";
import { Live, neon, type Palette, roleColour, still } from "./media-player-gl.ts";
import { MOODS, QUIET, type Signal, synthetic } from "./media-player-signal.ts";
import { song as analysedSong } from "./media-player-song.ts";
import jungle from "../../assets/audio/drum-and-bass-jungle.opus?url";
import intoTheLight from "../../assets/audio/into-the-light.opus?url";
import sintelPoster from "../../assets/video/sintel-poster.jpg?url";
import SINTEL from "../../assets/video/sintel.json";
import fetched from "virtual:kubaos-film";
import { VISUALS, type VizChoice, vizName, CHOICES } from "./media-player-viz.ts";
import neonLit from "../../../../../branding/photo/neon-sign-lit.jpg?url";
import stacked from "../../../../../branding/photo/neon-stacked-lit.jpg?url";
import purple from "../../assets/lock/shinjuku-purple-hour.jpg?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";

// What plays: a real film and real songs, and stand-ins for the other files, their frames the
// shipped wallpapers. The film is Sintel, cut by `just mockup-film` into a cache outside the
// checkout (`scripts/film.sh`): only its poster frame and its cut file are committed.

export interface Chapter {
  /** Where it starts, a fraction of the file. */
  readonly at: number;
  readonly name: string;
}

export interface MediaFile {
  readonly title: string;
  /** The line under the title: the artist, or where a stream comes from. */
  readonly by?: string;
  /** Its length in seconds. */
  readonly length: number;
  /** A frame of it, or an audio file's cover; none for audio with no cover. */
  readonly picture?: string;
  readonly audio?: boolean;
  readonly chapters?: readonly Chapter[];
  readonly stream?: boolean;
  /** A song's character, which its synthesised analysis plays: driving or ambient. */
  readonly mood?: keyof typeof MOODS;
  /** Its sound, for a song the player really plays: its signal is the sound's own, analysed. */
  readonly source?: string;
  /** Its moving picture, for a film the player really plays on the live stage; `picture` is its poster. */
  readonly video?: string;
  /** Its audio tracks and subtitles, for a file with its own; any other shows the film's. */
  readonly tracks?: Tracks;
}

/** A subtitle track: its name in the Tracks menu, a line of it, and its file where it really plays. */
export interface Subtitle {
  readonly name: string;
  /** A line of it the fixed samples show, the same on every run, with the film fetched or not. */
  readonly sample: string;
  /** Its WebVTT file, for the film really playing: its lines show at their times. */
  readonly src?: string;
}

/** A file's audio tracks and subtitles, in the order the Tracks menu lists them. */
export interface Tracks {
  readonly audio: readonly string[];
  readonly subtitles: readonly Subtitle[];
}

/** The cut's length in seconds: its frames, `start` to `end` of the source, at its rate. */
const FILM_LENGTH = (SINTEL.end - SINTEL.start) / SINTEL.rate;

/**
 * The film's tracks as its cut file names them, the mkv's own (`assets/video/sintel.json`), each
 * subtitle with a line of it for the fixed samples.
 */
const FILM_TRACKS: Tracks = { audio: [SINTEL.audio], subtitles: SINTEL.subtitles.map(({ name, sample }) => ({ name, sample })) };

/**
 * The film (`assets/video/CREDITS.md`) as the fixed samples show it, from the committed cut file
 * alone, so they read the same with the film fetched or not: its title and maker are its credit,
 * so they show wherever it shows. Its chapters start at the cut's scene changes. Its poster is
 * its frame at 0.35, where the samples stand, so a fixed sample shows the film as it is there.
 */
export const FILM: MediaFile = {
  title: SINTEL.title,
  by: SINTEL.by,
  length: FILM_LENGTH,
  picture: sintelPoster,
  chapters: SINTEL.chapters.map(({ at, name }) => ({ at: at / SINTEL.rate / FILM_LENGTH, name })),
  tracks: FILM_TRACKS,
};

/**
 * The film the live stage plays: the fetched cut (`just mockup-film`), its length, chapters and
 * tracks as its manifest has them; without it, the fixed one, whose poster plays by the clock.
 */
export const STAGE_FILM: MediaFile = fetched ? fetchedFilm(fetched) : FILM;

/** The fetched cut as the player plays it, its files where the dev server serves them. */
function fetchedFilm(cut: NonNullable<typeof fetched>): MediaFile {
  return {
    ...FILM,
    title: cut.title,
    by: cut.by,
    length: cut.length,
    chapters: cut.chapters.map(({ at, name }) => ({ at: at / cut.length, name })),
    video: cut.base + cut.video,
    tracks: {
      audio: cut.audio.map(({ name }) => name),
      subtitles: cut.subtitles.map(({ lang, name, file }) => ({ name, sample: SINTEL.subtitles.find((each) => each.lang === lang)?.sample ?? "", src: cut.base + file })),
    },
  };
}

/** Whether the film is fetched, so the live stage plays it. */
export const FILM_FETCHED = fetched !== null;

export const LONG_TITLE = "Sintel, the search for Scales across the wastes and the mountains, the extended cut with commentary";

export const PLAYLIST: readonly MediaFile[] = [
  { title: "Tokyo Tower at the blue hour", length: 412, picture: tower },
  FILM,
  { title: "Shinjuku, the purple hour", length: 1288, picture: purple },
  { title: "Neon sign, warming up", length: 96, picture: neonLit },
];

/**
 * The real songs (`assets/audio/CREDITS.md`): each one's title and artist are its credit, so they
 * show wherever it shows. Their covers are stand-ins of KubaOS's own. `SONG` is the first of them,
 * the one the samples show.
 */
export const SONG: MediaFile = { title: "Drum and Bass Jungle", by: "Echoes of Lumen", length: 198.87, picture: stacked, audio: true, source: jungle };
const INTO_THE_LIGHT: MediaFile = { title: "Into the Light", by: "BerryDeep", length: 165.97, picture: neonLit, audio: true, source: intoTheLight };
export const BARE_SONG: MediaFile = { title: "field-recording-03", length: 1841, audio: true, mood: "ambient" };

/** A local song's sidecar, `<name>.json` beside it: what to show instead of the file's name, and its length, s. */
interface Sidecar {
  readonly title?: string;
  readonly by?: string;
  readonly length?: number;
}

// More songs to try on this machine: every audio file in `assets/audio/local/`, which git ignores,
// as their licences may not allow passing the files on (`assets/audio/README.md`). None on a fresh
// checkout or CI.
const LOCAL_AUDIO = import.meta.glob<string>("../../assets/audio/local/*.{opus,ogg,oga,mp3,flac,wav,m4a,aac,webm}", { eager: true, query: "?url", import: "default" });
const LOCAL_SIDECARS = import.meta.glob<Sidecar>("../../assets/audio/local/*.json", { eager: true, import: "default" });

/** A file's name without its folder or its extension. */
function stemOf(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1).replace(/\.[^.]+$/, "");
}

/**
 * The local songs, in file-name order. A song whose sidecar gives no length takes the file's own,
 * read from its header as the page loads; until then, and should it fail, it shows none.
 */
const LOCAL_SONGS: readonly MediaFile[] = Object.keys(LOCAL_AUDIO)
  .sort((a, b) => stemOf(a).localeCompare(stemOf(b), undefined, { numeric: true }))
  .map((path) => {
    const source = LOCAL_AUDIO[path]!;
    const stem = stemOf(path);
    const sidecar = LOCAL_SIDECARS[path.replace(/\.[^.]+$/, ".json")] ?? {};
    let length = sidecar.length ?? 0;
    if (!length) {
      const probe = new Audio();
      probe.preload = "metadata";
      probe.addEventListener("loadedmetadata", () => (length = Number.isFinite(probe.duration) ? probe.duration : 0), { once: true });
      probe.src = source;
    }
    return {
      // Without a sidecar, the name: a leading order number such as `01-` left out, dashes and underscores as spaces.
      title: sidecar.title ?? stem.replace(/^\d+[-_ .]+/, "").replace(/[-_]+/g, " "),
      ...(sidecar.by ? { by: sidecar.by } : {}),
      get length() {
        return length;
      },
      // A stand-in cover of KubaOS's own, as a local file's own is not read.
      picture: stacked,
      audio: true,
      source,
    };
  });

/** The songs played one after the other: the two real ones, any local ones, then a silent one with a synthesised signal. */
export const SONGS: readonly MediaFile[] = [SONG, INTO_THE_LIGHT, ...LOCAL_SONGS, BARE_SONG];
export const STREAM: MediaFile = { title: "Tokyo walk, live", by: "a network stream", length: 5400, picture: purple, stream: true };
export const BROKEN: MediaFile = { title: "holiday-2019.mkv", length: 0 };

/** The live stage's playlist: the fixed one, the film in it the one the stage plays. */
export const STAGE_PLAYLIST: readonly MediaFile[] = PLAYLIST.map((file) => (file === FILM ? STAGE_FILM : file));

/** The desktop's wallpaper behind the player's samples: a picture the player already shows. */
export const DESKTOP = purple;

/** A file's tracks: its own, or the film's for a file with none of its own. */
export function tracksOf(file: MediaFile | undefined): Tracks {
  return file?.tracks ?? FILM_TRACKS;
}

/** The film's subtitle track by its language's code (`assets/video/sintel.json`), for the samples. */
export function subtitleIndex(lang: string): number {
  return SINTEL.subtitles.findIndex((each) => each.lang === lang);
}

export const SPEEDS = [0.5, 0.75, 1, 1.25, 1.5, 2] as const;

/** What the controller has open over it. */
export type Open = "tracks" | "more" | "speed" | "visualiser";

/** Whose colours a visualiser burns in: the wallpaper's (the night scheme's accents) or the cover's. */
export type Colours = "wallpaper" | "cover";

export interface Message {
  readonly icon: string;
  /** A level with its bar, as the shell's OSD has it; none for a line of words. */
  readonly fraction?: number;
  readonly words: string;
  /** The widest the words read, so the bar keeps one length. */
  readonly widest?: string;
}

export interface PlayerState {
  /** What plays; none while idle. */
  readonly file?: MediaFile;
  readonly playing?: boolean;
  /** The controller shown: the pointer moved, or paused. */
  readonly shown?: boolean;
  /** Where it is, a fraction. */
  readonly position?: number;
  /** How far ahead it has read, a fraction. */
  readonly buffered?: number;
  readonly volume?: number;
  readonly muted?: boolean;
  readonly speed?: number;
  /** The pointer over the seek bar, a fraction; dragging it, held. */
  readonly hover?: number;
  readonly dragging?: boolean;
  /** A frame of the file over the bar, under the pointer, rather than the time alone. */
  readonly preview?: boolean;
  readonly open?: Open;
  /** The playlist's sidebar open. */
  readonly sidebar?: boolean;
  readonly message?: Message;
  readonly buffering?: number;
  readonly error?: boolean;
  /** Its place in the playlist, from 0, and how many there are. */
  readonly index?: number;
  readonly count?: number;
  /** The files it plays one after the other, when it isn't `PLAYLIST`. */
  readonly list?: readonly MediaFile[];
  readonly audioTrack?: number;
  /** The subtitle shown, by index; none for off. */
  readonly subtitle?: number | null;
  /** The line the subtitles show now. */
  readonly line?: string;
  /** The length's place shows the time left. */
  readonly remaining?: boolean;
  readonly full?: boolean;
  /** The pointer on a control, held lit: the samples' view of the lit state. */
  readonly lit?: "play" | "bar" | "volume";
  /** A song's visualiser, or Off: its still cover. */
  readonly visual?: VizChoice;
  readonly colours?: Colours;
}

/** A change to a player's state: a part set to undefined is cleared. */
export type Patch = { readonly [K in keyof PlayerState]?: PlayerState[K] | undefined };

/** What the player tells its owner a person did with its own controls. */
export interface Hooks {
  /** The volume's slider moved, to 0 … 1. */
  readonly volume?: (value: number) => void;
  /** The volume's mute turned on or off, by its button. */
  readonly muted?: (on: boolean) => void;
  /** A file of the playlist taken in its sidebar, by its index. */
  readonly take?: (index: number) => void;
  /** The window is the live stage's: its visualiser plays; anywhere else it is a still. */
  readonly live?: boolean;
  /** Where the song playing out loud is, in seconds, which the visualiser follows; none if silent. */
  readonly time?: () => number | null;
  /** The film playing on the live stage reached its end. */
  readonly ended?: () => void;
}

/** A length in seconds as the player reads one: 4:05, 1:02:10. */
export function clock(seconds: number): string {
  const whole = Math.max(0, Math.round(seconds));
  const h = Math.floor(whole / 3600);
  const m = Math.floor((whole % 3600) / 60);
  const s = String(whole % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
}

/** The widest time a file of `length` reads, in its own digits' shape. */
function widestClock(length: number): string {
  return clock(length).replace(/\d/g, "0");
}

/**
 * A room for words that change in place, as wide as the widest they say: its advance, up to the
 * whole pixel so what follows starts on one, as a badge's room is (`lib/badge.ts`).
 */
function roomFor(style: TypeStyle, widest: string): string {
  return `round(up, calc(${advance(style, widest)} * var(--px) - 0.001px), 1px)`;
}

/** A speed as the player says it: 1×, 1.25×. */
function speedWords(speed: number): string {
  return `${Number(speed.toFixed(2))}×`;
}

/** The chapter at `fraction`. */
function chapterAt(file: MediaFile, fraction: number): Chapter | undefined {
  return [...(file.chapters ?? [])].reverse().find((chapter) => chapter.at <= fraction + 1e-6);
}

/** The volume's symbol for its level, the shell's OSD's steps. */
function volumeSymbol(level: number, muted = false): string {
  return muted || level <= 0 ? "volume_off" : level < 0.34 ? "volume_mute" : level < 0.67 ? "volume_down" : "volume_up";
}

/** The volume's symbols by level for its slider, lowest first: the OSD's steps above nothing. */
const VOLUME_LEVELS = ["volume_mute", "volume_down", "volume_up"] as const;

/** The visualiser a state plays: Off where it plays no song; with reduce motion, calm Water or Off. */
function choiceOf(state: PlayerState): VizChoice {
  if (!state.file?.audio) return "off";
  // With reduce motion on, Water stays, as calm water at a slow drift; the others give way to the cover.
  if (settings().reduced) return state.visual === "water" ? "water" : "off";
  return state.visual ?? "off";
}

/** `r g b` channels as 0 … 1. */
function channels(words: string): [number, number, number] | null {
  const parts = words.trim().split(/\s+/).map(Number);
  return parts.length === 3 && parts.every((each) => Number.isFinite(each)) ? [parts[0]! / 255, parts[1]! / 255, parts[2]! / 255] : null;
}

/** The wallpaper's colours, as the night scheme on `root` has them: its accent and its cold one. */
function wallpaperPalette(root: Element): Palette {
  return { primary: neon(roleColour(root, "primary")), cold: neon(roleColour(root, "cold")) };
}

/**
 * The cover's colours: its brightest cell, and of its corners the one furthest from it, as the
 * ambience read them. The wallpaper's until the cover is read.
 */
function coverPalette(root: HTMLElement): Palette {
  const cells = ["--mp-middle", "--mp-corner-0", "--mp-corner-1", "--mp-corner-2", "--mp-corner-3"].map((name) => channels(root.style.getPropertyValue(name))).filter((each): each is [number, number, number] => each !== null);
  if (cells.length < 2) return wallpaperPalette(root);
  const glow = (c: readonly number[]) => Math.max(...c);
  const primary = cells.reduce((best, each) => (glow(each) > glow(best) ? each : best));
  const far = (c: readonly number[]) => c.reduce((sum, value, i) => sum + Math.abs(value - primary[i]!), 0);
  const cold = cells.reduce((best, each) => (far(each) > far(best) ? each : best));
  return { primary: neon(primary), cold: neon(cold) };
}

/** A duration token in milliseconds under the speed setting; nothing with reduce motion. */
function ms(name: string): number {
  return settings().reduced ? 0 : durationMs(name) / settings().speed;
}

// The picture's own light.

/** The colours a picture lights the window with, each `r g b`: its edges, its middle, its corners. */
export interface Ambience {
  readonly left: string;
  readonly right: string;
  readonly top: string;
  readonly bottom: string;
  readonly middle: string;
  readonly corners: readonly [string, string, string, string];
}

const ambiences = new Map<string, Ambience>();

/** The few cells a picture is drawn down to for its light, a side. */
const CELLS = 4;
let cellPen: CanvasRenderingContext2D | null | undefined;

/**
 * The colours `source` lights the window with, worked out from the picture itself: drawn down to a
 * few cells, each edge's cells and each corner averaged. In mpv the script reads them from a
 * scaled-down frame (`screenshot-raw` at a few pixels).
 */
function sample(source: CanvasImageSource): Ambience | null {
  const cells = CELLS;
  if (cellPen === undefined) {
    const canvas = document.createElement("canvas");
    canvas.width = cells;
    canvas.height = cells;
    cellPen = canvas.getContext("2d", { willReadFrequently: true });
  }
  const pen = cellPen;
  if (!pen) return null;
  let data: Uint8ClampedArray;
  try {
    pen.drawImage(source, 0, 0, cells, cells);
    data = pen.getImageData(0, 0, cells, cells).data;
  } catch {
    return null;
  }
  // Each region's light is its brightest cell: a neon sign on a dark wall lights the room with the
  // sign's colour, not the wall's.
  const average = (points: readonly (readonly [number, number])[]) => {
    const glow = (x: number, y: number) => Math.max(data[(y * cells + x) * 4]!, data[(y * cells + x) * 4 + 1]!, data[(y * cells + x) * 4 + 2]!);
    const [x, y] = points.reduce((best, each) => (glow(each[0], each[1]) > glow(best[0], best[1]) ? each : best));
    return [0, 1, 2].map((c) => data[(y * cells + x) * 4 + c]!).join(" ");
  };
  const span = [...Array(cells).keys()];
  const made: Ambience = {
    left: average(span.map((y) => [0, y])),
    right: average(span.map((y) => [cells - 1, y])),
    top: average(span.map((x) => [x, 0])),
    bottom: average(span.map((x) => [x, cells - 1])),
    middle: average([1, 2].flatMap((x) => [1, 2].map((y) => [x, y] as const))),
    corners: [average([[0, 0], [1, 0], [0, 1]]), average([[3, 0], [2, 0], [3, 1]]), average([[0, 3], [1, 3], [0, 2]]), average([[3, 3], [2, 3], [3, 2]])],
  };
  return made;
}

/** The colours `picture` lights the window with, kept by picture, so they are worked out once. */
function ambienceOf(picture: HTMLImageElement): Ambience | null {
  const known = ambiences.get(picture.src);
  if (known) return known;
  if (picture.naturalWidth === 0) return null;
  const made = sample(picture);
  if (made) ambiences.set(picture.src, made);
  return made;
}

/** Lights `room` with the colours `made`. */
function paint(room: HTMLElement, made: Ambience | null): void {
  if (!made) return;
  room.style.setProperty("--mp-left", made.left);
  room.style.setProperty("--mp-right", made.right);
  room.style.setProperty("--mp-top", made.top);
  room.style.setProperty("--mp-bottom", made.bottom);
  room.style.setProperty("--mp-middle", made.middle);
  made.corners.forEach((each, i) => room.style.setProperty(`--mp-corner-${i}`, each));
  room.dataset.lit = "";
}

/** Lights `room` with `picture`'s colours as soon as the picture is in. */
function light(room: HTMLElement, picture: HTMLImageElement): void {
  // Read once the picture is decoded, never before: a picture loaded but not yet decoded draws nothing.
  picture.decode().then(() => paint(room, ambienceOf(picture)), () => undefined);
}

// The narrowing ladder: what the controller gives way in, in order, as its window narrows.

export interface Rung {
  readonly says: string;
}

/**
 * The order the controller gives way in. What only shows state goes before what acts; what has a
 * key goes before what has none; Play, the bar and ⋯ stay to the last. The playlist is never in
 * it: its button is the window's top corner's at every width.
 */
export const RUNGS: readonly Rung[] = [
  { says: "Volume stays folded" },
  { says: "Tracks join ⋯" },
  { says: "Length goes; elapsed stays" },
  { says: "Previous and next join ⋯" },
  { says: "Elapsed goes; the hint keeps it" },
  { says: "Volume joins ⋯" },
  { says: "Full screen joins ⋯" },
];

const TRACKS_JOIN = 2;
const LENGTH_GOES = 3;
const SKIPS_JOIN = 4;
const ELAPSED_GOES = 5;
const VOLUME_JOINS = 6;
const FULL_JOINS = 7;

/** Below this width the playlist's sidebar takes the whole window: `size.app_min_width`. */
const SIDEBAR_COVERS = "size-app_min_width";

// The parts.

/** `part`, gone from the rung `gone` on, and shown only from the rung `from` on. */
function rungs<E extends HTMLElement>(part: E, gone?: number, from?: number): E {
  if (gone !== undefined) part.dataset.gone = String(gone);
  if (from !== undefined) part.dataset.from = String(from);
  return part;
}

/**
 * DRAFT PART, the seek bar: the library's progress bar as a lit tube. Its track is
 * `size.scrollbar` at rest and grows to `size.progress` under the pointer; what a stream has read
 * ahead is `outline`; chapters stand `space.s4` apart; the played part is the accent, glowing as a lit
 * tube does (`focus.near`, `focus.far`), and its head lights to the core under the pointer and
 * brighter still while held. Built once, and moved in place (`moveBar`).
 */
function seekBar(file: MediaFile): HTMLElement {
  const bar = document.createElement("span");
  bar.className = "mp-seek";
  const glow = document.createElement("span");
  glow.className = "mp-seek-glow";
  const track = document.createElement("span");
  track.className = "mp-seek-track";
  const chapters = file.chapters && file.chapters.length > 1 ? file.chapters : [{ at: 0, name: "" }];
  chapters.forEach((chapter, i) => {
    const part = document.createElement("span");
    part.className = "mp-seek-part";
    part.style.flexGrow = String((chapters[i + 1]?.at ?? 1) - chapter.at);
    const read = document.createElement("span");
    read.className = "mp-seek-read";
    const played = document.createElement("span");
    played.className = "mp-seek-played";
    part.append(read, played);
    track.append(part);
  });
  const head = document.createElement("span");
  head.className = "mp-seek-head";
  bar.append(glow, track, head);
  return bar;
}

/**
 * Moves `bar`, a seek bar of `file`, to `position`, read ahead to `buffered`. Only a stream shows
 * what it has read: a file on the disk is read whole at once, so its read part would fill the
 * track and read as played.
 */
function moveBar(bar: HTMLElement, file: MediaFile, position: number, buffered: number): void {
  const chapters = file.chapters && file.chapters.length > 1 ? file.chapters : [{ at: 0, name: "" }];
  const read = file.stream ? Math.max(position, buffered) : position;
  bar.style.setProperty("--position", String(position));
  // The head stands on the played part's end, past the chapters' gaps before it, once laid out.
  const parts = [...bar.querySelectorAll<HTMLElement>(".mp-seek-part")];
  const current = chapters.findIndex((chapter, i) => position >= chapter.at && position < (chapters[i + 1]?.at ?? 1 + 1e-9));
  const part = parts[Math.max(0, current)];
  if (part && part.offsetWidth > 0) {
    const start = chapters[Math.max(0, current)]!.at;
    const span = (chapters[Math.max(0, current)! + 1]?.at ?? 1) - start;
    bar.style.setProperty("--head", `${part.offsetLeft + part.offsetWidth * Math.min(1, (position - start) / span)}px`);
  }
  parts.forEach((part, i) => {
    const start = chapters[i]!.at;
    const span = (chapters[i + 1]?.at ?? 1) - start;
    const share = (at: number) => String(Math.max(0, Math.min(1, (at - start) / span)));
    part.style.setProperty("--read", share(read));
    part.style.setProperty("--played", share(position));
  });
}

/** What the pointer over the bar says: its time, and its chapter's name. */
function tipWords(file: MediaFile, at: number): string {
  const chapter = chapterAt(file, at);
  const time = clock(at * file.length);
  return chapter && chapter.name !== "" ? `${time} · ${chapter.name}` : time;
}

/** Whether the pointer over `file`'s bar shows a frame of it: a film with a picture, never a song. */
function previews(file: MediaFile, preview: boolean): boolean {
  return preview && !file.audio && file.picture !== undefined;
}

/**
 * What the pointer over the bar shows, over the point it is at: its time and chapter in a hint, or
 * on a popover a frame with the words under it, one surface. `moving` is the film's own frame at
 * the time pointed at, where the film really plays; anywhere else the frame is its picture. It
 * rises into place as it arrives (`mp-tip-in`), then follows the pointer.
 */
function seekTip(file: MediaFile, at: number, preview: boolean, moving: HTMLVideoElement | null): HTMLElement {
  // A bubble over the bar, out of its flow: held to its own boxes, not the bar's.
  const tip = probe(document.createElement("span"), { role: "box" }, { floats: true });
  tip.className = "mp-seek-tip";
  tip.style.setProperty("--at", String(at));
  if (previews(file, preview)) {
    let frame: HTMLElement;
    if (moving) frame = moving;
    else {
      const still = document.createElement("img");
      still.src = file.picture!;
      still.alt = "";
      frame = still;
    }
    frame.classList.add("mp-preview-frame");
    const words = text(tipWords(file, at), "label_number");
    words.classList.add("mp-tip-words");
    tip.append(popover(4, stack("s4", "centre", frame, words)));
    tip.dataset.preview = "";
  } else tip.append(hint(tipWords(file, at), { shown: true }));
  return tip;
}

/** A Text icon button of the controller's one size, its hint above it: the controller is at the window's foot. */
function tool(name: string, label: string, opens?: string): HTMLButtonElement {
  const made = iconButton({ icon: name, label, kind: "Text", size: "Md", hintAbove: true });
  // What it opens, so its popover can stand by it.
  if (opens) made.dataset.opens = opens;
  made.dataset.does = label;
  return made;
}

/** Names `button` for what a press does now, in its hint too. */
function rename(button: HTMLElement, label: string): void {
  if (button.getAttribute("aria-label") === label) return;
  button.setAttribute("aria-label", label);
  button.dataset.does = label;
  button.querySelector(":scope > .k-hint")?.remove();
  withHint(button, label, { side: "above" });
}

/**
 * Play and pause: a Filled icon button, the accent's one fill on the controller, its symbol
 * morphing between the two (`morphIcon`'s `play_pause`): the triangle's halves part into the bars.
 */
function playButton(playing: boolean, lit = false): HTMLButtonElement {
  const made = iconButton({ icon: "play_arrow", label: playing ? "Pause" : "Play", kind: "Filled", size: "Md", hintAbove: true, morph: { pair: "play_pause", on: playing }, ...(lit ? { pinned: "lit" as const } : {}) });
  made.classList.add("mp-play");
  made.dataset.does = playing ? "Pause" : "Play";
  return made;
}

/**
 * The volume: the library's compact Slider with its mute toggle, its symbol stepping with the level
 * (`levels`), folded to its circle (the mute button) and unfolding to `size.bar_label` under the
 * pointer, over `medium`.
 */
function volume(state: PlayerState, hooks: Hooks): HTMLElement {
  const made = slider({
    icon: "volume_up",
    size: "Compact",
    value: state.volume ?? 0.62,
    levels: VOLUME_LEVELS,
    toggle: { icon: "volume_off", words: "Muted", label: "Mute", toggledLabel: "Unmute", on: state.muted ?? false, ...(hooks.muted ? { changed: hooks.muted } : {}) },
    ...(hooks.volume ? { changed: hooks.volume } : {}),
  });
  const room = document.createElement("span");
  room.className = "mp-volume";
  room.toggleAttribute("data-open", state.lit === "volume");
  room.append(made);
  return room;
}

/** The menu ⋯ opens: what has joined it as the window narrowed, then speed and the screenshot. */
function moreEntries(state: PlayerState, rung: number): MenuEntry[] {
  const joined: MenuEntry[] = [];
  const many = (state.count ?? 1) > 1;
  if (rung >= SKIPS_JOIN && many) joined.push({ label: "Previous", icon: "skip_previous", keys: "<" }, { label: "Next", icon: "skip_next", keys: ">" });
  if (rung >= TRACKS_JOIN && !state.file?.audio) joined.push({ label: "Audio and subtitles", icon: "subtitles", submenu: true });
  if (rung >= VOLUME_JOINS) joined.push({ label: state.muted ? "Unmute" : "Mute", icon: volumeSymbol(state.volume ?? 0, state.muted), keys: "M" });
  if (rung >= FULL_JOINS) joined.push({ label: state.full ? "Leave full screen" : "Full screen", icon: state.full ? "fullscreen_exit" : "fullscreen", keys: "F" });
  const visualiser: MenuEntry[] = state.file?.audio ? [{ label: `Visualiser · ${vizName(choiceOf(state))}`, icon: "wave_sine", submenu: true }] : [];
  return [...joined, ...(joined.length > 0 ? (["separator"] as const) : []), ...visualiser, { label: `Speed · ${speedWords(state.speed ?? 1)}`, icon: "speed", submenu: true }, { label: "Screenshot", icon: "photo_camera", keys: "S" }];
}

/** A popover's menu headed by its group's name: the tracks' two groups. */
function group(heading: string, entries: readonly MenuEntry[]): HTMLElement {
  return menu([{ heading }, ...entries]);
}

/** What a button of the controller opens, on a popover by it. */
function opened(state: PlayerState, rung: number): HTMLElement {
  const open = state.open!;
  let content: HTMLElement;
  if (open === "tracks") {
    const audio = state.audioTrack ?? 0;
    const subtitle = state.subtitle === undefined ? null : state.subtitle;
    const tracks = tracksOf(state.file);
    content = stack(
      "s16",
      "start",
      group("Audio", tracks.audio.map((label, i) => ({ label, checked: i === audio }))),
      group("Subtitles", [{ label: "Off", checked: subtitle === null }, ...tracks.subtitles.map(({ name }, i) => ({ label: name, checked: i === subtitle }))]),
    );
  } else if (open === "visualiser") {
    const now = choiceOf(state);
    content = menu(CHOICES.map((choice) => ({ label: vizName(choice), checked: choice === now })));
  } else if (open === "speed") {
    const speed = state.speed ?? 1;
    content = menu(SPEEDS.map((each) => ({ label: each === 1 ? "1× · normal" : speedWords(each), checked: Math.abs(each - speed) < 1e-6 })));
  } else content = menu(moreEntries(state, rung));
  const card = popover(4, content);
  const room = document.createElement("span");
  room.className = "mp-popover";
  room.dataset.for = open;
  room.append(card);
  return room;
}

/** Each rolling clock by its element, so the time moving on rolls it in place. */
const rollers = new WeakMap<HTMLElement, Rolling>();

/** The time a file of `length` reads in its rolling clock: figures roll, colons stay. */
function rollingClock(seconds: number, length: number, color: string, minus = false): Rolling {
  const widest = `${minus ? "−" : ""}${widestClock(length)}`;
  const made = rollingNumber({ value: Math.round(seconds), widest, format: (value) => `${minus ? "−" : ""}${clock(value)}`, style: "label_number", color });
  rollers.set(made.element, made);
  return made;
}

/** The title row: the file's name, the chapter it is in, and its place in the playlist. */
function titleRow(state: PlayerState): HTMLElement {
  const file = state.file!;
  const name = declare(clamped(file.title, "body_strong"), { variable: true });
  name.classList.add("mp-title");
  const after: HTMLElement[] = [];
  const chapter = chapterAt(file, state.position ?? 0);
  const said = text(chapter?.name ?? "", "small", "on_surface_variant");
  said.classList.add("mp-chapter");
  after.push(said);
  if (file.by) after.push(text(file.by, "small", "on_surface_variant"));
  if ((state.count ?? 1) > 1) after.push(text(`${(state.index ?? 0) + 1} of ${state.count}`, "small", "on_surface_variant"));
  const line = row("s8", "centre", name, ...after);
  line.classList.add("mp-title-row");
  return line;
}

/** The parts of a built controller its owner moves in place. */
interface Built {
  readonly box: HTMLElement;
  readonly bar: HTMLElement;
  readonly elapsed: Rolling | null;
  readonly length: HTMLElement | null;
  readonly play: HTMLButtonElement;
  readonly volume: HTMLElement;
}

/**
 * DRAFT PART, the controller: the title, the seek bar with its times, and the buttons, on a plate of
 * the library's glass raised over the picture, its rows laid out as the control center's media
 * widget lays out its own: the times round the bar, the transport centred. Every part of every rung is
 * built once, each marked with the rung it goes at (`data-gone`) or comes at (`data-from`); the
 * window's `data-r1`… say which rungs are taken, so finding the rung redraws nothing. Each part
 * carries its place in the arrival's stagger (`--i`).
 */
function controller(state: PlayerState, hooks: Hooks): Built {
  const file = state.file!;
  const position = state.position ?? 0;
  const many = (state.count ?? 1) > 1;
  const at = state.index ?? 0;
  const box = glass(document.createElement("span"));
  box.classList.add("mp-controller");

  const bar = filling(seekBar(file));
  moveBar(bar, file, position, state.buffered ?? position);
  const elapsed = rollingClock(position * file.length, file.length, "on_surface");
  // The rolling clock is as wide as the longest time the file reads, so the bar beside it never moves.
  const elapsedBox = row(0, "centre", elapsed.element);
  elapsedBox.classList.add("mp-time");
  const length = lengthOf(state);
  const barRow = row("s12", "centre", rungs(elapsedBox, ELAPSED_GOES), bar, rungs(length, LENGTH_GOES));
  barRow.classList.add("mp-bar-row");

  const previous = rungs(tool("skip_previous", "Previous"), SKIPS_JOIN);
  previous.disabled = !many || at === 0;
  const next = rungs(tool("skip_next", "Next"), SKIPS_JOIN);
  next.disabled = !many || at === (state.count ?? 1) - 1;
  const play = playButton(state.playing ?? false, state.lit === "play");
  // Folded, the slider is its circle: the mute button. It stays folded from the first rung on (`data-r1`).
  const level = volume(state, hooks);
  const left: HTMLElement[] = [rungs(level, VOLUME_JOINS)];
  const right: HTMLElement[] = [];
  if (!file.audio) right.push(rungs(tool("subtitles", "Audio and subtitles", "tracks"), TRACKS_JOIN));
  right.push(tool("more", "More", "more"));
  right.push(rungs(tool(state.full ? "fullscreen_exit" : "fullscreen", state.full ? "Leave full screen" : "Full screen"), FULL_JOINS));

  const transport = row("s8", "centre", previous, play, next);
  transport.classList.add("mp-transport");
  const leftGroup = row("s8", "centre", ...left);
  leftGroup.classList.add("mp-left");
  const rightGroup = row("s8", "centre", ...right);
  rightGroup.classList.add("mp-right");
  const buttons = document.createElement("span");
  buttons.append(leftGroup, transport, rightGroup);
  buttons.classList.add("mp-buttons");
  // The arrival's stagger: the rows, then each button, a `swap_lag` apart.
  [...buttons.querySelectorAll<HTMLElement>(".k-button, .mp-volume")].forEach((each, i) => each.style.setProperty("--i", String(i + 2)));
  barRow.style.setProperty("--i", "1");

  const rows: HTMLElement[] = [];
  // A song has its title set large over the controller: the controller doesn't say it again.
  if (!file.audio) rows.push(titleRow(state));
  rows.push(barRow, buttons);
  box.append(stack("s8", "start", ...rows));
  return { box, bar, elapsed, length, play, volume: level.querySelector<HTMLElement>(".k-slider")! };
}

/**
 * The length, or with `remaining` the time left counting down: a press of it swaps the two, its
 * hint saying which a press shows.
 */
function lengthOf(state: PlayerState): HTMLElement {
  const file = state.file!;
  const box = document.createElement("span");
  box.className = "mp-time mp-length";
  box.setAttribute("role", "button");
  if (file.stream) box.append(text("live", "label_number", "on_surface_variant"));
  else if (state.remaining) box.append(rollingClock(file.length * (1 - (state.position ?? 0)), file.length, "on_surface_variant", true).element);
  else box.append(text(clock(file.length), "label_number", "on_surface_variant"));
  withHint(box, state.remaining ? "Show the length" : "Show the time left", { side: "above", held: "end" });
  return box;
}

/**
 * DRAFT PART, a message from a key: the shell's OSD strip (`shell/osd`) as it is, its symbol in
 * `primary`, the bar and the level, or a line of words; hung from the window's top edge, since the
 * controller has its foot. It comes down and fades in, and goes back up fading.
 */
export function message(said: Message): HTMLElement {
  const parts: Element[] = [icon(said.icon, "var(--icon-s24)", "var(--color-primary)")];
  if (said.fraction !== undefined) {
    parts.push(filling(progressBar({ fraction: said.fraction })));
    const room = row(0, "centre", fill(), text(said.words, "body_strong"));
    room.style.width = roomFor("body_strong", said.widest ?? said.words);
    parts.push(room);
  } else parts.push(text(said.words, "body_strong"));
  const strip = row("s12", "centre", ...parts);
  strip.classList.add("mp-message");
  strip.toggleAttribute("data-words", said.fraction === undefined);
  return strip;
}

export const MESSAGES = {
  seek: (state: PlayerState, forward: boolean): Message => ({ icon: forward ? "fast_forward" : "fast_rewind", fraction: state.position ?? 0, words: clock((state.position ?? 0) * (state.file?.length ?? 0)), widest: widestClock(state.file?.length ?? 0) }),
  volume: (level: number, muted = false): Message => ({ icon: volumeSymbol(level, muted), fraction: muted ? 0 : level, words: muted ? "Muted" : String(Math.round(level * 100)), widest: "Muted" }),
  speed: (speed: number): Message => ({ icon: "speed", words: `Speed ${speedWords(speed)}` }),
  screenshot: (): Message => ({ icon: "photo_camera", words: "Screenshot saved" }),
  subtitles: (index: number | null, file?: MediaFile): Message => ({ icon: "subtitles", words: index === null ? "Subtitles off" : (tracksOf(file).subtitles[index]?.name ?? "") }),
  audio: (index: number, file?: MediaFile): Message => ({ icon: "music", words: tracksOf(file).audio[index] ?? "" }),
  visual: (choice: VizChoice): Message => ({ icon: "wave_sine", words: `Visualiser · ${vizName(choice)}` }),
  full: (on: boolean): Message => ({ icon: on ? "fullscreen" : "fullscreen_exit", words: on ? "Full screen" : "Windowed" }),
} as const;

/** The playlist's button: the window's top corner's. */
function playlistButton(): HTMLButtonElement {
  const made = iconButton({ icon: "playlist", label: "Playlist", kind: "Text", size: "Md" });
  made.dataset.opens = "sidebar";
  made.dataset.does = "Playlist";
  made.classList.add("mp-playlist-button");
  return made;
}

/** A playlist's file as its row says it: the one playing ticked, its state its symbol; each its length. */
function playlistRow(file: MediaFile, i: number, state: PlayerState): ListRowSpec {
  const at = state.index ?? 0;
  return { title: file.title, subtitle: file.by ? `${file.by} · ${clock(file.length)}` : clock(file.length), icon: i === at ? (state.playing ? "play_arrow" : "pause") : file.audio ? "music" : "movie", tile: true, accent: i === at, checked: i === at };
}

/** A playlist panel's own redraw: the rows and the count that changed, in place. */
const playlistUpdates = new WeakMap<Element, (state: PlayerState) => void>();

/**
 * DRAFT PART, the playlist's panel: the controller's glass docked to the window's end edge, the
 * window's whole height, flush with its edges and corners, over the picture, which goes on at its
 * size beneath. Its head says "Playlist" and which of how many plays; its files fill the rest in a
 * list (the one playing ticked, each with its length), scrolling when they don't fit; a press plays
 * one. It slides in from the end edge over `long` and back over `medium`, the controller's end
 * drawing in beside it; `size.panel` wide, the whole window below `size.app_min_width`.
 */
function sidebar(state: PlayerState, take: (index: number) => void): HTMLElement {
  const files = state.list ?? PLAYLIST;
  const side = glass(document.createElement("span"));
  side.classList.add("mp-sidebar");
  const count = text(`${(state.index ?? 0) + 1} of ${state.count ?? files.length}`, "small", "on_surface_variant");
  const head = row("s8", "centre", text("Playlist", "subheading"), count);
  head.classList.add("mp-sidebar-head");
  const made = listWith(
    files.map((file, i) => playlistRow(file, i, state)),
    { current: state.index ?? 0, onTake: take, slides: true },
  );
  const { box } = scroll(made.box, { height: "100%", above: "s12", below: "s16", fade: "s12", footFade: true });
  const room = document.createElement("span");
  room.className = "mp-sidebar-room";
  room.append(box);
  side.append(head, room);
  let shown = state;
  playlistUpdates.set(side, (now) => {
    const was = shown;
    shown = now;
    files.forEach((file, i) => {
      const before = playlistRow(file, i, was);
      const after = playlistRow(file, i, now);
      if (before.icon === after.icon && before.checked === after.checked) return;
      const fresh = made.replaceRow(i, after);
      // The highlight glides to the file now playing.
      if (after.checked && now.index !== was.index) made.rows.put(fresh);
    });
    if (now.index !== was.index) count.textContent = `${(now.index ?? 0) + 1} of ${now.count ?? files.length}`;
  });
  return side;
}

/** The visualiser's bars: how many, and each one's highest reach as a share of its room. */
const BARS = 32;
const reach = (i: number) => 0.35 + 0.65 * Math.abs(Math.sin(i * 1.7) * Math.cos(i * 0.45));

/**
 * DRAFT PART, a song's level strip: bars `size.scrollbar` wide, `space.s4` apart, in the accent
 * glowing as lit tubes do, rising and falling while it plays (each its own reach and its own beat
 * in the wave, `duration.hum`) and settling to their rest, a `size.scrollbar` stub, over `medium`
 * when it pauses. With reduce motion they stand still at their reach. In mpv a visualiser is an
 * audio filter's picture (`showfreqs`), not the script's: see the report.
 */
function bars(): HTMLElement {
  const strip = document.createElement("span");
  strip.className = "mp-bars";
  for (let i = 0; i < BARS; i++) {
    const bar = document.createElement("span");
    bar.className = "mp-bar";
    bar.style.setProperty("--reach", String(reach(i)));
    bar.style.setProperty("--beat", String(i));
    strip.append(bar);
  }
  return strip;
}

/**
 * DRAFT PART, a song: its cover large with depth (raised to step 3, the rim and shadow), never
 * cropped and never past `size.cover`, its shape kept, centred in the room it has; its title in
 * `title` and its artist in `body_large`; the level strip under them. A song with no cover has
 * the music symbol on a tile of the same room.
 */
function song(file: MediaFile): HTMLElement {
  const room = document.createElement("span");
  room.className = "mp-song";
  const cover = document.createElement("span");
  cover.className = "mp-cover";
  if (file.picture) {
    const picture = document.createElement("img");
    picture.src = file.picture;
    picture.alt = "";
    // The cover takes its shape once the picture is in, which moves what is centred round it: it is
    // put back on whole device pixels then, as anything drawn is.
    const shape = () => {
      if (!picture.naturalWidth) return;
      cover.style.setProperty("--ratio", String(picture.naturalWidth / picture.naturalHeight));
      if (room.isConnected) centreOnPixels(room);
    };
    if (picture.complete) shape();
    else picture.addEventListener("load", shape, { once: true });
    cover.append(picture);
  } else {
    cover.dataset.bare = "";
    cover.append(icon("music", "var(--icon-s48)", "var(--color-on_surface_variant)"));
  }
  const words = stack("s4", "centre", declare(clamped(file.title, "title"), { variable: true }), ...(file.by ? [text(file.by, "body_large", "on_surface_variant")] : []));
  words.classList.add("mp-song-words");
  room.append(cover, words, bars());
  return room;
}

/** The play or pause symbol that blooms in the picture's middle as a press starts or stops it. */
function bloom(): HTMLElement {
  const disc = glass(document.createElement("span"));
  disc.classList.add("mp-bloom");
  disc.append(icon("play_arrow", "var(--icon-s48)", "var(--color-on_surface)"), icon("pause", "var(--icon-s48)", "var(--color-on_surface)"));
  return disc;
}

/** What fills the window when it plays no picture: idle, a broken file, waiting for a stream. */
function middle(state: PlayerState): HTMLElement | null {
  const file = state.file;
  if (!file) {
    const room = document.createElement("span");
    room.className = "mp-centred mp-idle";
    const sign = document.createElement("span");
    sign.className = "mp-idle-sign";
    sign.append(icon("movie", "var(--icon-s48)", "var(--color-primary_lit)"));
    const words = stack("s8", "centre", text("Nothing playing", "headline"), wrapped("Drop a video or a song here, or open one from Files.", "body_large", "on_surface_variant"));
    words.classList.add("mp-idle-words");
    room.append(stack("s24", "centre", sign, words));
    return room;
  }
  if (state.error) {
    const room = document.createElement("span");
    room.className = "mp-centred mp-error";
    room.append(errorBlock({ message: `Can't play ${file.title}`, hint: "mpv found no video or sound it can read in it." }));
    return room;
  }
  if (state.buffering !== undefined) {
    const room = document.createElement("span");
    room.className = "mp-centred mp-buffering";
    room.append(stack("s12", "centre", spinner("s48"), text(`Buffering · ${Math.round(state.buffering * 100)}%`, "label_number", "on_surface")));
    return room;
  }
  return null;
}

/** What a player is: its window, how to change it, what it is now, and the bloom a press shows. */
export interface Player {
  readonly element: HTMLElement;
  readonly set: (patch: Patch) => void;
  readonly state: () => PlayerState;
  /** Blooms the symbol of the state it just entered in the picture's middle. */
  readonly bloom: () => void;
  /** The film's video, where it really plays (the live stage); none for any other file or place. */
  readonly film: () => HTMLVideoElement | null;
  /**
   * Moves the time on to `position`, read ahead to `buffered`: the bar, the times and the
   * chapter's name in place, nothing else, so playing can call it every frame.
   */
  readonly advance: (position: number, buffered: number) => void;
}

/** A file's signal: its sound's own analysis once ready (the calm `QUIET` until then), or its synthesised one. */
function signalOf(file: MediaFile): Signal {
  if (file.source) return analysedSong(file.source, file.length).now() ?? QUIET;
  return synthetic(MOODS[file.mood ?? "drive"]);
}

/** The moment into a song a still visualiser is drawn at: the same every time. */
const STILL_AT = 41.3;

/** What makes the window be built again, rather than changed in place. */
const STRUCTURE = new Set<keyof PlayerState>(["file", "error", "buffering", "count", "index", "full", "remaining"]);

/**
 * DRAFT PART, the player's window: the picture with its ambient light, a song, what plays over
 * them, the glass controller, the top corner's playlist button and its sidebar, a menu by its
 * button, a message, the play and pause bloom; borderless, as every KubaOS window is. It
 * fills its room, and finds the ladder's rung its width needs as it is laid out and as it resizes.
 */
export function player(initial: PlayerState, hooks: Hooks = {}): Player {
  let state = initial;
  const root = glassWindow(document.createElement("span"), true);
  root.classList.add("mp-window");
  root.dataset.mode = "dark";
  let rung = 0;
  let built: Built | null = null;
  const pictures = new Map<string, HTMLImageElement>();
  let screen = document.createElement("span");
  let tip: HTMLElement | null = null;
  // The visualiser's room, what it shows, and its live run.
  let visualRoom = document.createElement("span");
  let shownChoice: VizChoice = "off";
  let shownFile: MediaFile | undefined;
  let refit = 0;
  let running: Live | null = null;
  let pendingStill = false;
  // The film's own picture, where it really plays: the playing video, and a second one, muted and
  // never playing, seeked to the time the pointer is at over the bar for its frame preview, so the
  // preview never stalls the film. Each is made once, for the first film shown.
  const moves = hooks.live === true && CHROME && !ALL;
  let film: HTMLVideoElement | null = null;
  let filmSource = "";
  let preview: HTMLVideoElement | null = null;
  /** The time the preview is to show next, s, while it is still seeking to the last; or none. */
  let previewWanted: number | null = null;
  const take = (index: number) => hooks.take?.(index);
  // The volume's own slider moving: the state follows it, and nothing sets the slider back.
  const inner: Hooks = {
    volume: (value) => {
      state = { ...state, volume: value };
      hooks.volume?.(value);
    },
    muted: (on) => {
      state = { ...state, muted: on };
      hooks.muted?.(on);
    },
  };

  /** Whether the controller shows: the pointer moved, or paused, or a song, or a menu or the sidebar open. */
  // A song with its visualiser on is watched as a film is: the controller leaves it alone.
  const shows = () => (state.shown ?? false) || !state.playing || ((state.file?.audio ?? false) && choiceOf(state) === "off") || state.open !== undefined || (state.sidebar ?? false);

  /** Builds the window for what it plays and how it is laid out. */
  const build = () => {
    const file = state.file;
    // A song after a song keeps its visualiser running, its canvas moved into the new window, so
    // the flight goes on into the next song without a gap.
    const carried = running && file?.audio && choiceOf(state) !== "off" ? visualRoom.querySelector("canvas") : null;
    if (!carried) {
      running?.stop();
      running = null;
    }
    shownChoice = "off";
    root.toggleAttribute("data-full", state.full ?? false);
    root.toggleAttribute("data-song", file?.audio ?? false);
    root.toggleAttribute("data-idle", !file);
    root.toggleAttribute("data-broken", state.error ?? false);
    root.toggleAttribute("data-many", (state.count ?? 1) > 1);
    for (const name of ["--mp-left", "--mp-right", "--mp-top", "--mp-bottom", "--mp-middle", "--mp-corner-0", "--mp-corner-1", "--mp-corner-2", "--mp-corner-3"]) root.style.removeProperty(name);
    delete root.dataset.lit;
    screen = document.createElement("span");
    screen.className = "mp-screen";
    const ambient = document.createElement("span");
    ambient.className = "mp-ambient";
    screen.append(ambient);
    visualRoom = document.createElement("span");
    visualRoom.className = "mp-visual";
    if (carried) visualRoom.append(carried);
    screen.append(visualRoom);
    const playing = moves && file?.video && !state.error ? filmFor(file) : null;
    // A film left for another file stops: out of the page, it would play on, and be heard.
    if (film && !playing) film.pause();
    if (playing && file?.picture) {
      screen.append(playing);
      // Lit from the poster until the film's first frame is drawn, then from each frame it shows.
      const poster = document.createElement("img");
      poster.src = file.picture;
      light(root, poster);
    } else if (file && !file.audio && file.picture && !state.error) {
      // One picture a source, kept across builds, so a build never decodes it again.
      let frame = pictures.get(file.picture);
      if (!frame) {
        frame = document.createElement("img");
        frame.className = "mp-frame";
        frame.src = file.picture;
        frame.alt = "";
        pictures.set(file.picture, frame);
      }
      screen.append(frame);
      light(root, frame);
    }
    if (file?.audio && file.picture) {
      const source = document.createElement("img");
      source.src = file.picture;
      light(root, source);
    }
    if (file?.audio) screen.append(song(file));
    const centre = middle(state);
    if (centre) screen.append(centre);
    const line = text("", "heading", "on_surface");
    line.classList.add("mp-subtitle");
    const lineRoom = document.createElement("span");
    lineRoom.className = "mp-subtitle-room";
    lineRoom.append(line);
    screen.append(lineRoom);
    screen.append(bloom());
    built = null;
    if (file && !state.error) {
      built = controller(state, inner);
      screen.append(built.box);
      if ((state.count ?? 1) > 1) {
        const corner = document.createElement("span");
        corner.className = "mp-corner";
        const pill = glass(document.createElement("span"));
        pill.classList.add("mp-corner-glass");
        corner.append(pill, playlistButton());
        screen.append(corner);
      }
      if ((state.count ?? 1) > 1) screen.append(sidebar(state, (index) => take(index)));
    }
    const said = document.createElement("span");
    said.className = "mp-message-room";
    screen.append(said);
    root.replaceChildren(screen);
    root.style.setProperty("--mp-pad", `var(--space-${containerPadding(widthOf())})`);
    takeRung(rung);
    // What a moment changes, drawn for the state as it is.
    tip = null;
    momentary(state, {});
  };

  /**
   * The film's video for `file`, at the state's position, made once and kept across builds, with
   * a text track for each of its subtitles. It starts muted, and plays and sounds only when its
   * owner says. Each frame it shows lights the window: read at the video's `timeupdate`, four
   * times a second, from a few cells.
   */
  function filmFor(file: MediaFile): HTMLVideoElement {
    if (!film) {
      film = document.createElement("video");
      film.className = "mp-frame";
      film.muted = true;
      film.defaultMuted = true;
      film.playsInline = true;
      film.preload = "auto";
      film.disablePictureInPicture = true;
      const shown = film;
      const lit = () => {
        if (shown.isConnected && shown.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) paint(root, sample(shown));
      };
      shown.addEventListener("timeupdate", lit);
      shown.addEventListener("seeked", lit);
      shown.addEventListener("loadeddata", lit);
      shown.addEventListener("ended", () => hooks.ended?.());
    }
    if (filmSource !== file.video) {
      filmSource = file.video!;
      film.poster = file.picture ?? "";
      film.replaceChildren(...subtitleTracks(file));
      film.src = filmSource;
    }
    const at = (state.position ?? 0) * file.length;
    if (Math.abs(film.currentTime - at) > 0.25) film.currentTime = at;
    return film;
  }

  /**
   * A text track for each of `file`'s subtitles with a file, off until chosen: the chosen one's
   * line at the time is the line on the picture, its cue's lines joined into the player's one.
   */
  function subtitleTracks(file: MediaFile): HTMLTrackElement[] {
    return tracksOf(file).subtitles.flatMap((subtitle, i) => {
      if (!subtitle.src) return [];
      const track = document.createElement("track");
      track.kind = "subtitles";
      track.label = subtitle.name;
      track.src = subtitle.src;
      track.dataset.index = String(i);
      track.track.mode = "disabled";
      track.track.addEventListener("cuechange", () => {
        if (state.subtitle !== i || state.file?.video !== filmSource) return;
        const cues = [...(track.track.activeCues ?? [])].filter((cue): cue is VTTCue => cue instanceof VTTCue);
        const words = cues.map((cue) => cue.text.replace(/\s*\n\s*/g, " ")).join(" ");
        if ((state.line ?? "") !== words) update({ line: words || undefined });
      });
      return [track];
    });
  }

  /** Shows the chosen subtitle's track, where the film really plays, and hides the others. */
  function followSubtitles(): void {
    if (!film || state.file?.video !== filmSource) return;
    for (const each of film.querySelectorAll("track")) {
      const mode = Number(each.dataset.index) === state.subtitle ? "hidden" : "disabled";
      if (each.track.mode !== mode) each.track.mode = mode;
    }
  }

  /** The preview's video, showing `file`'s frame at `seconds`: one seek at a time, the latest wanted next. */
  function previewAt(file: MediaFile, seconds: number): HTMLVideoElement {
    if (!preview) {
      preview = document.createElement("video");
      preview.muted = true;
      preview.defaultMuted = true;
      preview.playsInline = true;
      preview.preload = "auto";
      preview.disablePictureInPicture = true;
      preview.poster = file.picture ?? "";
      preview.src = file.video!;
      const next = () => {
        const shown = preview!;
        if (previewWanted === null || shown.readyState < HTMLMediaElement.HAVE_METADATA || shown.seeking) return;
        const wanted = previewWanted;
        previewWanted = null;
        if (Math.abs(shown.currentTime - wanted) > 0.01) shown.currentTime = wanted;
      };
      preview.addEventListener("loadedmetadata", next);
      preview.addEventListener("seeked", next);
      previewWanted = seconds;
      return preview;
    }
    previewWanted = seconds;
    if (preview.readyState >= HTMLMediaElement.HAVE_METADATA && !preview.seeking) {
      previewWanted = null;
      if (Math.abs(preview.currentTime - seconds) > 0.01) preview.currentTime = seconds;
    }
    return preview;
  }

  /** Marks the rungs up to `at` as taken: the CSS hides and shows the parts they move. */
  function takeRung(at: number): void {
    for (let i = 1; i <= RUNGS.length; i++) root.toggleAttribute(`data-r${i}`, i <= at);
    root.dataset.rung = String(at);
  }

  /** Changes in place what a moment changes, from `was` to the state now. */
  function momentary(now: PlayerState, was: Partial<PlayerState>): void {
    const file = now.file;
    // The song's place follows the visualiser: measured before it moves, so it can glide there.
    if (choiceOf(now) !== shownChoice || now.colours !== was.colours) glide(() => visualise());
    else {
      showVisual(choiceOf(now));
      // Another song under the same visualiser: its flight goes on into the new song's sections.
      if (file !== shownFile && running) visualise();
    }
    root.toggleAttribute("data-shown", shows());
    root.toggleAttribute("data-playing", now.playing ?? false);
    root.toggleAttribute("data-sidebar", now.sidebar ?? false);
    // The controller's end draws in beside the panel: its parts fit to the width it is going to,
    // at once as the panel opens, once it has gone as it closes, so nothing ever overflows.
    if ((now.sidebar ?? false) !== (was.sidebar ?? false)) {
      window.clearTimeout(refit);
      if (now.sidebar) fit(true);
      else refit = window.setTimeout(() => fit(true), durationMs("medium") / (settings().speed || 1));
    }
    // The subtitle line: the film's own track's where it really plays, else the state's.
    followSubtitles();
    const line = screen.querySelector<HTMLElement>(".mp-subtitle");
    if (line) {
      const words = now.line && file && !file.audio && !now.error ? now.line : "";
      if (line.textContent !== words) {
        const fresh = text(words, "heading", "on_surface");
        fresh.classList.add("mp-subtitle");
        line.replaceWith(fresh);
      }
    }
    if (built && file) {
      showTime(now);
      // The pointer over the bar, and the bar held.
      built.bar.toggleAttribute("data-held", now.dragging ?? false);
      built.bar.toggleAttribute("data-lit", now.lit === "bar" || now.hover !== undefined);
      if (now.hover === undefined) {
        tip?.remove();
        tip = null;
      } else if (!tip || tip.hasAttribute("data-preview") !== previews(file, now.preview ?? false)) {
        tip?.remove();
        const moving = moves && file.video && previews(file, now.preview ?? false) ? previewAt(file, now.hover * file.length) : null;
        tip = seekTip(file, now.hover, now.preview ?? false, moving);
        built.bar.append(tip);
      } else {
        if (moves && file.video && tip.hasAttribute("data-preview")) previewAt(file, now.hover * file.length);
        tip.style.setProperty("--at", String(now.hover));
        const words = tip.querySelector<HTMLElement>(".mp-tip-words, .k-hint > .k-text");
        const said = tipWords(file, now.hover);
        if (words && words.textContent !== said) words.textContent = said;
      }
      if (tip) tip.style.setProperty("--tip-half", `${tip.offsetWidth / 2}px`);
      // Play and pause.
      setMorph(built.play, now.playing ?? false);
      rename(built.play, now.playing ? "Pause" : "Play");
      // The volume, set from outside only where it differs, so the slider's own moves never echo.
      if (now.volume !== undefined && now.volume !== was.volume) setSlider(built.volume, now.volume);
      if (now.muted !== was.muted) setSliderToggled(built.volume, now.muted ?? false);
      root.querySelector(".mp-volume")?.toggleAttribute("data-open", now.lit === "volume");
    }
    // A menu by its button.
    const openers = root.querySelectorAll<HTMLElement>("[data-opens]");
    for (const opener of openers) {
      const opens = opener.dataset.opens;
      const on = opens === "sidebar" ? (now.sidebar ?? false) : now.open === opens || (opens === "more" && now.open === "speed");
      if (on) opener.dataset.pinned = "lit";
      else if (opener.dataset.pinned === "lit") delete opener.dataset.pinned;
    }
    if (now.open !== was.open || now.speed !== was.speed || now.audioTrack !== was.audioTrack || now.subtitle !== was.subtitle || now.muted !== was.muted) {
      screen.querySelector(".mp-popover")?.remove();
      if (now.open && built) {
        const pop = opened(now, rung);
        screen.append(pop);
        place();
      }
    }
    // The sidebar's rows follow the file playing.
    if (now.index !== was.index || now.playing !== was.playing) {
      const side = screen.querySelector(".mp-sidebar");
      if (side) playlistUpdates.get(side)?.(now);
    }
    // A message from a key.
    const room = screen.querySelector<HTMLElement>(".mp-message-room");
    if (room && now.message !== was.message) {
      root.toggleAttribute("data-said", now.message !== undefined);
      const strip = room.querySelector<HTMLElement>(".mp-message");
      const fresh = now.message ? message(now.message) : null;
      // A message over one still showing changes its words in place: the strip stays down.
      if (strip && fresh) {
        strip.replaceChildren(...fresh.childNodes);
        strip.toggleAttribute("data-words", fresh.hasAttribute("data-words"));
      } else room.replaceChildren(...(fresh ? [fresh] : []));
    }
  }

  /**
   * Shows the state's time: the bar, the times and the chapter's name. The times roll only as
   * their whole second changes, so a frame that moves the bar alone leaves them be.
   */
  function showTime(now: PlayerState): void {
    const file = now.file;
    if (!built || !file) return;
    const position = now.position ?? 0;
    moveBar(built.bar, file, position, now.buffered ?? position);
    built.elapsed?.set(Math.round(position * file.length));
    const left = built.length?.querySelector<HTMLElement>(".k-rolling");
    if (left && now.remaining) rollers.get(left)?.set(Math.round(file.length * (1 - position)));
    // The chapter's name, fading in as the time crosses into a new one.
    const chapter = screen.querySelector<HTMLElement>(".mp-chapter");
    const name = chapterAt(file, position)?.name ?? "";
    if (chapter && chapter.textContent !== name) {
      const fresh = text(name, "small", "on_surface_variant");
      fresh.classList.add("mp-chapter");
      if (chapter.textContent !== "") fresh.dataset.fresh = "";
      chapter.replaceWith(fresh);
    }
  }

  /** Stands the open menu by its button: its own, or ⋯ where the button has joined it. */
  function place(): void {
    const room = screen.querySelector<HTMLElement>(".mp-popover");
    if (!room || !state.open) return;
    const wanted = state.open === "speed" ? "more" : state.open;
    const shown = (button: HTMLElement | null) => (button && button.offsetParent !== null ? button : null);
    const button = shown(root.querySelector<HTMLElement>(`[data-opens="${wanted}"]`)) ?? shown(root.querySelector<HTMLElement>('[data-opens="more"]'));
    if (!button) return;
    anchorPopover(room, button, screen, { side: "above", inset: length(root.hasAttribute("data-narrow") ? "space-s8" : "space-s16") * (settings().scale || 1) });
  }

  /** The window's width in logical pixels. */
  function widthOf(): number {
    return root.offsetWidth / (settings().scale || 1);
  }

  /** Takes rungs until the buttons fit their row, as the window's width now has it. */
  // The width the rung was last found for: a resize to the same width finds nothing new.
  let fittedWidth = -1;
  const fit = (force = false) => {
    if (root.offsetWidth === 0) return;
    if (!force && root.offsetWidth === fittedWidth) return measure();
    fittedWidth = root.offsetWidth;
    root.style.setProperty("--mp-pad", `var(--space-${containerPadding(widthOf())})`);
    root.toggleAttribute("data-narrow", widthOf() < length(SIDEBAR_COVERS));
    if (state.file && !state.error) {
      const was = rung;
      for (let at = 0; at <= RUNGS.length; at++) {
        rung = at;
        takeRung(at);
        if (fits()) break;
      }
      // A menu open lists what has joined ⋯: drawn again for the rung found.
      if (state.open && rung !== was) {
        screen.querySelector(".mp-popover")?.remove();
        screen.append(opened(state, rung));
      }
    }
    measure();
  };

  /**
   * Draws the song's visualiser in its room: live on the stage, a still anywhere else (a sample,
   * `?all`, a shot held still), the cover alone with no WebGL. Off leaves the room empty.
   */
  function visualise(): void {
    const choice = choiceOf(state);
    shownChoice = choice;
    const file = state.file;
    const another = file !== shownFile;
    shownFile = file;
    if (choice === "off" || !file?.audio) {
      running?.stop();
      running = null;
      visualRoom.replaceChildren();
      pendingStill = false;
      return;
    }
    const visual = VISUALS[choice];
    const signal = signalOf(file);
    const palette = () => (state.colours === "cover" ? coverPalette(root) : wallpaperPalette(root));
    if (hooks.live && CHROME && !ALL) {
      if (running && visualRoom.firstElementChild) running.set(visual, signal, another ? { start: (state.position ?? 0) * file.length } : undefined);
      else {
        const canvas = document.createElement("canvas");
        canvas.className = "mp-visual-canvas";
        visualRoom.replaceChildren(canvas);
        try {
          running = new Live(canvas, visual, signal, palette, () => state.playing ?? false, (state.position ?? 0) * file.length, hooks.time);
        } catch {
          visualRoom.replaceChildren();
        }
      }
      // A real song's analysis, once done, takes over from the calm look it showed meanwhile.
      if (signal === QUIET && file.source) {
        void analysedSong(file.source, file.length).ready.then((ready) => {
          if (state.file === file && running && choiceOf(state) !== "off") running.set(VISUALS[choiceOf(state) as Exclude<VizChoice, "off">], ready);
        });
      }
      return;
    }
    // A still waits until the window is laid out in the page, where its colours and size are known.
    pendingStill = true;
    drawStill();
  }

  function drawStill(): void {
    if (!pendingStill || !root.isConnected || visualRoom.offsetWidth === 0) return;
    pendingStill = false;
    const choice = choiceOf(state);
    const file = state.file;
    if (choice === "off" || !file) return;
    // A real song's still waits for its analysis: the same picture every time, never the calm stand-in.
    if (file.source && !analysedSong(file.source, file.length).now()) {
      void analysedSong(file.source, file.length).ready.then(() => {
        pendingStill = true;
        drawStill();
      });
      return;
    }
    const palette = state.colours === "cover" ? coverPalette(root) : wallpaperPalette(root);
    const picture = still(VISUALS[choice], signalOf(file), palette, STILL_AT, visualRoom.offsetWidth, visualRoom.offsetHeight, state.playing ?? true);
    if (picture) {
      picture.className = "mp-visual-canvas";
      visualRoom.replaceChildren(picture);
    } else showVisual("off");
  }

  /**
   * Says which visualiser the window shows. The song's words line up with it: centred under the
   * cover with none, from their start beside it with one.
   */
  function showVisual(choice: VizChoice): void {
    root.dataset.visual = choice;
    const words = root.querySelector<HTMLElement>(".mp-song-words");
    if (words) realign(words, choice === "off" ? "centre" : "start");
  }

  /**
   * The song moving between its two places as its visualiser turns on or off: in the middle, large,
   * with none; in the bottom start corner, small, with one. `change` makes the change; the song's
   * block then glides from where it stood, over `long` on `emphasized_decel`, its size with it.
   */
  function glide(change: () => void): void {
    const block = screen.querySelector<HTMLElement>(".mp-song");
    const before = block && block.offsetParent !== null ? block.getBoundingClientRect() : null;
    const was = root.dataset.visual;
    change();
    showVisual(choiceOf(state));
    if (!block || !before || (was === "off") === (root.dataset.visual === "off") || settings().reduced) return;
    const after = block.getBoundingClientRect();
    if (after.width === 0) return;
    const scale = before.width / after.width;
    block.animate(
      [
        { transformOrigin: "0 0", transform: `translate(${before.left - after.left}px, ${before.top - after.top}px) scale(${scale})` },
        { transformOrigin: "0 0", transform: "none" },
      ],
      { duration: ms("long"), easing: raw("easing-emphasized_decel") },
    );
  }

  /**
   * Hands the CSS what it can't know: the controller's height, and how far the bar's top is below
   * the controller's, which a frame preview stands clear of; and stands an open menu by its button.
   */
  function measure(): void {
    drawStill();
    root.style.setProperty("--mp-controller", built ? `${built.box.offsetHeight}px` : "0px");
    if (built) {
      // In the window's own pixels: its box is scaled by the page's zoom like everything in it.
      const box = built.box.getBoundingClientRect();
      const ratio = built.box.offsetWidth / Math.max(1, box.width);
      built.bar.style.setProperty("--mp-bar-top", `${(built.bar.getBoundingClientRect().top - box.top) * ratio}px`);
    }
    if (built && state.file) moveBar(built.bar, state.file, state.position ?? 0, state.buffered ?? 0);
    if (tip) tip.style.setProperty("--tip-half", `${tip.offsetWidth / 2}px`);
    place();
  }

  function fits(): boolean {
    const buttons = root.querySelector<HTMLElement>(".mp-buttons");
    const barRow = root.querySelector<HTMLElement>(".mp-bar-row");
    if (!buttons || !barRow) return true;
    // The controller as wide as it is going to be: drawn in beside the playlist's panel once that
    // has slid in, whole once it has slid out.
    const panel = root.hasAttribute("data-narrow") ? 0 : Math.min(length("size-panel") * (settings().scale || 1), root.offsetWidth);
    const sliding = Number.parseFloat(getComputedStyle(root).getPropertyValue("--mp-side")) || 0;
    const room = buttons.clientWidth + (sliding - (state.sidebar ? 1 : 0)) * panel;
    const width = (selector: string) => buttons.querySelector<HTMLElement>(selector)?.scrollWidth ?? 0;
    const gap = Number.parseFloat(getComputedStyle(buttons).columnGap) || 0;
    // The transport centred between two sides as wide as the wider. The end's overhang is left
    // out: it is room the row has.
    const needed = 2 * Math.max(width(".mp-left"), width(".mp-right")) + width(".mp-transport") + 2 * gap;
    // The bar keeps at least a menu's width beside its times.
    const seek = barRow.querySelector<HTMLElement>(".mp-seek");
    const least = length("size-menu") * (settings().scale || 1);
    const barFits = !seek || seek.offsetWidth >= least || !barRow.querySelector(".mp-time");
    return needed <= room + 0.5 && barFits;
  }

  /** Blooms the symbol of the state just entered: play as it starts, pause as it stops. */
  function blooming(): void {
    const disc = screen.querySelector<HTMLElement>(".mp-bloom");
    if (!disc) return;
    disc.toggleAttribute("data-pause", !(state.playing ?? false));
    for (const running of disc.getAnimations()) running.cancel();
    const bloomed = `${raw("scale-bloom")}`;
    if (settings().reduced) {
      disc.animate([{ opacity: 0 }, { opacity: 1, offset: 0.3 }, { opacity: 0 }], { duration: ms("short") || durationMs("short"), easing: raw("easing-standard") });
      return;
    }
    disc.animate(
      [
        { opacity: 0, scale: 1 },
        { opacity: 1, scale: 1, offset: 0.2 },
        { opacity: 0, scale: bloomed },
      ],
      { duration: ms("long"), easing: raw("easing-standard_decel") },
    );
  }

  /** Changes the state by `patch`: the window built again for a change of what it is, else in place. */
  function update(patch: Patch): void {
    const was = state;
    state = { ...state, ...patch } as PlayerState;
    const keys = Object.keys(patch) as (keyof PlayerState)[];
    if (keys.some((key) => STRUCTURE.has(key) && was[key] !== state[key])) {
      build();
      fit(true);
      return;
    }
    momentary(state, was);
    measure();
  }

  new ResizeObserver(() => fit()).observe(root);
  build();
  measure();
  return {
    element: root,
    set: update,
    state: () => state,
    bloom: blooming,
    film: () => (film && film.isConnected ? film : null),
    advance: (position, buffered) => {
      state = { ...state, position, buffered };
      showTime(state);
    },
  };
}

/**
 * DRAFT PART, the control center's media widget, sharing the controller's layout: the same
 * title row, the same bar with its times and the same three buttons in the same order, in a card
 * of the panel's width. The control center has no media widget yet; this is the proposal.
 */
export function widget(state: PlayerState): HTMLElement {
  const file = state.file!;
  const position = state.position ?? 0;
  const art = document.createElement("span");
  art.className = "mp-widget-art";
  if (file.picture) {
    const picture = document.createElement("img");
    picture.src = file.picture;
    picture.alt = "";
    art.append(picture);
  } else art.append(icon(file.audio ? "music" : "movie", "var(--icon-s24)", "var(--color-on_surface_variant)"));
  const words = stack("s4", "start", declare(clamped(file.title, "body_strong"), { variable: true }), text(file.by ?? "mpv", "small", "on_surface_variant"));
  const head = row("s12", "centre", art, filling(words));
  const bar = filling(seekBar(file));
  moveBar(bar, file, position, state.buffered ?? position);
  const elapsed = row(0, "centre", text(clock(position * file.length), "label_number"), fill());
  elapsed.classList.add("mp-time");
  elapsed.style.width = roomFor("label_number", widestClock(file.length));
  const barRow = row("s12", "centre", elapsed, bar, text(clock(file.length), "label_number", "on_surface_variant"));
  const many = (state.count ?? 1) > 1;
  const previous = tool("skip_previous", "Previous");
  previous.disabled = !many;
  const next = tool("skip_next", "Next");
  next.disabled = !many;
  const transport = row("s8", "centre", previous, playButton(state.playing ?? false), next);
  const card = stack("s12", "centre", head, barRow, transport);
  card.classList.add("mp-widget");
  return card;
}
