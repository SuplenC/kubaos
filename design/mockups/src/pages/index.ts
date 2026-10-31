// Every mockup page, by level. A new page is a file under its level's folder and its line here.

import type { AnyPage, Level } from "../site/page.ts";
import badge from "./atoms/badge.ts";
import button from "./atoms/button.ts";
import checkbox from "./atoms/checkbox.ts";
import chip from "./atoms/chip.ts";
import divider from "./atoms/divider.ts";
import field from "./atoms/field.ts";
import figure from "./atoms/figure.ts";
import hint from "./atoms/hint.ts";
import icon from "./atoms/icon.ts";
import iconButton from "./atoms/icon-button.ts";
import keycap from "./atoms/keycap.ts";
import radio from "./atoms/radio.ts";
import skeleton from "./atoms/skeleton.ts";
import spinner from "./atoms/spinner.ts";
import toggle from "./atoms/switch.ts";
import text from "./atoms/text.ts";
import tooltip from "./atoms/tooltip.ts";
import colour from "./foundations/colour.ts";
import elevation from "./foundations/elevation.ts";
import focus from "./foundations/focus.ts";
import icons from "./foundations/icons.ts";
import motion from "./foundations/motion.ts";
import radii from "./foundations/radii.ts";
import space from "./foundations/space.ts";
import type from "./foundations/type.ts";
import menu from "./molecules/menu.ts";
import popover from "./molecules/popover.ts";
import select from "./molecules/select.ts";
import slider from "./molecules/slider.ts";
import tabs from "./molecules/tabs.ts";

export const PAGES: readonly AnyPage[] = [colour, type, space, radii, elevation, focus, motion, icons, text, icon, button, iconButton, hint, toggle, checkbox, radio, chip, field, keycap, tooltip, badge, spinner, divider, skeleton, figure, slider, tabs, select, popover, menu];

export function pagesOf(level: Level): AnyPage[] {
  return PAGES.filter((page) => page.level === level);
}
