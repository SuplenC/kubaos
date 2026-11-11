// The Recording and countdowns page: `RecordingIndicator.qml`, `IdleIndicator.qml` and the clock's
// countdown in `Clock.qml`. What runs against time in the bar: a recording, Keep awake and a
// scheduled action, and the countdown the clock settings pin.

import type { Words } from "../../lib/content.ts";
import type { Shown } from "../../lib/control.ts";
import { row } from "../../lib/layout.ts";
import { makePill } from "../../lib/pill.ts";
import { ALL } from "../../site/settings.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { CLOCK, clockContent, clockPill, idleContent, idleOptions, idlePill, recordingContent, recordingPill } from "./bar-parts.ts";

type State = "rest" | "lit" | "pressed";

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
  const clock = makePill({ content: clockContent({ countdown: CLOCK.countdown }), lone: false, interactive: true, tooltip: "Tuesday, 27 October 2026" });
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
  return row("s8", "centre", clock.element, recording.element, idle.element);
}

const ROWS: readonly { name: string; make: (state: Shown) => HTMLElement }[] = [
  { name: "Recording", make: (state) => recordingPill({ state }) },
  { name: "Replay buffer", make: (state) => recordingPill({ replay: true, state }) },
  { name: "Keep awake", make: (state) => idlePill({ state }) },
  { name: "Scheduled", make: (state) => idlePill({ keepAwake: false, scheduled: SUSPEND, state }) },
  { name: "Both", make: (state) => idlePill({ scheduled: SUSPEND, state }) },
  { name: "Clock's countdown", make: (state) => clockPill({ countdown: CLOCK.countdown, state }) },
];

const NAME: Words = { short: "Exam", long: "Vacation in the mountains" };

export default page<State>({
  level: "shell",
  slug: "recording",
  title: "Recording and countdowns",
  lede: "What runs against time in the bar: a recording, Keep awake and a scheduled action, the clock's countdown.",
  code: { qml: "RecordingIndicator, IdleIndicator, Clock" },
  stage: { live, hint: "watch 9:59 turn 10:00: the pill eases", daylight: true },
  variants: [
    { name: "Recording", for: "the dot filled, the time elapsed", sample: () => recordingPill() },
    { name: "Replay buffer", for: "the dot hollow, how much is kept", sample: () => recordingPill({ replay: true }) },
    { name: "Keep awake", for: "a cup, `tertiary`, `space.s8` ends", sample: () => idlePill() },
    { name: "A scheduled action", for: "its symbol and the time left, `space.s12` ends", sample: () => idlePill({ keepAwake: false, scheduled: SUSPEND }) },
    { name: "Both", for: "the cup, then the action", sample: () => idlePill({ scheduled: SUSPEND }) },
    { name: "The clock's countdown", for: "after the time, to the minute, `tertiary`", sample: () => clockPill({ countdown: CLOCK.countdown }) },
  ],
  sizes: [
    { name: "Under an hour", for: "m:ss", sample: () => recordingPill({ elapsed: "1:23" }) },
    { name: "Hours", for: "h:mm:ss", sample: () => recordingPill({ elapsed: "1:04:09" }) },
    { name: "Two days or more", for: "whole days", sample: () => idlePill({ keepAwake: false, scheduled: { icon: "schedule", left: "3 d" } }) },
  ],
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "shown only while it runs",
      selected: "no pick",
      loading: "it shows once it runs",
      empty: "nothing running: gone",
      error: "a failed recording's notification says so",
    },
    rows: nonEmpty(ROWS.map(({ name, make }) => ({ name, cell: (state: State) => make(state) }))),
    looks: {
      rest: "recording on `error_container`; the rest on `surface_container`",
      lit: "pointer: tube in its tone, glow at night, hint",
      pressed: "tube white-hot; a click stops or cancels it",
    },
  },
  focus: {
    tubes: [
      { name: "Recording", tone: "error", filled: false, content: false },
      { name: "Keep awake, the clock", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the edge, the glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => ROWS[index]!.make("signs-off"),
  },
  content: {
    text: [{ name: "The countdown's name", words: NAME, sample: (words: string) => clockPill({ countdown: `${words} 52 d 2 h` }) }],
    missingPicture: doesntApply("its symbols are always there."),
    counts: doesntApply("one recording, one scheduled action, one pinned countdown at most."),
  },
  motion: {
    arrives: doesntApply("it shows at once as the recording or action starts."),
    leaves: doesntApply("it goes at once as it stops; no blinking, so the frame never repaints."),
    between: [
      {
        name: "A new digit",
        what: "9:59 to 10:00: the pill eases to its new width over `medium` on `standard_decel`.",
        tracks: [{ part: "width", on: "the recording", target: ".k-pill", property: "--grown", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
        reduced: [{ part: "width", on: "the recording", target: ".k-pill", property: "--grown", from: 0, to: 1, duration: "instant", curve: "standard_decel" }],
        back: "plays",
        sample: () => {
          const made = makePill({ content: recordingContent({ elapsed: "10:00" }), lone: false, tone: "recording" });
          made.element.style.setProperty("--from", "calc(var(--size-control_sm) * 2)");
          made.element.style.setProperty("--to", "calc(var(--size-control_sm) * 2 + var(--space-s8))");
          return made.element;
        },
      },
      { name: "Lit", what: "The pointer onto it: it lights over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => row("s8", "centre", recordingPill(), idlePill()) },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => row("s8", "centre", recordingPill({ state: "lit" }), idlePill({ state: "lit" })) },
      ...powerMoves(() => row("s8", "centre", recordingPill({ state: "lit" }), idlePill({ state: "lit" }))),
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
