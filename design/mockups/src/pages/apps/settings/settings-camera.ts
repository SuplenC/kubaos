// Settings › Camera: what fixes a camera's picture, beside its live picture. Its own samples: two
// cameras and the one whose backlight compensation is a switch, none plugged in, the camera in use
// by another app, a camera with two controls, an automatic switch off, and the reset held and done;
// and the picture beside the controls or above them, by the window's width.

import { screen } from "../../../site/page.ts";
import { CAMERA } from "./settings-data.ts";
import type { Values } from "./settings-values.ts";
import { sectionPage } from "./settings-section-page.ts";
import { DESKTOP, FULL_HD, type Size, settingsWindow } from "./settings-window.ts";

/** The camera's own values, as a sample starts them, over a camera's defaults. */
const seeded = (camera: object): Partial<Values> => ({ seed: { camera } });

/** Light behind you and Sharpness changed on the built-in camera, so their resets show. */
const TUNED = { values: { integrated: { light: 1, sharpness: 5 } } };

/** The window on Camera with the camera's values `camera`, at `size`. */
const at = (camera: object = TUNED, size?: Size, more: Parameters<typeof settingsWindow>[0] = {}) => screen(() => settingsWindow({ section: CAMERA, values: seeded(camera), ...more }, size));

export default sectionPage({
  from: ["media", "empty-state"],
  index: CAMERA,
  slug: "settings-camera",
  title: "Settings: Camera",
  lede: "What fixes a dark, soft or off-colour picture, beside the live picture.",
  hint: "change a row; the picture follows",
  seed: seeded(TUNED),
  variants: [
    { name: "Two cameras", for: "a Camera select first; no infrared one", sample: at({ ...TUNED, plugged: ["integrated", "c920", "infrared"] }) },
    { name: "Logitech C920 picked", for: "its Light behind you a switch", sample: at({ plugged: ["integrated", "c920", "infrared"], picked: 1, values: { c920: { light: 1 } } }) },
    { name: "No camera connected", for: "it says so; no sections, no Reset", sample: at({ plugged: [] }) },
    { name: "In use by Firefox", for: "the box says who; controls still apply", sample: at({ ...TUNED, busy: "Firefox" }) },
    { name: "In use by another app", for: "the system can't tell which", sample: at({ ...TUNED, busy: null }) },
    { name: "Two controls", for: "a camera offering Sharpness and flicker only", sample: at({ plugged: ["basic"], values: { basic: { sharpness: 5 } } }) },
    { name: "Automatic colour off", for: "Colour temperature set here", sample: at({ values: { integrated: { light: 1, sharpness: 5, autoColour: false, temperature: 26 } } }) },
    { name: "Reset, held halfway", for: "the title's Reset, held to confirm", sample: at(TUNED, undefined, { resets: { at: "section", held: 0.5 } }) },
    { name: "Reset, after", for: "the camera's own defaults; no Reset left", sample: at({}) },
  ],
  sizes: [
    { name: "1400 × 900", for: "the picture beside, 40% of the page", sample: at(TUNED, DESKTOP) },
    { name: "Full HD", for: "the controls at their most, 720", sample: at(TUNED, FULL_HD) },
  ],
  measures: [
    { part: "The picture at its least", token: "size-beside_picture_least" },
    { part: "The picture at its most", token: "size-beside_picture_most" },
    { part: "The controls at their most", token: "size-beside_controls_most" },
    { part: "The controls at their least", token: "size-beside_controls_least" },
    { part: "Controls to picture", token: "space-s24" },
    { part: "The picture's shape", token: "aspect-camera" },
  ],
});
