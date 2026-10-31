// A symbol of KubaOS Symbols, drawn from `kubaos-ui`'s own SVGs: `kubaos-ui`'s `icon::icon`. Its
// stroke is 2 of its 24 units scaled to its size, rounded half up to whole device pixels and never
// under one, at the box's scale (`.k-icon` in `lib.css`, `kubaos_ui::icon::device_stroke`).

import { probe } from "./probe.ts";

const drawings = import.meta.glob<string>("../../../../crates/kubaos-ui/assets/symbols/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

const byName = new Map(
  Object.entries(drawings).map(([path, svg]) => [path.replace(/^.*\/(.*)\.svg$/, "$1"), svg]),
);

/** Every symbol's name. */
export function symbols(): string[] {
  return [...byName.keys()].sort();
}

/** The symbol `name` in a box the length `size` names (`var(--icon-s20)`), in the colour `color`. */
export function icon(name: string, size: string, color: string): SVGElement {
  const svg = byName.get(name);
  if (!svg) throw new Error(`no symbol ${name}`);
  const holder = document.createElement("div");
  holder.innerHTML = svg;
  const drawing = holder.querySelector("svg");
  if (!drawing) throw new Error(`${name}.svg holds no drawing`);
  drawing.removeAttribute("width");
  drawing.removeAttribute("height");
  drawing.removeAttribute("class");
  drawing.classList.add("k-icon");
  drawing.style.setProperty("--icon-size", size);
  drawing.style.color = color;
  return probe(drawing, { role: "icon", name });
}

/**
 * A name with no drawing, as QML's `Icon` draws one: its box, empty, so nothing beside it moves.
 * Rust names a symbol by its enum, so it can't name one that isn't there.
 */
export function missingIcon(name: string, size: string): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-icon";
  box.style.setProperty("--icon-size", size);
  return probe(box, { role: "icon", name });
}
