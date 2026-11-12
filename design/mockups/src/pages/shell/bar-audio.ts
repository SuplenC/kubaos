// The Bar's audio page: the mute pill and now playing, as `config/quickshell/modules/bar/` is to
// draw them, with the players Menu that now playing's icon opens. The status pill beside them has
// lost its volume symbol to the mute pill.

import { row } from "../../lib/layout.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { shot as panelShot } from "./audio-parts.ts";
import {
  type CirclePinned,
  LONG_PLAYER,
  mutePill,
  type MuteOptions,
  nowPlaying,
  nowPlayingOpen,
  PLAYERS,
  type Player,
  playersMenu,
} from "./bar-audio-parts.ts";
import { bellPill, statusPill } from "./bar-parts.ts";

type State = "rest" | "lit" | "pressed" | "open";

/** The right end of the bar from now playing on: now playing, the mute pill, the bell, status. */
function end(mute: MuteOptions = {}, playing?: Parameters<typeof nowPlaying>[0] | null): HTMLElement {
  return row("s8", "centre", ...(playing === null ? [] : [nowPlaying(playing ?? {})]), mutePill(mute), bellPill(), statusPill());
}

/**
 * The stage: each circle does what it will. The output's and the mic's mute; the wheel over one
 * changes its level, its symbol following; a right-click opens the Audio panel's page. Now playing's
 * icon opens the players Menu, whose pick makes that player active; play and pause turn.
 */
function live(): HTMLElement {
  const room = document.createElement("span");
  room.className = "shell-tray-room";
  const line = row("s8", "centre");
  room.append(line);
  const players: readonly Player[] = PLAYERS;
  let active = 0;
  let playing = true;
  let level = 0.62;
  let mute = { output: false, input: false };
  let card: HTMLElement | null = null;

  const closeMenu = () => {
    card?.remove();
    card = null;
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
    const pressedIcon: { part: "icon"; state: CirclePinned } | undefined = card ? { part: "icon", state: "lit" } : undefined;
    const np = nowPlaying({
      players,
      active,
      playing,
      ...(pressedIcon ? { pinned: pressedIcon } : {}),
      onIcon: () => (card ? closeMenu() : openMenu()),
      onPlay: () => {
        playing = !playing;
        draw();
      },
    });
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
      location.hash = "#/shell/audio-panel";
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
    line.replaceChildren(np, mp, bellPill(), statusPill());
  }
  draw();
  return room;
}

const ONE: readonly Player[] = PLAYERS.slice(0, 1);
const FIVE: readonly Player[] = [...PLAYERS, LONG_PLAYER, { app: "mpv", picture: PLAYERS[1]!.picture }];

/** A row's cell: the mute pill or now playing, pinned on its first circle, or open. */
function muteCell(state: State): HTMLElement {
  if (state === "open") return panelShot({ headset: "calls" });
  return state === "rest" ? mutePill({ input: { recording: true } }) : mutePill({ input: { recording: true }, pinned: { side: "output", state } });
}

function playingCell(players: readonly Player[]): (state: State) => HTMLElement {
  return (state) => (state === "open" ? nowPlayingOpen({ players }) : state === "rest" ? nowPlaying({ players }) : nowPlaying({ players, pinned: { part: "icon", state } }));
}

export default page<State>({
  level: "shell",
  slug: "bar-audio",
  title: "Bar audio",
  lede: "The mute pill, the output and the mic a press from muted, and now playing with its players Menu.",
  code: { qml: "MutePill, NowPlaying", unbuilt: "QML: not built yet" },
  from: ["pill", "badged-icon", "menu", "popover"],
  stage: { live, hint: "click, wheel, right-click; the app icon opens", daylight: true },
  variants: [
    { name: "Both live", for: "each symbol `on_surface_variant`", sample: () => mutePill() },
    { name: "Output muted", for: "crossed, in `error`", sample: () => mutePill({ output: { muted: true } }) },
    { name: "Mic muted", for: "crossed, in `error`", sample: () => mutePill({ input: { muted: true } }) },
    { name: "Both muted", for: "both crossed", sample: () => mutePill({ output: { muted: true }, input: { muted: true } }) },
    { name: "An app recording", for: "the microphone's dot on the mic", sample: () => mutePill({ input: { recording: true } }) },
    { name: "Recording, mic muted", for: "crossed; no dot, nothing is heard", sample: () => mutePill({ input: { recording: true, muted: true } }) },
    { name: "Output low", for: "its symbol by the level", sample: () => mutePill({ level: 0.2 }) },
    { name: "Playing", for: "pause on the accent's fill", sample: () => nowPlaying() },
    { name: "Paused", for: "play on the accent's fill", sample: () => nowPlaying({ playing: false }) },
    { name: "Beside status", for: "now playing, mute, bell, status", sample: () => end({ input: { recording: true } }) },
    { name: "No player", for: "now playing is gone", sample: () => end({}, null) },
  ],
  sizes: [
    { name: "Now playing", for: "icon, previous, play or pause, next", sample: () => nowPlaying() },
    { name: "Now playing, given way", for: "a narrow bar: icon and play or pause", sample: () => nowPlaying({ compact: true }) },
    { name: "Mute pill", for: "one size: two circles", sample: () => mutePill() },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "open"],
    absent: {
      disabled: "a side with no device is still a press; see questions",
      selected: "the active player is checked in its Menu",
      loading: "each pill shows once PipeWire or MPRIS answers",
      empty: "no player: now playing is gone",
      error: "no device: its symbol crossed, as muted",
    },
    rows: nonEmpty([
      { name: "Mute pill", cell: muteCell },
      { name: "Now playing, one player", cell: playingCell(ONE) },
      { name: "Now playing, three players", cell: playingCell(PLAYERS) },
    ]),
    looks: {
      rest: "symbols on the pill, no fill but play's",
      lit: "pointer: the circle's state layer at `opacity.hover`",
      pressed: "the circle at `opacity.press`",
      open: "right-click: the Audio panel; the icon: its Menu",
    },
  },
  focus: doesntApply("it takes no Tab; its open Menu holds the keyboard, its items lit as a Menu's."),
  content: {
    text: [
      {
        name: "A player's app, in the Menu",
        words: { short: "mpv", long: LONG_PLAYER.app },
        sample: (words: string) => nowPlayingOpen({ players: [{ ...PLAYERS[0]!, app: words }, PLAYERS[1]!] }),
      },
    ],
    missingPicture: [{ name: "A player with no icon", for: "`music_note` in its place, here and in the Menu", sample: () => nowPlayingOpen({ players: [{ app: "mpv", picture: "" }, PLAYERS[0]!] }) }],
    counts: [
      { name: "No player", for: "now playing is gone", sample: () => end({}, null) },
      { name: "One player", for: "the Menu opens all the same", sample: () => nowPlayingOpen({ players: ONE }) },
      { name: "Five players", for: "one Menu row each", sample: () => nowPlayingOpen({ players: FIVE }) },
    ],
  },
  motion: {
    arrives: doesntApply("now playing shows at once as a player appears; the row eases."),
    leaves: doesntApply("it goes at once as the last player goes."),
    between: [
      {
        name: "Lit",
        what: "The pointer onto a circle: its state layer fades in over `short` on `standard`.",
        tracks: [{ part: "the state layer", on: "the output", target: ".shell-pill-item:first-child", property: "--hovered", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => mutePill(),
      },
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
      { name: "Now playing", for: "four circles", sample: () => nowPlaying() },
      { name: "The players Menu", for: "under the icon", sample: () => nowPlayingOpen() },
      { name: "The Menu alone", for: "its rows", sample: () => playersMenu().card },
    ]),
    measures: [
      { part: "Height", token: "size-control_sm" },
      { part: "Ends, between circles", token: "space-s4" },
      { part: "Circle", token: "size-chip" },
      { part: "Symbols, pictures", token: "icon-s16" },
      { part: "Recording dot", token: "size-badge_dot" },
      { part: "Menu's width", token: "size-menu" },
      { part: "Icon to the Menu", token: "space-s12" },
    ],
    reflow: doesntApply("as wide as its circles; a narrow bar drops ⏮ ⏭, then now playing (the Bar page)."),
  },
});
