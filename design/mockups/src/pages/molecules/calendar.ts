// The Calendar page: `docs/design/components/calendar.md`. A month as six weeks, the clock's: today
// filled, the picked day ringed, holidays dotted, the months around dimmed. The month
// is one stop for Tab, whose arrows move by the day and the week; a new month slides in.

import { type CalendarOptions, calendar, type Day, type DayLook, dayCell, monthCell, titlePill } from "../../lib/calendar.ts";
import type { Words } from "../../lib/content.ts";
import { type Pinned } from "../../lib/control.ts";
import { filling, flow, stack, within } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

const TODAY: Day = { year: 2026, month: 10, day: 27 };
const PICKED: Day = { year: 2026, month: 10, day: 30 };

/** Poland's public holidays around the month shown, as the clock's settings pick them. */
const HOLIDAYS: Record<string, string> = {
  "2026-11-01": "All Saints' Day",
  "2026-11-11": "National Independence Day",
  "2026-12-24": "Christmas Eve",
  "2026-12-25": "Christmas Day",
  "2026-12-26": "Second Day of Christmas",
  "2027-01-01": "New Year's Day",
  "2027-01-06": "Epiphany",
};

const HOLIDAY: Words = { short: "Epiphany", long: "National Independence Day" };
const TITLE: Words = { short: "May 2026", long: "September 2026" };

function month(extra: Partial<CalendarOptions> = {}): HTMLElement {
  return calendar({ shown: { year: 2026, month: 10 }, today: TODAY, picked: PICKED, holidays: HOLIDAYS, ...extra });
}

/** A year view's month alone, as wide as one of its three columns. */
function aMonth(name: string, shown: boolean, pinned?: Pinned): HTMLElement {
  return within("calc((var(--size-control_sm) + 7 * var(--size-control_md) - 2 * var(--space-s8)) / 3)", monthCell(name, shown, pinned));
}

const day = (d: number, m = 10): Day => ({ year: 2026, month: m, day: d });

const ROWS: readonly { readonly name: string; readonly cell: (pinned?: Pinned) => HTMLElement }[] = [
  { name: "A day", cell: (pinned) => dayCell({ day: day(14) }, pinned) },
  { name: "A weekend's", cell: (pinned) => dayCell({ day: day(17), weekend: true }, pinned) },
  { name: "Today", cell: (pinned) => dayCell({ day: TODAY, today: true }, pinned) },
  { name: "The picked day", cell: (pinned) => dayCell({ day: PICKED, picked: true }, pinned) },
  { name: "A holiday", cell: (pinned) => dayCell({ day: day(11, 11), holiday: HOLIDAYS["2026-11-11"]! }, pinned, false) },
  { name: "Another month's", cell: (pinned) => dayCell({ day: day(2, 11), other: true }, pinned) },
  { name: "A month, the year open", cell: (pinned) => aMonth("Mar", false, pinned) },
  { name: "The month shown", cell: (pinned) => aMonth("Oct", true, pinned) },
  { name: "The title", cell: (pinned) => titlePill("October 2026", false, pinned) },
  { name: "The title, the year open", cell: (pinned) => titlePill("2026", true, pinned) },
];

type MatrixState = "rest" | "lit" | "pressed";

const lookOf = (state: MatrixState): Pinned | undefined => (state === "rest" ? undefined : state);

/** The keyboard's day lit as `pinned`: the month with the keyboard in it. */
function keyed(pinned: Pinned = "lit", current: Day = day(15)): () => HTMLElement {
  return () => month({ current, pinned });
}

const at = (d: Day) => `.k-cal-day[data-date="${d.year}-${String(d.month).padStart(2, "0")}-${String(d.day).padStart(2, "0")}"]`;

/** A month sliding in from `from`: the next (1) or the last (-1). */
function sliding(name: string, from: 1 | -1, shown: { year: number; month: number }): Move {
  const side = from === 1 ? "right" : "left";
  return {
    name,
    what: `The weeks slide \`space.s64\` in from the ${side} as they fade, over \`short\` on \`emphasized_decel\`; the header stays.`,
    tracks: [
      { part: "the weeks' slide", on: "the month", target: ".k-cal-weeks", property: "--cal-slide", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
      { part: "the weeks' fade", on: "the month", target: ".k-cal-weeks", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    ],
    reduced: [{ part: "the weeks' fade", on: "the month", target: ".k-cal-weeks", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
    back: "cuts",
    sample: () => calendar({ shown, today: TODAY, picked: PICKED, holidays: HOLIDAYS, from }),
  };
}

export default page({
  level: "molecules",
  slug: "calendar",
  title: "Calendar",
  lede: "A month as six weeks led by their week numbers: today filled, the picked day ringed, holidays dotted.",
  code: { rust: "calendar::calendar", qml: "Calendar" },
  stage: {
    live: () => within(ACROSS, filling(flow("s48", month(), month({ year: true })))),
    hint: "press a day or the title, or Tab and the arrows",
    daylight: true,
  },
  variants: [
    { name: "Monday first", for: "ISO 8601's week, the clock's default", sample: () => month() },
    { name: "Sunday first", for: "the names and weeks follow; week numbers stay ISO's", sample: () => month({ firstWeekday: 0, locale: "en-US" }) },
    { name: "The year", for: "the title pressed: twelve months, the shown one filled", sample: () => month({ year: true }) },
    { name: "A holiday, lit", for: "its name in a hint under it", sample: () => calendar({ shown: { year: 2026, month: 11 }, today: TODAY, holidays: HOLIDAYS, current: day(11, 11) }) },
  ],
  sizes: doesntApply("one size: a small control's column and seven regular ones wide."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "every day can be picked",
      selected: "the picked day is a row here",
      loading: "the month is worked out before it shows",
      empty: "always six weeks",
      error: "a day can't be wrong",
    },
    rows: nonEmpty(ROWS.map(({ name, cell }) => ({ name, cell: (state: MatrixState) => cell(lookOf(state)) }))),
    looks: {
      rest: "only today filled, `primary`; holidays `tertiary`; other months dimmed",
      lit: "the tube inside the disc, glow at night",
      pressed: "white-hot, glow × `focus.pressed_glow`; by day a tint",
    },
  },
  focus: {
    tubes: [
      { name: "A day, a month, the title", tone: "accent", filled: false, content: false },
      { name: "Today, the month shown", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "the tube inside the disc, the core in today's, glow outside",
      signsOff: "the tube as `primary_glass`, no glow; the picked ring stays",
    },
    signsOff: (index: number) => ROWS[index]!.cell("signs-off"),
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => titlePill(words, false) },
      {
        name: "A holiday's name",
        words: HOLIDAY,
        sample: (words: string) => stack(0, "start", dayCell({ day: day(6, 1), holiday: words } satisfies DayLook, "lit")),
      },
    ],
    missingPicture: doesntApply("it draws no picture."),
    counts: [
      { name: "Four weeks of its own", for: "February 2027: two of March's keep the height", sample: () => calendar({ shown: { year: 2027, month: 2 }, today: TODAY, holidays: HOLIDAYS }) },
      { name: "Across six weeks", for: "August 2026 takes all six", sample: () => calendar({ shown: { year: 2026, month: 8 }, today: TODAY, holidays: HOLIDAYS }) },
      { name: "No holidays", for: "no dots, no hints", sample: () => calendar({ shown: { year: 2026, month: 10 }, today: TODAY, picked: PICKED }) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with the clock popup."),
    leaves: doesntApply("it goes with the clock popup."),
    between: [
      sliding("The next month", 1, { year: 2026, month: 11 }),
      sliding("The last month", -1, { year: 2026, month: 9 }),
      {
        name: "Opening the year",
        what: "The weeks fade out as the months fade in, over `short` on `emphasized_decel`; the title turns Tonal at once.",
        tracks: [{ part: "the month and the year", on: "the calendar", target: ".k-calendar", property: "--year", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
        reduced: "same",
        back: "plays",
        sample: () => month({ year: true }),
      },
      {
        name: "The keyboard moves a day",
        what: "The day reached lights over `short` on `standard` as the one left goes back to rest.",
        tracks: [
          { part: "the day left", on: "two days", target: at(day(15)), property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the day reached", on: "two days", target: at(day(16)), property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => month(),
      },
      { name: "Pressed", what: "The tube runs white-hot and the glow brightens over `micro`, and back as it comes up.", tracks: press(at(day(15)), "the keyboard's day"), reduced: "same", back: "plays", sample: keyed() },
      {
        name: "Picking",
        what: "The new ring fades in over `short` on `standard` as the last one fades out.",
        tracks: [
          { part: "the last pick", on: "two days", target: at(PICKED), property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the new pick", on: "two days", target: at(day(22)), property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => month(),
      },
      { name: "The keyboard comes to the month", what: "The keyboard's day lights up over `short` on `standard`.", tracks: lights(at(PICKED), "the keyboard's day", 0, 1), reduced: "same", back: "cuts", sample: () => month() },
      { name: "Focus leaves", what: "Its day goes back to rest over `short` on `standard`.", tracks: lights(at(day(15)), "the keyboard's day", 1, 0), reduced: "same", back: "cuts", sample: keyed() },
      ...powerMoves(keyed("lit", TODAY)),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "→", "h", "l"], does: "a day; crossing a month slides it in" },
      { keys: ["↑", "↓", "k", "j"], does: "a week" },
      { keys: ["Page Up", "Page Down"], does: "a month, held to its last day" },
      { keys: ["Home", "End"], does: "the week's first or last day" },
      { keys: ["Enter", "Space"], does: "picks the keyboard's day" },
      { keys: ["Tab", "Shift+Tab"], does: "Last month, the title, Next month, then the month" },
      { keys: ["Esc"], does: "year open: back to the month; else the popup's" },
    ],
    order: "The arrows and the title, then the month as one stop; with the year open, its months are that stop.",
    lands: "On the picked day, else today; a pick keeps the keyboard there.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The month", for: "header, names, week numbers, days, a disc, a dot", sample: () => month() },
      { name: "The year", for: "three months to a row, and their gaps", sample: () => month({ year: true }) },
    ]),
    measures: [
      { part: "Day, year's month", token: "size-control_md" },
      { part: "Disc, header, week column", token: "size-control_sm" },
      { part: "Names' row", token: "space-s24" },
      { part: "Header to month, year's gaps", token: "space-s8" },
      { part: "Disc inset, holiday dot", token: "space-s4" },
      { part: "Title inset", token: "space-s12" },
      { part: "New month's slide", token: "space-s64" },
      { part: "Day number line", token: "type-body-line" },
      { part: "Title line", token: "type-subheading-line" },
      { part: "Disc, arrows' corner", token: "radius-full" },
      { part: "Title, year's month corner", token: "radius-control" },
    ],
    reflow: doesntApply("one size at every window size."),
  },
});
