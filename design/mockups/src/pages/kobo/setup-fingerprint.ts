// First boot's Fingerprint card: the reader, the finger to enrol (the next one fprintd hasn't got,
// another behind its Select), the touches counted while it enrols, and what sudo does once it can.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS, HUM, TOUCH, setupRoom } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "error" | "disabled";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("fingerprint", "enroll"),
  loading: atCard("fingerprint", "touching"),
  selected: atCard("fingerprint", "done"),
  error: atCard("fingerprint", "failed"),
  disabled: atCard("fingerprint", "absent"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-fingerprint",
  title: "First boot: Fingerprint",
  lede: "The reader, the finger, and each touch counted until sudo takes it instead of the password.",
  code: { unbuilt: "Kōbō draws the old checklist" },
  from: ["step-rail-page", "card", "progress-bar", "select", "collapsible", "error-block"],
  stage: { live: liveCard("fingerprint"), hint: "Enter enrols: the password, then eight touches", daylight: true },
  variants: [
    { name: "To enrol", for: "the reader found; right index picked", sample: CELLS.rest },
    { name: "The password", for: "polkit's dialog is up", sample: atCard("fingerprint", "password") },
    { name: "First touch", for: "waiting on the finger: it hums", sample: atCard("fingerprint", "touch") },
    { name: "Touching", for: "3 of 8; shift it a little", sample: CELLS.loading },
    { name: "Last touch", for: "Lift it. Once more.", sample: atCard("fingerprint", "last") },
    { name: "Enrolled", for: "the finger; Add another finger", sample: CELLS.selected },
    { name: "Enrolled, sudo off", for: "a finger, but no PAM yet", sample: atCard("fingerprint", "pam") },
    { name: "Failed", for: "the cause; Under the hood open", sample: CELLS.error },
    { name: "Not on this machine", for: "no reader: passed by", sample: CELLS.disabled },
    { name: "Under the hood", for: "the reader, the command", sample: atCard("fingerprint", "enroll", { hood: true }) },
  ],
  sizes: doesntApply("the window's sizes are StepRailPage's; the card fills its page."),
  states: {
    shown: ["rest", "loading", "selected", "error", "disabled"],
    absent: { lit: "its fields and buttons light on their pages", pressed: "its buttons press on their pages", empty: "a card always has its words" },
    rows: [{ name: "Fingerprint", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "To do; Enroll a finger",
      loading: "Running; the touches counted",
      selected: "Done; the finger as a chip",
      error: "the error block; Try again",
      disabled: "Not here; Continue only",
    },
  },
  focus: doesntApply("no line of its own: its fields and buttons light on their pages."),
  content: {
    text: doesntApply("the reader's name and the finger's are short and fixed by fprintd."),
    missingPicture: doesntApply("no pictures: the reader is a symbol."),
    counts: [
      { name: "None", for: "no finger yet", sample: CELLS.rest },
      { name: "One", for: "right index", sample: CELLS.selected },
    ],
  },
  motion: {
    arrives: [arrivesAt("fingerprint", "enroll")],
    leaves: doesntApply("Continue slides the next card in: the YubiKey's page shows it."),
    between: [HUM, TOUCH, finishesAt("fingerprint", "done"), FAILS],
  },
  keyboard: {
    keys: [...CARD_KEYS, { keys: ["Space"], does: "opens the Finger select" }],
    order: "The rail, Finger, Under the hood, Back, Skip, Enroll a finger.",
    lands: "On Enroll a finger: the finger picked is a good one.",
  },
  measures: {
    samples: nonEmpty([{ name: "Touching", for: "the prompt, the bar and the count", sample: CELLS.loading }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Prompt's symbol", token: "icon-s48" }, { part: "Symbol → words", token: "space-s20" }, { part: "Bar → count", token: "space-s4" }]),
    reflow: { note: "The card's page scrolls under its header; the prompt keeps two lines of words.", sample: screen(() => setupRoom({ place: { at: "card", card: "fingerprint" }, views: { fingerprint: "touching" } })) },
  },
});
