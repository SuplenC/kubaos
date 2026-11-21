// Settings › Date & time: the time zone and how the clock keeps right, how times and dates are
// written, what the bar's clock shows, the clock popup's calendar, and the cities and their weather.
// It is the one place the clock's settings live. Its own parts are Kōbō's zone list on a page of its
// own, Kōbō's "How it reads" card, and the cities' list with the field that adds one.

import { screen, type Variant } from "../../../site/page.ts";
import type { DateTimeValues } from "./sections/date-time-parts.ts";
import { DATE_TIME } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { litOn, settingsWindow } from "./settings-window.ts";

/** Date & time's own values a sample starts with, over its defaults. */
const own = (values: Partial<DateTimeValues>) => ({ seed: { dateTime: values } });

/** The window on Date & time with its own `values`, the sub-page `opened` drawn open. */
const opened = (page: string, values: Partial<DateTimeValues> = {}) => screen(() => settingsWindow({ section: DATE_TIME, values: { opened: page, ...own(values) } }));

/** The keyboard on setting `id`'s row, with Date & time's own `values`. */
const lit = (id: string, values: Partial<DateTimeValues> = {}) => screen(() => settingsWindow(litOn(DATE_TIME, id, own(values))));

const TOKYO = ["Asia/Tokyo"];
const FOUR = ["Asia/Tokyo", "America/New_York", "Europe/London", "Atlantic/Reykjavik"];
const EIGHT = [...FOUR, "America/Sao_Paulo", "America/Los_Angeles", "Australia/Sydney", "Africa/Cairo"];

const VARIANTS: Variant[] = [
  { name: "Searching for a zone", for: "“tok”: the list narrows as it is typed", sample: opened("zone", { query: "tok" }) },
  { name: "No zone matches", for: "Kōbō's empty state in the list's place", sample: opened("zone", { query: "Atlantis" }) },
  { name: "Automatic time zone on", for: "Time zone dimmed, its reason its line", sample: lit("zone", { autoZone: true }) },
  { name: "Setting the time by hand", for: "Automatic time off: a field takes its place", sample: lit("set-time", { autoTime: false, setTime: "21:47" }) },
  { name: "A time it can't read", for: "the line says how to write it, in `error`", sample: lit("set-time", { autoTime: false, setTime: "tomorrow-ish" }) },
  { name: "Twelve hours, month first", for: "the card reads as the picks say", sample: lit("date-order", { timeFormat: 2, dateOrder: 2, firstWeekday: 6 }) },
  { name: "Matching Windows' clock", for: "a risky value: the warning under its well", sample: lit("match-windows", { matchWindows: true }) },
  { name: "No other cities", for: "the empty well over Add a city", sample: opened("cities") },
  { name: "One city", for: "Tokyo: its time there and how far ahead", sample: opened("cities", { cities: TOKYO }) },
  { name: "Eight cities", for: "the most: Add a city dimmed, its reason its line", sample: opened("cities", { cities: EIGHT }) },
  { name: "A city that isn't a zone", for: "Enter on “Atlantis”: the line says so, in `error`", sample: opened("cities", { cities: TOKYO, adding: "Atlantis", tried: true }) },
  { name: "Weather off", for: "its rows dimmed, the reason their line", sample: lit("weather", { weather: false }) },
  { name: "Only the time in the bar", for: "seal and date off", sample: lit("date", { seal: false, date: false }) },
];

export default sectionPage({
  from: ["card", "empty-state"],
  index: DATE_TIME,
  slug: "settings-date-time",
  title: "Settings: Date & time",
  lede: "The time zone, how times and dates read, the bar's clock, the calendar, and the cities and their weather.",
  hint: "change a row; hold a reset to put it back",
  seed: own({ timeFormat: 1, dateOrder: 1, holidays: 4, cities: FOUR }),
  variants: VARIANTS,
  keys: [
    { keys: ["↓"], does: "in the zone search: into the zones" },
    { keys: ["Delete"], does: "on a city: removes it" },
    { keys: ["Enter"], does: "in Add a city: adds the city typed" },
    { keys: ["Esc"], does: "in Add a city: clears it, then back to its row" },
  ],
  measures: [
    { part: "Zone search to its list", token: "space-s16" },
    { part: "The card's facts", token: "space-s8" },
  ],
});
