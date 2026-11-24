// The Tray page: `config/quickshell/modules/bar/Tray.qml`. The apps' StatusNotifierItems: each
// app's picture in a circle a chip tall, and its menu drawn as our own Menu on a popover under it.

import type { Words } from "../../lib/content.ts";
import { type MenuEntry, menu } from "../../lib/menu.ts";
import { popover } from "../../lib/tooltip.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { onGlass, TRAY, type TrayApp, trayPill as pillOf } from "./bar-parts.ts";

/** The tray on a piece of the bar's glass, as the bar holds it. */
const trayPill = (...args: Parameters<typeof pillOf>) => onGlass(pillOf(...args));

type State = "rest" | "hovered" | "pressed" | "open";

/** nm-applet's menu, as the shots' stand-in reads it. */
const NETWORK: readonly MenuEntry[] = [
  { label: "Enable Networking", checked: true },
  { label: "Enable Wi-Fi", checked: true },
  "separator",
  { label: "VPN Connections", submenu: true },
  { label: "Connection Information", disabled: true },
  "separator",
  { label: "Edit Connections…" },
];

/** Its submenu: the way back first. */
const VPNS: readonly MenuEntry[] = [{ label: "Back", icon: "arrow_back" }, "separator", { label: "Office", checked: false }, { label: "Home lab", checked: true }];

/** The menu of the tray's `index`th app open under it, `entries` in it. */
function opened(entries: readonly MenuEntry[], index = 1, current?: number): HTMLElement {
  const room = document.createElement("span");
  room.className = "shell-tray-room";
  const tray = trayPill(TRAY, { index, state: "hover" });
  const card = popover(2, menu(entries, current === undefined ? {} : { current }));
  card.classList.add("shell-tray-menu");
  room.append(tray, card);
  requestAnimationFrame(() => {
    const item = tray.querySelectorAll<HTMLElement>(".shell-tray-item")[index];
    if (item) card.style.setProperty("--menu-x", `${item.getBoundingClientRect().left - room.getBoundingClientRect().left}px`);
  });
  return room;
}

/** The tray, live: Network's menu opens under it, its submenu in place, Esc backing out. */
function live(): HTMLElement {
  const room = document.createElement("span");
  room.className = "shell-tray-room";
  const tray = trayPill();
  room.append(tray);
  let card: HTMLElement | null = null;
  const close = () => {
    card?.remove();
    card = null;
  };
  const show = (entries: readonly MenuEntry[]) => {
    close();
    const list = menu(entries, {
      onTake: (index) => {
        if (entries === NETWORK && index === 3) show(VPNS);
        else if (entries === VPNS && index === 0) show(NETWORK);
        else close();
      },
      onLeave: () => (entries === VPNS ? show(NETWORK) : close()),
    });
    card = popover(2, list);
    card.classList.add("shell-tray-menu");
    const item = tray.querySelectorAll<HTMLElement>(".shell-tray-item")[1]!;
    card.style.setProperty("--menu-x", `${item.getBoundingClientRect().left - room.getBoundingClientRect().left}px`);
    room.append(card);
    list.focus({ preventScroll: true });
  };
  tray.querySelectorAll<HTMLElement>(".shell-tray-item")[1]!.addEventListener("click", () => (card ? close() : show(NETWORK)));
  return room;
}

const TITLE: Words = { short: "Files", long: "Syncthing: 3 folders up to date, 1 device connected" };

function many(): TrayApp[] {
  return [...TRAY, ...TRAY, TRAY[2]!];
}

export default page<State>({
  level: "shell",
  slug: "tray",
  title: "Tray",
  lede: "The apps' tray icons in one pill: each picture in a circle, its menu ours, under it.",
  code: { qml: "Tray" },
  stage: { live, hint: "click the star: its menu; Esc backs out", daylight: true },
  variants: [
    { name: "A coloured picture", for: "keeps its own colours", sample: () => trayPill([TRAY[2]!]) },
    { name: "A grey picture", for: "under 3:1 on the bar: drawn in `on_surface`", sample: () => trayPill([TRAY[1]!]) },
  ],
  sizes: doesntApply("one size: circles `size.chip`, pictures `icon.s16`."),
  states: {
    shown: ["rest", "hovered", "pressed", "open"],
    absent: {
      lit: "no keyboard focus: the bar is never a Tab stop",
      disabled: "an app's tray item is there or gone",
      selected: "no item holds a pick",
      loading: "the menu shows once its first entries arrive",
      empty: "no items: the pill is gone",
      error: "an app's own failures stay in its menu",
    },
    rows: nonEmpty([
      {
        name: "Network",
        cell: (state: State) => (state === "open" ? opened(NETWORK) : trayPill(TRAY, state === "rest" ? undefined : { index: 1, state: state === "hovered" ? "hover" : state })),
      },
      {
        name: "Its submenu",
        cell: (state: State) => (state === "open" ? opened(VPNS, 1, 3) : trayPill(TRAY, state === "rest" ? undefined : { index: 2, state: state === "hovered" ? "hover" : state })),
      },
    ]),
    looks: {
      rest: "pictures flat on the glass",
      hovered: "the edge light under the app, its hint",
      pressed: "its picture dips; the edge light full",
      open: "our Menu on a popover `space.s12` under it",
    },
  },
  focus: doesntApply("it takes no Tab; its open menu holds the keyboard, its items lit as a Menu's."),
  content: {
    text: [{ name: "Its hint, the app's title", words: TITLE, sample: () => trayPill([TRAY[2]!], { index: 0, state: "hover" }) }],
    missingPicture: [{ name: "No picture", for: "the circle keeps its room", sample: () => trayPill([TRAY[2]!, { title: "Blank", picture: "", missing: true }, TRAY[0]!]) }],
    counts: [
      { name: "None", for: "the pill is gone", sample: () => document.createElement("span") },
      { name: "One", for: "`space.s4` round one circle", sample: () => trayPill([TRAY[2]!]) },
      { name: "Many", for: "circles side by side, no gap", sample: () => trayPill(many()) },
    ],
  },
  motion: {
    arrives: doesntApply("an app's item shows at once; the pill eases to its width."),
    leaves: doesntApply("it goes at once; the pill eases."),
    between: [
      {
        name: "Lit",
        what: "The pointer onto an app: the edge light comes in under it over `short` on `standard`.",
        tracks: [{ part: "the edge light", on: "Network", target: ".k-edge-hover", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => trayPill(TRAY, { index: 1, state: "hover" }),
      },
      ...powerMoves(() => opened(NETWORK)),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "j", "k"], does: "the menu's items, round" },
      { keys: ["Enter"], does: "takes one; a submenu opens in place" },
      { keys: ["Esc"], does: "back out of a submenu, then close" },
    ],
    order: "The tray takes no Tab; the open menu holds the keyboard until it closes.",
    lands: "Opening, on the menu with no item picked; closing, back where it was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The pill", for: "circles and its ends", sample: () => trayPill() },
      { name: "Its menu", for: "the popover under the icon", sample: () => opened(NETWORK) },
    ]),
    measures: [
      { part: "Height", token: "size-bar_row" },
      { part: "Ends", token: "space-s4" },
      { part: "An app's target", token: "size-chip" },
      { part: "Picture", token: "icon-s16" },
      { part: "Menu's width", token: "size-menu" },
      { part: "Icon to the menu", token: "space-s12" },
    ],
    reflow: doesntApply("as wide as its circles; the menu stays inside the screen, `space.s8` in."),
  },
});
