// Kōbō's Wi-Fi screen, there only when the live system has no network: the install needs none, so
// it is optional and says so, and the network picked is saved for the first boot, which does.

import type { Words } from "../../lib/content.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { NETWORKS } from "./kobo-data.ts";
import { at, live, onward, PANEL_MEASURES, refused } from "./kobo-pages.ts";
import { LAPTOP, room } from "./kobo-room.ts";

type Shown = "rest" | "selected" | "loading" | "empty" | "error";

const SHORT = { network: 0, networkPassword: "hoshi" };

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("network"),
  selected: at("network", () => ({ network: 0, networkPassword: "kissa-no-hoshi" })),
  loading: at("network", () => ({}), { machine: { ...LAPTOP, networks: "scanning" } }),
  empty: at("network", () => ({}), { machine: { ...LAPTOP, networks: [] } }),
  error: at("network", () => SHORT),
};

const NAME: Words = { short: "Kissa", long: "Orange_Swiatlowod_4F2A_5GHz_Extender_Upstairs" };

export default page<Shown>({
  level: "kobo",
  slug: "network",
  title: "Wi-Fi",
  lede: "Optional, and only with no network: a Wi-Fi saved for the first boot, which needs it.",
  code: { unbuilt: "Kōbō draws the old screen" },
  from: ["step-rail-page", "list", "field", "skeleton"],
  stage: { live: live("network"), hint: "↑ ↓ picks; a secured one asks its password", daylight: doesntApply("the live ISO is always dark: the installer has no day.") },
  variants: [
    { name: "No Wi-Fi", for: "the cable, or later: Continue says Skip", sample: CELLS.rest },
    { name: "A secured network", for: "its password under the list", sample: CELLS.selected },
    { name: "Its password too short", for: "the field Invalid; the blocker", sample: CELLS.error },
    { name: "An open network", for: "no password; Continue", sample: at("network", () => ({ network: 2 })) },
    { name: "Still looking", for: "skeleton rows and what it does", sample: CELLS.loading },
    { name: "None in range", for: "the cable row alone", sample: CELLS.empty },
  ],
  sizes: doesntApply("the screen's sizes are StepRailPage's; this question fills its page."),
  states: {
    shown: ["rest", "selected", "loading", "empty", "error"],
    absent: { lit: "its rows and field light on their pages", pressed: "its rows press on their pages", disabled: "every network can be picked" },
    rows: [{ name: "Wi-Fi", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "the cable row ticked; Skip",
      selected: "the network ticked; its password below",
      loading: "three skeleton rows; Looking for networks…",
      empty: "only the cable row",
      error: "the field Invalid; the reason before Continue",
    },
  },
  focus: doesntApply("no line of its own: its list and field light on their pages."),
  content: {
    text: [{ name: "A network's name", words: NAME, sample: screen((words: string) => room({ at: { panel: "network" }, machine: { ...LAPTOP, networks: [{ ...NETWORKS[1]!, name: words }, ...NETWORKS.slice(2)] } })) }],
    missingPicture: doesntApply("no pictures: a network's signal is a symbol."),
    counts: [
      { name: "None", for: "the cable row alone", sample: CELLS.empty },
      { name: "One", for: "one network beside the cable", sample: at("network", () => ({}), { machine: { ...LAPTOP, networks: NETWORKS.slice(0, 1) } }) },
      { name: "Many", for: "five, by signal", sample: CELLS.rest },
    ],
  },
  motion: {
    arrives: [onward("network")],
    leaves: doesntApply("Continue slides the next screen in: You's page shows it."),
    between: [refused(CELLS.error)],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓"], does: "picks a network, or the cable" },
      { keys: ["Tab", "Shift+Tab"], does: "list, password, Show, Back, Continue" },
      { keys: ["Enter"], does: "Continue, or Skip with no Wi-Fi" },
      { keys: ["Esc"], does: "back to Language" },
    ],
    order: "The list, the password and its Show, Back, Continue.",
    lands: "On the list, on the row picked; a secured pick puts the keyboard in its password.",
  },
  measures: {
    samples: nonEmpty([{ name: "A secured network", for: "the list and the password", sample: CELLS.selected }]),
    measures: nonEmpty([...PANEL_MEASURES, { part: "List → password", token: "space-s20" }]),
    reflow: { note: "The list grows with what is in range; past the page's height the page scrolls.", sample: live("network") },
  },
});
