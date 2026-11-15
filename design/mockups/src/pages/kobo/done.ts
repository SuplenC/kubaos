// Kōbō's done screen: the arrival. The street goes dark while クバ writes itself in as unlit glass,
// then the sign strikes and the street lights up round it and comes toward you: it worked, and the
// machine is on and yours. The headline rises word by word under it; Reboot switches it all off.

import { doesntApply, nonEmpty, page, screen } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { DONE_HEADLINE } from "./kobo-data.ts";
import { ARRIVING, FINISHING, GOING, room } from "./kobo-room.ts";

type Shown = "rest" | "loading";

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: screen(() => room({ at: { scene: "done" } })),
  loading: screen(() => room({ at: { scene: "done", going: true } })),
};

export default page<Shown>({
  level: "kobo",
  slug: "done",
  title: "Done",
  lede: "The arrival: the street goes dark, クバ writes itself in and strikes, the street lights up.",
  code: { unbuilt: "Kōbō draws the old done screen" },
  from: ["backdrop", "button", "keycap", "spinner", "dialog"],
  stage: {
    live: screen(() => room({ at: { scene: "done" }, live: true, width: windowSize().width, height: windowSize().height })),
    hint: "Enter reboots: the lights go out",
    daylight: doesntApply("the live ISO is always dark: the installer has no day."),
  },
  variants: [
    { name: "Done", for: "the sign lit, the street close, Reboot waiting", sample: CELLS.rest },
    { name: "Restarting", for: "Reboot taken: a spinner and the room going dark", sample: CELLS.loading },
    { name: "Power", for: "pull the stick first, it says", sample: screen(() => room({ at: { scene: "done" }, power: "ask" })) },
  ],
  sizes: [{ name: "Full HD", for: "the column in the middle", sample: screen(() => room({ at: { scene: "done" }, width: "calc(1920 * var(--px))", height: "calc(1080 * var(--px))" })) }],
  states: {
    shown: ["rest", "loading"],
    absent: { lit: "Reboot lights as a `Filled` button", pressed: "Reboot presses as a `Filled` button", disabled: "Reboot works from the first frame", selected: "nothing to pick", empty: "always its words and two actions", error: "a failed install never reaches it" },
    rows: [{ name: "Done", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: { rest: "Reboot and Stay; Enter reboot", loading: "a spinner and Restarting…" },
  },
  focus: doesntApply("no line of its own: Reboot and Stay light as buttons do."),
  content: {
    text: [{ name: "The headline", words: DONE_HEADLINE, sample: screen((words: string) => room({ at: { scene: "done" }, headline: words })) }],
    missingPicture: doesntApply("the mark is drawn from the logo's strokes; nothing loads."),
    counts: doesntApply("one screen, two actions."),
  },
  motion: {
    arrives: [ARRIVING, FINISHING],
    leaves: [GOING],
    between: doesntApply("it rests once it has arrived: nothing hums."),
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "Reboot" },
      { keys: ["Tab", "Shift+Tab"], does: "Reboot, Stay in the live system, Power" },
    ],
    order: "Reboot, Stay in the live system, then Power.",
    lands: "On Reboot from the first frame, while the mark still writes itself in.",
  },
  measures: {
    samples: nonEmpty([{ name: "Done", for: "the sign and the column", sample: CELLS.rest }]),
    measures: [
      { part: "The mark", token: "size-neon_mark" },
      { part: "Mark → headline", token: "space-s24" },
      { part: "Headline → lede", token: "space-s16" },
      { part: "Lede → actions", token: "space-s40" },
      { part: "Between actions", token: "space-s8" },
    ],
    reflow: { note: "The column stays in the middle; on a short screen the mark keeps its size.", sample: screen(() => room({ at: { scene: "done" }, width: windowSize().width, height: windowSize().height })) },
  },
});
