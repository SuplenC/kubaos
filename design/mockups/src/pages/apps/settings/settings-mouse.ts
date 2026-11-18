// Settings › Mouse & touchpad: the pointer, the wheel, the touchpad and its swipes, from Hyprland's
// input settings. Its own samples: the swapped buttons' warning, the rows that leave with what they
// hang on, and a machine with no touchpad.

import { screen } from "../../../site/page.ts";
import { MOUSE } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { litOn, settingsWindow } from "./settings-window.ts";

const lit = (id: string, mouse: object = {}) => screen(() => settingsWindow(litOn(MOUSE, id, { seed: { mouse } })));

export default sectionPage({
  from: ["empty-state"],
  index: MOUSE,
  slug: "settings-mouse",
  title: "Settings: Mouse & touchpad",
  lede: "How the pointer moves, scrolls and clicks, with a mouse or a touchpad.",
  hint: "change a row; hold a reset to put it back",
  seed: { seed: { mouse: { speed: 13, accel: 1, padScroll: 2 } } },
  variants: [
    { name: "Primary button on Right", for: "a warning: how to swap back", sample: lit("primary-button", { primary: 1 }) },
    { name: "Tap to click off", for: "Tap and drag leaves with it", sample: lit("tap-click", { tap: false }) },
    { name: "Swiping off", for: "Fingers and Direction leave with it", sample: lit("swipe", { swipe: false }) },
    { name: "No touchpad", for: "it says so; no Gestures", sample: screen(() => settingsWindow({ section: MOUSE, sub: 2, values: { seed: { mouse: { touchpad: false } } } })) },
  ],
});
