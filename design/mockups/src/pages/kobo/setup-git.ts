// First boot's Git card: the name and email every commit carries, the name from the account, the
// email checked as it is typed. Everyone meets it, Developer picked or not, and it says plainly at
// its top that whoever doesn't know what Git is can skip it.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { refused } from "./kobo-pages.ts";
import { aloneAt, atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";

type Shown = "rest" | "empty" | "selected" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  empty: atCard("git", "empty"),
  rest: atCard("git", "filled"),
  selected: atCard("git", "done"),
  error: atCard("git", "invalid"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-git",
  title: "First boot: Git",
  lede: "The name and email on every commit; anyone who doesn't know Git is told to skip it.",
  code: { unbuilt: "Kōbō draws the old Git card" },
  from: ["step-rail-page", "field", "collapsible", "button"],
  stage: { live: liveCard("git"), hint: "Type an email and Enter saves; Skip passes it by", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "Email empty", for: "the name from the account; Save waits", sample: CELLS.empty },
    { name: "Filled", for: "Save", sample: CELLS.rest },
    { name: "Not an email", for: "the field Invalid; Save waits", sample: CELLS.error },
    { name: "Saved", for: "Save changes, beside them", sample: CELLS.selected },
    { name: "From the Command menu", for: "the card alone", sample: aloneAt("git", "filled") },
  ],
  sizes: doesntApply("the screen's sizes are StepRailPage's Kōbō host's; the card fills its page."),
  states: {
    shown: ["rest", "empty", "selected", "error"],
    absent: {
      lit: "its fields light on their pages",
      pressed: "its buttons press on their pages",
      disabled: "every machine has git: always there",
      loading: "writing one file is instant",
    },
    rows: [{ name: "Git", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: { empty: "To do; Save waits on the email", rest: "To do; Save", selected: "Done; Save changes", error: "the email Invalid; the blocker" },
  },
  focus: doesntApply("no line of its own: its fields light on their pages."),
  content: {
    text: doesntApply("the name and email are the person's; a field scrolls its own text."),
    missingPicture: doesntApply("no pictures here."),
    counts: doesntApply("one name, one email."),
  },
  motion: {
    arrives: [arrivesAt("git", "empty")],
    leaves: doesntApply("Continue or Skip slides the next card in: Drivers' page shows it."),
    between: [refused(CELLS.error), finishesAt("git", "done")],
  },
  keyboard: {
    keys: CARD_KEYS,
    order: "The rail, Your name, Email, Under the hood, Back, Skip, Save.",
    lands: "On Email: the name is the account's already.",
  },
  measures: {
    samples: nonEmpty([{ name: "Filled", for: "the reassurance and the two fields", sample: CELLS.rest }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Fields apart", token: "space-s20" }]),
    reflow: { note: "Two fields to a row; the page scrolls under its header.", sample: CELLS.rest },
  },
});
