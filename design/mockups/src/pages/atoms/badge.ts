// The Badge page, with BadgedIcon: `docs/design/components/badge.md`. What is new about something,
// centred on its icon's top right corner: a count in a pill, or a dot, in `error`, or in `primary`
// where it marks something unread; in another fixed meaning, only a dot. A mark, not a control: it
// rests, by night and by day, and never lights.

import { badge, badgedIcon, MEANINGS } from "../../lib/badge.ts";
import { row } from "../../lib/layout.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The bar's symbols as the QML specimen shows them: none, a dot, a count, a count past 99. */
function bar(): HTMLElement {
  return row(
    "s24",
    "centre",
    badgedIcon({ name: "notifications" }),
    badgedIcon({ name: "notifications", badged: true }),
    badgedIcon({ name: "notifications", badged: true, count: 3 }),
    badgedIcon({ name: "notifications", badged: true, count: 128 }),
  );
}

/** A dot in each fixed meaning, which holds no count. */
function meanings(): HTMLElement {
  return row("s16", "centre", ...MEANINGS.map((meaning) => badge({ meaning })));
}

/** Every badge alone: counts, a dot, and the same in the accent. */
function badges(): HTMLElement {
  return row("s24", "centre", badge({ count: 3 }), badge({ count: 128 }), badge(), badge({ count: 3, accent: true }), badge({ accent: true }));
}

export default page({
  level: "atoms",
  slug: "badge",
  title: "Badge and BadgedIcon",
  lede: "What is new about something, on its icon's top right corner: a count or a dot.",
  code: { rust: "badge::badge, badge::accent_badge", qml: "Badge, BadgedIcon" },
  stage: { live: () => row("s48", "centre", bar(), badges(), meanings()), hint: "a mark, not a control: nothing to press", daylight: true },
  variants: [
    { name: "Count", for: "how many: security updates waiting", sample: () => row("s16", "centre", badge({ count: 3 }), badge({ count: 42 }), badge({ count: 128 })) },
    { name: "Dot", for: "new or wrong, with no number to give", sample: () => badge() },
    { name: "Accent", for: "unread, not wrong: `primary` under `on_primary`", sample: () => row("s16", "centre", badge({ count: 3, accent: true }), badge({ accent: true })) },
    { name: "In a meaning", for: "only a dot: a share, the microphone, a recording", sample: meanings },
    { name: "BadgedIcon", for: "the bar's symbol with its badge: QML only", sample: bar },
  ],
  sizes: [
    { name: "Count", for: "`size.badge` tall, one digit a circle", sample: () => row("s16", "centre", badge({ count: 3 }), badge({ count: 99 })) },
    { name: "Dot", for: "`size.badge_dot` across", sample: () => badge() },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "what it sits on lights, never the badge",
      pressed: "it takes no press",
      disabled: "on something disabled it still says what is new",
      selected: "it holds no choice",
      loading: "no badge until the count comes",
      empty: "nothing new is no badge",
      error: "`error` is its default tone, not a state",
    },
    rows: [
      { name: "One digit", cell: () => badge({ count: 3 }) },
      { name: "Two digits", cell: () => badge({ count: 42 }) },
      { name: "Past 99", cell: () => badge({ count: 128 }) },
      { name: "Dot", cell: () => badge() },
      { name: "Accent, one digit", cell: () => badge({ count: 3, accent: true }) },
      { name: "Accent dot", cell: () => badge({ accent: true }) },
      ...MEANINGS.map((meaning) => ({ name: `${meaning[0]!.toUpperCase()}${meaning.slice(1)} dot`, cell: () => badge({ meaning }) })),
      { name: "BadgedIcon, a count", cell: () => badgedIcon({ name: "notifications", badged: true, count: 3 }) },
    ],
    looks: {
      rest: "`error` under `on_error`, `label` figures; no line, no glow",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow."),
  content: {
    text: doesntApply("a count, never words: its extremes are under Counts."),
    missingPicture: doesntApply("a badge is drawn; `BadgedIcon`'s symbol is an `Icon`."),
    counts: [
      { name: "None", for: "no badge; the box is the symbol's own", sample: () => badgedIcon({ name: "notifications" }) },
      { name: "A dot", for: "`count` 0", sample: () => badgedIcon({ name: "system_update_alt", badged: true }) },
      { name: "One", for: "one digit: a circle", sample: () => badgedIcon({ name: "notifications", badged: true, count: 1 }) },
      { name: "Many", for: "two digits: the pill grows `space.s4` a side", sample: () => badgedIcon({ name: "notifications", badged: true, count: 99 }) },
      { name: "Past 99", for: "99+, the widest it gets", sample: () => badgedIcon({ name: "notifications", badged: true, count: 128 }) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows at once, on the frame its count arrives."),
    leaves: doesntApply("it goes at once, when nothing is new."),
    between: powerMoves(() => row("s24", "centre", bar(), badge({ count: 3, accent: true }), meanings())),
  },
  keyboard: doesntApply("no focus and no key: what it sits on is read out with its count."),
  measures: {
    samples: nonEmpty([
      { name: "One digit", for: "a circle", sample: () => badge({ count: 3 }) },
      { name: "Past 99", for: "figures and side room", sample: () => badge({ count: 128 }) },
      { name: "Dot", for: "its side", sample: () => badge() },
      { name: "BadgedIcon", for: "the symbol, the badge's half past it", sample: () => badgedIcon({ name: "notifications", badged: true, count: 3 }) },
    ]),
    measures: [
      { part: "Height, one-digit width", token: "size-badge" },
      { part: "Dot's side", token: "size-badge_dot" },
      { part: "Room beside two figures", token: "space-s4" },
      { part: "Figures' line", token: "type-label-line" },
      { part: "BadgedIcon's symbol", token: "icon-s16" },
    ],
    reflow: doesntApply("its size at every window size."),
  },
});
