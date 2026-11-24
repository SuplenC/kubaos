// First boot's Kotodama card, in four steps of its own. What: the one key, what to type and Enter,
// beside the Command menu playing by itself. Why: one question raced through Start and
// Kotodama, each clock running until its answer lands. Yours: the keys that open it and what it
// finds, live as they are made. Try it: the keys picked, large, and the menu they open over the
// whole screen.

import type { Easing, Move, NonEmpty, Track } from "../../site/page.ts";
import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { aloneAt, atCard, CARD_MEASURES, fullHd, liveCard } from "./setup-pages.ts";
import { setupRoom } from "./setup-room.ts";

type Shown = "rest" | "selected" | "loading";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: atCard("kotodama", "what"),
  selected: atCard("kotodama", "try-open"),
  loading: atCard("kotodama", "why"),
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
const TO_TRY = turning("Yours → Try it", "Yours slides off; Try it slides in, its keys and its two lines.", "try<2");
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

/** What plays by itself: an example lights as the menu beside it types it; the one before recedes. */
const DEMO: Move = {
  name: "What plays",
  what: "The next example lights over `medium` as the menu types it; the last recedes to `opacity.subdued`.",
  tracks: [
    { part: "the example typed", on: "the examples", target: ".setup-kotodama-example:nth-child(2)", property: "opacity", from: { opacity: "subdued" }, to: 1, duration: "medium", curve: "standard" },
    { part: "the one before", on: "the examples", target: ".setup-kotodama-example:nth-child(1)", property: "opacity", from: 1, to: { opacity: "subdued" }, duration: "medium", curve: "standard" },
  ],
  reduced: [{ part: "the example typed", on: "the examples", target: ".setup-kotodama-example:nth-child(2)", property: "opacity", from: { opacity: "subdued" }, to: 1, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: at("what"),
};

/** The ad's button pressed: its reply fades in above it, holds, and goes. */
const ANSWERS: Move = {
  name: "The ad answers",
  what: "Its reply fades in over `short` above the button, holds two `hold`s, then fades out.",
  tracks: [{ part: "the reply", on: "the ad's button", target: ".setup-kotodama-ad-reply > .k-tooltip", property: "--tooltip-shown", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "cuts",
  sample: at("why-reply"),
};

/** Try it's keys, `+` between them, as the key prompt lays them out. */
const PART = (at: number) => `.setup-kotodama-keys > :nth-child(${at})`;

/**
 * Try it arriving with its turn: the caps rise `space.s8` and fade in over `long`, a `micro` apart
 * after the step's `short`, the `+` in its turn; the two lines a `micro` after the last cap.
 */
const KEYS_ARRIVE: Move = {
  name: "The keys arrive",
  what: "With the step's turn, each cap rises `space.s8` and fades in over `long`, a `micro` after the one before; the lines follow.",
  tracks: [
    ...TO_TRY.tracks,
    ...[1, 2, 3].flatMap((at) => [
      { part: at === 2 ? "the plus" : `cap ${at === 1 ? 1 : 2}`, on: "the keys", target: PART(at), property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", ...Array<"micro">(at - 1).fill("micro")] as unknown as NonEmpty<"short">, curve: "emphasized_decel" },
      { part: `${at === 2 ? "the plus" : `cap ${at === 1 ? 1 : 2}`}'s rise`, on: "the keys", target: PART(at), property: "--kotodama-rise", from: 1, to: 0, duration: "long", cue: ["short", ...Array<"micro">(at - 1).fill("micro")] as unknown as NonEmpty<"short">, curve: "emphasized_decel" },
    ]),
    { part: "the lines", on: "the words", target: ".setup-kotodama-line", property: "opacity", from: 0, to: 1, duration: "long", cue: ["short", "micro", "micro", "micro"] as unknown as NonEmpty<"short">, curve: "emphasized_decel" },
  ] as unknown as NonEmpty<Track<Easing>>,
  reduced: TO_TRY.reduced,
  back: "cuts",
  sample: at("try<2"),
};

/** The keys pressed: each cap's fill lifts and its glow swells to `focus.pressed_glow`, then settles. */
const KEYS_PRESS: Move = {
  name: "The keys press",
  what: "Pressed, each cap lifts its fill and swells its glow over `micro`, and settles over `short`.",
  tracks: [
    { part: "pressed", on: "the keys", target: ".setup-kotodama-keys > .k-key-prompt-cap", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: "let go", on: "the keys", target: ".setup-kotodama-keys > .k-key-prompt-cap", property: "--press", from: 1, to: 0, duration: "short", cue: "micro", curve: "standard" },
  ],
  reduced: "same",
  back: "cuts",
  sample: at("try"),
};

/** The menu opening over the screen, as it opens on the desktop: down `space.s8`, fading in. */
const OPENS: Move = {
  name: "The menu opens",
  what: "It drops `space.s8` into place over `medium` on `emphasized_decel`; with the slide off, it only fades.",
  tracks: [
    { part: "the panel", on: "the menu", target: ".setup-kotodama-at", property: "opacity", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" },
  ],
  reduced: [{ part: "the panel", on: "the menu", target: ".setup-kotodama-at", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: at("try-open"),
};

export default page<Shown>({
  level: "kobo",
  slug: "setup-kotodama",
  title: "First boot: Kotodama",
  lede: "Four steps: what the Command menu is, why, your settings, live as you make them, then its keys.",
  code: { unbuilt: "Kōbō has no such card" },
  from: ["step-rail-page", "stepper", "list-detail", "list", "choice-card", "collapsible", "select", "keycap", "toast"],
  stage: { live: liveCard("kotodama"), hint: "Enter goes on: What, Why, Yours, Try it", daylight: doesntApply("first boot is a night place, as Kōbō is; a card opened alone is a window in the desktop's mode.") },
  variants: [
    { name: "What", for: "the key; the menu playing by itself", sample: CELLS.rest },
    { name: "Why", for: "the race run out; the clocks", sample: CELLS.loading },
    { name: "Why: mid-race", for: "3.3 s in: the ad up", sample: atCard("kotodama", "why-mid") },
    { name: "Why: the term asked", for: "Bofa's answer, shown", sample: atCard("kotodama", "why-tip") },
    { name: "Why: the ad pressed", for: "its reply, shown", sample: atCard("kotodama", "why-reply") },
    { name: "Yours", for: "what it finds; the keys, the engines; live, said at its foot", sample: atCard("kotodama", "yours") },
    { name: "Yours: changed", for: "Alt + Space, no web search; its foot follows", sample: atCard("kotodama", "yours-changed") },
    { name: "Try it", for: "the keys picked, large; it's already yours", sample: atCard("kotodama", "try") },
    { name: "Try it: Alt + Space", for: "another Opens with, its keys", sample: atCard("kotodama", "try-alt") },
    { name: "Try it: the menu open", for: "over the whole screen; Esc shuts it", sample: CELLS.selected },
    { name: "From the Command menu", for: "“Set up the Command menu”, in a window", sample: aloneAt("kotodama", "try") },
    { name: "From the Command menu: open", for: "the menu over the desktop and the window", sample: aloneAt("kotodama", "try-open") },
  ],
  sizes: [
    { name: "Laptop", for: "1280 × 800, the race", sample: atCard("kotodama", "why") },
    { name: "Between", for: "1520 × 1020, the race", sample: atCard("kotodama", "why", { width: "calc(1520 * var(--px))", height: "calc(1020 * var(--px))" }) },
    { name: "Full HD", for: "1920 × 1080, the race", sample: fullHd("kotodama", "why") },
    { name: "Full HD: Try it", for: "the keys and the lines", sample: fullHd("kotodama", "try") },
    { name: "Full HD: the menu open", for: "over the screen, with its pane", sample: fullHd("kotodama", "try-open") },
  ],
  states: {
    shown: ["rest", "selected", "loading"],
    absent: {
      lit: "its cards, rows and buttons light on their pages",
      pressed: "its cards and buttons press on their pages",
      disabled: "every machine has it: never passed by",
      error: "it runs nothing that can fail",
      empty: "every step holds its content",
    },
    rows: [{ name: "Kotodama", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "What: the key and the menu playing",
      selected: "Try it, the menu open over the screen",
      loading: "Why: the race running",
    },
  },
  focus: doesntApply("no line of its own: its cards, rows, fields and buttons light on their pages."),
  content: {
    text: doesntApply("the menu's words are its Items'; a row clamps its line."),
    missingPicture: doesntApply("stand-in symbols on tiles, as the menu's own rows draw them."),
    counts: doesntApply("nothing on it counts: Try it is one set of keys."),
  },
  motion: {
    arrives: [TO_WHY, TO_YOURS, TO_TRY, KEYS_ARRIVE, OPENS],
    leaves: [BACK_STEP],
    between: [DEMO, RACE, ANSWERS, KEYS_PRESS],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Next: the next step; in the menu, runs the Result" },
      { keys: ["Esc"], does: "back a step; in the menu, clears, then shuts it" },
      { keys: ["Super+Space"], does: "the keys picked: the menu over the screen, from any step" },
      { keys: ["↑", "↓"], does: "in the menu, the Result above or below" },
      { keys: ["Tab", "Shift+Tab"], does: "rail, the step's parts, the band" },
    ],
    order: "The rail; the step's parts; Back, Skip, Next.",
    lands: "On Next, so Enter four times goes through to Git; opened, the menu's field; shut, the card.",
  },
  measures: {
    samples: nonEmpty([{ name: "Try it", for: "the keys and the two lines", sample: atCard("kotodama", "try") }]),
    measures: nonEmpty([
      ...CARD_MEASURES,
      { part: "Stepper → step", token: "space-s24" },
      { part: "Keys → headline", token: "space-s40" },
      { part: "Headline → line", token: "space-s12" },
      { part: "Lanes apart", token: "space-s24" },
      { part: "Name → menu → line", token: "space-s12" },
      { part: "Between Yours's sections", token: "space-s20" },
    ]),
    reflow: { note: "Try it's group stays two fifths down its room; the menu takes the screen's measures, its pane once it is wide enough.", sample: CELLS.selected },
  },
});
