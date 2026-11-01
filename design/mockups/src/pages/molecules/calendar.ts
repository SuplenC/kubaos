// The Calendar page: `docs/design/components/calendar.md`. A month as six weeks, the clock's: today
// filled, the picked day ringed, holidays dotted, the months around dimmed. The month
// is one stop for Tab, whose arrows move by the day and the week; a new month slides in.

import { type CalendarOptions, calendar, type Day, type DayLook, dayCell, monthCell, titlePill } from "../../lib/calendar.ts";
import type { Words } from "../../lib/content.ts";
import { type Pinned } from "../../lib/control.ts";
import { stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

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
    what: `An arrow, the wheel, Page ${from === 1 ? "Down" : "Up"} or the keyboard's day crossing into it: the new month's weeks slide in from the ${side}, \`space.s64\` to their place, as they fade in, over \`short\` on \`emphasized_decel\`, cut at the month's edge. The names, the header and the arrows stay; the title changes at once. With reduced motion the weeks fade in where they are.`,
    tracks: [
      { part: "the weeks' slide", on: "the month", target: ".k-cal-weeks", property: "--slide", from: 1, to: 0, duration: "short", curve: "emphasized_decel" },
      { part: "the weeks' fade", on: "the month", target: ".k-cal-weeks", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    ],
    reduced: [{ part: "the weeks' fade", on: "the month", target: ".k-cal-weeks", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
    back: "cuts",
    sample: () => calendar({ shown, today: TODAY, picked: PICKED, holidays: HOLIDAYS, from }),
  };
}

const DESCRIBED =
  "Under the header, the weekdays' names, `space.s24` tall, in `label`: two letters each, the weekend's in `tertiary`, the rest `on_surface_variant`, after `Wk` over the week numbers. Then six weeks, so every month is as tall as every other, each led by its ISO week number (the week's Thursday's) in `caption`, `on_surface_variant`, in a column a small control wide (`size.control_sm`). A day is a regular control (`size.control_md`, 40) holding a small control's disc (`size.control_sm`, 32), a pill, `space.s4` in, its number in `body` centred on whole device pixels: `on_surface`, a weekend's `on_surface_variant`, a holiday's `tertiary`. The days of the months around are the same at `opacity.disabled_content`.";

export default page({
  level: "molecules",
  slug: "calendar",
  title: "Calendar",
  summary: "A month as six weeks led by their week numbers: today filled, the picked day ringed, holidays dotted, the months around dimmed.",
  live: () => stack("s8", "start", month(), text("Tab into the month: the arrows move a day or a week.", "small", "on_surface_variant")),
  description: [
    "Rust `calendar::calendar`, QML `Calendar`. Press a day, an arrow or the title; or Tab to the month and use the arrow keys, Page Up and Page Down, Home and End, Enter. The wheel moves a month. The clock popup's.",
    "The header, a small control tall: the arrows at either end, Text icon buttons (sm, `chevron_left` and `chevron_right`, hinted Last month and Next month), and between them the title, the month and year in `subheading`, `on_surface`, on a pill `space.s12` inside, centred on whole device pixels. The title opens the year; the month is `space.s8` under the header.",
    DESCRIBED,
    "Today's disc is filled with `primary` and its number is `body_strong` in `on_primary`. The picked day's disc is ringed with `primary` in the thin stroke (`stroke.thin`, whole device pixels), its number as it was; picking today shows today. A holiday has a `tertiary` dot, `space.s4` across, `space.s4` above the disc's foot (`on_primary` on today), and its name in a hint under the day while it is lit. The first weekday is the clock's setting, Monday by default as ISO 8601 has it, and the names are the locale's.",
    "Its days follow the neon, as a Text button does: lit (under the pointer, or the keyboard's day while the month has the keyboard), the tube burns inside the disc's edge, `primary` at `focus.tube`, with the glow outside at night; today's fill takes the core inside its edge. Pressed, the tube runs white-hot and the glow brightens, and by day the tone tints the disc; letting go picks the day, and a day of another month shows that month. The pointer moving onto a day puts the keyboard there, so one day is lit at a time. The title and the year's months light the same.",
    "The year: the title turns Tonal (`secondary_container`, `on_secondary_container`) and reads the year; the twelve months, three to a row `space.s8` apart, regular controls at the control radius in `surface_container_high` with their short names in `body_strong`, the one shown filled with `primary`, sit in the month's room, the arrows moving a year. The month and the year cross-fade over `short` on `emphasized_decel`. A month picked shows it.",
  ],
  variants: [
    { name: "Monday first", for: "ISO 8601's week, most of Europe's, and the clock's default.", sample: () => month() },
    { name: "Sunday first", for: "The clock set to start on Sunday: the names and the weeks follow, the week numbers still ISO's.", sample: () => month({ firstWeekday: 0, locale: "en-US" }) },
    { name: "The year", for: "The title pressed: the twelve months, the one shown filled.", sample: () => month({ year: true }) },
    { name: "A holiday, lit", for: "The keyboard on Independence Day: its name in a hint under it.", sample: () => calendar({ shown: { year: 2026, month: 11 }, today: TODAY, holidays: HOLIDAYS, current: day(11, 11) }) },
  ],
  sizes: doesntApply("a calendar has one size: a small control's column and seven regular controls wide, the header and six weeks tall."),
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "every day can be picked, and the arrows always move: a calendar has no disabled state.",
      selected: "the picked day is a row of its own here, ringed.",
      loading: "the owner works the month out before it shows: the weeks, today and the holidays come at once.",
      empty: "a month always has its six weeks.",
      error: "a day can't be wrong.",
    },
    rows: nonEmpty(ROWS.map(({ name, cell }) => ({ name, cell: (state: MatrixState) => cell(lookOf(state)) }))),
    described: {
      rest: "No fill but today's `primary`; a day's number `on_surface`, a weekend's `on_surface_variant`, a holiday's `tertiary` with its dot, the picked day's disc ringed in `primary`, the months around at `opacity.disabled_content`. A month of the year `surface_container_high`, the one shown `primary`; the title bare, or Tonal with the year open. By day the same roles in daylight colours.",
      lit: "Under the pointer, or the keyboard's day, here with the signs on: the tube inside the disc's edge (today's core inside its fill), a month's or the title's inside theirs, with the glow outside; a holiday shows its name in a hint (A holiday, lit, in the variants). By day the tube, or a dark line, with no glow.",
      pressed: "Lit and pressed: the tube runs white-hot and the glow brightens to `focus.pressed_glow` times the lit one; by day the tone tints the disc. Letting go picks it.",
    },
  },
  focus: {
    tubes: [
      { name: "A day, a month of the year, the title", tone: "accent", filled: false, content: false },
      { name: "Today, the month shown", tone: "accent", filled: true, content: false },
    ],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the disc's edge, the core inside today's fill, the glow outside. On the picked day the tube lies over its ring.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. The tube is `primary_glass`, today's line the fill's glass, with no glow; the picked day's ring stays `primary`.",
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
    missingPicture: doesntApply("a calendar draws no picture: its arrows are always there."),
    counts: [
      { name: "Four weeks of its own", for: "February 2027 starts on a Monday: four weeks, then two of March's, so the month keeps its height.", sample: () => calendar({ shown: { year: 2027, month: 2 }, today: TODAY, holidays: HOLIDAYS }) },
      { name: "Across six weeks", for: "August 2026 starts on a Saturday and ends on a Monday: it takes all six.", sample: () => calendar({ shown: { year: 2026, month: 8 }, today: TODAY, holidays: HOLIDAYS }) },
      { name: "No holidays", for: "The holidays set to none: no dots and no hints.", sample: () => calendar({ shown: { year: 2026, month: 10 }, today: TODAY, picked: PICKED }) },
    ],
  },
  motion: {
    arrives: doesntApply("a calendar comes with the clock popup it is on."),
    leaves: doesntApply("it goes with the clock popup."),
    between: [
      sliding("The next month", 1, { year: 2026, month: 11 }),
      sliding("The last month", -1, { year: 2026, month: 9 }),
      {
        name: "Opening the year",
        what: "The title pressed: the weeks fade out as the twelve months fade in, over `short` on `emphasized_decel`, and the title turns Tonal and reads the year at once. Closing (a month picked, the title again, Esc) plays it back. Reduced motion keeps the fade, which is within its cap.",
        tracks: [{ part: "the month and the year", on: "the calendar", target: ".k-calendar", property: "--year", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
        reduced: "same",
        back: "plays",
        sample: () => month({ year: true }),
      },
      {
        name: "The keyboard moves a day",
        what: "→, or the pointer onto the next day: it lights up over `short` on `standard` as the day it left goes back to rest. Never the flicker.",
        tracks: [
          { part: "the day left", on: "two days", target: at(day(15)), property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the day reached", on: "two days", target: at(day(16)), property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => month(),
      },
      { name: "Pressed", what: "A lit day pressed: its tube runs white-hot and the glow brightens over `micro`, and back as it comes up; letting go picks it.", tracks: press(at(day(15)), "the keyboard's day"), reduced: "same", back: "plays", sample: keyed() },
      {
        name: "Picking",
        what: "Enter, Space or a press on a day: its ring fades in over `short` on `standard` as the last pick's fades out.",
        tracks: [
          { part: "the last pick", on: "two days", target: at(PICKED), property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the new pick", on: "two days", target: at(day(22)), property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => month(),
      },
      { name: "The keyboard comes to the month", what: "Tab into the month: the keyboard's day, here the picked one, lights up over `short` on `standard`.", tracks: lights(at(PICKED), "the keyboard's day", 0, 1), reduced: "same", back: "cuts", sample: () => month() },
      { name: "Focus leaves", what: "The keyboard leaves the month: its day goes back to rest over `short` on `standard`.", tracks: lights(at(day(15)), "the keyboard's day", 1, 0), reduced: "same", back: "cuts", sample: keyed() },
      ...powerMoves(keyed("lit", TODAY)),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "→", "h", "l"], does: "The day before or after, into the month around where it crosses, which slides in." },
      { keys: ["↑", "↓", "k", "j"], does: "The same day a week before or after." },
      { keys: ["Page Up", "Page Down"], does: "The same day a month before or after, held to the month's last." },
      { keys: ["Home", "End"], does: "The first or the last day of the week, by the first weekday." },
      { keys: ["Enter", "Space"], does: "Picks the keyboard's day, once per press." },
      { keys: ["Tab", "Shift+Tab"], does: "Last month, the title, Next month, then the month: one stop, on its keyboard's day." },
      { keys: ["Esc"], does: "With the year open, back to the month; otherwise the clock popup's." },
    ],
    order: "The two arrows and the title between them, then the month as one stop. With the year open, its twelve months are one stop in the month's place: ← → a month, ↑ ↓ three, Enter picks.",
    lands: "Tabbing into the month, on the picked day, else today. A day picked keeps the keyboard there; a month picked in the year lands on its day.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The month", for: "The header, the names, the week numbers' column and the days, each day's disc and a holiday's dot.", sample: () => month() },
      { name: "The year", for: "Three months to a row and their gaps.", sample: () => month({ year: true }) },
    ]),
    measures: [
      { what: "A day, and a month of the year: a regular control.", token: "size-control_md" },
      { what: "A day's disc, the header's height and the week numbers' column: a small control.", token: "size-control_sm" },
      { what: "The names' row.", token: "space-s24" },
      { what: "Between the header and the month, and between the year's months.", token: "space-s8" },
      { what: "A disc in from its day's edge, and a holiday's dot, across and above the disc's foot.", token: "space-s4" },
      { what: "Inside the title's sides.", token: "space-s12" },
      { what: "How far a new month slides.", token: "space-s64" },
      { what: "A day's number, `body`'s line.", token: "type-body-line" },
      { what: "The title, `subheading`'s line.", token: "type-subheading-line" },
      { what: "A day's disc, the title and the arrows' corners.", token: "radius-full" },
      { what: "A month of the year's corners.", token: "radius-control" },
    ],
    reflow: doesntApply("a calendar is one size at every window size: a small control and seven regular ones wide, the header and six weeks tall."),
  },
});
