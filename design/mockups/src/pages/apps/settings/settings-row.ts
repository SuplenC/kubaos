// Settings' row, the organism every section's wells are made of (`settingRow` and `fit` in
// `settings-parts.ts`): a card of rows at five widths, on one line from
// `size.settings_row_one_line`, its wide rows stacked under it, their controls across the row from
// `size.settings_row_full` down; switch rows on one line at every width.

import { within } from "../../../lib/layout.ts";
import type { RegionHost } from "../../../lib/sidebar-page.ts";
import { doesntApply, nonEmpty, page } from "../../../site/page.ts";
import { asSection, type ResetPins, type Setting, type SettingsSection } from "./settings-parts.ts";
import { choice, pick, stops, toggle } from "./sections/kit.ts";

/** The widths a card is shown at. 440 is a stacked card's, between the two switch points. */
const WIDTHS = {
  "720": "var(--size-beside_controls_most)",
  "548": "var(--size-settings_row_one_line)",
  "440": "calc(440 * var(--px))",
  "360": "var(--size-settings_row_full)",
  "300": "var(--size-beside_controls_least)",
} as const;

type Width = keyof typeof WIDTHS;

const BOOSTS = Array.from({ length: 21 }, (_, i) => `${i * 5} %`);
const SHARPNESS = Array.from({ length: 8 }, (_, i) => String(i + 1));
const KELVIN = new Intl.NumberFormat("en-GB", { useGrouping: false });
const TEMPERATURES = Array.from({ length: 38 }, (_, i) => `${KELVIN.format(2800 + i * 100)} K`);

/** A card of the rows every kind of control ends in, Light behind you and Sharpness changed. */
function rows(): Setting[] {
  let camera = 0;
  let light = true;
  let boost = 3;
  let sharpness = 5;
  let flicker = 1;
  return [
    pick("camera", "Camera", "Each keeps its own settings.", ["Integrated Camera", "Logitech C920"], () => camera, (index) => (camera = index)),
    toggle("light", "Light behind you", "Lifts your face when a window or lamp is behind you. (Backlight compensation)", () => light, (on) => (light = on), { initial: () => false }),
    stops("boost", "Boost", "Brightens the picture electronically; more adds grain. (Gain)", "brightness_high", BOOSTS, () => boost, (index) => (boost = index), { initial: () => 3 }),
    stops("sharpness", "Sharpness", "Crisper edges; too much draws bright halos.", "visibility", SHARPNESS, () => sharpness, (index) => (sharpness = index), { initial: () => 3 }),
    choice("flicker", "Room light flicker", "Stops stripes from lights on the mains: 50 Hz in Europe, 60 Hz in the Americas. (Power-line frequency)", ["Off", "50 Hz", "60 Hz"], () => flicker, (index) => (flicker = index), { initial: () => 1 }),
    stops("temperature", "Colour temperature", "Lower looks cooler, higher looks warmer.", "thermostat", TEMPERATURES, () => 18, () => {}, {
      disabled: "Set by Automatic colour; turn it off to set it here. (White balance temperature)",
    }),
  ];
}

/** A page's host that does nothing: the card is drawn alone, with no sidebar to follow it. */
const ALONE: RegionHost = { rename: () => {}, spyOn: () => {}, enter: () => {}, refresh: () => {}, turn: (_deeper, swap) => swap() };

/** The card, under its heading, `width` wide. */
function card(width: string, settings: Setting[] = rows(), pins?: ResetPins): HTMLElement {
  const section: SettingsSection = { icon: "video", title: "Camera", accent: "カメラ", lede: "Seen in a better light.", subs: [{ title: "Picture", wells: [settings] }] };
  return within(width, asSection(section, pins).region!(ALONE));
}

type Shown = "rest" | "selected" | "disabled";

/** One row of each kind in a state, for the matrix. */
function oneOf(kind: "wide" | "switch", state: Shown, width: string): HTMLElement {
  const all = rows();
  const setting = kind === "wide" ? all[3]! : all[1]!;
  const shown: Setting = state === "disabled" ? { ...setting, disabled: "Set by Automatic colour; turn it off to set it here." } : state === "rest" ? { ...setting, initial: () => (kind === "wide" ? 5 : true) } : setting;
  return card(width, [shown]);
}

export default page<Shown>({
  level: "apps",
  slug: "settings-row",
  title: "Settings: Row",
  lede: "A setting's row at every card width: one line, its wide rows stacked, or their controls across.",
  code: { unbuilt: "kubaos-settings: not built yet" },
  from: nonEmpty(["list", "slider", "select", "tabs", "switch", "icon-button"]),
  stage: { live: () => card(WIDTHS["440"]), hint: "change a row; hold its reset", daylight: true },
  variants: nonEmpty(
    (Object.keys(WIDTHS) as Width[]).map((width) => ({
      name: `${width} px`,
      for: { "720": "one line", "548": "one line, at the switch point", "440": "wide rows stacked, controls at the end", "360": "stacked, controls across", "300": "stacked, controls across" }[width],
      sample: () => card(WIDTHS[width]),
    })),
  ),
  sizes: doesntApply("one size: its controls keep theirs at every width."),
  states: {
    shown: ["rest", "selected", "disabled"],
    absent: {
      lit: "a row lights as `ListRow` does",
      pressed: "a row presses as `ListRow` does",
      loading: "every value is read as the window opens",
      empty: "a card always has its rows",
      error: "a field says its own, under its name",
    },
    rows: [
      { name: "Wide, one line", cell: (state: Shown) => oneOf("wide", state, WIDTHS["720"]) },
      { name: "Wide, stacked", cell: (state: Shown) => oneOf("wide", state, WIDTHS["440"]) },
      { name: "Switch, narrow", cell: (state: Shown) => oneOf("switch", state, WIDTHS["300"]) },
    ],
    looks: {
      rest: "at its default: no reset",
      selected: "changed: its reset beside the control",
      disabled: "dimmed in place, saying why",
    },
  },
  focus: doesntApply("no line of its own: its controls light their own."),
  content: {
    text: doesntApply("its words are each setting's; a line wraps, a name clamps."),
    missingPicture: doesntApply("a row carries no picture."),
    counts: [
      { name: "One row", for: "a card of one wide row, stacked", sample: () => card(WIDTHS["440"], rows().slice(3, 4)) },
      { name: "Six rows", for: "every kind of control", sample: () => card(WIDTHS["548"]) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its section's page."),
    leaves: doesntApply("it goes with its section."),
    between: doesntApply("a card restacks at once as its width crosses a switch point."),
  },
  keyboard: doesntApply("the Settings page's keys: each section's page lists them."),
  measures: {
    samples: nonEmpty([
      { name: "One line", for: "720: words beside the control", sample: () => card(WIDTHS["720"]) },
      { name: "Stacked", for: "440: name and reset, line, control", sample: () => card(WIDTHS["440"]) },
    ]),
    measures: nonEmpty([
      { part: "One line from", token: "size-settings_row_one_line" },
      { part: "Control across at or under", token: "size-settings_row_full" },
      { part: "A stacked control", token: "size-menu" },
      { part: "Rows apart, one line", token: "space-s4" },
      { part: "Rows apart, stacked", token: "space-s8" },
      { part: "Line to stacked control", token: "space-s8" },
      { part: "A row's least height", token: "size-row_two_line" },
      { part: "Row padding above and below", token: "space-s8" },
    ]),
    reflow: { note: "The card restacks as its width crosses 548 and 360.", sample: () => card("100%") },
  },
});
