// The Dialog and Scrim page: `docs/design/components/dialog.md`. A question over a dimmed screen,
// raised to the ladder's step 3; a Danger one for a step that destroys something, and a prompt for
// what only you can type. Never shorter than 5:2 of its width, nor taller than its room less 48
// above and below, past which its middle scrolls. The keyboard goes into it, goes round it, and
// goes back as it closes.

import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { centreIn, type DialogOptions, dialog, scrim } from "../../lib/dialog.ts";
import { field, labelled } from "../../lib/field.ts";
import { row } from "../../lib/layout.ts";
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

/** An action whose name, as long as a translation can make it, leaves its buttons no room for one
 *  line: they stack, Restart on top and Cancel at the foot. */
const STACKED: DialogOptions = {
  ...QUESTION,
  body: BODY.long,
  actions: [QUESTION.actions[0]!, { label: "Restart and install the updates", icon: "restart_alt" }],
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
  room.style.setProperty("--room", "100%");
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
  const behind = row("s8", "centre", ...buttons);
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
      opener.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
    };
  }

  return room;
}

const SCRIM = "the scrim";
const DIALOG = "the dialog";

const ARRIVES: Move = {
  name: "Opening",
  what: "The scrim and dialog fade in over `long` on `emphasized_decel`, the dialog rising `space.s16`; reduced, a fade capped at `short`.",
  tracks: [
    { part: SCRIM, on: "a question", target: ".k-scrim", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: `${DIALOG}'s fade`, on: "a question", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: `${DIALOG}'s rise`, on: "a question", target: ".k-dialog", property: "--dialog-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
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
  what: "Both fade out over `medium` on `emphasized_accel`; the keyboard goes back to what opened it.",
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

export default page({
  level: "molecules",
  slug: "dialog",
  title: "Dialog and Scrim",
  lede: "A question over a dimmed screen, raised to step 3, with the keyboard kept inside until it is answered.",
  code: { rust: "dialog::dialog, dialog::prompt, dialog::scrim", qml: "Dialog, Scrim" },
  stage: {
    live,
    hint: "press a button; Tab goes round, Esc closes",
    daylight: true,
  },
  variants: [
    { name: "A question", for: "before something that can't wait to be undone", sample: () => still(QUESTION) },
    { name: "Short", for: "keeps its least height, `aspect.dialog`", sample: () => still(SHORT) },
    { name: "Long", for: "its middle scrolls; title and buttons stay", sample: () => capped(LONG, 0.5) },
    { name: "Danger", for: "before destroying: `error` edge, the keyboard on Cancel", sample: () => still(DANGER) },
    { name: "A prompt", for: "what only you can type", sample: () => still(prompt()) },
    { name: "A prompt, answered wrong", for: "the field Invalid; the dialog and keyboard stay", sample: () => still(prompt(true)) },
    { name: "Waiting", for: "the action disabled until it can be taken", sample: () => still(prompt(false, true)) },
  ],
  sizes: doesntApply("one width, `size.dialog`; as tall as it holds, within `aspect.dialog` and its room."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "not a control: its buttons and field light",
      pressed: "a press on it or the scrim does nothing",
      disabled: "open or not there",
      selected: "holds no choice",
      loading: "a slow step shows behind, once it has closed",
      empty: "always asks something",
      error: "a wrong answer is its field's Invalid state",
    },
    rows: [
      { name: "A question", cell: () => still(QUESTION) },
      { name: "Danger", cell: () => still(DANGER) },
      { name: "A prompt", cell: () => still(prompt()) },
      { name: "Long", cell: () => capped(LONG, 0.5) },
    ],
    looks: {
      rest: "`surface_container_highest` at night, `…_high` by day, over `opacity.scrim`",
    },
  },
  focus: doesntApply("no line of its own: its buttons and field light as their pages say."),
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => still({ ...QUESTION, title: words }) },
      { name: "The body", words: BODY, sample: (words: string) => still({ ...QUESTION, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is always there: `info` unless named, the warning sign on Danger."),
    counts: [
      { name: "One action", for: "Cancel and the action", sample: () => still(prompt()) },
      { name: "Two actions", for: "another way out, before the Enter one", sample: () => still(QUESTION) },
      { name: "Stacked buttons", for: "names too long: Enter's on top, Cancel at the foot", sample: () => still(STACKED) },
      { name: "Forty lines", for: "its middle at the top, half way, the end", sample: () => row("s16", "start", capped(LONG), capped(LONG, 0.5), capped(LONG, 1)) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: doesntApply("it doesn't change while open; its buttons and field move on their own."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "round its field and buttons, never out" },
      { keys: ["Up", "Down"], does: "from a button, scroll a tall middle a line" },
      { keys: ["Page Up", "Page Down"], does: "from a button, scroll it a page" },
      { keys: ["Enter"], does: "presses the button; in the field, its action" },
      { keys: ["Space"], does: "presses the button; in the field, a space" },
      { keys: ["Esc"], does: "does what Cancel does" },
    ],
    order: "What it asks, Cancel, then the actions, round; a disabled action is skipped, nothing behind is a stop.",
    lands: "Opening: what it asks, else Cancel on Danger, else the last action. Closing: back on what opened it.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A prompt", for: "width, padding, title, and the gaps down to the buttons", sample: () => still(prompt()) },
      { name: "A question", for: "Cancel at the start, two actions at the end", sample: () => still(QUESTION) },
      { name: "Short", for: "its least height", sample: () => still(SHORT) },
      { name: "Long", for: "its most height, the rules at the middle's edges", sample: () => capped(LONG, 0.5) },
    ]),
    measures: [
      { part: "Width", token: "size-dialog" },
      { part: "Least height", token: "aspect-dialog" },
      { part: "Clear of its room", token: "space-s48" },
      { part: "Inset, above the buttons", token: "space-s24" },
      { part: "Cancel's hang, gaps by the title", token: "space-s12" },
      { part: "Above what it asks", token: "space-s16" },
      { part: "Between buttons", token: "space-s8" },
      { part: "Symbol", token: "type-heading-icon" },
      { part: "Title line", token: "type-heading-line" },
      { part: "Button", token: "size-control_md" },
      { part: "Corner", token: "radius-content" },
      { part: "Rim, the middle's rules", token: "stroke-hairline_device_px" },
      { part: "Shadow drop", token: "elevation-3-ambient-y" },
      { part: "Shadow blur", token: "elevation-3-ambient-blur" },
      { part: "Rise as it opens", token: "space-s16" },
    ],
    reflow: doesntApply("one width in the middle of the scrim; an app's least window holds it with room."),
  },
});
