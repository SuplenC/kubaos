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
  error: "what failed",
  warn: "off, but it didn't stop the work",
  step: "a step starting, `==>`",
  info: "what the work is doing",
  debug: "the commands it ran",
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
      state.textContent = shown.following() ? "Following the newest line." : "Holding still. End follows again.";
    }, pace);
  }
  return within("100%", stack("s8", "start", shown.box, state));
}

const OPENS: Move = {
  name: "Opening",
  what: "Its room opens to `size.log` over `medium` on `emphasized_decel`, at its newest line; what is under it moves down.",
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
  what: "Its room closes over `medium`, on `emphasized_decel` played back; reduced, shut at once.",
  tracks: [{ part: "its room", on: "a log", target: ".k-log-room", property: "--log-open", from: 1, to: 0, duration: "medium", curve: "emphasized_decel" }],
  reduced: [{ part: "its room", on: "a log", target: ".k-log-room", property: "--log-open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => inRoom(logRoom(log(INSTALL_LOG).box, true)),
};

export default page<"rest" | "lit" | "empty">({
  level: "molecules",
  slug: "log",
  title: "Log",
  lede: "The log behind an error or a progress bar: streaming lines that follow the newest, hold still while you read, and copy as text.",
  code: { rust: "error::log", qml: "Log" },
  stage: {
    live,
    hint: "scroll up and it holds; End follows; drag to select",
    daylight: true,
  },
  variants: [
    { name: "Wrapping", for: "broken between words: the default", sample: () => inRoom(log(INSTALL_LOG).box) },
    { name: "One line each", for: "wide machine output, scrolling across too", sample: () => inRoom(log(INSTALL_LOG, { unwrapped: true }).box) },
    { name: "Scrolled up", for: "holds still as lines arrive below", sample: () => inRoom(log(LONG, { down: 0.4 }).box) },
  ],
  sizes: [
    { name: "size.log", for: "opened under an error or a progress bar", sample: () => inRoom(log(LONG).box) },
    { name: "Its place's", for: "the room its place leaves it", sample: () => inRoom(log(INSTALL_LOG, { height: SHORT }).box) },
  ],
  states: {
    shown: ["rest", "lit", "empty"],
    absent: {
      pressed: "a drag selects its words",
      disabled: "shown or not",
      selected: "selected text is the text's own highlight",
      loading: "shows the lines it has",
      error: "error lines are a level, in `error`",
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
    looks: {
      rest: "`surface_container_lowest` at `opacity.log`, `mono` lines by level",
      lit: "the accent's tube inside its edge, glow at night",
      empty: "the well alone: no lines, no thumb",
    },
  },
  focus: {
    tubes: [{ name: "The log, with the keyboard", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the tube at `radius.content`, the glow outside",
      signsOff: "the tube as `primary_glass`, no glow",
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
      { name: "None", for: "the well alone", sample: () => inRoom(log([], { height: SHORT }).box) },
      { name: "One", for: "a line, at the bottom", sample: () => inRoom(log(INSTALL_LOG.slice(0, 1), { height: SHORT }).box) },
      { name: "Many", for: "it scrolls, at its newest line", sample: () => inRoom(log(LONG, { height: SHORT }).box) },
      ...LEVELS.map((level) => ({ name: `A ${level} line`, for: `${LEVEL_FOR[level]}, in \`${LEVEL_COLOURS[level]}\``, sample: () => inRoom(log([example(level)], { height: SHORT }).box) })),
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
      { keys: ["↑", "↓", "k", "j"], does: "scroll a line" },
      { keys: ["Page Up", "Page Down"], does: "scroll a page, less a line" },
      { keys: ["Home"], does: "the first line it keeps" },
      { keys: ["End"], does: "the newest line, following again" },
      { keys: ["Ctrl+A"], does: "select every line" },
      { keys: ["Ctrl+C"], does: "copy what is selected" },
    ],
    order: "One stop in reading order: after Show log, before the screen's next action.",
    lands: "Opening it leaves the keyboard on Show log; Tab moves onto it.",
  },
  measures: {
    samples: nonEmpty([{ name: "A log", for: "padding, lines, thumb", sample: () => inRoom(log(LONG, { height: SHORT }).box) }]),
    measures: [
      { part: "Height, opened", token: "size-log" },
      { part: "Lines in at the sides", token: "space-s16" },
      { part: "Above first, below last", token: "space-s12" },
      { part: "Line", token: "type-mono-line" },
      { part: "Thumb", token: "size-scrollbar" },
      { part: "Corner", token: "radius-content" },
    ],
    reflow: {
      note: "It fills its room: lines wrap to the width; one line each, it scrolls across.",
      sample: () => within("100%", log([{ level: "step", text: shown(LINE) }, ...INSTALL_LOG]).box),
    },
  },
});
