// First boot's Kotodama card, in four steps of its own. What: the one key, what to type and Enter,
// beside the Command menu playing by itself. Why: one question raced through Start and
// Kotodama, each clock running until its answer lands. Yours: the keys that open it and what it
// finds, applied as they are made. Try it: the menu itself, three things to try ticking off.

import type { Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { aloneAt, atCard, CARD_MEASURES, fullHd, liveCard } from "./setup-pages.ts";
import { setupRoom } from "./setup-room.ts";

type Shown = "rest" | "selected" | "loading" | "empty";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("kotodama", "what"),
  selected: atCard("kotodama", "try-ticked"),
  loading: atCard("kotodama", "why"),
  empty: atCard("kotodama", "try"),
};

const at = (view: string) => () => setupRoom({ place: { at: "card", card: "kotodama" }, views: { kotodama: view } });

/** A step turning, as Make it yours's do: the step left slides `space.s24` off, the next in. */
function turning(name: string, what: string, view: string, forward = true): Move {
  const toward = forward ? -1 : 1;
  return {
    name,
    what,
    tracks: [
      { part: "the step left", on: "the step left", target: ".setup-kotodama-step[data-leaving]", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
      { part: "its slide off", on: "the step left", target: ".setup-kotodama-step[data-leaving]", property: "--kotodama-slide", from: 0, to: toward, duration: "short", curve: "emphasized_accel" },
      { part: "the next step", on: "the step shown", target: ".setup-kotodama-step:not([data-leaving])", property: "opacity", from: 0, to: 1, duration: "long", cue: "short", curve: "emphasized_decel" },
      { part: "its slide in", on: "the step shown", target: ".setup-kotodama-step:not([data-leaving])", property: "--kotodama-slide", from: -toward, to: 0, duration: "long", cue: "short", curve: "emphasized_decel" },
    ],
    reduced: [
      { part: "the step left", on: "the step left", target: ".setup-kotodama-step[data-leaving]", property: "opacity", from: 1, to: 0, duration: "instant", curve: "standard" },
      { part: "the next step", on: "the step shown", target: ".setup-kotodama-step:not([data-leaving])", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
    ],
    back: "cuts",
    sample: at(view),
  };
}

const TO_WHY = turning("What → Why", "What slides off; Why slides in and the race starts.", "why<0");
const TO_YOURS = turning("Why → Yours", "Why slides off; Yours slides in, its settings.", "yours<1");
const TO_TRY = turning("Yours → Try it", "Yours slides off; Try it slides in, the menu shut until its keys.", "try<2");
const BACK_STEP = turning("Back a step", "Back or Esc: Yours slides off to the right; Why slides back in.", "why<2", false);

const lane = (name: string) => `[data-lane="${name}"]`;

/**
 * The race, after the question's last key is let go: Kotodama's clock has stopped at once; the
 * others' answers land in their own time, each rising `space.s16` over `long`. Its timing is the
 * stand-in's, worked out from `LANDS` in milliseconds, so the recipe names the `long` it rises over.
 */
const RACE: Move = {
  name: "The race",
  what: "After the key: Start's web at 1 s, the ad at 3 s, the weather at 3.6 s, what you wanted at 4.4 s.",
  tracks: [
    { part: "Start's web rows", on: "Start", target: `${lane("start")} .setup-kotodama-answers > :first-child`, property: "opacity", from: 0, to: 1, duration: "long", cue: ["slow", "micro"] as unknown as NonEmpty<"slow">, curve: "emphasized_decel" },
    { part: "Start's ad", on: "Start", target: `${lane("start")} .setup-kotodama-ad`, property: "opacity", from: 0, to: 1, duration: "long", cue: ["scene", "long"] as unknown as NonEmpty<"scene">, curve: "emphasized_decel" },
  ] as unknown as NonEmpty<Track<Easing>>,
  reduced: [{ part: "the lanes", on: "the race", target: ".setup-kotodama-race", property: "opacity", from: 1, to: 1, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: at("why"),
};

/** A thing tried: its row's tick comes on and the row lights, over `short`. */
const TICKED: Move = {
  name: "A try ticks off",
  what: "Its tick fades in over `short`; the row's tile takes the accent.",
  tracks: [{ part: "the tick", on: "the try", target: ".setup-kotodama-try-card[data-done] .setup-kotodama-tick", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: at("try-ticked"),
};

/** The menu opening on Try it, as it opens on the desktop: down `space.s8`, fading in. */
const OPENS: Move = {
  name: "The menu opens",
  what: "It drops `space.s8` into place over `medium` on `emphasized_decel`; with the slide off, it only fades.",
  tracks: [
    { part: "the panel", on: "the menu", target: ".setup-kotodama-menu", property: "opacity", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "the panel", on: "the menu", target: ".setup-kotodama-menu", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: at("try-open"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-kotodama",
  title: "First boot: Kotodama",
  lede: "Four steps: what the Command menu is, why, your settings, then the menu itself to try.",
  code: { unbuilt: "Kōbō has no such card" },
  from: ["step-rail-page", "stepper", "list-detail", "list", "choice-card", "collapsible", "select", "keycap", "toast"],
  stage: { live: liveCard("kotodama"), hint: "Enter goes on: What, Why, Yours, Try it", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "What", for: "the key; the menu playing by itself", sample: CELLS.rest },
    { name: "Why", for: "the race run out; the clocks", sample: CELLS.loading },
    { name: "Why: mid-race", for: "3.3 s in: the ad up", sample: atCard("kotodama", "why-mid") },
    { name: "Why: the term asked", for: "Bofa's answer, shown", sample: atCard("kotodama", "why-tip") },
    { name: "Why: the ad pressed", for: "its reply, shown", sample: atCard("kotodama", "why-reply") },
    { name: "Yours", for: "what it finds; the keys, the engines", sample: atCard("kotodama", "yours") },
    { name: "Yours: changed", for: "Alt + Space, no web search", sample: atCard("kotodama", "yours-changed") },
    { name: "Try it: shut", for: "its keys; Open it", sample: CELLS.empty },
    { name: "Try it: open", for: "the menu; one ticked", sample: atCard("kotodama", "try-open") },
    { name: "Try it: wifi", for: "the setting, first row", sample: atCard("kotodama", "try-typed") },
    { name: "Try it: all three", for: "the sum; Next: Git", sample: CELLS.selected },
    { name: "From the Command menu", for: "“Set up the Command menu”, in a window", sample: aloneAt("kotodama", "try-open") },
  ],
  sizes: [
    { name: "Laptop", for: "1280 × 800, the race", sample: atCard("kotodama", "why") },
    { name: "Between", for: "1520 × 1020, the race", sample: atCard("kotodama", "why", { width: "calc(1520 * var(--px))", height: "calc(1020 * var(--px))" }) },
    { name: "Full HD", for: "1920 × 1080, the race", sample: fullHd("kotodama", "why") },
    { name: "Full HD: Try it", for: "the menu with its pane", sample: fullHd("kotodama", "try-typed") },
  ],
  states: {
    shown: ["rest", "selected", "loading", "empty"],
    absent: {
      lit: "its cards, rows and buttons light on their pages",
      pressed: "its cards and buttons press on their pages",
      disabled: "every machine has it: never passed by",
      error: "it runs nothing that can fail",
    },
    rows: [{ name: "Kotodama", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "What: the key and the menu playing",
      selected: "Try it, all three tried",
      loading: "Why: the race running",
      empty: "Try it, the menu shut",
    },
  },
  focus: doesntApply("no line of its own: its cards, rows, fields and buttons light on their pages."),
  content: {
    text: doesntApply("the menu's words are its Items'; a row clamps its line."),
    missingPicture: doesntApply("stand-in symbols on tiles, as the menu's own rows draw them."),
    counts: [
      { name: "None tried", for: "the menu shut", sample: CELLS.empty },
      { name: "One tried", for: "opened", sample: atCard("kotodama", "try-open") },
      { name: "All three", for: "the note says so", sample: CELLS.selected },
    ],
  },
  motion: {
    arrives: [TO_WHY, TO_YOURS, TO_TRY, OPENS],
    leaves: [BACK_STEP],
    between: [RACE, TICKED],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Next: the next step; in the menu, runs the Result" },
      { keys: ["Esc"], does: "back a step; in the menu, clears, then shuts it" },
      { keys: ["Super+Space"], does: "Try it: opens the menu, as on the desktop" },
      { keys: ["↑", "↓"], does: "in the menu, the Result above or below" },
      { keys: ["Tab", "Shift+Tab"], does: "rail, the step's parts, the band" },
    ],
    order: "The rail; the step's parts; Back, Skip, Next.",
    lands: "On Next, so Enter four times goes through to Git; opened, the menu's field.",
  },
  measures: {
    samples: nonEmpty([{ name: "Try it", for: "the tries and the menu", sample: CELLS.selected }]),
    measures: nonEmpty([
      ...CARD_MEASURES,
      { part: "Stepper → step", token: "space-s24" },
      { part: "Tries ↔ menu", token: "space-s24" },
      { part: "Lanes apart", token: "space-s16" },
      { part: "Between Yours's sections", token: "space-s20" },
    ]),
    reflow: { note: "The menu takes its room's width and the rows its height holds; its pane shows once it is wide enough.", sample: CELLS.selected },
  },
});
