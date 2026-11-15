// First boot's Git card: the name and email commits carry, the name guessed from the account, the
// email checked as it is typed. Done at the install when "Make it yours" was given them.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { refused } from "./kobo-pages.ts";
import { atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS } from "./setup-room.ts";

type Shown = "rest" | "selected" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("git", "fill"),
  selected: atCard("git", "done"),
  error: atCard("git", "invalid"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-git",
  title: "First boot: Git",
  lede: "The name and email on every commit: the name guessed from the account, the email checked.",
  code: { unbuilt: "Kōbō draws the old checklist" },
  from: ["step-rail-page", "field", "collapsible", "error-block"],
  stage: { live: liveCard("git"), hint: "Enter saves; it works offline", daylight: true },
  variants: [
    { name: "To fill", for: "the name from the account", sample: CELLS.rest },
    { name: "Not an email", for: "the field Invalid; Save waits", sample: CELLS.error },
    { name: "Saved", for: "Save changes, beside them", sample: CELLS.selected },
    { name: "~/.gitconfig wins", for: "where, and what to delete", sample: atCard("git", "wins") },
  ],
  sizes: doesntApply("the window's sizes are StepRailPage's; the card fills its page."),
  states: {
    shown: ["rest", "selected", "error"],
    absent: {
      lit: "its fields light on their pages",
      pressed: "its buttons press on their pages",
      disabled: "every machine has git: always there",
      loading: "writing one file is instant",
      empty: "the name is always guessed",
    },
    rows: [{ name: "Git", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: { rest: "To do; Save", selected: "Done; Save changes", error: "the email Invalid; the blocker" },
  },
  focus: doesntApply("no line of its own: its fields light on their pages."),
  content: {
    text: doesntApply("the name and email are the person's; a field scrolls its own text."),
    missingPicture: doesntApply("no pictures here."),
    counts: doesntApply("one name, one email."),
  },
  motion: {
    arrives: [arrivesAt("git", "fill")],
    leaves: doesntApply("Continue slides the next card in: Your phone's page shows it."),
    between: [refused(CELLS.error), finishesAt("git", "done"), FAILS],
  },
  keyboard: {
    keys: CARD_KEYS,
    order: "The rail, Your name, Email, Under the hood, Back, Skip, Save.",
    lands: "On Email: the name is guessed already.",
  },
  measures: {
    samples: nonEmpty([{ name: "To fill", for: "the two fields", sample: CELLS.rest }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Fields apart", token: "space-s20" }]),
    reflow: { note: "Two fields to a row; the page scrolls under its header.", sample: CELLS.rest },
  },
});
