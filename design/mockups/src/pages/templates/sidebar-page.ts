// The SidebarPage page: `docs/design/components/sidebar-page.md`. An app's window as its sections
// in a sidebar beside a page that scrolls, with Home, a page that is only search, at the top:
// Settings' layout. Live in a window that resizes, from the app's minimum to a 1080p screen.

import type { Shown } from "../../lib/control.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { stack } from "../../lib/layout.ts";
import { type SidebarPageOptions, sidebarPage, sidebarRow } from "../../lib/sidebar-page.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, nonEmpty, page, type Track } from "../../site/page.ts";
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
    what: "A section picked in the sidebar, by the pointer or the keys, or with Ctrl+Page Down: the new page fades in and rises `space.s8` into place over `short` on `emphasized_decel` (`--section-rise`, 0 to 1: its opacity, and `space.s8` less that much of it down). The sidebar opens the section's sub-sections at once and the bar is in place on the first; the page starts at its top. With reduced motion it is a plain swap.",
    tracks: rising(false),
    reduced: rising(true),
    back: "cuts",
    sample: () => small({ section: 3 }),
  },
  {
    name: "The bar glides",
    what: "The current sub-section changes, as the page scrolls past a heading or a sub-section is pressed: the bar glides from its row to the new one over `medium` on `standard` (`--bar-at`, 0 to 1, from `--bar-from` to `--bar-to`, landing on whole device pixels). A press also glides the page's scroll to the sub-section, its heading at the lead line `space.s32` below the page's top, on the same duration and curve. With reduced motion both are there at once.",
    tracks: [{ part: "the bar", on: "the current sub-section", target: ".k-sp-bar", property: "--bar-at", from: 0, to: 1, duration: "medium", curve: "standard" }],
    reduced: [{ part: "the bar", on: "the current sub-section", target: ".k-sp-bar", property: "--bar-at", from: 0, to: 1, duration: "instant", curve: "linear" }],
    back: "cuts",
    sample: barSample,
  },
  {
    name: "The find box arrives",
    what: "`/` or Ctrl+F on a section's page: the box fades in and drops `space.s8` into place over `short` on `standard` (`--find-shown`, 0 to 1: its opacity, and `space.s8` less that much of it up). It takes no layout, so nothing under it moves. With reduced motion only the fade plays.",
    tracks: showing(0, 1),
    reduced: "same",
    back: "cuts",
    sample: () => small({ section: 3, find: "" }),
  },
  {
    name: "The find box goes",
    what: "Esc in the box: it fades out and rises `space.s8` over `short` on `standard` (`--find-shown`, 1 to 0), and the page is put back where it was, at once. With reduced motion only the fade plays.",
    tracks: showing(1, 0),
    reduced: "same",
    back: "cuts",
    sample: () => small({ section: 3, find: "" }),
  },
  {
    name: "Landed",
    what: "A setting opened from Home's search: its section shows, scrolled to its row, and the row is lit by the landed highlight, the accent at `opacity.tint` over the row, fading over `slow` on `standard` (`--landed`, 1 to 0). The find box's current match takes the same highlight, held while it is current.",
    tracks: [{ part: "the landed highlight", on: "the row landed on", target: ".k-list-row[data-landed]", property: "--landed", from: 1, to: 0, duration: "slow", curve: "standard" }],
    reduced: "same",
    back: "cuts",
    sample: () => small({ section: 3, landed: 2 }),
  },
  {
    name: "A sidebar row lights up",
    what: "The pointer comes onto a row, or the keyboard does: it lights as a Text button does, its tube and the glow outside it fading in over `short` on `standard` (`--lit`), its words to `on_surface`. A press quietly selects: the tube shows only for the keyboard and the pointer over it.",
    tracks: lights(".k-sp-row", "a section's row", 0, 1),
    reduced: "same",
    back: "plays",
    sample: () => rowIn("section", shown(SECTION), "bolt", false, "rest"),
  },
  {
    name: "A sidebar row pressed",
    what: "A lit row pressed: its tube runs white-hot to the core over `micro` (`--press`), and back as it is let go.",
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
  summary: "An app's window as its sections in a sidebar beside a page that scrolls, with Home, a page that is only search, at the top: Settings.",
  live: () => resizable(settings({ focus: true })),
  description: [
    "Rust `template::sidebar_page`; iced only, an app's window. Drag the window's corner, from 720 × 480 to a 1080p screen. The window opens on Home with the keyboard in its search field: type a setting's name, ↑ ↓ to a result and Enter to land on it. Click a section, scroll its page and watch the bar follow; `/` or Ctrl+F finds on a page.",
    "The sidebar is a band `size.sidebar` wide in `surface_container_low`, full height, its rows `space.s12` in and `space.s4` apart. Home is its first row, then a hairline `space.s8` clear of the rows round it, then the sections. A section's row is `size.row` tall: its symbol `body`'s, `space.s12` before its words in `body`, `space.s12` inside its sides, at the control radius, which is the window's corner less the band's `space.s12`, so the two are concentric. The current section (or Home) is filled in `surface_container_high`, its symbol `primary` and its words `on_surface`; the rest are `on_surface_variant`. Only the current section opens its sub-sections under it, rows `size.control_sm` tall in `small`, their words where a section's start.",
    "Beside the current sub-section a short bar of neon says where the page is: `type.small`'s line tall and the tabs' rail thick, centred in the symbols' column; at night the core with the content glow round it, by day the tone's glass with none. It glides from row to row over `medium` as the page scrolls and on a press; a change of section puts it on the first at once. When the sidebar runs past the window it scrolls on its own, its rows staying `size.row`, the band's padding its thumb's room, and it keeps the keyboard's row in view with `space.s12` round it.",
    "The page is a `Page` that scrolls: the section's `Header` (accent, title, its pun lede), then each sub-section, a `Section` with its heading over its wells of rows (`List`), `space.s32` apart. Its thumb floats in the page's own padding at its end, with no gutter of its own. Pressing a sub-section in the sidebar glides the page until its heading is at the lead line, `space.s32` below the top; as the page scrolls, the current sub-section is the last whose heading has reached that line, or the last of all once the page can scroll no further. A sub-section pressed stays current while the page stays where it went for it, though it may be too short to reach the line.",
    "Home is a centred column `size.column` wide at most: Home's header, the search field across every setting (Kotodama's engine, which a stand-in matcher plays here), then under it, `space.s24` apart, the sections while the field is empty, or each setting that matches as a two-line row: its name with the typed letters lit, its path under it (Appearance › Theme) and its value at its end. Enter, or a press on a result, goes to the setting: its section, scrolled to its row, the row lit by the landed highlight, which fades over `slow`.",
    "The find box, `/` or Ctrl+F on a section's page, floats in the page's top-right corner `space.s16` below its top and the container padding in from its end: a `Popover` `size.find` wide, raised to the menus' step, over the content and taking no layout. It holds a field and, once something is typed, \"n of m\" in `mono`. It searches that page only: matching letters lit, rows that don't match at `opacity.miss`, the current match taking the landed highlight, scrolled into view below the box, the sidebar's bar following into its sub-section. With no match, key hints under the field offer Enter to search every setting from Home, the query carried over.",
  ],
  variants: [
    { name: "Home", for: "Where the window opens: search across every setting, the sections under it while the field is empty.", sample: () => small() },
    { name: "A section", for: "A section's page scrolled to its second sub-section, the bar beside it.", sample: () => small({ section: 3, sub: 1 }) },
    { name: "Find on a page", for: "The find box over a section's page, on its second match of three.", sample: () => small({ section: 3, find: "e", findAt: 1, keyboard: "find" }) },
    { name: "Home's results", for: "Typed in Home's field: each setting that matches, its letters lit, its path and its value.", sample: () => small({ query: "lo", keyboard: "home" }) },
  ],
  sizes: doesntApply("it has no sizes of its own: it fills its window, from the app's minimum to a full screen (Measures, Reflow)."),
  states: {
    shown: ["rest", "lit", "pressed", "selected"],
    absent: {
      disabled: "a section is never off; a setting that can't be changed is disabled on its own row.",
      loading: "the outline is the app's and is there as the window opens; a row whose value is still coming shows its own skeleton.",
      empty: "every section has a sub-section; Home's search with nothing found says so under its field (Content, Counts).",
      error: "an error is said by what the page holds: an `ErrorBlock` or a `Banner` in its sub-section.",
    },
    rows: nonEmpty(
      ROWS.map(({ name, kind, glyph, words }) => ({
        name,
        cell: (state: MatrixState) => rowIn(kind, shown(words), glyph, state === "selected", state === "selected" ? "rest" : state),
      })),
    ),
    described: {
      rest: "Words and symbol in `on_surface_variant`, no fill, no line. By day the same in daylight colours.",
      lit: "Under the pointer or with the keyboard on it: the tube inside its edge with the glow outside at night, its words `on_surface`. By day the tube is glass, with no glow.",
      pressed: "Lit and held: the tube runs white-hot to the core over `micro`; by day the glass line darkens as a Text button's does.",
      selected: "The current section: filled in `surface_container_high`, its symbol `primary`, its words `on_surface`. The current sub-section: its words `on_surface`, with the bar beside it. A press on a row that is already current changes nothing.",
    },
  },
  focus: {
    tubes: [
      { name: "A sidebar row, lit", tone: "accent", filled: false, content: false },
      { name: "A well of the page, with the keyboard on one of its rows", tone: "accent", filled: false, content: false },
      { name: "Home's field and the find box's field", tone: "accent", filled: false, content: false },
      { name: "The bar beside the current sub-section, always", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "At night: a lit sidebar row burns its tube inside its edge with the glow outside; the page's well with the keyboard burns its tube round the well, its row lit on `secondary_container`; the fields as a field does. The bar is always the core with its content glow.",
      signsOff: "By day, and at night while a power change has the lights out: each tube is glass, `primary_glass`, with no glow, and the bar is the tone's glass, a pixel thicker by day.",
    },
    signsOff: (index: number) => rowIn(ROWS[index]!.kind, shown(ROWS[index]!.words), ROWS[index]!.glyph, false, "signs-off"),
  },
  content: {
    text: [
      { name: "A section's name in the sidebar", words: SECTION, sample: (words: string) => rowIn("section", words, "bolt", true, "rest") },
      { name: "A sub-section's name", words: SUB, sample: (words: string) => rowIn("sub", words, null, false, "rest") },
      { name: "The whole window, its words at the toolbar's length", words: fixed("Settings"), sample: () => small({ section: 0 }) },
    ],
    missingPicture: doesntApply("every section has its symbol, and the template draws no picture of its own."),
    counts: [
      { name: "Nothing found from Home", for: "Typed what matches no setting: a line under the field says so, and the sections come back once it is cleared.", sample: () => small({ query: "xyzzy", keyboard: "home" }) },
      { name: "Nothing found on a page", for: "The find box with no match: every row stands back, and the hint under the field offers Enter to search every setting from Home.", sample: () => small({ section: 3, find: "xyzzy", keyboard: "find" }) },
      { name: "Few sections", for: "Three sections: the sidebar ends where its rows do, its band running on to the window's foot.", sample: () => small({ section: 0 }, fewer(3)) },
      { name: "Many sections", for: "Ten sections at 480 tall, a section open: the sidebar scrolls on its own, its rows staying `size.row`, the thumb in its padding.", sample: () => small({ section: 9 }) },
    ],
  },
  motion: { arrives: doesntApply("it comes with its window, which Hyprland opens; the window opens on Home, with nothing moving."), leaves: doesntApply("it goes with its window."), between: MOVES },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Round the window's stops: on Home, its search field and the sidebar; on a section, the find box while it is open, the sidebar, then the page." },
      { keys: ["↑", "↓", "j", "k"], does: "In the sidebar, through Home, the sections and the current section's sub-sections, round from either end, showing what they land on; in the page, through its rows, each brought into view with `space.s24` round it." },
      { keys: ["Ctrl+N", "Ctrl+P", "Ctrl+J", "Ctrl+K"], does: "The same, also from Home's field, where they move through the results." },
      { keys: ["Home", "End", "Page Up", "Page Down"], does: "The first or last row, or a page of rows at a time, stopping at the ends." },
      { keys: ["→", "l", "Enter"], does: "From the sidebar into the page, at the first row of the current sub-section; on Home, into its field." },
      { keys: ["←", "h", "Esc"], does: "From the page back to the sidebar, onto the row's sub-section." },
      { keys: ["Ctrl+Page Up", "Ctrl+Page Down"], does: "The section before or after, from anywhere, Home included, round from either end." },
      { keys: ["/", "Ctrl+F"], does: "On Home, into its field; on a section, opens the find box." },
      { keys: ["Enter", "Shift+Enter"], does: "In Home's field, goes to the result the keyboard is on; in the find box, the next or the previous match (↓ and ↑ too), round from either end; with no match, Enter searches from Home." },
      { keys: ["Esc"], does: "In Home's field, clears it, and a second Esc goes to the sidebar; in the find box, closes it, puts the page back where it was and the keyboard where it was." },
      { keys: ["Enter", "Space"], does: "On a row of the page, what the row does: opens it, or turns its switch." },
    ],
    order: "Sidebar, then the page, in reading order; on Home the field comes first. The find box is the first stop while it is open.",
    lands: "The window opens on Home with the keyboard in its field. A section picked by the pointer leaves the keyboard where it was; by the keys, it stays in the sidebar. A setting opened from Home's search lands the keyboard on its row.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A section at the minimum", for: "The sidebar `size.sidebar` wide, its rows `space.s12` in and `space.s4` apart, the page taking the rest, padded `space.s16` at 456 wide.", sample: () => small({ section: 3 }) },
      { name: "Home", for: "The column `size.column` wide at most, its header, field and answers `space.s24` apart.", sample: () => still(settings(), "var(--size-app_width)", "var(--size-app_height)") },
    ]),
    measures: [
      { what: "The sidebar's width.", token: "size-sidebar" },
      { what: "The sidebar's padding round its rows, and the band's room for its thumb.", token: "space-s12" },
      { what: "Between the sidebar's rows, and the hairline's clear room with it.", token: "space-s4" },
      { what: "A section's row.", token: "size-row" },
      { what: "A sub-section's row.", token: "size-control_sm" },
      { what: "A row's corners, the window's less the band's padding.", token: "radius-control" },
      { what: "The bar's length, `small`'s line.", token: "type-small-line" },
      { what: "The bar's thickness at night, and by day a pixel more.", token: "tabs-rail" },
      { what: "Between the page's parts, and the lead line below its top.", token: "space-s32" },
      { what: "Home's column, at most.", token: "size-column" },
      { what: "The find box's width.", token: "size-find" },
      { what: "The find box below the page's top.", token: "space-s16" },
      { what: "A row the find box doesn't match.", token: "opacity-miss" },
      { what: "The landed highlight, the accent over the row.", token: "opacity-tint" },
      { what: "The window's minimum width, which leaves the page 456.", token: "size-app_min_width" },
    ],
    reflow: {
      described:
        "Settings in a window of the toolbar's size. The sidebar keeps its `size.sidebar` at every width and the page takes the rest, its padding the container's for its own width: 16 on the 456 a minimal window leaves it, 24 from 640, 32 from 1024. Home's column is `size.column` wide at most and centred in the page; under 592 it takes the page's width less its padding. The sidebar scrolls on its own when the window is too short for its rows; nothing else in it is a fixed height.",
      sample: () => settings({ section: 0 }),
    },
  },
});
