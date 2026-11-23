// Desktop: how windows tile and take the keys, the workspaces and their groups, the bar, and where
// the system monitor's readings turn red. From Hyprland's config (`config/hypr/looks.lua`,
// `input.lua`), `kubaos-launches`, the groups file (`desktop/workspaces/groups.toml`), Quickshell's
// settings (`desktop/quickshell/settings.json`) and the system monitor's (`desktop/sysmon/sysmon.toml`).

import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { groupsEmpty, type Reading, readingsPart } from "./desktop-parts.ts";
import { choice, opens, pick, stops, toggle } from "./kit.ts";

const GAPS = ["None", "4 px", "8 px", "16 px"] as const;
const SPLITS = ["Right or below", "Toward the pointer", "Left or above"] as const;
const STYLES = ["Numbers", "Bars", "Graphs"] as const;
const LAYOUTS = ["Bar", "Floating", "Islands"] as const;
/** The workspaces' marks in the setting's order; the shell's default is Dot and dash. */
const MARKS = ["Numbers", "Dots", "Dashes", "Dot and dash"] as const;
const NUMBERS = 0;
const HOT = Array.from({ length: 7 }, (_, i) => `${70 + i * 5} °C`);
const MEMORY = Array.from({ length: 6 }, (_, i) => `${70 + i * 5} %`);
const DISK = ["80 %", "85 %", "90 %", "95 %", "98 %"];
const DRAW = Array.from({ length: 8 }, (_, i) => `${15 + i * 5} W`);

/** `kubaos-sysmon`'s metrics in its order, CPU and RAM on: `sysmon.toml`'s default. */
const READINGS: readonly Reading[] = [
  { id: "cpu", name: "Processor", on: true },
  { id: "ram", name: "Memory", on: true },
  { id: "temp", name: "Temperature", on: false },
  { id: "gpu", name: "Graphics", on: false },
  { id: "net", name: "Network speed", on: false },
  { id: "power", name: "Battery draw", on: false },
];

/** A screen of the sample's desk, as `hyprctl monitors all` describes it. */
interface Screen {
  readonly id: string;
  readonly name: string;
  readonly line: string;
  /** Pinned in the groups file but not plugged in: listed only while it is pinned. */
  readonly away?: true;
}

const PANEL: Screen = { id: "panel", name: "Laptop panel", line: "Shows its own 1–10 unless it's pinned." };

/** The desk the samples stand at: the laptop and two screens beside it. */
const DESK: readonly Screen[] = [
  PANEL,
  { id: "lg", name: "LG ULTRAGEAR+", line: "At desk position 1 now, on DP-1." },
  { id: "dell", name: "Dell U2724D", line: "At desk position 2 now, on DP-2." },
];

/** Two screens that report one description, a pair without serials: neither can be pinned. */
const TWINS: readonly Screen[] = [
  PANEL,
  { id: "dell-a", name: "Dell U2724D", line: "At desk position 1 now, on DP-1." },
  { id: "dell-b", name: "Dell U2724D", line: "At desk position 2 now, on DP-2." },
];

/** The other desk's screen, pinned to a group from there. */
const AWAY: Screen = { id: "benq", name: "BenQ ZOWIE XL", line: "Not plugged in: its group waits for it.", away: true };

/** Desk positions Hyprland has seen besides the panel: each one a group, 1 and 2. */
const SEEN = 2;

/** The name rows drawn: the groups seen, and the new ones a pin can make. */
const NAMES = 4;

/** Desktop's own values, every one at KubaOS's default; the last few are the sample's desk, not settings. */
interface Desk {
  gaps: number;
  split: number;
  followMouse: boolean;
  resizeByEdges: boolean;
  glass: boolean;
  takeFocus: boolean;
  whereLaunched: boolean;
  marks: number;
  kanji: number;
  onlyCurrent: boolean;
  layout: number;
  pill: boolean;
  readings: readonly Reading[];
  style: number;
  alerts: boolean;
  hotAt: number;
  memoryAt: number;
  diskAt: number;
  drawAt: number;
  /** Each screen's group, by its id; 0 or none: not pinned. */
  pins: Readonly<Record<string, number>>;
  /** Each group's name, from group 1; empty: none. */
  names: readonly string[];
  /** The desk has two screens alike. */
  twins: boolean;
  /** Only the laptop panel has ever been plugged in. */
  alone: boolean;
  /** The groups file has a mistake in it. */
  broken: boolean;
  /** The last reading's switch was refused. */
  refused: boolean;
  /** A reading's row drawn mid-drag, by its place. */
  lifted: number | null;
}

/** KubaOS's defaults: what every reset puts back. */
const DEFAULTS: Desk = {
  gaps: 1,
  split: 0,
  followMouse: true,
  resizeByEdges: true,
  glass: true,
  takeFocus: true,
  whereLaunched: true,
  marks: MARKS.indexOf("Dot and dash"),
  kanji: 0,
  onlyCurrent: false,
  layout: 0,
  pill: true,
  readings: READINGS,
  style: 0,
  alerts: true,
  hotAt: 4,
  memoryAt: 4,
  diskAt: 3,
  drawAt: 3,
  pins: {},
  names: Array.from({ length: NAMES }, () => ""),
  twins: false,
  alone: false,
  broken: false,
  refused: false,
  lifted: null,
};

/**
 * A window's own copy of the defaults. The values are only ever replaced, never changed in place, so
 * a sample's seed, spread over them, is never changed by the window it starts.
 */
function fresh(): Desk {
  return { ...DEFAULTS, readings: READINGS.map((each) => ({ ...each })) };
}

/** Whether the readings are on and in order as KubaOS ships them. */
function readingsAt(readings: readonly Reading[]): boolean {
  return readings.every((each, index) => each.id === READINGS[index]!.id && each.on === READINGS[index]!.on);
}

/** How many groups there are: the positions seen, and any a pin made new. */
function groupCount(desk: Desk): number {
  return Math.max(desk.alone ? 0 : SEEN, ...Object.values(desk.pins));
}

/** A group as a pin offers it: its number, and its name if it has one. */
function groupName(desk: Desk, group: number): string {
  const name = desk.names[group - 1] ?? "";
  return name === "" ? `Group ${group}` : `Group ${group}: ${name}`;
}

/** How many screens are pinned, as the row that opens the groups says it. */
function pinnedWords(count: number): string {
  if (count === 0) return "None pinned";
  return new Intl.PluralRules("en").select(count) === "one" ? `${count} screen pinned` : `${count} screens pinned`;
}

/** What is wrong with group `group`'s name `value`, or null. */
function nameProblem(desk: Desk, group: number, value: string): string | null {
  if (value.trim() === "") return "Needs a letter or a number.";
  if ([...value].length > 16) return "Too long: the bar fits 16 characters.";
  const twin = desk.names.findIndex((each, index) => index !== group - 1 && each !== "" && each.trim() === value.trim());
  return twin === -1 ? null : `Group ${twin + 1} is called that already.`;
}

/** A screen's row: which group it is pinned to, from Not pinned, each group, and a new one. */
function pinRow(desk: Desk, screen: Screen): Setting {
  const why = desk.broken ? "Nothing can be pinned until groups.toml is fixed." : desk.twins && screen.id.startsWith("dell-") ? "Two screens report this name, so neither can be pinned." : undefined;
  const pinned = () => desk.pins[screen.id] ?? 0;
  return {
    id: `pin-${screen.id}`,
    title: screen.name,
    line: screen.line,
    control: {
      kind: "select",
      // Read as the row is drawn: a new group, and a group's name, show the next time it is.
      get options() {
        return ["Not pinned", ...Array.from({ length: groupCount(desk) }, (_, i) => groupName(desk, i + 1)), "A new group"];
      },
      picked: pinned,
      set: (index) => (desk.pins = { ...desk.pins, [screen.id]: index }),
    },
    initial: () => 0,
    drives: true,
    ...(screen.away ? { shows: () => pinned() !== 0 } : {}),
    ...(why ? { disabled: why } : {}),
  };
}

/**
 * The screen showing group `group` now, as `config/hypr/workspaces.lua` places them: one pinned to
 * it, else the screens without a pin, left to right, take the groups no pin claims.
 */
function holderOf(desk: Desk, screens: readonly Screen[], group: number): Screen | null {
  const here = screens.filter((each) => !each.away);
  const pinned = here.find((each) => each !== PANEL && desk.pins[each.id] === group) ?? here.find((each) => each === PANEL && desk.pins[each.id] === group);
  if (pinned) return pinned;
  const claimed = new Set(Object.values(desk.pins).filter((each) => each !== 0));
  let next = 1;
  for (const each of here.filter((screen) => screen !== PANEL && !desk.pins[screen.id])) {
    while (claimed.has(next)) next++;
    if (next === group) return each;
    next++;
  }
  return null;
}

/** Group `group`'s name, shown while the group is there; its line says where it is now. */
function nameRow(desk: Desk, screens: readonly Screen[], group: number): Setting {
  return {
    id: `name-${group}`,
    title: `Group ${group}`,
    get line() {
      const holder = holderOf(desk, screens, group);
      return holder ? `On the ${holder.name} now.` : "No screen holds it: its windows wait on the laptop panel.";
    },
    control: {
      kind: "field",
      value: () => desk.names[group - 1] ?? "",
      placeholder: "No name",
      check: (value) => nameProblem(desk, group, value),
      set: (value) => (desk.names = desk.names.map((each, index) => (index === group - 1 ? value : each))),
    },
    initial: () => "",
    shows: () => groupCount(desk) >= group,
  };
}

/** Desktop, whole. */
export function desktop(values: Values): SettingsSection {
  const desk = own(values, "desktop", fresh);
  const screens = [...(desk.twins ? TWINS : DESK).filter((each) => !desk.alone || each === PANEL), ...(desk.alone ? [] : [AWAY])];
  const pins = screens.map((each) => pinRow(desk, each));
  if (desk.broken)
    pins[0] = { ...pins[0]!, warns: () => ({ title: "groups.toml doesn't read: group 2 lists a screen twice.", body: "Every screen takes its place left to right until it's fixed. Open it from the Command menu." }) };
  // A file that doesn't read names no group, as the Command menu offers no name Items then.
  const names = desk.alone || desk.broken ? [] : Array.from({ length: NAMES }, (_, i) => nameRow(desk, screens, i + 1));
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "computer",
    title: "Desktop",
    accent: "机",
    lede: "Everything in its pane.",
    subs: [
      {
        title: "Windows",
        body: "How they tile, and which one takes the keys.",
        wells: [
          [
            choice("gaps", "Space between windows", "The air between tiled windows, the bar and the screen's edges.", GAPS, () => desk.gaps, (index) => (desk.gaps = index), { initial: () => DEFAULTS.gaps }),
            pick("split", "New windows open", "Where a new window goes, beside the one you're in.", SPLITS, () => desk.split, (index) => (desk.split = index), { initial: () => DEFAULTS.split }),
            toggle("follow-mouse", "Focus follows the pointer", "The window under the pointer takes the keys. Off, a click does.", () => desk.followMouse, (on) => (desk.followMouse = on), { initial: () => DEFAULTS.followMouse }),
            toggle("resize-edges", "Resize by the edges", "Drag a window's edge or corner to resize it.", () => desk.resizeByEdges, (on) => (desk.resizeByEdges = on), { initial: () => DEFAULTS.resizeByEdges }),
          ],
          [
            toggle("glass", "Frosted glass", "Blurs what's behind the bar, its cards, windows and see-through terminals. Off saves power.", () => desk.glass, (on) => (desk.glass = on), { initial: () => DEFAULTS.glass }),
            toggle("take-focus", "Apps may come forward", "An app that asks for you takes the keys, its workspace shown.", () => desk.takeFocus, (on) => (desk.takeFocus = on), { initial: () => DEFAULTS.takeFocus }),
          ],
        ],
      },
      {
        title: "Workspaces",
        body: "Ten to a screen, and where new apps open.",
        wells: [
          [
            opens("groups", "Workspace groups", "Pin a screen, and its workspaces follow it to any desk.", "groups", () => pinnedWords(Object.values(desk.pins).filter((group) => group !== 0).length), { drives: true }),
            toggle("where-launched", "Open where launched", "A slow app still opens on the workspace you started it from.", () => desk.whereLaunched, (on) => (desk.whereLaunched = on), { initial: () => DEFAULTS.whereLaunched }),
            pick("marks", "Workspace marks", "How the bar marks each workspace, and the one you're on.", MARKS, () => desk.marks, (index) => (desk.marks = index), { initial: () => DEFAULTS.marks, drives: true, unbuilt: true }),
            choice("kanji", "Numbers in", "Digits, or kanji numerals.", ["1 2 3", "一 二 三"], () => desk.kanji, (index) => (desk.kanji = index), { initial: () => DEFAULTS.kanji, shows: () => desk.marks === NUMBERS, unbuilt: true }),
            toggle("only-current", "Only the current one", "The other numbers show while the pointer is on them.", () => desk.onlyCurrent, (on) => (desk.onlyCurrent = on), { initial: () => DEFAULTS.onlyCurrent, shows: () => desk.marks === NUMBERS, unbuilt: true }),
          ],
        ],
      },
      {
        title: "Bar",
        body: "What sits along the top, beside the clock.",
        wells: [
          [
            choice("bar-layout", "Layout", "A band along the top, one floating strip, or islands.", LAYOUTS, () => desk.layout, (index) => (desk.layout = index), { initial: () => DEFAULTS.layout, unbuilt: true }),
          ],
          [
            toggle("pill", "System monitor", "Small readings in the bar; a click opens the monitor.", () => desk.pill, (on) => (desk.pill = on), { drives: true, initial: () => DEFAULTS.pill }),
            opens("readings", "Readings", "Which it shows, in what order, and how they're drawn.", "readings", () => `${desk.readings.filter((each) => each.on).length} of ${desk.readings.length}`, { shows: () => desk.pill, drives: true }),
          ],
        ],
      },
      {
        title: "System monitor",
        body: "Where its readings turn red, and when you're told.",
        wells: [
          [
            toggle("alerts", "Alerts", "A notification when the machine runs hot, fills up or drains fast.", () => desk.alerts, (on) => (desk.alerts = on), { initial: () => DEFAULTS.alerts }),
            stops("hot-at", "Hot at", "The processor's temperature that counts as too hot.", "thermostat", HOT, () => desk.hotAt, (index) => (desk.hotAt = index), { initial: () => DEFAULTS.hotAt }),
            stops("memory-at", "Memory full at", "Past this, the system starts dropping what it keeps at hand.", "memory_alt", MEMORY, () => desk.memoryAt, (index) => (desk.memoryAt = index), { initial: () => DEFAULTS.memoryAt }),
            stops("disk-at", "Disk full at", "Any drive this full turns the bar's drive red.", "hard_drive", DISK, () => desk.diskAt, (index) => (desk.diskAt = index), { initial: () => DEFAULTS.diskAt }),
            stops("draw-at", "Draining fast at", "On battery, a draw this high is a runaway, not a build.", "bolt", DRAW, () => desk.drawAt, (index) => (desk.drawAt = index), { initial: () => DEFAULTS.drawAt }),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "groups",
        title: "Workspace groups",
        body: "Each desk position keeps its own ten. A pinned screen brings its group wherever it's plugged in.",
        back: "Workspaces",
        ...(desk.alone ? { before: groupsEmpty } : {}),
        wells: [pins, ...(names.length ? [names] : [])],
      },
      {
        id: "readings",
        title: "Readings",
        body: "What the system monitor shows in the bar, in this order.",
        back: "Bar",
        before: () =>
          readingsPart({
            readings: desk.readings,
            style: desk.style,
            refused: desk.refused,
            lifted: desk.lifted,
            changed: (readings, refused) => {
              desk.readings = readings;
              desk.refused = refused;
            },
          }),
        wells: [
          [
            choice("style", "Drawn as", "Numbers, a small bar each, or the last minute as a graph.", STYLES, () => desk.style, (index) => (desk.style = index), {
              drives: true,
              initial: () => DEFAULTS.style,
              // The readings above hold their own values: this page's reset puts them back too.
              unseen: {
                at: () => readingsAt(desk.readings),
                reset: () => {
                  desk.readings = READINGS.map((each) => ({ ...each }));
                  desk.refused = false;
                },
              },
            }),
          ],
        ],
      },
    ],
  };
}
