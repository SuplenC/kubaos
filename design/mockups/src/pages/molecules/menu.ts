// The Menu page, with MenuItem: `docs/design/components/menu.md`. A few actions or choices one
// under the other on a popover: a tray icon's menu, a select's list. The item the keyboard is on is
// lit on the secondary container; the pointer moving onto an item puts the keyboard on it.

import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type ItemPinned, type MenuEntry, type MenuItemSpec, menu, menuItem } from "../../lib/menu.ts";
import { text } from "../../lib/text.ts";
import { popover } from "../../lib/tooltip.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const ACTION: Words = { short: "Open", long: "Open the folder this download was saved in" };

/** A tray app's menu, as the specimens draw it: two groups of actions, a disabled one, and Quit. */
const ACTIONS: readonly MenuEntry[] = [
  { label: "Open", icon: "folder_open", keys: "Enter" },
  { label: "Copy path", icon: "content_paste", keys: "Ctrl+C" },
  "separator",
  { label: "Pin to the top", icon: "push_pin" },
  { label: "Forget", icon: "close", disabled: true },
  "separator",
  { label: "Quit", icon: "logout", danger: true },
];

/** A select's list: the holidays' countries, one checked. */
const COUNTRIES = ["None", "Germany", "Italy", "Poland"];

function choices(picked: number): MenuEntry[] {
  return COUNTRIES.map((label, index) => ({ label, checked: index === picked }));
}

/** `entries` on a menu's popover, the keyboard on `current`. */
function onPopover(entries: readonly MenuEntry[], current = 0, pinned?: ItemPinned): HTMLElement {
  return popover(2, menu(entries, { current, ...(pinned ? { pinned } : {}) }));
}

/** `content` on a popover as wide as its room, as a select's menu takes its select's width. */
function filled(content: HTMLElement): HTMLElement {
  const card = popover(2, content);
  card.style.setProperty("--popover-width", "100%");
  return card;
}

/** One item in the room a menu gives it: the popover's width less its padding. */
function alone(spec: MenuItemSpec, current = false, pinned?: ItemPinned): HTMLElement {
  return within("calc(var(--size-menu) - 2 * var(--space-s12))", menuItem(spec, current, pinned));
}

/** A tray menu that scrolls past eight items. */
function long(): HTMLElement {
  const months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
  return popover(2, menu(months.map((label, index) => ({ label, checked: index === 9 })), { current: 9, most: 8 }));
}

const ITEMS: readonly { readonly name: string; readonly spec: MenuItemSpec }[] = [
  { name: "With a symbol and keys", spec: { label: "Copy path", icon: "content_paste", keys: "Ctrl+C" } },
  { name: "A choice, checked", spec: { label: "Poland", checked: true } },
  { name: "A choice, unchecked", spec: { label: "Italy", checked: false } },
  { name: "Danger", spec: { label: "Quit", icon: "logout", danger: true } },
];

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

/** An item in `state`: lit is the keyboard on it. */
function cell(spec: MenuItemSpec, state: MatrixState): HTMLElement {
  if (state === "disabled") return alone({ ...spec, disabled: true });
  return alone(spec, state !== "rest", state === "rest" ? undefined : state === "lit" ? "lit" : "pressed");
}

const DESCRIBED =
  "An item is `size.control_sm` (32) tall, `space.s12` inside, its corners concentric inside the popover's (`radius.surface` − `space.s12`, the control radius): an optional `body` symbol, its words in `body` on one line, ending in an ellipsis where they don't fit, the keys that do the same as keycaps `space.s4` apart, and a check, `body`'s symbol, whose room is kept when unchecked so every item's end lines up; `space.s12` between each. Its content is `on_surface`, or `error` on an item that destroys something (Quit, Delete). A hairline `space.s4` clear of each side separates groups.";

export default page({
  level: "molecules",
  slug: "menu",
  title: "Menu and MenuItem",
  summary: "A few actions or choices one under the other on a popover: the item the keyboard is on is lit, and Enter takes it.",
  live: () => stack("s24", "start", onPopover(ACTIONS), onPopover(choices(3), 3)),
  description: [
    "Rust `menu::menu`, `menu::menu_item` and `menu::menu_separator`, QML `Menu` and `MenuItem`. Tab into a menu here and use ↑ and ↓, or move the pointer over it.",
    "A menu holds a few actions (a tray icon's app menu) or choices (a select's list), one under the other on a `Popover` as wide as `size.menu` or its select. It is for what doesn't fit the surface: a surface's own actions are its buttons.",
    DESCRIBED,
    "The item the keyboard is on is lit: it fades onto `secondary_container` over `short`, its content to `on_secondary_container`, a danger item's too; pressed, the container lifts to `secondary_container_pressed` over `micro`. The pointer moving onto an item puts the keyboard on it, so one item is lit at a time, by the pointer and the keys alike. An item that can't be taken now is dimmed, `on_surface` at `opacity.disabled_content`, and the keys pass over it. Items aren't neon: no tube, no glow, the popover's own edge their room.",
    "A menu taller than its room (a select's eight items, the room under a tray icon) scrolls, with no bar, the item the keyboard is on kept in view. In the tray's menus a submenu opens in place, with a Back item at its top; Esc steps back one level before it closes.",
  ],
  variants: [
    { name: "Actions", for: "What can be done to one thing: a tray app's menu, with symbols, keys and groups.", sample: () => onPopover(ACTIONS) },
    { name: "Choices", for: "One of a list, checked: a select's list. Every item keeps the check's room.", sample: () => onPopover(choices(3), 3) },
    { name: "MenuItem, with keys", for: "An action with a shortcut: its keys as keycaps at its end.", sample: () => alone({ label: "Copy path", icon: "content_paste", keys: "Ctrl+C" }) },
    { name: "MenuItem, danger", for: "An action that destroys something: in `error`.", sample: () => alone({ label: "Quit", icon: "logout", danger: true }) },
  ],
  sizes: doesntApply("an item is always a small control tall; a menu is as tall as its items, up to its room, and as wide as its popover."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "a checked choice is its check, in the rows here; the item the keyboard is on is lit.",
      loading: "a menu shows what it holds at once; a list still on its way isn't a menu yet.",
      empty: "a menu with nothing in it isn't shown: its owner leaves its control disabled.",
      error: "an item can't be wrong; one that can't be taken now is disabled.",
    },
    rows: nonEmpty(ITEMS.map(({ name, spec }) => ({ name, cell: (state: MatrixState) => cell(spec, state) }))),
    described: {
      rest: "No fill; its content `on_surface`, a danger item's `error`. By day the same roles in daylight colours.",
      lit: "The keyboard on it, which the pointer moves: `secondary_container` behind it, its content `on_secondary_container`, a danger item's too. No line and no glow, by night or by day.",
      pressed: "Lit and pressed: the container lifts to `secondary_container_pressed`, 0.035 lighter at night and 0.04 darker by day. Letting go takes it.",
      disabled: "Dimmed: `on_surface` at `opacity.disabled_content`, a danger item's too, never lit, and passed over by the keys.",
    },
  },
  focus: doesntApply("a menu's items light on the secondary container, with no tube and no glow: the keyboard is in the menu, and its popover's edge is right there. The menu takes the keyboard as one stop; the item it is on is its lit state."),
  content: {
    text: [{ name: "An item's words", words: ACTION, sample: (words: string) => onPopover([{ label: words, icon: "folder_open", keys: "Enter" }, { label: "Copy path", icon: "content_paste", keys: "Ctrl+C" }]) }],
    missingPicture: [{ name: "No symbol", for: "An item may have none: its words start `space.s12` in. A menu's items all have one or none.", sample: () => onPopover(choices(3), 3) }],
    counts: [
      { name: "One", for: "A single action: still a menu, as a tray app with only Quit has.", sample: () => onPopover([{ label: "Quit", icon: "logout", danger: true }]) },
      { name: "A few, in groups", for: "Groups apart by hairlines.", sample: () => onPopover(ACTIONS) },
      { name: "Past eight", for: "A list of twelve under a select: eight show and the rest scroll, the keyboard's item in view.", sample: long },
    ],
  },
  motion: {
    arrives: doesntApply("a menu comes with its popover (the Popover page)."),
    leaves: doesntApply("a menu goes with its popover."),
    between: [
      {
        name: "The keyboard moves to an item",
        what: "↓ or the pointer moves the keyboard to the next item: it lights up over `short` on `standard`, onto `secondary_container`, as the one it left goes back to rest over `short`. Never the flicker.",
        tracks: [
          { part: "the item left", on: "two items", target: ".k-menu-item:first-child", property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the item reached", on: "two items", target: ".k-menu-item:last-child", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => popover(2, menu([{ label: "Open", icon: "folder_open" }, { label: "Copy path", icon: "content_paste" }], { current: -1 })),
      },
      {
        name: "Pressed",
        what: "A lit item pressed: its container lifts over `micro`, and back as it comes up; letting go takes it.",
        tracks: [{ part: "pressed", on: "the lit item", target: ".k-menu-item", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => alone({ label: "Copy path", icon: "content_paste", keys: "Ctrl+C" }, true, "lit"),
      },
      ...powerMoves(() => onPopover(ACTIONS, 0, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "j", "Ctrl+J", "Ctrl+N"], does: "The next item that can be taken, round from the last to the first." },
      { keys: ["↑", "k", "Ctrl+K", "Ctrl+P"], does: "The item before that can be taken, round from the first to the last." },
      { keys: ["Home", "End"], does: "The first or the last item that can be taken." },
      { keys: ["Enter"], does: "Takes the item, once per press, never on key repeat; the menu closes." },
      { keys: ["Esc"], does: "Leaves the menu, or steps back out of a submenu first; the keyboard goes back to what opened it." },
    ],
    order: "A menu is one stop: the keys move inside it. Disabled items and the hairlines are passed over.",
    lands: "As it opens, on the picked choice of a select's list, or on its first item that can be taken.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A tray menu", for: "Each item's height, padding, gaps, keys, and the hairlines between groups.", sample: () => onPopover(ACTIONS) },
      { name: "A choice", for: "The check's room, kept when unchecked.", sample: () => alone({ label: "Italy", checked: false }, true) },
    ]),
    measures: [
      { what: "An item's height.", token: "size-control_sm" },
      { what: "Inside an item's sides, and between its parts.", token: "space-s12" },
      { what: "Between keycaps, and above and below a hairline.", token: "space-s4" },
      { what: "An item's symbol and its check.", token: "type-body-icon" },
      { what: "An item's words, `body`'s line.", token: "type-body-line" },
      { what: "A keycap.", token: "size-keycap" },
      { what: "The hairline between groups: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "The popover's corners, which an item's nest in.", token: "radius-surface" },
      { what: "A menu's width, a popover's.", token: "size-menu" },
    ],
    reflow: {
      described: "A menu is as wide as its popover (\`size.menu\`, or its select's width) at every window size, and as tall as its items up to its room, then it scrolls. An item's words end in an ellipsis where they don't fit: here on a popover held to a third of the window.",
      sample: () => within("33%", stack("s8", "start", filled(menu([{ label: shown(ACTION), icon: "folder_open", keys: "Enter" }], { current: -1 })), text("A popover a third of the window wide.", "small", "on_surface_variant"))),
    },
  },
});
