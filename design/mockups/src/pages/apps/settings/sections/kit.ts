// The rows a section is written with: a switch, a few named choices, a list, a slider of named stops,
// a fact, a row opening a page, and the values a row holds but doesn't show.

import type { Setting } from "../settings-parts.ts";

/** The values a setting holds per power source or per stop that its row doesn't show. */
export function unseen<T>(all: T[], initial: readonly T[], shown: () => number): NonNullable<Setting["unseen"]> {
  return {
    at: () => all.every((each, index) => index === shown() || each === initial[index]),
    reset: () => all.forEach((_, index) => index !== shown() && (all[index] = initial[index]!)),
  };
}

/** A setting with a switch. */
export function toggle(id: string, title: string, line: string, on: () => boolean, set: (on: boolean) => void, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "switch", on, set }, ...more };
}

/** A setting picked from a few choices, side by side. */
export function choice(id: string, title: string, line: string, choices: readonly string[], picked: () => number, set: (index: number) => void, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "choice", choices, picked, set }, ...more };
}

/** A setting picked from a list. */
export function pick(id: string, title: string, line: string, options: readonly string[], picked: () => number, set: (index: number) => void, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "select", options, picked, set }, ...more };
}

/** A setting on a slider of named stops. */
export function stops(id: string, title: string, line: string, icon: string, names: readonly string[], index: () => number, set: (index: number) => void, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "slider", icon, names, index, set }, ...more };
}

/** A setting that is only a fact. */
export function fact(id: string, title: string, line: string, detail: string, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "fact", detail: () => detail }, ...more };
}

/** A row that opens a page. */
export function opens(id: string, title: string, line: string, to: string, detail?: () => string, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "opens", to, ...(detail ? { detail } : {}) }, ...more };
}
