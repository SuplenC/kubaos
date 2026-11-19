// Tips: a faster or hidden way to do what a setting is about, so none of it is a secret. Each is one
// whole sentence naming its keys as `[Key]` where they fall, true of KubaOS as it is, and says where:
// the bind or the Command menu Item it teaches. A Command menu tip always reads "Press [Super] [Space]
// and type “…” to …". They are laid on the sections' rows, headings and parts by the setting's id
// and the heading's title, so a section is written without them. A part's tip is drawn by the part
// before a sub-section's wells, inside it (About's update card); search finds it as a heading's.

import type { SettingsSection, SettingsSub, SubPage } from "../settings-parts.ts";

/** One section's tips: by a setting's id, by a sub-section's title, and by the title of the sub-section whose part draws it. */
export interface SectionTips {
  readonly rows?: Readonly<Record<string, string>>;
  readonly subs?: Readonly<Record<string, string>>;
  readonly parts?: Readonly<Record<string, string>>;
}

/** Every section's tips, by its place in the sidebar. */
export const TIPS: readonly SectionTips[] = [
  // About
  {
    parts: {
      // config/kotodama/kubaos.toml: "Update KubaOS". Inside the update's card.
      Updates: "Press [Super] [Space] and type “update KubaOS” to update now.",
    },
  },
  // Appearance
  {
    rows: {
      // config/hypr/keybinds.lua: "SUPER + CTRL + Space", Wallpaper picker.
      "night-wallpaper": "[Super] [Ctrl] [Space] opens the wallpaper picker from anywhere.",
      // config/kotodama/kubaos.toml: "Animate wallpaper always" / "on AC" / "Never animate wallpaper".
      "wallpaper-motion": "Press [Super] [Space] and type “animate wallpaper” to change it from anywhere.",
      // crates/kotodama-kubaos/src/shell.rs: "Focus stutter: on/off".
      flicker: "Press [Super] [Space] and type “focus stutter” to turn it on or off from anywhere.",
      // config/kotodama/kubaos.toml: "Dark cursor", "Accent cursor", "Classic cursor", "Set cursor size".
      "cursor-style": "Press [Super] [Space] and type “cursor” to change its look or size from anywhere.",
    },
  },
  // Displays
  {
    rows: {
      // config/hypr/keybinds.lua: "ALT + XF86MonBrightnessUp/Down", Brightness up/down 1%.
      brightness: "Hold [Alt] as you press the brightness keys to step by 1 % instead of 5 %.",
      // config/hypr/keybinds.lua: "SUPER + CTRL + ALT + Right", Next workspace group.
      group: "[Super] [Ctrl] [Alt] [→] goes to the next workspace group.",
    },
    subs: {
      // config/hypr/keybinds.lua: "SUPER + SHIFT + ALT + Left/Right", Move workspace to screen.
      Screens: "[Super] [Shift] [Alt] [←] sends the workspace you're on to the screen on the left.",
    },
  },
  // Desktop
  {
    rows: {
      // config/hypr/keybinds.lua: "SUPER + SHIFT + BackSpace", Toggle gaps (until the next reload).
      gaps: "[Super] [Shift] [Backspace] takes the gaps away for now, and brings them back.",
      // config/hypr/keybinds.lua: "SUPER + E", Toggle split direction.
      split: "[Super] [E] flips the split of the window you're in.",
      // config/hypr/keybinds.lua: "ALT + Tab", Next window.
      "follow-mouse": "[Alt] [Tab] goes to the next window without the pointer.",
      // config/hypr/keybinds.lua: "SUPER + mouse:272" Drag window, "SUPER + mouse:273" Resize window.
      "resize-edges": "Hold [Super] and drag with the right button to resize a window from anywhere on it; the left button moves it.",
    },
    subs: {
      // config/hypr/keybinds.lua: "SUPER + Tab", Next workspace; hl.gesture, three fingers horizontal.
      Workspaces: "[Super] [Tab] goes to the next workspace, and so does a three-finger swipe sideways.",
    },
  },
  // Keyboard
  {
    subs: {
      // config/hypr/keybinds.lua: "SUPER + V", Paste, Ctrl+Shift+V in a terminal (universal).
      Typing: "[Super] [V] pastes in every app, terminals too.",
    },
  },
  // Mouse & touchpad
  {
    rows: {
      // config/hypr/keybinds.lua: "SUPER + mouse_down/mouse_up", Next/Previous workspace.
      swipe: "Hold [Super] and turn the mouse wheel to step through the workspaces too.",
    },
  },
  // Power
  {
    rows: {
      // config/kotodama/kubaos.toml: "Charge limit: 80 %", "90 %", "full".
      "charge-limit": "Press [Super] [Space] and type “charge limit” to change it from anywhere.",
      // docs/USING.md, Idle: "lock after 2m" typed in the Command menu.
      lock: "Press [Super] [Space] and type “lock after 2m” to set it from anywhere.",
      // config/hypr/keybinds.lua: "SUPER + CTRL + I", Keep awake on/off.
      "keep-awake": "[Super] [Ctrl] [I] turns Keep awake on and off from anywhere.",
      // config/kotodama/kubaos.toml: "Super Saver" … "Max Performance".
      "stop-battery": "Press [Super] [Space] and type a stop's name, as “Balanced”, to switch to it from anywhere.",
    },
    subs: {
      // config/hypr/keybinds.lua: "SUPER + CTRL + Escape", Lock.
      "Sleep & lock": "[Super] [Ctrl] [Esc] locks the screen now.",
    },
  },
  // Notifications
  {
    rows: {
      // config/hypr/keybinds.lua: "SUPER + CTRL + comma", Do not disturb.
      dnd: "[Super] [Ctrl] [,] turns Do not disturb on and off from anywhere.",
      // config/hypr/keybinds.lua: "SUPER + SHIFT + comma", Clear notifications.
      "popup-time": "[Super] [Shift] [,] clears every notification at once.",
    },
    subs: {
      // config/hypr/keybinds.lua: "SUPER + N", Notifications.
      Popups: "[Super] [N] opens the panel with every notification kept.",
    },
  },
  // Proxy
  {
    rows: {
      // config/kotodama/kubaos.toml: "Proxy on", "Proxy off".
      proxy: "Press [Super] [Space] and type “proxy on” or “proxy off” to switch it from anywhere.",
    },
  },
  // Network
  {
    rows: {
      // config/hypr/keybinds.lua: "SUPER + CTRL + W", Control center (Wi-Fi).
      wifi: "[Super] [Ctrl] [W] opens Wi-Fi in the control center.",
      // config/hypr/keybinds.lua: "SUPER + CTRL + B", Control center (Bluetooth).
      bluetooth: "[Super] [Ctrl] [B] opens Bluetooth in the control center.",
    },
  },
  // Phone
  {
    rows: {
      // config/kotodama/kubaos.toml: "Send the clipboard to the phone".
      "send-copies": "Press [Super] [Space] and type “send the clipboard” to send what you copied now.",
      // config/kotodama/kubaos.toml: "Ring the phone".
      "paired-phones": "Press [Super] [Space] and type “ring the phone” to make it ring wherever it is.",
    },
  },
  // Date & time
  {
    rows: {
      // config/kotodama/kubaos.toml: "Time format".
      "time-format": "Press [Super] [Space] and type “time format” to change it from anywhere.",
    },
    subs: {
      // config/kotodama/kubaos.toml: "Open calendar".
      Calendar: "Press [Super] [Space] and type “calendar” to open the month at once.",
    },
  },
  // Apps
  {
    rows: {
      // docs/USING.md, Startup apps: an app's Actions (Ctrl+K) in the Command menu, Start on login.
      startup: "Press [Ctrl] [K] on an app in the Command menu to find Start on login among its actions.",
    },
  },
];

/** `list`'s rows and headings given their tips, each time they are read: a section may draw them anew. */
function lay<T extends SettingsSub | SubPage>(list: readonly T[], tips: SectionTips): readonly T[] {
  for (const part of list) {
    const heading = "back" in part ? undefined : tips.subs?.[part.title];
    if (heading) Object.assign(part, { tip: heading });
    const inside = "back" in part ? undefined : tips.parts?.[part.title];
    if (inside) Object.assign(part, { partTip: inside });
    for (const setting of part.wells.flat()) {
      const tip = tips.rows?.[setting.id];
      if (tip) Object.assign(setting, { tip });
    }
  }
  return list;
}

/** `section` with `tips` on its rows and headings, its sub-pages' rows included. */
export function withTips(section: SettingsSection, tips: SectionTips | undefined): SettingsSection {
  if (!tips) return section;
  return Object.create(section, {
    subs: { get: () => lay(section.subs, tips), enumerable: true },
    pages: { get: () => (section.pages ? lay(section.pages, tips) : undefined), enumerable: true },
  }) as SettingsSection;
}
