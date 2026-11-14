// The SidebarPage page: `docs/design/components/sidebar-page.md`. An app's window as its sections
// in a sidebar beside a page that scrolls, with Home, a page that is only search, at the top:
// Settings' layout. Live in a window that resizes, from the app's minimum to a 1080p screen.

import type { Shown } from "../../lib/control.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { stack } from "../../lib/layout.ts";
import { type SidebarPageOptions, sidebarPage, sidebarRow } from "../../lib/sidebar-page.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, nonEmpty, type Track, page } from "../../site/page.ts";
import { fewer, outline } from "./settings.ts";
import { resizable, still } from "./window.ts";

/** The template drawn from Settings' outline, `options` saying where it stands. */
function settings(options: Omit<SidebarPageOptions, "outline"> = {}, sections = outline()): HTMLElement {
  return sidebarPage({ outline: sections, ...options });
}

/** The template at the app's minimum, 720 × 480, standing as `options` say: a state's sample. */
function small(options: Omit<SidebarPageOptions, "outline"> = {}, sections = outline()): HTMLElement {
  return still(settings(options, sections));
}

/** A sidebar row on the sidebar's band, in a state. */
function rowIn(kind: "section" | "sub", words: string, glyph: string | null, current: boolean, state: Shown): HTMLElement {
  const band = document.createElement("span");
  band.className = "k-sp-side t-band";
  const button = sidebarRow(kind, words, glyph, current, state === "rest" || state === "disabled" ? undefined : state);
  band.append(button);
  return band;
}

/**
 * The stage's window: it opens at the app's minimum so the whole of it reads inside a stage's pane,
 * and the stack lets a pane too narrow for it scale it instead of clipping it.
 */
function live(): HTMLElement {
  const room = resizable(settings({ focus: true }));
  const frame = room.querySelector<HTMLElement>(".t-window")!;
  frame.style.width = "var(--size-app_min_width)";
  frame.style.height = "var(--size-app_min_height)";
  return stack(0, "start", room);
}

const SECTION: Words = { short: "Power", long: "Power, battery and sleep" };
const SUB: Words = { short: "Battery", long: "Battery and charging" };

const ROWS = [
  { name: "A section", kind: "section", glyph: "bolt", words: SECTION },
  { name: "A sub-section", kind: "sub", glyph: null, words: SUB },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "selected";

/** The page's change of section: the page fading in and rising `space.s8`. */
function rising(reduced: boolean): [Track<Easing>, ...Track<Easing>[]] {
  return reduced
    ? [{ part: "the page", on: "the section's page", target: ".k-sp-rise", property: "--section-rise", from: 0, to: 1, duration: "instant", curve: "linear" }]
    : [{ part: "the page", on: "the section's page", target: ".k-sp-rise", property: "--section-rise", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }];
}

/** The find box arriving or going: `--find-shown` from `from` to `to`. */
function showing(from: 0 | 1, to: 0 | 1): [Track<Easing>, ...Track<Easing>[]] {
  return [{ part: "the find box", on: "the find box", target: ".k-sp-find", property: "--find-shown", from, to, duration: "short", curve: "standard" }];
}

/** A sample whose bar stands at the first sub-section, to glide to the third. */
function barSample(): HTMLElement {
  const window = small({ section: 3 });
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      const bar = window.querySelector<HTMLElement>(".k-sp-bar");
      const subs = [...window.querySelectorAll<HTMLElement>(".k-sp-row[data-kind='sub']")];
      if (!bar || subs.length < 3) return;
      bar.dataset.still = "";
      bar.style.setProperty("--bar-from", `${subs[0]!.offsetTop}px`);
      bar.style.setProperty("--bar-to", `${subs[2]!.offsetTop}px`);
    }),
  );
  return window;
}

const MOVES: Move[] = [
  {
    name: "Changing section",
    what: "A section picked: the new page fades in and rises, the bar already on its first sub-section.",
    tracks: rising(false),
    reduced: rising(true),
    back: "cuts",
    sample: () => small({ section: 3 }),
  },
  {
    name: "The bar glides",
    what: "The current sub-section changes: the bar glides to it; a press glides the page too.",
    tracks: [{ part: "the bar", on: "the current sub-section", target: ".k-sp-bar", property: "--bar-at", from: 0, to: 1, duration: "medium", curve: "standard" }],
    reduced: [{ part: "the bar", on: "the current sub-section", target: ".k-sp-bar", property: "--bar-at", from: 0, to: 1, duration: "instant", curve: "linear" }],
    back: "cuts",
    sample: barSample,
  },
  {
    name: "The find box arrives",
    what: "`/` or Ctrl+F: the box fades in and drops into place, taking no layout.",
    tracks: showing(0, 1),
    reduced: "same",
    back: "cuts",
    sample: () => small({ section: 3, find: "" }),
  },
  {
    name: "The find box goes",
    what: "Esc: it fades out and rises; the page is back where it was at once.",
    tracks: showing(1, 0),
    reduced: "same",
    back: "cuts",
    sample: () => small({ section: 3, find: "" }),
  },
  {
    name: "Landed",
    what: "A setting opened from Home: its row is lit by the highlight, which fades.",
    tracks: [{ part: "the landed highlight", on: "the row landed on", target: ".k-list-row[data-landed]", property: "--landed", from: 1, to: 0, duration: "slow", curve: "standard" }],
    reduced: "same",
    back: "cuts",
    sample: () => small({ section: 3, landed: 2 }),
  },
  {
    name: "A sidebar row lights up",
    what: "Pointer or keyboard on a row: the tube and glow fade in, as on a Text button.",
    tracks: lights(".k-sp-row", "a section's row", 0, 1),
    reduced: "same",
    back: "plays",
    sample: () => rowIn("section", shown(SECTION), "bolt", false, "rest"),
  },
  {
    name: "A sidebar row pressed",
    what: "A lit row pressed: its tube runs white-hot, and back as it is let go.",
    tracks: press(".k-sp-row", "a section's row"),
    reduced: "same",
    back: "plays",
    sample: () => rowIn("section", shown(SECTION), "bolt", false, "lit"),
  },
  ...powerMoves(() => stack("s4", "start", rowIn("section", shown(SECTION), "bolt", true, "lit"), rowIn("sub", shown(SUB), null, true, "lit"))),
];

export default page({
  level: "templates",
  slug: "sidebar-page",
  title: "SidebarPage",
  lede: "An app's window as its sections in a sidebar beside a page that scrolls, with Home, only search, on top.",
  code: { unbuilt: "Rust template::sidebar_page: not built yet" },
  stage: {
    live,
    hint: "type a setting, then Enter; click a section; / finds",
    daylight: true,
  },
  variants: [
    { name: "Home", for: "where it opens: search over the sections", sample: () => small() },
    { name: "A section", for: "scrolled to its second sub-section, the bar beside it", sample: () => small({ section: 3, sub: 1 }) },
    { name: "Find on a page", for: "the find box, on its second match of three", sample: () => small({ section: 3, find: "e", findAt: 1, keyboard: "find" }) },
    { name: "Home's results", for: "matches with their letters lit, path and value", sample: () => small({ query: "lo", keyboard: "home" }) },
  ],
  sizes: doesntApply("it fills its window, from the app's minimum to a full screen."),
  states: {
    shown: ["rest", "lit", "pressed", "selected"],
    absent: {
      disabled: "a section is never off; a row disables itself",
      loading: "the outline is there as the window opens",
      empty: "every section has a sub-section",
      error: "said by the page: an `ErrorBlock` or a `Banner`",
    },
    rows: nonEmpty(
      ROWS.map(({ name, kind, glyph, words }) => ({
        name,
        cell: (state: MatrixState) => rowIn(kind, shown(words), glyph, state === "selected", state === "selected" ? "rest" : state),
      })),
    ),
    looks: {
      rest: "`on_surface_variant`, no fill, no line",
      lit: "the tube inside its edge, glow at night, words `on_surface`",
      pressed: "the tube runs hot over `micro`; by day the glass darkens",
      selected: "filled `surface_container_high`, symbol `primary`; a sub-section, the bar",
    },
  },
  focus: {
    tubes: [
      { name: "A sidebar row, lit", tone: "accent", filled: false, content: false },
      { name: "A page's well, with the keyboard", tone: "accent", filled: false, content: false },
      { name: "Home's and the find box's fields", tone: "accent", filled: false, content: false },
      { name: "The bar, always", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "a row's tube inside, a well's round it; the bar's core",
      signsOff: "each tube as `primary_glass`, no glow; the bar the tone's glass",
    },
    signsOff: (index: number) => rowIn(ROWS[index]!.kind, shown(ROWS[index]!.words), ROWS[index]!.glyph, false, "signs-off"),
  },
  content: {
    text: [
      { name: "A section's name in the sidebar", words: SECTION, sample: (words: string) => rowIn("section", words, "bolt", true, "rest") },
      { name: "A sub-section's name", words: SUB, sample: (words: string) => rowIn("sub", words, null, false, "rest") },
      { name: "The whole window", words: fixed("Settings"), sample: () => small({ section: 0 }) },
    ],
    missingPicture: doesntApply("every section has its symbol; it draws no picture."),
    counts: [
      { name: "Nothing found from Home", for: "a line under the field says so", sample: () => small({ query: "xyzzy", keyboard: "home" }) },
      { name: "Nothing found on a page", for: "the rows stand back; Enter searches from Home", sample: () => small({ section: 3, find: "xyzzy", keyboard: "find" }) },
      { name: "Few sections", for: "the band runs on to the window's foot", sample: () => small({ section: 0 }, fewer(3)) },
      { name: "Many sections", for: "the sidebar scrolls on its own, its thumb in its padding", sample: () => small({ section: 9 }) },
    ],
  },
  motion: { arrives: doesntApply("it comes with its window, opening on Home with nothing moving."), leaves: doesntApply("it goes with its window."), between: MOVES },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "round the stops: field or find box, sidebar, page" },
      { keys: ["↑", "↓", "j", "k"], does: "through the sidebar's or the page's rows, round" },
      { keys: ["Ctrl+N", "Ctrl+P", "Ctrl+J", "Ctrl+K"], does: "the same; in Home's field, through the results" },
      { keys: ["Home", "End", "Page Up", "Page Down"], does: "first, last or a page of rows, stopping at the ends" },
      { keys: ["→", "l", "Enter"], does: "sidebar into the page, at the sub-section's first row" },
      { keys: ["←", "h", "Esc"], does: "page back to the sidebar, on the row's sub-section" },
      { keys: ["Ctrl+Page Up", "Ctrl+Page Down"], does: "the section before or after, from anywhere, round" },
      { keys: ["/", "Ctrl+F"], does: "Home's field; on a section, opens the find box" },
      { keys: ["Enter", "Shift+Enter"], does: "Home: the result; find box: the next or previous match" },
      { keys: ["Esc"], does: "clears Home's field; closes the find box, the page put back" },
      { keys: ["Enter", "Space"], does: "on a row: what the row does" },
    ],
    order: "The sidebar, then the page in reading order; Home's field or an open find box comes first.",
    lands: "It opens on Home, in the field; a setting from Home's search, on its row.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A section at the minimum", for: "the sidebar and its rows, the page taking the rest", sample: () => small({ section: 3 }) },
      { name: "Home", for: "the column, `size.column` at most", sample: () => still(settings(), "var(--size-app_width)", "var(--size-app_height)") },
    ]),
    measures: [
      { part: "Sidebar's width", token: "size-sidebar" },
      { part: "Its padding, its thumb's room", token: "space-s12" },
      { part: "Between its rows", token: "space-s4" },
      { part: "A section's row", token: "size-row" },
      { part: "A sub-section's row", token: "size-control_sm" },
      { part: "A row's corners", token: "radius-control" },
      { part: "Bar's length", token: "type-small-line" },
      { part: "Bar's thickness", token: "tabs-rail" },
      { part: "Page's parts; the lead line", token: "space-s32" },
      { part: "Home's column", token: "size-column" },
      { part: "Find box's width", token: "size-find" },
      { part: "Find box below the top", token: "space-s16" },
      { part: "A row unmatched", token: "opacity-miss" },
      { part: "The landed highlight", token: "opacity-tint" },
      { part: "The app's least width", token: "size-app_min_width" },
    ],
    reflow: {
      note: "The sidebar keeps its width; the page takes the rest, its padding the container's for its own width.",
      sample: () => settings({ section: 0 }),
    },
  },
});
