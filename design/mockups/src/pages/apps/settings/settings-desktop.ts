// Settings › Desktop: how windows tile and take the keys, the workspaces and their groups, the bar,
// and where the system monitor's readings turn red. Its own parts are the readings' page, the pill
// the bar draws over the list that picks and orders what it shows, and the workspace groups' page,
// a screen to a row.

import { screen } from "../../../site/page.ts";
import { DESKTOP } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { landedOn, settingsWindow } from "./settings-window.ts";

/** The window on Desktop, with the sample's own values `desk` and the sub-page `opened`. */
const at = (desk: object, opened?: string) => screen(() => settingsWindow({ section: DESKTOP, values: { ...(opened ? { opened } : {}), seed: { desktop: desk } } }));

/** The other desk's screen pinned to group 1, and the screen here too, named Left. */
const PINNED = { pins: { lg: 1, benq: 1 }, names: ["Left", "", "", ""] };

const SOME_ON = [
  { id: "cpu", name: "Processor", on: true },
  { id: "temp", name: "Temperature", on: true },
  { id: "ram", name: "Memory", on: true },
  { id: "gpu", name: "Graphics", on: false },
  { id: "net", name: "Network speed", on: false },
  { id: "power", name: "Battery draw", on: false },
];

const ONE_ON = SOME_ON.map((each) => ({ ...each, on: each.id === "cpu" }));

export default sectionPage({
  from: ["empty-state", "order-list"],
  index: DESKTOP,
  slug: "settings-desktop",
  title: "Settings: Desktop",
  lede: "Windows and workspaces: how they tile, take the keys and gather, and the bar over them.",
  hint: "drag a reading; pin a screen",
  seed: { seed: { desktop: { gaps: 1, followMouse: false, ...PINNED } } },
  variants: [
    { name: "Groups: a screen pinned away", for: "the other desk's screen listed while it is pinned", sample: at(PINNED, "groups") },
    { name: "Groups: a new group", for: "the Dell pinned to group 3; its name row comes", sample: at({ pins: { dell: 3 } }, "groups") },
    { name: "Groups: a name refused", for: "too long for the bar: the line says why", sample: at({ names: ["Left side of the big desk", "", "", ""] }, "groups") },
    { name: "Groups: two screens alike", for: "no serials to tell them apart: both dimmed, saying why", sample: at({ twins: true }, "groups") },
    { name: "Groups: the file doesn't read", for: "every row dimmed; the warning says what and where", sample: at({ broken: true }, "groups") },
    { name: "Groups: only the laptop", for: "no other screen seen yet: what to do", sample: at({ alone: true }, "groups") },
    { name: "Readings: three, as bars", for: "temperature moved second; the pill follows", sample: at({ readings: SOME_ON, style: 1 }, "readings") },
    { name: "Readings: as graphs", for: "the last minute of each", sample: at({ readings: SOME_ON, style: 2 }, "readings") },
    { name: "Readings: a row lifted", for: "mid-drag, over the rows it passes", sample: at({ readings: SOME_ON, lifted: 1 }, "readings") },
    { name: "Readings: the last one kept", for: "its switch turned back on; the banner says why", sample: at({ readings: ONE_ON, refused: true }, "readings") },
    { name: "System monitor off", for: "Readings goes with it", sample: screen(() => settingsWindow(landedOn(DESKTOP, "pill", { seed: { desktop: { pill: false } } }))) },
  ],
  keys: [{ keys: ["Alt+↑", "Alt+↓"], does: "on a reading: moves it a place up or down" }],
  measures: [
    { part: "A reading's row", token: "size-control_md" },
    { part: "The pill to the list", token: "space-s16" },
  ],
});
