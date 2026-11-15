// First boot's Bitwarden card: log in once, and the vault gives the SSH keys, commit signing, the gh
// token and the projects' env. Another way in (an API key, a YubiKey's OTP, a server of your own)
// sits behind the login it replaces; the folders keys come from, behind the keys.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { refused } from "./kobo-pages.ts";
import { atCard, arrivesAt, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "error" | "disabled";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("bitwarden", "login"),
  loading: atCard("bitwarden", "pulling"),
  selected: atCard("bitwarden", "done"),
  error: atCard("bitwarden", "wrong"),
  disabled: atCard("bitwarden", "absent"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-bitwarden",
  title: "First boot: Bitwarden",
  lede: "Log in once: SSH keys, commit signing, gh and every project's env come from the vault.",
  code: { unbuilt: "Kōbō draws the old checklist" },
  from: ["step-rail-page", "field", "radio", "collapsible", "card", "log"],
  stage: { live: liveCard("bitwarden"), hint: "Enter logs in and pulls; ← → switch the server", daylight: true },
  variants: [
    { name: "Log in", for: "server, email, password, code", sample: CELLS.rest },
    { name: "No code yet", for: "the blocker says which", sample: atCard("bitwarden", "code") },
    { name: "Another way in", for: "an API key on your own server", sample: atCard("bitwarden", "apikey") },
    { name: "Logged in, locked", for: "only the password; the folders", sample: atCard("bitwarden", "unlock") },
    { name: "Pulling", for: "the step it's on; the log", sample: CELLS.loading },
    { name: "Pulled", for: "the keys it wrote; Pull again", sample: CELLS.selected },
    { name: "Offline", for: "waiting on the internet", sample: atCard("bitwarden", "offline") },
    { name: "Wrong password", for: "bw's reason; Try again", sample: CELLS.error },
    { name: "No bw", for: "Not here: passed by", sample: CELLS.disabled },
  ],
  sizes: doesntApply("the window's sizes are StepRailPage's; the card fills its page."),
  states: {
    shown: ["rest", "loading", "selected", "error", "disabled"],
    absent: { lit: "its fields light on their pages", pressed: "its buttons press on their pages", empty: "a card always has its words" },
    rows: [{ name: "Bitwarden", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "To do; the form, filled",
      loading: "Running; the form hidden",
      selected: "Done; the keys in a card",
      error: "the error block; the form again",
      disabled: "Not here; Continue only",
    },
  },
  focus: doesntApply("no line of its own: its fields and radios light on their pages."),
  content: {
    text: doesntApply("the email and server are the person's; a field scrolls its own text."),
    missingPicture: doesntApply("no pictures: the vault is a symbol."),
    counts: [
      { name: "No keys pulled", for: "logged out", sample: CELLS.rest },
      { name: "Two keys", for: "each with what it became", sample: CELLS.selected },
    ],
  },
  motion: {
    arrives: [arrivesAt("bitwarden", "login")],
    leaves: doesntApply("Continue slides the next card in: Git's page shows it."),
    between: [refused(atCard("bitwarden", "code")), finishesAt("bitwarden", "done"), FAILS],
  },
  keyboard: {
    keys: [...CARD_KEYS, { keys: ["←", "→"], does: "the server, or the way in" }],
    order: "The rail, the server, the fields, Another way to log in, Under the hood, the band.",
    lands: "On the first empty field, or Log in and pull when none is.",
  },
  measures: {
    samples: nonEmpty([{ name: "Log in", for: "the radios and the fields", sample: CELLS.rest }]),
    measures: nonEmpty([...CARD_MEASURES, { part: "Fields apart", token: "space-s20" }, { part: "Radios apart", token: "space-s20" }]),
    reflow: { note: "The form scrolls under the header; two fields to a row.", sample: CELLS.rest },
  },
});
