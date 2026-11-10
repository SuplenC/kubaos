// The notification popups: `config/quickshell/modules/notifications/Popups.qml`, `ToastList.qml`,
// `PopupHold.qml` and `PopupsBelow.qml`. New notifications in the frame's top right corner, one
// app's in a group, each leaving when its time is up unless the pointer holds the stack.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { BATTERY, BUILD, cornerOnScreen, CRITICAL, group, type GroupOptions, type Note, SLACK, stackOf, UPDATE } from "./notification-parts.ts";

type Shown = "rest" | "lit" | "open";

/** The popups' timeout: the shell's setting, `Settings.notificationTimeout`, not a design token. */
const TIMEOUT = 6000;

/** A stack of popups on the screen's corner, each group a popup. */
function popups(groups: readonly (readonly Note[])[], options: GroupOptions & { remaining?: number; unfolded?: boolean } = {}): HTMLElement {
  const made = groups.map((notes, index) => {
    const each = group(notes, { ...options, popup: true, fan: index === 0 ? (options.fan ?? 0) : 0 });
    if (options.remaining !== undefined) each.style.setProperty("--remaining", String(options.remaining));
    if (options.unfolded && index === 0) requestAnimationFrame(() => each.cards()[0]?.press());
    return each;
  });
  return cornerOnScreen(stackOf(...made));
}

/**
 * The corner live: the popups run their time down and leave, held while the pointer is on the stack,
 * and new ones slide in once it is empty.
 */
function live(): HTMLElement {
  const stack = stackOf();
  const screen = cornerOnScreen(stack);
  const runs: Animation[] = [];
  let held = false;
  const batches: (readonly Note[])[][] = [[[BUILD], [BATTERY]], [SLACK, [UPDATE]]];
  let next = 0;

  function arrive(): void {
    const batch = batches[next++ % batches.length]!;
    for (const notes of batch) {
      const each = group(notes, { popup: true, live: true, onDismiss: () => leave(each) });
      each.classList.add("k-notif-arriving");
      stack.append(each);
      const run = each.animate([{ "--remaining": "1" }, { "--remaining": "0" }], { duration: TIMEOUT, fill: "forwards" });
      runs.push(run);
      if (held) run.pause();
      run.finished.then(() => leave(each), () => {});
    }
  }

  function leave(each: HTMLElement): void {
    if (each.classList.contains("k-notif-leaving")) return;
    each.classList.remove("k-notif-arriving");
    each.classList.add("k-notif-leaving");
    each.addEventListener("animationend", () => {
      each.remove();
      if (stack.children.length === 0) window.setTimeout(arrive, TIMEOUT / 4);
    }, { once: true });
  }

  stack.addEventListener("pointerenter", () => {
    held = true;
    for (const run of runs) if (run.playState === "running") run.pause();
  });
  stack.addEventListener("pointerleave", () => {
    held = false;
    for (const run of runs) if (run.playState === "paused") run.play();
  });
  if (!ALL) requestAnimationFrame(arrive);
  else arrive();
  return screen;
}

const SLIDES_IN: Move = {
  name: "A popup arrives",
  what: "It slides in from the right as it fades in; the others close up under it.",
  tracks: [
    { part: "the slide", on: "Build finished", target: ".k-notif-stack > *", property: "--slide-by", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "the fade", on: "Build finished", target: ".k-notif-stack > *", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [{ part: "the fade", on: "Build finished", target: ".k-notif-stack > *", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  back: "cuts",
  sample: () => popups([[BUILD]]),
};

const SLIDES_OUT: Move = {
  name: "A popup leaves",
  what: "Its time is up, it is dismissed or acted on: out two `space.s64`s to the right, fading.",
  tracks: [
    { part: "the slide", on: "Build finished", target: ".k-notif-stack > *", property: "--slide-by", from: 0, to: 2, duration: "medium", curve: "emphasized_accel" },
    { part: "the fade", on: "Build finished", target: ".k-notif-stack > *", property: "opacity", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
  ],
  reduced: [{ part: "the fade", on: "Build finished", target: ".k-notif-stack > *", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  back: "cuts",
  sample: () => popups([[BUILD]]),
};

const IN_STATE = (state: Shown) => popups([[BUILD], [BATTERY]], { remaining: state === "lit" ? 0.45 : 0.8, unfolded: state === "open" });

export default page<Shown>({
  level: "shell",
  slug: "notification-popups",
  title: "Notification popups",
  lede: "New notifications in the frame's top right corner, each gone when its time is up.",
  code: { qml: "Popups" },
  stage: {
    live,
    hint: "point at the stack to hold it",
    daylight: true,
  },
  variants: [
    { name: "One", for: "a card with its time left", sample: () => popups([[BUILD]], { remaining: 0.7 }) },
    { name: "Several apps", for: "a group each, at most five", sample: () => popups([[BUILD], [BATTERY]], { remaining: 0.7 }) },
    { name: "A burst from one app", for: "one stack, its count", sample: () => popups([SLACK, [BUILD]], { remaining: 0.7 }) },
    { name: "Previewed", for: "the preview hangs past the corner", sample: () => popups([SLACK, [BUILD]], { fan: 1 }) },
    { name: "Critical", for: "no time left: it stays until acted on", sample: () => popups([[CRITICAL], [BUILD]], { remaining: 0.7 }) },
    { name: "Critical under a panel", for: "a stack under the open panel", sample: () => popups([[CRITICAL]]) },
  ],
  sizes: doesntApply("`size.panel` wide; as tall as its cards."),
  states: {
    shown: ["rest", "lit", "open"],
    absent: {
      pressed: "the press is its cards'",
      disabled: "shown or gone",
      selected: "it never takes the keyboard",
      loading: "a popup arrives whole",
      empty: "no popups: the corner closes, or the peek shows",
      error: "a critical popup is a variant",
    },
    rows: [{ name: "Two apps", cell: (state) => IN_STATE(state) }],
    looks: {
      rest: "each line running down",
      lit: "held: every line stopped, a new one waits",
      open: "a card unfolded; the corner grows",
    },
  },
  focus: doesntApply("it never takes the keyboard."),
  content: {
    text: doesntApply("its words are its cards'."),
    missingPicture: doesntApply("its pictures are its cards' tiles."),
    counts: [
      { name: "None", for: "the corner closes", sample: () => cornerOnScreen(stackOf()) },
      { name: "One", for: "one card", sample: () => popups([[BUILD]]) },
      { name: "Five apps", for: "the most it shows", sample: () => popups([[BUILD], [BATTERY], SLACK, [CRITICAL], [UPDATE]], { remaining: 0.7 }) },
    ],
  },
  motion: {
    arrives: [SLIDES_IN],
    leaves: [SLIDES_OUT],
    between: powerMoves(() => popups([[BUILD]], { remaining: 0.7 })),
  },
  keyboard: doesntApply("it never takes the keyboard: the pointer and its buttons do; right-click opens a card's menu."),
  measures: {
    samples: nonEmpty([{ name: "Two apps", for: "padding, gap, width", sample: () => popups([[BUILD], [BATTERY]], { remaining: 0.6 }) }]),
    measures: [
      { part: "Width", token: "size-panel" },
      { part: "Inside, between popups", token: "space-s8" },
      { part: "Slide in, one step", token: "space-s64" },
      { part: "Time left's line", token: "stroke-thin" },
      { part: "Corner", token: "radius-surface" },
    ],
    reflow: doesntApply("it keeps its corner and `size.panel` on every screen."),
  },
});
