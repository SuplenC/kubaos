// First boot's TUXEDO drivers card: the drivers from TUXEDO's repository, and with Secure Boot on the
// one-time password for MokManager's blue screen, which the reboot after it asks for.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { refused } from "./kobo-pages.ts";
import { atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "error" | "disabled";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("tuxedo", "secure"),
  loading: atCard("tuxedo", "installing"),
  selected: atCard("tuxedo", "queued"),
  error: atCard("tuxedo", "mismatch"),
  disabled: atCard("tuxedo", "absent"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-tuxedo",
  title: "First boot: TUXEDO drivers",
  lede: "TUXEDO's drivers from its own repository, and with Secure Boot on, the key for the blue screen.",
  code: { unbuilt: "Kōbō draws the old checklist" },
  from: ["step-rail-page", "banner", "field", "card", "collapsible", "error-block"],
  stage: { live: liveCard("tuxedo"), hint: "Enter installs: the password, the build, then Reboot", daylight: true },
  variants: [
    { name: "Secure Boot on", for: "the blue screen explained; its password", sample: CELLS.rest },
    { name: "Secure Boot off", for: "nothing to enrol", sample: atCard("tuxedo", "install") },
    { name: "The passwords differ", for: "the field Invalid; the blocker", sample: CELLS.error },
    { name: "The password", for: "polkit's dialog is up", sample: atCard("tuxedo", "password") },
    { name: "Installing", for: "the step it's on; the log", sample: CELLS.loading },
    { name: "Key queued", for: "Reboot, then the blue screen", sample: CELLS.selected },
    { name: "Installed", for: "they load after a reboot", sample: atCard("tuxedo", "load") },
    { name: "Loaded", for: "the modules running", sample: atCard("tuxedo", "done") },
    { name: "Offline", for: "waiting on the internet", sample: atCard("tuxedo", "offline") },
    { name: "Dialog closed", for: "nothing changed; Try again", sample: atCard("tuxedo", "closed") },
    { name: "No key made", for: "DKMS's fault; the fix", sample: atCard("tuxedo", "nokey") },
    { name: "Not a TUXEDO", for: "Not here: passed by", sample: CELLS.disabled },
  ],
  sizes: doesntApply("the window's sizes are StepRailPage's; the card fills its page."),
  states: {
    shown: ["rest", "loading", "selected", "error", "disabled"],
    absent: { lit: "its fields and buttons light on their pages", pressed: "its buttons press on their pages", empty: "a card always has its words" },
    rows: [{ name: "TUXEDO drivers", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "To do; the banner; the password twice",
      loading: "Running; One thing at a time.",
      selected: "Reboot on the accent; Reboot now",
      error: "the second field Invalid; Install waits",
      disabled: "Not here; Continue only",
    },
  },
  focus: doesntApply("no line of its own: its fields and buttons light on their pages."),
  content: {
    text: doesntApply("the machine's name is DMI's; the card's words are fixed."),
    missingPicture: doesntApply("no pictures: the drivers are a symbol."),
    counts: doesntApply("one machine, one set of drivers."),
  },
  motion: {
    arrives: [arrivesAt("tuxedo", "secure")],
    leaves: doesntApply("Continue slides the next card in: Fingerprint's page shows it."),
    between: [refused(CELLS.error), finishesAt("tuxedo", "done"), FAILS],
  },
  keyboard: {
    keys: [...CARD_KEYS, { keys: ["Space"], does: "Show or Hide the password" }],
    order: "The rail, the password, Show, Same again, Under the hood, Back, Skip, Install.",
    lands: "On the first password field: Secure Boot needs one.",
  },
  measures: {
    samples: nonEmpty([{ name: "Secure Boot on", for: "the banner and the two fields", sample: CELLS.rest }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Fields apart", token: "space-s20" }]),
    reflow: { note: "The page scrolls under its header; the two fields stay side by side.", sample: CELLS.rest },
  },
});
