// Settings › Displays: the screens on the canvas and the picked one's settings, the desks and their
// workspace groups, the laptop's brightness and the night light. Its own samples are the canvas
// with one, two and three screens, one unplugged, one mirrored, and "keep this?" counting down.

import { screen } from "../../../site/page.ts";
import { DISPLAYS } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import type { Values } from "./settings-values.ts";
import { litOn, settingsWindow } from "./settings-window.ts";
import type { DisplaysValues } from "./sections/displays-parts.ts";

/** A window's values with Displays' own started at `own`, over its defaults. */
const seeded = (own: Partial<DisplaysValues>, more: Partial<Values> = {}): Partial<Values> => ({ seed: { displays: own }, ...more });

/** The window on Displays with `own`, scrolled to sub-section `sub`. */
const at = (own: Partial<DisplaysValues>, sub = 0, more: Partial<Values> = {}) => screen(() => settingsWindow({ section: DISPLAYS, sub, values: seeded(own, more) }));

/** The laptop and the Dell, as Hyprland places a set of screens it has never seen: left to right. */
const TWO: Partial<DisplaysValues> = { plugged: [true, true, false], desk: "", x: [1920, 0, 0], y: [0, 0, 0] };

/** The Dell made 2× a moment ago, eleven seconds left to keep it. */
const ASKING: Partial<DisplaysValues> = {
  scale: [3, 2, 0],
  x: [0, 320, 1920],
  y: [0, 1080, 360],
  asking: { screen: 0, what: "scale", back: { mode: [0, 0, 0], rate: [0, 0, 1], scale: [2, 2, 0], x: [0, 320, 2560], y: [0, 1440, 360] }, left: 11 },
};

export default sectionPage({
  from: ["canvas-page", "dialog", "progress-bar"],
  index: DISPLAYS,
  slug: "settings-displays",
  title: "Settings: Displays",
  lede: "The screens: where each stands, how sharp and big it draws, its workspaces, brightness and night light.",
  hint: "drag a screen; pick one; change its scale",
  seed: seeded({ scale: [1, 2, 0], x: [0, 320, 3072], y: [0, 1728, 360], pin: [1, 0, 0], names: ["Left", ""], night: 1 }),
  variants: [
    { name: "One screen", for: "the laptop alone: it fills the canvas; no desk to name", sample: at({ plugged: [false, true, false], picked: 1 }) },
    { name: "Two screens", for: "a set never seen: Hyprland's placing, a new desk", sample: at(TWO) },
    { name: "Three screens, another picked", for: "the LG lit; the rows are its own", sample: at({ picked: 2 }) },
    { name: "A screen unplugged", for: "the LG away: its group waits on the laptop", sample: screen(() => settingsWindow({ section: DISPLAYS, values: seeded({ plugged: [true, true, false] }, { opened: "groups" }) })) },
    { name: "Keep this scale?", for: "the Dell at 2×: going back to 1.5× in 11 s", sample: at(ASKING) },
    { name: "Mirrored", for: "the Dell on the laptop's place; no group of its own", sample: at({ plugged: [true, true, false], mirror: [1, -1, -1] }) },
    { name: "A screen pinned", for: "the LG pinned to group 1: the Dell counts on to 2", sample: screen(() => settingsWindow({ section: DISPLAYS, values: seeded({ picked: 2, pin: [0, 0, 1] }, { opened: "groups" }) })) },
    { name: "The lid closed on a desk", for: "the panel off: no brightness to set", sample: at({ plugged: [true, false, true] }, 2) },
    { name: "A desk name taken", for: "the line says why, in `error`", sample: screen(() => settingsWindow(litOn(DISPLAYS, "desk-name", seeded({ desk: "Desk at work" })))) },
    { name: "Night light by the sun", for: "sunset and sunrise, and the warmth, under it", sample: at({ night: 1 }, 2) },
  ],
  keys: [
    { keys: ["←", "→"], does: "on the canvas: the screen before or after; the rows follow" },
    { keys: ["Shift+←", "Shift+→", "Shift+↑", "Shift+↓"], does: "on the canvas: the screen to its next snap" },
    { keys: ["Enter"], does: "on the canvas: into the screen's rows" },
    { keys: ["Enter", "Esc"], does: "in Keep this?: keep it, or go back now" },
  ],
  measures: [
    { part: "The canvas's height to width", token: "aspect-canvas" },
    { part: "Keep this?", token: "size-dialog" },
  ],
});
