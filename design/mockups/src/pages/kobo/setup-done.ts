// First boot's ending, the installer's done screen's sibling: the desktop comes close behind the
// glass, クバ writes itself in and strikes alone in the middle, the headline rises word by word,
// then what got done, what waits in the Command menu, and the way to the desktop; a queued key
// asks for the reboot first.

import { windowSize } from "../../site/settings.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { ALL_DONE, CARD_IDS, type CardId, type Views } from "./setup-cards.ts";
import { DESKTOP, ENDS, REBOOT, setupRoom } from "./setup-room.ts";

type Shown = "rest" | "selected" | "loading";

const SOME: Partial<Views> = { yours: "done", drivers: "done", fingerprint: "done" };
const LATER: readonly CardId[] = ["yubikey", "phone"];
const COMING: Partial<Views> = { yours: "installing", drivers: "done", fingerprint: "done", yubikey: "done" };
const QUEUED: Partial<Views> = { yours: "done", drivers: "queued", fingerprint: "done", yubikey: "done" };

const at = (views: Partial<Views>, skipped: readonly CardId[] = [], going = false, size: { width?: string; height?: string } = {}) => screen(() => setupRoom({ place: { at: "ending", going }, views, skipped, ...size }));

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at(SOME, LATER),
  selected: at(QUEUED, ["phone"]),
  loading: at(QUEUED, ["phone"], true),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-done",
  title: "First boot: That's it",
  lede: "The ending: the desktop comes close, クバ strikes, and the desktop is handed over.",
  code: { unbuilt: "Kōbō ends on Done for now" },
  from: ["step", "button", "keycap", "spinner", "backdrop"],
  stage: { live: screen(() => setupRoom({ place: { at: "ending" }, views: SOME, skipped: LATER, live: true, width: windowSize().width, height: windowSize().height })), hint: "Enter: the desktop; the words lift off it", daylight: true },
  variants: [
    { name: "Some for later", for: "where they wait", sample: CELLS.rest },
    { name: "All of it", for: "nothing left", sample: at(ALL_DONE) },
    { name: "Apps still coming", for: "they land in the background", sample: at(COMING, ["phone"]) },
    { name: "A reboot waits", for: "Reboot now first", sample: CELLS.selected },
    { name: "Restarting", for: "Reboot taken", sample: CELLS.loading },
    { name: "Everything skipped", for: "all of it for later", sample: at({}, CARD_IDS) },
    { name: "The desktop", for: "handed over, at its own size", sample: screen(() => setupRoom({ place: { at: "desktop" } })) },
  ],
  sizes: [{ name: "Full HD", for: "the column in the middle", sample: at(SOME, LATER, false, { width: "calc(1920 * var(--px))", height: "calc(1080 * var(--px))" }) }],
  states: {
    shown: ["rest", "selected", "loading"],
    absent: { lit: "its buttons light as buttons do", pressed: "its buttons press as buttons do", disabled: "the way out always works", empty: "there are always five cards", error: "a card's failure stays on its card" },
    rows: [{ name: "The ending", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: { rest: "Go to the desktop", selected: "Reboot now, then the desktop", loading: "a spinner and Restarting…" },
  },
  focus: doesntApply("no line of its own: its buttons light as buttons do."),
  content: {
    text: doesntApply("the headline is fixed; the lede counts the cards."),
    missingPicture: doesntApply("the mark is drawn from the logo's strokes; the wallpaper is the desktop's."),
    counts: [
      { name: "None left", for: "all done", sample: at(ALL_DONE) },
      { name: "Two left", for: "for later", sample: CELLS.rest },
      { name: "All left", for: "everything skipped", sample: at({}, CARD_IDS) },
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
    samples: nonEmpty([{ name: "Some for later", for: "the mark, the headline, the plan", sample: CELLS.rest }]),
    measures: [
      { part: "The column", token: "size-column" },
      { part: "The mark", token: "size-neon_mark" },
      { part: "Mark → headline", token: "space-s24" },
      { part: "Headline → lede", token: "space-s16" },
      { part: "Lede → plan → actions", token: "space-s32" },
      { part: "Actions → keys", token: "space-s12" },
    ],
    reflow: { note: "The column stays in the screen's middle; on a short screen the mark keeps its size.", sample: screen(() => setupRoom({ place: { at: "ending" }, views: SOME, skipped: LATER, width: windowSize().width, height: windowSize().height })) },
  },
});
