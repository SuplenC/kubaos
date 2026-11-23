// The Desktop page: the screen with no frame round it (`config/quickshell/modules/frame/Frame.qml`
// draws today's frame). The bar lies on the glass in one of three layouts, a setting: Bar, a band
// touching the top edge; Floating, one strip `size.gap` from the edges; Islands, a strip per group.
// Each panel is a detached card under the item that opens it, and the desktop never dims.

import { type BarLayout, BAR_LAYOUTS, twins } from "../../lib/sheet.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { powerMoves } from "../../site/moves.ts";
import { WINDOWS, windowSize } from "../../site/settings.ts";
import { controlCenter, notifications, system } from "../templates/panels.ts";
import { audioPanel } from "./audio-parts.ts";
import { barScreen, logicalWidth } from "./bar-parts.ts";

type State = "rest" | "open";

const LAPTOP = WINDOWS[2]!;
const LAYOUT_NAMES = ["Bar", "Floating", "Islands"] as const;

interface DesktopOptions {
  readonly layout?: BarLayout;
  readonly width?: string;
  readonly height?: string;
  /** The twin whose card is open: 0 the system monitor, 1 audio, 2 the bell, 3 status. */
  readonly open?: number;
  /** Its buttons open, swap and close the cards. */
  readonly live?: boolean;
}

/** The desktop in `layout`, the cards of the bar's four panel items on it, one open or none. */
function desktop(options: DesktopOptions = {}): HTMLElement {
  const room = barScreen(logicalWidth(options.width ?? LAPTOP.width), options.height ?? LAPTOP.height, options.layout ? { layout: options.layout } : {});
  const right = room.element.querySelector(".shell-bar-right");
  const items = right ? [...right.children].slice(-4) : [];
  const audio = audioPanel();
  const contents = [system().element, audio.element, notifications().element, controlCenter().element];
  const set = twins(
    room,
    contents.map((content, side) => ({ content, side, ...(items[side] ? { under: items[side] } : {}), ...(side === 1 ? { land: audio.land } : {}) })),
  );
  if (options.open !== undefined) {
    const shown = options.open;
    requestAnimationFrame(() => set.toggle(shown));
  }
  if (options.live) items.forEach((item, index) => item.addEventListener("click", () => set.toggle(index)));
  return room.element;
}

const ON = "the system monitor";

const OPENS: Move = {
  name: "A card opens",
  what: "Its item: the card fades in as it comes down `space.s8` into place, and the light lights.",
  tracks: [{ part: "the card", on: ON, target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the card", on: ON, target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => desktop({ open: 0 }),
};

const CLOSES: Move = {
  name: "A card closes",
  what: "Esc, a press outside, or its item: the card rises `space.s8` as it fades out.",
  tracks: [{ part: "the card", on: ON, target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: "the card", on: ON, target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => desktop({ open: 0 }),
};

export default page<State>({
  level: "shell",
  slug: "frame",
  title: "Desktop",
  lede: "No frame: the bar on the glass in three layouts, each panel a card under its item.",
  code: { qml: "Frame" },
  stage: {
    live: () => desktop({ live: true }),
    switches: {
      each: [{ label: "Bar layout", choices: ["Bar", "Floating", "Islands"] }],
      live: ([layout]) => desktop({ layout: BAR_LAYOUTS[layout ?? 0]!, live: true }),
    },
    hint: "click the monitor, audio, bell or status",
    daylight: true,
  },
  variants: [
    ...BAR_LAYOUTS.map((layout, index) => ({
      name: LAYOUT_NAMES[index]!,
      for: layout === "bar" ? "a band touching the top edge; the default" : layout === "floating" ? "one strip `size.gap` from the edges" : "a strip per group",
      sample: () => desktop({ layout }),
    })),
    ...BAR_LAYOUTS.map((layout, index) => ({
      name: `${LAYOUT_NAMES[index]!}, a card open`,
      for: "`size.gap` under the bar, under its item",
      sample: () => desktop({ layout, open: 1 }),
    })),
  ],
  sizes: doesntApply("its gaps are tokens at every screen size: the reflow shows them."),
  states: {
    shown: ["rest", "open"],
    absent: {
      lit: "takes no pointer; its bar's items light",
      pressed: "a click on the desktop closes a card",
      disabled: "always there",
      selected: "nothing to pick",
      loading: "drawn with the shell",
      empty: "the bar always has its clock",
      error: "nothing it can fail at",
    },
    rows: nonEmpty(BAR_LAYOUTS.map((layout, index) => ({ name: LAYOUT_NAMES[index]!, cell: (state: State) => desktop({ layout, ...(state === "open" ? { open: 0 } : {}) }) }))),
    looks: {
      rest: "the bar on the glass, nothing dimmed",
      open: "a card under its item, the light lit",
    },
  },
  focus: doesntApply("no line of its own; a card's controls light inside it."),
  content: {
    text: doesntApply("no words of its own; the bar's are the Bar page's."),
    missingPicture: doesntApply("the wallpaper is always there; the desktop draws none."),
    counts: doesntApply("one desktop per screen."),
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: powerMoves(() => desktop({ open: 0 })),
  },
  keyboard: {
    keys: [{ keys: ["Esc"], does: "closes the open card" }],
    order: "The desktop takes no Tab; an open card holds the keyboard on its screen.",
    lands: "A card closing gives the keyboard back to the window.",
  },
  measures: {
    samples: nonEmpty(BAR_LAYOUTS.map((layout, index) => ({ name: LAYOUT_NAMES[index]!, for: "the bar and a card under it", sample: () => desktop({ layout, open: 1 }) }))),
    measures: [
      { part: "Band, the Bar layout", token: "size-bar" },
      { part: "A strip", token: "size-bar_strip" },
      { part: "The items' row", token: "size-bar_row" },
      { part: "Every gap", token: "size-gap" },
      { part: "A card's corners", token: "radius-card" },
      { part: "Glass shadow's drop", token: "shadow-glass-y" },
      { part: "Glass shadow's blur", token: "shadow-glass-blur" },
    ],
    reflow: {
      note: "The gaps keep their tokens at every size; a card stops `size.gap` from the edges.",
      sample: () => {
        const size = windowSize();
        return desktop({ width: size.width, height: size.height, open: 3 });
      },
    },
  },
});
