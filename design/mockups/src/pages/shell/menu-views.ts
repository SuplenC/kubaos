// The Views an Extension pushes onto the Command menu (`Menu.qml`, `DetailView.qml`): a list View's
// Items under its filter, a detail View's Markdown and metadata, and a progress View following a job
// to its end. Each has Back before its head; Esc pops it. The form View has a page of its own.

import type { Words } from "../../lib/content.ts";
import type { KeyHint } from "../../lib/keycap.ts";
import { stack } from "../../lib/layout.ts";
import type { Job } from "../../lib/progress-panel.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { type Move, nonEmpty, page } from "../../site/page.ts";
import { DETAIL, type Detail, detailView, filterHead, LAYOUTS, panel, progressView, results, titleHead } from "./menu-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "loading" | "empty" | "error";

const BACK: readonly KeyHint[] = [["Esc", "Back"]];
const LIST_KEYS: readonly KeyHint[] = [
  ["↑↓", "Select"],
  ["⌃K", "Actions"],
  ["Esc", "Back"],
];

type Screen = "full" | "laptop" | "narrow";

const listView = (extra: { screen?: Screen; typed?: string; rows?: number; current?: number; back?: "lit" | "pressed"; items?: typeof LAYOUTS } = {}) =>
  panel({
    screen: extra.screen ?? "full",
    head: filterHead("Pick a layout", extra.typed ?? "", extra.back),
    query: "",
    results: results(extra.typed ?? "", extra.items ?? LAYOUTS),
    hints: extra.typed ? [["↑↓", "Select"], ["⌃K", "Actions"], ["Esc", "Clear"]] : LIST_KEYS,
    ...(extra.rows ? { rows: extra.rows } : {}),
    ...(extra.current !== undefined ? { current: extra.current } : {}),
  });

const detail = (extra: { screen?: Screen; rows?: number; content?: Detail; title?: string; down?: number } = {}) =>
  panel({ screen: extra.screen ?? "full", head: titleHead(extra.title ?? "Firefox"), body: detailView(extra.content ?? DETAIL, extra.down), hints: BACK, ...(extra.rows ? { rows: extra.rows } : {}) });

const JOB: Job = { title: "Update", state: "running", progress: 0.42, message: "Fetching 12 of 28" };

const progress = (job: Job = JOB, extra: { screen?: Screen; rows?: number } = {}) =>
  panel({ screen: extra.screen ?? "full", head: titleHead(job.title), body: progressView(job), hints: BACK, ...(extra.rows ? { rows: extra.rows } : {}) });

/** The stage: Enter on a Result pushes the list View; Back or Esc pops it. */
function live(): HTMLElement {
  const room = document.createElement("span");
  let pushed = true;
  const draw = () => {
    const shown = pushed ? listView() : panel();
    room.replaceChildren(shown);
    shown.querySelector(".k-button")?.addEventListener("click", () => ((pushed = false), draw()));
    shown.addEventListener("keydown", (event) => {
      if ((pushed && event.key === "Escape") || (!pushed && event.key === "Enter")) {
        pushed = !pushed;
        draw();
      }
    });
  };
  draw();
  return room;
}

const LONG: Detail = {
  markdown: ["# Document Viewer", ...Array.from({ length: 8 }, () => "View, search or annotate documents in many different formats. It opens PDF, PostScript, DjVu, TIFF and XPS, and remembers where you were in each.")],
  metadata: DETAIL.metadata,
};

const TITLE: Words = { short: "Firefox", long: "Firefox Developer Edition, the browser for people who build the web" };

const few = (screen: Screen = "narrow") => ({ screen, rows: 4 });

const PUSHED: Move = {
  name: "A View pushed",
  what: "Enter on an Item that opens a View: the body slides `space.s24` in from the right and settles.",
  tracks: [
    { part: "slide", on: "the body", target: ".k-menu-body", property: "--view-push", from: 1, to: 0, duration: "medium", curve: "emphasized_decel" },
    { part: "settle", on: "the body", target: ".k-menu-body", property: "opacity", from: 0.4, to: 1, duration: "medium", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "settle", on: "the body", target: ".k-menu-body", property: "opacity", from: 0.4, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => listView(few()),
};

const POPPED: Move = {
  name: "A View popped",
  what: "Back or Esc: the View under it slides `space.s24` in from the left and settles.",
  tracks: [
    { part: "slide", on: "the body", target: ".k-menu-body", property: "--view-push", from: -1, to: 0, duration: "medium", curve: "emphasized_decel" },
    { part: "settle", on: "the body", target: ".k-menu-body", property: "opacity", from: 0.4, to: 1, duration: "medium", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "settle", on: "the body", target: ".k-menu-body", property: "opacity", from: 0.4, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => panel(few()),
};

export default page<Shown>({
  level: "shell",
  slug: "menu-views",
  title: "Command menu Views",
  lede: "What an Extension pushes onto the Command menu: a list under its filter, a detail, a job's progress.",
  code: { qml: "DetailView" },
  from: nonEmpty(["list-detail", "icon-button", "field", "description-list", "scroll", "progress-panel"]),
  stage: { live, hint: "Esc or Back pops the View; Enter pushes it again", daylight: true },
  variants: [
    { name: "List", for: "an Extension's Items, its filter titled", sample: () => listView() },
    { name: "List, filtered", for: "what's typed narrows the Items", sample: () => listView({ typed: "ger" }) },
    { name: "Detail", for: "Markdown, then the metadata", sample: () => detail() },
    { name: "Progress", for: "a job followed to its end", sample: () => progress() },
  ],
  sizes: [
    { name: "Full HD", for: "a list View keeps the pane", sample: () => listView() },
    { name: "1280 × 800", for: "no pane; the footer has the row's keys", sample: () => listView({ screen: "narrow" }) },
    { name: "Detail, 1280 × 800", for: "the body is the View's", sample: () => detail({ screen: "narrow" }) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "loading", "empty", "error"],
    absent: {
      disabled: "a View is never off; its Items may be",
    },
    rows: [
      {
        name: "List",
        cell: (state) => {
          if (state === "lit" || state === "pressed") return listView({ ...few(), back: state });
          if (state === "selected") return listView({ ...few(), current: 2 });
          if (state === "empty") return listView({ ...few(), items: [] });
          if (state === "loading") return progress({ title: "Find layouts", state: "running", progress: -1 }, few());
          if (state === "error") return progress({ title: "Update", state: "failed", progress: -1, message: "No network" }, few());
          return listView(few());
        },
      },
      {
        name: "Progress",
        cell: (state) => {
          if (state === "loading") return progress({ title: "Update", state: "running", progress: -1, message: "Starting" }, few());
          if (state === "error") return progress({ title: "Update", state: "failed", progress: -1, message: "No network" }, few());
          if (state === "selected" || state === "empty") return progress({ title: "Update", state: "succeeded", progress: 1, message: "28 packages" }, few());
          return progress(JOB, few());
        },
      },
    ],
    looks: {
      rest: "Back, then the filter or the title",
      lit: "Back's text tube",
      pressed: "Back's fill presses",
      selected: "a list's row on `secondary_container`; a job done",
      loading: "a job that can't tell: the spinner",
      empty: "a list with none: Nothing here yet",
      error: "a job failed: its symbol and line in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "The filter", tone: "accent", filled: false, content: false },
      { name: "Back", tone: "accent", filled: false, content: true },
    ],
    looks: { lit: "the filter's tube; Back's symbol burns", signsOff: "each line `primary_glass`, no glow" },
    signsOff: () => listView({ ...few(), back: "lit" }),
  },
  content: {
    text: [
      { name: "A View's title", words: TITLE, sample: (words: string) => detail({ title: words }) },
      { name: "A list View's title", words: { short: "Layouts", long: "Pick a keyboard layout to add to the ones you switch between" }, sample: (words: string) => panel({ head: filterHead(words), query: "", results: results("", LAYOUTS), hints: LIST_KEYS }) },
    ],
    missingPicture: [{ name: "An Item without an icon", for: "its Extension's symbol", sample: () => listView(few()) }],
    counts: [
      { name: "No Items", for: "Nothing here yet", sample: () => listView({ items: [] }) },
      { name: "Detail, short", for: "no scroll", sample: () => detail() },
      { name: "Detail, long", for: "it scrolls under the head", sample: () => detail({ content: LONG, down: 0.4 }) },
    ],
  },
  motion: {
    arrives: [PUSHED],
    leaves: [POPPED],
    between: [{ name: "Back lights", what: "The pointer onto Back: its tube fades in.", tracks: lights(".k-button", "Back", 0, 1), reduced: "same", back: "plays", sample: () => stack(0, "start", listView(few())) }, ...powerMoves(() => listView({ ...few(), back: "lit" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Esc"], does: "clears the filter, else pops the View" },
      { keys: ["↑", "↓", "Enter", "Ctrl+K"], does: "a list View: as in the search" },
      { keys: ["Page Up", "Page Down"], does: "a list View: a screen of rows" },
    ],
    order: "No Tab stops: Back takes only the pointer.",
    lands: "A list View in its filter, a detail or progress View in the menu; popped, where it was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "List", for: "Back, the filter, the rows", sample: () => listView() },
      { name: "Detail", for: "the words, the metadata", sample: () => detail() },
      { name: "Progress", for: "the panel centred in the body", sample: () => progress() },
    ]),
    measures: [
      { part: "Back to the filter", token: "space-s8" },
      { part: "Back", token: "size-control_md" },
      { part: "Title", token: "type-subheading-line" },
      { part: "Markdown to metadata", token: "space-s24" },
      { part: "Progress column's most", token: "size-column" },
      { part: "Progress parts apart", token: "space-s16" },
      { part: "Slide's travel", token: "space-s24" },
    ],
    reflow: { note: "The body is the panel's: the screen sets it, as in the search.", sample: () => detail({ screen: "laptop" }) },
  },
});
