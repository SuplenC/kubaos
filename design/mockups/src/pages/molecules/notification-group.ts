// The NotificationGroup page: `docs/design/components/notification-group.md`. One app's
// notifications as one thing: a stack at rest, a preview under a resting pointer, opened in place.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import type { Note } from "../../lib/notification-card.ts";
import { group, type GroupOptions } from "../../lib/notification-group.ts";
import { BUILD, KUBAOS, SIGNAL, SLACK } from "../notifications.ts";
import { inRoom } from "./notification-card.ts";

type Shown = "rest" | "lit" | "open";

/** A group in a room with space under it for its preview, which floats past its end. */
function held(notes: readonly Note[], options: GroupOptions = {}): HTMLElement {
  const box = inRoom(group(notes, options));
  box.dataset.tall = "";
  box.style.setProperty("--notif-overhang", (options.fan ?? 0) > 0 ? "calc(var(--size-control_lg) * 9)" : "0px");
  return box;
}

const SPREADS: Move = {
  name: "The preview spreads",
  what: "The pointer rests `medium` on a stack: up to three cards spread into whole cards over the list.",
  tracks: [{ part: "the spread", on: "Slack, four", target: ".k-notif-group", property: "--fan", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [
    { part: "the spread", on: "Slack, four", target: ".k-notif-group", property: "--fan", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the cards", on: "Slack, four", target: ".k-notif-slot", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => held(SLACK, { fan: 0.0001 }),
};

const TUCKS: Move = {
  name: "The preview tucks in",
  what: "The pointer leaves: the cards go back under the newest.",
  tracks: [{ part: "the spread", on: "Slack, four", target: ".k-notif-group", property: "--fan", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [
    { part: "the spread", on: "Slack, four", target: ".k-notif-group", property: "--fan", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the cards", on: "Slack, four", target: ".k-notif-slot", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => held(SLACK, { fan: 1 }),
};

const OPENS: Move = {
  name: "Opens",
  what: "A click, Enter or →: every card takes its place under the heading; only what is below moves.",
  tracks: [{ part: "the opening", on: "Slack, four", target: ".k-notif-group", property: "--opening", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [
    { part: "the opening", on: "Slack, four", target: ".k-notif-group", property: "--opening", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the cards", on: "Slack, four", target: ".k-notif-slot", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => {
    const box = held(SLACK);
    box.querySelector(".k-notif-heading")?.setAttribute("data-shown", "");
    return box;
  },
};

const CLOSES: Move = {
  name: "Closes",
  what: "Show less, ← or Esc: the cards tuck back into the stack.",
  tracks: [{ part: "the opening", on: "Slack, four", target: ".k-notif-group", property: "--opening", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [
    { part: "the opening", on: "Slack, four", target: ".k-notif-group", property: "--opening", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the cards", on: "Slack, four", target: ".k-notif-slot", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => held(SLACK, { open: true }),
};

const IN_STATE = (notes: readonly Note[], state: Shown) => held(notes, { fan: state === "lit" ? 1 : 0, open: state === "open" });

export default page<Shown>({
  level: "molecules",
  slug: "notification-group",
  title: "NotificationGroup",
  lede: "One app's notifications as one thing: a stack, a preview under the pointer, opened in place.",
  code: { qml: "NotificationGroup", unbuilt: "QML: still the shell's CardGroup" },
  stage: {
    live: () => held(SLACK, { live: true, fan: 0 }),
    hint: "rest the pointer on it; click opens",
    daylight: true,
  },
  variants: [
    { name: "One", for: "the card alone, no stack", sample: () => held([BUILD]) },
    { name: "A pair", for: "one narrower edge peeks out", sample: () => held(SIGNAL) },
    { name: "Three or more", for: "two edges; the count chip", sample: () => held(SLACK) },
  ],
  sizes: doesntApply("its width is its list's; its height its cards'."),
  states: {
    shown: ["rest", "lit", "open"],
    absent: {
      pressed: "the press is its newest card's",
      disabled: "it is there or dismissed",
      loading: "it arrives whole",
      selected: "the keyboard lights its newest card",
      empty: "no notifications, no group",
      error: "a critical card is the card's variant",
    },
    rows: [
      { name: "Four", cell: (state) => IN_STATE(SLACK, state) },
      { name: "Two", cell: (state) => IN_STATE(KUBAOS, state) },
    ],
    looks: {
      rest: "newest card, edges `space.s8` down and in",
      lit: "pointer rested: three spread, step 2",
      open: "heading, count and Show less; every card",
    },
  },
  focus: doesntApply("its cards light their own tube: the card's page."),
  content: {
    text: doesntApply("its words are its cards'; the heading's app name is the card's."),
    missingPicture: doesntApply("its pictures are its cards' tiles."),
    counts: [
      { name: "One", for: "a card, nothing tucked", sample: () => held([BUILD]) },
      { name: "Two", for: "one edge", sample: () => held(SIGNAL) },
      { name: "Four, previewed", for: "three spread, +1 more · click to open", sample: () => held(SLACK, { fan: 1 }) },
    ],
  },
  motion: {
    arrives: doesntApply("its list brings it: the popups' slide, the panel's open."),
    leaves: doesntApply("its list takes it away."),
    between: [SPREADS, TUCKS, OPENS, CLOSES, ...powerMoves(() => held(SLACK, { selected: true }))],
  },
  keyboard: {
    keys: [
      { keys: ["→", "l"], does: "opens it, the keyboard on its first card" },
      { keys: ["←", "h"], does: "closes it" },
      { keys: ["↑", "↓", "j", "k"], does: "card to card inside it opened; past its ends, the list" },
      { keys: ["Enter", "Space"], does: "opens a stack; unfolds an opened card" },
      { keys: ["Del"], does: "dismisses the card, or the whole stack" },
      { keys: ["Esc"], does: "folds a card, then closes it" },
    ],
    order: "No Tab stop: the panel's list holds the keyboard.",
    lands: "Opened, on its first card; ↑ into an opened group, on its last.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A stack", for: "the edges' peek and inset", sample: () => held(SLACK) },
      { name: "Opened", for: "heading, gaps", sample: () => held(SLACK, { open: true }) },
    ]),
    measures: [
      { part: "Edge's peek below", token: "space-s8" },
      { part: "Edge narrower each side", token: "space-s8" },
      { part: "Between spread cards", token: "space-s8" },
      { part: "Heading", token: "size-control_sm" },
      { part: "Corner", token: "radius-surface" },
      { part: "Cast on an edge, step 1", token: "elevation-1-ambient-blur" },
      { part: "Spread, step 2", token: "elevation-2-ambient-blur" },
      { part: "Rows behind the preview", token: "opacity-recede" },
    ],
    reflow: doesntApply("its width is its list's: the panel's or the popups'."),
  },
});
