// Settings › Notifications: the popups and the bell, Do not disturb by hand and by itself, and each
// app's rule on a page of its own.

import { screen } from "../../../site/page.ts";
import { FULLSCREEN, HEARD, SHARING } from "./sections/notifications.ts";
import { NOTIFICATIONS } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { litOn, settingsWindow } from "./settings-window.ts";

/** The window with Notifications' own values started at `own`, over its defaults. */
const seeded = (own: object, more: object = {}) => ({ seed: { notifications: own }, ...more });

const lit = (id: string, own: object) => screen(() => settingsWindow(litOn(NOTIFICATIONS, id, seeded(own))));
const rules = (own: object) => screen(() => settingsWindow({ section: NOTIFICATIONS, values: seeded(own, { opened: "app-rules" }) }));

export default sectionPage({
  from: ["empty-state"],
  index: NOTIFICATIONS,
  slug: "settings-notifications",
  title: "Settings: Notifications",
  lede: "What may interrupt, how loudly, and when nothing should, app by app too.",
  hint: "flip Do not disturb; set an app's rule",
  seed: seeded({ popupTime: 4, sounds: 1, rules: { Slack: 1 } }),
  variants: [
    { name: "On by itself, sharing", for: "a screen shared: the fact under it says why", sample: lit("dnd-why", { now: SHARING }) },
    { name: "On by itself, fullscreen", for: "a fullscreen window: the same fact, its reason", sample: lit("dnd-why", { now: FULLSCREEN }) },
    { name: "Turned off during a share", for: "off till it ends; its reset turns it back on", sample: lit("dnd", { now: SHARING, overridden: true }) },
    { name: "On by hand", for: "no fact: it stays till turned off", sample: lit("dnd", { manual: true }) },
    { name: "A quiet app", for: "Slack's popups held; the rules row counts it", sample: lit("app-rules", { rules: { Slack: 1 } }) },
    { name: "Blocking KubaOS", for: "the warning under its well: urgent ones still come", sample: rules({ heard: HEARD.slice(0, 4), rules: { KubaOS: 2 } }) },
    { name: "No app yet", for: "the empty list says how it fills", sample: rules({ heard: [] }) },
  ],
});
