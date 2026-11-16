// The Task page: `docs/design/components/task.md`. One thing to try, ticked once it is done: a
// round tick and its words on a card, lit as a picked card is when done.

import type { Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { task, type TaskOptions } from "../../lib/task.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const ROOM = "var(--size-panel)";

const OPEN: TaskOptions = { before: "Open it", line: "Super + Space" };
const WIFI: TaskOptions = { before: "Try", typed: "wifi", after: ", then Enter", line: "A setting, no Settings app" };
const SUM: TaskOptions = { before: "Try", typed: "12*7", line: "Sums answer as you type" };

const LINE: Words = { short: "Sums", long: "Sums answer as you type, units and currencies too, and Enter copies the answer" };

function one(options: TaskOptions): HTMLElement {
  return within(ROOM, task(options).element);
}

function three(done: readonly boolean[]): HTMLElement {
  return within(ROOM, stack("s12", "start", ...[OPEN, WIFI, SUM].map((each, at) => task({ ...each, done: done[at] ?? false }).element)));
}

/** A live one: a press ticks it. */
function live(): HTMLElement {
  const made = task({ ...WIFI, onPress: () => made.tick() });
  return within(ROOM, made.element);
}

const TICKS: Move = {
  name: "Ticked",
  what: "The card lights over `medium`; its tick fills and pops in over `medium` on `pop`.",
  tracks: [{ part: "the tick", on: "a task done", target: ".k-task-tick", property: "scale", from: 0.4, to: 1, duration: "medium", curve: "pop" }],
  reduced: [{ part: "the tick", on: "a task done", target: ".k-task-tick", property: "scale", from: 1, to: 1, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => one({ ...WIFI, done: true }),
};

export default page<"rest" | "selected">({
  level: "molecules",
  slug: "task",
  title: "Task",
  lede: "One thing to try, ticked once it is done.",
  code: { unbuilt: "neither library has it yet" },
  stage: { live, hint: "press it to tick it", daylight: true },
  variants: [
    { name: "Words alone", for: "something to do", sample: () => one(OPEN) },
    { name: "Something to type", for: "the words in mono", sample: () => one(WIFI) },
    { name: "A list of three", for: "one done", sample: () => three([true, false, false]) },
  ],
  sizes: doesntApply("one size: its tick is `size.check`, its words `body_strong` over `small`."),
  states: {
    shown: ["rest", "selected"],
    absent: {
      lit: "a press goes to what it asks for; it lights only once done",
      pressed: "a press starts the thing to try; nothing stays pressed",
      disabled: "nothing to try is ever out of reach",
      loading: "it knows what it asks for when drawn",
      empty: "a task has words",
      error: "a try can't fail: it is done, or not yet",
    },
    rows: [{ name: "Something to type", cell: (state) => one({ ...WIFI, done: state === "selected" }) }],
    looks: {
      rest: "to do: an empty ring",
      selected: "done: lit, the tick filled",
    },
  },
  focus: doesntApply("a pointer's shortcut to what the page's keys already do; the keys stay on the page's own controls."),
  content: {
    text: [{ name: "Its line", words: LINE, sample: (words: string) => one({ ...SUM, line: words }) }],
    missingPicture: doesntApply("its tick is drawn, its words type."),
    counts: [{ name: "Three", for: "first boot's Try it", sample: () => three([true, true, false]) }],
  },
  motion: {
    arrives: doesntApply("it shows with its page."),
    leaves: doesntApply("it goes with its page."),
    between: [TICKS],
  },
  keyboard: doesntApply("it takes no focus: the page's own keys do what it asks."),
  measures: {
    samples: nonEmpty([{ name: "Something to type", for: "tick, words, padding", sample: () => one(WIFI) }]),
    measures: [
      { part: "Height, at least", token: "size-row_two_line" },
      { part: "Padding", token: "space-s16" },
      { part: "Tick", token: "size-check" },
      { part: "Tick → words", token: "space-s12" },
      { part: "Corner", token: "radius-content" },
    ],
    reflow: { note: "It takes the width it is given; a long line wraps under its title.", sample: () => within("100%", task(SUM).element) },
  },
});
