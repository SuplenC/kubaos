// The ListDetail page: `docs/design/components/list-detail.md`. A list beside the detail of its
// selected row, as its two builds draw it today: the Command menu's Results beside its pane, and
// Kōbō's first-boot cards beside the picked card's page. Its new design is sketched later; every
// difference between the two builds is a question on the page's issue.

import "./list-detail-page.css";
import { banner } from "../../lib/banner.ts";
import { type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { commandMenu, menuMeasures, menuOnScreen, type Result, type SetupCard, setupWindow } from "../../lib/list-detail.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { windowSize } from "../../site/settings.ts";

const TITLE: Words = { short: "Files", long: "Files, the file manager that opens where you last were" };

/** The Command menu's Results for "fi". */
function results(title = "Files"): readonly Result[] {
  const enter = { title: "Open", keys: "↵" };
  return [
    { row: { title: "Firefox", matched: [{ start: 0, end: 2 }], subtitle: "Web browser", icon: "public", tile: true, tall: true, clear: true, tight: true }, kind: "Application", actions: [enter, { title: "Open in a new window", keys: "⌃↵" }, { title: "Forget", keys: "⌃D" }], hints: [["↵", "Open"], ["⌃K", "Actions"]] },
    { row: { title, matched: [{ start: 0, end: 2 }], subtitle: "Browse the disk", icon: "folder", tile: true, tall: true, clear: true, tight: true }, kind: "Application · fi", actions: [enter, { title: "Open the home folder", keys: "⌃H" }], hints: [["↵", "Open"], ["⌃K", "Actions"]] },
    { row: { title: "Focus stutter: off", matched: [{ start: 0, end: 1 }], icon: "bolt", tile: true, tall: true, clear: true, tight: true }, kind: "Setting", actions: [{ title: "Turn it on", keys: "↵" }], hints: [["↵", "Turn on"]] },
    { row: { title: "Find on screen", matched: [{ start: 0, end: 2 }], subtitle: "Search what's showing", icon: "travel_explore", tile: true, tall: true, clear: true, tight: true }, kind: "Command", actions: [enter], hints: [["↵", "Run"]] },
    { row: { title: "Fingerprint", matched: [{ start: 0, end: 2 }], subtitle: "First boot's card", icon: "fingerprint", tile: true, tall: true, clear: true, tight: true }, kind: "Setting", actions: [enter], hints: [["↵", "Open"]] },
    { row: { title: "Fix the clock", matched: [{ start: 0, end: 2 }], subtitle: "Sync with the network", icon: "schedule", tile: true, tall: true, clear: true, tight: true }, kind: "Command", actions: [enter], hints: [["↵", "Run"]] },
    { row: { title: "Flight mode", matched: [{ start: 0, end: 1 }], subtitle: "Every radio off", icon: "wifi_off", tile: true, tall: true, clear: true, tight: true }, kind: "Setting", actions: [enter], hints: [["↵", "Turn on"]] },
    { row: { title: "Fan curve", matched: [{ start: 0, end: 1 }], subtitle: "System monitor", icon: "mode_fan", tile: true, tall: true, clear: true, tight: true }, kind: "Setting", actions: [enter], hints: [["↵", "Open"]] },
  ];
}

/** First boot's cards. */
const CARDS: readonly SetupCard[] = [
  { step: { title: "Fingerprint", subtitle: "Unlock with a finger on the laptop's reader.", icon: "fingerprint", state: { is: "done" } }, chip: { kind: "Accent", label: "Done", icon: "check" }, explain: ["Two fingers are enrolled: the right index and the right thumb.", "The disk, sudo and the lock screen take either. The password still works when the reader doesn't."], primary: "Enroll another", skip: "Next" },
  { step: { title: "YubiKey", subtitle: "Touch the key to unlock the disk and sudo.", icon: "key", state: { is: "todo" } }, chip: { kind: "Neutral", label: "To do" }, explain: ["Plug the key in and touch it when it blinks.", () => banner({ tone: "Warning", icon: "warning", title: "Keep a second key", body: "Lose the only one and the password is all that's left." })], secondary: "Not now", primary: "Set it up", skip: "Skip" },
  { step: { title: "Secure Boot", subtitle: "Enroll the key, then turn it on in the firmware.", icon: "lock", state: { is: "action", label: "Reboot", glyph: "restart_alt" } }, chip: { kind: "Filled", label: "Reboot", icon: "restart_alt" }, explain: ["The key is enrolled. On the next boot a blue screen asks for the one-time password below.", "Type it, pick Continue, and turn Secure Boot on in the firmware's settings."], primary: "Reboot now", skip: "Next" },
  { step: { title: "Snapshots", subtitle: "Roll back a bad update from the boot menu.", icon: "schedule", state: { is: "todo" }, words: { label: "Waiting", glyph: "schedule" } }, chip: { kind: "Neutral", label: "Waiting", icon: "schedule" }, explain: ["Waiting on the first update: there's nothing to roll back to yet."], skip: "Skip" },
  { step: { title: "Bitwarden", subtitle: "Your passwords, unlocked with the login.", icon: "lock", state: { is: "todo" } }, chip: { kind: "Neutral", label: "To do" }, explain: ["Sign in once; after that the login unlocks it."], primary: "Sign in", skip: "Skip" },
  { step: { title: "Git", subtitle: "Sign your commits with the YubiKey.", icon: "code", state: { is: "todo" } }, chip: { kind: "Neutral", label: "To do" }, explain: ["Needs the YubiKey first."], primary: "Set it up", skip: "Skip" },
];

/** The menu alone, its measures a screen's. */
function menu(width: number, height: number, extra: { rows?: number; current?: number; title?: string } = {}): HTMLElement {
  return commandMenu({ results: results(extra.title), query: "fi", ...(extra.rows ? { rows: extra.rows } : {}), ...(extra.current !== undefined ? { current: extra.current } : {}) }, menuMeasures(width, height));
}

/** `content` in a box `width` × `height` (lengths' CSS). */
function framed(content: HTMLElement, width: string, height: string): HTMLElement {
  const box = within(width, content);
  box.classList.add("t-frame");
  box.style.height = height;
  return box;
}

function setup(extra: { selected?: number; rising?: number; online?: boolean } = {}): HTMLElement {
  return setupWindow({ cards: CARDS, ...extra });
}

const OPENS: Move = {
  name: "The Command menu opens",
  what: "Super+Space: the panel fades in and drops `space.s8` into place over `short` on `emphasized_decel`, one step quicker than a panel's `long`, since a key summons it many times an hour. Its Results are there in its first frame. With the setting \"fades\" or reduced motion it only fades.",
  tracks: [
    { part: "fade", on: "the panel", target: ".k-menu", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "drop", on: "the panel", target: ".k-menu", property: "--menu-drop", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "fade", on: "the panel", target: ".k-menu", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => stack(0, "start", menu(1280, 800, { rows: 3 })),
};

const CLOSES: Move = {
  name: "The Command menu closes",
  what: "Esc, or an Item run: the panel fades out and lifts `space.s8` over `micro` on `emphasized_accel`; what it runs waits for it to be gone.",
  tracks: [
    { part: "fade", on: "the panel", target: ".k-menu", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" },
    { part: "drop", on: "the panel", target: ".k-menu", property: "--menu-drop", from: 0, to: 1, duration: "micro", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "fade", on: "the panel", target: ".k-menu", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => stack(0, "start", menu(1280, 800, { rows: 3 })),
};

const FOLLOWS: Move = {
  name: "The pane follows the selection",
  what: "↓ moves the list's selection (it slides between rows over `short`, the List page); the pane crossfades from the row left to the row reached over `short` on `standard`, once typing has paused. Results a keystroke brought swap the pane at once, and the same row changing in place just updates.",
  tracks: [{ part: "the row reached", on: "the pane's content", target: ".k-menu-pane-content", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: () => stack(0, "start", menu(1536, 864, { rows: 4, current: 1 })),
};

const PUSHED: Move = {
  name: "A View pushed",
  what: "An Item that opens a View (a list, a form, a detail): the body slides `space.s24` in from the right and settles from 0.4 to whole over `medium` on `emphasized_decel`, so a pushed list's rows read in its first frame. Back slides it in from the left. With \"fades\" or reduced motion it only settles.",
  tracks: [
    { part: "slide", on: "the body", target: ".k-menu-body", property: "--view-push", from: 1, to: 0, duration: "medium", curve: "emphasized_decel" },
    { part: "settle", on: "the body", target: ".k-menu-body", property: "opacity", from: 0.4, to: 1, duration: "medium", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "settle", on: "the body", target: ".k-menu-body", property: "opacity", from: 0.4, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => stack(0, "start", menu(1536, 864, { rows: 3 })),
};

const RISES: Move = {
  name: "Kōbō's card page rises in",
  what: "A card picked (↑ ↓, or a click): its page rises `space.s16` into place and fades in over `long` on `emphasized_decel`, its offset on whole device pixels; the list doesn't move. With reduced motion it only fades.",
  tracks: [
    { part: "rise", on: "the card's page", target: ".k-setup-room", property: "--setup-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "fade", on: "the card's page", target: ".k-setup-room", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "fade", on: "the card's page", target: ".k-setup-room", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => framed(setup({ selected: 1 }), "var(--size-app_min_width)", "var(--size-app_min_height)"),
};

export default page<"rest" | "selected" | "empty" | "error" | "loading">({
  level: "templates",
  slug: "list-detail",
  title: "ListDetail",
  summary: "A list beside the detail of its selected row, as the Command menu and Kōbō's first-boot setup draw it today.",
  live: () => menu(1536, 864),
  description: [
    "Two builds, one template to be: the Command menu's Results beside its pane (`config/quickshell/modules/menu/Menu.qml`, `DetailPane.qml`), and Kōbō's first-boot cards beside the picked card's page (`crates/kubaos-kobo/src/setup/view.rs`). Its new design is sketched once the mockups are done; this page mocks what both draw now, side by side, so every difference shows. ↑ and ↓ on the menu move its selection and the pane follows; on Kōbō's list they pick a card.",
    "**The Command menu** is a panel of the shell, sized by its screen: half the screen's width, from one `size.column` to two, on the 4 grid; rows for about two thirds of its height, 8 to 14, each `size.row_two_line` with `space.s4` between; two fifths down the room it leaves. It is the frame's glass (`surface` at `opacity.frame`) in the soft hairline at `radius.surface`, `space.s12` inside and between its parts: the search field (a md `Field`), the body, a hairline across the whole panel, and the footer, a sm control tall, holding the keys. The body is the Results (`ScrollingList`, tall tiled rows, the selection the list's and sliding) and, where the panel is as wide as `size.dialog` beside `size.rail`, the pane: a `Card` two fifths of the room and a rail wide at the least, holding the selected row's 48 symbol, its title in `heading`, its subtitle, its kind in `label`, `space.s24` then its Actions with their keycaps, and its keys at its foot. Without the pane the footer shows the row's keys instead.",
    "**Kōbō's setup** is an app's window, one solid `surface`. The side is a third of the window and never wider than `size.column`, on the well's fill: a clear `Page` holding the header (初期設定, First boot, its line), `space.s20`, the cards as two-line `Step`s in a `ScrollingList` that takes the height left, `space.s12` twice (a \"Checking again…\" line between them while it re-detects), the Online chip, `space.s16`, Done for now (`Tonal`), `space.s16` and the keys flowing onto lines. Beside it the picked card's page, a clear `Page`: its 48 symbol beside its title in `title` with its state's chip and its summary, then `space.s24` apart its words in a `Scroll` and its actions: Skip or Next (`Text`) on the left, a `Tonal` and the `Filled` action on the right.",
  ],
  variants: [
    { name: "The Command menu", for: "A shell panel summoned by a key: the list is what you search, the pane what the row does. Drawn for a 1536 × 864 screen (1080p at 1.25), where the pane fits.", sample: () => menu(1536, 864) },
    { name: "Kōbō's setup", for: "An app's window: the list is what's left to do, the page how to do it. At an app's default size.", sample: () => framed(setup(), "var(--size-app_width)", "var(--size-app_height)") },
  ],
  sizes: [
    { name: "The Command menu without its pane", for: "A 1280 × 800 screen: a 640 panel is narrower than `size.dialog` beside `size.rail`, so the list takes the body and the footer shows the row's keys.", sample: () => menu(1280, 800) },
    { name: "The Command menu on a screen of the toolbar's window size", for: "Its width, rows and place follow the screen: pick a window size in the toolbar. On a screen too short for eight rows the mock lets it take fewer (a question).", sample: () => framed(menuOnScreen({ results: results(), query: "fi" }), windowSize().width, windowSize().height) },
  ],
  states: {
    shown: ["rest", "selected", "empty", "error", "loading"],
    absent: {
      lit: "the template takes no pointer of its own: its list, rows, cards and buttons light, each on its page (List, Step, Card, Button).",
      pressed: "the same: what it holds is pressed, never the template.",
      disabled: "a row that can't run is dimmed by its list (an unavailable Result); the template never is.",
    },
    rows: [
      {
        name: "The Command menu",
        cell: (state) => {
          if (state === "empty") return commandMenu({ results: [], query: "fiq", rows: 2 }, menuMeasures(1536, 864));
          if (state === "error") return text("A failed Action's reason in `error` across the footer.", "small", "error");
          if (state === "loading") return text("Not drawn: Results come with the first frame.", "small", "on_surface_variant");
          return menu(1536, 864, { rows: 2, current: state === "selected" ? 1 : 0 });
        },
      },
      {
        name: "Kōbō's setup",
        cell: (state) => {
          if (state === "loading") return text("Checking what this machine has…, a spinner in the list's place.", "small", "on_surface_variant");
          if (state === "empty") return text("Doesn't occur: every machine has its cards.", "small", "on_surface_variant");
          if (state === "error") return framed(setup({ online: false }), "var(--size-app_min_width)", "var(--size-app_min_height)");
          return framed(setup({ selected: state === "selected" ? 2 : 0 }), "var(--size-app_min_width)", "var(--size-app_min_height)");
        },
      },
    ],
    described: {
      rest: "The first row selected, its detail beside it. By day the same roles in daylight colours.",
      selected: "Another row picked: the menu's selection slides to it and the pane follows; Kōbō's card turns `secondary_container` and its page rises in.",
      empty: "The menu with no Results: an `EmptyState` (`search_off`, No results) in the list's room, and the pane's own (Nothing selected).",
      error: "The menu: why an Action failed, in `error` across the footer. Kōbō: offline, the chip saying what needs the internet.",
      loading: "The menu never waits: Results come with the frame that typed them. Kōbō's first detection: a spinner and its line in the list's place.",
    },
  },
  focus: doesntApply("the template lights no line of its own: the menu's field keeps the keyboard (its tube, the Field page), and Kōbō's list lights its well (ScrollingList)."),
  content: {
    text: [{ name: "A row's title", words: TITLE, sample: (words: string) => menu(1536, 864, { rows: 2, current: 1, title: words }) }],
    missingPicture: doesntApply("a Result without an icon of its own shows its Extension's symbol on its tile (IconTile); a card's symbol is one of KubaOS Symbols."),
    counts: [
      { name: "None", for: "The menu's empty state in the list, Nothing selected in the pane.", sample: () => commandMenu({ results: [], query: "fiq", rows: 4 }, menuMeasures(1536, 864)) },
      { name: "One", for: "One Result, selected, the pane on it.", sample: () => commandMenu({ results: results().slice(0, 1), query: "firef", rows: 4 }, menuMeasures(1536, 864)) },
      { name: "Many", for: "More than the body holds: the list scrolls in its well, keeping the selection in view.", sample: () => menu(1536, 864, { rows: 4 }) },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: [FOLLOWS, PUSHED, RISES, ...powerMoves(() => menu(1536, 864, { rows: 2 }))],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "Ctrl+N", "Ctrl+P"], does: "The menu: the selection up or down, the field keeping the keyboard. Kōbō: the card above or below (also k and j), its page rising in." },
      { keys: ["Page Up", "Page Down"], does: "The menu: a page of Results at a time." },
      { keys: ["Enter"], does: "The menu: runs the selected Result's first Action. Kōbō: opens or presses what has the keyboard." },
      { keys: ["Ctrl+K"], does: "The menu: the selected Result's Action panel, over the bottom right of the list." },
      { keys: ["Tab", "Shift+Tab"], does: "The menu: Tab enters an app's scope on its row. Kōbō: from the list through the page's controls and its actions." },
      { keys: ["Esc"], does: "The menu: clears what is typed, then goes back a View, then closes. Kōbō: back to the cards." },
    ],
    order: "The menu is one stop, its field, which hands the list its keys. Kōbō: the list, the page's controls, its actions, then Done for now.",
    lands: "The menu opens with the keyboard in its field and the first Result selected. Kōbō starts on its list, the first card that is still to do selected.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The Command menu", for: "Its `space.s12` padding and gaps, the field, the body beside the pane, the rule and the footer.", sample: () => menu(1536, 864, { rows: 3 }) },
      { name: "Kōbō's setup", for: "The side's page and the card's page.", sample: () => framed(setup(), "var(--size-app_min_width)", "var(--size-app_min_height)") },
    ]),
    measures: [
      { what: "The menu's padding and the gaps between its parts; the room between the list and the pane; Kōbō's gaps round the online chip.", token: "space-s12" },
      { what: "One column: the menu's least width, and the most of Kōbō's side; two of them, the menu's widest.", token: "size-column" },
      { what: "With `size.rail`, the narrowest panel that shows the pane.", token: "size-dialog" },
      { what: "The pane's least width.", token: "size-rail" },
      { what: "A Result's row; with `space.s4`, the pitch the menu counts its rows by.", token: "size-row_two_line" },
      { what: "The field.", token: "size-control_md" },
      { what: "The menu's footer.", token: "size-control_sm" },
      { what: "The menu's corners.", token: "radius-surface" },
      { what: "Kōbō: between the header and the list.", token: "space-s20" },
      { what: "Kōbō: between the card's head, its words and its actions.", token: "space-s24" },
    ],
    reflow: {
      described:
        "Kōbō's setup in a window of the toolbar's size: the side is a third of it (240 at the 720 minimum, 373 at 1120) up to `size.column` (560 from a 1680 window), the page takes the rest; the cards scroll in the height their list is left. The menu on a screen of the same size is under Sizes.",
      sample: () => setup(),
    },
  },
});

