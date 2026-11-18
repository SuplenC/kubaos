// What the Settings window holds: its twelve sections, each in a file of its own under `sections/`,
// assembled here in the sidebar's order.

import type { Outline } from "../../../lib/sidebar-page.ts";
import { asSection, type ResetPins, type SettingsSection } from "./settings-parts.ts";
import type { Values } from "./settings-values.ts";
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
export const SECTIONS = ["Appearance", "Displays", "Desktop", "Keyboard", "Mouse & touchpad", "Power", "Notifications", "Proxy", "Network", "Phone", "Date & time", "Apps & updates"] as const;
export const APPEARANCE = 0;
export const DISPLAYS = 1;
export const DESKTOP = 2;
export const KEYBOARD = 3;
export const MOUSE = 4;
export const POWER = 5;
export const NOTIFICATIONS = 6;
export const PROXY = 7;
export const NETWORK = 8;
export const PHONE = 9;
export const DATE_TIME = 10;
export const APPS = 11;

/** Each section's data, drawn from a window's values, in the sidebar's order. */
const ORDER: readonly ((values: Values) => SettingsSection)[] = [appearance, displays, desktop, keyboard, mouse, power, notifications, proxy, network, phone, dateTime, apps].map(
  (section, index) => (values: Values) => withTips(section(values), TIPS[index]),
);

/** The window's outline: Home, then the twelve sections, each reading `values`. */
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
