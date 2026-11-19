// What the Settings window holds: its thirteen sections, each in a file of its own under `sections/`,
// assembled here in the sidebar's order.

import type { Outline } from "../../../lib/sidebar-page.ts";
import { asSection, type ResetPins, type SettingsSection } from "./settings-parts.ts";
import type { Values } from "./settings-values.ts";
import { about } from "./sections/about.ts";
import { appearance } from "./sections/appearance.ts";
import { apps } from "./sections/apps.ts";
import { dateTime } from "./sections/date-time.ts";
import { desktop } from "./sections/desktop.ts";
import { displays } from "./sections/displays.ts";
import { keyboard } from "./sections/keyboard.ts";
import { mouse } from "./sections/mouse.ts";
import { network } from "./sections/network.ts";
import { notifications } from "./sections/notifications.ts";
import { phone } from "./sections/phone.ts";
import { power } from "./sections/power.ts";
import { proxy } from "./sections/proxy.ts";
import { TIPS, withTips } from "./sections/tips.ts";

export { appearance } from "./sections/appearance.ts";
export { power } from "./sections/power.ts";
export { defaults, type Values } from "./settings-values.ts";

/** The sections, in the sidebar's order. */
export const SECTIONS = ["About", "Appearance", "Displays", "Desktop", "Keyboard", "Mouse & touchpad", "Power", "Notifications", "Proxy", "Network", "Phone", "Date & time", "Apps"] as const;
export const ABOUT = 0;
export const APPEARANCE = 1;
export const DISPLAYS = 2;
export const DESKTOP = 3;
export const KEYBOARD = 4;
export const MOUSE = 5;
export const POWER = 6;
export const NOTIFICATIONS = 7;
export const PROXY = 8;
export const NETWORK = 9;
export const PHONE = 10;
export const DATE_TIME = 11;
export const APPS = 12;

/** Each section's data, drawn from a window's values, in the sidebar's order. */
const ORDER: readonly ((values: Values) => SettingsSection)[] = [about, appearance, displays, desktop, keyboard, mouse, power, notifications, proxy, network, phone, dateTime, apps].map(
  (section, index) => (values: Values) => withTips(section(values), TIPS[index]),
);

/** The window's outline: Home, then the thirteen sections, each reading `values`. */
export function outline(values: Values, pins?: ResetPins): Outline {
  return {
    home: {
      name: "Home",
      accent: "設定",
      title: "Settings",
      lede: "Everything about this machine, one search away.",
      placeholder: "Search every setting",
    },
    sections: ORDER.map((section) => asSection(section(values), pins)),
  };
}

/** The section `index`'s data, as `outline` draws it, for a sample that lands on one of its rows. */
export function sectionData(values: Values, index: number): SettingsSection {
  return ORDER[index]!(values);
}
