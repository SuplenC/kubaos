// Settings' outline, which the template pages draw SidebarPage and CanvasPage with: Home's words
// and the sections, each with its sub-sections and their rows. The words are stand-ins for the app's,
// at the length the toolbar asks for.

import { shown, type Words } from "../../lib/content.ts";
import type { ListRowSpec } from "../../lib/list.ts";
import type { Outline, Section, SubSection } from "../../lib/sidebar-page.ts";

const words = (short: string, long: string): Words => ({ short, long });

/** A setting's row: its name, a line on what it does, and its value or switch. */
function setting(title: Words, subtitle: Words, value?: string | boolean): ListRowSpec {
  return {
    title: shown(title),
    subtitle: shown(subtitle),
    ...(typeof value === "boolean" ? { toggle: value } : value !== undefined ? { detail: value } : { opens: true as const }),
  };
}

function sub(title: string, body: string | undefined, ...wells: ListRowSpec[][]): SubSection {
  return { title, ...(body ? { body } : {}), wells };
}

const LONG_THEME = words("Theme", "Theme, accent and the colours the wallpaper brings");

/** The sections of Settings, as many as the window has, each a few sub-sections deep. */
export function sections(): Section[] {
  return [
    {
      icon: "wallpaper",
      title: shown(words("Appearance", "Appearance, wallpapers and the colours they bring")),
      accent: "外観",
      lede: shown(words("Make it yours.", "Make it yours. It was already pretty, but go on, nobody's judging.")),
      subs: [
        sub(shown(LONG_THEME), "Where the colours come from.", [
          setting(words("Mode", "Mode, light or dark or on a schedule"), words("Light, dark, or on a schedule.", "Light by day and dark by night, on a schedule of your own."), "Dark"),
          setting(words("Accent", "Accent colour"), words("From the wallpaper.", "From the wallpaper, or a colour of your own."), "Wallpaper"),
        ]),
        sub("Wallpaper", "A new one every hour, from the sets you picked.", [
          setting(words("Sets", "Wallpaper sets"), words("3 sets · 42 wallpapers", "3 sets, 42 wallpapers, all of them good")),
          setting(words("Change every hour", "Change the wallpaper every hour"), words("A new one, on the hour.", "A new one on the hour, every hour, for as long as you like."), true),
        ]),
        sub("Motion", undefined, [
          setting(words("Reduce motion", "Reduce motion everywhere"), words("Fades, not moves.", "Fades instead of moves, everywhere."), false),
          setting(words("Speed", "Animation speed"), words("How fast things move.", "How fast every animation plays."), "1×"),
          setting(words("Focus stutter", "Focus stutter on power changes"), words("The neon's flicker.", "The neon sign's flicker when the power changes."), true),
        ]),
      ],
    },
    {
      icon: "desktop_windows",
      title: shown(words("Displays", "Displays and how they are arranged")),
      accent: "画面",
      lede: shown(words("Pixels, arranged.", "Pixels, arranged. Drag them where your eyes already are.")),
      subs: [
        sub("Built-in display", "2560 × 1600 · 1.5×", [
          setting(words("Resolution", "Resolution"), words("The screen's own.", "The screen's own, the sharpest it does."), "2560 × 1600"),
          setting(words("Scale", "Scale"), words("How big things are.", "How big everything is drawn on it."), "1.5×"),
        ]),
        sub("Night light", undefined, [setting(words("Night light", "Night light at sunset"), words("Warmer after sunset.", "Warmer colours from sunset to sunrise."), true)]),
      ],
    },
    {
      icon: "keyboard",
      title: shown(words("Keyboard", "Keyboard, layouts and shortcuts")),
      accent: "鍵盤",
      lede: shown(words("Clack responsibly.", "Clack responsibly. The neighbours can hear you.")),
      subs: [
        sub("Layouts", undefined, [
          setting(words("English (US)", "English (United States, international)"), words("Default · Super+Space", "The default, switched with Super+Space")),
          setting(words("Polish", "Polish (programmer's)"), words("Super+Space", "Switched with Super+Space")),
        ]),
        sub("Typing", undefined, [
          setting(words("Repeat delay", "Repeat delay"), words("Before a held key repeats.", "How long a held key waits before it repeats."), "300 ms"),
          setting(words("Caps Lock is Esc", "Caps Lock is Escape"), words("Vim says thanks.", "Vim says thanks, and so does your pinky."), true),
        ]),
        sub("Shortcuts", "Every key that does something.", [
          setting(words("Command menu", "Command menu"), words("Super", "Super, tapped on its own")),
          setting(words("Lock", "Lock the screen"), words("Super+L", "Super+L, or close the lid")),
        ]),
      ],
    },
    {
      icon: "bolt",
      title: shown(words("Power", "Power, battery and sleep")),
      accent: "電源",
      lede: shown(words("Juice, managed.", "Juice, managed. The battery thanks you in advance.")),
      subs: [
        sub("Battery", undefined, [
          setting(words("Charge limit", "Charge limit"), words("Kinder to the cells.", "Stops short of full, kinder to the cells."), "80%"),
          setting(words("Battery saver", "Battery saver"), words("When it runs low.", "Turns on by itself when the battery runs low."), "At 20%"),
        ]),
        sub("Sleep and lock", undefined, [
          setting(words("Screens off after", "Turn the screens off after"), words("Idle, unplugged.", "Idle, and on battery."), "5 min"),
          setting(words("When the lid closes", "When the lid closes"), words("Unless a screen is plugged in.", "Unless a screen is plugged in, then nothing."), "Sleep"),
        ]),
        sub("Performance", undefined, [setting(words("Profile", "Power profile"), words("Quiet or fast.", "Quiet and cool, or fast and loud."), "Balanced")]),
      ],
    },
    {
      icon: "notifications",
      title: shown(words("Notifications", "Notifications and do not disturb")),
      accent: "通知",
      lede: shown(words("Ping, but politely.", "Ping, but politely, and only when it matters.")),
      subs: [
        sub("Popups", undefined, [setting(words("Show popups", "Show popups"), words("Top right, briefly.", "In the top right corner, for a few seconds."), true)]),
        sub("Do not disturb", undefined, [setting(words("Do not disturb", "Do not disturb"), words("Only alarms get through.", "Only alarms and calls get through."), false)]),
      ],
    },
    {
      icon: "wifi",
      title: shown(words("Network", "Network, Wi-Fi and VPN")),
      accent: "網",
      lede: shown(words("Bars and bytes.", "Bars and bytes, and the proxy in between.")),
      subs: [
        sub("Wi-Fi", undefined, [setting(words("Wi-Fi", "Wi-Fi"), words("Home network", "Connected to the home network"), true)]),
        sub("Proxy", undefined, [setting(words("Proxy", "Proxy"), words("Off", "Off, every app goes straight out"), false)]),
      ],
    },
    {
      icon: "volume_up",
      title: shown(words("Sound", "Sound, speakers and microphones")),
      accent: "音",
      lede: shown(words("Loud and clear.", "Loud and clear, or quiet as you like.")),
      subs: [sub("Output", undefined, [setting(words("Speakers", "Built-in speakers"), words("Built-in", "The laptop's own"), "60%")])],
    },
    {
      icon: "smartphone",
      title: shown(words("Phone", "Phone and what it shares")),
      accent: "電話",
      lede: shown(words("Your other screen.", "Your other screen, paired with this one.")),
      subs: [sub("Pairing", undefined, [setting(words("Paired phone", "Paired phone"), words("Pixel 8", "Pixel 8, in your pocket"))])],
    },
    {
      icon: "schedule",
      title: shown(words("Date and time", "Date, time and time zone")),
      accent: "時間",
      lede: shown(words("Time flies.", "Time flies. We keep track of it.")),
      subs: [sub("Clock", undefined, [setting(words("Time zone", "Time zone"), words("Automatic", "Set from where you are"), "Warsaw")])],
    },
    {
      icon: "system_update_alt",
      title: shown(words("Updates", "Updates and the apps they bring")),
      accent: "更新",
      lede: shown(words("Fresh, never stale.", "Fresh, never stale, and never in your way.")),
      subs: [sub("Updates", undefined, [setting(words("Update now", "Update now"), words("Last checked an hour ago.", "Last checked an hour ago, nothing new since."))])],
    },
  ];
}

/** Settings' whole outline. */
export function outline(region?: Section["region"]): Outline {
  const all = sections();
  const displays = all[1]!;
  if (region) all[1] = { ...displays, region };
  return {
    home: {
      name: "Home",
      accent: "設定",
      title: shown(words("Settings", "Settings for everything on this machine")),
      lede: shown(words("Everything about this machine, one search away.", "Everything about this machine, one search away. Type what you're after.")),
      placeholder: "Search every setting",
    },
    sections: all,
  };
}

/** Only the first `count` sections: a window with few, so nothing in the sidebar overflows. */
export function fewer(count: number): Outline {
  const whole = outline();
  return { ...whole, sections: whole.sections.slice(0, count) };
}
