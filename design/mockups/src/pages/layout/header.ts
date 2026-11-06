// The Header and Section page: `docs/design/components/layout.md`, "Header" and "Section". A page's
// header (its accent, title and the line under it) and a titled part of a page. A surface's
// headers have their subtitle always or never.

import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { filling, flow, header, section, stack, within } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { appearanceHeader, SUBTITLE, TITLE, settingsWindow, themeSection } from "./example.ts";

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
  return within(COLUMN, header({ accent: "通知", title, subtitle, actions: actions() }));
}

/** The buttons beside a panel's header. */
function actions(): HTMLElement[] {
  return [iconButton({ icon: "settings", label: "Settings", kind: "Text", size: "Sm", hint: false }), button({ label: "Clear all", kind: "Tonal", size: "Sm" })];
}

/** A section with its line under the heading, holding chips. */
function described(heading = "Theme", body: string | undefined = "Where the colours come from."): HTMLElement {
  return within(COLUMN, section(heading, body, flow("s8", chip({ label: "From the wallpaper", kind: "Selected" }), chip({ label: "Neon sign" }), chip({ label: "Rose" }))));
}

/** A section with its heading alone, holding a list. */
function headingAlone(): HTMLElement {
  return within(COLUMN, fonts());
}

function fonts(): HTMLElement {
  return section("Fonts", undefined, list([{ title: "Interface", subtitle: "Inter, 14", opens: true }, { title: "Monospace", subtitle: "JetBrains Mono, 13", opens: true }]));
}

/** A page's opening across the room it has: its header with buttons, then two sections. */
function opening(): HTMLElement {
  // A lit button's glow reaches past its box: this leaves it room in the pane, which a sample
  // overflowing would scale down.
  return within("calc(100% - var(--focus-far))", stack("s32", "start", ...[header({ accent: "外観", title: shown(TITLE), subtitle: shown(SUBTITLE), actions: actions() }), themeSection(), fonts()].map(filling)));
}

/** A page's header over two sections, the stack's `space.s32` between them. */
function together(): HTMLElement {
  return within(COLUMN, stack("s32", "start", ...[appearanceHeader(), themeSection(), filling(headingAlone())].map(filling)));
}

export default page({
  level: "layout",
  slug: "header",
  title: "Header and Section",
  lede: "A page's accent, title and subtitle, and the titled parts of the page under them.",
  code: { rust: "layout::header, layout::section", qml: "Header, Section" },
  stage: {
    live: opening,
    hint: "pick Text long: titles wrap, the buttons stay put",
    daylight: true,
  },
  variants: [
    { name: "Header, with a subtitle", for: "every Settings page, Kōbō step and panel", sample: () => withSubtitle() },
    { name: "Header, without", for: "a surface decides once: subtitles always or never", sample: () => withoutSubtitle() },
    { name: "Header, with buttons", for: "at its end, centred on title and subtitle", sample: () => withActions() },
    { name: "Section, with its line", for: "a heading and what the group is for", sample: () => described() },
    { name: "Section, a heading alone", for: "a heading that says it all", sample: headingAlone },
  ],
  sizes: doesntApply("one size each: a header's `title`, a section's `heading`."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "no pointer or focus; buttons beside it light on their own",
      pressed: "they take no press",
      disabled: "words aren't disabled; what a section holds is",
      selected: "neither is picked",
      loading: "known before the page shows; a waiting subtitle is empty",
      error: "said in the page's body, never in a header",
    },
    rows: [
      { name: "Header", cell: (state: "rest" | "empty") => (state === "empty" ? emptySubtitle() : withSubtitle("Notifications", "Nothing new. Suspiciously quiet.")) },
      { name: "Header, with buttons", cell: (state: "rest" | "empty") => (state === "empty" ? within(COLUMN, header({ accent: "通知", title: "Notifications", subtitle: "", actions: [button({ label: "Clear all", kind: "Tonal", size: "Sm" })] })) : withActions()) },
    ],
    looks: {
      rest: "accent `primary`, title `on_surface`, subtitle `on_surface_variant`",
      empty: "the subtitle's line keeps its height, so nothing moves",
    },
  },
  focus: doesntApply("neither is a control; buttons beside a header light on their own page."),
  content: {
    text: [
      { name: "A header's title", words: TITLE, sample: (words: string) => withSubtitle(words) },
      { name: "A header's subtitle", words: SUBTITLE, sample: (words: string) => withSubtitle("Appearance", words) },
      { name: "A title, with buttons", words: TITLE, sample: (words: string) => withActions(words) },
      { name: "A section's heading", words: HEADING, sample: (words: string) => described(words) },
      { name: "A section's line", words: BODY, sample: (words: string) => described("Theme", words) },
      { name: "The accent", words: fixed("外観"), sample: () => withSubtitle() },
    ],
    missingPicture: doesntApply("neither draws a picture or a symbol."),
    counts: [
      { name: "A header and no sections", for: "a page of one thing", sample: () => withSubtitle() },
      { name: "A header and sections", for: "each named group `space.s32` apart", sample: together },
    ],
  },
  motion: {
    arrives: doesntApply("they come with their page, which moves them."),
    leaves: doesntApply("they go with their page."),
    between: powerMoves(() => withActions()),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "a header's buttons after its title; a section's controls after its heading" }],
    order: "Read out as headings: the page's first level, a section's second.",
    lands: "A page lands on its first thing to do, never its header.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Header, with a subtitle", for: "accent, title, `space.s8`, subtitle", sample: () => withSubtitle() },
      { name: "Header, with buttons", for: "block to buttons `space.s16`, buttons `space.s8` apart", sample: () => withActions() },
      { name: "Section", for: "heading, `space.s4`, its line, `space.s16` to the content", sample: () => described() },
    ]),
    measures: [
      { part: "The accent's line", token: "type-accent-line" },
      { part: "The title's line", token: "type-title-line" },
      { part: "Title to subtitle", token: "space-s8" },
      { part: "The subtitle's line, kept when empty", token: "type-body-line" },
      { part: "Title block to buttons", token: "space-s16" },
      { part: "A heading's line", token: "type-heading-line" },
      { part: "Heading to its line", token: "space-s4" },
      { part: "Heading to its content", token: "space-s16" },
      { part: "Between sections, in Settings", token: "space-s32" },
    ],
    reflow: {
      note: "In Settings' page, titles and lines wrap between words at any width; the subtitle keeps its line.",
      sample: settingsWindow,
    },
  },
});
