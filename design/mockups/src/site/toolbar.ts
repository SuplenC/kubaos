// The toolbar every page has: light or dark, the scale, reduce motion, the speed and the focus
// stutter, the text's length, and the window size a template or a screen is shown in.

import { change, SCALES, type Settings, settings, SPEEDS, WINDOWS } from "./settings.ts";
import { el } from "./dom.ts";

/** A row of buttons, one pressed: the setting's value. */
function choice<K extends keyof Settings>(
  name: string,
  key: K,
  options: readonly { value: Settings[K]; words: string }[],
  disabled = "",
): HTMLElement {
  const group = el("div", "site-choice", el("span", "site-choice-name", name));
  group.setAttribute("role", "group");
  group.setAttribute("aria-label", name);
  if (disabled) group.title = disabled;
  for (const option of options) {
    const button = el("button", "site-control", option.words);
    button.type = "button";
    button.disabled = disabled !== "";
    button.dataset.setting = key;
    button.dataset.value = String(option.value);
    button.setAttribute("aria-pressed", String(settings()[key] === option.value));
    button.addEventListener("click", () => change({ [key]: option.value } as Partial<Settings>));
    group.append(button);
  }
  return group;
}

const onOff = [
  { value: false, words: "Off" },
  { value: true, words: "On" },
] as const;

/** Marks the buttons of `bar` that are the settings now, keeping focus where it is. */
export function updateToolbar(bar: Element): void {
  const now = settings();
  for (const button of bar.querySelectorAll<HTMLElement>("[data-setting]")) {
    const key = button.dataset.setting as keyof Settings;
    button.setAttribute("aria-pressed", String(String(now[key]) === button.dataset.value));
  }
}

/**
 * What the display's pixel ratio means for the page: a CSS pixel is a device pixel only at a whole
 * ratio, since Chromium lays a page out in steps that don't divide a fractional one. On a screen
 * at 1.25 the browser's zoom at 80% makes the ratio 1.
 */
export function ratioNote(): HTMLElement | null {
  const ratio = devicePixelRatio;
  if (Math.abs(ratio - Math.round(ratio)) < 1e-3) return null;
  const whole = Math.max(1, Math.round(ratio));
  const zoom = Math.round((whole / ratio) * 100);
  return el("div", "site-note", `This screen's pixel ratio is ${ratio}: boxes land between device pixels. Zoom the browser to ${zoom}% for exact pixels.`);
}

/** The toolbar; `hasWindow` when the page shows something in a window of the picked size. */
export function toolbar(hasWindow: boolean): HTMLElement {
  const note = ratioNote();
  return el(
    "div",
    "site-toolbar",
    ...(note ? [note] : []),
    choice("Mode", "mode", [
      { value: "dark", words: "Dark" },
      { value: "light", words: "Light" },
    ]),
    choice("Scale", "scale", SCALES.map((s) => ({ value: s, words: `${s}×` }))),
    choice("Reduce motion", "reduced", onOff),
    choice("Speed", "speed", SPEEDS.map((s) => ({ value: s, words: `${s}×` }))),
    choice("Stutter", "stutter", onOff),
    choice("Text", "text", [
      { value: "short", words: "Short" },
      { value: "long", words: "Long" },
      { value: "translated", words: "Translated" },
    ]),
    choice(
      "Window",
      "window",
      WINDOWS.map((w) => ({ value: w.name, words: w.name })),
      hasWindow ? "" : "This page shows nothing in a window: its reflow doesn't apply.",
    ),
  );
}
