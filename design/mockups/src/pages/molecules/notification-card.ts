// The NotificationCard page: `docs/design/components/notification-card.md`. One notification, the
// same card in the popup stack, the peek and the notifications panel.

import type { Words } from "../../lib/content.ts";
import { type CardOptions, card, type Note } from "../../lib/notification-card.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { BATTERY, BUILD, CRITICAL, KUBA_ONE, OLA } from "../notifications.ts";

type Shown = "rest" | "lit" | "pressed" | "open";

const SUMMARY: Words = { short: "Done", long: "Build finished for kubaos-ui, kubaos-specimen and every crate under the workspace" };
const BODY: Words = {
  short: "Ok.",
  long: "kubaos-ui and kubaos-specimen built in 41 s; 312 tests passed, none skipped. The specimen's shots changed for the components page, the motion page and the colour page.",
};
const APP: Words = { short: "Qt", long: "Thunderbird Daily Nightly Developer Edition" };

/** A card in a room as wide as the popup stack holds one. */
export function inRoom(shown: HTMLElement): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-notif-room";
  box.append(shown);
  return box;
}

const sample = (note: Note, options: CardOptions = {}) => inRoom(card(note, options));

const shown = (note: Note, state: Shown) =>
  sample(note, {
    ...(state === "lit" ? { pinned: "lit" as const } : {}),
    ...(state === "pressed" ? { pinned: "pressed" as const } : {}),
    unfolded: state === "open",
  });

const ON = "Build finished";

const UNFOLDS: Move = {
  name: "Unfolds",
  what: "A click, Enter or Space: the text's room opens to its whole height; below it moves with it.",
  tracks: [{ part: "the text's room", on: ON, target: ".k-notif-card", property: "--unfold", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [
    { part: "the text's room", on: ON, target: ".k-notif-card", property: "--unfold", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the text", on: ON, target: ".k-notif-text", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => sample(BUILD, { opened: true }),
};

const FOLDS: Move = {
  name: "Folds",
  what: "Esc or another click: back to two lines each, cut after the last whole word.",
  tracks: [{ part: "the text's room", on: ON, target: ".k-notif-card", property: "--unfold", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [
    { part: "the text's room", on: ON, target: ".k-notif-card", property: "--unfold", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the text", on: ON, target: ".k-notif-text", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => sample(BUILD, { unfolded: true }),
};

export default page<Shown>({
  level: "molecules",
  slug: "notification-card",
  title: "NotificationCard",
  lede: "One notification: who sent it and when, its words, its actions; the same card everywhere.",
  code: { qml: "NotificationCard", unbuilt: "QML: still the shell's CardFace" },
  stage: {
    live: () => sample(BUILD, { live: true, focusable: true }),
    hint: "click it, or Tab then Enter; Esc folds",
    daylight: true,
  },
  variants: [
    { name: "Unread, no actions", for: "the dot on its tile; Mark read", sample: () => sample(OLA) },
    { name: "With actions and an image", for: "main filled, others as text", sample: () => sample(BUILD) },
    { name: "Read, no actions", for: "words alone; takes no click", sample: () => sample(KUBA_ONE) },
    { name: "A stack's newest", for: "the count chip; dismiss takes all", sample: () => sample(OLA, { count: 4, opens: true, pinned: "lit" }) },
    { name: "Critical", for: "`error_container`; main action Danger", sample: () => sample(CRITICAL) },
    { name: "A popup", for: "time left along the bottom; length from Settings", sample: () => sample(BUILD, { popup: true, remaining: 0.6 }) },
  ],
  sizes: doesntApply("one size: its room's width, its words' height."),
  states: {
    shown: ["rest", "lit", "pressed", "open"],
    absent: {
      disabled: "a notification is there or dismissed",
      selected: "the keyboard on it is lit",
      loading: "it arrives whole",
      empty: "no summary and no body: the band alone",
      error: "a critical card is a variant, not a state",
    },
    rows: [
      { name: "With actions", cell: (state) => shown(BUILD, state) },
      { name: "Critical", cell: (state) => shown(CRITICAL, state) },
    ],
    looks: {
      rest: "`surface_container`; the dismiss hidden, its room kept",
      lit: "pointer or keyboard: the accent's tube, the dismiss shows",
      pressed: "the tube white-hot; by day `opacity.press_tint`",
      open: "unfolded to its whole text, chevron up",
    },
  },
  focus: {
    tubes: [{ name: "Lit", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the accent's tube inside its edge, glowing",
      signsOff: "the tube as glass, no glow",
    },
    signsOff: (index) => sample(index === 0 ? BUILD : CRITICAL, { pinned: "signs-off" }),
  },
  content: {
    text: [
      { name: "The summary", words: SUMMARY, sample: (words) => sample({ ...BUILD, summary: words }) },
      { name: "The body", words: BODY, sample: (words) => sample({ ...BUILD, body: words }) },
      { name: "The app's name", words: APP, sample: (words) => sample({ ...OLA, app: words }, { count: 12 }) },
    ],
    missingPicture: [
      { name: "No icon, no image", for: "the tile's `notifications` symbol", sample: () => sample(BATTERY) },
      { name: "An image", for: "covers the tile", sample: () => sample(BUILD) },
      { name: "No app name", for: "says Notification", sample: () => sample({ ...OLA, app: "" }) },
    ],
    counts: doesntApply("one notification; a stack of them is NotificationGroup's."),
  },
  motion: {
    arrives: doesntApply("what holds it brings it: the popup stack's slide, the panel's open."),
    leaves: doesntApply("what holds it takes it away."),
    between: [UNFOLDS, FOLDS, ...powerMoves(() => sample(BUILD, { pinned: "lit" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space"], does: "unfolds or folds; on a stack, opens it" },
      { keys: ["Shift+Enter"], does: "runs its main action" },
      { keys: ["Esc"], does: "folds it; folded, goes on to what holds it" },
    ],
    order: "No Tab stop in a list: the list holds the keyboard and moves it with ↑ ↓.",
    lands: "It never takes the keyboard; its buttons and its dismiss are pointer-only.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With actions", for: "band, words, buttons", sample: () => sample(BUILD, { pinned: "lit" }) },
      { name: "A popup", for: "the time left", sample: () => sample(OLA, { popup: true, remaining: 0.5 }) },
    ]),
    measures: [
      { part: "Corner", token: "radius-surface" },
      { part: "Band in from top and corner", token: "space-s8" },
      { part: "Band, chevron, dismiss", token: "size-control_sm" },
      { part: "Inset at the sides, foot", token: "space-s12" },
      { part: "Tile, name, count, time apart", token: "space-s8" },
      { part: "Band to words", token: "space-s8" },
      { part: "Summary to body", token: "space-s4" },
      { part: "Words to buttons", token: "space-s12" },
      { part: "Between buttons", token: "space-s8" },
      { part: "Time left's line", token: "stroke-thin" },
    ],
    reflow: doesntApply("its width is what holds it: the panel's or the popup stack's."),
  },
});
