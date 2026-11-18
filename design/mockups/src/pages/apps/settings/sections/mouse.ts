// Mouse & touchpad: how the pointer moves, scrolls and clicks, from Hyprland's input settings
// (`config/hypr/input.lua`, its `cursor` and `misc` beside them) and the workspace swipe
// (`config/hypr/keybinds.lua`). The pointer's look is Appearance's (Motion & pointer).

import { emptyState } from "../../../../lib/empty-state.ts";
import type { SettingsSection, SettingsSub } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { choice, stops, toggle } from "./kit.ts";

// Hyprland's `input:sensitivity`, -1 to 1, in its own tenths; 0 leaves libinput's speed as it is.
const SENSITIVITY = new Intl.NumberFormat("en-GB", { minimumFractionDigits: 1, maximumFractionDigits: 1, signDisplay: "exceptZero" });
const SPEEDS = Array.from({ length: 21 }, (_, i) => SENSITIVITY.format((i - 10) / 10));
const SCROLLS = ["0.25", "0.5", "0.75", "1", "1.5", "2", "3"].map((each) => `${each}×`);

/** KubaOS's values: what every reset puts back. `touchpad` is the machine's, not a setting. */
const DEFAULTS = {
  speed: 10,
  accel: 0,
  primary: 0,
  middlePaste: true,
  hideTyping: true,
  wheelNatural: false,
  wheelScroll: 3,
  tap: true,
  tapDrag: true,
  rightClick: 0,
  padNatural: true,
  padScroll: 1,
  padTyping: true,
  swipe: true,
  fingers: 0,
  swipeWay: 0,
  touchpad: true,
};

/** The Touchpad sub-section on a machine without one: it says so, in place of its settings. */
function noTouchpad(): SettingsSub {
  return {
    title: "Touchpad",
    body: "Taps, clicks and two-finger scrolling.",
    wells: [],
    before: () => emptyState({ icon: "desktop_windows", title: "No touchpad on this machine.", body: "Its settings and its swipes show here once one is connected." }),
  };
}

/** Mouse & touchpad, whole. */
export function mouse(values: Values): SettingsSection {
  const mine = own(values, "mouse", () => ({ ...DEFAULTS }));
  const pointer: SettingsSub = {
    title: "Pointer",
    body: "How it moves, with the mouse and the touchpad alike.",
    wells: [
      [
        stops("pointer-speed", "Pointer speed", "How far it goes for a move of the hand. 0 is as the device sends it.", "mouse", SPEEDS, () => mine.speed, (index) => (mine.speed = index), { initial: () => DEFAULTS.speed }),
        choice("acceleration", "Acceleration", "Further on a quick flick, or the same at any pace.", ["Adaptive", "Flat"], () => mine.accel, (index) => (mine.accel = index), { initial: () => DEFAULTS.accel }),
      ],
      [
        choice("primary-button", "Primary button", "The button that clicks. Right suits a mouse in the left hand.", ["Left", "Right"], () => mine.primary, (index) => (mine.primary = index), {
          drives: true,
          initial: () => DEFAULTS.primary,
          warns: () => (mine.primary === 1 ? { title: "The right button clicks now; the left opens menus.", body: "Swapped by mistake? Hold this row's reset with the right button." } : null),
        }),
        toggle("middle-paste", "Middle click pastes", "Pastes the text you last selected, wherever the pointer is.", () => mine.middlePaste, (on) => (mine.middlePaste = on), { initial: () => DEFAULTS.middlePaste }),
        toggle("hide-typing", "Hide while typing", "The pointer steps out of the way of your words until it moves.", () => mine.hideTyping, (on) => (mine.hideTyping = on), { initial: () => DEFAULTS.hideTyping }),
      ],
    ],
  };
  const wheel: SettingsSub = {
    title: "Mouse",
    body: "The wheel, on any mouse.",
    wells: [
      [
        toggle("wheel-natural", "Natural scrolling", "The page moves the way the wheel turns, as on a touchpad.", () => mine.wheelNatural, (on) => (mine.wheelNatural = on), { initial: () => DEFAULTS.wheelNatural }),
        stops("wheel-scroll", "Scroll speed", "How far a notch of the wheel goes. 1× is as the mouse sends it.", "swap_vert", SCROLLS, () => mine.wheelScroll, (index) => (mine.wheelScroll = index), { initial: () => DEFAULTS.wheelScroll }),
      ],
    ],
  };
  const touchpad: SettingsSub = {
    title: "Touchpad",
    body: "Taps, clicks and two-finger scrolling.",
    wells: [
      [
        toggle("tap-click", "Tap to click", "A light tap clicks; no need to press the pad down.", () => mine.tap, (on) => (mine.tap = on), { drives: true, initial: () => DEFAULTS.tap }),
        toggle("tap-drag", "Tap and drag", "Tap, then touch again and slide, to drag what's under the pointer.", () => mine.tapDrag, (on) => (mine.tapDrag = on), { shows: () => mine.tap, initial: () => DEFAULTS.tapDrag }),
        choice("right-click", "Right click", "Press down with two fingers, or in the pad's bottom right corner.", ["Two fingers", "Corner"], () => mine.rightClick, (index) => (mine.rightClick = index), { initial: () => DEFAULTS.rightClick }),
      ],
      [
        toggle("pad-natural", "Natural scrolling", "The page follows your fingers, as on a phone.", () => mine.padNatural, (on) => (mine.padNatural = on), { initial: () => DEFAULTS.padNatural }),
        stops("pad-scroll", "Scroll speed", "How far the page moves for a stroke of two fingers.", "swap_vert", SCROLLS, () => mine.padScroll, (index) => (mine.padScroll = index), { initial: () => DEFAULTS.padScroll }),
        toggle("pad-typing", "Off while typing", "A palm on the pad doesn't move the pointer while you type.", () => mine.padTyping, (on) => (mine.padTyping = on), { initial: () => DEFAULTS.padTyping }),
      ],
    ],
  };
  const gestures: SettingsSub = {
    title: "Gestures",
    body: "Swipes on the touchpad.",
    wells: [
      [
        toggle("swipe", "Swipe between workspaces", "Fingers sideways bring the next or the previous one.", () => mine.swipe, (on) => (mine.swipe = on), { drives: true, initial: () => DEFAULTS.swipe }),
        choice("swipe-fingers", "Fingers", "How many fingers the swipe takes.", ["Three", "Four"], () => mine.fingers, (index) => (mine.fingers = index), { shows: () => mine.swipe, initial: () => DEFAULTS.fingers }),
        choice("swipe-way", "Direction", "Natural: the workspaces follow your fingers, as pages on a phone.", ["Natural", "Reversed"], () => mine.swipeWay, (index) => (mine.swipeWay = index), { shows: () => mine.swipe, initial: () => DEFAULTS.swipeWay }),
      ],
    ],
  };
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "mouse",
    title: "Mouse & touchpad",
    accent: "指先",
    lede: "Point taken.",
    subs: mine.touchpad ? [pointer, wheel, touchpad, gestures] : [pointer, wheel, noTouchpad()],
  };
}
