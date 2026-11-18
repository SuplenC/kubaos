// Keyboard: the layouts (`layouts` in `keyboard.toml`, `kubaos keyboard`), how held keys repeat and
// what the odd keys do (`config/hypr/input.lua`, `mac_accents`), and every shortcut Hyprland binds.

import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { CAPS, COMPOSE, countOf, DELAYS, type Kb, KB_DEFAULTS, keysAtEnd, layoutsWell, needsAltGr, RATES, shortcutsPage } from "./keyboard-parts.ts";
import { opens, pick, stops, toggle } from "./kit.ts";

/** A row that says which keys do something, as keycaps at its end. */
function keysRow(id: string, title: string, line: string, combination: string): Setting {
  return { id, title, line, control: { kind: "part", make: () => keysAtEnd(combination) } };
}

/** Draws the section again from where it is: the layouts' well, and what follows from them. */
function redraw(values: Values): void {
  values.window?.querySelector<HTMLElement>(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
}

/** Keyboard, whole. */
export function keyboard(values: Values): SettingsSection {
  const kb = own<Kb>(values, "keyboard", () => ({ ...KB_DEFAULTS }));
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "keyboard",
    title: "Keyboard",
    accent: "鍵盤",
    lede: "Clack responsibly.",
    subs: [
      {
        title: "Layouts",
        body: "What your keys type. The first is your default.",
        before: () => layoutsWell(kb, () => redraw(values)),
        wells: [
          [
            keysRow("next-layout", "Next layout", "Switches between your layouts; the bar shows the one in use.", "Super+Shift+Space"),
            {
              id: "console-layout",
              title: "Before you log in",
              line: "The disk unlock and the console type with your default.",
              control: { kind: "fact", detail: () => kb.layouts[0]?.name ?? "" },
            },
          ],
        ],
      },
      {
        title: "Typing",
        body: "How held keys repeat, and what the odd keys do.",
        wells: [
          [
            stops("repeat-delay", "Repeat starts after", "How long a held key waits before it types again.", "hourglass_top", DELAYS, () => kb.delay, (index) => (kb.delay = index), { unbuilt: true, initial: () => KB_DEFAULTS.delay }),
            stops("repeat-rate", "Repeat speed", "How fast a held key types once it starts.", "repeat", RATES, () => kb.rate, (index) => (kb.rate = index), { unbuilt: true, initial: () => KB_DEFAULTS.rate }),
            {
              id: "try-typing",
              title: "Try it here",
              line: "Hold a key to feel the repeat; accents work too.",
              control: { kind: "field", value: () => kb.tryIt, placeholder: "Hold a key here", check: () => null, set: (value) => (kb.tryIt = value) },
            },
          ],
          [
            toggle("mac-accents", "Mac-style accents", "Left Alt with e, `, u, i or n, then a letter: é à ö ô ñ.", () => kb.accents, (on) => (kb.accents = on), { initial: () => KB_DEFAULTS.accents }),
            pick("caps-lock", "Caps Lock key", "Make the key nobody needs into one you do.", CAPS, () => kb.caps, (index) => (kb.caps = index), { unbuilt: true, initial: () => KB_DEFAULTS.caps }),
            pick("compose", "Compose key", "Press it, then two keys for one character: - and > type →.", COMPOSE, () => kb.compose, (index) => (kb.compose = index), {
              unbuilt: true,
              initial: () => KB_DEFAULTS.compose,
              warns: () => {
                const layout = needsAltGr(kb.layouts);
                return kb.compose === 1 && layout ? { title: `${layout.name} types some characters with Right Alt.`, body: "As the compose key it stops typing them; Right Ctrl is free." } : null;
              },
            }),
          ],
        ],
      },
      {
        title: "Shortcuts",
        body: "Every key that does something, and keys of your own.",
        wells: [
          [
            opens("shortcuts", "All shortcuts", "Find one by what it does, or add one of your own.", "shortcuts", () => `${countOf(kb)} shortcuts`),
            keysRow("cheat-sheet", "Shortcuts over any app", "Lists every shortcut over what you're doing; Esc closes it.", "Super+F1"),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "shortcuts",
        title: "All shortcuts",
        body: "Every key KubaOS answers to, found by what it does.",
        back: "Shortcuts",
        before: () => shortcutsPage(kb),
        wells: [],
      },
    ],
  };
}
