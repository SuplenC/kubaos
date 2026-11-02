// The Header and Section page: `docs/design/components/layout.md`, "Header" and "Section". A page's
// header (its accent, title and the line under it) and a titled part of a page. A surface's
// headers have their subtitle always or never.

import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import { fixed, type Words } from "../../lib/content.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { filling, flow, header, section, stack, within } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { SUBTITLE, TITLE, settingsWindow, themeSection } from "./example.ts";

const COLUMN = "var(--size-column)";

const HEADING: Words = { short: "Theme", long: "Where the colours on every screen come from" };
const BODY: Words = { short: "Where the colours come from.", long: "Where the colours come from: the wallpaper you have on, a theme of ours, or a colour you pick yourself." };

/** A header with a subtitle, `words` its title or its subtitle. */
function withSubtitle(title = "Appearance", subtitle = "Make it yours."): HTMLElement {
  return within(COLUMN, header({ accent: "外観", title, subtitle }));
}

/** A header whose surface has no subtitles. */
function withoutSubtitle(title = "Appearance"): HTMLElement {
  return within(COLUMN, header({ accent: "外観", title }));
}

/** A header whose subtitle has no words just now: its line keeps its height. */
function emptySubtitle(): HTMLElement {
  return within(COLUMN, header({ accent: "通知", title: "Notifications", subtitle: "" }));
}

/** A header with buttons beside it, centred on its title and subtitle. */
function withActions(title = "Notifications", subtitle = "Nothing new. Suspiciously quiet."): HTMLElement {
  const actions = [iconButton({ icon: "settings", label: "Settings", kind: "Text", size: "Sm", hint: false }), button({ label: "Clear all", kind: "Tonal", size: "Sm" })];
  return within(COLUMN, header({ accent: "通知", title, subtitle, actions }));
}

/** A section with its line under the heading, holding chips. */
function described(heading = "Theme", body: string | undefined = "Where the colours come from."): HTMLElement {
  return within(COLUMN, section(heading, body, flow("s8", chip({ label: "From the wallpaper", kind: "Selected" }), chip({ label: "Neon sign" }), chip({ label: "Rose" }))));
}

/** A section with its heading alone, holding a list. */
function headingAlone(): HTMLElement {
  return within(COLUMN, section("Fonts", undefined, list([{ title: "Interface", subtitle: "Inter, 14", opens: true }, { title: "Monospace", subtitle: "JetBrains Mono, 13", opens: true }])));
}

/** A page's header over two sections, the stack's `space.s32` between them. */
function together(): HTMLElement {
  return within(COLUMN, stack("s32", "start", ...[header({ accent: "外観", title: "Appearance", subtitle: "Make it yours." }), themeSection(), filling(headingAlone())].map(filling)));
}

export default page({
  level: "layout",
  slug: "header",
  title: "Header and Section",
  summary: "A page's header (its accent, its title and the line under it) and a titled part of the page under it.",
  live: together,
  description: [
    "Rust `layout::header` and `layout::section`; QML `Header` and `Section`. A page opens with one header and is cut into sections; neither is a card or a well, so neither draws anything but its words.",
    "Header is the page's accent in `accent` (`primary`), a Japanese word for what the page is about; its title in `title` (`on_surface`), breaking between words; and its subtitle in `body` (`on_surface_variant`), `space.s8` under the title. The title and the subtitle wrap; the accent stays on one line.",
    "A header has a subtitle always or never. A surface decides once whether its headers have one (every Settings page, every Kōbō step, every right-edge panel has one) and then every header on it has the line. Given, the line keeps its height even when it has no words just now, so the title, the subtitle and the buttons beside them never move: not while a panel is open, not when twin panels swap, not when a count goes to zero. The subtitle is where the surface's attitude goes, a pun or a joke in the voice of `docs/BRAND.md`; labels stay plain. Counts and status stay in the page's body.",
    "Buttons beside a header sit at its end, `space.s16` from the title's room and `space.s8` apart, centred on the title-and-subtitle block (not on the accent above it), so they stay put whatever the subtitle says.",
    "Section is a titled part of a page: its heading in `heading` (`on_surface`), an optional line `space.s4` under it in `body` (`on_surface_variant`), then its content `space.s16` below. Sections follow one another in the page's stack, `space.s32` apart in Settings; a section is never a card, and a card never holds a section.",
    "When to use which: one header per page, at its top; a section for each group of the page's content that has a name. A group with no name needs no section: it is the stack's next child. A heading inside a card is the card's header, not a section.",
  ],
  variants: [
    { name: "Header, with a subtitle", for: "A surface whose headers have the line: every Settings page, every Kōbō step, every panel.", sample: () => withSubtitle() },
    { name: "Header, without", for: "A surface whose headers never have it: the line and its room are gone.", sample: () => withoutSubtitle() },
    { name: "Header, with buttons", for: "Buttons at its end, centred on the title and subtitle: a panel's actions.", sample: () => withActions() },
    { name: "Section, with its line", for: "A heading and a line saying what the group is for.", sample: () => described() },
    { name: "Section, a heading alone", for: "A heading that says it all.", sample: headingAlone },
  ],
  sizes: doesntApply("one size each: a header is the page's, in `title`; a section's heading is `heading`. A panel's header band is PanelPage's, on the templates' pages."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "neither takes a pointer or focus; buttons beside a header light on their own.",
      pressed: "neither takes a press.",
      disabled: "words aren't disabled; what a section holds is, each control on its own.",
      selected: "neither is picked.",
      loading: "the title and the heading are known before the page shows; a subtitle that waits for its words is empty, below.",
      error: "an error is said in the page's body (`ErrorBlock`, `Banner`), never in a header.",
    },
    rows: [
      { name: "Header", cell: (state: "rest" | "empty") => (state === "empty" ? emptySubtitle() : withSubtitle("Notifications", "Nothing new. Suspiciously quiet.")) },
      { name: "Header, with buttons", cell: (state: "rest" | "empty") => (state === "empty" ? within(COLUMN, header({ accent: "通知", title: "Notifications", subtitle: "", actions: [button({ label: "Clear all", kind: "Tonal", size: "Sm" })] })) : withActions()) },
    ],
    described: {
      rest: "The accent in `primary`, the title in `on_surface`, the subtitle and a section's line in `on_surface_variant`. By day the same roles in daylight colours.",
      empty: "A subtitle with no words just now keeps its line's height (`type.body.line`), so the title and the buttons beside it stay where they were. A surface with no subtitles has none, not an empty one.",
    },
  },
  focus: doesntApply("neither is a control; buttons beside a header light as buttons do, on their own page."),
  content: {
    text: [
      { name: "A header's title", words: TITLE, sample: (words: string) => withSubtitle(words) },
      { name: "A header's subtitle", words: SUBTITLE, sample: (words: string) => withSubtitle("Appearance", words) },
      { name: "A header's title, with buttons", words: TITLE, sample: (words: string) => withActions(words) },
      { name: "A section's heading", words: HEADING, sample: (words: string) => described(words) },
      { name: "A section's line", words: BODY, sample: (words: string) => described("Theme", words) },
      { name: "The accent", words: fixed("外観"), sample: () => withSubtitle() },
    ],
    missingPicture: doesntApply("neither draws a picture or a symbol."),
    counts: [
      { name: "A header and no sections", for: "A page of one thing: its header, then the thing.", sample: () => withSubtitle() },
      { name: "A header and sections", for: "Each named group a section, `space.s32` apart.", sample: together },
    ],
  },
  motion: {
    arrives: doesntApply("they come with their page; a page changing (SidebarPage's change of section, Kōbō's slide) moves the whole page."),
    leaves: doesntApply("they go with their page."),
    between: powerMoves(() => withActions()),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "Nothing of their own: a header's buttons are stops in reading order after its title, and a section's controls follow its heading." }],
    order: "Headers and headings are read out as headings, the page's at the first level and a section's at the second, so a screen reader can jump between them.",
    lands: "A page lands on its first thing to do, never on its header.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Header, with a subtitle", for: "The accent, the title, `space.s8`, the subtitle.", sample: () => withSubtitle() },
      { name: "Header, with buttons", for: "The block and the buttons, `space.s16` apart, the buttons `space.s8` apart, centred on the block.", sample: () => withActions() },
      { name: "Section", for: "The heading, `space.s4`, its line, then `space.s16` to the content.", sample: () => described() },
    ]),
    measures: [
      { what: "The accent's line.", token: "type-accent-line" },
      { what: "The title's line.", token: "type-title-line" },
      { what: "Between the title and the subtitle.", token: "space-s8" },
      { what: "The subtitle's line, kept when it has no words.", token: "type-body-line" },
      { what: "Between the title-and-subtitle block and the buttons beside it.", token: "space-s16" },
      { what: "A section's heading's line.", token: "type-heading-line" },
      { what: "Between a heading and its line.", token: "space-s4" },
      { what: "Between a section's heading and its content.", token: "space-s16" },
      { what: "Between sections, in Settings' page.", token: "space-s32" },
    ],
    reflow: {
      described:
        "In Settings' page at the toolbar's window size: the title and the subtitle wrap between words in the page's width, at the minimum as at a full screen; the subtitle keeps its line whatever it says. Sections take the page's width, their headings and lines wrapping the same way.",
      sample: settingsWindow,
    },
  },
});
