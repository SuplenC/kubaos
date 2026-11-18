// Date & time's organisms, built from the library's parts and kept in the Settings app because only
// it draws them: the time zone picker (Kōbō's zone list with its search and its empty state), the "How
// it reads" card (Kōbō's, following the format's settings), the extra clocks' list (each a row that
// can be removed, or the empty well) and the field that adds one. The places and the moment they
// read come from Kōbō's zones and the clock popup's.

import { card } from "../../../../lib/card.ts";
import { descriptionList } from "../../../../lib/description.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { field, setInvalid, setValue } from "../../../../lib/field.ts";
import { filling, stack } from "../../../../lib/layout.ts";
import { emptyList, list, listWith, type ListRowSpec, ROW_CONTROL } from "../../../../lib/list.ts";
import { LANGUAGES, ZONES, zoneName } from "../../../kobo/kobo-data.ts";
import { ZONES as CLOCK_ZONES } from "../../../shell/clock-parts.ts";
import type { Setting } from "../settings-parts.ts";

/** Date & time's own values: its settings, and what its parts hold while a person uses them. */
export interface DateTimeValues {
  autoZone: boolean;
  /** The time zone, by its index in Kōbō's `ZONES`: picked at install, so it has no default. */
  zone: number;
  /** What the zone search holds. */
  query: string;
  autoTime: boolean;
  /** The time typed by hand while automatic time is off. */
  setTime: string;
  matchWindows: boolean;
  timeFormat: number;
  dateOrder: number;
  firstWeekday: number;
  holidays: number;
  analog: boolean;
  barCountdown: number;
  /** The extra clocks, by IANA name, in the order they were added. */
  clocks: readonly string[];
  /** What Add a clock holds, and whether Enter was pressed on it since it last changed. */
  adding: string;
  tried: boolean;
}

/** The clock file's most extra clocks (`MAX_ZONES` in `kubaos-clock`). */
export const MOST_CLOCKS = 4;

/** The language the sample machine reads in, which Auto follows. */
const LANGUAGE = LANGUAGES[1]!;

/** The clock popup's frozen moment, Tuesday 27 October 2026 at 21:47 in Warsaw, as an instant. */
const MOMENT = new Date(Date.UTC(2026, 9, 27, 20, 47));

/** The weekdays as `first_weekday` names them, Monday first, in the sample's language. */
export const WEEKDAYS = Array.from({ length: 7 }, (_, index) => new Intl.DateTimeFormat(LANGUAGE.tag, { weekday: "long", timeZone: "UTC" }).format(new Date(Date.UTC(2026, 9, 26 + index))));

/** A place a clock can be named by: its city and its IANA name. */
interface Place {
  readonly city: string;
  readonly iana: string;
}

/** Every place the sample's clock engine knows: Kōbō's zones, and the popup's extra clocks. */
const PLACES: readonly Place[] = [...ZONES.map((zone) => ({ city: zone.city, iana: zone.iana })), ...CLOCK_ZONES.map((zone) => ({ city: zone.label, iana: zone.name }))].filter(
  (place, index, all) => all.findIndex((other) => other.iana === place.iana) === index,
);

/** How the settings write a time and a date: Auto reads the sample language's way. */
function reading(values: DateTimeValues): { readonly twelve: boolean; readonly dayFirst: boolean } {
  const twelve = values.timeFormat === 0 ? new Intl.DateTimeFormat(LANGUAGE.tag, { hour: "numeric" }).resolvedOptions().hour12 === true : values.timeFormat === 2;
  const dayFirst = values.dateOrder === 0 ? !LANGUAGE.tag.endsWith("US") : values.dateOrder === 1;
  return { twelve, dayFirst };
}

/** The moment's time in `iana`, written as the settings say. */
function timeIn(values: DateTimeValues, iana: string): string {
  const { twelve } = reading(values);
  return new Intl.DateTimeFormat(twelve ? "en-US" : "en-GB", { hour: twelve ? "numeric" : "2-digit", minute: "2-digit", hour12: twelve, timeZone: iana }).format(MOMENT);
}

/** `iana`'s offset from UTC at the moment, in minutes. */
function offsetOf(iana: string): number {
  const parts = new Intl.DateTimeFormat("en-GB", { timeZone: iana, hourCycle: "h23", year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric" }).formatToParts(MOMENT);
  const part = (type: string) => Number(parts.find((each) => each.type === type)?.value);
  return (Date.UTC(part("year"), part("month") - 1, part("day"), part("hour"), part("minute")) - MOMENT.getTime()) / 60_000;
}

/** How far `iana`'s clock is from the zone the machine is in, as a clock's line says it. */
function apart(values: DateTimeValues, iana: string): string {
  const hours = (offsetOf(iana) - offsetOf(ZONES[values.zone]!.iana)) / 60;
  if (hours === 0) return `${iana} · same time`;
  return hours > 0 ? `${iana} · ${hours} h ahead` : `${iana} · ${-hours} h behind`;
}

/** The place a typed name means: its city or its IANA name, either case. */
function placeNamed(name: string): Place | undefined {
  const typed = name.trim().toLowerCase();
  return PLACES.find((place) => place.city.toLowerCase() === typed || place.iana.toLowerCase() === typed);
}

/** What is wrong with adding the clock `name`, as the clock engine says it, or null. */
export function clockProblem(values: DateTimeValues, name: string): string | null {
  const place = placeNamed(name);
  if (!place) return `No time zone called “${name.trim()}”. Try a city, as Lisbon.`;
  if (values.clocks.includes(place.iana)) return `${place.city} is one of your clocks already.`;
  return null;
}

/** The extra clocks as the row that opens their page says them. */
export function clocksDetail(values: DateTimeValues): string {
  if (values.clocks.length === 0) return "None";
  if (values.clocks.length === 1) return PLACES.find((place) => place.iana === values.clocks[0])?.city ?? values.clocks[0]!;
  return `${values.clocks.length} clocks`;
}

/** Asks the Settings region to draw itself again, for a part whose change shows elsewhere on it. */
function redraw(from: Element): void {
  from.closest(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
}

/**
 * The time zone picker, Kōbō's zone list on a page of its own. Its search narrows the
 * zones by city, region or IANA name as it is typed; the zone in use is ticked, and taking another
 * ticks it. Nothing matching, Kōbō's empty state stands in the list's place.
 */
export function zonePicker(values: DateTimeValues): HTMLElement {
  const matching = () => {
    const typed = values.query.trim().toLowerCase();
    return ZONES.filter((zone) => [zone.city, zone.region, zone.iana].some((words) => words.toLowerCase().includes(typed)));
  };
  const spec = (index: number): ListRowSpec => ({ title: zoneName(ZONES[index]!), detail: ZONES[index]!.iana, checked: index === values.zone });
  const listOf = (): HTMLElement => {
    const shown = matching();
    if (shown.length === 0) return emptyState({ icon: "search_off", title: "Nothing matches.", body: "Try a city near you, or a region, as Europe." });
    const made = listWith(
      shown.map((zone) => spec(ZONES.indexOf(zone))),
      {
        onTake: (index) => {
          const was = shown.findIndex((zone) => ZONES.indexOf(zone) === values.zone);
          values.zone = ZONES.indexOf(shown[index]!);
          if (was >= 0) made.replaceRow(was, spec(ZONES.indexOf(shown[was]!)));
          made.replaceRow(index, spec(values.zone));
        },
      },
    );
    return made.box;
  };
  let below = listOf();
  const search = field({
    icon: "search",
    placeholder: "Search cities and zones",
    ...(values.query ? { value: values.query } : {}),
    onInput: (value) => {
      values.query = value;
      const next = filling(listOf());
      below.replaceWith(next);
      below = next;
    },
  });
  search.addEventListener("keydown", (event) => {
    // Down goes from the search into the zones, as Kōbō's does.
    if (event.key === "ArrowDown" && below.classList.contains("k-list")) {
      event.preventDefault();
      below.focus({ preventScroll: true });
    }
    // The rest is the text's: the page's rows and its Esc wait until the search is left.
    if (event.key !== "Tab") event.stopPropagation();
  });
  return stack("s16", "start", filling(search), filling(below));
}

/**
 * Kōbō's "How it reads" card, the moment written as the format's settings say: the
 * time, the date, and the week from its first day.
 */
export function readsCard(values: DateTimeValues): HTMLElement {
  const { dayFirst } = reading(values);
  const zone = ZONES[values.zone]!.iana;
  const date = new Intl.DateTimeFormat(dayFirst ? "en-GB" : "en-US", { weekday: "long", day: "numeric", month: "long", timeZone: zone }).format(MOMENT);
  const first = WEEKDAYS[values.firstWeekday]!;
  const last = WEEKDAYS[(values.firstWeekday + 6) % 7]!;
  return card({
    header: { title: "How it reads" },
    body: [
      descriptionList([
        { label: "Time", value: timeIn(values, zone) },
        { label: "Date", value: date },
        { label: "Week", value: `${first} to ${last}` },
      ]),
    ],
  });
}

/**
 * The extra clocks, each a row with its time there and how far it is from yours, and
 * the × a saved row is removed with (Delete does the same); none, the empty well.
 */
export function clocksList(values: DateTimeValues): HTMLElement {
  if (values.clocks.length === 0) return emptyList("No extra clocks yet.");
  const remove = (iana: string, from: Element) => {
    values.clocks = values.clocks.filter((each) => each !== iana);
    redraw(from);
  };
  const box = list(
    values.clocks.map((iana) => ({
      title: PLACES.find((place) => place.iana === iana)?.city ?? iana,
      subtitle: apart(values, iana),
      detail: timeIn(values, iana),
      forget: () => remove(iana, box),
    })),
  );
  box.addEventListener("keydown", (event) => {
    if (event.key !== "Delete") return;
    const rows = [...box.querySelectorAll(".k-list-row")];
    const at = rows.findIndex((each) => each.hasAttribute("data-current"));
    if (at < 0) return;
    event.preventDefault();
    remove(values.clocks[at]!, box);
  });
  return box;
}

/**
 * Add a clock's field, at its row's end. Enter adds the place typed, or says in the
 * row's line what is wrong with it, in `error`, the field invalid until it changes. Enter on the
 * row puts the keyboard in it; Esc clears it, then goes back to the row.
 */
function addField(values: DateTimeValues, setting: Setting): HTMLElement {
  const full = values.clocks.length >= MOST_CLOCKS;
  const made = field({
    size: ROW_CONTROL,
    icon: "add_location_alt",
    placeholder: "Lisbon",
    ...(values.adding ? { value: values.adding } : {}),
    ...(full ? { state: "Disabled" as const } : {}),
    onInput: (value) => {
      values.adding = value;
      values.tried = false;
      tell(null);
    },
  });
  const tell = (problem: string | null) => {
    const row = made.closest(".k-list-row");
    const line = row?.querySelector<HTMLElement>(".k-list-title + *");
    setInvalid(made, problem !== null);
    row?.toggleAttribute("data-invalid", problem !== null);
    if (!line) return;
    line.textContent = problem ?? setting.line;
    line.style.color = problem === null ? "var(--muted)" : "var(--color-error)";
  };
  made.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      const problem = clockProblem(values, values.adding);
      if (problem) {
        values.tried = true;
        tell(problem);
      } else {
        values.clocks = [...values.clocks, placeNamed(values.adding)!.iana];
        values.adding = "";
        values.tried = false;
        const region = made.closest(".st-region");
        redraw(made);
        region?.querySelector<HTMLElement>("[data-setting='add-clock'] .k-field")?.focus({ preventScroll: true });
      }
    } else if (event.key === "Escape") {
      event.preventDefault();
      if (values.adding) {
        values.adding = "";
        values.tried = false;
        setValue(made, "");
        tell(null);
      } else made.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
    }
    if (event.key !== "Tab") event.stopPropagation();
  });
  // A press in the field keeps the keyboard there; its row still becomes the current one.
  made.addEventListener("click", (event) => {
    event.stopPropagation();
    made.closest(".k-list-row")?.dispatchEvent(new PointerEvent("pointermove", { bubbles: true }));
  });
  requestAnimationFrame(() => {
    if (values.tried) tell(clockProblem(values, values.adding));
    const well = made.closest<HTMLElement>(".k-list");
    well?.addEventListener("keydown", (event) => {
      const current = well.querySelector(".k-list-row[data-current]");
      if (event.target !== well || !current?.contains(made) || (event.key !== "Enter" && event.key !== " ") || full) return;
      event.preventDefault();
      event.stopPropagation();
      made.focus({ preventScroll: true });
    });
  });
  return made;
}

/** Add a clock's row: its field at the row's end, dimmed with its reason once there are four. */
export function addClock(values: DateTimeValues): Setting {
  const setting: Setting = {
    id: "add-clock",
    title: "Add a clock",
    line: "A city or a zone's IANA name: Lisbon, America/Chicago.",
    control: { kind: "part", make: () => addField(values, setting) },
  };
  return whileDisabled(setting, () => (values.clocks.length >= MOST_CLOCKS ? "Four at most: remove one to add another." : null));
}

/**
 * `setting`, dimmed with `why` while it says so. A setting's `disabled` is read each time its row is
 * drawn, so a getter makes it follow the value it depends on.
 */
export function whileDisabled(setting: Setting, why: () => string | null): Setting {
  Object.defineProperty(setting, "disabled", { get: () => why() ?? undefined, enumerable: true });
  return setting;
}
