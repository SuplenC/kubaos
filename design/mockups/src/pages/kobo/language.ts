// Kōbō's Language screen: what the person reads and types with. Picking a language picks its
// keyboard and its time zone too, until either is changed, so most people answer one question here.

import type { Words } from "../../lib/content.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { at, live, onward, PANEL_MEASURES } from "./kobo-pages.ts";
import { LAPTOP, room } from "./kobo-room.ts";

const POLSKI = { language: 2, layouts: [3], zone: 2 };

const LANGUAGE: Words = { short: "Polski", long: "English (United States, computer)" };

type Shown = "rest" | "selected" | "open" | "empty";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("language", () => ({}), {}, true),
  selected: at("language", () => POLSKI, {}, true),
  open: at("language", () => POLSKI, { language: { open: "layout" } }, true),
  empty: at("language", () => ({}), { language: { query: "Klingon" } }, true),
};

export default page<Shown>({
  level: "kobo",
  slug: "language",
  title: "Language",
  lede: "One question for most people: the keyboard and the clock follow the language until changed.",
  code: { unbuilt: "Kōbō asks it on two screens" },
  from: ["step-rail-page", "scrolling-list", "select", "field", "card", "description-list"],
  stage: { live: live("language", () => ({}), true), hint: "↑ ↓ picks a language; Tab to the keyboard", daylight: doesntApply("the live ISO is always dark: the installer has no day.") },
  variants: [
    { name: "As Kōbō opens it", for: "English (US), its layout and New York", sample: CELLS.rest },
    { name: "Polski picked", for: "Polish keys and Warsaw follow; the dates read Polish", sample: CELLS.selected },
    { name: "Picking a layout", for: "the select open over the field below", sample: CELLS.open },
    { name: "Picking a time zone", for: "the zone's select open", sample: at("language", () => POLSKI, { language: { open: "zone" } }, true) },
    { name: "Two layouts", for: "the second with its Drop; Super+Space switches", sample: at("language", () => ({ ...POLSKI, layouts: [3, 0], following: false }), {}, true) },
    { name: "Nothing matches", for: "the list's empty state", sample: CELLS.empty },
    { name: "With a cable in", for: "no Wi-Fi step in the rail", sample: at("language", () => ({}), { machine: { ...LAPTOP, online: true } }, true) },
  ],
  sizes: doesntApply("the panel's sizes are StepRailPage's; this screen fills its page."),
  states: {
    shown: ["rest", "selected", "open", "empty"],
    absent: {
      lit: "its list, field and selects light on their pages",
      pressed: "its rows and selects press on their pages",
      disabled: "every answer here can change",
      loading: "the live system lists its languages at once",
      error: "nothing here can be wrong: every pick is valid",
    },
    rows: [{ name: "Language", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "the first language ticked; its keys and zone below",
      selected: "the pick ticked; the right column follows it",
      open: "the layout list over the column, `elevation` 2",
      empty: "Nothing matches. in the list's place",
    },
  },
  focus: doesntApply("no line of its own: its list, field and selects light on their pages."),
  content: {
    text: [{ name: "A language's name", words: LANGUAGE, sample: screen((words: string) => room({ at: { panel: "language" }, language: { first: words } })) }],
    missingPicture: doesntApply("no pictures: the languages, layouts and zones are words."),
    counts: [
      { name: "One layout", for: "the default, following the language", sample: CELLS.rest },
      { name: "Two layouts", for: "Drop on the second", sample: at("language", () => ({ ...POLSKI, layouts: [3, 0], following: false }), {}, true) },
      { name: "Four layouts", for: "Add a layout disabled: XKB holds four", sample: at("language", () => ({ ...POLSKI, layouts: [3, 0, 4, 8], following: false }), {}, true) },
    ],
  },
  motion: {
    arrives: [onward("language", CELLS.rest)],
    leaves: doesntApply("Continue slides the next screen in: You's page shows it."),
    between: doesntApply("its list, selects and fields move as their pages say."),
  },
  keyboard: {
    keys: [
      { keys: ["A–Z"], does: "in the search: narrows the languages" },
      { keys: ["↑", "↓"], does: "picks the language; its keys and zone follow" },
      { keys: ["Tab", "Shift+Tab"], does: "search, list, keyboard, try-it field, zone, Back, Continue" },
      { keys: ["Space"], does: "opens a select; in it, picks" },
      { keys: ["Enter"], does: "Continue" },
      { keys: ["Esc"], does: "back to the welcome" },
    ],
    order: "The search, the list, the keyboard's select and Add, the try-it field, the zone, Back, Continue.",
    lands: "In the search: typing finds a language at once, the arrows pick one.",
  },
  measures: {
    samples: nonEmpty([{ name: "Polski picked", for: "the two columns and the card", sample: CELLS.selected }]),
    measures: nonEmpty([...PANEL_MEASURES, { part: "Between the columns", token: "space-s20" }, { part: "Label → its control", token: "space-s8" }, { part: "Right column's parts", token: "space-s20" }]),
    reflow: { note: "The list takes the left column's height; the right column scrolls with the page on a short screen.", sample: live("language", () => ({}), true) },
  },
});
