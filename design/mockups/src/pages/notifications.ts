// What the notification pages share: the notifications they show, as the shell's shots do
// (`test/shots/fakes/qs/services/Notifs.qml`), and the notifications panel's header and keys, which the
// shell's panel and PanelPage's example of it both draw. Only the library's parts go in here.

import wallpaper from "../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import "./notifications.css";
import { iconButton } from "../lib/icon-button.ts";
import { keyHints } from "../lib/keycap.ts";
import { stack } from "../lib/layout.ts";
import type { Note } from "../lib/notification-card.ts";
import { group } from "../lib/notification-group.ts";
import { text } from "../lib/text.ts";
import { floatingTooltip, tooltip } from "../lib/tooltip.ts";

export const OLA: Note = { app: "Slack", summary: "Ola", body: "Stand-up moved to 10:30 tomorrow.", when: "21:47", unread: true };
export const ANIA: Note = { app: "Slack", summary: "Ania", body: "Pushed the fix for the bar, can you check it before the stand-up?", when: "21:46", main: "Reply", unread: true };
export const MARCO: Note = { app: "Slack", summary: "Marco", body: "Lunch at 13:00?", when: "21:44", unread: true };
export const DEPLOYS: Note = { app: "Slack", summary: "#deploys", body: "kubaos-ui 0.4 is out on the staging mirror.", when: "20:58", unread: true };
export const BUILD: Note = {
  app: "Ghostty",
  summary: "Build finished",
  body: "kubaos-ui and kubaos-specimen built in 41 s; 312 tests passed, none skipped. The specimen's shots changed for the components page.",
  when: "21:41",
  main: "Open",
  others: ["Open log"],
  unread: true,
  photo: wallpaper,
};
export const KUBA_ONE: Note = { app: "Signal", summary: "Kuba", body: "Or ramen at Ippudo, if Fuunji is full.", when: "12:08" };
export const KUBA_TWO: Note = { app: "Signal", summary: "Kuba", body: "Lunch at Fuunji? The queue is short today.", when: "12:08" };
export const BATTERY: Note = {
  app: "KubaOS",
  summary: "Battery at 20%",
  body: "Plug in soon, or switch to Power Saver. At this rate it lasts about 40 min; Power Saver dims the screen and slows the apps in the background to stretch it.",
  when: "Yesterday 19:30",
};
export const UPDATE: Note = { app: "KubaOS", summary: "Update ready", body: "KubaOS 0.9 is ready to install.", when: "Yesterday 19:30" };
export const CRITICAL: Note = { app: "KubaOS", summary: "Battery at 5%", body: "Plug in now: it suspends in 3 min.", when: "21:48", main: "Power Saver", unread: true, critical: true };

export const SLACK: readonly Note[] = [OLA, ANIA, MARCO, DEPLOYS];
export const SIGNAL: readonly Note[] = [KUBA_ONE, KUBA_TWO];
export const KUBAOS: readonly Note[] = [BATTERY, UPDATE];

/** Why Do not disturb is on, when it is. */
export type Silenced = "manual" | "sharing" | "fullscreen";

/**
 * The panel's line under its title, where another panel's pun goes: why it is silenced, how many
 * are new, or a word that there is nothing at all. There is always a line, so the header never moves.
 */
export function status(unread: number, count: number, reason?: Silenced): string {
  const silenced = { manual: "Silenced until you turn it off", sharing: "Silenced while you share your screen", fullscreen: "Silenced while a window is fullscreen" };
  if (reason) return silenced[reason];
  if (unread > 0) return `${unread} new`;
  return count > 0 ? "All read" : "Silence is golden";
}

/** Do not disturb's card: whether it is on, and why or what turns it on. */
export function dndTip(on: boolean): HTMLElement {
  return on ? tooltip("Do not disturb is on", "Until you turn it off") : tooltip("Do not disturb", "Turns on by itself while you share your screen or a window is fullscreen");
}

/**
 * Do not disturb's button, filled while it is on, its card floating under it; the card says what the
 * hint would, so it has no hint of its own.
 */
export function dndButton(on: boolean, shown = false): { readonly anchor: HTMLElement; readonly button: HTMLButtonElement } {
  const button = iconButton({ icon: "dark_mode", label: on ? "Do not disturb is on" : "Do not disturb", kind: on ? "Filled" : "Text", size: "Sm", hint: false });
  return { anchor: floatingTooltip(button, dndTip(on), { held: "end", shown }), button };
}

/** Clear all, with its hint bubble. */
export function clearButton(): HTMLButtonElement {
  return iconButton({ icon: "clear_all", label: "Clear all", kind: "Text", size: "Sm" });
}

/** The keys button's card: every key the panel takes. */
export function keysTip(): HTMLElement {
  const lines = stack(
    "s8",
    "start",
    keyHints([["↑ ↓", "Move"], ["↵", "Unfold or open"]]),
    keyHints([["⇧↵", "Run its main action"]]),
    keyHints([["→ ←", "Open or close a group"]]),
    keyHints([["Del", "Dismiss"], ["D", "Do not disturb"]]),
    keyHints([["Esc", "Fold, close or leave"]]),
  );
  return tooltip("Keys", "", lines);
}

/**
 * The keys button, floating in the panel's bottom right corner `space.s16` in, its card above it held to
 * its end. Not a Tab stop: the keys are what the card lists.
 */
export function keysButton(shown = false): HTMLElement {
  const button = iconButton({ icon: "keyboard", label: "Keys", kind: "Tonal", size: "Sm", hint: false });
  button.tabIndex = -1;
  if (shown) button.dataset.pinned = "lit";
  const anchor = floatingTooltip(button, keysTip(), { side: "above", held: "end", shown });
  anchor.classList.add("k-notif-keys");
  return anchor;
}

/**
 * The panel's list, still: New then Earlier under their labels, one app's in a group, `count` groups
 * (Slack's, Ghostty's, Signal's, KubaOS's), Slack's opened past four, so the list runs on.
 */
export function sectionList(count = 4): HTMLElement {
  const groups: (readonly Note[])[] = [SLACK, [BUILD], SIGNAL, KUBAOS];
  const list = document.createElement("span");
  list.className = "k-notif-list";
  const label = (words: string) => {
    const holder = document.createElement("span");
    holder.className = "k-notif-label";
    holder.append(text(words, "label", "on_surface_variant"));
    return holder;
  };
  groups.slice(0, Math.min(count, 4)).forEach((notes, index) => {
    if (index === 0) list.append(label("New"));
    if (index === 2) list.append(label("Earlier"));
    list.append(group(notes, { open: index === 0 && count > 4 }));
  });
  return list;
}
