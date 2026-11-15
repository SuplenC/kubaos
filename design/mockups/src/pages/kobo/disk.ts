// Kōbō's Disk screen: where it goes. The safe answer is picked for the person (next to Windows
// when there is room) and the map under the choice shows what each answer does to the disk.

import type { Words } from "../../lib/content.ts";
import { doesntApply, type Move, nonEmpty, page, screen } from "../../site/page.ts";
import { DATA_DISK, EMPTY_DISK, FULL_DISK, LAPTOP_DISK, SMALL_DISK, STICK } from "./kobo-data.ts";
import { at, live, onward, PANEL_MEASURES, refused } from "./kobo-pages.ts";
import { LAPTOP } from "./kobo-room.ts";

type Shown = "rest" | "selected" | "disabled" | "empty" | "error";

const machine = (...disks: (typeof LAPTOP_DISK)[]) => ({ machine: { ...LAPTOP, disks } });

const CELLS: Record<Shown, () => HTMLElement> = {
  rest: at("disk"),
  selected: at("disk", () => ({ mode: "erase" as const })),
  disabled: at("disk", () => ({ mode: "erase" as const }), machine(FULL_DISK, STICK)),
  empty: at("disk", () => ({ disk: -1 }), machine(STICK)),
  error: at("disk", () => ({ disk: 0 }), machine(SMALL_DISK, STICK)),
};

/** A part of the disk's bar by its name: the now bar's first, the after bar's last. */
const NOW = (piece: string) => `.kobo-map > .kobo-bar:nth-child(2) > [data-piece='${piece}'] > .kobo-part-fill`;
const AFTER = (piece: string) => `.kobo-map > .kobo-bar:last-child > [data-piece='${piece}']`;

const ERASING: Move = {
  name: "Erase picked",
  what: "What goes turns to an `error` outline; Windows gives its room back over `medium` and KubaOS grows across the disk.",
  tracks: [
    { part: "Windows, now", on: "the now bar", target: NOW("Windows 11"), property: "--kobo-gone", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "Windows' share", on: "the after bar", target: AFTER("Windows 11"), property: "--kobo-share", from: 0.59, to: 0, duration: "medium", curve: "standard_decel" },
    { part: "Windows showing", on: "the after bar", target: AFTER("Windows 11"), property: "--kobo-shown", from: 1, to: 0, duration: "medium", curve: "standard_decel" },
    { part: "KubaOS's share", on: "the after bar", target: AFTER("kubaos"), property: "--kobo-share", from: 0.409, to: 1, duration: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "Windows, now", on: "the now bar", target: NOW("Windows 11"), property: "--kobo-gone", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "Windows' share", on: "the after bar", target: AFTER("Windows 11"), property: "--kobo-share", from: 0.59, to: 0, duration: "instant", curve: "standard" },
    { part: "Windows showing", on: "the after bar", target: AFTER("Windows 11"), property: "--kobo-shown", from: 1, to: 0, duration: "instant", curve: "standard" },
    { part: "KubaOS's share", on: "the after bar", target: AFTER("kubaos"), property: "--kobo-share", from: 0.409, to: 1, duration: "instant", curve: "standard" },
  ],
  back: "plays",
  sample: at("disk"),
};

const OWN: Move = {
  name: "Use my password off",
  what: "The passphrase's two fields open under the switch over `medium`, fading in; on again, they shut over `short`.",
  tracks: [{ part: "the passphrase's fields", on: "the encryption card", target: ".kobo-reveal[data-reveal='passphrase']", property: "--kobo-open", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [{ part: "the passphrase's fields", on: "the encryption card", target: ".kobo-reveal[data-reveal='passphrase']", property: "--kobo-open", from: 0, to: 1, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: at("disk", () => ({ samePassword: false })),
};

const MODEL: Words = { short: "SD card", long: "Samsung SSD 990 PRO with Heatsink 2TB (MZ-V9P2T0GW)" };

export default page<Shown>({
  level: "kobo",
  slug: "disk",
  title: "Disk",
  lede: "Where it goes: the safe answer picked, and a map of what each answer does to the disk.",
  code: { unbuilt: "Kōbō draws the old screen" },
  from: ["step-rail-page", "list", "choice-card", "card", "switch", "field", "empty-state"],
  stage: { live: live("disk"), hint: "pick Erase: the map shows what goes", daylight: doesntApply("the live ISO is always dark: the installer has no day.") },
  variants: [
    { name: "Next to Windows", for: "picked for you: Windows stays, KubaOS takes the free space", sample: CELLS.rest },
    { name: "Erase picked", for: "what goes struck in `error`; KubaOS across the disk", sample: CELLS.selected },
    { name: "No room beside Windows", for: "the first card disabled with why", sample: CELLS.disabled },
    { name: "An empty disk", for: "erasing loses nothing; the first card says so", sample: at("disk", () => ({ mode: "erase" as const }), machine(EMPTY_DISK, STICK)) },
    { name: "Two disks", for: "the data disk too; the stick listed, never a target", sample: at("disk", () => ({}), machine(LAPTOP_DISK, DATA_DISK, { ...STICK, path: "/dev/sdb" })) },
    { name: "Too small", for: "a warning chip; the blocker", sample: CELLS.error },
    { name: "No disks", for: "an empty state with what to check", sample: CELLS.empty },
    { name: "Its own passphrase", for: "Use my password off: its two fields under it", sample: at("disk", () => ({ samePassword: false })) },
    { name: "Passphrases differ", for: "the blocker names it", sample: at("disk", () => ({ samePassword: false, passphrase: "kissa no hoshi", passphraseConfirm: "kissa no hoshl" })) },
    { name: "Encryption off", for: "the switch off, the cost said plainly", sample: at("disk", () => ({ encrypt: false })) },
  ],
  sizes: doesntApply("the screen's sizes are StepRailPage's; this question fills its page."),
  states: {
    shown: ["rest", "selected", "disabled", "empty", "error"],
    absent: { lit: "its cards, rows and switches light on their pages", pressed: "they press on their pages", loading: "the probe found the disks before the welcome" },
    rows: [{ name: "Disk", cell: screen((state: Shown) => CELLS[state]()) }],
    looks: {
      rest: "next to Windows picked; the map's after bar",
      selected: "erase picked; what goes struck in `error`",
      disabled: "no room beside: its card disabled with why",
      empty: "no disks: an empty state",
      error: "too small: a warning chip; the reason before Continue",
    },
  },
  focus: doesntApply("no line of its own: its list, cards and switches light on their pages."),
  content: {
    text: [{ name: "A disk's model", words: MODEL, sample: screen((words: string) => at("disk", () => ({}), machine({ ...LAPTOP_DISK, model: words }, STICK))()) }],
    missingPicture: doesntApply("no pictures: the map is drawn from the disk's parts."),
    counts: [
      { name: "None", for: "an empty state", sample: CELLS.empty },
      { name: "One", for: "the laptop's own, and the stick", sample: CELLS.rest },
      { name: "Two", for: "a second disk to pick", sample: at("disk", () => ({}), machine(LAPTOP_DISK, DATA_DISK, { ...STICK, path: "/dev/sdb" })) },
    ],
  },
  motion: {
    arrives: [onward("disk")],
    leaves: doesntApply("Continue slides Last look in: its page shows it."),
    between: [ERASING, OWN, refused(CELLS.error)],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓"], does: "picks a disk; the stick takes no pick" },
      { keys: ["←", "→"], does: "next to Windows, or erase" },
      { keys: ["Space"], does: "turns the switch it is on" },
      { keys: ["Tab", "Shift+Tab"], does: "disks, the choice, the two switches, the passphrase, Back, Continue" },
      { keys: ["Enter"], does: "Continue, or the reason flashes" },
      { keys: ["Esc"], does: "back a screen" },
    ],
    order: "The disks, the two choices as one stop, Encrypt, Use my password, the passphrase's fields while it is off, Back, Continue.",
    lands: "On the choice picked for you, so ← → is the first thing to try.",
  },
  measures: {
    samples: nonEmpty([{ name: "Erase picked", for: "the choice, the map, the card", sample: CELLS.selected }]),
    measures: nonEmpty([
      ...PANEL_MEASURES,
      { part: "Between sections", token: "space-s24" },
      { part: "Between the choices", token: "space-s16" },
      { part: "Choice → map → line", token: "space-s12" },
      { part: "A bar's height", token: "size-control_sm" },
      { part: "Between a bar's parts", token: "space-s4" },
      { part: "Bar's label column", token: "space-s16" },
      { part: "A part's corner", token: "radius-small" },
    ]),
    reflow: { note: "The map's parts keep their share of the width; a part too narrow for its words drops them.", sample: live("disk") },
  },
});
