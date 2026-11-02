// The ErrorBlock page: `docs/design/components/error-block.md`. What broke and what to do next,
// shown where it broke, with the log behind it and the actions that follow it on the screen.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { log, type LogLine, logRoom } from "../../lib/log.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const ROOM = "var(--size-column)";

const MESSAGE: Words = { short: "Fuck. The disk is busy.", long: "Fuck. Couldn't partition /dev/nvme0n1: something still has it mounted, so the table can't be rewritten." };
const HINT: Words = { short: "Try again.", long: "Unmount it in a terminal and try again, or save the log and go read it: it has the gory details." };

const BUSY = { message: "Fuck. The disk is busy.", hint: "Something still has /dev/nvme0n1 mounted. Unmount it and try again." };
const COMMAND = {
  message: "sgdisk --zap-all /dev/nvme0n1 exited 2: Problem opening /dev/nvme0n1 for reading! Error is 16 (Device or resource busy).",
  hint: "The log has the gory details. Try again, or save it and go read it.",
};

export const INSTALL_LOG: readonly LogLine[] = [
  { level: "step", text: "==> Partitioning /dev/nvme0n1" },
  { level: "info", text: "Wiping the old partition table" },
  { level: "debug", text: "sgdisk --zap-all /dev/nvme0n1" },
  { level: "warn", text: "/dev/nvme0n1p2 is mounted at /run/media/kuba/data" },
  { level: "error", text: "Problem opening /dev/nvme0n1 for reading! Error is 16 (Device or resource busy)" },
  { level: "error", text: "partition: sgdisk exited 2" },
];

function inRoom(element: HTMLElement): HTMLElement {
  return within(ROOM, element);
}

/**
 * The block as Kōbō's failed screen shows it: the error, the log under it when it is open, and the
 * screen's actions at its foot: Show log beside the error's own actions, Save log and Try again.
 */
function onScreen(open: boolean, live = false): HTMLElement {
  const room = logRoom(log(INSTALL_LOG).box, open, "s16");
  const toggle = button({ label: open ? "Hide log" : "Show log", kind: "Text", icon: "terminal" });
  const retry = button({ label: "Try again", icon: "restart_alt" });
  const foot = row("s12", "centre", toggle, filling(document.createElement("span")), button({ label: "Save log", kind: "Tonal", icon: "save" }), retry);
  if (live) {
    toggle.addEventListener("click", () => {
      const opening = room.style.getPropertyValue("--log-open") !== "1";
      room.style.setProperty("--log-open", opening ? "1" : "0");
      toggle.querySelector(".k-text")!.textContent = opening ? "Hide log" : "Show log";
    });
  }
  return inRoom(stack("s16", "start", filling(stack(0, "start", errorBlock(BUSY), room)), filling(foot)));
}

const OPENS: Move = {
  name: "The log opens",
  what: "Show log: the room under the error opens from nothing to `size.log` over `medium` on `emphasized_decel`, a log opening, cutting the log as it goes; what is under it moves down with it. Hide log closes it the same way back. With reduced motion the room is open at once and the log fades in over `short`.",
  tracks: [{ part: "the log's room", on: "the failed screen", target: ".k-log-room", property: "--log-open", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [
    { part: "the log's room", on: "the failed screen", target: ".k-log-room", property: "--log-open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the log", on: "the failed screen", target: ".k-log", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => onScreen(false),
};

const DESCRIBED =
  "A status message lying flat in the page: `error_container` at `opacity.error_fill`, the sunken soft hairline inside its edge, `radius.content` and `space.s16` inside. Down its start side runs a rail of neon in `error`, from the top padding to the bottom one with round ends: the tabs' and the collapsible's rail made thicker, `banner.rail` at night and `banner.rail_day` by day, filling a lane `space.s4` wide `space.s12` in, so the words start `space.s12` past the lane, 28 in; `error_core` with the content glow round it at night while the signs are on, glass while they are off, `error` by day. In it the `error` symbol at `subheading`'s 24, `space.s16` from the message in `subheading` over the hint in `body`, `on_surface_variant`, `space.s4` under it; the symbol is centred down the words, and the words wrap.";

export default page<"error">({
  level: "molecules",
  slug: "error-block",
  title: "ErrorBlock",
  summary: "What broke and what to do next, where it broke: the message, the hint, and the log behind it.",
  live: () => onScreen(false, true),
  description: [
    "Rust `error::error_block(look, message, hint)`, QML `ErrorBlock` (`message`, `hint`). Kōbō's failed install and a setup step that failed. Press Show log: the log behind the error opens under it.",
    DESCRIBED,
    "A message past 90 characters (`LONG_MESSAGE`) drops to `body_strong`, since a whole command line in 16 px semibold is a wall. The message says what broke, in the voice of the place (`docs/BRAND.md`); the hint says the next thing to try. An error with no next action isn't finished.",
    "It holds no buttons. What to do about it is the screen's: the Log behind it, opened by a Text button with the `terminal` symbol, Save log (Tonal) and Try again, the screen's one Filled action, at the screen's foot. It is shown inline, where the user is looking (`docs/DESIGN.md`, \"Notify or show inline\"), and is never also a toast or a notification.",
  ],
  variants: [
    { name: "A short message", for: "What broke in a sentence, in `subheading`.", sample: () => inRoom(errorBlock(BUSY)) },
    { name: "A long message", for: "Past 90 characters, a command line and its output: `body_strong`.", sample: () => inRoom(errorBlock(COMMAND)) },
    { name: "With its log", for: "On a screen: the log open under it, the screen's actions at its foot.", sample: () => onScreen(true) },
  ],
  sizes: doesntApply("one size: its padding and its words' styles; its width is its room's."),
  states: {
    shown: ["error"],
    absent: {
      rest: "it shows only when something broke.",
      lit: "it takes no pointer and no focus; the actions round it are buttons.",
      pressed: "it takes no press.",
      disabled: "an error is shown or not.",
      selected: "it holds no choice.",
      loading: "trying again takes it away: the work shows its progress again.",
      empty: "an error always says what broke.",
    },
    rows: [
      { name: "Short", cell: () => inRoom(errorBlock(BUSY)) },
      { name: "Long", cell: () => inRoom(errorBlock(COMMAND)) },
    ],
    described: {
      error: `${DESCRIBED} By day the same roles in daylight colours, the rail \`error\` with no glow.`,
    },
  },
  focus: doesntApply("it takes no focus. Its rail is a fixed signal, always lit: it burns at night while the signs are on, flickers with them in a power change, and is the plain tone by day."),
  content: {
    text: [
      { name: "The message", words: MESSAGE, sample: (words: string) => inRoom(errorBlock({ ...BUSY, message: words })) },
      { name: "The hint", words: HINT, sample: (words: string) => inRoom(errorBlock({ ...BUSY, hint: words })) },
    ],
    missingPicture: doesntApply("its symbol is always `error`."),
    counts: doesntApply("one error says one thing; a second failure replaces it."),
  },
  motion: {
    arrives: doesntApply("it shows at once in place of the work that broke, with its screen."),
    leaves: doesntApply("trying again replaces it at once with the work's progress."),
    between: [OPENS, ...powerMoves(() => inRoom(errorBlock(BUSY)))],
  },
  keyboard: doesntApply("it takes no key; the screen's actions round it do, in reading order: Show log, Save log, Try again."),
  measures: {
    samples: nonEmpty([
      { name: "Short", for: "Its padding, the symbol and its gap, the words and theirs.", sample: () => inRoom(errorBlock(BUSY)) },
      { name: "With its log", for: "The gaps to the log and to the screen's actions.", sample: () => onScreen(true) },
    ]),
    measures: [
      { what: "Inside it at the top, the end and the foot, the rail's ends in from its top and foot, between the symbol and the words, and between it, the log and the actions.", token: "space-s16" },
      { what: "The rail's lane in from its start edge, and the words past the lane.", token: "space-s12" },
      { what: "The rail's lane, the rail centred in it.", token: "space-s4" },
      { what: "The rail, at night, before it is rounded to whole device pixels.", token: "banner-rail" },
      { what: "The rail by day.", token: "banner-rail_day" },
      { what: "Between the message and the hint.", token: "space-s4" },
      { what: "The symbol, `subheading`'s.", token: "type-subheading-icon" },
      { what: "The message's line, `subheading`'s.", token: "type-subheading-line" },
      { what: "The hint's, `body`'s.", token: "type-body-line" },
      { what: "Its corners.", token: "radius-content" },
      { what: "The log open under it.", token: "size-log" },
    ],
    reflow: {
      described: "It fills its room at every window size; its words wrap, and the symbol stays centred down them.",
      sample: () => within("100%", stack("s8", "start", errorBlock({ message: shown(MESSAGE), hint: shown(HINT) }), text("An error as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
