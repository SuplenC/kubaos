// The Popover page: `docs/design/components/menu.md`, "Popover". The floating surface a menu or a
// select's list sits on, raised to the ladder's step 2 over the window or the panel it is in. It
// isn't neon and isn't a control: it fades in as it opens and out as it closes, never flickering.

import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
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

/** A popover as wide as its room: a select's list takes the select's width. */
function wide(): HTMLElement {
  const made = popover(2, menu(["None", "Germany", "Italy", "Poland"].map((label, index) => ({ label, checked: index === 3 })), { current: 3 }));
  made.style.setProperty("--popover-width", "100%");
  return within("var(--size-column)", made);
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
  what: "Summoned to act and go, it arrives one step quicker than a panel: it fades in over `short` on `emphasized_decel`, where it will stay, with its shadow; it doesn't move or grow, and never flickers: a popover isn't a light. With reduced motion it is there at once.",
  tracks: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  reduced: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: card,
};

const CLOSES: Move = {
  name: "Closes",
  what: "An item taken, Esc or a press elsewhere: it fades out over `micro` on `emphasized_accel`, where it is, so what was taken runs at once. With reduced motion it is gone at once.",
  tracks: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  reduced: [{ part: "popover", on: "a tray menu", target: ".k-popover", property: "opacity", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: card,
};

const DESCRIBED =
  "`surface_container`, `surface_container_high` at night (a raised part takes the container role one above its day one), at the surface radius (`radius.surface`, 24), `space.s12` inside, so the items in it take 24 − 12 = 12, the control radius. It is raised to the ladder's step 2 over the plane it is in: the contact layer, 4 blur, and the ambient layer, 8 down and 32 blur, both of `shadow`, stronger at night; and the rim, a hairline of `on_surface` inside its edge at `elevation.rim` (`light_rim` by day). It is `size.menu` (240) wide, or as wide as its select.";

export default page({
  level: "molecules",
  slug: "popover",
  title: "Popover",
  summary: "The floating surface a menu or a select's list sits on: raised to step 2 over the window or the panel it is in.",
  live: () => row("s32", "start", card(), words()),
  description: [
    "Rust `menu::popover`, QML `Popover`. It floats over the window or the panel at its anchor: under a select, as wide as it and `space.s4` below; under a tray icon; at the pointer for a menu it opens. `kubaos-ui` draws one where the app lays it out; in QML it floats over the window's content, so no panel that clips cuts it.",
    DESCRIBED,
    "It floats over everything in its plane, no part of the layout, so it moves nothing as it comes and goes; a press on it stays on it. It is held inside the window or the panel: one that would hang past an edge moves back until its edge is on it. A menu or a tooltip that reaches past its window's edge keeps its step's shadow.",
    "It opens over `short` and closes over `micro`, a fade with no move, as a surface a key or a press summons to act and go: never the flicker, since it isn't a light.",
  ],
  variants: [
    { name: "A menu's", for: "`size.menu` wide: a tray app's menu, a menu at the pointer.", sample: card },
    { name: "A select's", for: "As wide as its select, `space.s4` under it.", sample: wide },
    { name: "Words", for: "Something to say that a tooltip is too small for and a dialog too much: a short note at its anchor.", sample: () => words() },
  ],
  sizes: doesntApply("a popover is `size.menu` wide, or its select's width, and as tall as what it holds."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "it takes no pointer of its own and no focus: what it holds lights.",
      pressed: "a press on it stays on it and does nothing of its own.",
      disabled: "a popover is open or not there.",
      selected: "it holds no choice: its menu's items do.",
      loading: "what it holds is known as it opens.",
      empty: "a popover with nothing to hold isn't opened.",
      error: "it has no value to be wrong.",
    },
    rows: [
      { name: "A menu's", cell: card },
      { name: "Words", cell: () => words() },
    ],
    described: {
      rest: `Open. ${DESCRIBED} By day the same roles in daylight colours, with the day's lighter shadow and fainter rim.`,
    },
  },
  focus: doesntApply("a popover isn't a control and isn't neon: it has no line and no glow. The keyboard goes into what it holds."),
  content: {
    text: [{ name: "Words it holds", words: NOTE, sample: (note: string) => words(note) }],
    missingPicture: doesntApply("a popover draws no picture of its own."),
    counts: [
      { name: "One item", for: "A menu of one: still as wide as a menu.", sample: () => popover(2, menu([{ label: "Quit", icon: "logout", danger: true }], { current: -1 })) },
      { name: "Many", for: "A tray menu.", sample: card },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: powerMoves(over),
  },
  keyboard: {
    keys: [
      { keys: ["Esc"], does: "Closes it; the keyboard goes back to what opened it." },
      { keys: ["Tab", "Shift+Tab"], does: "Inside what it holds: a popover is no stop of its own." },
    ],
    order: "No stop of its own: the keyboard goes into what it holds as it opens.",
    lands: "On what it holds: a menu's picked or first item.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A menu's", for: "Its width, padding, corners and shadow.", sample: card },
      { name: "Words", for: "Its padding round what it holds.", sample: () => words() },
    ]),
    measures: [
      { what: "Its width, a menu's.", token: "size-menu" },
      { what: "Its padding.", token: "space-s12" },
      { what: "Its corners.", token: "radius-surface" },
      { what: "The rim inside its edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "Its shadow's contact layer: its blur.", token: "elevation-2-contact-blur" },
      { what: "Its shadow's ambient layer: its drop.", token: "elevation-2-ambient-y" },
      { what: "Its shadow's ambient layer: its blur.", token: "elevation-2-ambient-blur" },
      { what: "The gap under a select.", token: "space-s4" },
    ],
    reflow: doesntApply("a popover is its width at every window size; near the window's edge it moves back to stay inside."),
  },
});
