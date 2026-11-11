// The Menu page, with MenuItem: `docs/design/components/menu.md`. A few actions or choices one
// under the other on a popover: a tray icon's menu. A select's options are menu items too, on its
// own grown surface. The item the keyboard is on is lit on the secondary container; the pointer
// moving onto an item puts the keyboard on it.

import { shown, type Words } from "../../lib/content.ts";
import { row, within } from "../../lib/layout.ts";
import { type ItemPinned, type MenuEntry, type MenuItemSpec, menu, menuItem } from "../../lib/menu.ts";
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

/** A few choices: the holidays' countries, one checked. */
const COUNTRIES = ["None", "Germany", "Italy", "Poland"];

function choices(picked: number): MenuEntry[] {
  return COUNTRIES.map((label, index) => ({ label, checked: index === picked }));
}

/** `entries` on a menu's popover, the keyboard on `current`. */
function onPopover(entries: readonly MenuEntry[], current = 0, pinned?: ItemPinned): HTMLElement {
  return popover(2, menu(entries, { current, ...(pinned ? { pinned } : {}) }));
}

/** `content` on a popover as wide as its room. */
function filled(content: HTMLElement): HTMLElement {
  const card = popover(2, content);
  card.style.setProperty("--popover-width", "100%");
  return card;
}

/** One item in the room a menu gives it: the popover's width less its padding. */
function alone(spec: MenuItemSpec, current = false, pinned?: ItemPinned): HTMLElement {
  return within("calc(var(--size-menu) - 2 * var(--space-s4))", menuItem(spec, current, pinned));
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

export default page({
  level: "molecules",
  slug: "menu",
  title: "Menu and MenuItem",
  lede: "A few actions or choices one under the other on a popover: the item the keyboard is on is lit.",
  code: { rust: "menu::menu, menu::menu_item, menu::menu_separator", qml: "Menu, MenuItem" },
  stage: { live: () => row("s32", "start", onPopover(ACTIONS), onPopover(choices(3), 3)), hint: "Tab in and ↑ ↓, or point at it", daylight: true },
  variants: [
    { name: "Actions", for: "what can be done to one thing: symbols, keys, groups", sample: () => onPopover(ACTIONS) },
    { name: "Choices", for: "one of a few, checked; each item keeps the check's room", sample: () => onPopover(choices(3), 3) },
    { name: "MenuItem, with keys", for: "a shortcut's keys as keycaps at its end", sample: () => alone({ label: "Copy path", icon: "content_paste", keys: "Ctrl+C" }) },
    { name: "MenuItem, danger", for: "destroys something: in `error`", sample: () => alone({ label: "Quit", icon: "logout", danger: true }) },
    { name: "MenuItem, opening a submenu", for: "a chevron at its end: the tray's VPN Connections", sample: () => alone({ label: "VPN Connections", submenu: true }) },
  ],
  sizes: doesntApply("an item is a small control tall; a menu as tall as its items, as wide as its popover."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "a checked choice is its check",
      loading: "a menu shows what it holds at once",
      empty: "never shown empty: its owner disables its control",
      error: "an item can't be wrong",
    },
    rows: nonEmpty(ITEMS.map(({ name, spec }) => ({ name, cell: (state: MatrixState) => cell(spec, state) }))),
    looks: {
      rest: "no fill, `on_surface`; a danger item's `error`",
      lit: "`secondary_container`, keycaps `primary`; danger fills `error`; no line or glow",
      pressed: "lifts to `secondary_container_pressed`, or `error_pressed`",
      disabled: "`on_surface` at `opacity.disabled_content`, passed over by the keys",
    },
  },
  focus: doesntApply("items light on the secondary container, with no tube or glow; the menu is one stop."),
  content: {
    text: [{ name: "An item's words", words: ACTION, sample: (words: string) => onPopover([{ label: words, icon: "folder_open", keys: "Enter" }, { label: "Copy path", icon: "content_paste", keys: "Ctrl+C" }]) }],
    missingPicture: [{ name: "No symbol", for: "words start `space.s8` in; a menu's items all have one or none", sample: () => onPopover(choices(3), 3) }],
    counts: [
      { name: "One", for: "still a menu: a tray app with only Quit", sample: () => onPopover([{ label: "Quit", icon: "logout", danger: true }]) },
      { name: "A few, in groups", for: "groups apart by hairlines", sample: () => onPopover(ACTIONS) },
      { name: "Past eight", for: "twelve held to eight: the rest scroll", sample: long },
    ],
  },
  motion: {
    arrives: doesntApply("a menu comes with its popover (the Popover page)."),
    leaves: doesntApply("a menu goes with its popover."),
    between: [
      {
        name: "The keyboard moves to an item",
        what: "↓ or the pointer: it lights onto `secondary_container` over `short` on `standard` as the last goes back.",
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
        what: "Its container lifts over `micro`, and back as it comes up; letting go takes it.",
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
      { keys: ["↓", "j", "Ctrl+J", "Ctrl+N"], does: "next item that can be taken, wrapping round" },
      { keys: ["↑", "k", "Ctrl+K", "Ctrl+P"], does: "previous item that can be taken, wrapping round" },
      { keys: ["Home", "End"], does: "first or last item that can be taken" },
      { keys: ["Enter"], does: "takes it, never on key repeat; the menu closes" },
      { keys: ["Esc"], does: "leaves, or steps out of a submenu first" },
    ],
    order: "One stop: the keys move inside it, past disabled items and hairlines.",
    lands: "On its checked choice, or its first item that can be taken.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A tray menu", for: "item height, padding, gaps, keys, hairlines", sample: () => onPopover(ACTIONS) },
      { name: "A choice", for: "the check's room, kept when unchecked", sample: () => alone({ label: "Italy", checked: false }, true) },
    ]),
    measures: [
      { part: "Item height", token: "size-control_sm" },
      { part: "Item sides, gaps, hairline inset", token: "space-s8" },
      { part: "Popover padding, keycap gap", token: "space-s4" },
      { part: "Symbol, check", token: "icon-s16" },
      { part: "Item words' line", token: "type-body-line" },
      { part: "Keycap", token: "size-keycap" },
      { part: "Hairline", token: "stroke-hairline_device_px" },
      { part: "Popover corners", token: "radius-control" },
      { part: "Item corners", token: "radius-row" },
      { part: "Menu width", token: "size-menu" },
    ],
    reflow: {
      note: "As wide as its popover, as tall as its items up to its room, then it scrolls; words end in an ellipsis.",
      sample: () => within("33%", filled(menu([{ label: shown(ACTION), icon: "folder_open", keys: "Enter" }], { current: -1 }))),
    },
  },
});
