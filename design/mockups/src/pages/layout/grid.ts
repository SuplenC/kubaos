// The Grid, Split and Flow page: `docs/design/components/layout.md`, "Grid", "Split" and "Flow".
// The two-way layouts: equal columns, columns of set shares, and children that wrap onto new lines.
// Every column's edge lands on a whole device pixel.

import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import { card } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import type { Words } from "../../lib/content.ts";
import { filling, flow, grid, splitColumns, stack, within } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { media } from "../../lib/media.ts";
import { role } from "../../lib/radius.ts";
import { wrapped } from "../../lib/text.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { settingsWindow } from "./example.ts";

const COLUMN = "var(--size-column)";

const LANGUAGES = ["English (US)", "Italian", "German", "Japanese", "Polish", "French", "Spanish", "Portuguese (Brazil)"];

const VALUE: Words = { short: "tuxedo", long: "tuxedo-infinitybook-pro-14-gen9, on the desk by the window" };
const CHIP: Words = { short: "Polish", long: "Portuguese (Brazil), with the ABNT2 layout" };

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
  summary: "Columns whose edges line up from row to row (equal ones, or of set shares) and children that flow onto a new line where the next doesn't fit.",
  live: () => within(COLUMN, stack("s32", "start", ...[pictures(3, 3, "100%"), facts("tuxedo", "100%"), languages("100%")].map(filling))),
  description: [
    "Rust `layout::grid`, `layout::split` and `layout::flow`; QML `Grid`, `Split` and `Flow`. Each lays its children out both ways, one spacing step apart across and down.",
    "Grid is equal columns, filled row by row. Its width less the gaps is split in whole device pixels, the spare ones to the first columns, so every edge lands on one and no two columns differ by more than one device pixel. A row is as tall as its tallest cell, and a cell that fills is stretched to it, so cards side by side end together. Its rows may all be as tall as the tallest (`uniform_rows`): Kōbō's four summary cards, one size. It may fill all the height there is (`fill_height`), sharing it between its rows, each a whole device pixel: columns of lists that run to the page's foot. In a scroll, where the height has no end, each row keeps its own.",
    "Split is a grid whose columns take their share of set weights: `[1, 3]` is a label column a quarter wide beside its values, `[2, 1]` a list beside its detail. It splits in whole device pixels as a grid does, and can centre each row's cells down it (`center`), a small label on its taller value.",
    "Flow puts its children side by side, one step apart both ways, and starts a new line where the next doesn't fit: chips, tags, checkboxes. No child is wider than the room; its lines are as tall as their tallest child. Its lines tell the geometry check nothing; each child still says what it is.",
    "When to use which: things whose edges should line up down the page (pictures, cards, labels and values) are a Grid or a Split; things of their own widths that only need to wrap are a Flow; one line that never wraps is a Row. A Grid's columns are a set count at every width.",
  ],
  variants: [
    { name: "Grid", for: "Equal columns: wallpapers, three across.", sample: () => pictures() },
    { name: "Grid, cells that fill", for: "Cards in a row stretched to the tallest, so they end together.", sample: () => cards() },
    { name: "Grid, uniform rows", for: "Every row as tall as the tallest: Kōbō's summary cards.", sample: () => cards(true) },
    { name: "Split, [1, 3], centred", for: "A label a quarter wide, centred on its value.", sample: () => facts() },
    { name: "Split, [2, 1], filling the height", for: "Kōbō's time zone: a list two thirds wide beside its card, to the page's foot.", sample: timezone },
    { name: "Flow", for: "Chips that wrap onto new lines.", sample: () => languages() },
  ],
  sizes: doesntApply("none has a size: columns take their share of the room, rows their tallest cell or their share of the height, a flow's lines its children."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "they take no pointer and no focus.",
      pressed: "they take no press.",
      disabled: "they hold controls, disabled each on their own.",
      selected: "a picked cell is its own part's state: a `ChoiceCard`, a `PictureChoice`.",
      loading: "a cell that loads holds its own `Skeleton` in its final box, so the grid doesn't move.",
      error: "they have no value to be wrong.",
    },
    rows: [
      { name: "Grid", cell: (state: "rest" | "empty") => (state === "empty" ? pictures(3, 0) : pictures(3, 3)) },
      { name: "Flow", cell: (state: "rest" | "empty") => (state === "empty" ? languages(COLUMN, 0) : languages(COLUMN, 5)) },
    ],
    described: {
      rest: "They draw nothing: what shows is what they hold, where they put it.",
      empty: "With nothing in it a grid or a flow takes no room; a list of nothing shows its `EmptyList`, a page its `EmptyState`.",
    },
  },
  focus: doesntApply("none is a control; they draw no line and no glow."),
  content: {
    text: [
      { name: "A split's value", words: VALUE, sample: (words: string) => facts(words) },
      { name: "A flow's chip", words: CHIP, sample: (words: string) => languages(COLUMN, 5, words) },
    ],
    missingPicture: doesntApply("they draw no picture: a cell's missing picture is its `Media`'s, which keeps its box, so the grid doesn't move."),
    counts: [
      { name: "One", for: "One cell, a column wide; the other columns stay empty.", sample: () => pictures(3, 1) },
      { name: "A row and a part", for: "A last row that isn't full starts at the first column.", sample: () => pictures(3, 4) },
      { name: "Many, in a flow", for: "As many lines as the chips need.", sample: () => languages("var(--size-panel)") },
    ],
  },
  motion: {
    arrives: doesntApply("they draw nothing to arrive; what they hold arrives by its own moves."),
    leaves: doesntApply("they draw nothing to leave."),
    between: doesntApply("layout never animates: a cell that moves to another column as the window resizes is there at once (`docs/DESIGN.md`, \"Motion\")."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Through what they hold in reading order: a grid row by row, each from its start; a flow line by line." },
      { keys: ["←", "→", "↑", "↓"], does: "Nothing of their own: a set picked as one stop (`ChoiceCard`, `PictureChoice`) moves by the grid's columns and rows." },
    ],
    order: "The order on screen is the order of the keys.",
    lands: "Nowhere of their own: the surface lands on its first thing to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Grid", for: "Three columns, `space.s16` apart both ways, split in whole device pixels.", sample: () => pictures(3, 6) },
      { name: "Split, [1, 3]", for: "The label's quarter, `space.s8` to the values and between rows.", sample: () => facts() },
      { name: "Split, [2, 1], filling the height", for: "Kōbō's `space.s20` between the list and its card.", sample: timezone },
    ]),
    measures: [
      { what: "Between a grid's columns and rows of pictures or cards.", token: "space-s16" },
      { what: "Between a split's label and value, and its rows.", token: "space-s8" },
      { what: "Between a flow's chips, both ways.", token: "space-s8" },
      { what: "Between Kōbō's two columns.", token: "space-s20" },
      { what: "A wallpaper's shape in its cell.", token: "aspect-wallpaper" },
      { what: "The room most samples are given, a column of text.", token: "size-column" },
    ],
    reflow: {
      described:
        "In Settings' page at the toolbar's window size: the wallpapers keep three columns, each a third of the page less the gaps, from 131 wide at the minimum to over 500 on a 1080p screen; the split keeps its quarter; the theme's chips flow onto a second line where the page is narrow.",
      sample: settingsWindow,
    },
  },
});
