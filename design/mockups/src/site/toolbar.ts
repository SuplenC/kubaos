// The top bar every page has: every setting the page is shown at, grouped under a label each as
// the library's segmented controls (`tabs`), and the search over the pages on the right. Look is
// the mode (night, day, or both side by side on the stage) and the theme; Scale; Text, the words'
// length; Motion, the speed and the flicker or reduced motion; Window, the size a template or a
// screen's reflow is shown at, off on a page with none.

import { field } from "../lib/field.ts";
import { keycap } from "../lib/keycap.ts";
import { tabs } from "../lib/tabs.ts";
import { el, label } from "./dom.ts";
import { change, SCALES, type Settings, settings, SPEEDS, WINDOWS } from "./settings.ts";
import { THEMES } from "./themes.ts";

/** A segmented control's choice: its words, whether it is the setting now, and what picking it changes. */
interface Choice {
  readonly words: string;
  readonly on: (now: Settings) => boolean;
  readonly patch: Partial<Settings>;
}

/** A segmented control of `choices`, the one that is the setting now picked. */
function segmented(name: string, choices: readonly Choice[], disabled = false): HTMLElement {
  const now = settings();
  const picked = Math.max(0, choices.findIndex((choice) => choice.on(now)));
  const well = tabs({
    choices: choices.map((choice) => ({ label: choice.words })),
    picked,
    disabled,
    onPick: (index) => change(choices[index]!.patch),
  });
  well.setAttribute("aria-label", name);
  return well;
}

/** A labelled group of segmented controls. */
function group(name: string, ...controls: HTMLElement[]): HTMLElement {
  const box = el("div", "site-group", label(name, "label", "on_surface_variant"), ...controls);
  box.setAttribute("role", "group");
  box.setAttribute("aria-label", name);
  return box;
}

/** Each of `values` as a choice of `key`, in words by `words`. */
function each<K extends keyof Settings>(key: K, values: readonly Settings[K][], words: (value: Settings[K]) => string): Choice[] {
  return values.map((value) => ({ words: words(value), on: (now) => now[key] === value, patch: { [key]: value } as Partial<Settings> }));
}

const LOOK: readonly Choice[] = [
  { words: "Night", on: (now) => !now.both && now.mode === "dark", patch: { mode: "dark", both: false } },
  { words: "Day", on: (now) => !now.both && now.mode === "light", patch: { mode: "light", both: false } },
  { words: "Both", on: (now) => now.both, patch: { both: true } },
];

/** The words' lengths, short enough to keep the bar on one row at 1920: a translation is 40% longer. */
const TEXT_WORDS: Record<Settings["text"], string> = { short: "Short", long: "Long", translated: "+40%" };

const MOTION: readonly Choice[] = [
  { words: "Flicker", on: (now) => !now.reduced && now.flicker, patch: { reduced: false, flicker: true } },
  { words: "Steady", on: (now) => !now.reduced && !now.flicker, patch: { reduced: false, flicker: false } },
  { words: "Reduced", on: (now) => now.reduced, patch: { reduced: true } },
];

/**
 * What the display's pixel ratio means for the page: a CSS pixel is a device pixel only at a whole
 * ratio, since Chromium lays a page out in steps that don't divide a fractional one. On a screen
 * at 1.25 the browser's zoom at 80% makes the ratio 1.
 */
function ratioNote(): HTMLElement | null {
  const ratio = devicePixelRatio;
  if (Math.abs(ratio - Math.round(ratio)) < 1e-3) return null;
  const whole = Math.max(1, Math.round(ratio));
  const zoom = Math.round((whole / ratio) * 100);
  return el("div", "site-note", label(`Pixel ratio ${ratio}: zoom to ${zoom}% for exact pixels`, "small", "warning"));
}

/** The search over the pages: what is typed filters the navigation, and Enter opens the first left. */
function search(onQuery: (query: string) => void, onEnter: () => void): HTMLElement {
  const box = field({ placeholder: "Search", icon: "search", onInput: onQuery });
  box.classList.add("site-search");
  box.addEventListener("keydown", (event) => {
    if (event.key === "Enter") onEnter();
  });
  addEventListener("keydown", (event) => {
    const typing = event.target instanceof HTMLElement && (event.target.isContentEditable || event.target.matches("input, textarea, [role=textbox]"));
    // A sample that handles the key has prevented its default: the key is the sample's.
    if (event.key !== "/" || typing || event.defaultPrevented) return;
    event.preventDefault();
    box.focus();
  });
  return el("div", "site-group site-find", box, keycap("/"));
}

export interface Toolbar {
  readonly element: HTMLElement;
  /** Turns the Window group on for a page that shows something in a window, off for one that doesn't. */
  windows(shown: boolean): void;
}

/** The top bar; `onQuery` and `onEnter` are the search's, over the navigation. */
export function toolbar(onQuery: (query: string) => void, onEnter: () => void): Toolbar {
  const note = ratioNote();
  const windowGroup = (shown: boolean) =>
    group(
      "Window",
      segmented(
        "Window",
        each("window", WINDOWS.map((w) => w.name), (name) => (name === "Minimum" ? "Min" : name)),
        !shown,
      ),
    );
  let windows = windowGroup(false);
  const element = el(
    "div",
    "site-toolbar",
    label("Mockups", "subheading", "on_surface"),
    group("Look", segmented("Mode", LOOK), segmented("Theme", each("theme", THEMES.map((theme) => theme.slug), (slug) => THEMES.find((theme) => theme.slug === slug)!.name))),
    group("Scale", segmented("Scale", each("scale", SCALES, (s) => `${s}×`))),
    group("Text", segmented("Text", each("text", ["short", "long", "translated"], (length) => TEXT_WORDS[length]))),
    group("Motion", segmented("Speed", each("speed", SPEEDS, (s) => (s === 0.5 ? "½×" : `${s}×`))), segmented("Motion", MOTION)),
    windows,
    ...(note ? [note] : []),
    search(onQuery, onEnter),
  );
  return {
    element,
    windows(shown: boolean) {
      if (windows.querySelector("[aria-disabled=true]") === null === shown) return;
      const next = windowGroup(shown);
      windows.replaceWith(next);
      windows = next;
    },
  };
}
