// The PopupStack page: `docs/design/components/popup-stack.md`, the shell's `Popups.qml`,
// `ToastList.qml`, `PopupHold.qml` and `PopupsBelow.qml` today. New notifications as toasts under the
// bell, each a two-line NotificationPopup on its own glass with its own time, one per app; a critical
// one as today's full card. With a card open at the bell's end they stand beside it toward the
// middle; the Notifications card opening takes them in.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import type { PopupNote } from "../../lib/notification-popup.ts";
import { ANIA, BATTERY, BUILD, CRITICAL, KUBA_TWO, MARCO, OLA, UPDATE } from "../notifications.ts";
import { besideOnScreen, popupStackOnScreen } from "./notification-parts.ts";

type Shown = "rest" | "lit" | "selected" | "open";

const SIGNAL: PopupNote = { ...KUBA_TWO, main: "Reply" };
/** A burst: four apps, Slack sending three, one after another. */
const BURST: readonly PopupNote[] = [BUILD, OLA, SIGNAL, ANIA, MARCO, BATTERY];
/** The stack as that burst leaves it, newest first. */
const SETTLED: readonly PopupNote[] = [BATTERY, { ...MARCO, count: 3 }, SIGNAL, BUILD];
const LEFT = [0.95, 0.8, 0.55, 0.3];

const still = (notes: readonly PopupNote[], extra: Parameters<typeof popupStackOnScreen>[0] = {}) => popupStackOnScreen({ notes, remaining: LEFT, unread: true, ...extra });

const STACK = ".k-popup-slot:first-child";
const INNER = ".k-popup-slot:first-child > .k-popup-inner";
const ON = "the newest popup";

const ARRIVES: Move = {
  name: "A popup arrives",
  what: "It drops `space.s8` into place as it fades in; the others make room under it.",
  tracks: [
    { part: "the drop", on: ON, target: INNER, property: "--toast-drop", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "its room", on: ON, target: STACK, property: "--slot-open", from: 0, to: 1, duration: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "its room", on: ON, target: STACK, property: "--slot-open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => still(SETTLED),
};

const DISMISSED: Move = {
  name: "Dismissed",
  what: "× or Delete: it slides out to the right, then those below close the gap.",
  tracks: [
    { part: "the slide", on: ON, target: INNER, property: "--slide-by", from: 0, to: 2, duration: "medium", curve: "emphasized_accel" },
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
    { part: "its room", on: ON, target: STACK, property: "--slot-open", from: 1, to: 0, duration: "medium", cue: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 1, to: 0, duration: "short", curve: "standard_accel" },
    { part: "its room", on: ON, target: STACK, property: "--slot-open", from: 1, to: 0, duration: "instant", cue: "short", curve: "linear" },
  ],
  back: "cuts",
  sample: () => still(SETTLED),
};

const TIME_UP: Move = {
  name: "Its time is up",
  what: "It slides up as it fades, and those below close the gap at once.",
  tracks: [
    { part: "the rise", on: ON, target: INNER, property: "--rise", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 1, to: 0, duration: "medium", curve: "standard_accel" },
    { part: "its room", on: ON, target: STACK, property: "--slot-open", from: 1, to: 0, duration: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 1, to: 0, duration: "short", curve: "standard_accel" },
    { part: "its room", on: ON, target: STACK, property: "--slot-open", from: 1, to: 0, duration: "instant", cue: "short", curve: "linear" },
  ],
  back: "cuts",
  sample: () => still(SETTLED),
};

const TAKEN: Move = {
  name: "Taken into the card",
  what: "The bell opens Notifications: each popup drops `space.s48` into it as it fades.",
  tracks: [
    { part: "the drop", on: ON, target: INNER, property: "--toast-taken", from: 0, to: 1, duration: "medium", curve: "emphasized_accel" },
    { part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
  ],
  reduced: [{ part: "the fade", on: ON, target: INNER, property: "--toast-shown", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  back: "cuts",
  sample: () => still(SETTLED),
};

const IN_STATE = (state: Shown) =>
  still(SETTLED, {
    ...(state === "lit" ? { pinned: { at: 1, as: "pointed" as const } } : {}),
    ...(state === "selected" ? { pinned: { at: 1, as: "lit" as const } } : {}),
    ...(state === "open" ? { open: 1 } : {}),
  });

export default page<Shown>({
  level: "shell",
  slug: "popup-stack",
  title: "PopupStack",
  lede: "Toasts under the bell, each on its own glass, leaving when its time is up.",
  code: { qml: "PopupStack", unbuilt: "QML: still Popups and ToastList" },
  stage: {
    live: () => popupStackOnScreen({ schedule: BURST }),
    hint: "point to hold; click opens; the bell takes them in",
    daylight: true,
  },
  variants: [
    { name: "One", for: "its time left; its length from Settings", sample: () => still([BUILD]) },
    { name: "Four apps", for: "one popup per app, newest first", sample: () => still(SETTLED) },
    { name: "Several from one app", for: "Slack's three: the count, the newest's words", sample: () => still([{ ...MARCO, count: 3 }]) },
    { name: "One opened", for: "its card in place; its time held", sample: () => still(SETTLED, { open: 1 }) },
    { name: "Critical among them", for: "today's full card, no timer", sample: () => still([CRITICAL, { ...MARCO, count: 3 }, BUILD]) },
    { name: "Beside an open card", for: "`size.gap` beside it; its pill toggles it", sample: () => besideOnScreen([BUILD, OLA], true) },
  ],
  sizes: doesntApply("`size.panel` wide; as tall as its popups."),
  states: {
    shown: ["rest", "lit", "selected", "open"],
    absent: {
      pressed: "a press opens a popup",
      disabled: "shown or gone",
      loading: "a popup arrives whole",
      empty: "no popups: nothing under the bell",
      error: "a critical popup is a variant",
    },
    rows: [{ name: "Four apps", cell: (state) => IN_STATE(state) }],
    looks: {
      rest: "each line running down on its own",
      lit: "pointer on one: its ×; every line held",
      selected: "keyboard on one: its tube and ×",
      open: "one opened into its card; its line held",
    },
  },
  focus: doesntApply("its popups light their own tube: NotificationPopup's page."),
  content: {
    text: doesntApply("its words are its popups'."),
    missingPicture: doesntApply("its popups show their app's symbol, never an image."),
    counts: [
      { name: "None", for: "nothing under the bell", sample: () => popupStackOnScreen({ notes: [] }) },
      { name: "One", for: "one popup", sample: () => still([BUILD]) },
      { name: "Five apps", for: "one each, newest first", sample: () => still([UPDATE, BATTERY, { ...MARCO, count: 3 }, SIGNAL, BUILD], { remaining: [1, 0.9, 0.7, 0.5, 0.3] }) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [DISMISSED, TIME_UP, TAKEN],
    between: powerMoves(() => still([BUILD])),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "each popup, newest first, then a critical card" },
      { keys: ["Enter"], does: "opens the popup into its card, or folds it" },
      { keys: ["Delete"], does: "dismisses it" },
      { keys: ["Esc"], does: "folds an opened one" },
    ],
    order: "Its popups are stops for Tab, newest first; it never takes the keyboard by itself.",
    lands: "A popup arriving leaves the keyboard where it was.",
  },
  measures: {
    samples: nonEmpty([{ name: "Four apps", for: "padding, gap, width", sample: () => still(SETTLED) }]),
    measures: [
      { part: "Width", token: "size-panel" },
      { part: "Under the bar, from the edges", token: "size-gap" },
      { part: "Between popups", token: "space-s4" },
      { part: "Drop in", token: "space-s8" },
      { part: "Slide out, one step", token: "space-s64" },
      { part: "Drop into the card", token: "space-s48" },
      { part: "Rise when its time is up", token: "space-s24" },
      { part: "Time left's line", token: "stroke-thin" },
    ],
    reflow: doesntApply("under the bell, `size.panel` wide, on every screen."),
  },
});
