// The Dialog and Scrim page: `docs/design/components/dialog.md`. A question over a dimmed screen,
// raised to the ladder's step 3; a Danger one for a step that destroys something, and a prompt for
// what only you can type. Never shorter than 5:2 of its width, nor taller than its room less 48
// above and below, past which its middle scrolls. The keyboard goes into it, goes round it, and
// goes back as it closes.

import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { centreIn, type DialogOptions, dialog, scrim } from "../../lib/dialog.ts";
import { field, labelled } from "../../lib/field.ts";
import { row, stack } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const TITLE: Words = { short: "Restart now?", long: "Restart now to finish installing the updates you downloaded?" };
const BODY: Words = {
  short: "Everything open closes. I'll be back in a few seconds.",
  long: "Everything open closes, and anything you haven't saved is lost. The updates install while the screen is dark, and I'll be back in about a minute.",
};

const QUESTION: DialogOptions = {
  icon: "power_settings_new",
  title: "Restart now?",
  body: "Everything open closes. I'll be back in a few seconds.",
  dismiss: "Cancel",
  actions: [
    { label: "Shut down", kind: "Tonal", icon: "power_settings_new" },
    { label: "Restart", icon: "restart_alt" },
  ],
};

/** One line and three buttons: shorter than its least height, which it keeps. */
const SHORT: DialogOptions = { ...QUESTION, body: "Everything open closes." };

/** The files a long Danger dialog lists. */
const FILES = Array.from({ length: 40 }, (_, index) => `holiday-${String(index + 1).padStart(2, "0")}.jpg`);

/** Forty files to delete: more than a short window shows, so its middle scrolls. */
const LONG: DialogOptions = {
  tone: "Danger",
  title: "Delete 40 files?",
  body: "These go for good, not to the bin:",
  lists: FILES,
  dismiss: "Cancel",
  actions: [{ label: "Delete 40 files", kind: "Danger", icon: "warning" }],
};

const DANGER: DialogOptions = {
  tone: "Danger",
  title: "Last chance",
  body: "Everything on Samsung SSD 990 PRO (nvme0n1, 2 TB) gets erased. There's no undo.",
  dismiss: "Cancel",
  actions: [{ label: "Erase and install", kind: "Danger", icon: "warning" }],
};

/** The password prompt: the field it asks for, wrong once when `wrong`. */
function prompt(wrong = false, waiting = false): DialogOptions {
  const password = field({ icon: "key", secret: true, ...(wrong ? { value: "hunter", state: "Invalid" as const } : {}) });
  return {
    icon: "admin_panel_settings",
    title: "Authentication required",
    body: "Installing btop needs your password.",
    asks: labelled("Password for Kuba", password, wrong ? { note: "That didn't work. Fat fingers? Try again.", invalid: true } : {}),
    dismiss: "Cancel",
    actions: [{ label: "Authenticate", ...(waiting ? { disabled: true } : {}) }],
  };
}

/** `options`' dialog over its scrim in a window's room, drawn still, the keyboard on where it lands. */
function still(options: DialogOptions): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-dialog-room";
  room.append(scrim(), dialog({ ...options, landed: true }).element);
  return room;
}

/** `options`' dialog drawn still in a room an app's least height tall (`size.app_min_height`),
 *  which caps it: its middle scrolls, `down` of the way. */
function capped(options: DialogOptions, down = 0): HTMLElement {
  const room = still(options);
  room.dataset.tall = "";
  room.style.setProperty("--tall", "var(--size-app_min_height)");
  const scroller = room.querySelector<HTMLElement>(".k-dialog-scroll")!;
  requestAnimationFrame(() => {
    scroller.scrollTop = Math.round((scroller.scrollHeight - scroller.clientHeight) * down);
  });
  return room;
}

/** A window with four things to ask about; each opens its dialog over the scrim. */
function live(): HTMLElement {
  const room = document.createElement("span");
  room.className = "k-dialog-room";
  room.dataset.live = "";
  room.style.setProperty("--room", "calc(var(--size-dialog) + 2 * var(--space-s64))");
  room.style.setProperty("--tall", "calc(var(--size-column) * 3 / 4)");
  const openers: [string, () => DialogOptions][] = [
    ["Restart…", () => QUESTION],
    ["Erase the disk…", () => DANGER],
    ["Install btop…", () => prompt()],
    ["Delete the photos…", () => LONG],
  ];
  const buttons = openers.map(([label, options], index) => {
    const opener = button({ label, kind: index === 0 ? "Tonal" : "Text", size: "Sm" });
    opener.addEventListener("click", () => open(opener, options()));
    return opener;
  });
  const behind = stack("s8", "start", row("s8", "centre", ...buttons), text("Each opens its dialog; Esc or Cancel closes it.", "small", "on_surface_variant"));
  behind.classList.add("k-dialog-behind");
  room.append(behind);

  function open(opener: HTMLElement, options: DialogOptions): void {
    const dim = scrim();
    const asked = dialog(options);
    room.append(dim, asked.element);
    centreIn(room, asked.element);
    asked.land();
    asked.onAnswer = () => {
      for (const each of [dim, asked.element]) {
        each.dataset.leaving = "";
        each.addEventListener("transitionend", (event) => {
          if (event.propertyName === "opacity") each.remove();
        });
      }
      opener.focus({ focusVisible: true } as FocusOptions);
    };
  }

  return room;
}

const SCRIM = "the scrim";
const DIALOG = "the dialog";

const ARRIVES: Move = {
  name: "Opening",
  what: "The scrim fades in over `long` on `emphasized_decel` as the dialog fades in with it, rising `space.s16` into the middle a whole device pixel at a time, so its edges stay sharp: a dialog arriving, looked at as it comes. Nothing flickers: a dialog isn't a light. Its buttons are lit as the keyboard lands on one. With reduced motion both fade in where they stay, capped at `short`, with no rise.",
  tracks: [
    { part: SCRIM, on: "a question", target: ".k-scrim", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: `${DIALOG}'s fade`, on: "a question", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: `${DIALOG}'s rise`, on: "a question", target: ".k-dialog", property: "--rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
  ],
  reduced: [
    { part: SCRIM, on: "a question", target: ".k-scrim", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: `${DIALOG}'s fade`, on: "a question", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
  ],
  back: "cuts",
  sample: () => still(QUESTION),
};

const LEAVES: Move = {
  name: "Closing",
  what: "An answer, Esc or Cancel: the dialog and the scrim fade out together where they are over `medium` on `emphasized_accel`, already done with, and the keyboard is back on what opened it. With reduced motion the fade is capped at `short`.",
  tracks: [
    { part: SCRIM, on: "a question", target: ".k-scrim", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: DIALOG, on: "a question", target: ".k-dialog", property: "opacity", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
  ],
  reduced: [
    { part: SCRIM, on: "a question", target: ".k-scrim", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: DIALOG, on: "a question", target: ".k-dialog", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
  ],
  back: "cuts",
  sample: () => still(QUESTION),
};

const DESCRIBED =
  "`size.dialog` (500) wide, `space.s24` inside on every side, under its buttons too, at the content radius (`radius.content`, 16), raised to the ladder's step 3, a dialog's: `surface_container_high` by day and `surface_container_highest` at night, the step's contact and ambient layers of `shadow`, and the rim, a hairline of `on_surface` inside its edge at `elevation.rim`. A 24 symbol (`heading`'s) in `primary` and the title in `heading`, `on_surface`, `space.s12` apart; the body in `body`, `on_surface_variant`, `space.s12` under the title and in line with it, beside the symbol; what it asks for `space.s16` under the body, in line with it too; and its button bar `space.s24` under the last, across its whole width. Its words wrap in its width.";

const HEIGHTS =
  "It is never shorter than `aspect.dialog`, 5:2, of its width: 200, on the grid. A dialog that asks little, one line and its buttons, would be a strip; it stands as a card instead, its words at the top and its buttons at its foot, `space.s24` from the bottom as always, with the room it gains between them. A dialog with no buttons (a progress) keeps it too, its words at the top. Taller than that, it is as tall as what it holds. It is never taller than its room (the screen, or the window it opens over) less `space.s48` above and below, so the dimmed screen always shows round it. Past that only the middle scrolls, the body and what it lists or asks for, with no bar of its own: the symbol and title stay at the top and the buttons at the foot. While more is below, a hairline in the soft outline, a divider's, crosses the dialog's padding above the buttons, the middle scrolling under it; while more is above, the same one under the title. Each fades over `short` on `standard` as you reach its end.";

export default page({
  level: "molecules",
  slug: "dialog",
  title: "Dialog and Scrim",
  summary: "A question over a dimmed screen, raised to step 3: Cancel at the start, the actions at the end, the one Enter presses last.",
  live,
  description: [
    "Rust `dialog::dialog`, `dialog::prompt` and `dialog::scrim`, QML `Dialog` and `Scrim`. Press one of the window's buttons: its dialog opens over the scrim with the keyboard in it. Tab goes round it, Esc or Cancel closes it, and the keyboard is back on the button.",
    DESCRIBED,
    "The buttons are a button bar (Molecules, Button bar): the dismiss button, Cancel, a Text button, at the start; the actions at the end, the one Enter presses last: the screen's one Filled button, a Tonal one for another way out before it (Shut down beside Restart), `space.s8` apart, all regular (`md`). A Danger dialog asks before a step that destroys something: the warning sign in `error` instead of the symbol, an edge of `error` at `opacity.danger_edge` over the rim, and its one action a Danger button naming what it does (Erase and install). A prompt asks for what only you can type (a password): a labelled field `space.s16` under the body, Enter in it doing what its action does; a wrong answer is the field's Invalid state and its note, and the dialog stays.",
    HEIGHTS,
    "The scrim is everything behind the dialog, dimmed: `scrim` at `opacity.scrim` over all of the window or the screen. It takes every press, hover and scroll, so none reaches what is behind, and a press on it does nothing: Esc or Cancel closes the dialog, so a stray press never loses what was typed.",
    "Opening, the keyboard goes into the dialog: onto what it asks for, else onto Cancel on a Danger dialog, so Enter twice never destroys anything, else onto the last action. Tab and Shift+Tab go round its field and buttons and never leave it, scrolling a field in a scrolled middle into view `space.s12` clear of its edges. From a button, the arrows scroll a middle too tall to show whole by a line and the page keys by what it shows. Esc does what Cancel does. Closing puts the keyboard back on what opened it.",
  ],
  variants: [
    { name: "A question", for: "Ask before something that can't wait to be undone: Restart, with Shut down as another way out.", sample: () => still(QUESTION) },
    { name: "Short", for: "One line and its buttons: the dialog keeps its least height, 5:2 of its width, the room it gains above its buttons.", sample: () => still(SHORT) },
    { name: "Long", for: "More than its room shows: capped `space.s48` from the room's top and bottom, the title and buttons stay, the middle scrolls, with a hairline above the buttons and, scrolled, one under the title.", sample: () => capped(LONG, 0.5) },
    { name: "Danger", for: "Ask before something is destroyed: the warning sign and its edge in `error`, a Danger action, the keyboard on Cancel.", sample: () => still(DANGER) },
    { name: "A prompt", for: "Ask for what only you can type: the password, its field under the body.", sample: () => still(prompt()) },
    { name: "A prompt, answered wrong", for: "The field Invalid with its note; the dialog stays and the keyboard stays in the field.", sample: () => still(prompt(true)) },
    { name: "Waiting", for: "Nothing to answer yet (the system hasn't asked): the action disabled until it can be taken.", sample: () => still(prompt(false, true)) },
  ],
  sizes: doesntApply("a dialog is one width, `size.dialog`, and as tall as what it holds, between 5:2 of its width and its room less `space.s48` above and below."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a dialog isn't a control: its buttons and its field light, each as its own page says.",
      pressed: "a press on it, or on the scrim, does nothing of its own.",
      disabled: "a dialog is open or not there; an action that can't be taken yet is a disabled button (Waiting, above).",
      selected: "it holds no choice of its own.",
      loading: "what it asks is known as it opens; a step that takes a moment after the answer shows on the screen behind, once it has closed.",
      empty: "a dialog always asks something.",
      error: "a wrong answer is its field's Invalid state and note (A prompt, answered wrong, above); the dialog itself can't be wrong.",
    },
    rows: [
      { name: "A question", cell: () => still(QUESTION) },
      { name: "Danger", cell: () => still(DANGER) },
      { name: "A prompt", cell: () => still(prompt()) },
      { name: "Long", cell: () => capped(LONG, 0.5) },
    ],
    described: {
      rest: "Open over its scrim, the keyboard on where it lands. At night `surface_container_highest`, the step's stronger shadow and the rim at `elevation.rim`; by day `surface_container_high`, the lighter shadow and the rim at `elevation.light_rim`. The scrim is the same black at `opacity.scrim` in both.",
    },
  },
  focus: doesntApply("the dialog has no line of its own: its buttons and its field light as their pages say. The keyboard lands on what it asks for, else Cancel on a Danger dialog, else the last action."),
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => still({ ...QUESTION, title: words }) },
      { name: "The body", words: BODY, sample: (words: string) => still({ ...QUESTION, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is always there: `info` unless it names one, and the warning sign on a Danger dialog."),
    counts: [
      { name: "One action", for: "Cancel and the action: Authenticate.", sample: () => still(prompt()) },
      { name: "Two actions", for: "Another way out before the one Enter presses: Shut down, Restart.", sample: () => still(QUESTION) },
      { name: "Forty lines", for: "What it lists, more than its room shows: its middle scrolls, at its top, half way and at its end.", sample: () => row("s16", "start", capped(LONG), capped(LONG, 0.5), capped(LONG, 1)) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: doesntApply("a dialog doesn't change while it is open; its buttons and its field move between their own states."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Round what it asks for and its buttons, from the last back to the first, scrolling a field in a scrolled middle into view; never out of the dialog while it is open." },
      { keys: ["Up", "Down"], does: "From a button, scroll a middle too tall to show whole by a line." },
      { keys: ["Page Up", "Page Down"], does: "From a button, scroll it by what it shows, less a line." },
      { keys: ["Enter"], does: "Presses the button the keyboard is on, once per press; in the field, does what its action does." },
      { keys: ["Space"], does: "Presses the button the keyboard is on; in the field, a space." },
      { keys: ["Esc"], does: "Does what Cancel does: closes it without an answer." },
    ],
    order: "What it asks for, then Cancel, then the actions in order, round; a disabled action is skipped. Nothing behind the scrim is a stop while it is open.",
    lands: "Opening: on what it asks for, else Cancel on a Danger dialog, else the last action. Closing: back on what opened it.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A prompt", for: "Its width, padding, the symbol and title, the gaps between the title, the body, the field and the buttons.", sample: () => still(prompt()) },
      { name: "A question", for: "Two actions at the end and Cancel at the start.", sample: () => still(QUESTION) },
      { name: "Short", for: "Its least height, the buttons at its foot.", sample: () => still(SHORT) },
      { name: "Long", for: "Its most height in a short room, the rules at the middle's edges.", sample: () => capped(LONG, 0.5) },
    ]),
    measures: [
      { what: "Its width.", token: "size-dialog" },
      { what: "Its least height: 5:2 of its width, 200.", token: "aspect-dialog" },
      { what: "Its most height: its room less this above and below.", token: "space-s48" },
      { what: "The rules at the middle's edges: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "Inside its edges on every side, under the buttons too, and above the buttons.", token: "space-s24" },
      { what: "Between the symbol and the title, and between the title and the body.", token: "space-s12" },
      { what: "Above what it asks for.", token: "space-s16" },
      { what: "Between buttons.", token: "space-s8" },
      { what: "The symbol, `heading`'s.", token: "type-heading-icon" },
      { what: "The title's line.", token: "type-heading-line" },
      { what: "A button's height.", token: "size-control_md" },
      { what: "Its corners.", token: "radius-content" },
      { what: "The rim inside its edge: one device pixel.", token: "stroke-hairline_device_px" },
      { what: "Its shadow's ambient layer: its drop.", token: "elevation-3-ambient-y" },
      { what: "Its shadow's ambient layer: its blur.", token: "elevation-3-ambient-blur" },
      { what: "How far under its place it starts as it opens.", token: "space-s16" },
    ],
    reflow: doesntApply("a dialog is `size.dialog` wide at every window size, in the middle of the scrim, which fills the window or the screen: the narrowest window, an app's minimum (`size.app_min_width`, 720), holds it with room."),
  },
});
