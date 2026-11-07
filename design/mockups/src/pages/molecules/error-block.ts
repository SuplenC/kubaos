// The ErrorBlock page: `docs/design/components/error-block.md`. What broke and what to do next,
// shown where it broke, with the log behind it and the actions that follow it on the screen.

import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { errorBlock } from "../../lib/error-block.ts";
import { filling, row, stack, within } from "../../lib/layout.ts";
import { log, type LogLine, logRoom } from "../../lib/log.ts";
import { advance } from "../../lib/text.ts";
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

/** Says `words` on `toggle`: its label and the lit layer's copy of it, each measured anew. */
function relabel(toggle: HTMLElement, words: string): void {
  for (const label of toggle.querySelectorAll<HTMLElement>(".k-text")) {
    label.textContent = words;
    label.style.setProperty("--natural", String(advance("button", words)));
  }
}

/**
 * The block as Kōbō's failed screen shows it: the error, the screen's actions under it (Show log
 * beside the error's own actions, Save log and Try again), and the log under them when it is open,
 * so opening it moves none of them.
 */
function onScreen(open: boolean, live = false, width = ROOM): HTMLElement {
  const room = logRoom(log(INSTALL_LOG).box, open, "s16");
  const toggle = button({ label: open ? "Hide log" : "Show log", kind: "Text", icon: "terminal" });
  const retry = button({ label: "Try again", icon: "restart_alt" });
  const foot = row("s12", "centre", toggle, filling(document.createElement("span")), button({ label: "Save log", kind: "Tonal", icon: "save" }), retry);
  if (live) {
    toggle.addEventListener("click", () => {
      const opening = room.style.getPropertyValue("--log-open") !== "1";
      room.style.setProperty("--log-open", opening ? "1" : "0");
      relabel(toggle, opening ? "Hide log" : "Show log");
    });
  }
  return within(width, stack(0, "start", filling(stack("s16", "start", filling(errorBlock(BUSY)), filling(foot))), room));
}

const OPENS: Move = {
  name: "The log opens",
  what: "The room under the actions opens to `size.log` over `medium` on `emphasized_decel`; nothing above it moves.",
  tracks: [{ part: "the log's room", on: "the failed screen", target: ".k-log-room", property: "--log-open", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [
    { part: "the log's room", on: "the failed screen", target: ".k-log-room", property: "--log-open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the log", on: "the failed screen", target: ".k-log", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => onScreen(false),
};

export default page<"error">({
  level: "molecules",
  slug: "error-block",
  title: "ErrorBlock",
  lede: "What broke and what to do next, where it broke: the message, the hint, and the log behind it.",
  code: { rust: "error::error_block", qml: "ErrorBlock" },
  stage: {
    live: () => onScreen(false, true, "100%"),
    hint: "press Show log",
    daylight: true,
  },
  variants: [
    { name: "A short message", for: "a sentence, in `subheading`", sample: () => inRoom(errorBlock(BUSY)) },
    { name: "A long message", for: "past 90 characters: `body_strong`", sample: () => inRoom(errorBlock(COMMAND)) },
    { name: "With its log", for: "the screen's actions, the log open under them", sample: () => onScreen(true) },
  ],
  sizes: doesntApply("one size; its width is its room's."),
  states: {
    shown: ["error"],
    absent: {
      rest: "shows only when something broke",
      lit: "takes no pointer or focus",
      pressed: "takes no press",
      disabled: "shown or not",
      selected: "holds no choice",
      loading: "trying again takes it away",
      empty: "always says what broke",
    },
    rows: [
      { name: "Short", cell: () => inRoom(errorBlock(BUSY)) },
      { name: "Long", cell: () => inRoom(errorBlock(COMMAND)) },
    ],
    looks: {
      error: "`error_container` at `opacity.error_fill`, a rail in `error`, no glow by day",
    },
  },
  focus: doesntApply("it takes no focus; its rail is a fixed signal, burning at night and plain by day."),
  content: {
    text: [
      { name: "The message", words: MESSAGE, sample: (words: string) => inRoom(errorBlock({ ...BUSY, message: words })) },
      { name: "The hint", words: HINT, sample: (words: string) => inRoom(errorBlock({ ...BUSY, hint: words })) },
    ],
    missingPicture: doesntApply("its symbol is always `error`."),
    counts: doesntApply("one error says one thing; a second replaces it."),
  },
  motion: {
    arrives: doesntApply("it shows at once in place of the work that broke."),
    leaves: doesntApply("trying again replaces it at once with the work's progress."),
    between: [OPENS, ...powerMoves(() => inRoom(errorBlock(BUSY)))],
  },
  keyboard: doesntApply("it takes no key; the screen's actions under it do: Show log, Save log, Try again."),
  measures: {
    samples: nonEmpty([
      { name: "Short", for: "padding, symbol, words, their gaps", sample: () => inRoom(errorBlock(BUSY)) },
      { name: "With its log", for: "the gaps to the actions and the log", sample: () => onScreen(true) },
    ]),
    measures: [
      { part: "Inset, gaps to actions and log", token: "space-s16" },
      { part: "Rail lane in, words past it", token: "space-s12" },
      { part: "Rail lane", token: "space-s4" },
      { part: "Rail at night", token: "banner-rail" },
      { part: "Rail by day", token: "banner-rail_day" },
      { part: "Message → hint", token: "space-s4" },
      { part: "Symbol", token: "type-subheading-icon" },
      { part: "Message line", token: "type-subheading-line" },
      { part: "Hint line", token: "type-body-line" },
      { part: "Corner", token: "radius-content" },
      { part: "Log, open", token: "size-log" },
    ],
    reflow: {
      note: "It fills its room; its words wrap, the symbol centred down them.",
      sample: () => within("100%", errorBlock({ message: shown(MESSAGE), hint: shown(HINT) })),
    },
  },
});
