// The BadgedIcon page: `docs/design/components/badge.md`, the badged symbol. A symbol with a badge
// centred on its top right corner, a count or a dot, which takes the room it hangs past the symbol
// on both sides so the symbol stays centred.

import { BADGED_SIZES, type BadgedOptions, type BadgedSize, badgedIcon } from "../../lib/badge.ts";
import { filling, flow, row, within } from "../../lib/layout.ts";
import { pill } from "../../lib/pill.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** What each size sits beside. */
const BESIDE: Record<BadgedSize, string> = {
  s16: "the bar's, in a pill",
  s20: "beside `body` or `body_strong`",
  s24: "beside `subheading`, or a two-line row",
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
  lede: "A symbol with a badge on its top right corner: how many wait, or a dot that something does.",
  code: { qml: "BadgedIcon", unbuilt: "Rust: not built yet" },
  stage: {
    live: () => within("100%", filling(flow("s32", bar(), counts("s16"), counts("s20"), counts("s24")))),
    hint: "a mark: nothing to press",
    daylight: true,
  },
  variants: [
    { name: "A count", for: "how many: security updates waiting", sample: () => badgedIcon({ name: "system_update_alt", badged: true, count: 3 }) },
    { name: "A dot", for: "new or wrong, no number: a failed proxy", sample: () => badgedIcon({ name: "vpn_lock", badged: true }) },
    { name: "In the accent", for: "unread, not wrong", sample: () => badgedIcon({ name: "notifications", badged: true, accent: true }) },
    { name: "No badge", for: "the symbol alone, its box as wide", sample: () => badgedIcon({ name: "notifications" }) },
  ],
  sizes: nonEmpty(BADGED_SIZES.map((size) => ({ name: size.slice(1), for: BESIDE[size], sample: () => counts(size) }))),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "a mark: its pill or row lights",
      pressed: "takes no press",
      disabled: "dims with what it sits in",
      selected: "holds no choice",
      loading: "no badge until the count is known",
      error: "the badge's colour already says it",
    },
    rows: nonEmpty(BADGED_SIZES.map((size) => ({ name: size.slice(1), cell: (state: "rest" | "empty") => badgedIcon({ name: "notifications", size, ...(state === "rest" ? { badged: true, count: 3 } : {}) }) }))),
    looks: {
      rest: "`error` pill with `on_error`, or `primary` with `on_primary`",
      empty: "the symbol alone, no room for a badge",
    },
  },
  focus: doesntApply("it takes no input and has no line: its pill or row lights."),
  content: {
    text: doesntApply("its only words are the count, at most three characters: 99+."),
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, always there."),
    counts: nonEmpty(COUNTS.map(({ name, options }) => ({ name, for: options.badged ? (options.count === undefined ? "`count` 0" : `the count, ${String(options.count)}`) : "`badged` off", sample: () => badgedIcon({ name: "notifications", ...options }) }))),
  },
  motion: {
    arrives: doesntApply("it shows at once, its box taking its room; a pill eases to its new width."),
    leaves: doesntApply("it goes at once, on the frame its count ends."),
    between: powerMoves(() => counts("s16")),
  },
  keyboard: doesntApply("it takes no input; the bar's actions are Items in the Command menu."),
  measures: {
    samples: nonEmpty([
      { name: "A count at 16", for: "symbol, badge on its corner, room both sides", sample: () => badgedIcon({ name: "notifications", badged: true, count: 3 }) },
      { name: "99+ at 24", for: "the widest badge on the largest symbol", sample: () => badgedIcon({ name: "notifications", size: "s24", badged: true, count: 128 }) },
    ]),
    measures: [
      { part: "The bar's symbol", token: "icon-s16" },
      { part: "A row's symbol", token: "icon-s20" },
      { part: "A two-line row's symbol", token: "icon-s24" },
      { part: "Count height, one-digit width", token: "size-badge" },
      { part: "Dot", token: "size-badge_dot" },
      { part: "Beside a longer count", token: "space-s4" },
    ],
    reflow: doesntApply("its box is the symbol's and the badge's, at every window size."),
  },
});
