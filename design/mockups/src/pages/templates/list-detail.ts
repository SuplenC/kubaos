// The ListDetail page: `docs/design/components/list-detail.md`. A list beside the detail of its
// selected row, as its two builds draw it today: the Command menu's Results beside its pane, and
// Kōbō's first-boot cards beside the picked card's page. Its new design is sketched later.

import "./list-detail-page.css";
import { banner } from "../../lib/banner.ts";
import { type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { commandMenu, menuMeasures, menuOnScreen, type Result, type SetupCard, setupWindow } from "../../lib/list-detail.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
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
  what: "Super+Space: the panel fades in and drops into place, its Results there from the first frame.",
  tracks: [
    { part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "drop", on: "the panel", target: ".k-command", property: "--menu-drop", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => stack(0, "start", menu(1280, 800, { rows: 3 })),
};

const CLOSES: Move = {
  name: "The Command menu closes",
  what: "Esc, or an Item run: the panel fades and lifts; what it runs waits until it is gone.",
  tracks: [
    { part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" },
    { part: "drop", on: "the panel", target: ".k-command", property: "--menu-drop", from: 0, to: 1, duration: "micro", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => stack(0, "start", menu(1280, 800, { rows: 3 })),
};

const FOLLOWS: Move = {
  name: "The pane follows the selection",
  what: "↓ slides the selection; the pane crossfades once typing has paused.",
  tracks: [{ part: "the row reached", on: "the pane's content", target: ".k-menu-pane-content", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: () => stack(0, "start", menu(1536, 864, { rows: 4, current: 1 })),
};

const PUSHED: Move = {
  name: "A View pushed",
  what: "An Item opens a View: the body slides in from the right; Back, from the left.",
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
  what: "A card picked: its page rises and fades in; the list stays.",
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
  lede: "A list beside the detail of its selected row, as the Command menu and Kōbō's setup draw it today.",
  code: { unbuilt: "not built yet" },
  stage: {
    live: () => menu(1536, 864),
    hint: "click a row, then ↑ ↓; the pane follows",
    daylight: true,
  },
  variants: [
    { name: "The Command menu", for: "a key summons it; drawn for a 1536 × 864 screen", sample: () => menu(1536, 864) },
    { name: "Kōbō's setup", for: "an app's window: what's left to do, and how", sample: () => framed(setup(), "var(--size-app_width)", "var(--size-app_height)") },
  ],
  sizes: [
    { name: "The menu without its pane", for: "a 1280 × 800 screen: the footer shows the row's keys", sample: () => menu(1280, 800) },
    { name: "The menu at the toolbar's size", for: "width, rows and place follow the screen", sample: () => framed(menuOnScreen({ results: results(), query: "fi" }), windowSize().width, windowSize().height) },
  ],
  states: {
    shown: ["rest", "selected", "empty", "error", "loading"],
    absent: {
      lit: "its list, rows and cards light on their pages",
      pressed: "what it holds is pressed, never it",
      disabled: "an unavailable Result is dimmed by its list",
    },
    rows: [
      {
        name: "The Command menu",
        cell: (state) => {
          if (state === "empty") return commandMenu({ results: [], query: "fiq", rows: 2 }, menuMeasures(1536, 864));
          if (state === "error") return text("Why the Action failed, across the footer.", "small", "error");
          if (state === "loading") return text("Not drawn: Results come with the first frame.", "small", "on_surface_variant");
          return menu(1536, 864, { rows: 2, current: state === "selected" ? 1 : 0 });
        },
      },
      {
        name: "Kōbō's setup",
        cell: (state) => {
          if (state === "loading") return text("A spinner in the list's place.", "small", "on_surface_variant");
          if (state === "empty") return text("Doesn't occur: every machine has its cards.", "small", "on_surface_variant");
          if (state === "error") return framed(setup({ online: false }), "var(--size-app_min_width)", "var(--size-app_min_height)");
          return framed(setup({ selected: state === "selected" ? 2 : 0 }), "var(--size-app_min_width)", "var(--size-app_min_height)");
        },
      },
    ],
    looks: {
      rest: "the first row selected, its detail beside it",
      selected: "the selection slides, the pane follows; Kōbō's card turns `secondary_container`",
      empty: "an `EmptyState` in the list; the pane's Nothing selected",
      error: "the menu's reason in `error`; Kōbō's offline chip",
      loading: "the menu never waits; Kōbō shows a spinner",
    },
  },
  focus: doesntApply("no line of its own: the menu's field and Kōbō's list light theirs."),
  content: {
    text: [{ name: "A row's title", words: TITLE, sample: (words: string) => menu(1536, 864, { rows: 2, current: 1, title: words }) }],
    missingPicture: doesntApply("a Result with no icon shows its Extension's symbol on its tile."),
    counts: [
      { name: "None", for: "the empty state in the list, Nothing selected", sample: () => commandMenu({ results: [], query: "fiq", rows: 4 }, menuMeasures(1536, 864)) },
      { name: "One", for: "one Result, the pane on it", sample: () => commandMenu({ results: results().slice(0, 1), query: "firef", rows: 4 }, menuMeasures(1536, 864)) },
      { name: "Many", for: "the list scrolls, the selection stays in view", sample: () => menu(1536, 864, { rows: 4 }) },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: [FOLLOWS, PUSHED, RISES, ...powerMoves(() => menu(1536, 864, { rows: 2 }))],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "Ctrl+N", "Ctrl+P"], does: "menu: moves the selection; Kōbō: the card, also k j" },
      { keys: ["Page Up", "Page Down"], does: "menu: a page of Results" },
      { keys: ["Enter"], does: "menu: the first Action; Kōbō: presses what has the keyboard" },
      { keys: ["Ctrl+K"], does: "menu: the Result's Action panel" },
      { keys: ["Tab", "Shift+Tab"], does: "menu: an app's scope; Kōbō: through the page's controls" },
      { keys: ["Esc"], does: "menu: clears, back a View, closes; Kōbō: back to the cards" },
    ],
    order: "The menu is one stop, its field; Kōbō goes list, page, actions, Done for now.",
    lands: "The menu, in its field on the first Result; Kōbō, on the first card still to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The Command menu", for: "padding and gaps, field, body, pane, footer", sample: () => menu(1536, 864, { rows: 3 }) },
      { name: "Kōbō's setup", for: "the side and the card's page", sample: () => framed(setup(), "var(--size-app_min_width)", "var(--size-app_min_height)") },
    ]),
    measures: [
      { part: "Menu's padding, gaps; Kōbō's chip", token: "space-s12" },
      { part: "Menu's least; Kōbō's side's most", token: "size-column" },
      { part: "With a rail, shows the pane", token: "size-dialog" },
      { part: "Pane's least width", token: "size-rail" },
      { part: "A Result's row", token: "size-row_two_line" },
      { part: "Field", token: "size-control_md" },
      { part: "Menu's footer", token: "size-control_sm" },
      { part: "Menu's corners", token: "radius-surface" },
      { part: "Kōbō: header → list", token: "space-s20" },
      { part: "Kōbō: head, words, actions", token: "space-s24" },
    ],
    reflow: {
      note: "Kōbō's side is a third of the window up to `size.column`; the page takes the rest.",
      sample: () => setup(),
    },
  },
});
