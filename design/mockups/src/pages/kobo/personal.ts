// Kōbō's Make it yours screen: the wallpapers the picker shows, every one picked to start with, and
// Git's name and email folded away for the developers who want them set now.

import type { Words } from "../../lib/content.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { SETS } from "./kobo-data.ts";
import { wallpaperKey } from "./kobo-parts.ts";
import { at, live, onward, PANEL_MEASURES, refused } from "./kobo-pages.ts";

type Shown = "rest" | "selected" | "open" | "empty" | "error";

const some = () => ({ wallpapers: new Set([wallpaperKey("Shinjuku", "kabukicho rain"), wallpaperKey("Shinjuku", "purple hour"), wallpaperKey("Tokyo", "tokyo tower blue hour")]) });

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("yours"),
  selected: at("yours", some),
  open: at("yours", () => ({ gitName: "Mika Nowak", gitEmail: "mika@nowak.dev" }), { gitOpen: true }),
  empty: at("yours", () => ({ wallpapers: new Set<string>() })),
  error: at("yours", () => ({ gitName: "Mika Nowak", gitEmail: "mika@nowak" }), { gitOpen: true }),
};

const SET: Words = { short: "Tokyo", long: "Neon nights in Kabukicho and the backstreets of Golden Gai" };

export default page<Shown>({
  level: "kobo",
  slug: "personal",
  title: "Make it yours",
  lede: "Every wallpaper picked to start with; Git folded away for whoever wants it set now.",
  code: { unbuilt: "Kōbō asks it on two screens" },
  from: ["step-rail-page", "picture-choice", "checkbox", "chip", "collapsible", "field", "card"],
  stage: { live: live("yours"), hint: "Space on a picture drops it; Git is folded below", daylight: doesntApply("the live ISO is always dark: the installer has no day.") },
  variants: [
    { name: "Every one picked", for: "as Kōbō starts it; All 6 picked.", sample: CELLS.rest },
    { name: "Some picked", for: "a set's box partial, its chip counting", sample: CELLS.selected },
    { name: "Git opened", for: "name and email; the config it writes", sample: CELLS.open },
    { name: "Not an email", for: "the field Invalid; the blocker", sample: CELLS.error },
    { name: "None picked", for: "the blocker: one at least", sample: CELLS.empty },
  ],
  sizes: doesntApply("the panel's sizes are StepRailPage's; the pictures take three a row."),
  states: {
    shown: ["rest", "selected", "open", "empty", "error"],
    absent: { lit: "its pictures, boxes and fields light on their pages", pressed: "they press on their pages", disabled: "every wallpaper can be picked", loading: "the thumbnails ship on the stick" },
    rows: [{ name: "Make it yours", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "every picture picked; All 6 picked. before Continue",
      selected: "a partial box; picked ones marked",
      open: "Git open: two fields and `~/.gitconfig`",
      empty: "none picked: the reason before Continue",
      error: "the email Invalid; the reason",
    },
  },
  focus: doesntApply("no line of its own: its pictures, boxes and fields light on their pages."),
  content: {
    text: [{ name: "A set's name", words: SET, sample: screen((words: string) => at("yours", () => ({}), { setName: words })()) }],
    missingPicture: doesntApply("the wallpapers' thumbnails ship inside Kōbō, on the stick."),
    counts: [
      { name: "One in a set", for: "Tokyo: its box and its picture", sample: CELLS.rest },
      { name: "Many", for: "Shinjuku's three", sample: CELLS.selected },
    ],
  },
  motion: {
    arrives: [onward("yours", CELLS.rest)],
    leaves: doesntApply("Continue slides Last look in: its page shows it."),
    between: [refused(CELLS.empty)],
  },
  keyboard: {
    keys: [
      { keys: ["Space"], does: "picks or drops the picture or set it is on" },
      { keys: ["←", "→", "↑", "↓"], does: "between a set's pictures" },
      { keys: ["Tab", "Shift+Tab"], does: "set to set, Git, its fields, Back, Continue" },
      { keys: ["Enter"], does: "Continue, or the reason flashes" },
      { keys: ["Esc"], does: "back a screen" },
    ],
    order: "Each set's box then its pictures as one stop, Git's header, its two fields, Back, Continue.",
    lands: "On the first set's box.",
  },
  measures: {
    samples: nonEmpty([{ name: "Git opened", for: "the grid, the sets, the fold", sample: CELLS.open }]),
    measures: nonEmpty([...PANEL_MEASURES, { part: "Between sets", token: "space-s32" }, { part: "Set's head → pictures", token: "space-s12" }, { part: "Between pictures", token: "space-s16" }]),
    reflow: { note: "Three pictures a row at every width the panel takes; the page scrolls under its header.", sample: live("yours") },
  },
});

export { SETS };
