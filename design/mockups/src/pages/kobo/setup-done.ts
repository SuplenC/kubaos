// First boot's ending: 完了 stamped, the headline word by word, what got done and what waits in the
// Command menu, and the way to the desktop; a queued key asks for the reboot first.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import type { CardId, Views } from "./setup-cards.ts";
import { DESKTOP, ENDS, REBOOT, setupRoom } from "./setup-room.ts";

type Shown = "rest" | "selected" | "loading";

const SOME: Partial<Views> = { tuxedo: "done", fingerprint: "done", yubikey: "done", git: "done" };
const LATER: readonly CardId[] = ["bitwarden", "phone"];
const ALL: Partial<Views> = { tuxedo: "done", fingerprint: "done", yubikey: "done", bitwarden: "done", git: "done", phone: "done" };
const QUEUED: Partial<Views> = { tuxedo: "queued", fingerprint: "done", yubikey: "done", bitwarden: "done", git: "done" };

const at = (views: Partial<Views>, skipped: readonly CardId[] = [], going = false, desk = true) => screen(() => setupRoom({ place: { at: "ending", going }, views, skipped, desk }));

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at(SOME, LATER),
  selected: at(QUEUED, ["phone"]),
  loading: at(QUEUED, ["phone"], true),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-done",
  title: "First boot: That's it",
  lede: "The ending: 完了 stamped, what got done and what waits, and the desktop handed over.",
  code: { unbuilt: "Kōbō ends on Done for now" },
  from: ["step", "button", "keycap", "spinner", "backdrop"],
  stage: { live: screen(() => setupRoom({ place: { at: "ending" }, views: SOME, skipped: LATER, desk: true, live: true })), hint: "Enter: the desktop; the window steps back", daylight: true },
  variants: [
    { name: "Some for later", for: "where they wait", sample: CELLS.rest },
    { name: "All of it", for: "nothing left", sample: at(ALL) },
    { name: "A reboot waits", for: "Reboot now first", sample: CELLS.selected },
    { name: "Restarting", for: "Reboot taken", sample: CELLS.loading },
    { name: "Everything skipped", for: "all of it for later", sample: at({}, ["tuxedo", "fingerprint", "yubikey", "bitwarden", "git", "phone"]) },
    { name: "The desktop", for: "the window gone", sample: screen(() => setupRoom({ place: { at: "desktop" }, desk: true })) },
  ],
  sizes: [{ name: "The window alone", for: "1024 × 680", sample: at(SOME, LATER, false, false) }],
  states: {
    shown: ["rest", "selected", "loading"],
    absent: { lit: "its buttons light as buttons do", pressed: "its buttons press as buttons do", disabled: "the way out always works", empty: "there are always six cards", error: "a card's failure stays on its card" },
    rows: [{ name: "The ending", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: { rest: "Go to the desktop", selected: "Reboot now, then the desktop", loading: "a spinner and Restarting…" },
  },
  focus: doesntApply("no line of its own: its buttons light as buttons do."),
  content: {
    text: doesntApply("the headline is fixed; the lede counts the cards."),
    missingPicture: doesntApply("the hanko is type; the wallpaper is the desktop's."),
    counts: [
      { name: "None left", for: "all done", sample: at(ALL) },
      { name: "Two left", for: "for later", sample: CELLS.rest },
      { name: "All left", for: "everything skipped", sample: at({}, ["tuxedo", "fingerprint", "yubikey", "bitwarden", "git", "phone"]) },
    ],
  },
  motion: {
    arrives: [ENDS],
    leaves: [DESKTOP, REBOOT],
    between: doesntApply("it rests once it has arrived: nothing hums."),
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Go to the desktop, or Reboot now when one waits" },
      { keys: ["Tab", "Shift+Tab"], does: "Reboot now, then Go to the desktop" },
    ],
    order: "Reboot now when one waits, then Go to the desktop.",
    lands: "On the first action from the first frame.",
  },
  measures: {
    samples: nonEmpty([{ name: "Some for later", for: "the hanko, the headline, the plan", sample: CELLS.rest }]),
    measures: [
      { part: "The column", token: "size-column" },
      { part: "The hanko", token: "size-hanko" },
      { part: "Hanko → headline", token: "space-s24" },
      { part: "Headline → lede", token: "space-s16" },
      { part: "Lede → plan → actions", token: "space-s32" },
      { part: "Actions → keys", token: "space-s12" },
    ],
    reflow: { note: "The column stays in the window's middle.", sample: at(SOME, LATER, false, false) },
  },
});
