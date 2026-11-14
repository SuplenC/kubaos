// Kōbō's You screen: a name and a password. The username follows the name and the machine keeps the
// image's name; both are a Collapsible away for whoever wants their own.

import type { Words } from "../../lib/content.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { at, live, onward, PANEL_MEASURES, refused } from "./kobo-pages.ts";

type Shown = "rest" | "selected" | "open" | "error";

const EMPTY = { name: "", username: "", password: "", confirm: "" };

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("you", () => EMPTY),
  selected: at("you"),
  open: at("you", () => ({ namesOpen: true })),
  error: at("you", () => ({ confirm: "correct horse batterz" })),
};

const NAME: Words = { short: "Mika", long: "Maria Magdalena Wasilewska-Nakamura" };

export default page<Shown>({
  level: "kobo",
  slug: "account",
  title: "You",
  lede: "A name and a password: the login follows from them, and stays yours to change.",
  code: { unbuilt: "Kōbō draws the old screen" },
  from: ["step-rail-page", "field", "collapsible", "meter"],
  stage: { live: live("you", () => EMPTY), hint: "type a name: the login follows it", daylight: doesntApply("the live ISO is always dark: the installer has no day.") },
  variants: [
    { name: "Empty", for: "the name first; Continue waits for it", sample: CELLS.rest },
    { name: "Filled", for: "mika@tuxedo, a strong password", sample: CELLS.selected },
    { name: "Login opened", for: "username and machine name, each with its note", sample: CELLS.open },
    { name: "A username it refuses", for: "the field Invalid; the rule under it", sample: at("you", () => ({ username: "Mika", ownUsername: true })) },
    { name: "A system account", for: "root: pick another", sample: at("you", () => ({ username: "root", ownUsername: true, namesOpen: true })) },
    { name: "A machine name it refuses", for: "the rule under it", sample: at("you", () => ({ host: "-tuxedo-" })) },
    { name: "Still typing it again", for: "no error yet: Type it again to confirm.", sample: at("you", () => ({ confirm: "correct horse" })) },
    { name: "The passwords differ", for: "the second Invalid", sample: CELLS.error },
    { name: "A weak password", for: "the meter a quarter, in `error`", sample: at("you", () => ({ password: "kuba", confirm: "kuba" })) },
  ],
  sizes: doesntApply("the panel's sizes are StepRailPage's; this screen fills its page."),
  states: {
    shown: ["rest", "selected", "open", "error"],
    absent: { lit: "its fields light on their pages", pressed: "its Show presses as a Text button", disabled: "every field takes typing", loading: "nothing to wait for", empty: "Empty is its rest: see Variants" },
    rows: [{ name: "You", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "the fields empty; the reason before Continue",
      selected: "filled; the meter and Strong in `success`",
      open: "the Collapsible open: username, machine name",
      error: "Type it again Invalid; its note in `error`",
    },
  },
  focus: doesntApply("no line of its own: its fields light on their pages."),
  content: {
    text: [{ name: "A name", words: NAME, sample: screen((words: string) => at("you", () => ({ name: words, username: words.split(" ")[0]!.toLowerCase() }))()) }],
    missingPicture: doesntApply("no pictures here."),
    counts: doesntApply("one person, one machine."),
  },
  motion: {
    arrives: [onward("you", CELLS.rest)],
    leaves: doesntApply("Continue slides Disk in: its page shows it."),
    between: [refused(CELLS.error)],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "name, the login's toggle, password, again, Show, Back, Continue" },
      { keys: ["Space", "Enter"], does: "on the login's header: opens or shuts it" },
      { keys: ["Enter"], does: "Continue, or the reason flashes" },
      { keys: ["Esc"], does: "back a screen" },
    ],
    order: "Your name, the login's header (and its two fields, open), the password, again, Show, Back, Continue.",
    lands: "In Your name; coming back with it filled, in the password.",
  },
  measures: {
    samples: nonEmpty([{ name: "Login opened", for: "the fields, notes and meter", sample: CELLS.open }]),
    measures: nonEmpty([...PANEL_MEASURES, { part: "Between fields", token: "space-s20" }, { part: "Field → meter", token: "space-s8" }, { part: "Meter → its word", token: "space-s12" }]),
    reflow: { note: "Two columns of fields at every width the panel takes; a short screen scrolls them.", sample: live("you", () => EMPTY) },
  },
});
