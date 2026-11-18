// Notifications: the popups and the bar's bell, Do not disturb by hand and by itself, and each app's
// rule. Do not disturb follows `config/quickshell/services/dnd.js`: on by hand, or by itself while a
// screen is shared or a window is fullscreen, and turned off during one, off until it ends.

import { emptyState } from "../../../../lib/empty-state.ts";
import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { choice, fact, opens, stops, toggle } from "./kit.ts";

/** An app that has notified, newest first, as the shell's history knows it. */
export interface Heard {
  readonly app: string;
  /** When it last did, and how often lately. */
  readonly last: string;
}

/** What Hyprland says now: nothing, a screen shared, or a window fullscreen. */
export const NOTHING = 0;
export const SHARING = 1;
export const FULLSCREEN = 2;

/** Notifications' own values: Do not disturb's state as the shell holds it, and the settings. */
export interface NotificationValues {
  /** Turned on by hand. */
  manual: boolean;
  /** Turned off by hand during a share or a fullscreen window, until it ends. */
  overridden: boolean;
  /** What Hyprland says now: `NOTHING`, `SHARING` or `FULLSCREEN`. */
  now: number;
  whileSharing: boolean;
  whileFullscreen: boolean;
  popupTime: number;
  sounds: number;
  bell: boolean;
  newApps: number;
  heard: readonly Heard[];
  /** Each app's rule by its name, Show where it has none; replaced, never changed in place. */
  rules: Readonly<Record<string, number>>;
}

/** The apps the samples have heard from: those of the notification mockups, and KubaOS's own senders. */
export const HEARD: readonly Heard[] = [
  { app: "KubaOS", last: "Last at 21:48 · 3 today" },
  { app: "Slack", last: "Last at 21:47 · 14 today" },
  { app: "Ghostty", last: "Last at 20:58 · 2 today" },
  { app: "Signal", last: "Last yesterday at 12:08" },
  { app: "Files", last: "Last on Monday · 2 this week" },
  { app: "KDE Connect", last: "Last on Sunday" },
  { app: "Firefox", last: "Last on 3 November" },
];

/** KubaOS's defaults, made anew for each window so no two share a value. */
function fresh(): NotificationValues {
  return {
    manual: false,
    overridden: false,
    now: NOTHING,
    whileSharing: true,
    whileFullscreen: true,
    popupTime: 3,
    sounds: 0,
    bell: false,
    newApps: 0,
    heard: HEARD,
    rules: {},
  };
}

const DEFAULTS = fresh();

/** `Settings.notificationTimeout` is 6000 ms; an app that asks for its own time keeps it. */
const POPUP_TIMES = ["3 s", "4 s", "5 s", "6 s", "8 s", "10 s", "15 s"] as const;
const SOUNDS = ["Off", "Urgent only", "Every one"] as const;
const RULES = ["Show", "Quiet", "Block"] as const;
const SHOW = 0;
const BLOCK = 2;

/** Whether Hyprland's say turns it on by itself now, as the person allows. */
function automatic(mine: NotificationValues): boolean {
  return (mine.now === SHARING && mine.whileSharing) || (mine.now === FULLSCREEN && mine.whileFullscreen);
}

/** Whether Do not disturb is on: by hand, or by itself and not turned off during it. */
function on(mine: NotificationValues): boolean {
  return mine.manual || (automatic(mine) && !mine.overridden);
}

/** The switch, as `dnd.js`'s `toggle`: off during a share or a fullscreen window holds until it ends. */
function flip(mine: NotificationValues, wanted: boolean): void {
  if (on(mine) === wanted) return;
  if (on(mine)) {
    mine.manual = false;
    mine.overridden = automatic(mine);
  } else if (automatic(mine)) mine.overridden = false;
  else mine.manual = true;
}

/** An app's row on the rules page: its rule, and a warning where blocking it hides what matters. */
function ruleOf(mine: NotificationValues, heard: Heard): Setting {
  const rule = () => mine.rules[heard.app] ?? SHOW;
  return choice(`app-${heard.app.toLowerCase().replace(/\W+/g, "-")}`, heard.app, heard.last, RULES, rule, (index) => (mine.rules = { ...mine.rules, [heard.app]: index }), {
    initial: () => SHOW,
    ...(heard.app === "KubaOS"
      ? { warns: () => (rule() === BLOCK ? { title: "KubaOS says here when a drive is ready or an update is waiting.", body: "Urgent ones, a battery about to die, still get through." } : null) }
      : {}),
  });
}

/** What the rules row says: how many apps, and how many aren't simply shown. */
function rulesDetail(mine: NotificationValues): string {
  if (mine.heard.length === 0) return "None yet";
  const held = mine.heard.filter((each) => (mine.rules[each.app] ?? SHOW) !== SHOW).length;
  return held === 0 ? `${mine.heard.length} apps` : `${mine.heard.length} apps, ${held} held`;
}

/** Notifications, whole. */
export function notifications(values: Values): SettingsSection {
  const mine = own(values, "notifications", fresh);
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "notifications",
    title: "Notifications",
    accent: "通知",
    lede: "Ping, but politely.",
    subs: [
      {
        title: "Popups",
        body: "How long they stay, how loudly they come, and the bell.",
        wells: [
          [
            stops("popup-time", "Popups stay for", "Unless the app asks for longer. Pointing at them stops the clock.", "timer", POPUP_TIMES, () => mine.popupTime, (index) => (mine.popupTime = index), {
              unbuilt: true,
              initial: () => DEFAULTS.popupTime,
            }),
            choice("notification-sounds", "Sounds", "A chime as one pops up. Alarms and a dying battery ring anyway.", SOUNDS, () => mine.sounds, (index) => (mine.sounds = index), {
              unbuilt: true,
              initial: () => DEFAULTS.sounds,
            }),
            fact("urgent", "Urgent ones", "", "Till closed", { disabled: "Stays up till closed, through Do not disturb too." }),
          ],
          [
            toggle("bell", "Always show the bell", "Otherwise it shows in the bar only while there's something in the panel.", () => mine.bell, (shown) => (mine.bell = shown), {
              initial: () => DEFAULTS.bell,
            }),
          ],
        ],
      },
      {
        title: "Do not disturb",
        body: "Popups held, by hand or by itself; the panel keeps them.",
        wells: [
          [
            toggle("dnd", "Do not disturb", "Holds every popup back until you turn it off.", () => on(mine), (wanted) => flip(mine, wanted), {
              drives: true,
              // What it would be with no hand on it: on while a share or a fullscreen window turns it on.
              initial: () => automatic(mine),
            }),
            {
              id: "dnd-why",
              title: "On by itself",
              line: "It goes off when that ends. Turn it off to see popups now.",
              control: { kind: "fact", detail: () => (mine.now === SHARING ? "Screen shared" : "Fullscreen window") },
              shows: () => !mine.manual && automatic(mine) && !mine.overridden,
            },
          ],
          [
            toggle("dnd-sharing", "While sharing a screen", "Meet, Discord or OBS taking the screen turns it on until they stop.", () => mine.whileSharing, (wanted) => (mine.whileSharing = wanted), {
              unbuilt: true,
              drives: true,
              initial: () => DEFAULTS.whileSharing,
            }),
            toggle("dnd-fullscreen", "With a window fullscreen", "A video or a game. A maximised window doesn't count.", () => mine.whileFullscreen, (wanted) => (mine.whileFullscreen = wanted), {
              unbuilt: true,
              drives: true,
              initial: () => DEFAULTS.whileFullscreen,
            }),
          ],
        ],
      },
      {
        title: "Apps",
        body: "What each app may do, one at a time.",
        wells: [
          [
            opens("app-rules", "Rules per app", "Show, quiet or block each app that has notified.", "app-rules", () => rulesDetail(mine), { unbuilt: true }),
            choice("new-apps", "New apps start as", "The rule an app gets the first time it notifies.", RULES.slice(0, 2), () => mine.newApps, (index) => (mine.newApps = index), {
              unbuilt: true,
              initial: () => DEFAULTS.newApps,
            }),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "app-rules",
        title: "Rules per app",
        body: "Quiet skips the popup but keeps it in the panel; Block drops it.",
        back: "Apps",
        ...(mine.heard.length === 0
          ? { before: () => emptyState({ icon: "notifications", title: "No app has notified yet", body: "An app joins this list with its first notification, ready for its rule." }) }
          : {}),
        wells: mine.heard.length === 0 ? [] : [mine.heard.map((each) => ruleOf(mine, each))],
      },
    ],
  };
}
