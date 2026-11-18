// Settings › Keyboard: the layouts in the control center's well, typing, and every shortcut on a
// page of its own, searched by what it does, with a card that adds one and catches keys already taken.

import { screen } from "../../../site/page.ts";
import { type Capture, type Kb } from "./sections/keyboard-parts.ts";
import { KEYBOARD, type Values } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { litOn, settingsWindow } from "./settings-window.ts";

/** Values with Keyboard's own changed by `kb`. */
const keyed = (kb: Partial<Kb>, more: Partial<Values> = {}): Partial<Values> => ({ ...more, seed: { keyboard: kb } });

/** The window on Keyboard, its own values changed by `kb`. */
const at = (kb: Partial<Kb>, more: Partial<Values> = {}) => screen(() => settingsWindow({ section: KEYBOARD, values: keyed(kb, more) }));

/** The keyboard on setting `id`'s row, its own values changed by `kb`. */
const lit = (id: string, kb: Partial<Kb>) => screen(() => settingsWindow(litOn(KEYBOARD, id, keyed(kb))));

/** The Shortcuts page open, its own values changed by `kb`. */
const shortcuts = (kb: Partial<Kb>) => at(kb, { opened: "shortcuts" });

/** The new shortcut's card open with `capture`, the keyboard in its keys. */
const capturing = (capture: Partial<Capture>) => shortcuts({ capture: { chord: "", held: "", name: "", runs: "", ...capture }, lit: "keys" });

const FOUR = [
  { name: "English (US)", spelling: "us" },
  { name: "German", spelling: "de" },
  { name: "Polish", spelling: "pl" },
  { name: "Japanese", spelling: "jp" },
];

const YAZI = { chord: "Super+Shift+Y", name: "Yazi in Downloads", runs: "kubaos-files open ~/Downloads" };

export default sectionPage({
  from: ["button-bar", "card", "empty-state"],
  index: KEYBOARD,
  slug: "settings-keyboard",
  title: "Settings: Keyboard",
  lede: "Your layouts, how keys repeat and what odd keys do, and every shortcut, searched and added to.",
  hint: "add a layout; open All shortcuts",
  seed: keyed({ delay: 1, rate: 5, caps: 1 }),
  variants: [
    { name: "A layout picked", for: "lit; the toolbar moves or removes it", sample: at({ picked: 1, lit: "layouts" }) },
    { name: "Typing with German", for: "its tick; English (US) stays the default", sample: at({ active: 1, picked: 1 }) },
    { name: "Add a language", for: "the control center's add view, in the same well", sample: at({ adding: true }) },
    { name: "Four layouts", for: "the most XKB holds: + off", sample: at({ layouts: FOUR, picked: 3 }) },
    { name: "One layout", for: "the arrows and × off", sample: at({ layouts: FOUR.slice(0, 1), picked: 0 }) },
    { name: "Compose on Right Alt", for: "German types with it: the warning under the well", sample: lit("compose", { compose: 1 }) },
    { name: "Try it here, lit", for: "the row lit, its small field waiting", sample: lit("try-typing", {}) },
    { name: "Trying the repeat", for: "a held key typed into the field", sample: lit("try-typing", { tryIt: "Hello, wooooooooooorld" }) },
    { name: "Mac-style accents off", for: "their chords leave All shortcuts too", sample: lit("mac-accents", { accents: false }) },
    { name: "Searching shortcuts", for: "the matched letters lit, their groups kept", sample: shortcuts({ query: "workspace", lit: "search" }) },
    { name: "No shortcut matches", for: "the empty state in the list's place", sample: shortcuts({ query: "xyzzy", lit: "search" }) },
    { name: "Capturing keys", for: "the card open, its keys field waiting", sample: capturing({}) },
    { name: "Holding Super and Shift", for: "the keys held show; the chord isn't done", sample: capturing({ held: "Super+Shift" }) },
    { name: "Keys already taken", for: "Super+T: the field red, the note names its shortcut", sample: capturing({ chord: "Super+T", name: "Terminal", runs: "ghostty" }) },
    { name: "Keys that would type", for: "Shift+T alone: refused, the note says why", sample: capturing({ chord: "Shift+T" }) },
    { name: "Free keys, ready", for: "named, with a command: Add lights up", sample: capturing(YAZI) },
    { name: "Your own shortcut", for: "Yours first, its × to remove it", sample: shortcuts({ mine: [{ keys: YAZI.chord, does: YAZI.name }] }) },
    { name: "Not built: adding a shortcut", for: "nothing writes your binds yet", sample: capturing(YAZI) },
  ],
  keys: [
    { keys: ["Esc"], does: "in the keys field, with nothing held: gives up" },
    { keys: ["Backspace"], does: "in the keys field: clears the keys" },
    { keys: ["Enter"], does: "in the name or the command: adds it" },
  ],
  measures: [
    { part: "A layout's row", token: "size-row_two_line" },
    { part: "The search and Add a shortcut", token: "size-control_md" },
    { part: "A keycap", token: "size-keycap" },
    { part: "A shortcut's row", token: "size-row" },
  ],
});
