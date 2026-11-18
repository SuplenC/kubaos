// Appearance: the mode and how it switches, a wallpaper for each, where the accent comes from, and how
// things move and what points.

import { type ColourPicker, colourPicker } from "../../../../lib/colour-picker.ts";
import { paintPage, playChange } from "../../../../site/change.ts";
import { change, type Mode, settings, SPEEDS } from "../../../../site/settings.ts";
import type { Setting, SettingsSection } from "../settings-parts.ts";
import { D, type Values } from "../settings-values.ts";
import { choice, fact, opens, pick, stops, toggle } from "./kit.ts";

const CURSOR_SIZES = Array.from({ length: 21 }, (_, i) => `${16 + i * 4} px`);
const PRESETS = ["KubaOS neon", "Shōwa", "Heisei"] as const;

/** The seed's picker, one per drawing of its page: its swatches, then its hue track and its tone. */
let seedPicker: ColourPicker | null = null;

/** A row of the seed's page holding part `which` of the picker; the swatches' row draws a new one. */
function seedPart(id: string, title: string, line: string, which: "choice" | "hue" | "tone"): Setting {
  const make = () => {
    if (which === "choice" || !seedPicker) seedPicker = colourPicker({ picked: { mode: "custom", h: 250, l: 0.75 }, swatches: [{ role: "primary", name: "The wallpaper's", colour: "var(--color-primary)" }] });
    return seedPicker[which];
  };
  return { id, title, line, control: { kind: "part", make }, unbuilt: true };
}

/** The window shown in `to`: the dark ⇄ light switch played on it, or at once. */
function flip(values: Values, to: Mode): void {
  const frame = values.window;
  const from = values.drawn;
  values.drawn = to;
  if (!frame || from === to) return;
  const theme = settings().theme;
  frame.dataset.mode = to;
  if (values.switchMove === 2) paintPage([frame], { theme, mode: to });
  else playChange([frame], { theme, mode: from }, { theme, mode: to });
}

/** The mode by the sun, as the sample's evening has it. */
const BY_THE_SUN: Mode = "dark";

/** Appearance, whole. */
export function appearance(values: Values): SettingsSection {
  const speedAt = () => Math.max(0, SPEEDS.indexOf(settings().speed));
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "wallpaper",
    title: "Appearance",
    accent: "外観",
    lede: "Eye candy, zero calories.",
    subs: [
      {
        title: "Mode",
        body: "Night or day, or let the sun decide.",
        wells: [
          [
            choice("mode", "Mode", "Dark rooms, light rooms, or each in its hours.", ["Dark", "Light", "By the sun"], () => values.mode, (index) => {
              values.mode = index;
              flip(values, index === 0 ? "dark" : index === 1 ? "light" : BY_THE_SUN);
            }, { unbuilt: true, drives: true, initial: () => D.mode }),
            fact("sun", "Sunset and sunrise", "Worked out from your time zone, every day.", "16:05 · 07:12", { shows: () => values.mode === 2, unbuilt: true }),
            choice("switch-move", "Switching", "How the screen changes from one mode to the other.", ["Neon", "Wipe", "None"], () => values.switchMove, (index) => (values.switchMove = index), { unbuilt: true, initial: () => D.switchMove }),
          ],
        ],
      },
      {
        title: "Wallpaper",
        body: "One for the night and one for the day.",
        wells: [
          [
            opens("night-wallpaper", "Dark mode's wallpaper", "Its colours give every screen its accent.", "gallery", () => "Shinjuku, rain"),
            opens("day-wallpaper", "Light mode's wallpaper", "Leave it and the night's serves both.", "gallery", () => (values.dayWallpaper === 0 ? "Same as dark" : "Kyoto, noon"), { unbuilt: true }),
            choice("wallpaper-motion", "Moving wallpapers", "A video plays when allowed; otherwise its still shows.", ["Always", "Plugged in", "Never"], () => values.wallpaperMotion, (index) => (values.wallpaperMotion = index), { initial: () => D.wallpaperMotion }),
          ],
        ],
      },
      {
        title: "Colour",
        body: "Where the accent comes from, and how far to trust it.",
        wells: [
          [
            choice("accent-from", "Accent from", "The wallpaper picks it, or you plant a seed it grows from.", ["Wallpaper", "Seed", "Preset"], () => values.accentFrom, (index) => (values.accentFrom = index), { unbuilt: true, drives: true, initial: () => D.accentFrom }),
            opens("seed", "Seed colour", "The one colour every other is grown from.", "seed", () => "Custom", { shows: () => values.accentFrom === 1, unbuilt: true }),
            pick("preset", "Preset", "A finished set of colours, the wallpaper aside.", PRESETS, () => values.preset, (index) => (values.preset = index), { shows: () => values.accentFrom === 2, unbuilt: true, initial: () => D.preset }),
            pick("cold", "Cold accent", "The sign's other tube, used rarely, for what is cool or off.", ["Opposite the accent", "A seed of its own"], () => values.cold, (index) => (values.cold = index), { unbuilt: true, initial: () => D.cold }),
            opens("roles", "Fine-tune colours", "Change one role; the rest keep their contrast.", "roles", () => "None changed", { unbuilt: true }),
          ],
        ],
      },
      {
        title: "Motion & pointer",
        body: "How fast things move, and what you move them with.",
        wells: [
          [
            stops("speed", "Animation speed", "Every move, faster or slower; 1× is as designed.", "motion_photos_on", SPEEDS.map((each) => `${each}×`), speedAt, (index) => change({ speed: SPEEDS[index]! }), { unbuilt: true, initial: () => SPEEDS.indexOf(1) }),
            toggle("reduce", "Reduce motion", "Things fade instead of fly; nothing slides or bounces.", () => settings().reduced, (on) => change({ reduced: on }), { unbuilt: true, initial: () => false }),
            choice("menu-motion", "Command menu motion", "How the menu opens and moves between its views.", ["Full", "Fades"], () => values.menuMotion, (index) => (values.menuMotion = index), { initial: () => D.menuMotion }),
            toggle("flicker", "Neon flicker", "The sign's stutter when something switches on.", () => settings().flicker, (on) => change({ flicker: on }), { initial: () => true }),
          ],
          [
            choice("cursor-style", "Pointer style", "Dark with a light edge, in the accent, or plain.", ["Dark", "Accent", "Classic"], () => values.cursorStyle, (index) => (values.cursorStyle = index), { initial: () => D.cursorStyle }),
            stops("cursor-size", "Pointer size", "How big the pointer is drawn, on every screen.", "mouse", CURSOR_SIZES, () => values.cursorSize, (index) => (values.cursorSize = index), { initial: () => D.cursorSize }),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "seed",
        title: "Seed colour",
        body: "Every colour on the screen grows from this one.",
        back: "Colour",
        wells: [
          [
            seedPart("seed-pick", "Seed", "The wallpaper's own, or one of yours.", "choice"),
            seedPart("seed-hue", "Hue", "Round the colour wheel.", "hue"),
            seedPart("seed-tone", "Tone", "From deep to pale.", "tone"),
          ],
          [fact("seed-contrast", "Contrast", "Kept readable whatever the seed: text never fades out.", "Kept")],
        ],
      },
      {
        id: "roles",
        title: "Fine-tune colours",
        body: "One role at a time; contrast is checked as you go.",
        back: "Colour",
        wells: [
          [
            fact("role-accent", "Accent on the page", "Buttons, links and the lit line.", "7.2 : 1"),
            fact("role-text", "Text on the page", "Everything you read.", "15.8 : 1"),
            fact("role-error", "Errors", "What went wrong, said in this colour.", "6.1 : 1"),
          ],
          [toggle("role-keep", "Keep contrast", "A colour that would read badly is moved until it reads.", () => true, () => {}, { disabled: "Always on: unreadable text helps nobody." })],
        ],
      },
    ],
  };
}
