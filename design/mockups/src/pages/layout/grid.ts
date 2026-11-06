// The Grid, Split and Flow page: `docs/design/components/layout.md`, "Grid", "Split" and "Flow".
// The two-way layouts: equal columns, columns of set shares, and children that wrap onto new lines.
// Every column's edge lands on a whole device pixel.

import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import { card } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { shown, type Words } from "../../lib/content.ts";
import { filling, flow, grid, splitColumns, stack, within } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { media } from "../../lib/media.ts";
import { role } from "../../lib/radius.ts";
import { wrapped } from "../../lib/text.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { settingsWindow } from "./example.ts";

const COLUMN = "var(--size-column)";

const LANGUAGES = ["English (US)", "Italian", "German", "Japanese", "Polish", "French", "Spanish", "Portuguese (Brazil)"];

const VALUE: Words = { short: "tuxedo", long: "tuxedo-infinitybook-pro-14-gen9, on the desk by the window" };
const CHIP: Words = { short: "English (US)", long: "Portuguese (Brazil), with the ABNT2 layout" };

/** Wallpapers in `columns` equal columns, `space.s16` apart both ways. */
function pictures(columns = 3, count = 6, width = COLUMN): HTMLElement {
  const cells = Array.from({ length: count }, (_, index) => media({ source: photo, corner: role("control"), caption: `Shinjuku, ${index + 1}` }));
  return within(width, grid(columns, "s16", cells));
}

/** Cards side by side, each filling its row, so they end together; `uniform`, every row as tall. */
function cards(uniform = false): HTMLElement {
  const words = ["Samsung 990 Pro, 2 TB", "Encrypted, with your passphrase and your YubiKey", "Kuba", "KubaOS 0.4"];
  const titles = ["Disk", "Encryption", "Account", "Version"];
  const cells = titles.map((title, index) => filling(card({ header: { title }, body: [wrapped(words[index]!, "body", "on_surface_variant")] })));
  return within(COLUMN, grid(2, "s16", cells, { uniformRows: uniform }));
}

/** Facts in a split a quarter and three quarters, each label centred on its value. */
function facts(value = "tuxedo", width = COLUMN): HTMLElement {
  const cells = [
    ["Hostname", value],
    ["Kernel", "6.17.0-42"],
    ["Uptime", "3 days"],
  ].flatMap(([label, shown]) => [wrapped(label!, "small", "on_surface_variant"), wrapped(shown!, "body_strong")]);
  return within(width, splitColumns([1, 3], "s8", cells, { centred: true }));
}

/** Kōbō's time zone: a list two thirds wide beside its card, the grid filling the height. */
function timezone(): HTMLElement {
  const cities = list([{ title: "Warsaw", subtitle: "Europe", checked: true }, { title: "Tokyo", subtitle: "Asia" }, { title: "Lisbon", subtitle: "Europe" }]);
  const side = filling(card({ header: { glyph: "public", title: "Warsaw", subtitle: "Europe" } }));
  const box = within(COLUMN, splitColumns([2, 1], "s20", [cities, side], { fillHeight: true }));
  box.style.height = "calc(var(--size-app_min_height) / 2)";
  box.classList.add("l-room");
  return box;
}

/** Language chips flowing onto new lines in `width`. */
function languages(width = COLUMN, count = LANGUAGES.length, first?: string): HTMLElement {
  const labels = LANGUAGES.slice(0, count);
  if (first !== undefined && labels.length > 0) labels[0] = first;
  return within(width, flow("s8", ...labels.map((label, index) => chip({ label, kind: index === 0 ? "Selected" : "Neutral" }))));
}

export default page({
  level: "layout",
  slug: "grid",
  title: "Grid, Split and Flow",
  lede: "Columns whose edges line up from row to row, equal or of set shares, and children that flow onto a new line.",
  code: { rust: "layout::grid, layout::split, layout::flow", qml: "Grid, Split, Flow" },
  stage: {
    live: () => within("100%", stack("s32", "start", ...[pictures(3, 3, "100%"), facts(shown(VALUE), "100%"), languages("100%", LANGUAGES.length, shown(CHIP))].map(filling))),
    hint: "pick Text long: the chips wrap, the columns hold",
    daylight: true,
  },
  variants: [
    { name: "Grid", for: "equal columns: wallpapers, three across", sample: () => pictures() },
    { name: "Grid, cells that fill", for: "cards in a row end together", sample: () => cards() },
    { name: "Grid, uniform rows", for: "every row as tall: Kōbō's summary cards", sample: () => cards(true) },
    { name: "Split, [1, 3], centred", for: "a label a quarter wide, centred on its value", sample: () => facts() },
    { name: "Split, [2, 1], filling the height", for: "a list beside its card, to the page's foot", sample: timezone },
    { name: "Flow", for: "chips that wrap onto new lines", sample: () => languages() },
  ],
  sizes: doesntApply("none has a size: columns share the room, rows take their tallest cell."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "they take no pointer and no focus",
      pressed: "they take no press",
      disabled: "what they hold is disabled on its own",
      selected: "a picked cell is its own part's state",
      loading: "a `Skeleton` in the cell's final box, so nothing moves",
      error: "they have no value to be wrong",
    },
    rows: [
      { name: "Grid", cell: (state: "rest" | "empty") => (state === "empty" ? pictures(3, 0) : pictures(3, 3)) },
      { name: "Flow", cell: (state: "rest" | "empty") => (state === "empty" ? languages(COLUMN, 0) : languages(COLUMN, 5)) },
    ],
    looks: {
      rest: "they draw nothing but what they hold",
      empty: "no room; a list of nothing shows its `EmptyList`",
    },
  },
  focus: doesntApply("none is a control: no line and no glow."),
  content: {
    text: [
      { name: "A split's value", words: VALUE, sample: (words: string) => facts(words) },
      { name: "A flow's chip", words: CHIP, sample: (words: string) => languages(COLUMN, 5, words) },
    ],
    missingPicture: doesntApply("a cell's `Media` keeps its box, so the grid holds."),
    counts: [
      { name: "One", for: "a column wide; the others stay empty", sample: () => pictures(3, 1) },
      { name: "A row and a part", for: "a short last row starts at the first column", sample: () => pictures(3, 4) },
      { name: "Many, in a flow", for: "as many lines as the chips need", sample: () => languages("var(--size-panel)") },
    ],
  },
  motion: {
    arrives: doesntApply("they draw nothing to arrive; what they hold has its own moves."),
    leaves: doesntApply("they draw nothing to leave."),
    between: doesntApply("layout never animates: a cell that moves is there at once."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "what they hold: row by row, line by line" },
      { keys: ["←", "→", "↑", "↓"], does: "only a set picked as one stop moves by columns and rows" },
    ],
    order: "The order on screen is the order of the keys.",
    lands: "Nowhere of their own: the surface lands on its first thing to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Grid", for: "three columns, `space.s16` apart both ways", sample: () => pictures(3, 6) },
      { name: "Split, [1, 3]", for: "a quarter, `space.s8` to its values and between rows", sample: () => facts() },
      { name: "Split, [2, 1], filling the height", for: "`space.s20` between the list and its card", sample: timezone },
    ]),
    measures: [
      { part: "A grid's gaps", token: "space-s16" },
      { part: "A split's gaps", token: "space-s8" },
      { part: "A flow's gaps", token: "space-s8" },
      { part: "Kōbō's two columns", token: "space-s20" },
      { part: "A wallpaper's shape", token: "aspect-wallpaper" },
      { part: "The samples' room", token: "size-column" },
    ],
    reflow: {
      note: "Wallpapers keep three columns, 131 wide to over 500; the split keeps its quarter; chips flow onto a second line.",
      sample: settingsWindow,
    },
  },
});
