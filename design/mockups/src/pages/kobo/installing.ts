// Kōbō's install: a scene of its own, not a question. The street comes back into focus at a
// scrim's dark and lights up as the system arrives; one figure, one bar, what it is doing, and
// something to learn while it waits. The steps and the log are a key away, on glass under Install
// in the rail, as is a failure.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { filledAnswers } from "./kobo-parts.ts";
import { ADVANCING, DETAILS, FAILING, FINISHING, LAPTOP, room, STARTING, TIP } from "./kobo-room.ts";

type Shown = "rest" | "loading" | "error";

const scene = (share: number, tip = 0, more: Partial<Parameters<typeof room>[0]> = {}) => screen(() => room({ at: { scene: "installing", share, tip }, ...more }));

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: scene(0.41),
  loading: scene(0),
  error: screen(() => room({ at: { panel: "failed" } })),
};

export default page<Shown>({
  level: "kobo",
  slug: "installing",
  title: "Installing",
  lede: "A scene of its own: the street lights up as the system arrives, the steps and the log a key away.",
  code: { unbuilt: "Kōbō draws it in its panel" },
  from: ["backdrop", "rolling-number", "progress-bar", "keycap", "step-rail-page", "step", "log", "error-block", "dialog"],
  stage: {
    live: screen(() => room({ at: { panel: "review" }, answers: filledAnswers(LAPTOP), live: true, width: windowSize().width, height: windowSize().height })),
    hint: "hold Enter on Last look to start it",
    daylight: doesntApply("the live ISO is always dark: the installer has no day."),
  },
  variants: [
    { name: "Just started", for: "0%, a scrim's dark; no time yet, its line held", sample: CELLS.loading },
    { name: "Copying the system", for: "41%: its figures in `primary`", sample: CELLS.rest },
    { name: "Nearly there", for: "93%: the street almost lit", sample: scene(0.93, 2) },
    { name: "The last of it", for: "Under a minute left", sample: scene(0.99, 3) },
    { name: "Another tip", for: "Super+Q, in the same place", sample: scene(0.62, 2) },
    { name: "The steps and the log", for: "on glass, on Install, the rail locked", sample: screen(() => room({ at: { panel: "details" } })) },
    { name: "Failed, next to Windows", for: "what broke, Windows untouched, the log", sample: CELLS.error },
    { name: "Failed, erasing", for: "the disk won't boot until it's installed again", sample: screen(() => room({ at: { panel: "failed" }, answers: filledAnswers(LAPTOP, { mode: "erase" }) })) },
    { name: "Power, mid-install", for: "a Danger dialog: Stop the install?", sample: scene(0.41, 0, { power: "ask" }) },
    { name: "Stopping", for: "no buttons: it unwinds, then goes down", sample: scene(0.41, 0, { power: "stopping" }) },
  ],
  sizes: [{ name: "Full HD", for: "the column in the middle, the street round it", sample: screen(() => room({ at: { scene: "installing", share: 0.41 }, width: "calc(1920 * var(--px))", height: "calc(1080 * var(--px))" })) }],
  states: {
    shown: ["rest", "loading", "error"],
    absent: { lit: "its one button lights as a Text button", pressed: "it presses as a Text button", disabled: "nothing here waits on a condition", selected: "nothing to pick", empty: "an install always has its steps" },
    rows: [{ name: "The install", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "the figure, the bar, the step and its detail",
      loading: "0%: Checking the machine; the room dark",
      error: "on glass: what broke, the disk's state, the log",
    },
  },
  focus: doesntApply("no line of its own: Show the steps lights as a Text button."),
  content: {
    text: doesntApply("its words are the engine's steps, each a few words, and the tips."),
    missingPicture: doesntApply("the wallpaper ships inside Kōbō."),
    counts: doesntApply("always the engine's eleven steps."),
  },
  motion: {
    arrives: [STARTING],
    leaves: [FINISHING, FAILING],
    between: [ADVANCING, TIP, DETAILS],
  },
  keyboard: {
    keys: [
      { keys: ["Tab"], does: "to Show the steps and the log, then Power" },
      { keys: ["Enter"], does: "the focused button" },
      { keys: ["Esc"], does: "in the steps and the log, back to the scene" },
    ],
    order: "Show the steps and the log, then Power; failed, Back to the review, Show log, Save log, Try again.",
    lands: "On Show the steps and the log; failed, on Try again.",
  },
  measures: {
    samples: nonEmpty([{ name: "Copying the system", for: "the column and its gaps", sample: CELLS.rest }]),
    measures: [
      { part: "The column", token: "size-column" },
      { part: "Accent → figure → lede", token: "space-s8" },
      { part: "Lede → time left", token: "space-s32" },
      { part: "Time → bar → step", token: "space-s8" },
      { part: "Figures under a long step", token: "space-s4" },
      { part: "Bar → tips → action", token: "space-s40" },
      { part: "Tip's label → tip", token: "space-s8" },
      { part: "Keys → what they do", token: "space-s12" },
    ],
    reflow: { note: "The column stays in the middle at every screen size; the street fills the rest.", sample: screen(() => room({ at: { scene: "installing", share: 0.41 }, width: windowSize().width, height: windowSize().height })) },
  },
});
