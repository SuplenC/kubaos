// Date & time: the time zone and how the clock keeps right, how times and dates are written, what
// the bar's clock shows, what the clock popup's calendar shows, and the cities and their weather.
// This section is the one place the clock's settings live; the Command menu's clock Items land on
// its rows. The format's, the calendar's, the bar clock's and the cities' settings are the clock's
// own, `clock.toml`, written through `kubaos-clock`, so the file stays the one truth everything
// reads. The zone and the time are the system's (systemd-timedated and systemd-timesyncd), which
// nothing in KubaOS writes yet.

import { COUNTDOWNS } from "../../../shell/clock-parts.ts";
import { ZONES, zoneName } from "../../../kobo/kobo-data.ts";
import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { addCity, citiesDetail, citiesList, type DateTimeValues, homeCities, readsCard, WEEKDAYS, whileDisabled, zonePicker } from "./date-time-parts.ts";
import { choice, fact, opens, pick, toggle } from "./kit.ts";

/** KubaOS's defaults, `clock.toml`'s with each line commented out; the zone is the one Kōbō set. */
const DEFAULTS: DateTimeValues = {
  autoZone: false,
  zone: ZONES.findIndex((zone) => zone.iana === "Europe/Warsaw"),
  query: "",
  autoTime: true,
  setTime: "",
  matchWindows: false,
  timeFormat: 0,
  dateOrder: 0,
  firstWeekday: 0,
  holidays: 0,
  weekNumbers: false,
  barCountdown: 0,
  seal: true,
  date: true,
  seconds: false,
  barWeather: true,
  weather: true,
  homeCity: 0,
  temperature: 0,
  wind: 0,
  cities: [],
  adding: "",
  tried: false,
};

const COUNTRIES = ["Auto (United Kingdom)", "None", "Germany", "Japan", "Poland", "United Kingdom", "United States"] as const;
const BAR_COUNTDOWNS = ["None", ...COUNTDOWNS.filter((each) => !each.passed).map((each) => each.name)];

/** A time typed by hand, as the clock reads one: `21:47`, `27 oct 21:47`, or `2026-10-27 21:47`. */
function timeProblem(value: string): string | null {
  const typed = value.trim();
  const time = String.raw`([01]?\d|2[0-3]):[0-5]\d`;
  const forms = [new RegExp(`^${time}$`), new RegExp(`^\\d{1,2} [a-z]{3,9} ${time}$`, "i"), new RegExp(`^\\d{4}-\\d{2}-\\d{2} ${time}$`)];
  return forms.some((form) => form.test(typed)) ? null : "Try 21:47, or 27 oct 21:47.";
}

/** Date & time, whole. */
export function dateTime(values: Values): SettingsSection {
  const v = own(values, "dateTime", () => ({ ...DEFAULTS }));
  /** `setting`, dimmed with its reason while the weather is off. */
  const weatherOnly = (setting: Setting) => whileDisabled(setting, () => (v.weather ? null : "The weather is off."));
  const zone: Setting = whileDisabled(
    opens("zone", "Time zone", "The city whose clock you live by.", "zone", () => zoneName(ZONES[v.zone]!), { unbuilt: true }),
    () => (v.autoZone ? "Set by Automatic time zone; turn it off to pick one." : null),
  );
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "schedule",
    title: "Date & time",
    accent: "時間",
    lede: "It's about time.",
    subs: [
      {
        title: "Time & zone",
        body: "Where you are, and how the clock stays right.",
        wells: [
          [
            toggle("auto-zone", "Automatic time zone", "Follows you when you travel.", () => v.autoZone, (on) => (v.autoZone = on), { unbuilt: true, drives: true, initial: () => DEFAULTS.autoZone }),
            zone,
          ],
          [
            toggle("auto-time", "Automatic time", "Asks a time server now and then, so the clock never drifts.", () => v.autoTime, (on) => (v.autoTime = on), { unbuilt: true, drives: true, initial: () => DEFAULTS.autoTime }),
            fact("synced", "Last checked", "With ntp.ubuntu.com, Ubuntu's time server.", "4 min ago", { shows: () => v.autoTime, unbuilt: true }),
            {
              id: "set-time",
              title: "Date and time",
              line: "As you would say it: 21:47, or 27 oct 21:47.",
              control: { kind: "field", value: () => v.setTime, placeholder: "21:47", check: timeProblem, set: (value) => (v.setTime = value) },
              shows: () => !v.autoTime,
              unbuilt: true,
            },
            toggle("match-windows", "Match Windows' clock", "If this machine also starts Windows, so neither is hours off.", () => v.matchWindows, (on) => (v.matchWindows = on), {
              unbuilt: true,
              initial: () => DEFAULTS.matchWindows,
              warns: () => (v.matchWindows ? { title: "Summer time can come in an hour late.", body: "With the machine's own clock on local time, a change while it's off isn't caught. Set Windows to keep UTC instead, if you can." } : null),
            }),
          ],
        ],
      },
      {
        title: "Format",
        body: "How the bar, the lock screen and the calendar write it.",
        before: () => readsCard(v),
        wells: [
          [
            choice("time-format", "Time format", "Auto writes it the way your language does.", ["Auto", "24-hour", "12-hour"], () => v.timeFormat, (index) => (v.timeFormat = index), { drives: true, initial: () => DEFAULTS.timeFormat }),
            choice("date-order", "Date order", "27 October or October 27; Auto follows your language.", ["Auto", "Day first", "Month first"], () => v.dateOrder, (index) => (v.dateOrder = index), { drives: true, initial: () => DEFAULTS.dateOrder }),
            pick("first-weekday", "Week starts on", "The calendar's first column.", WEEKDAYS, () => v.firstWeekday, (index) => (v.firstWeekday = index), { drives: true, initial: () => DEFAULTS.firstWeekday }),
          ],
        ],
      },
      {
        title: "The bar's clock",
        body: "What it shows beside the time; a narrow bar sheds them first.",
        wells: [
          [
            toggle("seal", "Day seal", "The weekday's kanji on a disc.", () => v.seal, (on) => (v.seal = on), { initial: () => DEFAULTS.seal }),
            toggle("date", "Date", "The weekday and the date, as Tue 27 Oct.", () => v.date, (on) => (v.date = on), { initial: () => DEFAULTS.date }),
            toggle("seconds", "Seconds", "The time to the second.", () => v.seconds, (on) => (v.seconds = on), { initial: () => DEFAULTS.seconds }),
            weatherOnly(
              toggle("bar-weather", "Weather", "Home's temperature and its symbol; a press opens it.", () => v.barWeather, (on) => (v.barWeather = on), { initial: () => DEFAULTS.barWeather }),
            ),
          ],
        ],
      },
      {
        title: "Calendar",
        body: "What the popup's month marks, and the bar's countdown.",
        wells: [
          [
            pick("holidays", "Public holidays", "Marked in the calendar and left out of working days.", COUNTRIES, () => v.holidays, (index) => (v.holidays = index), { initial: () => DEFAULTS.holidays }),
            toggle("week-numbers", "Week numbers", "A column of them; off, a line under the month names the week.", () => v.weekNumbers, (on) => (v.weekNumbers = on), { initial: () => DEFAULTS.weekNumbers }),
            pick("bar-countdown", "Countdown in the bar", "Time left to one of your countdowns, in the bar.", BAR_COUNTDOWNS, () => v.barCountdown, (index) => (v.barCountdown = index), { initial: () => DEFAULTS.barCountdown }),
          ],
        ],
      },
      {
        title: "Cities & weather",
        body: "Other cities' times, and the weather where you are.",
        wells: [
          [
            opens("cities", "Cities", "Their times and weather in the clock popup, eight at most.", "cities", () => citiesDetail(v), { drives: true }),
            toggle("weather", "Weather", "In the bar's clock and the clock popup, from Open-Meteo.", () => v.weather, (on) => (v.weather = on), { drives: true, initial: () => DEFAULTS.weather }),
            weatherOnly(pick("home-city", "Home city", "Whose weather leads.", homeCities(v), () => v.homeCity, (index) => (v.homeCity = index), { initial: () => DEFAULTS.homeCity })),
            weatherOnly(choice("temperature", "Temperature", "Auto follows your language.", ["Auto", "°C", "°F"], () => v.temperature, (index) => (v.temperature = index), { initial: () => DEFAULTS.temperature })),
            weatherOnly(pick("wind", "Wind speed", "Auto follows your language.", ["Auto", "km/h", "m/s", "mph"], () => v.wind, (index) => (v.wind = index), { initial: () => DEFAULTS.wind })),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "zone",
        title: "Time zone",
        body: "Pick the city whose clock you live by. Search finds it by city, region or name.",
        back: "Time & zone",
        before: () => zonePicker(v),
        wells: [],
      },
      {
        id: "cities",
        title: "Cities",
        body: "Other cities' times and weather, in the clock popup's row of cards. Eight at most.",
        back: "Cities & weather",
        before: () => citiesList(v),
        wells: [
          [
            addCity(v),
            // The list's default, none, as a setting the resets read; it draws no row: a setting
            // holding a list has no other way into the resets.
            choice("cities-kept", "Cities", "", ["None", "Some"], () => (v.cities.length === 0 ? 0 : 1), (index) => {
              if (index === 0) v.cities = [];
            }, { shows: () => false, initial: () => 0 }),
          ],
        ],
      },
    ],
  };
}
