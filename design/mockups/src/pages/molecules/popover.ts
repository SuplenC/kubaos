// The Popover page: `docs/design/components/menu.md`, "Popover". The floating surface a menu sits
// on, raised to the ladder's step 2 over the window or the panel it is in. It
// isn't neon and isn't a control: it fades in as it opens and out as it closes, never flickering.

import { shown, type Words } from "../../lib/content.ts";
import { row, stack } from "../../lib/layout.ts";
import { menu } from "../../lib/menu.ts";
import { text, wrapped } from "../../lib/text.ts";
import { popover } from "../../lib/tooltip.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const NOTE: Words = { short: "Saved.", long: "Saved to Downloads, next to the three other copies of the same file you saved this morning." };

/** A tray app's menu: what a popover holds most often. */
function trayMenu(): HTMLElement {
  return menu(
    [
      { label: "Open", icon: "folder_open", keys: "Enter" },
      { label: "Copy path", icon: "content_paste", keys: "Ctrl+C" },
      "separator",
      { label: "Quit", icon: "logout", danger: true },
    ],
    { current: -1 },
  );
}

/** A popover `size.menu` wide holding a tray app's menu. */
function card(): HTMLElement {
  return popover(2, trayMenu());
}

/** A popover holding words: its own content, as wide as `size.menu`. */
function words(note: string = shown(NOTE)): HTMLElement {
  return popover(2, stack("s4", "start", text("Downloads", "body_strong"), wrapped(note, "small", "on_surface_variant")));
}

/** The popover over a few lines of the surface, so its step's shadow shows on something. */
function over(): HTMLElement {
  const below = stack("s8", "start", ...["Wi-Fi", "Bluetooth", "Output"].map((each) => text(each, "body", "on_surface_variant")));
  const room = document.createElement("span");
  room.className = "k-overlap";
  room.append(below, card());
  return room;
}

const OPENS: Move = {
  name: "Opens",
  what: "It fades in where it stays over `short` on `emphasized_decel`, with its shadow, never flickering or moving.",
  tracks: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  reduced: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: card,
};

const CLOSES: Move = {
  name: "Closes",
  what: "An item taken, Esc or a press elsewhere: it fades out over `micro` on `emphasized_accel`.",
  tracks: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  reduced: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: card,
};

export default page({
  level: "molecules",
  slug: "popover",
  title: "Popover",
  lede: "The floating surface a menu sits on, raised to step 2 over the window or panel it is in.",
  code: { rust: "menu::popover", qml: "Popover" },
  stage: { live: () => row("s32", "start", card(), words()), hint: "point at its menu's items", daylight: true },
  variants: [
    { name: "A menu's", for: "`size.menu` wide: a tray menu, a menu at the pointer", sample: card },
    { name: "Words", for: "a short note at its anchor", sample: () => words() },
  ],
  sizes: doesntApply("`size.menu` wide, as tall as what it holds."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "what it holds lights",
      pressed: "a press on it does nothing",
      disabled: "open or not there",
      selected: "its menu's items hold the choice",
      loading: "what it holds is known as it opens",
      empty: "with nothing to hold it doesn't open",
      error: "it has no value to be wrong",
    },
    rows: [
      { name: "A menu's", cell: card },
      { name: "Words", cell: () => words() },
    ],
    looks: {
      rest: "`surface_container`, `…_high` at night; step 2 shadow and rim",
    },
  },
  focus: doesntApply("a popover isn't neon: no line, no glow; the keyboard goes into what it holds."),
  content: {
    text: [{ name: "Words it holds", words: NOTE, sample: (note: string) => words(note) }],
    missingPicture: doesntApply("a popover draws no picture of its own."),
    counts: [
      { name: "One item", for: "a menu of one: still a menu's width", sample: () => popover(2, menu([{ label: "Quit", icon: "logout", danger: true }], { current: -1 })) },
      { name: "Many", for: "a tray menu", sample: card },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: powerMoves(over),
  },
  keyboard: {
    keys: [
      { keys: ["Esc"], does: "closes it; the keyboard goes back to its opener" },
      { keys: ["Tab", "Shift+Tab"], does: "inside what it holds" },
    ],
    order: "No stop of its own: the keyboard goes into what it holds.",
    lands: "On what it holds: a menu's picked or first item.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A menu's", for: "width, padding round a menu, corners, shadow", sample: card },
      { name: "Words", for: "padding round words", sample: () => words() },
    ]),
    measures: [
      { part: "Width", token: "size-menu" },
      { part: "Padding round a menu", token: "space-s4" },
      { part: "Padding round words", token: "space-s12" },
      { part: "Corners", token: "radius-control" },
      { part: "Rim", token: "stroke-hairline_device_px" },
      { part: "Shadow contact blur", token: "elevation-2-contact-blur" },
      { part: "Shadow ambient drop", token: "elevation-2-ambient-y" },
      { part: "Shadow ambient blur", token: "elevation-2-ambient-blur" },
    ],
    reflow: doesntApply("its width at every window size; near an edge it moves back inside."),
  },
});
