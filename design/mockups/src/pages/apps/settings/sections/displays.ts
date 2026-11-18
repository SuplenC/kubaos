// Displays: the screens as they stand on the desk and each one's mode, scale and workspace group;
// the desks and their groups; the laptop's brightness and the night light.

import type { Setting, SettingsSection, SettingsSub, SubPage } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import {
  arrangement,
  changeShown,
  displayDefaults,
  type DisplaysValues,
  GROUP_COUNT,
  hertz,
  modeOf,
  mirroring,
  numbersOf,
  OTHER_DESKS,
  owned,
  pluggedIn,
  primary,
  SCALES,
  SCREENS,
  shown,
  sizeOf,
  standingAt,
  standings,
  times,
  TURNS,
} from "./displays-parts.ts";
import { choice, fact, opens, pick, stops, toggle, unseen } from "./kit.ts";

const D = displayDefaults();
const BRIGHTNESS = Array.from({ length: 20 }, (_, i) => `${(i + 1) * 5} %`);
const NIGHT = ["Off", "By the sun", "Always"] as const;
const WARMTH = ["Mild", "Warm", "Candle"] as const;

/** The rate screen `index` runs `mode` at by default: its preferred mode's own, else the fastest. */
function preferredRate(index: number, mode: number): number {
  return mode === D.mode[index] ? D.rate[index]! : 0;
}

/** A group's name in a list: "Group 1", or "Group 1: Left" once named. */
function groupWords(v: DisplaysValues, n: number): string {
  const name = v.names[n - 1]?.trim();
  return name ? `Group ${n}: ${name}` : `Group ${n}`;
}

/** A desk's name taken by another desk: names are how the Command menu's "Desk:" Items tell them apart. */
function deskProblem(value: string): string | null {
  const name = value.trim();
  return OTHER_DESKS.some((each) => each.toLowerCase() === name.toLowerCase()) ? `Another desk is called ${name} already.` : null;
}

/** The picked screen's settings: its picture, then where it stands and the workspaces it shows. */
function screenWells(v: DisplaysValues): Setting[][] {
  const index = shown(v);
  const screen = SCREENS[index]!;
  const at = () => shown(v);
  const mode = modeOf(v, index);
  const of = mirroring(v, index);
  const others = pluggedIn(v).filter((each) => each !== index && mirroring(v, each) === null);
  const standing = standings(v).get(index);
  const position = standing?.at === "panel" ? "Laptop panel" : standing?.at === "position" ? `Position ${standing.n}` : "Mirroring";
  const groups = ["Where it stands", ...Array.from({ length: GROUP_COUNT }, (_, n) => groupWords(v, n + 1))];
  return [
    [
      fact("screen", screen.name, numbersOf(v, index), ""),
      pick("resolution", "Resolution", "The screen's own is the sharpest; smaller draws everything bigger.", screen.modes.map(sizeOf), () => v.mode[at()]!, (picked) => changeShown(v, "mode", () => {
        v.mode[at()] = picked;
        v.rate[at()] = preferredRate(at(), picked);
      }), { unbuilt: true, drives: true, initial: () => D.mode[at()]!, unseen: unseen(v.mode, D.mode, at) }),
      pick("refresh", "Refresh rate", "How often it redraws. On battery the laptop's follows Power's stop.", mode.rates.map(hertz), () => Math.min(v.rate[at()]!, mode.rates.length - 1), (picked) => changeShown(v, "rate", () => (v.rate[at()] = picked)), {
        unbuilt: true,
        drives: true,
        initial: () => preferredRate(at(), v.mode[at()]!),
        unseen: unseen(v.rate, D.rate, at),
      }),
      choice("scale", "Scale", "How big everything is drawn on it. The default is Hyprland's pick.", SCALES.map(times), () => v.scale[at()]!, (picked) => changeShown(v, "scale", () => (v.scale[at()] = picked)), {
        unbuilt: true,
        drives: true,
        initial: () => D.scale[at()]!,
        unseen: unseen(v.scale, D.scale, at),
      }),
      pick("orientation", "Orientation", "Turned on its side, the picture turns with it.", TURNS, () => v.turn[at()]!, (picked) => (v.turn[at()] = picked), { unbuilt: true, drives: true, initial: () => D.turn[at()]!, unseen: unseen(v.turn, D.turn, at) }),
      pick("mirror", "Mirror", "Shows another screen's picture instead of a place of its own.", ["Off", ...others.map((each) => SCREENS[each]!.name)], () => (of === null ? 0 : others.indexOf(of) + 1), (picked) => (v.mirror[at()] = picked === 0 ? -1 : others[picked - 1]!), {
        unbuilt: true,
        drives: true,
        initial: () => 0,
        unseen: unseen(v.mirror, D.mirror, at),
      }),
    ],
    [
      fact("desk-position", "Desk position", of === null ? "Where it stands: the laptop's own place, or counted from the left." : `It shows ${SCREENS[of]!.name}'s picture, so it has no place of its own.`, position),
      pick("group", "Workspace group", "Pinned to a group, it shows that group's workspaces on any desk.", groups, () => v.pin[at()]!, (picked) => (v.pin[at()] = picked), {
        drives: true,
        initial: () => D.pin[at()]!,
        unseen: unseen(v.pin, D.pin, at),
        ...(of !== null ? { disabled: `It mirrors ${SCREENS[of]!.name}, so it shows that one's workspaces.` } : {}),
      }),
    ],
  ];
}

/** Where group `n` is now: on the screen at its position, or waiting on the primary screen. */
function whereIs(v: DisplaysValues, n: number): string {
  const at = standingAt(v, n);
  if (at !== null) return `On ${SCREENS[at]!.name}.`;
  return `Waits on ${SCREENS[primary(v)]!.name} while its screen is away.`;
}

/** The workspace groups' names, a field each, saying where each group is now. */
function groupsPage(v: DisplaysValues): SubPage {
  const places = ["Left", "Centre"];
  return {
    id: "groups",
    title: "Workspace groups",
    body: "Each place on the desk keeps its own workspaces 1 to 10. A name shows in the bar and the Command menu.",
    back: "Desks",
    wells: [
      Array.from({ length: GROUP_COUNT }, (_, i): Setting => ({
        id: `group-${i + 1}`,
        title: `Group ${i + 1}`,
        line: whereIs(v, i + 1),
        control: { kind: "field", value: () => v.names[i]!, placeholder: places[i] ?? "", check: () => null, set: (value) => (v.names[i] = value) },
        initial: () => D.names[i]!,
      })),
    ],
  };
}

/** The sub-sections, from what is plugged in and the screen picked now. */
function subsOf(v: DisplaysValues): SettingsSub[] {
  const alone = pluggedIn(v).length === 1 && SCREENS[pluggedIn(v)[0]!]!.panel;
  const panel = pluggedIn(v).some((each) => SCREENS[each]!.panel);
  const named = v.names.filter((name) => name.trim() !== "").length;
  return [
    {
      title: "Screens",
      body: "Drag each where it stands; pick one to set it.",
      before: () => arrangement(v),
      wells: screenWells(v),
    },
    {
      title: "Desks",
      body: "Each set of screens is a desk with its own arrangement.",
      wells: [
        [
          {
            id: "desk-name",
            title: "This desk",
            line: "A name for these screens together, for the Command menu.",
            control: { kind: "field", value: () => v.desk, placeholder: "Desk at home", check: deskProblem, set: (value) => (v.desk = value) },
            initial: () => D.desk,
            unbuilt: true,
            ...(alone ? { disabled: "The laptop alone is no desk: plug a screen in to name one." } : {}),
          },
          toggle("auto-desk", "Switch desks by themselves", "Plug into a desk's screens and its arrangement comes back.", () => v.autoDesk, (on) => (v.autoDesk = on), { unbuilt: true, initial: () => D.autoDesk }),
          opens("groups", "Workspace groups", "Name them; the bar and the Command menu call them that.", "groups", () => (named === 0 ? "None named" : named === 1 ? "One named" : `${named} named`)),
        ],
      ],
    },
    {
      title: "Brightness & night light",
      body: "The laptop's brightness, and how warm the screens get at night.",
      wells: [
        [
          stops("brightness", "Brightness", "The laptop's screen. Others set their own, on their buttons.", "brightness_medium", BRIGHTNESS, () => v.brightness, (index) => (v.brightness = index), {
            ...(panel ? {} : { disabled: "The laptop's screen is off: the others set their own brightness." }),
          }),
          toggle("auto-brightness", "Automatic brightness", "Follows the room's light, on a laptop with a light sensor.", () => v.autoBrightness, (on) => (v.autoBrightness = on), {
            unbuilt: true,
            initial: () => D.autoBrightness,
            ...(panel ? {} : { disabled: "The laptop's screen is off, and only it follows the room." }),
          }),
        ],
        [
          choice("night-light", "Night light", "Warmer colours in the evening, easier on the eyes.", NIGHT, () => v.night, (index) => (v.night = index), { unbuilt: true, drives: true, initial: () => D.night }),
          fact("sun", "Sunset and sunrise", "Worked out from your time zone, every day.", "16:05 · 07:12", { shows: () => v.night === 1, unbuilt: true }),
          choice("warmth", "Warmth", "How far towards candlelight the colours go.", WARMTH, () => v.warmth, (index) => (v.warmth = index), { shows: () => v.night !== 0, unbuilt: true, initial: () => D.warmth }),
        ],
      ],
    },
  ];
}

/** The values already given lists of their own (`owned`). */
const kept = new WeakSet<DisplaysValues>();

/** Displays, whole. */
export function displays(values: Values): SettingsSection {
  const v = own(values, "displays", displayDefaults);
  // A seed's lists are shared by every window drawn from it: each window takes copies of its own.
  if (!kept.has(v)) {
    owned(v);
    kept.add(v);
  }
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "desktop_windows",
    title: "Displays",
    accent: "画面",
    lede: "Pixels, arranged.",
    // A getter: the rows are the picked screen's, and follow what is plugged in.
    get subs() {
      return subsOf(v);
    },
    pages: [groupsPage(v)],
  };
}
