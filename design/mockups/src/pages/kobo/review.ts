// Kōbō's Last look: everything answered on four cards, each a Change away, and the one sentence
// naming what the disk loses, on a banner whose button is held to install.

import { doesntApply, type Move, nonEmpty, page, screen } from "../../site/page.ts";
import { EMPTY_DISK, STICK } from "./kobo-data.ts";
import { at, live, onward, PANEL_MEASURES } from "./kobo-pages.ts";
import { LAPTOP, STARTING } from "./kobo-room.ts";

type Shown = "rest" | "pressed" | "selected" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("review"),
  pressed: at("review", () => ({}), { review: { held: 0.6 } }),
  selected: at("review", () => ({ mode: "erase" as const })),
  error: at("review", () => ({}), { review: { lowBattery: 14 }, machine: { ...LAPTOP, battery: { percent: 14, charging: false } } }),
};

const HOLDING: Move = {
  name: "Holding the button",
  what: "Held: the banner's foot fills over `hold`, the words Keep holding…; let go early and it drains over `short`.",
  tracks: [{ part: "the hold", on: "the banner's foot", target: ".k-banner .k-progress", property: "--progress", from: 0, to: 1, duration: "hold", curve: "linear" }],
  reduced: "same",
  back: "cuts",
  sample: CELLS.rest,
};

export default page<Shown>({
  level: "kobo",
  slug: "review",
  title: "Last look",
  lede: "Every answer on a card a Change away, and the sentence naming what the disk loses, held to install.",
  code: { unbuilt: "Kōbō draws the old screen" },
  from: ["step-rail-page", "card", "description-list", "banner", "progress-bar", "keycap"],
  stage: { live: live("review"), hint: "hold Enter, or the button, to install", daylight: doesntApply("the live ISO is always dark: the installer has no day.") },
  variants: [
    { name: "Next to Windows", for: "an Info banner: Windows stays where it is", sample: CELLS.rest },
    { name: "Erasing", for: "a Danger banner naming the disk; Hold to erase and install", sample: CELLS.selected },
    { name: "Held halfway", for: "Keep holding…; the foot half full", sample: CELLS.pressed },
    { name: "Erasing, held halfway", for: "the Danger foot filling in `error`", sample: at("review", () => ({ mode: "erase" as const }), { review: { held: 0.6 } }) },
    { name: "Low battery", for: "a Warning banner over the cards", sample: CELLS.error },
    { name: "An empty disk", for: "nothing to lose: There's no undo.", sample: at("review", () => ({ mode: "erase" as const }), { machine: { ...LAPTOP, disks: [EMPTY_DISK, STICK] } }) },
  ],
  sizes: doesntApply("the screen's sizes are StepRailPage's; the cards scroll above the banner."),
  states: {
    shown: ["rest", "pressed", "selected", "error"],
    absent: { lit: "the hold lights as its button kind does", disabled: "the answers are checked before this screen", loading: "nothing to wait for", empty: "four cards, always" },
    rows: [{ name: "Last look", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "an Info banner; Hold to install",
      pressed: "held: Keep holding…, the foot filling",
      selected: "erasing: a Danger banner, Hold to erase and install",
      error: "low battery: a Warning banner on top",
    },
  },
  focus: doesntApply("no line of its own: Change, Back and the hold light as buttons do."),
  content: {
    text: doesntApply("its words are the answers: each screen's page shows their extremes."),
    missingPicture: doesntApply("no pictures here."),
    counts: doesntApply("four cards, always."),
  },
  motion: {
    arrives: [onward("review")],
    leaves: [STARTING],
    between: [HOLDING],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "held for `hold` on the button: installs" },
      { keys: ["Space"], does: "the same, held" },
      { keys: ["Tab", "Shift+Tab"], does: "each card's Change, Back, the hold" },
      { keys: ["Esc"], does: "back a screen" },
    ],
    order: "Each card's Change in reading order (System's only with Wi-Fi asked), Back, the hold.",
    lands: "On the hold, so Enter held installs and nothing else needs finding.",
  },
  measures: {
    samples: nonEmpty([{ name: "Erasing", for: "the cards and the banner", sample: CELLS.selected }]),
    measures: nonEmpty([...PANEL_MEASURES, { part: "Header → cards → banner", token: "space-s20" }, { part: "Between cards", token: "space-s16" }, { part: "Hold → or hold Enter", token: "space-s8" }, { part: "The hold", token: "size-control_lg" }]),
    reflow: { note: "The cards scroll between the header and the banner, which stays at the foot.", sample: live("review") },
  },
});
