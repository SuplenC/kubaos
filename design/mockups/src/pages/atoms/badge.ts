// The Badge page, with BadgedIcon: `docs/design/components/badge.md`. What is new about something,
// over its icon's top right corner: a count in a pill, or a dot, in `error`, or in `primary` where
// it marks something unread. A mark, not a control: it rests, by night and by day, and never lights.

import { badge, badgedIcon } from "../../lib/badge.ts";
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

/** Every badge alone: counts, a dot, and the same in the accent. */
function badges(): HTMLElement {
  return row("s24", "centre", badge({ count: 3 }), badge({ count: 128 }), badge(), badge({ count: 3, accent: true }), badge({ accent: true }));
}

const DESCRIBED =
  "A count in a pill `size.badge` tall at the full radius, its figures `label` type centred across it on a whole device pixel: a circle for one digit, `space.s4` either side for more, and 99+ past 99. Or a dot, `size.badge_dot` across. In `error` with `on_error` on it, so it reads over any icon; in `primary` with `on_primary` where it marks something unread. It never lights: no line, no glow.";

export default page({
  level: "atoms",
  slug: "badge",
  title: "Badge and BadgedIcon",
  summary: "What is new about something, over its icon's top right corner: a count, or a dot, in the error colour or the accent.",
  live: () => row("s48", "centre", bar(), badges()),
  description: [
    "Rust `badge::badge` and `badge::accent_badge`, QML `Badge` (`count`, 0 for a dot; `accent`). It says what is new about something: waiting updates, a proxy that failed, an unread notification.",
    DESCRIBED,
    "The error badge says something wants attention: updates waiting, something wrong. The accent badge marks something unread rather than something wrong (an unread notification's dot on its app's tile), since red there would make every new message look like a failure.",
    "`BadgedIcon` is the bar's: QML only, on the QML specimen's \"Shell only\" page. A 16 symbol in `on_surface_variant` with a badge centred on its top right corner while `badged`: a count, or a dot when `count` is 0. Its box is the symbol's height, and the badge hangs past its top; while the badge shows, the box takes the half of the badge past the symbol as room on both sides, so the badge keeps the row's gap to the symbol beside it and the symbol stays centred in a pill of its own.",
  ],
  variants: [
    { name: "Count", for: "How many: security updates waiting.", sample: () => row("s16", "centre", badge({ count: 3 }), badge({ count: 42 }), badge({ count: 128 })) },
    { name: "Dot", for: "Something is new or wrong, with no number to give: a release waiting, a proxy that failed.", sample: () => badge() },
    { name: "Accent", for: "Something unread, not wrong: an unread notification.", sample: () => row("s16", "centre", badge({ count: 3, accent: true }), badge({ accent: true })) },
    { name: "BadgedIcon", for: "The bar's symbol with its badge on the corner: QML only.", sample: bar },
  ],
  sizes: [
    { name: "Count", for: "`size.badge` tall, a line of `label` text: a circle for one digit, wider for more.", sample: () => row("s16", "centre", badge({ count: 3 }), badge({ count: 99 })) },
    { name: "Dot", for: "`size.badge_dot` across.", sample: () => badge() },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "a badge takes no pointer and no focus; the icon or tile it sits on is what lights.",
      pressed: "it takes no press.",
      disabled: "a badge on something disabled still says what is new, unchanged.",
      selected: "it holds no choice.",
      loading: "a count still on its way shows no badge until it comes.",
      empty: "nothing new is no badge: `BadgedIcon` with `badged` off is the symbol alone, in a box as wide as the symbol.",
      error: "the error badge is its default tone, not a state: a badge itself can't be wrong.",
    },
    rows: [
      { name: "One digit", cell: () => badge({ count: 3 }) },
      { name: "Two digits", cell: () => badge({ count: 42 }) },
      { name: "Past 99", cell: () => badge({ count: 128 }) },
      { name: "Dot", cell: () => badge() },
      { name: "Accent, one digit", cell: () => badge({ count: 3, accent: true }) },
      { name: "Accent dot", cell: () => badge({ accent: true }) },
      { name: "BadgedIcon, a count", cell: () => badgedIcon({ name: "notifications", badged: true, count: 3 }) },
    ],
    described: {
      rest: `Always. ${DESCRIBED} By day the same roles in daylight colours: the day's red and accent with white on them.`,
    },
  },
  focus: doesntApply("a badge isn't a control and isn't neon: it has no line and no glow, by night or by day."),
  content: {
    text: doesntApply("a badge holds a count, never words: its extremes are one digit, two, and past 99 under Counts."),
    missingPicture: doesntApply("a badge is drawn; `BadgedIcon`'s symbol is an `Icon`, whose page shows a name with no drawing."),
    counts: [
      { name: "None", for: "Nothing new: no badge, and `BadgedIcon`'s box is the symbol's own.", sample: () => badgedIcon({ name: "notifications" }) },
      { name: "A dot", for: "Something new with no number: `count` 0.", sample: () => badgedIcon({ name: "system_update_alt", badged: true }) },
      { name: "One", for: "One digit: a circle.", sample: () => badgedIcon({ name: "notifications", badged: true, count: 1 }) },
      { name: "Many", for: "Two digits: the pill grows, `space.s4` either side.", sample: () => badgedIcon({ name: "notifications", badged: true, count: 99 }) },
      { name: "Past 99", for: "99+: the widest a badge gets.", sample: () => badgedIcon({ name: "notifications", badged: true, count: 128 }) },
    ],
  },
  motion: {
    arrives: doesntApply("a badge shows at once, on the frame its count arrives: nothing moves, and the `BadgedIcon`'s box takes its room at once."),
    leaves: doesntApply("a badge goes at once, when nothing is new."),
    between: powerMoves(() => row("s24", "centre", bar(), badge({ count: 3, accent: true }))),
  },
  keyboard: doesntApply("a badge takes no focus and no key: what it sits on is read out with its count."),
  measures: {
    samples: nonEmpty([
      { name: "One digit", for: "A circle: as wide as tall.", sample: () => badge({ count: 3 }) },
      { name: "Past 99", for: "The figures and `space.s4` either side.", sample: () => badge({ count: 128 }) },
      { name: "Dot", for: "Its side.", sample: () => badge() },
      { name: "BadgedIcon", for: "The symbol, and the badge's half past it as room on both sides.", sample: () => badgedIcon({ name: "notifications", badged: true, count: 3 }) },
    ]),
    measures: [
      { what: "A count's height, and a one-digit badge's width.", token: "size-badge" },
      { what: "A dot's side.", token: "size-badge_dot" },
      { what: "The room either side of two figures or more.", token: "space-s4" },
      { what: "The figures' line, which is the badge's height.", token: "type-label-line" },
      { what: "`BadgedIcon`'s symbol.", token: "icon-s16" },
    ],
    reflow: doesntApply("a badge is its size at every window size."),
  },
});
