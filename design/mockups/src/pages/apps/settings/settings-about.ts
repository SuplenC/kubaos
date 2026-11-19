// Settings › About: the machine's name and its rename, the update at a glance in each of its states,
// the hardware's tiles and the facts in groups with the hidden ones, Copy all and its toast; and its
// page, Updates, with the card, what an update holds, the automatic checks and installs, the
// channel, the steps and the history. A rename, a channel or a rollback asks in the library's Dialog.

import { dialogOver, type DialogOptions } from "../../../lib/dialog.ts";
import { type Move, screen, type Track, type Variant } from "../../../site/page.ts";
import { ABOUT, type Values } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { settingsWindow } from "./settings-window.ts";
import { copiedFact, copiedWords, HIDDEN, MACHINES, renameDialog, toastStill } from "./sections/about-parts.ts";
import { channelQuestion, FAILS_AT, rollBackQuestion } from "./sections/updates-parts.ts";

/** The section's own values over its defaults, under its key. */
const own = (about: object, more: Partial<Values> = {}): Partial<Values> => ({ seed: { about }, ...more });

/** The sub-sections, by their place. */
const NAME = 0;
const UPDATES = 1;
const HARDWARE = 2;
const SYSTEM = 3;
const IDENTITY = 4;

const CHECKING = { update: "checking" };
const WAITING = { update: "waiting" };
const UPDATING = { update: "updating", at: 4 };
const FAILED = { update: "failed", at: FAILS_AT };
const RENAMED = { name: "kuba-studio", was: "kuba-pc" };
const LONGEST = { machine: 2, name: MACHINES[2]!.name };
const DESKTOP = { machine: 1, name: MACHINES[1]!.name };

/** About scrolled to sub-section `sub`, with `about`. */
const at = (sub: number, about: object = {}) => screen(() => settingsWindow({ section: ABOUT, sub, values: own(about) }));

/** The Updates page open, with `about`. */
const updates = (about: object = {}) => screen(() => settingsWindow({ section: ABOUT, values: own(about, { opened: "updates" }) }));

/** About with the rename dialog over it, `typed` in its field, the keyboard in it. */
const renaming = (typed: string, about: object = {}) =>
  screen(() => {
    const frame = settingsWindow({ section: ABOUT, values: own(about) });
    renameDialog(frame, (about as { name?: string }).name ?? "kuba-pc", () => {}, typed, true);
    return frame;
  });

/** The window with `about` on page `opened`, asking `question` over it. */
const asking = (about: object, question: DialogOptions) =>
  screen(() => {
    const frame = settingsWindow({ section: ABOUT, values: own(about, { opened: "updates" }) });
    dialogOver(frame, question, () => {}, true);
    return frame;
  });

/** About at its identity, a toast saying what Copy all copied. */
const copied = (about: object, count: number, left: number) => () => toastStill(settingsWindow({ section: ABOUT, sub: IDENTITY, values: own(about) }), copiedWords(count, left));

const VARIANTS: Variant[] = [
  { name: "Up to date", for: "one line: the state, Updates, Check now; the tip", sample: at(UPDATES) },
  { name: "Checking", for: "Check now off while it asks", sample: at(UPDATES, CHECKING) },
  { name: "An update is ready", for: "the release and its size; Update", sample: at(UPDATES, WAITING) },
  { name: "Updating", for: "the step running; its bar under the line", sample: at(UPDATES, UPDATING) },
  { name: "Couldn't update", for: "Try again on the line; ErrorBlock under it", sample: at(UPDATES, FAILED) },
  { name: "Rename, unchanged", for: "the current name; Rename off", sample: renaming("kuba-pc") },
  { name: "Rename, empty", for: "Kōbō's “Name this machine.”; Rename off", sample: renaming("") },
  { name: "Rename, invalid", for: "a space: Kōbō's rule under the field", sample: renaming("kuba pc") },
  { name: "Rename, valid", for: "a new name; Rename on", sample: renaming("kuba-studio") },
  { name: "Renamed", for: "the new name; the banner until the restart", sample: at(NAME, RENAMED) },
  { name: "The longest name and model", for: "the name breaks anywhere; the line wraps", sample: at(NAME, LONGEST) },
  { name: "Renamed, longest", for: "the banner's line wraps", sample: at(NAME, { ...LONGEST, was: "kuba-pc" }) },
  { name: "One disk, one screen", for: "the laptop: a line each, a battery", sample: at(HARDWARE) },
  { name: "Four disks, three screens", for: "the desktop: numbered lines; no battery", sample: at(HARDWARE, DESKTOP) },
  { name: "Nearly full", for: "memory and disk past their margin", sample: at(HARDWARE, LONGEST) },
  { name: "Identity, hidden", for: "Show in place of each value", sample: at(IDENTITY) },
  { name: "Identity, shown", for: "each value in `mono`, kept for the window", sample: at(IDENTITY, { shown: [...HIDDEN] }) },
  { name: "A line's copy button", for: "at the line's end while pointed at; lit", sample: at(SYSTEM, { pinCopy: "Kernel" }) },
  { name: "A shown fact's copy button", for: "only once its value shows", sample: at(IDENTITY, { shown: [...HIDDEN], pinCopy: "Serial number" }) },
  { name: "A hidden fact", for: "no copy button until shown", sample: at(IDENTITY, { pinCopy: "Serial number" }) },
  { name: "One fact copied", for: "“Label: value”; the toast names it", sample: screen(() => toastStill(settingsWindow({ section: ABOUT, sub: SYSTEM, values: own({}) }), copiedFact({ label: "Kernel", value: "" }))) },
  { name: "Reporting a problem?", for: "Info banner: title, line, Copy all inside", sample: at(IDENTITY) },
  { name: "Copied, hidden left out", for: "the toast says what stayed out", sample: screen(copied({}, 22, 3)) },
  { name: "Copied, everything", for: "every fact shown first", sample: screen(copied({ shown: [...HIDDEN] }, 25, 0)) },
  { name: "Updates, checking", for: "the page's card while it asks", sample: updates(CHECKING) },
  { name: "Updates, ready", for: "what it holds by part, sizes; what's new", sample: updates(WAITING) },
  { name: "Updates, updating", for: "the card's bar; the steps follow", sample: updates(UPDATING) },
  { name: "Updates, failed", for: "the step it stopped at; Roll back, Try again", sample: updates({ ...FAILED, logOpen: true }) },
  { name: "Installing on its own", for: "When and the notice show under it", sample: updates({ autoInstall: true }) },
  { name: "Checks off", for: "How often leaves", sample: updates({ autoCheck: false }) },
  { name: "Switching to edge", for: "Dialog: what edge means", sample: asking({}, channelQuestion(1, "0.4.0")) },
  { name: "Rolling back", for: "Dialog: what goes back and what stays", sample: asking(FAILED, rollBackQuestion("0.4.0")) },
];

const TOAST = "Copy all's toast";
const fade: Track<"emphasized_decel"> = { part: "its fade", on: TOAST, target: ".k-toast", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" };
const DIALOG = "the rename dialog";
const dialogFade: Track<"emphasized_decel"> = { part: "its fade", on: DIALOG, target: ".k-dialog, .k-scrim", property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" };

const COPY = "a line's copy button";
const MOVES: Move[] = [
  {
    name: "A line's copy button shows",
    what: "Pointed at or reached by the keyboard, it fades in over `short` on `standard`; nothing moves.",
    tracks: [{ part: "its fade", on: COPY, target: ".ab-copy[data-shown]", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
    reduced: "same",
    back: "plays",
    sample: at(SYSTEM, { pinCopy: "Kernel" }),
  },
  {
    name: "Rename opens",
    what: "The scrim and the dialog fade in over `long` on `emphasized_decel`, the dialog rising; no rise reduced.",
    tracks: [dialogFade, { part: "its rise", on: DIALOG, target: ".k-dialog", property: "--dialog-rise", from: 1, to: 0, duration: "long", curve: "emphasized_decel" }],
    reduced: [dialogFade],
    back: "cuts",
    sample: renaming("kuba-pc"),
  },
  {
    name: "Copy all confirms",
    what: "The toast fades in over `short` on `emphasized_decel`, rising `space.s32`; no rise reduced.",
    tracks: [fade, { part: "its rise", on: TOAST, target: ".k-toast", property: "--toast-rise", from: 1, to: 0, duration: "short", curve: "emphasized_decel" }],
    reduced: [fade],
    back: "cuts",
    sample: copied({}, 22, 3),
  },
];

export default sectionPage({
  from: ["card", "icon-tile", "meter", "description-list", "dialog", "field", "banner", "toast", "button-bar", "error-block", "progress-bar", "log", "step"],
  index: ABOUT,
  slug: "settings-about",
  title: "Settings: About",
  lede: "The machine's name, where updates stand, and every fact about it, ready to copy.",
  hint: "press Rename; point at a fact, copy it",
  seed: own({ autoInstall: true, ...WAITING }),
  variants: VARIANTS,
  moves: MOVES,
  keys: [
    { keys: ["Enter"], does: "in the rename field: Rename, once it is on" },
    { keys: ["Esc"], does: "in a dialog: Cancel, nothing changed" },
    { keys: ["Tab"], does: "Rename, the card's buttons, each copy button and Show, Copy all" },
    { keys: ["Enter", "Space"], does: "on a copy button: copies its line" },
  ],
  measures: [
    { part: "The name card, inside", token: "space-s16" },
    { part: "The system's mark", token: "size-control_lg" },
    { part: "Mark to name", token: "space-s16" },
    { part: "Tiles apart", token: "space-s16" },
    { part: "A tile's meter", token: "size-progress" },
    { part: "Update line to its tip", token: "space-s12" },
    { part: "Update line's buttons apart", token: "space-s8" },
    { part: "A line's copy button", token: "size-control_sm" },
    { part: "A rename's dialog", token: "size-dialog" },
    { part: "Field to its message", token: "space-s4" },
  ],
});
