// Every mockup page, by level. A new page is a file under its level's folder and its line here.

import type { AnyPage, Level } from "../site/page.ts";
import button from "./atoms/button.ts";
import checkbox from "./atoms/checkbox.ts";
import chip from "./atoms/chip.ts";
import field from "./atoms/field.ts";
import hint from "./atoms/hint.ts";
import iconButton from "./atoms/icon-button.ts";
import radio from "./atoms/radio.ts";
import toggle from "./atoms/switch.ts";
import colour from "./foundations/colour.ts";
import elevation from "./foundations/elevation.ts";
import focus from "./foundations/focus.ts";
import icons from "./foundations/icons.ts";
import motion from "./foundations/motion.ts";
import radii from "./foundations/radii.ts";
import space from "./foundations/space.ts";
import type from "./foundations/type.ts";

export const PAGES: readonly AnyPage[] = [colour, type, space, radii, elevation, focus, motion, icons, button, iconButton, hint, toggle, checkbox, radio, chip, field];

export function pagesOf(level: Level): AnyPage[] {
  return PAGES.filter((page) => page.level === level);
}
