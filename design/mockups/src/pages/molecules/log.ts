// The Log page: `docs/design/components/log.md`. The log behind an error or a progress bar:
// streaming `mono` lines in a sunken well that follows its newest line, holds still while you read
// further up, and selects and copies as text.

import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { LEVEL_COLOURS, LEVELS, type Level, log, type LogLine, logRoom } from "../../lib/log.ts";
import { text } from "../../lib/text.ts";
import { CHROME } from "../../site/settings.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { INSTALL_LOG } from "./error-block.ts";

const ROOM = "var(--size-column)";
/** A short log's height, for samples that show a few lines: four of `mono`'s and its padding. */
const SHORT = "calc(4 * var(--type-mono-line) + 2 * var(--space-s12))";

const LINE: Words = {
  short: "==> Partitioning",
  long: "==> Copying the system closure into /mnt/nix/store: 4,812 paths, 6.3 GiB, from the install medium's own store",
};

/** What each level is for. */
const LEVEL_FOR: Record<Level, string> = {
  error: "What failed: the line the error came from.",
  warn: "Something off that didn't stop the work.",
  step: "A step starting: a line opening with `==>`.",
  info: "What the work is doing.",
  debug: "The commands it ran and their detail.",
};

/** An install going on: the lines a stream adds, one after another, round and round. */
const STREAM: readonly LogLine[] = [
  { level: "step", text: "==> Copying the system" },
  { level: "info", text: "Copying /nix/store/0b6kmd…-glibc-2.40 (31 MiB)" },
  { level: "info", text: "Copying /nix/store/7xq1fd…-systemd-257.3 (94 MiB)" },
  { level: "debug", text: "nix copy --to /mnt --no-check-sigs /run/current-system" },
  { level: "info", text: "Copying /nix/store/ahm2l9…-linux-6.14.2 (142 MiB)" },
  { level: "warn", text: "Slow write: 38 MB/s to /dev/nvme0n1p2" },
  { level: "info", text: "Copying /nix/store/c4p9rw…-hyprland-0.48.1 (22 MiB)" },
  { level: "step", text: "==> Installing the boot loader" },
  { level: "info", text: "Writing GRUB to /boot/efi" },
  { level: "debug", text: "grub-install --target=x86_64-efi --efi-directory=/boot/efi" },
];

/** A long log: the stream, a few times over. */
const LONG: readonly LogLine[] = Array.from({ length: 3 }, () => STREAM).flat();

/** A line of `level` from the samples. */
function example(level: Level): LogLine {
  return [...STREAM, ...INSTALL_LOG].find((each) => each.level === level) ?? { level, text: level };
}

function inRoom(element: HTMLElement): HTMLElement {
  return within(ROOM, element);
}

/** How often a live log adds a line: an install's pace. */
const PACE = "duration-long";

/**
 * A log a stream adds lines to, at an install's pace: it follows the newest while it is at its
 * end, and holds still once scrolled up. Its line under it says which. Held still in a shot.
 */
function live(): HTMLElement {
  const shown = log(LONG, { most: 200 });
  const state = text("Following the newest line.", "small", "on_surface_variant");
  let at = 0;
  const pace = Number.parseFloat(getComputedStyle(document.documentElement).getPropertyValue(`--${PACE}`)) || 0;
  if (CHROME && pace > 0) {
    const timer = window.setInterval(() => {
      if (!shown.box.isConnected) {
        window.clearInterval(timer);
        return;
      }
      shown.append(STREAM[at % STREAM.length]!);
      at += 1;
      state.textContent = shown.following() ? "Following the newest line." : "Holding still while you read. End follows again.";
    }, pace);
  }
  return inRoom(stack("s8", "start", shown.box, state));
}

const OPENS: Move = {
  name: "Opening",
  what: "Show log under an error or a progress bar: the room under it opens from nothing to `size.log` over `medium` on `emphasized_decel`, cutting the log as it goes, already at its newest line; what is under it moves down with it. Hide log closes it the same way back. With reduced motion the room is open at once and the log fades in over `short` on `standard`.",
  tracks: [{ part: "its room", on: "a log", target: ".k-log-room", property: "--log-open", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [
    { part: "its room", on: "a log", target: ".k-log-room", property: "--log-open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the log", on: "a log", target: ".k-log", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => inRoom(logRoom(log(INSTALL_LOG).box, false)),
};

const CLOSES: Move = {
  name: "Closing",
  what: "Hide log: the room closes from `size.log` to nothing over `medium`, on `emphasized_decel` played back from open, as Kōbō closes it. With reduced motion it is shut at once.",
  tracks: [{ part: "its room", on: "a log", target: ".k-log-room", property: "--log-open", from: 1, to: 0, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "its room", on: "a log", target: ".k-log-room", property: "--log-open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => inRoom(logRoom(log(INSTALL_LOG).box, true)),
};

const DESCRIBED =
  "A sunken well, the scrolling list's: `surface_container_lowest` at `opacity.log`, the soft hairline inside its edge, `radius.content`, `size.log` (232) tall unless its place gives it a height, its own thumb in its padding while its lines are taller than it. Its lines are `mono`, `space.s16` in at the sides and `space.s12` above and below, newest at the bottom, each in its level's colour: an error `error`, a warning `warning`, a step starting `cold`, the rest `on_surface`, debug `on_surface_variant`.";

export default page<"rest" | "lit" | "empty">({
  level: "molecules",
  slug: "log",
  title: "Log",
  summary: "The log behind an error or a progress bar: streaming lines that follow the newest, hold still while you read, and copy as text.",
  live,
  description: [
    "Rust `error::log(look, lines, height)`, QML `Log` (`lines`). Kōbō's install and failed screens and a setup step's run open it on demand, with a Text button (Show log, `terminal`). Here lines arrive at an install's pace: scroll up and what you read holds still; End, or scrolling back to the bottom, follows again.",
    DESCRIBED,
    "A live log never jumps. At its end it follows the newest line as lines arrive. Scrolled up, what you read stays where it is: new lines add below it, and lines a long log drops from its top give their room back without moving it. End, or scrolling back to the end, follows again. Lines wrap between words, so a long path is read whole, its wrapped lines under it; a log of wide machine output can run each line on one line instead, scrolling across as well.",
    "Its lines are text: a drag selects them, in `primary` at `opacity.selection`, Ctrl+A selects them all and Ctrl+C copies them. To keep it, the screen it sits on saves it (Save log). It is a stop for Tab, lit as a well is, the accent's tube inside its edge; the arrows, Page Up and Page Down, Home and End scroll it.",
  ],
  variants: [
    { name: "Wrapping", for: "Lines broken between words: the default, a long path read whole.", sample: () => inRoom(log(INSTALL_LOG).box) },
    { name: "One line each", for: "Wide machine output, a table's columns: each line on one line, the log scrolling across too.", sample: () => inRoom(log(INSTALL_LOG, { unwrapped: true }).box) },
    { name: "Scrolled up", for: "Read further up: it holds still as lines arrive below.", sample: () => inRoom(log(LONG, { down: 0.4 }).box) },
  ],
  sizes: [
    { name: "size.log", for: "The log opened under an error or a progress bar: Kōbō's, the specimen's.", sample: () => inRoom(log(LONG).box) },
    { name: "Its place's", for: "A log given its room by its place: Kōbō's failed screen gives it what the error leaves.", sample: () => inRoom(log(INSTALL_LOG, { height: SHORT }).box) },
  ],
  states: {
    shown: ["rest", "lit", "empty"],
    absent: {
      pressed: "it takes no press; a drag selects its words.",
      disabled: "a log is shown or not.",
      selected: "its lines aren't choices; selected text is the text's own highlight, shown under Content.",
      loading: "it fills as lines come: a log that is waiting shows the lines it has.",
      error: "error lines are a level, in `error`; the log itself doesn't fail.",
    },
    rows: [
      {
        name: "A log",
        cell: (state) => {
          const shown = log(state === "empty" ? [] : INSTALL_LOG, { height: SHORT });
          if (state === "lit") shown.box.dataset.pinned = "lit";
          return inRoom(shown.box);
        },
      },
    ],
    described: {
      rest: `${DESCRIBED} By day the same roles in daylight colours.`,
      lit: "The keyboard on it: the accent's tube inside its edge, with the glow outside at night, cut only where its place's edge is; by day the line alone.",
      empty: "Opened before anything was said: the well alone, with no lines and no thumb.",
    },
  },
  focus: {
    tubes: [{ name: "The log, with the keyboard on it", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night: the accent's tube inside its edge at `radius.content`, the glow outside it.",
      signsOff: "By day, and at night while the signs are off: the tube as glass, `primary_glass`, no glow.",
    },
    signsOff: () => {
      const shown = log(INSTALL_LOG, { height: SHORT });
      shown.box.dataset.pinned = "signs-off";
      return inRoom(shown.box);
    },
  },
  content: {
    text: [{ name: "A line", words: LINE, sample: (words: string) => inRoom(log([{ level: "step", text: words }, ...INSTALL_LOG.slice(1)], { height: SHORT }).box) }],
    missingPicture: doesntApply("a log is text."),
    counts: [
      { name: "None", for: "Opened before anything was said: the well alone.", sample: () => inRoom(log([], { height: SHORT }).box) },
      { name: "One", for: "A line, at the bottom.", sample: () => inRoom(log(INSTALL_LOG.slice(0, 1), { height: SHORT }).box) },
      { name: "Many", for: "More than its height: it scrolls, its thumb in its padding, at its newest line.", sample: () => inRoom(log(LONG, { height: SHORT }).box) },
      ...LEVELS.map((level) => ({ name: `A ${level} line`, for: `${LEVEL_FOR[level]} In \`${LEVEL_COLOURS[level]}\`.`, sample: () => inRoom(log([example(level)], { height: SHORT }).box) })),
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: powerMoves(() => {
      const shown = log(INSTALL_LOG, { height: SHORT });
      shown.box.dataset.pinned = "lit";
      return inRoom(shown.box);
    }),
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "k", "j"], does: "Scroll a line up or down." },
      { keys: ["Page Up", "Page Down"], does: "Scroll a page, less a line, so the line at the edge stays in sight." },
      { keys: ["Home"], does: "To the first line it keeps." },
      { keys: ["End"], does: "To the newest line, following it again." },
      { keys: ["Ctrl+A"], does: "Select every line." },
      { keys: ["Ctrl+C"], does: "Copy what is selected." },
    ],
    order: "One stop for Tab in reading order: after what opened it (Show log), before the screen's next action.",
    lands: "Opening it leaves the keyboard on Show log; Tab moves onto it.",
  },
  measures: {
    samples: nonEmpty([{ name: "A log", for: "Its padding, its lines and its thumb.", sample: () => inRoom(log(LONG, { height: SHORT }).box) }]),
    measures: [
      { what: "Its height, opened under an error or a progress bar.", token: "size-log" },
      { what: "Its lines in at the sides.", token: "space-s16" },
      { what: "Above the first line and below the last.", token: "space-s12" },
      { what: "A line, `mono`'s.", token: "type-mono-line" },
      { what: "Its thumb.", token: "size-scrollbar" },
      { what: "Its corners.", token: "radius-content" },
    ],
    reflow: {
      described: "It fills its room at every window size: its lines wrap to the width, so a narrower window holds fewer characters a line and more lines; one line each, it scrolls across.",
      sample: () => within("100%", stack("s8", "start", log([{ level: "step", text: shown(LINE) }, ...INSTALL_LOG]).box, text("A log as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});

