// The notification card: `config/quickshell/modules/notifications/CardFace.qml` and
// `NotificationCard.qml`. One notification, the same card in the popups, the peek and the panel.

import type { Words } from "../../lib/content.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { BATTERY, BUILD, card, CRITICAL, KUBA_ONE, type Note, OLA, room } from "./notification-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "open" | "selected";

const SUMMARY: Words = { short: "Done", long: "Build finished for kubaos-ui, kubaos-specimen and every crate under the workspace" };
const BODY: Words = {
  short: "Ok.",
  long: "kubaos-ui and kubaos-specimen built in 41 s; 312 tests passed, none skipped. The specimen's shots changed for the components page, the motion page and the colour page.",
};
const APP: Words = { short: "Qt", long: "Thunderbird Daily Nightly Developer Edition" };

const shown = (note: Note, state: Shown) =>
  room(
    card(note, {
      ...(state === "lit" ? { pinned: "hovered" as const } : {}),
      ...(state === "pressed" ? { pinned: "pressed" as const } : {}),
      selected: state === "selected",
      unfolded: state === "open",
    }),
  );

const UNFOLDS: Move = {
  name: "Unfolds",
  what: "A click, Enter or Space: the text's room opens to its whole height; below it moves with it.",
  tracks: [{ part: "the text's room", on: "Build finished", target: ".k-notif-card", property: "--unfold", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [{ part: "the text's room", on: "Build finished", target: ".k-notif-card", property: "--unfold", from: 0, to: 1, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => room(card(BUILD, { opened: true })),
};

const FOLDS: Move = {
  name: "Folds",
  what: "Esc or another click: back to two lines each, the ellipsis once it has closed.",
  tracks: [{ part: "the text's room", on: "Build finished", target: ".k-notif-card", property: "--unfold", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [{ part: "the text's room", on: "Build finished", target: ".k-notif-card", property: "--unfold", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: () => room(card(BUILD, { unfolded: true })),
};

export default page<Shown>({
  level: "shell",
  slug: "notification-card",
  title: "Notification card",
  lede: "One notification: who sent it and when, its words, its actions; the same card everywhere.",
  code: { qml: "CardFace" },
  stage: {
    live: () => room(card(BUILD, { live: true, focusable: true })),
    hint: "click it, or Tab then Enter; Esc folds",
    daylight: true,
  },
  variants: [
    { name: "Unread, no actions", for: "the dot on its tile; Mark read", sample: () => room(card(OLA)) },
    { name: "With actions and an image", for: "main filled, others as text", sample: () => room(card(BUILD)) },
    { name: "Read, no actions", for: "words alone, no buttons", sample: () => room(card(KUBA_ONE)) },
    { name: "A stack's newest", for: "the count chip; dismiss takes all", sample: () => room(card(OLA, { count: 4, selected: true })) },
    { name: "Critical", for: "`error_container`; main action Danger", sample: () => room(card(CRITICAL)) },
    { name: "A popup", for: "its time left along the bottom", sample: () => room(card(BUILD, { popup: true, remaining: 0.6 })) },
  ],
  sizes: doesntApply("one size: its room's width, its words' height."),
  states: {
    shown: ["rest", "lit", "pressed", "open", "selected"],
    absent: {
      disabled: "a notification is there or dismissed",
      loading: "it arrives whole",
      empty: "no summary and no body: the band alone",
      error: "a critical card is a variant, not a state",
    },
    rows: [
      { name: "With actions", cell: (state) => shown(BUILD, state) },
      { name: "Critical", cell: (state) => shown(CRITICAL, state) },
    ],
    looks: {
      rest: "`surface_container`, the dismiss hidden, its room kept",
      lit: "pointer: `opacity.hover` layer, the dismiss shows",
      pressed: "`opacity.press` layer",
      open: "unfolded to its whole text, chevron up",
      selected: "keyboard: the accent's tube, the dismiss shows",
    },
  },
  focus: {
    tubes: [{ name: "The keyboard on it", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the accent's tube inside its edge, glowing",
      signsOff: "the tube as glass, no glow",
    },
    signsOff: (index) => {
      const face = card(index === 0 ? BUILD : CRITICAL, { selected: true });
      face.dataset.pinned = "signs-off";
      return room(face);
    },
  },
  content: {
    text: [
      { name: "The summary", words: SUMMARY, sample: (words) => room(card({ ...BUILD, summary: words })) },
      { name: "The body", words: BODY, sample: (words) => room(card({ ...BUILD, body: words })) },
      { name: "The app's name", words: APP, sample: (words) => room(card({ ...OLA, app: words }, { count: 12 })) },
    ],
    missingPicture: [
      { name: "No icon, no image", for: "the tile's `notifications` symbol", sample: () => room(card(BATTERY)) },
      { name: "An image", for: "covers the tile", sample: () => room(card(BUILD)) },
      { name: "No app name", for: "says Notification", sample: () => room(card({ ...OLA, app: "" })) },
    ],
    counts: doesntApply("one notification; a stack of them is the group's."),
  },
  motion: {
    arrives: doesntApply("its list brings it: the popups' slide, the panel's open."),
    leaves: doesntApply("its list takes it away."),
    between: [UNFOLDS, FOLDS, ...powerMoves(() => room(card(BUILD, { selected: true })))],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space"], does: "unfolds or folds; on a stack, opens it" },
      { keys: ["Shift+Enter"], does: "runs its main action" },
      { keys: ["Esc"], does: "folds it; folded, goes on to its list" },
    ],
    order: "No Tab stop: its list holds the keyboard and moves it with ↑ ↓.",
    lands: "It never takes the keyboard; its buttons are pointer-only.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With actions", for: "band, words, buttons", sample: () => room(card(BUILD, { selected: true })) },
      { name: "A popup", for: "the time left", sample: () => room(card(OLA, { popup: true, remaining: 0.5 })) },
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
    reflow: doesntApply("its width is its list's: the panel's or the popups'."),
  },
});
