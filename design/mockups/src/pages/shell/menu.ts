// The Command menu's search (`Menu.qml`, `ResultRow.qml`, `DetailPane.qml`, `PaneContent.qml`,
// `IntroCard.qml`): the field over the Results, the pane beside them where the screen has room, the
// jobs under the field, an app's intro card, and the keys in the footer. Super+Space opens it.

import type { Words } from "../../lib/content.ts";
import { field } from "../../lib/field.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { menuOnScreen } from "../../lib/list-detail.ts";
import { powerMoves } from "../../site/moves.ts";
import { type Move, nonEmpty, page } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { footerWords, introCard, ITEMS, type Item, jobs, JOBS, LAYOUT_INTRO, LAYOUTS, panel, results, searchHead } from "./menu-parts.ts";

type Shown = "rest" | "selected" | "disabled" | "loading" | "empty" | "error";

const PLACEHOLDER = "Search apps, projects and windows. Sums work too";

/** The stage: typing filters the Results, ↑ ↓ move the selection and the pane follows. */
function live(): HTMLElement {
  const room = document.createElement("span");
  let query = "fi";
  const search = field({
    placeholder: PLACEHOLDER,
    icon: "search",
    value: query,
    onInput: (value) => {
      query = value;
      draw();
      search.focus();
    },
  });
  const draw = () => {
    const head = row("s8", "centre", search);
    head.classList.add("m-head");
    search.style.flex = "1 1 0";
    room.replaceChildren(panel({ query, results: results(query), head }));
  };
  draw();
  return room;
}

/** The menu in an app, its intro card over the app's Items before anything is typed. */
const inApp = (screen: "full" | "narrow" = "full") => panel({ screen, query: "", results: results("", LAYOUTS), head: searchHead("", "layout"), above: introCard(LAYOUT_INTRO), rows: 6, hints: [["↑↓", "Select"], ["⌃K", "Actions"], ["Esc", "Leave app"]] });

const small = (extra: Parameters<typeof panel>[0] = {}) => panel({ screen: "narrow", rows: 3, ...extra });

const TITLE: Words = { short: "Files", long: "Files, the file manager that opens in the folder you were last in" };

/** The Items with the first's title `title`. */
const titled = (title: string): Item[] => [{ ...ITEMS[0]!, title }, ...ITEMS.slice(1)];

const OPENS: Move = {
  name: "Opens",
  what: "Super+Space: the panel fades in and drops `space.s8` into place, its Results there in the first frame.",
  tracks: [
    { part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: "drop", on: "the panel", target: ".k-command", property: "--menu-drop", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => stack(0, "start", small()),
};

const CLOSES: Move = {
  name: "Closes",
  what: "Esc, or an Item run: the panel fades and lifts; what it runs waits until it is gone.",
  tracks: [
    { part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" },
    { part: "drop", on: "the panel", target: ".k-command", property: "--menu-drop", from: 0, to: 1, duration: "micro", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "fade", on: "the panel", target: ".k-command", property: "opacity", from: 1, to: 0, duration: "micro", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => stack(0, "start", small()),
};

const SLIDES: Move = {
  name: "The selection slides",
  what: "↓ between rows: the highlight slides over `short`; while typing, and with reduced motion, it jumps.",
  tracks: [
    { part: "the selection", on: "the Results", target: ".k-list-highlight", property: "--along", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the row left", on: "the Results", target: ".k-list-row:nth-child(2)", property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the row reached", on: "the Results", target: ".k-list-row:nth-child(3)", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [
    { part: "the selection", on: "the Results", target: ".k-list-highlight", property: "--along", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the row left", on: "the Results", target: ".k-list-row:nth-child(2)", property: "--lit", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the row reached", on: "the Results", target: ".k-list-row:nth-child(3)", property: "--lit", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "plays",
  sample: () => {
    const box = small({ query: "", results: results("") });
    requestAnimationFrame(() => box.querySelector<HTMLElement>(".k-list-highlight")?.setAttribute("data-stepped", ""));
    return box;
  },
};

const FOLLOWS: Move = {
  name: "The pane follows",
  what: "The selection moved: the pane crossfades once typing has paused; Results a key brought swap at once.",
  tracks: [{ part: "the row reached", on: "the pane's content", target: ".k-menu-pane-content", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: () => panel({ screen: "laptop", current: 1 }),
};

const ROWS: Move = {
  name: "Results change",
  what: "Late Results: a row fades in, one gone fades out, the rest glide; none of it while typing.",
  tracks: [{ part: "a row arriving", on: "the Results", target: ".k-list-row:nth-child(3)", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: () => small({ query: "", results: results("") }),
};

export default page<Shown>({
  level: "shell",
  slug: "menu",
  title: "Command menu",
  lede: "Super+Space: a search over every Item, its Results beside the selected one's pane, and the keys it takes.",
  code: { qml: "Menu" },
  from: nonEmpty(["list-detail", "field", "list", "card", "keycap", "empty-state", "chip", "progress-bar"]),
  stage: { live, hint: "type to filter; ↑ ↓ move, the pane follows", daylight: true },
  variants: [
    { name: "Typed", for: "the Results for fi, matches lit", sample: () => panel() },
    { name: "Nothing typed", for: "every Item, ranked for the time of day", sample: () => panel({ query: "", results: results("") }) },
    { name: "In an app", for: "its chip, its intro card, its Items", sample: () => inApp() },
    { name: "Jobs running", for: "each job under the field, its bar", sample: () => panel({ under: jobs(JOBS) }) },
    { name: "An unavailable Result", for: "dimmed, why in its subtitle's place", sample: () => panel({ query: "su" }) },
  ],
  sizes: [
    { name: "Full HD", for: "half the screen, 9 rows, the pane", sample: () => panel({ screen: "full" }) },
    { name: "1080p at 1.25", for: "a dialog and a rail wide: the pane", sample: () => panel({ screen: "laptop" }) },
    { name: "1280 × 800", for: "no pane: the footer shows the row's keys", sample: () => panel({ screen: "narrow" }) },
    { name: "At the toolbar's window", for: "width, rows and place follow the screen", sample: () => within(windowSize().width, menuOnScreen({ results: results("fi"), query: "fi" })) },
  ],
  states: {
    shown: ["rest", "selected", "disabled", "loading", "empty", "error"],
    absent: {
      lit: "the selection is the lit row; the field holds the keyboard",
      pressed: "a Result runs on press, and the menu closes",
    },
    rows: [
      {
        name: "The menu",
        cell: (state) => {
          if (state === "selected") return small({ current: 1, query: "", results: results("") });
          if (state === "disabled") return small({ query: "su" });
          if (state === "loading") return small({ under: jobs(JOBS.slice(1)) });
          if (state === "empty") return small({ query: "fiq", results: [] });
          if (state === "error") return small({ footer: footerWords("Couldn't open Files: no such file or directory", true) });
          return small();
        },
      },
    ],
    looks: {
      rest: "the first row selected; the field's tube lit",
      selected: "`secondary_container` slides behind the row",
      disabled: "the row at `opacity.disabled_content`, why in the subtitle",
      loading: "never waits: a job spins under the field",
      empty: "`EmptyState`: No results, or Nothing here yet",
      error: "why the Action failed, across the footer in `error`",
    },
  },
  focus: {
    tubes: [{ name: "The search field", tone: "accent", filled: false, content: false }],
    looks: { lit: "the field's tube burns, the caret in it", signsOff: "the tube as `primary_glass`, no glow" },
    signsOff: () => small({ head: searchHead("fi", "", "signs-off") }),
  },
  content: {
    text: [{ name: "A Result's title", words: TITLE, sample: (words: string) => panel({ screen: "laptop", results: results("", titled(words)), query: "" }) }],
    missingPicture: [
      { name: "No icon of its own", for: "its Extension's symbol on the tile", sample: () => panel({ query: "", results: results("", ITEMS.slice(4, 9)) }) },
      { name: "No Extension symbol", for: "a bolt", sample: () => panel({ query: "", results: results("", [{ title: "Rebuild the index", subtitle: "Kotodama", extension: "kotodama" }]) }) },
    ],
    counts: [
      { name: "None", for: "the empty state; the pane's Nothing selected", sample: () => panel({ query: "fiq", results: [] }) },
      { name: "One", for: "the pane on it", sample: () => panel({ query: "firef" }) },
      { name: "Many", for: "the list scrolls; the selection stays in view", sample: () => panel({ query: "", results: results(""), current: 10 }) },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CLOSES],
    between: [SLIDES, FOLLOWS, ROWS, ...powerMoves(() => small())],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓", "Ctrl+P", "Ctrl+N"], does: "moves the selection" },
      { keys: ["Page Up", "Page Down"], does: "a screen of rows" },
      { keys: ["Enter"], does: "runs the Result's first Action" },
      { keys: ["Ctrl+K"], does: "the Action panel" },
      { keys: ["Tab"], does: "on an app's Item: enters the app" },
      { keys: ["Backspace"], does: "in an app, nothing typed: leaves it" },
      { keys: ["Esc"], does: "clears, else back a View, else leaves the app, else closes" },
      { keys: ["Ctrl+Z"], does: "while a Toast shows: undoes it" },
    ],
    order: "One stop: the field keeps the keyboard and hands the list its keys.",
    lands: "In the field, on the first Result; the menu opens where it was left.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Full HD", for: "padding, gaps, field, rows, pane, footer", sample: () => panel() },
      { name: "In an app", for: "the chip, the intro card", sample: () => inApp() },
      { name: "Jobs", for: "the jobs' rows and bars", sample: () => small({ under: jobs(JOBS) }) },
    ]),
    measures: [
      { part: "Padding, between parts", token: "space-s12" },
      { part: "Least width", token: "size-column" },
      { part: "With a rail, shows the pane", token: "size-dialog" },
      { part: "Pane's least width", token: "size-rail" },
      { part: "A Result's row", token: "size-row_two_line" },
      { part: "Between rows", token: "space-s4" },
      { part: "Field, head's row", token: "size-control_md" },
      { part: "Head's parts apart", token: "space-s8" },
      { part: "Footer", token: "size-control_sm" },
      { part: "Corners", token: "radius-surface" },
      { part: "Pane: symbol", token: "icon-s48" },
      { part: "Pane: symbol to title", token: "space-s16" },
      { part: "Pane: words to Actions", token: "space-s24" },
      { part: "Intro card's lines", token: "space-s8" },
    ],
    reflow: {
      note: "Half the screen, one column to two; 8 to 14 rows for two thirds of its height; the pane from 736 wide.",
      sample: () => within(windowSize().width, menuOnScreen({ results: results("fi"), query: "fi" })),
    },
  },
});
