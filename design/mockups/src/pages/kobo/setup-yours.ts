// First boot's Make it yours card: what the person uses the machine for (Developer, Designer,
// Everyday: any of them, all three or none), the apps that brings as a live list grouped by the
// profile that brought each, any of them untick-able and more addable, and beside the list the
// settings that come with them: Git's name and email with Developer, the Bitwarden login with
// Bitwarden. Install runs in the background while first boot goes on.

import type { Move, NonEmpty, Track } from "../../site/page.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { refused } from "./kobo-pages.ts";
import { APPS } from "./setup-data.ts";
import { aloneAt, atCard, fullHd, CARD_KEYS, CARD_MEASURES, finishesAt, liveCard } from "./setup-pages.ts";
import { FAILS, setupRoom } from "./setup-room.ts";

type Shown = "rest" | "empty" | "selected" | "loading" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  empty: atCard("yours", "none"),
  rest: atCard("yours", "developer"),
  selected: atCard("yours", "all"),
  loading: atCard("yours", "installing"),
  error: atCard("yours", "failed"),
};

const CARD = "the card";

/** The rows of the apps only Designer brings, with Developer picked too, and its label. */
const DESIGNER_ONLY = `.k-live-row:is([data-key="group-designer"], ${APPS.filter((each) => each.profiles.includes("designer") && !each.profiles.includes("developer"))
  .map((each) => `[data-key="${each.id}"]`)
  .join(", ")})`;

/** The first pick on the empty card: the list and the settings rise in under the choices, the rows opening in the list. */
const PICKED: Move = {
  name: "The first pick",
  what: "The list and its settings rise in as one over `long`; the rows open their room over `medium` and fade in over `slow`.",
  tracks: [
    { part: "the columns' fade", on: CARD, target: ".setup-yours-columns", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the columns' rise", on: CARD, target: ".setup-yours-columns", property: "--kobo-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "the rows' room", on: CARD, target: ".setup-yours-apps .k-live-row", property: "--grow", from: 0, to: 1, duration: "medium", curve: "standard_decel" },
    { part: "the rows' fade", on: CARD, target: ".setup-yours-apps .k-live-row", property: "opacity", from: 0, to: 1, duration: "slow", curve: "standard_decel" },
  ] as unknown as NonEmpty<Track<"standard_decel">>,
  reduced: [
    { part: "the columns' fade", on: CARD, target: ".setup-yours-columns", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the rows' fade", on: CARD, target: ".setup-yours-apps .k-live-row", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "yours" }, views: { yours: "developer" } }),
};

/** Developer picked beside another: Git rises in beside the list as Developer's apps arrive in it. */
const GIT: Move = {
  name: "Developer picked: Git",
  what: "Git rises in over `long` where it lands, Bitwarden under it moving down as its rows open.",
  tracks: [
    { part: "Git's fade", on: CARD, target: ".setup-yours-git", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "Git's rise", on: CARD, target: ".setup-yours-git", property: "--kobo-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "Git's fade", on: CARD, target: ".setup-yours-git", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "yours" }, views: { yours: "two" } }),
};

/** Designer unpicked beside Developer: what only it brought closes up; what both bring stays. */
const UNPICKED: Move = {
  name: "A profile unpicked",
  what: "What only it brought fades and gives its room back over `medium`; an app another pick shares stays.",
  tracks: [
    { part: "the rows' room", on: CARD, target: DESIGNER_ONLY, property: "--grow", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
    { part: "the rows' fade", on: CARD, target: DESIGNER_ONLY, property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
  ],
  reduced: [
    { part: "the rows' fade", on: CARD, target: DESIGNER_ONLY, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the rows' room", on: CARD, target: DESIGNER_ONLY, property: "--grow", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => setupRoom({ place: { at: "card", card: "yours" }, views: { yours: "two" } }),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-yours",
  title: "First boot: Make it yours",
  lede: "What you use it for, the apps that brings, and the settings that come with them.",
  code: { unbuilt: "Kōbō has no such card" },
  from: ["step-rail-page", "choice-card", "live-list", "field", "select", "progress-bar", "error-block"],
  stage: { live: liveCard("yours"), hint: "Pick any; untick an app; Enter installs in the background", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "Nothing picked", for: "the three choices; the well waits", sample: CELLS.empty },
    { name: "Developer", for: "its apps; Git, email empty", sample: CELLS.rest },
    { name: "Git filled", for: "and Bitwarden's login", sample: atCard("yours", "filled") },
    { name: "Not an email", for: "the field Invalid; Install waits", sample: atCard("yours", "invalid") },
    { name: "All three", for: "grouped; shared apps once", sample: CELLS.selected },
    { name: "Unticked and added", for: "two dropped; More apps open", sample: atCard("yours", "dropped") },
    { name: "Installing", for: "7 of 20 in; carry on", sample: CELLS.loading },
    { name: "Installed", for: "every app in", sample: atCard("yours", "done") },
    { name: "An app failed", for: "the cause; Try it again", sample: CELLS.error },
    { name: "From the Command menu", for: "the card alone", sample: aloneAt("yours", "done") },
  ],
  sizes: [
    { name: "Laptop", for: "1280 × 800, all three", sample: atCard("yours", "all") },
    { name: "Full HD", for: "1920 × 1080, all three", sample: fullHd("yours", "all") },
  ],
  states: {
    shown: ["rest", "empty", "selected", "loading", "error"],
    absent: {
      lit: "its cards, rows and fields light on their pages",
      pressed: "its cards and buttons press on their pages",
      disabled: "every machine has it: never passed by",
    },
    rows: [{ name: "Make it yours", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      empty: "nothing picked; the well's words",
      rest: "one picked; its apps ticked",
      selected: "all three; four groups",
      loading: "Running; the bar; each app's chip",
      error: "the error block; Failed on its row",
    },
  },
  focus: doesntApply("no line of its own: its cards, rows and fields light on their pages."),
  content: {
    text: doesntApply("the apps' words are their metadata's; a row clamps its line."),
    missingPicture: doesntApply("stand-in symbols on tiles: an app's own icon fills its tile once metadata has it."),
    counts: [
      { name: "No app", for: "nothing picked", sample: CELLS.empty },
      { name: "Eight apps", for: "one profile", sample: CELLS.rest },
      { name: "Twenty apps", for: "all three; the well scrolls", sample: CELLS.selected },
    ],
  },
  motion: {
    arrives: [PICKED],
    leaves: [UNPICKED],
    between: [GIT, refused(atCard("yours", "invalid")), finishesAt("yours", "done"), FAILS],
  },
  keyboard: {
    keys: [
      ...CARD_KEYS,
      { keys: ["←", "→"], does: "between the three choices" },
      { keys: ["Space"], does: "picks a choice, or ticks an app" },
    ],
    order: "The rail, the choices, More apps, the apps, Git, Bitwarden, Under the hood, the band.",
    lands: "On the first choice: nothing is picked for you.",
  },
  measures: {
    samples: nonEmpty([{ name: "All three", for: "the choices, the list and the settings", sample: CELLS.selected }]),
    measures: nonEmpty([
      ...CARD_MEASURES,
      { part: "Choices apart", token: "space-s16" },
      { part: "List ↔ settings", token: "space-s24" },
      { part: "Settings apart", token: "space-s32" },
      { part: "Fields apart", token: "space-s16" },
    ]),
    reflow: { note: "Two columns under the choices; the page scrolls under its header.", sample: CELLS.selected },
  },
});
