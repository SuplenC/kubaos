// The NotificationPopup page: `docs/design/components/notification-card.md`. A new notification in
// the popup stack, two lines and quiet, opening in place into its NotificationCard.

import type { Words } from "../../lib/content.ts";
import { notificationPopup, type PopupNote, type PopupOptions } from "../../lib/notification-popup.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { BATTERY, BUILD, MARCO } from "../notifications.ts";
import { inRoom } from "./notification-card.ts";

type Shown = "rest" | "lit" | "selected" | "open";

const SUMMARY: Words = { short: "Done", long: "Build finished for kubaos-ui, kubaos-specimen and every crate under the workspace" };
const BODY: Words = { short: "Ok.", long: "kubaos-ui and kubaos-specimen built in 41 s; 312 tests passed, none skipped. The specimen's shots changed." };

const sample = (note: PopupNote, options: PopupOptions = {}) => inRoom(notificationPopup(note, options));

const SLACK3: PopupNote = { ...MARCO, count: 3 };

const shown = (note: PopupNote, state: Shown) =>
  sample(note, {
    ...(state === "lit" ? { pinned: "pointed" as const } : {}),
    ...(state === "selected" ? { pinned: "lit" as const } : {}),
    open: state === "open",
    remaining: 0.6,
  });

const ON = "Build finished";

const OPENS: Move = {
  name: "Opens",
  what: "A click or Enter: it grows in place into its card, the two lines giving way to it.",
  tracks: [{ part: "the opening", on: ON, target: ".k-notif-pop", property: "--open", from: 0, to: 1, duration: "medium", curve: "emphasized_decel" }],
  reduced: [
    { part: "the opening", on: ON, target: ".k-notif-pop", property: "--open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the card", on: ON, target: ".k-notif-pop-full", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => sample(BUILD, { remaining: 0.6 }),
};

const FOLDS: Move = {
  name: "Folds",
  what: "A second click on its words, Esc or a click outside: back to two lines.",
  tracks: [{ part: "the opening", on: ON, target: ".k-notif-pop", property: "--open", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [
    { part: "the opening", on: ON, target: ".k-notif-pop", property: "--open", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the two lines", on: ON, target: ".k-notif-pop-face-room", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => sample(BUILD, { open: true, remaining: 0.6 }),
};

const DISMISS: Move = {
  name: "The dismiss shows",
  what: "The pointer on it: the × takes the app and the time's place; it stays two lines.",
  tracks: [
    { part: "the app and time", on: ON, target: ".k-notif-pop-meta", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the dismiss", on: ON, target: ".k-notif-pop-dismiss", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: () => sample(BUILD, { remaining: 0.6 }),
};

export default page<Shown>({
  level: "molecules",
  slug: "notification-popup",
  title: "NotificationPopup",
  lede: "A new notification, two lines and quiet; a click opens it in place into its card.",
  code: { qml: "NotificationPopup", unbuilt: "QML: not built yet" },
  stage: {
    live: () => sample(BUILD, { live: true, remaining: 0.6 }),
    hint: "point at it; click or Enter opens, Esc folds",
    daylight: true,
  },
  variants: [
    { name: "One", for: "the app's symbol, title, one body line", sample: () => sample(BUILD, { remaining: 0.6 }) },
    { name: "Several from one app", for: "the count in the symbol's place", sample: () => sample(SLACK3, { remaining: 0.6 }) },
    { name: "Opened", for: "its card, with the count chip", sample: () => sample(SLACK3, { open: true, remaining: 0.6 }) },
  ],
  sizes: doesntApply("one size: the stack's width, two lines tall."),
  states: {
    shown: ["rest", "lit", "selected", "open"],
    absent: {
      pressed: "a press opens it",
      disabled: "it is there or gone",
      loading: "it arrives whole",
      empty: "no notification, no popup",
      error: "a critical one is a full card in the stack",
    },
    rows: [
      { name: "One", cell: (state) => shown(BUILD, state) },
      { name: "Three from Slack", cell: (state) => shown(SLACK3, state) },
    ],
    looks: {
      rest: "two lines; the app and the time at its end",
      lit: "pointer: only the × appears, in their place",
      selected: "keyboard: the accent's tube, and the ×",
      open: "today's card in place; its time held",
    },
  },
  focus: {
    tubes: [{ name: "The keyboard on it", tone: "accent", filled: false, content: false }],
    looks: { lit: "the accent's tube inside its edge, glowing", signsOff: "the tube as glass, no glow" },
    signsOff: (index) => {
      const shownNow = sample(index === 0 ? BUILD : SLACK3, { pinned: "lit", remaining: 0.6 });
      shownNow.querySelector<HTMLElement>(".k-notif-pop")!.dataset.pinned = "signs-off";
      return shownNow;
    },
  },
  content: {
    text: [
      { name: "The title", words: SUMMARY, sample: (words) => sample({ ...BUILD, summary: words }) },
      { name: "The body", words: BODY, sample: (words) => sample({ ...BUILD, body: words }) },
    ],
    missingPicture: [
      { name: "An image", for: "not on the popup: the app's symbol", sample: () => sample(BUILD) },
      { name: "No app name", for: "says Notification", sample: () => sample({ ...BATTERY, app: "" }) },
    ],
    counts: [
      { name: "One", for: "the symbol", sample: () => sample(BUILD) },
      { name: "Three", for: "3 in the symbol's place", sample: () => sample(SLACK3) },
      { name: "Past 99", for: "99+", sample: () => sample({ ...MARCO, count: 140 }) },
    ],
  },
  motion: {
    arrives: doesntApply("the popup stack brings it: from the right."),
    leaves: doesntApply("the popup stack takes it: right, or up when its time is up."),
    between: [DISMISS, OPENS, FOLDS, ...powerMoves(() => sample(BUILD, { pinned: "lit", remaining: 0.6 }))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab"], does: "reaches each popup, newest first" },
      { keys: ["Enter"], does: "opens it into its card, or folds it" },
      { keys: ["Delete"], does: "dismisses it" },
      { keys: ["Esc"], does: "folds it" },
    ],
    order: "Each popup is a stop, newest first; its card's buttons take the pointer.",
    lands: "It never takes the keyboard by itself arriving.",
  },
  measures: {
    samples: nonEmpty([
      { name: "One", for: "tile, lines, end, padding", sample: () => sample(BUILD, { pinned: "pointed", remaining: 0.6 }) },
      { name: "Three from Slack", for: "the count in the tile", sample: () => sample(SLACK3, { remaining: 0.6 }) },
    ]),
    measures: [
      { part: "Corner", token: "radius-surface" },
      { part: "In from top, foot, far corner", token: "space-s8" },
      { part: "In from the near corner", token: "space-s12" },
      { part: "Tile, end", token: "size-control_sm" },
      { part: "Tile to the lines", token: "space-s12" },
      { part: "Title line", token: "type-label-line" },
      { part: "Body line", token: "type-small-line" },
      { part: "Count", token: "type-body_strong-line" },
      { part: "Time left's line", token: "stroke-thin" },
    ],
    reflow: doesntApply("its width is the popup stack's."),
  },
});
