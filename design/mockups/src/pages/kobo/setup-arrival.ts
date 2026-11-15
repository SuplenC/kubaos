// First boot's arrival: the first frame of KubaOS proper. The window opens on the new desktop, says
// what's ahead and that all of it can wait, and one button deals the cards.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { TOWER } from "./setup-cards.ts";
import { BACK, CARDS_IN, HERO, type Moment, OPENS, setupRoom } from "./setup-room.ts";

type Shown = "rest" | "loading" | "selected" | "disabled";

const at = (moment: Moment, more: Partial<Parameters<typeof setupRoom>[0]> = {}) => screen(() => setupRoom({ place: { at: "arrival", moment }, desk: true, ...more }));

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("rest"),
  loading: at("detecting"),
  selected: at("settled", { views: { tuxedo: "done", fingerprint: "done", yubikey: "done", bitwarden: "done", git: "done", phone: "done" } }),
  disabled: at("rest", { views: TOWER }),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-arrival",
  title: "First boot: Arrival",
  lede: "The first frame of KubaOS proper: one window on the new desktop, what's ahead, one button.",
  code: { unbuilt: "Kōbō opens on the checklist" },
  from: ["step", "button", "keycap", "spinner", "backdrop"],
  stage: { live: screen(() => setupRoom({ place: { at: "arrival" }, desk: true, live: true })), hint: "Enter deals the cards; Esc goes to the desktop", daylight: true },
  variants: [
    { name: "First login", for: "what's ahead; Let's go", sample: CELLS.rest },
    { name: "Setup opens", for: "the opening, played as it is picked", sample: screen(() => setupRoom({ place: { at: "arrival" }, desk: true, live: true, opening: true })) },
    { name: "Checking the machine", for: "detection running; Let's go waits", sample: CELLS.loading },
    { name: "Offline", for: "what waits on the network, in `warning`", sample: at("offline") },
    { name: "A desktop, no reader", for: "two cards Not here", sample: CELLS.disabled },
    { name: "Git from the install", for: "Done before it starts", sample: at("rest", { views: { git: "done" } }) },
    { name: "After the MOK reboot", for: "opened once more to check", sample: at("rebooted", { views: { tuxedo: "done", fingerprint: "done", git: "done" } }) },
    { name: "Nothing to do", for: "opened again with all done", sample: CELLS.selected },
  ],
  sizes: [{ name: "The window alone", for: "1024 × 680, as Hyprland floats it", sample: at("rest", { desk: false }) }],
  states: {
    shown: ["rest", "loading", "selected", "disabled"],
    absent: { lit: "Let's go lights as a `Filled` button", pressed: "Let's go presses as a `Filled` button", empty: "there are always six cards", error: "detection never fails: a card it can't read is To do" },
    rows: [{ name: "The arrival", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "the plan's chips; Let's go and Later",
      loading: "a spinner in the plan's place; Let's go disabled",
      selected: "Nothing to do; Go to the desktop",
      disabled: "Not here on two cards",
    },
  },
  focus: doesntApply("no line of its own: Let's go and Later light as buttons do."),
  content: {
    text: [{ name: "The hero line", words: HERO, sample: screen((words: string) => setupRoom({ place: { at: "arrival" }, desk: true, hero: words })) }],
    missingPicture: doesntApply("the wallpaper is the desktop's own, on the disk since the install."),
    counts: [
      { name: "Four of six", for: "a desktop with no reader", sample: CELLS.disabled },
      { name: "Six", for: "the TUXEDO", sample: CELLS.rest },
    ],
  },
  motion: {
    arrives: [OPENS],
    leaves: [CARDS_IN],
    between: [BACK],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Let's go: the first card that wants you" },
      { keys: ["Esc"], does: "Later: the desktop; the Command menu keeps it all" },
      { keys: ["Tab", "Shift+Tab"], does: "Let's go, then Later" },
    ],
    order: "Let's go, then Later; the plan is read, not pressed.",
    lands: "On Let's go once the words are in; coming back, on Let's go.",
  },
  measures: {
    samples: nonEmpty([{ name: "First login", for: "the column, the plan, the actions", sample: CELLS.rest }]),
    measures: [
      { part: "The column", token: "size-column" },
      { part: "Accent → hero", token: "space-s8" },
      { part: "Hero → lede", token: "space-s16" },
      { part: "Lede → plan → actions", token: "space-s32" },
      { part: "Plan's cards apart", token: "space-s8" },
      { part: "Actions → keys", token: "space-s12" },
      { part: "Keys → promise", token: "space-s24" },
      { part: "Window's padding", token: "space-s32" },
    ],
    reflow: { note: "The column stays in the window's middle; the plan keeps two cards to a row.", sample: at("rest", { desk: false }) },
  },
});
