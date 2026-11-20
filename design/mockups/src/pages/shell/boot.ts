// The Boot page (`boot-parts.ts` draws it): the boot menu, the splash and its passphrase, into the
// lock and the desktop, and back down to black at power off, the sign in the exact centre of every
// screen throughout.

import { fixed } from "../../lib/content.ts";
import { doesntApply, type Move, nonEmpty, page, screen as whole } from "../../site/page.ts";
import { goingDown, goingDownReduced, goingDownUnflickered, room, SHUTDOWN } from "./power-room.ts";
import {
  ADVANCED,
  type Drawn,
  ENTRIES,
  choosing,
  from,
  HINT,
  intoTheLock,
  LAPTOP,
  liveBoot,
  LONG_ENTRY,
  MANY,
  MONITOR,
  named,
  restarting,
  screen,
  screens,
  shaking,
  SNAPSHOTS,
  starting,
  startingReduced,
  startingUnflickered,
  straightIn,
} from "./boot-parts.ts";

const at = (drawn: Drawn) => whole(() => screen(drawn).element);

/** The two-screen sample: the laptop and a monitor, the field on the primary alone, or on each. */
const twoScreens = (fallback: boolean) =>
  whole(() => screens(named(LAPTOP, screen({ scene: "splash", display: LAPTOP, prompt: "empty" }).element), named(MONITOR, screen({ scene: "splash", display: MONITOR, prompt: fallback ? "empty" : "none", primary: fallback }).element)));

/** The boot menu on two screens: the menu on the primary alone, the unlit sign on the other. */
const twoMenus = whole(() => screens(named(LAPTOP, screen({ scene: "menu", display: LAPTOP }).element), named(MONITOR, screen({ scene: "menu", display: MONITOR, primary: false }).element)));

/** バ's blink as the recipe shows it: down at its cue, back a `micro` later. */
const BLINKS = [
  { part: "バ's blink", on: "the boot screen", target: ".neon-sign-ba", property: "opacity", from: 1, to: 0.35, duration: "instant", curve: "standard", cue: ["scene", "slow"] },
  { part: "バ's blink", on: "the boot screen", target: ".neon-sign-ba", property: "opacity", from: 0.35, to: 1, duration: "instant", curve: "standard", cue: ["scene", "slow", "micro"] },
] as const;

const POWERED = () => {
  const box = document.createElement("span");
  box.className = "power-held";
  box.append(room({ open: true, at: SHUTDOWN, held: 1 }));
  return box;
};

const CHOOSES: Move = {
  name: "Choosing an entry",
  what: "Enter: GRUB clears the menu, the countdown and the hints at once; the unlit sign stays.",
  tracks: choosing(),
  reduced: "same",
  back: "cuts",
  sample: from(choosing(), () => screen({ scene: "menu" }).element),
};

const STARTS: Move = {
  name: "Starting, the disk encrypted",
  what: "The field is there at once; a group strikes through `neon.strike` over `medium`, ク first, OS last.",
  tracks: starting(true),
  reduced: startingReduced(true),
  unflickered: startingUnflickered(true),
  back: "cuts",
  sample: from(starting(true), () => screen({ scene: "splash", prompt: "empty" }).element),
};

const STARTS_PLAIN: Move = {
  name: "Starting, no encrypted disk",
  what: "The same switch-on with nothing under the sign.",
  tracks: starting(false),
  reduced: startingReduced(false),
  unflickered: startingUnflickered(false),
  back: "cuts",
  sample: from(starting(false), () => screen({ scene: "splash" }).element),
};

const HUMS: Move = {
  name: "Humming, once lit",
  what: "Once lit: the sign steps 0.97–1 every `micro`; バ blinks 0.35, 1, 0.5, 1 every 4 s. The recipe shows the blink's span.",
  tracks: [...BLINKS],
  reduced: [{ part: "no hum", on: "the boot screen", target: ".neon-sign-ba", property: "opacity", from: 1, to: 1, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: at({ scene: "splash" }),
};

const SHAKES: Move = {
  name: "A wrong passphrase",
  what: "The field shakes as the lock's does, empties and turns Invalid; reduced motion keeps it still.",
  tracks: shaking(),
  reduced: [{ part: "the field's shake", on: "the boot screen", target: ".boot-field", property: "--jolt", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: at({ scene: "splash", prompt: "wrong" }),
};

const INTO_LOCK: Move = {
  name: "Unlocked, another session password",
  what: "The field fades over `medium`, the lit sign holds a `long`, then fades over `slow` into the rest clock.",
  tracks: intoTheLock(true),
  reduced: intoTheLock(true, true),
  back: "cuts",
  sample: from(intoTheLock(true), () => screen({ scene: "splash", prompt: "busy", typed: 12, lock: true }).element),
};

const STRAIGHT_IN: Move = {
  name: "Unlocked, the same password",
  what: "Into the rest clock as above, then the lock's own unlock from rest at once: the desktop.",
  tracks: straightIn(),
  reduced: straightIn(true),
  back: "cuts",
  sample: from(straightIn(), () => screen({ scene: "splash", prompt: "busy", typed: 12, lock: true }).element),
};

const INTO_LOCK_PLAIN: Move = {
  name: "No encrypted disk, into the lock",
  what: "Once lit, the sign holds a `long` and fades over `slow` into the rest clock; the lock waits.",
  tracks: intoTheLock(false),
  reduced: intoTheLock(false, true),
  back: "cuts",
  sample: from(intoTheLock(false), () => screen({ scene: "splash", lock: true }).element),
};

const GOES_DOWN: Move = {
  name: "Shut down or Restart",
  what: "The Power menu's own: the room goes dark under the lit sign, which goes out a group at a time.",
  tracks: goingDown(SHUTDOWN),
  reduced: goingDownReduced(SHUTDOWN),
  unflickered: goingDownUnflickered(SHUTDOWN),
  back: "cuts",
  sample: POWERED,
};

const RESTARTS: Move = {
  name: "Restart, back to the menu",
  what: "Black on every screen while it is off, a `scene` here; then GRUB draws the menu at once.",
  tracks: restarting(),
  reduced: "same",
  back: "cuts",
  sample: from(restarting(), () => screen({ scene: "menu" }).element),
};

type Shown = "rest" | "lit" | "loading" | "error";

const PROMPTS = { rest: "empty", lit: "typing", loading: "busy", error: "wrong" } as const;

export default page<Shown>({
  level: "shell",
  slug: "boot",
  title: "Boot",
  lede: "From the boot menu to the desktop and back to black, the splash's sign in the exact centre of every screen.",
  code: { unbuilt: "GRUB theme, Plymouth splash: old design" },
  from: ["list", "field", "keycap", "lock"],
  stage: {
    live: () => liveBoot({ encrypted: true, same: true, displays: [MONITOR] }),
    switches: {
      each: [
        { label: "Disk", choices: ["Encrypted", "Not encrypted"] },
        { label: "Session password", choices: ["The passphrase", "Another"] },
        { label: "Screens", choices: ["One", "Two"] },
      ],
      live: ([disk, password, displays]) => liveBoot({ encrypted: disk === 0, same: password === 0, displays: displays === 0 ? [MONITOR] : [LAPTOP, MONITOR] }),
    },
    hint: "Enter boots; type kuba, Enter; a key restarts",
    daylight: doesntApply("the boot screens draw in the tokens' dark palette always."),
  },
  variants: [
    { name: "Boot menu", for: "the sign unlit, the entries under it; nothing animates", sample: at({ scene: "menu" }) },
    { name: "Advanced options", for: "its submenu: each kernel, then its recovery mode", sample: at({ scene: "menu", entries: ADVANCED, countdown: null }) },
    { name: "KubaOS snapshots", for: "grub-btrfs' submenu: a header, then snapshots by date", sample: at({ scene: "menu", entries: SNAPSHOTS, countdown: null }) },
    { name: "UEFI Firmware Settings", for: "the machine leaves for its firmware: black", sample: at({ scene: "black" }) },
    { name: "Clock stopped", for: "after any key: the countdown goes, the hints stay", sample: at({ scene: "menu", countdown: null, at: 2 }) },
    { name: "Entry chosen", for: "the menu gone, the unlit sign where it was", sample: at({ scene: "chosen" }) },
    { name: "Starting, the field there", for: "ク and バ struck, the field already waiting", sample: at({ scene: "splash", lit: 2, prompt: "empty" }) },
    { name: "Starting, nothing encrypted", for: "the same with nothing under the sign", sample: at({ scene: "splash", lit: 2 }) },
    { name: "Lit, no encrypted disk", for: "the sign alone, until the lock", sample: at({ scene: "splash" }) },
    { name: "Caps Lock on", for: "the note says so, its key a keycap", sample: at({ scene: "splash", prompt: "caps", typed: 3 }) },
    { name: "Unlocked, the sign holding", for: "the field gone, the lit sign before the lock", sample: at({ scene: "splash" }) },
    { name: "Into the lock", for: "the lock's rest clock, the splash faded off it", sample: whole(() => screen({ scene: "splash", lock: true }).element) },
    { name: "Power off: the sign lit", for: "the Power menu's room gone dark", sample: whole(() => room({ dark: { sign: 0 } })) },
    { name: "Power off: going out", for: "OS and Kuba out, the rest still burning", sample: whole(() => room({ dark: { sign: 2 } })) },
    { name: "Power off: black", for: "every screen, until it is off or restarts", sample: at({ scene: "black" }) },
    { name: "GRUB shell", for: "C: a text screen, no sign", sample: at({ scene: "shell" }) },
    { name: "Edit the entry", for: "E: a text screen, no sign", sample: at({ scene: "edit" }) },
  ],
  sizes: doesntApply("one size: the splash's, on any screen."),
  groups: [
    {
      title: "Several screens",
      rows: [
        { name: "Two screens, the menu", for: "the menu on the primary; the unlit sign alone on the other", sample: twoMenus },
        { name: "Two screens", for: "the sign in each centre; the field on the primary", sample: twoScreens(false) },
        { name: "Fallback: primary unknown", for: "if the splash can't tell, the field on each", sample: twoScreens(true) },
      ],
    },
    {
      title: "What the real boot must do",
      rows: [
        { name: "A quiet kernel", for: "`quiet splash loglevel=3`: no text between menu and splash", sample: at({ scene: "chosen" }) },
        { name: "The menu's picture kept", for: "no mode set after GRUB: its last frame stays until the splash", sample: at({ scene: "chosen" }) },
        { name: "The menu on the primary", for: "the target: GRUB on the primary alone; firmware often mirrors", sample: twoMenus },
        { name: "One place for the sign", for: "GRUB and the splash centre it alike, to the pixel", sample: at({ scene: "splash", lit: 0 }) },
        { name: "The sign's size per screen", for: "the splash scales it per screen, its @2× on HiDPI", sample: twoScreens(false) },
        { name: "Hand-off to the lock", for: "the splash quits onto the lock's first frame, never black", sample: whole(() => screen({ scene: "splash", lock: true }).element) },
        { name: "Shutdown draws nothing", for: "no splash or text after the Power menu: black, then off", sample: at({ scene: "black" }) },
      ],
    },
  ],
  states: {
    shown: ["rest", "lit", "loading", "error"],
    absent: {
      pressed: "nothing on these screens takes a press",
      disabled: "the field always takes the passphrase",
      selected: "the menu's entry is lit, not picked",
      empty: "an empty field is rest",
    },
    rows: nonEmpty([{ name: "The passphrase", cell: (state: Shown) => screen({ scene: "splash", prompt: PROMPTS[state], typed: 9 }).element }]),
    looks: {
      rest: "empty and lit, the hint under it",
      lit: "typing: a bullet a character, the caret",
      loading: "read-only, an hourglass, no ring, Unlocking…",
      error: "Invalid and empty, Wrong passphrase in `error`",
    },
  },
  focus: doesntApply("the field and the menu's rows light as their own pages say."),
  content: {
    text: [
      { name: "A long menu entry", words: LONG_ENTRY, sample: (words: string) => screen({ scene: "menu", entries: [words, ...ENTRIES.slice(1)] }).element },
      { name: "A long hint", words: HINT, sample: (words: string) => screen({ scene: "splash", prompt: "empty", hint: words }).element },
      { name: "An entry's name", words: fixed("KubaOS Shinjuku"), sample: (words: string) => screen({ scene: "menu", entries: [words] }).element },
    ],
    missingPicture: doesntApply("the sign's pictures ship with GRUB's theme and the splash."),
    counts: [
      { name: "One entry", for: "the first row lit, the rest of the room empty", sample: at({ scene: "menu", entries: ENTRIES.slice(0, 1) }) },
      { name: "Four entries", for: "the room full: today's menu", sample: at({ scene: "menu" }) },
      { name: "Many entries", for: "four in view; the list scrolls with the keyboard", sample: at({ scene: "menu", entries: MANY, at: 5 }) },
    ],
  },
  motion: {
    arrives: [CHOOSES, STARTS, STARTS_PLAIN, RESTARTS],
    leaves: [INTO_LOCK, STRAIGHT_IN, INTO_LOCK_PLAIN, GOES_DOWN],
    between: [HUMS, SHAKES],
  },
  keyboard: {
    keys: [
      { keys: ["↑", "↓"], does: "the menu: the entry above or below, no wrap" },
      { keys: ["Enter"], does: "the menu: boots, opens a submenu, or leaves for the firmware" },
      { keys: ["E", "C"], does: "the menu: edit the entry, or GRUB's shell" },
      { keys: ["Esc"], does: "a submenu, edit or shell: back, its entry lit again" },
      { keys: ["Ctrl+X", "F10"], does: "edit: boots the edited entry" },
      { keys: ["Any key"], does: "the menu: stops the countdown" },
      { keys: ["Typing", "Backspace"], does: "the splash: into the passphrase's field" },
      { keys: ["Enter"], does: "the splash: unlocks the disk" },
    ],
    order: "No Tab stops: the menu's rows move with the arrows, the field takes every key.",
    lands: "The menu, on the default entry; the splash, in the field as it appears.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Boot menu", for: "the sign's centre, the rows, the foot", sample: at({ scene: "menu" }) },
      { name: "Starting, the field there", for: "the field and its note under the sign", sample: at({ scene: "splash", lit: 2, prompt: "empty" }) },
      { name: "Two screens", for: "the sign in each centre", sample: twoScreens(false) },
    ]),
    measures: [
      { part: "Field", token: "size-password" },
      { part: "Field's height", token: "size-control_md" },
      { part: "Field to its note", token: "space-s8" },
      { part: "Note's height", token: "size-keycap" },
      { part: "Menu", token: "size-dialog" },
      { part: "Menu row", token: "size-row" },
      { part: "Rows apart", token: "space-s4" },
      { part: "Countdown to hints", token: "space-s16" },
      { part: "Hints off the foot", token: "space-s48" },
      { part: "GRUB's text box corner", token: "radius-surface" },
    ],
    reflow: doesntApply("every screen centres the sign; see Several screens for two sizes."),
  },
});
