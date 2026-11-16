// First boot's Kotodama card, in four steps of its own. What: the one key, what to type and Enter,
// beside the Command menu playing by itself. Why: one question raced through Start, Spotlight and
// Kotodama, each clock running until its answer lands. Yours: the keys that open it and what it
// finds, beside a preview. Try it: the menu itself, three things to try ticking off. Super tapped
// alone, on any step, brings up the other menu, stamped "Nope.".

import type { Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { aloneAt, atCard, CARD_MEASURES, fullHd, liveCard } from "./setup-pages.ts";
import { setupRoom } from "./setup-room.ts";

type Shown = "rest" | "selected" | "loading" | "empty" | "error";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("kotodama", "what"),
  selected: atCard("kotodama", "try-ticked"),
  loading: atCard("kotodama", "why"),
  empty: atCard("kotodama", "try"),
  error: atCard("kotodama", "gag"),
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
const TO_YOURS = turning("Why → Yours", "Why slides off; Yours slides in, the preview on “fi”.", "yours<1");
const TO_TRY = turning("Yours → Try it", "Yours slides off; Try it slides in, the menu shut until its keys.", "try<2");
const BACK_STEP = turning("Back a step", "Back or Esc: Yours slides off to the right; Why slides back in.", "why<2", false);

const lane = (name: string) => `.setup-kotodama-lane[data-lane="${name}"]`;

/**
 * The race, after the question's last key is let go: Kotodama's clock has stopped at once; the
 * others' answers land in their own time, each rising `space.s16` over `long`. Its timing is the
 * stand-in's, worked out from `LANDS` in milliseconds, so the recipe names the `long` it rises over.
 */
const RACE: Move = {
  name: "The race",
  what: "After the last key: Start's web at 0.9 s, its ad at 1.7 s, weather at 2.3 s; Spotlight's website at 1.1 s.",
  tracks: [
    { part: "Start's web rows", on: "Start", target: `${lane("start")} .setup-kotodama-answers > :first-child`, property: "opacity", from: 0, to: 1, duration: "long", cue: ["slow", "micro"] as unknown as NonEmpty<"slow">, curve: "emphasized_decel" },
    { part: "Start's ad", on: "Start", target: `${lane("start")} .setup-kotodama-ad`, property: "opacity", from: 0, to: 1, duration: "long", cue: ["scene", "long"] as unknown as NonEmpty<"scene">, curve: "emphasized_decel" },
    { part: "Spotlight's website", on: "Spotlight", target: `${lane("spotlight")} .setup-kotodama-answers > :first-child`, property: "opacity", from: 0, to: 1, duration: "long", cue: ["slow", "medium"] as unknown as NonEmpty<"slow">, curve: "emphasized_decel" },
  ] as unknown as NonEmpty<Track<Easing>>,
  reduced: [{ part: "the lanes", on: "the race", target: ".setup-kotodama-lane", property: "opacity", from: 1, to: 1, duration: "instant", curve: "standard" }],
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

/** Super alone: the other menu rises in the corner, is stamped, and goes back down. */
const NOPE: Move = {
  name: "Super alone",
  what: "The other menu rises `space.s32` over `long`; “Nope.” pops on over `medium` a `slow` later; it leaves after three holds.",
  tracks: [
    { part: "the other menu", on: "the gag", target: ".setup-kotodama-gag", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the stamp", on: "the gag", target: ".setup-kotodama-stamp", property: "opacity", from: 0, to: 1, duration: "medium", cue: "slow", curve: "pop" },
  ],
  reduced: [{ part: "the other menu", on: "the gag", target: ".setup-kotodama-gag", property: "opacity", from: 1, to: 1, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: CELLS.error,
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-kotodama",
  title: "First boot: Kotodama",
  lede: "Four steps: what the Command menu is, why, your settings, then the menu itself to try.",
  code: { unbuilt: "Kōbō has no such card" },
  from: ["step-rail-page", "stepper", "list-detail", "list", "choice-card", "collapsible", "select", "keycap", "toast"],
  stage: { live: liveCard("kotodama"), hint: "Enter goes on: What, Why, Yours, Try it; Super alone anywhere", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "What", for: "the key; the menu playing by itself", sample: CELLS.rest },
    { name: "Why", for: "the race; the clocks", sample: CELLS.loading },
    { name: "Yours", for: "the keys, three switches, the preview", sample: atCard("kotodama", "yours") },
    { name: "Yours: more", for: "what it finds, the engine, the mark", sample: atCard("kotodama", "yours-more") },
    { name: "Yours: changed", for: "Alt + Space, no web search", sample: atCard("kotodama", "yours-changed") },
    { name: "Try it: shut", for: "its keys; Open it", sample: CELLS.empty },
    { name: "Try it: open", for: "the menu; one ticked", sample: atCard("kotodama", "try-open") },
    { name: "Try it: wifi", for: "the setting, first row", sample: atCard("kotodama", "try-typed") },
    { name: "Try it: all three", for: "the sum; Next: Git", sample: CELLS.selected },
    { name: "Super alone", for: "the other menu, stamped", sample: CELLS.error },
    { name: "From the Command menu", for: "“Set up the Command menu”, in a window", sample: aloneAt("kotodama", "yours") },
  ],
  sizes: [
    { name: "Laptop", for: "1280 × 800, the race", sample: atCard("kotodama", "why") },
    { name: "Full HD", for: "1920 × 1080, the race", sample: fullHd("kotodama", "why") },
    { name: "Full HD: Try it", for: "the menu with its pane", sample: fullHd("kotodama", "try-typed") },
  ],
  states: {
    shown: ["rest", "selected", "loading", "empty", "error"],
    absent: {
      lit: "its cards, rows and buttons light on their pages",
      pressed: "its cards and buttons press on their pages",
      disabled: "every machine has it: never passed by",
    },
    rows: [{ name: "Kotodama", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "What: the key and the menu playing",
      selected: "Try it, all three tried",
      loading: "Why: the race running",
      empty: "Try it, the menu shut",
      error: "Super alone: “Nope.”",
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
    between: [RACE, TICKED, NOPE],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Next: the next step; in the menu, runs the Result" },
      { keys: ["Esc"], does: "back a step; in the menu, clears, then shuts it" },
      { keys: ["Super+Space"], does: "Try it: opens the menu, as on the desktop" },
      { keys: ["Super"], does: "tapped alone, anywhere: the other menu, stamped" },
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
      { part: "Settings ↔ preview", token: "space-s24" },
    ]),
    reflow: { note: "The menu takes its room's width and the rows its height holds; its pane shows once it is wide enough.", sample: CELLS.selected },
  },
});
