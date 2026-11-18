// A symbol of KubaOS Symbols, drawn from `kubaos-ui`'s own SVGs: `kubaos-ui`'s `icon::icon`. Its
// stroke is 2 of its 24 units scaled to its size, rounded half up to whole device pixels and never
// under one, at the box's scale (`.k-icon` in `lib.css`, `kubaos_ui::icon::device_stroke`).

import { probe } from "./probe.ts";

const drawings = import.meta.glob<string>("../../../../crates/kubaos-ui/assets/symbols/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

// Symbols the mockups draw before the libraries have them: new ones, waiting to join the set.
const fresh = import.meta.glob<string>("../assets/symbols/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

const byName = new Map(
  [...Object.entries(drawings), ...Object.entries(fresh)].map(([path, svg]) => [path.replace(/^.*\/(.*)\.svg$/, "$1"), svg]),
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

/**
 * Symbols that morph into each other: each two shapes on the symbols' 24-unit grid, whose corners
 * move from one state's to the other's. Play's triangle is its two halves, which part into pause's
 * two bars, the first half running half a unit into the second, so no seam shows between them. Drawn
 * solid, a stroke's width out from the outline symbols' paths, so they stand as big.
 */
const MORPHS = {
  play_pause: {
    off: [
      [[6, 3], [6, 21], [14, 16.2], [14, 7.8]],
      [[13.5, 7.4], [13.5, 16.6], [21, 12], [21, 12]],
    ],
    on: [
      [[5, 4], [5, 20], [11, 20], [11, 4]],
      [[13, 4], [13, 20], [19, 20], [19, 4]],
    ],
  },
} as const;

export type Morph = keyof typeof MORPHS;

/**
 * A symbol between two states that morphs from one to the other (`setMorph`): `--morph` goes 0 to
 * 1 over `short` on `standard`, and every corner of its shapes moves with it, never a cross-fade.
 * `on` is the second state: pause, for `play_pause`. With reduce motion it changes at once.
 */
export function morphIcon(pair: Morph, size: string, color: string, on = false): HTMLElement {
  const box = document.createElement("span");
  // A symbol like any other: sized and placed as an icon is, in a button's slot too.
  box.className = "k-icon k-morph";
  box.style.setProperty("--icon-size", size);
  box.style.color = color;
  const shapes = MORPHS[pair];
  shapes.off.forEach((off, i) => {
    const to = shapes.on[i]!;
    const corner = (axis: 0 | 1, at: number) => `calc((${off[at]![axis]} + ${to[at]![axis] - off[at]![axis]} * var(--morph)) * 100% / 24)`;
    const part = document.createElement("span");
    part.className = "k-morph-part";
    part.style.clipPath = `polygon(${off.map((_, at) => `${corner(0, at)} ${corner(1, at)}`).join(", ")})`;
    box.append(part);
  });
  box.dataset.pair = pair;
  box.toggleAttribute("data-on", on);
  return probe(box, { role: "icon", name: pair });
}

/**
 * A morphing symbol's shapes as they stand now, at its `--morph`: each a list of corners on the
 * 24-unit grid. What draws it outside CSS (a lit button's glow) draws these.
 */
export function morphShapes(box: Element): number[][][] {
  const pair = (box.getAttribute("data-pair") ?? "play_pause") as Morph;
  const at = Number.parseFloat(getComputedStyle(box).getPropertyValue("--morph")) || 0;
  const shapes = MORPHS[pair];
  return shapes.off.map((off, i) => off.map((corner, c) => corner.map((value, axis) => value + (shapes.on[i]![c]![axis]! - value) * at)));
}

/** Morphs `element`'s morphing symbol, or the one in it, to its second state, or back. */
export function setMorph(element: Element, on: boolean): void {
  const box = element.matches(".k-morph") ? element : element.querySelector(".k-morph");
  box?.toggleAttribute("data-on", on);
}
