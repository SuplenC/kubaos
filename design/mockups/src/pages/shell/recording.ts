// The Recording and Keep awake page: `RecordingIndicator.qml` and `IdleIndicator.qml`. What runs
// against time in the bar: a recording, Keep awake and a scheduled action.

import type { Shown } from "../../lib/control.ts";
import { row } from "../../lib/layout.ts";
import { makePill, pillChanging } from "../../lib/pill.ts";
import { ALL } from "../../site/settings.ts";
import { hovers, press, pillSwap } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { idleContent, idleOptions, idlePill, recordingContent, recordingPill } from "./bar-parts.ts";

type State = "rest" | "hovered" | "pressed";

/** Seconds as the recorder and Keep awake say them: m:ss, h:mm:ss, or whole days past two. */
function face(seconds: number): string {
  if (seconds >= 172800) return `${Math.round(seconds / 86400)} d`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor(seconds / 60) % 60;
  const s = seconds % 60;
  const mmss = `${String(m).padStart(h ? 2 : 1, "0")}:${String(s).padStart(2, "0")}`;
  return h ? `${h}:${mmss}` : mmss;
}

const SUSPEND = { icon: "bedtime", left: "12:08" };

/** A recording that ticks, and a scheduled action counting down: each pill eases as its words widen. */
function live(): HTMLElement {
  let elapsed = 9 * 60 + 55;
  let left = 10 * 60 + 4;
  const recording = makePill({ content: recordingContent({ elapsed: face(elapsed) }), lone: false, tone: "recording", interactive: true, tooltip: "Recording. Click to stop" });
  const idle = makePill({ content: idleContent({ keepAwake: false, scheduled: { icon: "bedtime", left: face(left) } }), lone: false, ...idleOptions({ scheduled: SUSPEND }), interactive: true, tooltip: "Suspend at 23:00. Click to cancel it" });
  if (!ALL) {
    const tick = window.setInterval(() => {
      if (!recording.element.isConnected) {
        window.clearInterval(tick);
        return;
      }
      elapsed += 1;
      left = Math.max(0, left - 1);
      const before = recording.element.offsetWidth;
      const content = recordingContent({ elapsed: face(elapsed) });
      // Mono figures are one width: only a new digit changes the pill's.
      const line = recording.element.querySelector(".k-row")!;
      line.replaceChildren(...content);
      if (recording.element.offsetWidth !== before) recording.change(recordingContent({ elapsed: face(elapsed) }));
      idle.element.querySelector(".k-row")!.replaceChildren(...idleContent({ keepAwake: false, scheduled: { icon: "bedtime", left: face(left) } }));
    }, 1000);
  }
  return row("s8", "centre", recording.element, idle.element);
}

const ROWS: readonly { name: string; make: (state: Shown) => HTMLElement }[] = [
  { name: "Recording", make: (state) => recordingPill({ state }) },
  { name: "Replay buffer", make: (state) => recordingPill({ replay: true, state }) },
  { name: "Keep awake", make: (state) => idlePill({ state }) },
  { name: "Scheduled", make: (state) => idlePill({ keepAwake: false, scheduled: SUSPEND, state }) },
  { name: "Both", make: (state) => idlePill({ scheduled: SUSPEND, state }) },
];

export default page<State>({
  level: "shell",
  slug: "recording",
  title: "Recording and Keep awake",
  lede: "What runs against time in the bar: a recording, Keep awake and a scheduled action.",
  code: { qml: "RecordingIndicator, IdleIndicator, Clock" },
  stage: { live, hint: "watch 9:59 turn 10:00: the pill eases", daylight: true },
  variants: [
    { name: "Recording", for: "the dot filled, the time elapsed", sample: () => recordingPill() },
    { name: "Replay buffer", for: "the dot hollow, how much is kept", sample: () => recordingPill({ replay: true }) },
    { name: "Keep awake", for: "a cup, `tertiary`, `space.s8` ends", sample: () => idlePill() },
    { name: "A scheduled action", for: "its symbol and the time left, `space.s12` ends", sample: () => idlePill({ keepAwake: false, scheduled: SUSPEND }) },
    { name: "Both", for: "the cup, then the action", sample: () => idlePill({ scheduled: SUSPEND }) },
  ],
  sizes: [
    { name: "Under an hour", for: "m:ss", sample: () => recordingPill({ elapsed: "1:23" }) },
    { name: "Hours", for: "h:mm:ss", sample: () => recordingPill({ elapsed: "1:04:09" }) },
    { name: "Two days or more", for: "whole days", sample: () => idlePill({ keepAwake: false, scheduled: { icon: "schedule", left: "3 d" } }) },
  ],
  states: {
    shown: ["rest", "hovered", "pressed"],
    absent: {
      lit: "no keyboard focus: the bar is never a Tab stop",
      disabled: "shown only while it runs",
      selected: "no pick",
      loading: "it shows once it runs",
      empty: "nothing running: gone",
      error: "a failed recording's notification says so",
    },
    rows: nonEmpty(ROWS.map(({ name, make }) => ({ name, cell: (state: State) => make(state) }))),
    looks: {
      rest: "recording on `error_container`; the rest on `surface_container`",
      hovered: "the state layer over its tone's fill, its hint",
      pressed: "fill a step past hover; a click stops or cancels it",
    },
  },
  focus: doesntApply("the bar takes no keyboard focus, so a pill never lights its tube."),
  content: {
    text: doesntApply("its words are times and fixed names."),
    missingPicture: doesntApply("its symbols are always there."),
    counts: doesntApply("one recording and one scheduled action at most."),
  },
  motion: {
    arrives: doesntApply("it shows at once as the recording or action starts."),
    leaves: doesntApply("it goes at once as it stops; no blinking, so the frame never repaints."),
    between: [
      {
        name: "A new digit",
        what: "9:59 to 10:00: the old time fades as its room closes, the new as its opens, over `medium`.",
        ...pillSwap("the recording"),
        back: "plays",
        sample: () => pillChanging({ lone: false, tone: "recording" }, recordingContent({ elapsed: "9:59" }), recordingContent({ elapsed: "10:00" })),
      },
      { name: "Hovered", what: "The pointer onto it: its fill a step on over `short`.", tracks: hovers(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => row("s8", "centre", recordingPill(), idlePill()) },
      { name: "Pressed", what: "Over `micro` its fill goes a step past hovered, with no glow.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => row("s8", "centre", recordingPill({ state: "hovered" }), idlePill({ state: "hovered" })) },
    ],
  },
  keyboard: {
    keys: [{ keys: ["SUPER+SHIFT+P"], does: "the replay buffer: saves the last 30 s" }],
    order: "Never a stop for Tab.",
    lands: "Nothing takes the keyboard.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Recording", for: "dot, words, ends", sample: () => recordingPill() },
      { name: "Both", for: "symbols, gaps, ends", sample: () => idlePill({ scheduled: SUSPEND }) },
    ]),
    measures: [
      { part: "Height", token: "size-control_sm" },
      { part: "Recording's ends, scheduled's ends", token: "space-s12" },
      { part: "Keep awake's ends", token: "space-s8" },
      { part: "Symbol to value; Keep awake's gaps", token: "space-s4" },
      { part: "Dot", token: "size-radio_dot" },
      { part: "Hollow dot's ring", token: "stroke-thin" },
      { part: "Time's line", token: "type-mono-line" },
    ],
    reflow: doesntApply("as wide as its words at every window size."),
  },
});
