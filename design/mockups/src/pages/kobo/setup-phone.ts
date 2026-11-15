// First boot's phone card: the link stays off until it is set up here; then the app on the phone,
// the wait for its pairing request (checked every 2 s), the key to compare, and what works each way.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS, HUM } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "error" | "empty";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("phone", "off"),
  loading: atCard("phone", "starting"),
  empty: atCard("phone", "waiting"),
  selected: atCard("phone", "done"),
  error: atCard("phone", "failed"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-phone",
  title: "First boot: Your phone",
  lede: "Off until set up here; then the phone's app, its pairing request, and what works each way.",
  code: { unbuilt: "Kōbō draws the old checklist" },
  from: ["step-rail-page", "card", "banner", "collapsible", "error-block"],
  stage: { live: liveCard("phone"), hint: "Enter turns it on; the phone asks by itself", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "Off", for: "the app to get first", sample: CELLS.rest },
    { name: "Starting", for: "KDE Connect coming up", sample: CELLS.loading },
    { name: "Waiting for the phone", for: "it hums; nothing to press", sample: CELLS.empty },
    { name: "Still no phone", for: "after 30 s: by its address", sample: atCard("phone", "address") },
    { name: "A phone asks", for: "the key to compare; Accept", sample: atCard("phone", "request") },
    { name: "Paired", for: "what works each way; the tests", sample: CELLS.selected },
    { name: "Test sent", for: "what to paste on the phone", sample: atCard("phone", "sent") },
    { name: "Stopped", for: "on, but not running", sample: atCard("phone", "stopped") },
    { name: "Failed", for: "the cause; journalctl", sample: CELLS.error },
  ],
  sizes: doesntApply("the screen's sizes are StepRailPage's Kōbō host's; the card fills its page."),
  states: {
    shown: ["rest", "loading", "selected", "error", "empty"],
    absent: { lit: "its buttons light on their pages", pressed: "its buttons press on their pages", disabled: "every machine can pair a phone: always there" },
    rows: [{ name: "Your phone", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "To do; Set up a phone",
      loading: "Running; a spinner",
      empty: "Waiting; the phone symbol hums",
      selected: "Done; the tests beside it",
      error: "the error block; Start again",
    },
  },
  focus: doesntApply("no line of its own: its buttons light on their pages."),
  content: {
    text: doesntApply("the phone's name is its own; a line wraps whole."),
    missingPicture: doesntApply("no pictures: the phone is a symbol."),
    counts: [
      { name: "No phone", for: "waiting for one", sample: CELLS.empty },
      { name: "One asking", for: "the request", sample: atCard("phone", "request") },
      { name: "Paired", for: "one phone", sample: CELLS.selected },
    ],
  },
  motion: {
    arrives: [arrivesAt("phone", "off")],
    leaves: doesntApply("Continue past the last card is the ending: That's it shows it."),
    between: [HUM, finishesAt("phone", "done"), FAILS],
  },
  keyboard: {
    keys: CARD_KEYS,
    order: "The rail, the card's buttons, Under the hood, Back, Skip, the band's action.",
    lands: "On the band's action; while it waits, nothing needs a key.",
  },
  measures: {
    samples: nonEmpty([{ name: "Waiting for the phone", for: "the prompt and its words", sample: CELLS.empty }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Prompt's symbol", token: "icon-s48" }]),
    reflow: { note: "The page scrolls under its header; the prompt keeps two lines of words.", sample: CELLS.empty },
  },
});
