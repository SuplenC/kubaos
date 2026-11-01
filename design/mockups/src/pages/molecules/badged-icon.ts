// The BadgedIcon page: `docs/design/components/badge.md`, the badged symbol. A symbol with a badge
// centred on its top right corner, a count or a dot, which takes the room it hangs past the symbol
// on both sides so the symbol stays centred.

import { BADGED_SIZES, type BadgedOptions, type BadgedSize, badgedIcon } from "../../lib/badge.ts";
import { row } from "../../lib/layout.ts";
import { pill } from "../../lib/pill.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** What each size sits beside. */
const BESIDE: Record<BadgedSize, string> = {
  s16: "The bar's: `button_small`'s symbol in a pill a small control tall.",
  s20: "Beside `body` or `body_strong`: a row's symbol, a regular button's.",
  s24: "Beside `subheading` or a two-line row's title over its subtitle.",
};

/** The bar's group of symbols in a pill: updates, the proxy, notifications. */
function bar(): HTMLElement {
  return pill({
    lone: false,
    content: [
      badgedIcon({ name: "system_update_alt", badged: true, count: 3 }),
      badgedIcon({ name: "vpn_lock", badged: true }),
      badgedIcon({ name: "notifications", badged: true, accent: true }),
    ],
  });
}

const COUNTS: readonly { name: string; options: Partial<BadgedOptions> }[] = [
  { name: "No badge", options: {} },
  { name: "A dot", options: { badged: true } },
  { name: "One digit", options: { badged: true, count: 3 } },
  { name: "Two digits", options: { badged: true, count: 42 } },
  { name: "Past 99", options: { badged: true, count: 128 } },
];

/** Every count at `size`, side by side `space.s16` apart: each box its own width. */
function counts(size: BadgedSize): HTMLElement {
  return row("s16", "centre", ...COUNTS.map(({ options }) => badgedIcon({ name: "notifications", size, ...options })));
}

export default page({
  level: "molecules",
  slug: "badged-icon",
  title: "BadgedIcon",
  summary: "A symbol with a badge centred on its top right corner: how many wait, or a dot that something does.",
  live: () => row("s32", "centre", bar(), counts("s20")),
  description: [
    "QML `BadgedIcon`; the bar's. `kubaos-ui` gets it as a widget of its own with the first app that badges a symbol, since its badge hangs past the top of its box, which iced lays out only inside it; until then it is on the QML specimen's \"Shell only\" page. The bar's updates (how many security updates wait, or a dot while only a release does), its proxy (a dot while something is wrong with it) and an unread notification (a dot in the accent).",
    "An `Icon` (`name`, `size`, `color`; 16 in `on_surface_variant` by default) with a `Badge` centred on its top right corner while `badged`: a count, or a dot when `count` is 0, in `error`, or in `primary` where it marks something unread. Its centre is the symbol box's corner at every size, on a whole device pixel, so the badge covers the same share of a small symbol as of a large one's corner and never its middle.",
    "Its box is the symbol's height: the badge hangs past its top by half the badge, and a row or a pill holding it leaves that room. While the badge shows, the box takes the half of the badge past the symbol as room on both sides, so the badge keeps the row's gap to the symbol beside it and the symbol stays centred in a pill of its own. A wider count (99+) widens the box by as much on both sides.",
    "A badge comes and goes at once, on the frame its count arrives or ends, and the box takes or gives back its room at once. It never lights: it is a mark, and the pill or row it sits in takes the pointer.",
  ],
  variants: [
    { name: "A count", for: "How many: security updates waiting.", sample: () => badgedIcon({ name: "system_update_alt", badged: true, count: 3 }) },
    { name: "A dot", for: "Something is new or wrong, with no number: a release waiting, a proxy that failed.", sample: () => badgedIcon({ name: "vpn_lock", badged: true }) },
    { name: "In the accent", for: "Something unread, not wrong: an unread notification.", sample: () => badgedIcon({ name: "notifications", badged: true, accent: true }) },
    { name: "No badge", for: "Nothing new: the symbol alone, in a box as wide as it.", sample: () => badgedIcon({ name: "notifications" }) },
  ],
  sizes: nonEmpty(BADGED_SIZES.map((size) => ({ name: size.slice(1), for: BESIDE[size], sample: () => counts(size) }))),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "it is a mark, not a control: the pill or row it sits in lights.",
      pressed: "it takes no press of its own.",
      disabled: "what it sits in dims it with its other content.",
      selected: "it holds no choice.",
      loading: "a count it waits for shows no badge until it is known.",
      error: "the badge itself says something is wrong: its colour is the error's.",
    },
    rows: nonEmpty(BADGED_SIZES.map((size) => ({ name: size.slice(1), cell: (state: "rest" | "empty") => badgedIcon({ name: "notifications", size, ...(state === "rest" ? { badged: true, count: 3 } : {}) }) }))),
    described: {
      rest: "The symbol with its badge on its corner: the error's pill, `on_error` on it; or the accent's, `on_primary`. The same by night and by day, in each mode's roles.",
      empty: "Nothing new: the symbol alone, its box as wide as the symbol, with no room for a badge.",
    },
  },
  focus: doesntApply("it takes no input and has no line: the pill or row it sits in lights."),
  content: {
    text: doesntApply("its only words are the badge's count, at most three characters: 99+."),
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, always there; QML draws a name with no drawing as its empty box, as the `Icon` page shows."),
    counts: nonEmpty(COUNTS.map(({ name, options }) => ({ name, for: options.badged ? (options.count === undefined ? "`count` 0." : `The count, ${String(options.count)}.`) : "`badged` off.", sample: () => badgedIcon({ name: "notifications", ...options }) }))),
  },
  motion: {
    arrives: doesntApply("a badge shows at once, on the frame its count arrives, and the box takes its room at once: a pill holding it eases to its new width as the `Pill` page shows."),
    leaves: doesntApply("it goes at once, on the frame its count ends."),
    between: powerMoves(() => row("s16", "centre", counts("s16"), text("A mark just recolours.", "small", "on_surface_variant"))),
  },
  keyboard: doesntApply("it takes no input; the bar's actions are Items in the Command menu."),
  measures: {
    samples: nonEmpty([
      { name: "A count at 16", for: "The symbol, the badge on its corner, and the room on both sides.", sample: () => badgedIcon({ name: "notifications", badged: true, count: 3 }) },
      { name: "99+ at 24", for: "The widest badge on the largest symbol.", sample: () => badgedIcon({ name: "notifications", size: "s24", badged: true, count: 128 }) },
    ]),
    measures: [
      { what: "The bar's symbol.", token: "icon-s16" },
      { what: "A row's symbol.", token: "icon-s20" },
      { what: "A two-line row's symbol.", token: "icon-s24" },
      { what: "A count's height, and a one-digit badge's width.", token: "size-badge" },
      { what: "A dot's side.", token: "size-badge_dot" },
      { what: "Either side of a count of more than one digit.", token: "space-s4" },
    ],
    reflow: doesntApply("its box is the symbol's and the badge's, at every window size."),
  },
});
