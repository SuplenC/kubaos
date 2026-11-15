// Kōbō's welcome: the first thing anyone sees of KubaOS. The street lights up, the neon sign
// strikes, the words rise in, and one button goes on. Live, the stage is the whole installer.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { WINDOWS, windowSize } from "../../site/settings.ts";
import { HERO, LEDE, NETWORKS } from "./kobo-data.ts";
import { ENTERING, LAPTOP, LEAVING, type Moment, OPENING, room } from "./kobo-room.ts";

const at = (moment: Moment) => ({ scene: "welcome" as const, moment });

/** The welcome at a screen's size, `moment` showing. */
const still = (moment: Moment = "rest", more: Partial<Parameters<typeof room>[0]> = {}) => screen(() => room({ at: at(moment), ...more }));

const sized = (name: string) => {
  const size = WINDOWS.find((each) => each.name === name)!;
  return screen(() => room({ at: at("rest"), width: size.width, height: size.height }));
};

type Shown = "rest" | "loading" | "error" | "disabled";

const MOMENT: Record<Shown, Moment> = { rest: "rest", loading: "loading", error: "error", disabled: "noimage" };

export default page<Shown>({
  level: "kobo",
  slug: "welcome",
  title: "Welcome",
  lede: "The first thing anyone sees: the street lights up, the sign strikes, one button goes on.",
  code: { unbuilt: "Kōbō draws the old welcome" },
  from: ["backdrop", "step-rail-page", "button", "keycap", "spinner", "dialog"],
  stage: {
    live: screen(() => room({ at: at("rest"), live: true, width: windowSize().width, height: windowSize().height })),
    hint: "Enter goes on; Esc on Language comes back",
    daylight: doesntApply("the live ISO is always dark: the installer has no day."),
  },
  variants: [
    { name: "At rest", for: "Let's go waiting; the promise under it", sample: still() },
    { name: "Kōbō starts", for: "the opening, played as it is picked", sample: screen(() => room({ at: at("rest"), opening: true, live: true })) },
    { name: "Reading the machine", for: "the probe running; Let's go waits", sample: still("loading") },
    { name: "Couldn't read it", for: "the cause in `error`; Try again", sample: still("error") },
    { name: "No image on the stick", for: "nothing to install, nothing to press", sample: still("noimage") },
    { name: "Low battery", for: "the battery in `warning` at the foot", sample: still("rest", { machine: { ...LAPTOP, battery: { percent: 14, charging: false } } }) },
    { name: "Charging", for: "the charging symbol", sample: still("rest", { machine: { ...LAPTOP, battery: { percent: 64, charging: true } } }) },
    { name: "A desktop", for: "no battery: the foot's end stays empty", sample: still("rest", { machine: { disks: LAPTOP.disks, online: true, networks: NETWORKS } }) },
    { name: "Power", for: "the dialog over the welcome; nothing written yet", sample: still("rest", { power: "ask" }) },
  ],
  sizes: [
    { name: "The smallest screen", for: "the sign gives way; the words stay", sample: sized("Minimum") },
    { name: "A laptop", for: "1280 × 800, the sample's", sample: sized("Laptop") },
    { name: "Full HD", for: "the column stays a column", sample: sized("Full HD") },
  ],
  states: {
    shown: ["rest", "loading", "error", "disabled"],
    absent: {
      lit: "Let's go lights as a `Filled` button does",
      pressed: "Let's go presses as a `Filled` button does",
      selected: "nothing to pick here",
      empty: "it always has its words and one action",
    },
    rows: [{ name: "The welcome", cell: screen((state: Shown) => room({ at: at(MOMENT[state]) })) }],
    looks: {
      rest: "Let's go; the promise in `on_surface_variant`",
      loading: "Let's go disabled; a spinner and what it reads",
      error: "Try again; the cause in `error`",
      disabled: "no image: Let's go disabled, no Enter hint",
    },
  },
  focus: doesntApply("no line of its own: Let's go and Power light as buttons do."),
  content: {
    text: [
      { name: "The hero line", words: HERO, sample: screen((words: string) => room({ at: at("rest"), hero: words })) },
      { name: "The lede", words: LEDE, sample: screen((words: string) => room({ at: at("rest"), lede: words })) },
    ],
    missingPicture: doesntApply("the wallpaper and the sign ship inside Kōbō, on the stick."),
    counts: doesntApply("one screen, one action."),
  },
  motion: {
    arrives: [OPENING],
    leaves: [ENTERING],
    between: [LEAVING],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Let's go, or Try again after a failed read" },
      { keys: ["Tab", "Shift+Tab"], does: "Let's go, then Power, round" },
      { keys: ["Esc"], does: "nothing here; on Language, back here" },
    ],
    order: "Let's go, then Power; nothing else on the screen takes the keyboard.",
    lands: "On Let's go once the opening's words are in; coming back, on Let's go.",
  },
  measures: {
    samples: nonEmpty([{ name: "At rest", for: "the column, its gaps, the foot", sample: still() }]),
    measures: [
      { part: "The column", token: "size-column" },
      { part: "Accent → sign", token: "space-s8" },
      { part: "Sign → hero line", token: "space-s20" },
      { part: "Hero → lede; between actions", token: "space-s16" },
      { part: "Lede → Let's go", token: "space-s40" },
      { part: "Band past the words", token: "space-s64" },
      { part: "Foot from the edge", token: "space-s32" },
      { part: "Foot's inset, as the rail's", token: "space-s12" },
      { part: "Foot above the edge", token: "space-s16" },
      { part: "A part's rise", token: "space-s24" },
    ],
    reflow: { note: "The column stays in the middle; on a short screen the sign gives way first.", sample: screen(() => room({ at: at("rest"), width: windowSize().width, height: windowSize().height })) },
  },
});
