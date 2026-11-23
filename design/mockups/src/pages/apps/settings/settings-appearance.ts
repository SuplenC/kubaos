// Settings › Appearance: the mode and how it switches, a wallpaper per mode and its motion, where the
// accent comes from, and how things move and what points. The page reacts where it's cheap: the mode
// switches the window with the dark ⇄ light sequence, and the speed, reduce motion and the flicker
// are the site's own, so the whole page moves as they say.

import { type Variant, doesntApply, nonEmpty, page, screen } from "../../../site/page.ts";
import { paletteName } from "../../../site/recolour.ts";
import { toDark, toLight } from "../../../site/sequences.ts";
import { deeperMoves } from "../../templates/sidebar-turns.ts";
import { APPEARANCE, appearance, defaults } from "./settings-data.ts";
import { D, glassSteps } from "./settings-values.ts";
import { settingsOf } from "./settings-parts.ts";
import { landedOn, litOn, liveWindow, MINIMUM, sceneOf, seeded, settingsWindow } from "./settings-window.ts";

type Shown = "rest" | "selected" | "disabled";

const at = (sub: number) => () => settingsWindow({ section: APPEARANCE, sub });

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at(0),
  selected: () => settingsWindow({ section: APPEARANCE, sub: 2, values: { accentFrom: 1 } }),
  disabled: () => settingsWindow({ section: APPEARANCE, values: { opened: "roles" } }),
};

/** Every Appearance setting no backend writes yet, each landed on so it can be found. */
const UNBUILT: Variant[] = settingsOf(appearance(defaults()))
  .filter((each) => each.unbuilt)
  .map((each) => {
    const values = each.id === "sun" ? { mode: 2 } : each.id === "seed" ? { accentFrom: 1 } : each.id === "preset" ? { accentFrom: 2 } : {};
    // A setting on a sub-page shows with its page open; the rest are landed on.
    const page = appearance(defaults()).pages?.find((sub) => sub.wells.flat().some((setting) => setting.id === each.id));
    const draw = page ? () => settingsWindow({ section: APPEARANCE, values: { accentFrom: 1, opened: page.id } }) : () => settingsWindow(landedOn(APPEARANCE, each.id, values));
    return { name: `Not built: ${each.title}`, for: "no backend writes it yet", sample: screen(draw) };
  });

/** The glass setting at `percent` by night, the day's at its default. */
function glassAt(percent: number): [number, number] {
  return [glassSteps().indexOf(percent), D.glass[1]];
}

const NEON = { dark: { theme: "neon", mode: "dark" as const }, light: { theme: "neon", mode: "light" as const } };

export default page<Shown>({
  level: "apps",
  slug: "settings-appearance",
  title: "Settings: Appearance",
  lede: "The mode, a wallpaper for each, where the accent comes from, and how things move and point.",
  code: { unbuilt: "kubaos-settings: not built yet" },
  from: ["sidebar-page", "list", "tabs", "slider", "select", "switch", "colour-picker", "banner", "keycap", "button", "icon-button"],
  stage: {
    live: () => liveWindow({ section: APPEARANCE }),
    hint: "pick Light; hold a reset to put it back",
    daylight: true,
  },
  variants: [
    { name: "Mode, Wallpaper", for: "the section as it opens", sample: screen(CELLS.rest) },
    { name: "By the sun", for: "its hours show under it", sample: screen(() => settingsWindow(litOn(APPEARANCE, "mode", { mode: 2 }))) },
    { name: "Light", for: "the window in light mode", sample: screen(() => settingsWindow({ section: APPEARANCE, values: { mode: 1, drawn: "light" } })) },
    { name: "Accent from a seed", for: "Seed colour shows under it", sample: screen(CELLS.selected) },
    { name: "Accent from a preset", for: "Preset shows under it", sample: screen(() => settingsWindow(litOn(APPEARANCE, "preset", { accentFrom: 2 }))) },
    { name: "Seed colour, opened", for: "a sub-page: the picker", sample: screen(() => settingsWindow({ section: APPEARANCE, values: { accentFrom: 1, opened: "seed" } })) },
    { name: "Fine-tune colours, opened", for: "each role and its contrast", sample: screen(CELLS.disabled) },
    { name: "Glass opacity", for: "the default, 80 % by night", sample: screen(() => settingsWindow(litOn(APPEARANCE, "glass"))) },
    { name: "Glass opacity, changed", for: "90 %: the window's glass follows, Reset shows", sample: screen(() => settingsWindow(litOn(APPEARANCE, "glass", { glass: glassAt(90) }))) },
    { name: "Glass opacity, lowest", for: "73 %: body text still 4.5 : 1 over white", sample: screen(() => settingsWindow(litOn(APPEARANCE, "glass", { glass: glassAt(73) }))) },
    { name: "Glass opacity, solid", for: "100 %: nothing shows through; the edge stays", sample: screen(() => settingsWindow(litOn(APPEARANCE, "glass", { glass: glassAt(100) }))) },
    { name: "Glass opacity by day", for: "the default, 90 %", sample: screen(() => settingsWindow(litOn(APPEARANCE, "glass", { drawn: "light" }))) },
    { name: "Motion & pointer", for: "speed, motion, flicker, pointer", sample: screen(at(3)) },
    { name: "Changed: Motion & pointer", for: "menu motion and pointer size reset beside them", sample: screen(() => settingsWindow({ section: APPEARANCE, sub: 3, values: seeded() })) },
    { name: "Its heading's Reset held halfway", for: "Motion & pointer's: the body half filled", sample: screen(() => settingsWindow({ section: APPEARANCE, sub: 3, values: seeded(), resets: { at: "sub:3", held: 0.5 } })) },
    { name: "Its reset held halfway", for: "Pointer size's, the ring half round", sample: screen(() => settingsWindow({ ...litOn(APPEARANCE, "cursor-size", seeded()), resets: { at: "cursor-size", held: 0.5 } })) },
    ...UNBUILT,
  ],
  sizes: [
    { name: "Minimum", for: "720 × 480: words give way first", sample: screen(() => settingsWindow({ section: APPEARANCE }, MINIMUM)) },
    { name: "Default", for: "1120 × 720", sample: screen(CELLS.rest) },
  ],
  states: {
    shown: ["rest", "selected", "disabled"],
    absent: {
      lit: "its rows and controls light on their pages",
      pressed: "its rows and controls press on their pages",
      loading: "every value is read as it opens",
      empty: "every sub-section has its settings",
      error: "nothing here can be typed wrong",
    },
    rows: [{ name: "Appearance", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "Mode and Wallpaper in view",
      selected: "Seed picked: its row under Accent from",
      disabled: "Keep contrast, always on, says why",
    },
  },
  focus: doesntApply("no line of its own: its rows and controls light their own."),
  content: {
    text: doesntApply("its words are fixed; a wallpaper's name clamps."),
    missingPicture: doesntApply("rows carry no picture; the wallpaper is named."),
    counts: [
      { name: "Accent from the wallpaper", for: "nothing more to set", sample: screen(() => settingsWindow({ section: APPEARANCE, sub: 2 })) },
      { name: "From a seed", for: "one row more", sample: screen(CELLS.selected) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its section's rise, as every section does."),
    leaves: doesntApply("it goes with its section."),
    between: [
      toLight({ from: paletteName("neon", "dark"), to: paletteName("neon", "light") }, sceneOf(NEON.dark, NEON.light, { section: APPEARANCE })),
      toDark({ from: paletteName("neon", "light"), to: paletteName("neon", "dark") }, sceneOf(NEON.light, NEON.dark, { section: APPEARANCE })),
      ...deeperMoves(() => settingsWindow({ section: APPEARANCE, values: { accentFrom: 1, opened: "seed" } })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓"], does: "the setting above or below" },
      { keys: ["←", "→"], does: "a choice's or a slider's value" },
      { keys: ["Enter", "Space"], does: "a switch flips; a select opens; a row opens" },
      { keys: ["Esc"], does: "a sub-page: back to Colour" },
      { keys: ["Tab"], does: "on a changed row: its reset, before the control" },
      { keys: ["Enter", "Space"], does: "held on a reset or Reset: puts it back once full" },
    ],
    order: "The section's wells in reading order, one stop each.",
    lands: "On its first row from the sidebar; on the setting from search.",
  },
  measures: {
    samples: nonEmpty([{ name: "Mode", for: "a choice row and its column", sample: CELLS.rest }]),
    measures: nonEmpty([
      { part: "A setting's row", token: "size-row_two_line" },
      { part: "The control column", token: "size-menu" },
      { part: "A choice", token: "size-control_sm" },
      { part: "Wells apart", token: "space-s16" },
    ]),
    reflow: doesntApply("the window's own; Settings shows it."),
  },
});
