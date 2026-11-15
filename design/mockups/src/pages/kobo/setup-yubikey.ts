// First boot's YubiKey card: waiting for the key to be plugged in, a finger on its sensor, and the
// key added for sudo, polkit and the lock screen; the key's own PIN and fingers go in a terminal.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS, HUM } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "error" | "empty";

const CELLS: Record<Shown, () => HTMLElement> = {
  empty: atCard("yubikey", "plug"),
  rest: atCard("yubikey", "enroll"),
  loading: atCard("yubikey", "touch"),
  selected: atCard("yubikey", "done"),
  error: atCard("yubikey", "failed"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-yubikey",
  title: "First boot: YubiKey",
  lede: "Plug the key in, touch its sensor, and it stands in for the password at sudo and the lock.",
  code: { unbuilt: "Kōbō draws the old checklist" },
  from: ["step-rail-page", "card", "banner", "collapsible", "error-block"],
  stage: { live: liveCard("yubikey"), hint: "Enter checks for the key, then enrols it", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "No key yet", for: "waiting on the plug: it hums", sample: CELLS.empty },
    { name: "Plugged in", for: "Enroll this key", sample: CELLS.rest },
    { name: "Needs its own setup", for: "the ykman commands; a terminal", sample: atCard("yubikey", "prep") },
    { name: "A finger on the key", for: "waiting on the sensor: it hums", sample: CELLS.loading },
    { name: "The password", for: "polkit's dialog, for the mapping", sample: atCard("yubikey", "password") },
    { name: "Enrolled", for: "a backup key, nudged once", sample: CELLS.selected },
    { name: "Two keys in", for: "leave only the one", sample: atCard("yubikey", "two") },
    { name: "Didn't confirm", for: "wrong finger, or too slow", sample: CELLS.error },
  ],
  sizes: doesntApply("the screen's sizes are StepRailPage's Kōbō host's; the card fills its page."),
  states: {
    shown: ["rest", "loading", "selected", "error", "empty"],
    absent: { lit: "its buttons light on their pages", pressed: "its buttons press on their pages", disabled: "every machine can take a key: always there" },
    rows: [{ name: "YubiKey", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      empty: "Waiting; the key symbol hums",
      rest: "To do; Enroll this key",
      loading: "Running; touch the sensor",
      selected: "Done; Check again for a backup",
      error: "the error block; Try again",
    },
  },
  focus: doesntApply("no line of its own: its buttons light on their pages."),
  content: {
    text: doesntApply("the key's name is its USB product name, short and fixed."),
    missingPicture: doesntApply("no pictures: the key is a symbol."),
    counts: [
      { name: "None", for: "nothing plugged in", sample: CELLS.empty },
      { name: "One", for: "the key to enrol", sample: CELLS.rest },
      { name: "Two", for: "leave only the one", sample: atCard("yubikey", "two") },
    ],
  },
  motion: {
    arrives: [arrivesAt("yubikey", "plug")],
    leaves: doesntApply("Continue slides the next card in: Your phone's page shows it."),
    between: [HUM, finishesAt("yubikey", "done"), FAILS],
  },
  keyboard: {
    keys: CARD_KEYS,
    order: "The rail, Under the hood, Back, Skip, Check again or Enroll this key.",
    lands: "On the band's action: there is nothing to type.",
  },
  measures: {
    samples: nonEmpty([{ name: "A finger on the key", for: "the prompt and its words", sample: CELLS.loading }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Prompt's symbol", token: "icon-s48" }]),
    reflow: { note: "The page scrolls under its header; the prompt keeps two lines of words.", sample: CELLS.loading },
  },
});
