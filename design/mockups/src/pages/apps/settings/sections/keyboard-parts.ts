// Keyboard's values and its organisms: the layouts' well, a row's keys as keycaps, and the
// Shortcuts page's search, list and new-shortcut card with its keys field. The binds are Hyprland's
// own (`config/hypr/keybinds.lua`, `fallback.lua`, `accents.lua`), by their descriptions.
//
// Its organisms, kept in the Settings app because only it draws them:
// - `layoutsWell`: the control center's Keyboard tab (`keyboardTab`, `addView`) in a Settings
//   sub-section, as tall as what it holds, with no Mac accents chip (Settings has a row for them);
// - `keysAtEnd`: a row's keys as keycaps at the end of the control column;
// - `chordField`: a Field that takes a chord and shows it as keycaps;
// - `shortcutsPage`: the search over every bind, the list grouped as the binds are, and the card
//   that adds one.

import "./keyboard.css";
import { button } from "../../../../lib/button.ts";
import { buttonBar } from "../../../../lib/button-bar.ts";
import { card } from "../../../../lib/card.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { field, labelled, note, setInvalid, setValue } from "../../../../lib/field.ts";
import { keycap, keyHints, keys } from "../../../../lib/keycap.ts";
import { fill, filling, row, stack } from "../../../../lib/layout.ts";
import { list, type ListItem, listLabel, type ListRowSpec } from "../../../../lib/list.ts";
import { matchRuns } from "../../../../lib/sidebar-page.ts";
import { addView, keyboardTab, LAYOUTS, MATCHES } from "../../../shell/control-center-parts.ts";

export interface Layout {
  readonly name: string;
  readonly spelling: string;
}

/** A bind: its keys, as `Super+Shift+T`, and what it does, its description. */
export interface Bind {
  readonly keys: string;
  readonly does: string;
}

/** A new shortcut being made: its keys once pressed, the modifiers held meanwhile, its name and command. */
export interface Capture {
  readonly chord: string;
  readonly held: string;
  readonly name: string;
  readonly runs: string;
}

/** Keyboard's own values in a window. */
export interface Kb {
  /** `layouts` in `keyboard.toml`, the first the default. */
  layouts: readonly Layout[];
  /** The layout typed with now: its row ticked. */
  active: number;
  /** The row the toolbar acts on, lit; -1 for none. */
  picked: number;
  /** The well showing "Add a language". */
  adding: boolean;
  delay: number;
  rate: number;
  tryIt: string;
  /** `mac_accents` in `keyboard.toml`. */
  accents: boolean;
  caps: number;
  compose: number;
  query: string;
  /** The new shortcut's card, open; null while it is shut. */
  capture: Capture | null;
  /** Shortcuts the person added. */
  mine: readonly Bind[];
  /** The layouts' well or a field drawn with the keyboard in it: a sample's. */
  lit: "" | "layouts" | "search" | "keys";
}

export const DELAYS = ["150 ms", "200 ms", "250 ms", "300 ms", "400 ms", "500 ms", "600 ms"] as const;
export const RATES = ["15/s", "20/s", "25/s", "30/s", "40/s", "50/s", "60/s", "80/s"] as const;
export const CAPS = ["Caps Lock", "Escape", "Ctrl", "Esc, or Ctrl held", "Backspace", "Nothing"] as const;
export const COMPOSE = ["Off", "Right Alt", "Right Ctrl", "Menu key"] as const;

/** KubaOS's defaults: `config/hypr/input.lua`'s repeat, `keyboard.toml`'s accents, XKB's own keys. */
export const KB_DEFAULTS: Kb = {
  layouts: LAYOUTS,
  active: 0,
  picked: -1,
  adding: false,
  delay: DELAYS.indexOf("250 ms"),
  rate: RATES.indexOf("40/s"),
  tryIt: "",
  accents: true,
  caps: 0,
  compose: 0,
  query: "",
  capture: null,
  mine: [],
  lit: "",
};

/** The layouts that type characters with Right Alt (AltGr), which a compose key there takes away. */
const ALT_GR = new Set(["de", "pl", "pl(legacy)", "fr", "it", "es", "cz", "us(intl)", "us(altgr-intl)"]);

/** The first of `layouts` that needs Right Alt, if any. */
export function needsAltGr(layouts: readonly Layout[]): Layout | undefined {
  return layouts.find((each) => ALT_GR.has(each.spelling));
}

// The binds.

const DIRECTIONS = [
  { key: "H", arrow: "←", name: "left" },
  { key: "J", arrow: "↓", name: "down" },
  { key: "K", arrow: "↑", name: "up" },
  { key: "L", arrow: "→", name: "right" },
] as const;

/** The number keys, bound by keycode, as the US layout prints them. */
const NUMBERS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] as const;

const b = (keys: string, does: string): Bind => ({ keys, does });

/** Every bind KubaOS ships, grouped as `keybinds.lua` groups them; the mouse's left out. */
const GROUPS: readonly { readonly name: string; readonly binds: readonly Bind[] }[] = [
  {
    name: "Apps",
    binds: [
      b("Super+Return", "Terminal (Ghostty, foot if missing)"),
      b("Super+Ctrl+Return", "Terminal in tmux"),
      b("Super+Alt+Return", "Fallback terminal (foot)"),
      b("Super+Space", "Command menu (Kotodama, fuzzel if the shell is down)"),
      b("Super+Alt+Space", "Fallback launcher (fuzzel)"),
      b("Super+Shift+Return", "Browser"),
      b("Super+Shift+B", "Browser"),
      b("Super+Alt+Shift+B", "Browser (incognito)"),
      b("Super+Shift+F", "File manager (Files)"),
      b("Super+Ctrl+F", "File manager (yazi)"),
      b("Super+Shift+N", "Editor (nvim)"),
      b("Super+Shift+D", "Docker (lazydocker)"),
      b("Super+Ctrl+T", "Activity (btop)"),
      b("Super+F1", "Keyboard shortcuts"),
    ],
  },
  {
    name: "Windows",
    binds: [
      b("Super+Q", "Close window"),
      b("Ctrl+Alt+Delete", "Close all windows"),
      b("Super+E", "Toggle split direction"),
      b("Super+P", "Pseudo-tile window"),
      b("Super+T", "Toggle floating"),
      b("Super+F", "Fullscreen"),
      b("Super+Alt+F", "Maximize (keep gaps and bar)"),
      b("Super+O", "Pop window out (float and pin)"),
      ...DIRECTIONS.flatMap((d) => [
        b(`Super+${d.key}`, `Focus window ${d.name}`),
        b(`Super+Shift+${d.key}`, `Move window ${d.name}`),
        b(`Super+Ctrl+${d.key}`, `Resize window ${d.name}`),
        b(`Super+Shift+${d.arrow}`, `Swap window ${d.name}`),
        b(`Super+Alt+${d.arrow}`, `Move window into group ${d.name}`),
      ]),
      b("Super+-", "Shrink width"),
      b("Super+=", "Grow width"),
      b("Super+Shift+-", "Shrink height"),
      b("Super+Shift+=", "Grow height"),
      b("Alt+Tab", "Next window"),
      b("Alt+Shift+Tab", "Previous window"),
      b("Super+G", "Toggle group"),
      b("Super+Alt+G", "Move window out of group"),
      b("Super+Alt+Tab", "Next window in group"),
      b("Super+Alt+Shift+Tab", "Previous window in group"),
      ...NUMBERS.slice(0, 5).map((key) => b(`Super+Alt+${key}`, `Group window ${key}`)),
    ],
  },
  {
    name: "Screens and workspaces",
    binds: [
      ...DIRECTIONS.map((d) => b(`Super+${d.arrow}`, `Focus screen ${d.name}`)),
      b("Ctrl+Alt+Tab", "Next monitor"),
      b("Ctrl+Alt+Shift+Tab", "Previous monitor"),
      ...DIRECTIONS.flatMap((d) => [b(`Super+Alt+Shift+${d.key}`, `Move workspace to screen ${d.name}`), b(`Super+Alt+Shift+${d.arrow}`, `Move workspace to screen ${d.name}`)]),
      ...NUMBERS.map((key, index) => b(`Super+${key}`, `Go to workspace ${index + 1}`)),
      ...NUMBERS.map((key, index) => b(`Super+Shift+${key}`, `Move window to workspace ${index + 1}`)),
      ...NUMBERS.map((key, index) => b(`Super+Alt+Shift+${key}`, `Move window silently to workspace ${index + 1}`)),
      b("Super+Ctrl+Shift+→", "Move window to a new workspace"),
      b("Super+Tab", "Next workspace"),
      b("Super+Shift+Tab", "Previous workspace"),
      b("Super+Ctrl+→", "Next workspace"),
      b("Super+Ctrl+←", "Previous workspace"),
      b("Super+Ctrl+Alt+→", "Next workspace group"),
      b("Super+Ctrl+Alt+←", "Previous workspace group"),
      b("Super+Ctrl+Tab", "Former workspace"),
      b("Super+S", "Toggle scratchpad"),
      b("Super+`", "Toggle scratchpad"),
      b("Super+Alt+S", "Move window to scratchpad"),
      b("Super+Shift+`", "Move window to scratchpad"),
    ],
  },
  {
    name: "Shell",
    binds: [
      b("Super+Ctrl+A", "Control center (audio)"),
      b("Super+Ctrl+B", "Control center (Bluetooth)"),
      b("Super+Ctrl+W", "Control center (Wi-Fi)"),
      b("Super+N", "Notifications"),
      b("Super+Ctrl+,", "Do not disturb"),
      b("Super+Shift+,", "Clear notifications"),
      b("Super+Ctrl+Space", "Wallpaper picker"),
      b("Super+Ctrl+Shift+Space", "Next wallpaper"),
      b("Super+Ctrl+Alt+Space", "Switch wallpaper folder"),
      b("Super+Ctrl+I", "Keep awake on/off"),
      b("Super+Ctrl+=", "Add 1 to the counter used last"),
      b("Super+Shift+Backspace", "Toggle gaps"),
      b("Super+Ctrl+Z", "Zoom in"),
      b("Super+Ctrl+Alt+Z", "Reset zoom"),
    ],
  },
  {
    name: "Typing",
    binds: [
      b("Super+C", "Copy"),
      b("Super+V", "Paste"),
      b("Super+X", "Cut"),
      b("Super+A", "Select all"),
      b("Super+Shift+Space", "Next keyboard layout"),
    ],
  },
  {
    name: "Screenshots and recording",
    binds: [
      b("Print", "Screenshot region"),
      b("Super+Shift+S", "Screenshot region"),
      b("Shift+Print", "Screenshot monitor"),
      b("Alt+Print", "Screenshot window"),
      b("Super+Shift+A", "Screenshot region and annotate (satty)"),
      b("Super+Print", "Color picker"),
      b("Super+Shift+R", "Record region (again to stop)"),
      b("Super+Alt+R", "Record monitor (again to stop)"),
      b("Super+Alt+Shift+R", "Record monitor with sound (again to stop)"),
      b("Super+Ctrl+R", "Replay buffer on/off (last 30 s)"),
      b("Super+Shift+P", "Save the last 30 s (replay)"),
    ],
  },
  {
    name: "Sound, light and media",
    binds: [
      b("Volume Up", "Volume up"),
      b("Volume Down", "Volume down"),
      b("Alt+Volume Up", "Volume up 1%"),
      b("Alt+Volume Down", "Volume down 1%"),
      b("Mute", "Mute"),
      b("Mic Mute", "Mute microphone"),
      b("Brightness Up", "Brightness up"),
      b("Brightness Down", "Brightness down"),
      b("Alt+Brightness Up", "Brightness up 1%"),
      b("Alt+Brightness Down", "Brightness down 1%"),
      b("Shift+Brightness Up", "Brightness max"),
      b("Shift+Brightness Down", "Brightness min"),
      b("Kbd Light Up", "Keyboard light up"),
      b("Kbd Light Down", "Keyboard light down"),
      b("Play", "Play/pause"),
      b("Pause", "Play/pause"),
      b("Next", "Next track"),
      b("Previous", "Previous track"),
      b("Stop", "Stop"),
    ],
  },
  {
    name: "Session",
    binds: [
      b("Super+Esc", "Power menu"),
      b("Power", "Power menu"),
      b("Super+Ctrl+Esc", "Lock"),
      b("Super+Ctrl+Shift+L", "Lock with hyprlock (fallback lock)"),
      b("Super+Ctrl+Alt+R", "Restart Quickshell"),
      b("Super+Ctrl+Shift+R", "Reload Hyprland config"),
      b("Super+Ctrl+Shift+Esc", "Exit Hyprland"),
    ],
  },
];

/** The Mac accents' cheat sheet (`accents.lua`): listed while the accents are on. */
const ACCENTS: readonly Bind[] = [
  b("Left Alt+E", "Mac accent, then a letter: é (acute)"),
  b("Left Alt+`", "Mac accent, then a letter: à (grave)"),
  b("Left Alt+U", "Mac accent, then a letter: ö (diaeresis)"),
  b("Left Alt+I", "Mac accent, then a letter: ô (circumflex)"),
  b("Left Alt+N", "Mac accent, then a letter: ñ (tilde)"),
];

/** The groups the list shows for `kb`: the person's own first, then KubaOS's, then the accents while on. */
function groupsOf(kb: Kb): { readonly name: string; readonly binds: readonly Bind[]; readonly yours?: true }[] {
  return [
    ...(kb.mine.length > 0 ? [{ name: "Yours", binds: kb.mine, yours: true as const }] : []),
    ...GROUPS,
    ...(kb.accents ? [{ name: "Mac accents", binds: ACCENTS }] : []),
  ];
}

/** How many shortcuts the list holds. */
export function countOf(kb: Kb): number {
  return groupsOf(kb).reduce((sum, group) => sum + group.binds.length, 0);
}

// Keys.

/** The modifiers in the one order every chord is written in. */
const MODIFIERS = ["Super", "Ctrl", "Alt", "Shift"] as const;

/** The modifiers a key event holds, in order. */
function heldBy(event: KeyboardEvent): string[] {
  const on = { Super: event.metaKey, Ctrl: event.ctrlKey, Alt: event.altKey, Shift: event.shiftKey };
  return MODIFIERS.filter((each) => on[each]);
}

/** A key event's own keys that are modifiers, which start a chord rather than end it. */
const MODIFIER_KEYS = new Set(["Meta", "OS", "Super", "Hyper", "Control", "Alt", "AltGraph", "Shift"]);

/** Keys by their place on the board, as the binds write them, so a layout doesn't rename them. */
const BY_CODE: Readonly<Record<string, string>> = {
  Minus: "-",
  Equal: "=",
  Backquote: "`",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Semicolon: ";",
  Quote: "'",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Enter: "Return",
  NumpadEnter: "Return",
  Space: "Space",
  Escape: "Esc",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  Home: "Home",
  End: "End",
  PageUp: "Page Up",
  PageDown: "Page Down",
  PrintScreen: "Print",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  ArrowDown: "↓",
};

/** The key an event ends a chord with, as the binds write it. */
function keyOf(event: KeyboardEvent): string {
  const code = event.code;
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit\d$/.test(code)) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  return BY_CODE[code] ?? event.key;
}

/** Keys that type: bound with no Super, Ctrl or Alt, typing them would run the shortcut instead. */
function types(chord: string): boolean {
  const parts = keys(chord);
  const last = parts.at(-1) ?? "";
  const quiet = !parts.some((each) => each === "Super" || each === "Ctrl" || each === "Alt");
  return quiet && (last.length === 1 || last === "Space" || last === "Return" || last === "Tab" || last === "Backspace");
}

/** What is wrong with `chord` as a new shortcut's keys, said plainly, or null. */
export function chordProblem(kb: Kb, chord: string): string | null {
  if (chord === "") return null;
  const owner = groupsOf(kb)
    .flatMap((group) => group.binds)
    .find((each) => each.keys === chord);
  if (owner) return `“${owner.does}” already has these keys.`;
  if (types(chord)) return "Hold Super, Ctrl or Alt with it, or typing it would run it.";
  return null;
}

/** `combination`'s keycaps at the end of a row's control column. */
export function keysAtEnd(combination: string): HTMLElement {
  return row("s4", "centre", fill(), ...keys(combination).map(keycap));
}

// The layouts.

/**
 * The control center's layouts well, or its "Add a language" view, in a sub-section.
 * `changed` is told after the person changes the layouts, so the section shows them again.
 */
export function layoutsWell(kb: Kb, changed: () => void): HTMLElement {
  const room = document.createElement("span");
  room.className = "kb-layouts";
  if (kb.adding) {
    const view = addView();
    view.querySelector<HTMLElement>("[aria-label='Back']")?.addEventListener("click", () => {
      kb.adding = false;
      changed();
    });
    view.querySelectorAll<HTMLElement>(".k-list-row").forEach((match, index) =>
      match.addEventListener("click", () => {
        const picked = MATCHES[index]!;
        if (!kb.layouts.some((each) => each.spelling === picked.subtitle)) kb.layouts = [...kb.layouts, { name: picked.title, spelling: picked.subtitle ?? "" }];
        kb.adding = false;
        changed();
      }),
    );
    room.append(view);
    return room;
  }
  const well = keyboardTab({
    layouts: kb.layouts,
    active: kb.active,
    selected: kb.picked,
    onTake: (index) => {
      kb.active = index;
      kb.picked = index;
      changed();
    },
  });
  // Settings turns the accents on its own row; the tab's chip would be a second switch for them.
  well.querySelector(".k-chip")?.remove();
  // The tab draws a picked row with the keyboard in the well. A press picks it quietly; the glow
  // is the keyboard's alone.
  if (kb.lit !== "layouts") delete well.dataset.pinned;
  const on = (label: string, act: () => void) =>
    well.querySelector<HTMLButtonElement>(`button[aria-label='${label}']`)?.addEventListener("click", (event) => {
      event.stopPropagation();
      act();
      changed();
    });
  on("Move up", () => moveBy(kb, -1));
  on("Move down", () => moveBy(kb, 1));
  on("Add a language", () => (kb.adding = true));
  on("Remove", () => removePicked(kb));
  room.append(well);
  return room;
}

/** The picked layout moved one place up or down, the tick staying on the layout typed with. */
function moveBy(kb: Kb, by: -1 | 1): void {
  const from = kb.picked;
  const to = from + by;
  if (from < 0 || to < 0 || to >= kb.layouts.length) return;
  const next = [...kb.layouts];
  [next[from], next[to]] = [next[to]!, next[from]!];
  kb.layouts = next;
  if (kb.active === from) kb.active = to;
  else if (kb.active === to) kb.active = from;
  kb.picked = to;
}

/** The picked layout taken out; the last one stays. Typing with it, the default takes over. */
function removePicked(kb: Kb): void {
  const at = kb.picked;
  if (at < 0 || kb.layouts.length < 2) return;
  kb.layouts = kb.layouts.filter((_, index) => index !== at);
  if (kb.active === at) kb.active = 0;
  else if (kb.active > at) kb.active--;
  kb.picked = Math.min(at, kb.layouts.length - 1);
}

// The Shortcuts page.

/** Typing in a field stays there: the page's rows don't take its letters or arrows. */
function keepKeys(element: HTMLElement): void {
  element.addEventListener("keydown", (event) => {
    if (event.key !== "Tab") event.stopPropagation();
  });
}

/** A chord field and what shows a capture in it. */
interface ChordField {
  readonly element: HTMLElement;
  readonly field: HTMLElement;
  readonly show: (capture: Capture, invalid: boolean) => void;
}

/**
 * A Field that takes a chord. With the keyboard in it, the keys held show as keycaps
 * as they go down, and the chord stays once a key that isn't a modifier ends it, until a modifier
 * goes down again. Tab leaves it, Esc gives up, Backspace clears it; with a modifier held, each is
 * a key of the chord like any other.
 */
function chordField(capture: Capture, pinned: boolean, on: { readonly chord: (chord: string) => void; readonly held: (held: string, pressed: boolean) => void; readonly cancel: () => void }): ChordField {
  const made = field({ icon: "keyboard", placeholder: "Press the keys", ...(pinned ? { pinned: "lit" as const } : {}) });
  made.classList.add("kb-chord");
  const room = document.createElement("span");
  room.className = "kb-chord-room";
  room.append(made);
  // The field's own keys and presses edit text; here they are the chord's, so they stop before it.
  room.addEventListener(
    "keydown",
    (event) => {
      const held = heldBy(event);
      const plain = !event.metaKey && !event.ctrlKey && !event.altKey;
      if (plain && event.key === "Tab") return;
      event.preventDefault();
      event.stopPropagation();
      if (MODIFIER_KEYS.has(event.key)) on.held(held.join("+"), true);
      else if (plain && !event.shiftKey && event.key === "Escape") on.cancel();
      else if (plain && !event.shiftKey && event.key === "Backspace") on.chord("");
      else on.chord([...held, keyOf(event)].join("+"));
    },
    true,
  );
  room.addEventListener(
    "keyup",
    (event) => {
      event.stopPropagation();
      if (MODIFIER_KEYS.has(event.key)) on.held(heldBy(event).join("+"), false);
    },
    true,
  );
  room.addEventListener(
    "mousedown",
    (event) => {
      event.preventDefault();
      event.stopPropagation();
      made.focus({ preventScroll: true });
    },
    true,
  );
  const show = (now: Capture, invalid: boolean) => {
    const shown = now.chord || now.held;
    const value = made.querySelector<HTMLElement>(".k-field-room");
    if (shown && value) {
      const caps = row("s4", "centre", ...keys(shown).map(keycap));
      caps.classList.add("kb-chord-keys");
      value.replaceChildren(caps);
    } else setValue(made, "");
    setInvalid(made, invalid);
  };
  show(capture, false);
  return { element: room, field: made, show };
}

/** Whether the new shortcut can be added: free keys, a name and a command. */
function ready(kb: Kb, capture: Capture): boolean {
  return capture.chord !== "" && chordProblem(kb, capture.chord) === null && capture.name.trim() !== "" && capture.runs.trim() !== "";
}

/**
 * The card that adds a shortcut: its keys pressed into a chord field, with what is
 * wrong with them under it, its name and its command, then Cancel and Add.
 */
function newShortcut(kb: Kb, done: () => void): { readonly element: HTMLElement; readonly land: () => void } {
  const capture = () => kb.capture ?? { chord: "", held: "", name: "", runs: "" };
  const put = (change: Partial<Capture>) => {
    kb.capture = { ...capture(), ...change };
    refresh();
  };
  const chord = chordField(capture(), kb.lit === "keys", {
    chord: (keys) => put({ chord: keys, held: "" }),
    // A modifier going down starts the chord again; one let go after a chord leaves it be.
    held: (held, pressed) => {
      if (pressed || capture().chord === "") put({ chord: "", held });
    },
    cancel: () => {
      kb.capture = null;
      done();
    },
  });
  const said = document.createElement("span");
  said.className = "kb-chord-note";
  const name = field({ placeholder: "As the menu and this list show it", ...(capture().name ? { value: capture().name } : {}), onInput: (value) => put({ name: value }) });
  const runs = field({ icon: "terminal", placeholder: "A command, as in a terminal", ...(capture().runs ? { value: capture().runs } : {}), onInput: (value) => put({ runs: value }) });
  const bar = buttonBar({ dismiss: "Cancel", actions: [{ label: "Add" }], size: "Sm" });
  const add = bar.actions[0]!;
  bar.dismiss?.addEventListener("click", () => {
    kb.capture = null;
    done();
  });
  const take = () => {
    const now = capture();
    if (!ready(kb, now)) return;
    kb.mine = [...kb.mine, { keys: now.chord, does: now.name.trim() }];
    kb.capture = null;
    done();
  };
  add.addEventListener("click", take);
  for (const each of [name, runs]) {
    keepKeys(each);
    each.addEventListener("keydown", (event) => {
      if (event.key === "Enter") take();
      else if (event.key === "Escape") {
        kb.capture = null;
        done();
      }
    });
  }

  function refresh(): void {
    const now = capture();
    const problem = chordProblem(kb, now.chord);
    chord.show(now, problem !== null);
    said.replaceChildren(problem ? note(problem, true) : keyHints([["Esc", "gives up"], ["Backspace", "clears the keys"]]));
    add.disabled = !ready(kb, now);
  }
  refresh();

  const element = card({
    header: { glyph: "keyboard", title: "A new shortcut", subtitle: "Press its keys, name it, and say what it runs." },
    body: [stack("s8", "start", filling(labelled("Keys", chord.element)), said), labelled("Name", name), labelled("Runs", runs)],
    actions: bar.element,
  });
  element.classList.add("kb-new");
  return { element, land: () => chord.field.focus({ preventScroll: true }) };
}

/** The list for `kb`'s search: every group with a match, its matched letters lit; or the empty state. */
function bindList(kb: Kb, changed: () => void): HTMLElement {
  const query = kb.query.trim();
  const items: ListItem[] = [];
  for (const group of groupsOf(kb)) {
    const rows: ListRowSpec[] = [];
    for (const each of group.binds) {
      const runs = query === "" ? [] : matchRuns(each.does, query);
      if (!runs) continue;
      const forget = () => {
        kb.mine = kb.mine.filter((mine) => mine !== each);
        changed();
      };
      rows.push({ title: each.does, keys: each.keys, ...(runs.length > 0 ? { matched: runs } : {}), ...(group.yours ? { forget } : {}) });
    }
    if (rows.length > 0) items.push(listLabel(group.name, items.length > 0), ...rows);
  }
  if (items.length === 0) return emptyState({ icon: "search_off", title: "No match", body: `No shortcut matches “${query}”.` });
  return list(items);
}

/**
 * The Shortcuts page: a search over every bind and Add a shortcut on one line, the new
 * shortcut's card under them while it is open, then the binds grouped, the person's own first.
 */
export function shortcutsPage(kb: Kb): HTMLElement {
  const root = stack("s16", "start");
  root.classList.add("kb-shortcuts");
  const search = field({
    icon: "search",
    placeholder: "Search shortcuts",
    ...(kb.query ? { value: kb.query } : {}),
    ...(kb.lit === "search" ? { pinned: "lit" as const } : {}),
    onInput: (value) => {
      kb.query = value;
      drawList();
    },
  });
  keepKeys(search);
  const add = button({ label: "Add a shortcut", icon: "add", kind: "Tonal", size: "Md" });
  add.addEventListener("click", () => {
    kb.capture = { chord: "", held: "", name: "", runs: "" };
    drawCard()?.land();
  });
  const cardRoom = document.createElement("span");
  const listRoom = document.createElement("span");
  root.append(filling(row("s8", "centre", filling(search), add)), filling(cardRoom), filling(listRoom));

  function drawCard(): ReturnType<typeof newShortcut> | null {
    add.disabled = kb.capture !== null;
    cardRoom.hidden = kb.capture === null;
    if (!kb.capture) {
      cardRoom.replaceChildren();
      return null;
    }
    const made = newShortcut(kb, () => {
      drawCard();
      drawList();
      add.focus({ preventScroll: true });
    });
    cardRoom.replaceChildren(made.element);
    return made;
  }

  function drawList(): void {
    listRoom.replaceChildren(bindList(kb, drawList));
  }

  drawCard();
  drawList();
  return root;
}
