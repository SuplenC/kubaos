// First boot's Make it yours card, in three steps of its own. What for: Developer, Designer and
// Everyday as lit cards, any of them, all three or none. Your apps: every app of the catalogue by
// category, an index beside them, the picks' apps lit in a wave, each tick saying where it came
// from. Go: how many and which, then Install, which runs in the background while first boot goes on.

import type { Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { aloneAt, atCard, CARD_MEASURES, fullHd, liveCard } from "./setup-pages.ts";
import { setupRoom } from "./setup-room.ts";

type Shown = "rest" | "empty" | "selected" | "loading" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  empty: atCard("yours", "none"),
  selected: atCard("yours", "developer"),
  rest: atCard("yours", "apps"),
  loading: atCard("yours", "installing"),
  error: atCard("yours", "failed"),
};

const CARD = "the card";
const at = (view: string) => () => setupRoom({ place: { at: "card", card: "yours" }, views: { yours: view } });

/** The Developer card, picked, on What for. */
const DEVELOPER = ".setup-yours-profiles > .k-choice-card:nth-child(1)";

const strike = (curve: "neon.strike" | "standard", duration: "strike" | "short"): Track[] => [
  { part: "the card's light", on: CARD, target: DEVELOPER, property: "--choice-burn", from: 0, to: 1, duration, curve },
];

/** A profile picked: its card strikes on as the neon does, tube, glow, tile and foot together. */
const STRIKES: Move = {
  name: "A card strikes on",
  what: "Its light steps through `neon.strike` over `strike`: tube, glow, tint, tile and foot lines as one.",
  tracks: strike("neon.strike", "strike") as unknown as NonEmpty<Track>,
  unflickered: strike("standard", "short") as unknown as NonEmpty<Track<Easing>>,
  reduced: [{ part: "the card's light", on: CARD, target: DEVELOPER, property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: at("developer"),
};

/** A profile unpicked: its light goes out over `short`; nothing flickers going out. */
const GOES_OUT: Move = {
  name: "A card goes out",
  what: "Its light fades over `short` on `standard`, the cold tube of the pointer staying.",
  tracks: [{ part: "the card's light", on: CARD, target: DEVELOPER, property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: at("developer"),
};

/**
 * A step turning, the step left sliding `space.s24` off and the next in from the other side, a
 * `short` after: with Your apps, its picks' apps coming up in a wave, a `micro` apart.
 */
function turning(name: string, what: string, view: string, wave: boolean): Move {
  const waveTracks: Track[] = wave
    ? [0, 1, 2, 3, 4, 5, 6].map((index) => ({
        part: index === 6 ? "the rest of the wave" : `app ${index + 1} of the wave`,
        on: "the apps",
        target: `.setup-yours-step:not([data-leaving]) .k-list-row[data-wave="${index}"]`,
        property: "opacity",
        from: { opacity: "subdued" },
        to: 1,
        duration: "long",
        cue: ["short", ...Array<"micro">(index).fill("micro")] as unknown as NonEmpty<"short">,
        curve: "emphasized_decel",
      }))
    : [];
  return {
    name,
    what,
    tracks: [
      { part: "the step left", on: "the step left", target: ".setup-yours-step[data-leaving]", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
      { part: "its slide off", on: "the step left", target: ".setup-yours-step[data-leaving]", property: "--yours-slide", from: 0, to: -1, duration: "short", curve: "emphasized_accel" },
      { part: "the next step", on: "the step shown", target: ".setup-yours-step:not([data-leaving])", property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
      { part: "its slide in", on: "the step shown", target: ".setup-yours-step:not([data-leaving])", property: "--yours-slide", from: 1, to: 0, duration: "long", cue: "short", curve: "emphasized_decel" },
      ...waveTracks,
    ] as unknown as NonEmpty<Track<Easing>>,
    reduced: [
      { part: "the step left", on: "the step left", target: ".setup-yours-step[data-leaving]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" },
      { part: "the next step", on: "the step shown", target: ".setup-yours-step:not([data-leaving])", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    ],
    back: "cuts",
    sample: at(view),
  };
}

const TO_APPS = turning("What for → Your apps", "What for slides off; Your apps slides in, its picks' apps lighting in a wave.", "apps<0", true);
const TO_GO = turning("Your apps → Go", "Your apps slides off; Go slides in, the count and the chosen apps.", "go<1", false);
const BACK_STEP = turning("Back a step", "Back or Esc: Your apps slides off; What for slides back in.", "none<1", false);

const VSCODE_ROW = ".setup-yours-list .k-list-row:has(.k-chip[data-kind='Cold'])";

/** VS Code ticked by hand: its tick comes on and its line gives way to Kōbō's opinion of that. */
const VSCODE_TICKED: Move = {
  name: "VS Code ticked anyway",
  what: "Its tick and its new line fade in over `short`: “Bold. LazyVim is right there.”",
  tracks: [
    { part: "the tick", on: "VS Code's row", target: `${VSCODE_ROW} .k-list-check`, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the line", on: "VS Code's row", target: `${VSCODE_ROW} .k-list-title + *`, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [
    { part: "the tick", on: "VS Code's row", target: `${VSCODE_ROW} .k-list-check`, property: "opacity", from: 0, to: 1, duration: "instant", curve: "standard" },
    { part: "the line", on: "VS Code's row", target: `${VSCODE_ROW} .k-list-title + *`, property: "opacity", from: 0, to: 1, duration: "instant", curve: "standard" },
  ],
  back: "cuts",
  sample: at("vscode"),
};

/** The install finished: the figure says how many are in, and the rail's step takes its check. */
const FINISHED: Move = {
  name: "Installed",
  what: "The figure fades in over `short`, and the rail's check with it.",
  tracks: [
    { part: "the figure", on: CARD, target: ".setup-yours-figure", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the check", on: "the rail", target: ".k-rail-steps > .k-step:nth-child(1) > * > :last-child", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: "same",
  back: "cuts",
  sample: at("done"),
};

/** An app failed: its error block fades in over the groups, its chip in `warning`. */
const FAILED: Move = {
  name: "An app fails",
  what: "The error block fades in over `short`; Under the hood opens on the log.",
  tracks: [{ part: "the error block", on: CARD, target: ".setup-yours .k-error-block", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: at("failed"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-yours",
  title: "First boot: Make it yours",
  lede: "Three steps: what you use it for, every app by category, then go; it installs in the background.",
  code: { unbuilt: "Kōbō has no such card" },
  from: ["step-rail-page", "stepper", "choice-card", "list", "chip", "progress-bar", "error-block"],
  stage: { live: liveCard("yours"), hint: "Light a card, Enter, tick any, Enter, Enter installs", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "What for: nothing lit", for: "the newcomer's line; pick apps myself", sample: CELLS.empty },
    { name: "What for: Developer", for: "its card lit; 13 picked for you", sample: CELLS.selected },
    { name: "What for: all three", for: "all lit", sample: atCard("yours", "all") },
    { name: "Your apps", for: "Designer's lit; two yours", sample: CELLS.rest },
    { name: "Your apps: nothing", for: "every app, none ticked", sample: atCard("yours", "apps-none") },
    { name: "VS Code ticked", for: "its line has an opinion", sample: atCard("yours", "vscode") },
    { name: "Go", for: "the count and the apps", sample: atCard("yours", "go") },
    { name: "Go: no apps", for: "said plainly; Continue", sample: atCard("yours", "go-none") },
    { name: "Installing", for: "7 of 30 in; carry on", sample: CELLS.loading },
    { name: "Installed", for: "every app in", sample: atCard("yours", "done") },
    { name: "An app failed", for: "the cause; Try Steam again", sample: CELLS.error },
    { name: "From the Command menu", for: "the card alone, in a window", sample: aloneAt("yours", "apps") },
  ],
  sizes: [
    { name: "Laptop", for: "1280 × 800, Your apps", sample: atCard("yours", "apps") },
    { name: "Full HD", for: "1920 × 1080, Your apps", sample: fullHd("yours", "apps") },
  ],
  states: {
    shown: ["rest", "empty", "selected", "loading", "error"],
    absent: {
      lit: "its cards, rows and buttons light on their pages",
      pressed: "its cards and buttons press on their pages",
      disabled: "every machine has it: never passed by",
    },
    rows: [{ name: "Make it yours", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      empty: "What for, nothing lit",
      selected: "Developer's card lit",
      rest: "Your apps; picks lit, two yours",
      loading: "Go: the bar; a chip per app",
      error: "the error block; Try Steam again",
    },
  },
  focus: doesntApply("no line of its own: its cards, rows and buttons light on their pages."),
  content: {
    text: doesntApply("the apps' words are their metadata's; a row clamps its line."),
    missingPicture: doesntApply("stand-in symbols on tiles: an app's own icon fills its tile once metadata has it."),
    counts: [
      { name: "No app", for: "nothing picked", sample: atCard("yours", "go-none") },
      { name: "Twelve apps", for: "Designer and two", sample: atCard("yours", "go") },
      { name: "Thirty apps", for: "all three", sample: CELLS.loading },
    ],
  },
  motion: {
    arrives: [STRIKES, TO_APPS, TO_GO],
    leaves: [GOES_OUT, BACK_STEP],
    between: [VSCODE_TICKED, FINISHED, FAILED],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Next: the next step; at Go, Install" },
      { keys: ["Esc"], does: "back a step; from What for, back a card" },
      { keys: ["←", "→"], does: "What for: between the cards; Your apps: between apps" },
      { keys: ["↑", "↓"], does: "Your apps: the app above or below; in the index, a category" },
      { keys: ["Space"], does: "lights a card, or ticks an app" },
      { keys: ["Tab", "Shift+Tab"], does: "rail, the step's parts, the band" },
    ],
    order: "The rail, the cards; at Your apps the index, then the apps; Back, Skip, Next.",
    lands: "On the first card; a step turned keeps the keys on Next, so Enter thrice installs.",
  },
  measures: {
    samples: nonEmpty([{ name: "Your apps", for: "the stepper, the index and the apps", sample: CELLS.rest }]),
    measures: nonEmpty([
      ...CARD_MEASURES,
      { part: "Stepper → step", token: "space-s24" },
      { part: "Cards apart", token: "space-s16" },
      { part: "Index ↔ apps", token: "space-s24" },
      { part: "Apps apart", token: "space-s16" },
    ]),
    reflow: { note: "Apps flow into as many columns as fit; the index stays in view as they scroll.", sample: CELLS.rest },
  },
});
