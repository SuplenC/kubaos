// First boot's Drivers card: the machine recognised by its maker, what works already and what the
// maker's drivers would fix, said as the parts they make work; ticking a part chooses it, the
// package beside it says exactly what goes on. With Secure Boot on, the one-time password for
// MokManager's blue screen, which the reboot after it asks for.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { refused } from "./kobo-pages.ts";
import { aloneAt, atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("drivers", "secure"),
  loading: atCard("drivers", "installing"),
  selected: atCard("drivers", "works"),
  error: atCard("drivers", "mismatch"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-drivers",
  title: "First boot: Drivers",
  lede: "The machine's maker recognised: what works already, and what its drivers would fix.",
  code: { unbuilt: "Kōbō draws the TUXEDO card" },
  from: ["step-rail-page", "list", "banner", "field", "collapsible", "error-block"],
  stage: { live: liveCard("drivers"), hint: "Enter installs: the password, the build, then Reboot", daylight: true },
  variants: [
    { name: "TUXEDO, Secure Boot on", for: "a part to install; the blue screen's password", sample: CELLS.rest },
    { name: "A part chosen too", for: "Control Center ticked", sample: atCard("drivers", "choose") },
    { name: "Secure Boot off", for: "nothing to enrol", sample: atCard("drivers", "install") },
    { name: "System76", for: "three parts, all ticked", sample: atCard("drivers", "system76") },
    { name: "Everything works", for: "Framework: nothing to install", sample: CELLS.selected },
    { name: "A maker I don't know", for: "what I checked; where to look", sample: atCard("drivers", "unknown") },
    { name: "The passwords differ", for: "the field Invalid; the blocker", sample: CELLS.error },
    { name: "The password", for: "polkit's dialog is up", sample: atCard("drivers", "password") },
    { name: "Installing", for: "the step it's on; the log", sample: CELLS.loading },
    { name: "Key queued", for: "Reboot, then the blue screen", sample: atCard("drivers", "queued") },
    { name: "Installed", for: "they load after a reboot", sample: atCard("drivers", "load") },
    { name: "Loaded", for: "everything works now", sample: atCard("drivers", "done") },
    { name: "Offline", for: "waiting on the internet", sample: atCard("drivers", "offline") },
    { name: "Dialog closed", for: "nothing changed; Try again", sample: atCard("drivers", "closed") },
    { name: "No key made", for: "DKMS's fault; the fix", sample: atCard("drivers", "nokey") },
    { name: "From the Command menu", for: "the card alone", sample: aloneAt("drivers", "secure") },
  ],
  sizes: doesntApply("the window's sizes are StepRailPage's; the card fills its page."),
  states: {
    shown: ["rest", "loading", "selected", "error"],
    absent: {
      lit: "its rows, fields and buttons light on their pages",
      pressed: "its buttons press on their pages",
      empty: "a machine always has something that works",
      disabled: "every machine is checked: never passed by",
    },
    rows: [{ name: "Drivers", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "To do; the parts ticked; the password twice",
      loading: "Running; One thing at a time.",
      selected: "Done; every part Works",
      error: "the second field Invalid; Install waits",
    },
  },
  focus: doesntApply("no line of its own: its rows, fields and buttons light on their pages."),
  content: {
    text: doesntApply("the machine's name is DMI's; the parts' words are the maker data's."),
    missingPicture: doesntApply("no pictures: each part is a symbol on a tile."),
    counts: [
      { name: "No part", for: "everything works", sample: CELLS.selected },
      { name: "One part", for: "and one optional", sample: CELLS.rest },
      { name: "Three parts", for: "System76", sample: atCard("drivers", "system76") },
    ],
  },
  motion: {
    arrives: [arrivesAt("drivers", "secure")],
    leaves: doesntApply("Continue slides the next card in: Fingerprint's page shows it."),
    between: [refused(CELLS.error), finishesAt("drivers", "done"), FAILS],
  },
  keyboard: {
    keys: [...CARD_KEYS, { keys: ["Space"], does: "ticks the part the keys are on" }],
    order: "The rail, the parts, the password, Show, Same again, Under the hood, Back, Skip, Install.",
    lands: "On the parts' list, the first part lit.",
  },
  measures: {
    samples: nonEmpty([{ name: "TUXEDO, Secure Boot on", for: "the parts, the banner and the two fields", sample: CELLS.rest }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Fields apart", token: "space-s20" }, { part: "Rows apart", token: "space-s4" }]),
    reflow: { note: "The page scrolls under its header; the two fields stay side by side.", sample: CELLS.rest },
  },
});
