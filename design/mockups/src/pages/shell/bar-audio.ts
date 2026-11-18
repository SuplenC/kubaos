// The Bar's audio page: the mute pill and now playing, as `config/quickshell/modules/bar/` is to
// draw them, with the players Menu that now playing's icon opens. Now playing ends the bar's left,
// beside the clock: what you're doing. The mute pill stays on the right, before the bell and status,
// and status has lost its volume symbol to it.

import { hint } from "../../lib/hint.ts";
import { row, stack } from "../../lib/layout.ts";
import { pillSwap, powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { marqueeChanging } from "../../lib/marquee.ts";
import { HEADSET, shot as panelShot } from "./audio-parts.ts";
import {
  type CirclePinned,
  LONG_PLAYER,
  LONG_SONG,
  HINTS,
  makeNowPlaying,
  mutePill,
  type MuteOptions,
  NEXT_SONG,
  nowPlaying,
  nowPlayingOpen,
  PLAYERS,
  type Player,
  playersMenu,
  SONG,
  type Song,
  songParts,
} from "./bar-audio-parts.ts";
import { bellPill, clockPill, statusPill } from "./bar-parts.ts";

type State = "rest" | "lit" | "pressed" | "open";

/** The bar's right end: the mute pill, the bell, status. */
function end(mute: MuteOptions = {}): HTMLElement {
  return row("s8", "centre", mutePill(mute), bellPill(), statusPill());
}

/** Now playing at the left's end, `space.s40` clear of the clock, as the bar lays them. */
function besideClock(playing: Parameters<typeof nowPlaying>[0] = {}): HTMLElement {
  return row("s40", "centre", nowPlaying(playing), clockPill());
}

/** Opens the Audio panel's page, as a press on a side with no device opens the panel. */
const toPanel = () => {
  location.hash = "#/shell/audio-panel";
};

/**
 * The bar's end with a mute circle pinned lit, over a room as tall as its Hint and the gap to it, so
 * the bubble floating under the circle stays inside the sample. The room is a copy of the bubble, unseen.
 */
function hinted(mute: MuteOptions, words: string): HTMLElement {
  const room = hint(words);
  room.style.visibility = "hidden";
  room.setAttribute("aria-hidden", "true");
  for (const each of [room, ...room.querySelectorAll<HTMLElement | SVGElement>("[data-probe]")]) delete each.dataset.probe;
  return stack("s8", "start", end(mute), room);
}

/** The songs ⏮ and ⏭ step through: a short pair, one that scrolls, another short one. */
const SONGS: readonly Song[] = [SONG, LONG_SONG, NEXT_SONG];

/**
 * The stage: each circle does what it will. Now playing beside the clock: its icon opens the players
 * Menu, whose pick makes that player active; play and pause turn, the words holding while paused;
 * ⏮ and ⏭ change the song in its fixed room. On the right, the output's and the mic's mute; the
 * wheel over one changes its level, its symbol following; a right-click opens the Audio panel's page.
 */
function live(): HTMLElement {
  const room = document.createElement("span");
  room.className = "shell-tray-room";
  const line = row("s8", "centre");
  room.append(line);
  const players: readonly Player[] = PLAYERS;
  let active = 0;
  let playing = true;
  let song = 0;
  let level = 0.62;
  let mute = { output: false, input: false };
  let card: HTMLElement | null = null;

  const step = (by: number) => {
    song = (song + by + SONGS.length) % SONGS.length;
    np.change(SONGS[song]!);
  };

  let np = playingPill();

  function playingPill() {
    return makeNowPlaying({
      players,
      active,
      playing,
      song: SONGS[song]!,
      ...(card ? { pinned: { part: "icon" as const, state: "lit" as CirclePinned } } : {}),
      onIcon: () => (card ? closeMenu() : openMenu()),
      onPlay: () => {
        playing = !playing;
        np.play(playing);
      },
      onPrevious: () => step(-1),
      onNext: () => step(1),
    });
  }

  const closeMenu = () => {
    card?.remove();
    card = null;
    np = playingPill();
    draw();
  };

  function openMenu(): void {
    const { card: made, list } = playersMenu({
      players,
      active,
      current: active,
      onTake: (index) => {
        if (index < players.length) active = index;
        closeMenu();
      },
      onLeave: closeMenu,
    });
    card = made;
    np = playingPill();
    draw();
    room.append(made);
    const circle = line.querySelector<HTMLElement>(".shell-playing-pill .shell-pill-item");
    if (circle) {
      const ratio = room.offsetWidth / Math.max(1, room.getBoundingClientRect().width);
      made.style.setProperty("--menu-x", `${(circle.getBoundingClientRect().left - room.getBoundingClientRect().left) * ratio}px`);
    }
    list.focus({ preventScroll: true });
  }

  function draw(): void {
    const mp = mutePill({
      level,
      output: { muted: mute.output },
      input: { muted: mute.input, recording: true },
      onOutput: () => {
        mute = { ...mute, output: !mute.output };
        draw();
      },
      onInput: () => {
        mute = { ...mute, input: !mute.input };
        draw();
      },
    });
    mp.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      toPanel();
    });
    const circles = mp.querySelectorAll<HTMLElement>(".shell-pill-item");
    circles[0]?.addEventListener(
      "wheel",
      (event) => {
        event.preventDefault();
        level = Math.max(0, Math.min(1, level - Math.sign(event.deltaY) * 0.05));
        mute = { ...mute, output: false };
        draw();
      },
      { passive: false },
    );
    line.replaceChildren(row("s40", "centre", np.element, clockPill()), mp, bellPill(), statusPill());
  }
  draw();
  return room;
}

/** Now playing caught as it folds or opens, the song's room at the start of closing or opening. */
function folding(fold: boolean): HTMLElement {
  const made = makeNowPlaying({ folded: !fold });
  made.folding(fold);
  return made.element;
}

/** Now playing as a new song takes its room: the old words and the new both in it. */
function changing(): HTMLElement {
  const made = nowPlaying();
  made.querySelector(".k-marquee")?.replaceWith(marqueeChanging(songParts(SONG), songParts(NEXT_SONG), "now_playing"));
  return made;
}

const PLAYING_MOVES: readonly Move[] = [
  {
    name: "A new song",
    what: "The old words fade over `micro`; the new fade in over `short` and rise `space.s8` over `medium`.",
    tracks: [
      { part: "the old words", on: "the song's room", target: ".k-marquee-track[data-out]", property: "opacity", from: 1, to: 0, duration: "micro", curve: "standard_accel" },
      { part: "the new words' fade", on: "the song's room", target: ".k-marquee-track[data-in]", property: "--marquee-fade", from: 0, to: 1, duration: "short", cue: "micro", curve: "standard" },
      { part: "the new words' rise", on: "the song's room", target: ".k-marquee-track[data-in]", property: "--marquee-rise", from: 0, to: 1, duration: "medium", cue: "micro", curve: "emphasized_decel" },
    ],
    reduced: [
      { part: "the old words", on: "the song's room", target: ".k-marquee-track[data-out]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" },
      { part: "the new words' rise", on: "the song's room", target: ".k-marquee-track[data-in]", property: "--marquee-rise", from: 0, to: 1, duration: "instant", curve: "linear" },
    ],
    back: "cuts",
    sample: changing,
  },
  {
    name: "Scrolling",
    what: "A long song holds `marquee_hold`, then travels `space.s32` per `marquee_pace`, on `linear`.",
    tracks: [{ part: "the words", on: "a song past its room", target: ".k-marquee", property: "--marquee-x", from: 0, to: 1, duration: "marquee_pace", cue: "marquee_hold", curve: "linear" }],
    reduced: [{ part: "the words", on: "a song past its room", target: ".k-marquee", property: "--marquee-x", from: 0, to: 0, duration: "instant", curve: "linear" }],
    back: "cuts",
    sample: () => nowPlaying({ song: LONG_SONG, scrolled: 0 }),
  },
  {
    name: "Paused mid-scroll",
    what: "The words glide to the nearer start over `medium` on `standard`, and hold.",
    tracks: [{ part: "the words", on: "a song past its room", target: ".k-marquee", property: "--marquee-x", from: 3, to: 0, duration: "medium", curve: "standard" }],
    reduced: "same",
    back: "cuts",
    sample: () => nowPlaying({ song: LONG_SONG, scrolled: 3, playing: false }),
  },
  {
    name: "Folding",
    what: "The words fade as their room closes over `medium` on `standard_decel`; the pill narrows with it.",
    ...pillSwap("now playing", "out"),
    back: "cuts",
    sample: () => folding(true),
  },
  {
    name: "Opening",
    what: "The words fade in as their room opens over `medium`; the pill widens with it.",
    ...pillSwap("now playing", "in"),
    back: "cuts",
    sample: () => folding(false),
  },
];

const ONE: readonly Player[] = PLAYERS.slice(0, 1);
const FIVE: readonly Player[] = [...PLAYERS, LONG_PLAYER, { app: "mpv" }];

/** A row's cell: the mute pill or now playing, pinned on its first circle, or open. */
function muteCell(state: State): HTMLElement {
  if (state === "open") return panelShot({ device: HEADSET });
  return state === "rest" ? mutePill({ input: { recording: true } }) : mutePill({ input: { recording: true }, pinned: { side: "output", state } });
}

function playingCell(players: readonly Player[], folded = false): (state: State) => HTMLElement {
  return (state) => (state === "open" ? nowPlayingOpen({ players, folded }) : state === "rest" ? nowPlaying({ players, folded }) : nowPlaying({ players, folded, pinned: { part: "icon", state } }));
}

export default page<State>({
  level: "shell",
  slug: "bar-audio",
  title: "Bar audio",
  lede: "The mute pill, the output and the mic a press from muted, and now playing beside the clock with its players Menu.",
  code: { qml: "MutePill, NowPlaying", unbuilt: "QML: not built yet" },
  from: ["pill", "badged-icon", "menu", "popover", "marquee"],
  stage: { live, hint: "click, wheel, right-click; ⏮ ⏭ change the song", daylight: true },
  variants: [
    { name: "Both live", for: "each symbol `on_surface_variant`", sample: () => mutePill() },
    { name: "Output muted", for: "crossed, in `error`", sample: () => mutePill({ output: { muted: true } }) },
    { name: "Mic muted", for: "crossed, in `error`", sample: () => mutePill({ input: { muted: true } }) },
    { name: "Both muted", for: "both crossed", sample: () => mutePill({ output: { muted: true }, input: { muted: true } }) },
    { name: "An app recording", for: "the microphone's dot on the mic", sample: () => mutePill({ input: { recording: true } }) },
    { name: "Recording, mic muted", for: "crossed; no dot, nothing is heard", sample: () => mutePill({ input: { recording: true, muted: true } }) },
    { name: "Output low", for: "its symbol by the level", sample: () => mutePill({ level: 0.2 }) },
    { name: "No output device", for: "crossed, as muted; a press opens the panel", sample: () => hinted({ output: { missing: true }, pinned: { side: "output", state: "lit" }, onPanel: toPanel }, HINTS.noOutput) },
    { name: "No mic", for: "crossed, as muted; a press opens the panel", sample: () => hinted({ input: { missing: true }, pinned: { side: "input", state: "lit" }, onPanel: toPanel }, HINTS.noMic) },
    { name: "Volume's hint", for: "lit: where the output stands", sample: () => hinted({ pinned: { side: "output", state: "lit" } }, HINTS.volume("62%")) },
    { name: "Mic muted, its hint", for: "lit: the mic's side, muted", sample: () => hinted({ input: { muted: true }, pinned: { side: "input", state: "lit" } }, HINTS.micMuted) },
    { name: "Playing", for: "pause on the accent's fill", sample: () => nowPlaying() },
    { name: "Paused", for: "play on the fill; the words hold still", sample: () => nowPlaying({ playing: false, song: LONG_SONG }) },
    { name: "A long song", for: "scrolling in its fixed room", sample: () => nowPlaying({ song: LONG_SONG }) },
    { name: "A long song, mid-scroll", for: "cut at both ends, both faded", sample: () => nowPlaying({ song: LONG_SONG, scrolled: 3 }) },
    { name: "Beside the clock", for: "the left's end, `space.s40` clear", sample: () => besideClock() },
    { name: "The right's end", for: "mute, bell, status", sample: () => end({ input: { recording: true } }) },
  ],
  sizes: [
    { name: "Now playing, wide", for: "icon, the song's room, the controls", sample: () => nowPlaying() },
    { name: "Now playing, folded", for: "a narrow bar: icon and the controls", sample: () => nowPlaying({ folded: true }) },
    { name: "Mute pill", for: "one size: two circles", sample: () => mutePill() },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "open"],
    absent: {
      disabled: "a side with no device: crossed, its press opens the panel",
      selected: "the active player is checked in its Menu",
      loading: "each pill shows once PipeWire or MPRIS answers",
      empty: "nothing playing or paused: now playing is gone",
      error: "no device: a variant, crossed as muted",
    },
    rows: nonEmpty([
      { name: "Mute pill", cell: muteCell },
      { name: "Now playing, one player", cell: playingCell(ONE) },
      { name: "Now playing, three players", cell: playingCell(PLAYERS) },
      { name: "Now playing, folded", cell: playingCell(PLAYERS, true) },
    ]),
    looks: {
      rest: "symbols on the pill, no fill but play's",
      lit: "pointer: the state layer at `opacity.hover`; a mute circle's Hint",
      pressed: "the circle at `opacity.press`",
      open: "right-click: the Audio panel; the icon: its Menu",
    },
  },
  focus: doesntApply("it takes no Tab; its open Menu holds the keyboard, its items lit as a Menu's."),
  content: {
    text: [
      {
        name: "The song and its artist",
        words: { short: "Halo", long: LONG_SONG.title },
        sample: (words: string) => nowPlaying({ song: { title: words, artist: SONG.artist } }),
      },
      {
        name: "A player's app, in the Menu",
        words: { short: "mpv", long: LONG_PLAYER.app },
        sample: (words: string) => nowPlayingOpen({ players: [{ ...PLAYERS[0]!, app: words }, PLAYERS[1]!] }),
      },
    ],
    missingPicture: [{ name: "A player with no icon", for: "its app's initial, here and in the Menu", sample: () => nowPlayingOpen({ players: [{ app: "mpv" }, PLAYERS[0]!] }) }],
    counts: [
      { name: "No player", for: "now playing is gone", sample: () => end() },
      { name: "One player", for: "the Menu opens all the same", sample: () => nowPlayingOpen({ players: ONE }) },
      { name: "Five players", for: "one Menu row each", sample: () => nowPlayingOpen({ players: FIVE }) },
    ],
  },
  motion: {
    arrives: doesntApply("now playing shows at once as a player plays; the title's room eases."),
    leaves: doesntApply("it goes at once as nothing plays or is paused."),
    between: [
      {
        name: "Lit",
        what: "The pointer onto a circle: its state layer fades in over `short` on `standard`.",
        tracks: [{ part: "the state layer", on: "the output", target: ".shell-pill-item:first-child", property: "--hovered", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => mutePill(),
      },
      ...PLAYING_MOVES,
      ...powerMoves(() => nowPlayingOpen()),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "j", "k"], does: "the Menu's items, round" },
      { keys: ["Home", "End"], does: "the first or the last item" },
      { keys: ["Enter"], does: "takes one: a player, or Show" },
      { keys: ["Esc"], does: "closes the Menu" },
    ],
    order: "The bar takes no Tab; the open Menu holds the keyboard until it closes.",
    lands: "Opening, on the active player; closing, back where it was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Mute pill", for: "circles, their gap, the ends", sample: () => mutePill({ input: { recording: true } }) },
      { name: "Now playing, wide", for: "the song's room between the circles", sample: () => nowPlaying() },
      { name: "Now playing, folded", for: "four circles", sample: () => nowPlaying({ folded: true }) },
      { name: "The players Menu", for: "under the icon", sample: () => nowPlayingOpen() },
      { name: "The Menu alone", for: "its rows", sample: () => playersMenu().card },
    ]),
    measures: [
      { part: "Height", token: "size-control_sm" },
      { part: "Ends, between circles", token: "space-s4" },
      { part: "Circle", token: "size-chip" },
      { part: "Song's room", token: "size-now_playing" },
      { part: "Room to its circles", token: "space-s8" },
      { part: "Edge fades", token: "space-s16" },
      { part: "Symbols, pictures", token: "icon-s16" },
      { part: "Recording dot", token: "size-badge_dot" },
      { part: "Menu's width", token: "size-menu" },
      { part: "Icon to the Menu", token: "space-s12" },
    ],
    reflow: doesntApply("two fixed widths; the Bar page's steps say when it folds."),
  },
});
