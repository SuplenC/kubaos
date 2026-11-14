// The moves every control in the state model makes (`docs/DESIGN.md`, "States"): lighting up and
// going back to rest, pressed, and the power changes it takes part in, the theme change and dark ⇄
// light, played on its lit samples. A control's page names its samples; the moves are the same.

import { el, label } from "./dom.ts";
import type { Move, NonEmpty, Track } from "./page.ts";
import { type ColourPath, paintBetween, palette, paletteName, themeName } from "./recolour.ts";
import { themeChange, toDark, toLight } from "./sequences.ts";
import { type Mode, settings } from "./settings.ts";

/**
 * A control lighting up or going back to rest, `--lit` from `from` to `to` over `short`, on
 * `target` (`on` names which of the sample's variants): its fill, content, line and glow all follow
 * it, each from its rest colour to its lit one. A fade, never the flicker.
 */
export function lights(target: string, on: string, from: 0 | 1, to: 0 | 1): NonEmpty<Track<"standard">> {
  return [{ part: "lit", on, target, property: "--lit", from, to, duration: "short", curve: "standard" }];
}

/** A lit control pressed, `--press` from 0 to 1 over `micro`. */
export function press(target: string, on: string): NonEmpty<Track<"standard">> {
  return [{ part: "pressed", on, target, property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" }];
}

/** A palette: a theme in a mode. */
interface Painted {
  readonly theme: string;
  readonly mode: Mode;
}

/**
 * `lit`, the page's lit samples, on a surface of their own, painted on the way from `from` to `to`
 * along `path`, as `from`'s mode shows them: what a theme change or a switch plays on. A palette
 * without a mode is in the mode picked when the scene is drawn.
 */
function scene(lit: () => Element, from: Painted | string, to: Painted | string, path: ColourPath): () => Element {
  return () => {
    const mode = settings().mode;
    const [a, b] = [from, to].map((each) => (typeof each === "string" ? { theme: each, mode } : each)) as [Painted, Painted];
    const painted = el("div", "page-scene", label("Every light and every colour, on one clock", "body_strong", "on_surface"), lit());
    painted.dataset.mode = a.mode;
    paintBetween(painted, path, palette(a.theme, a.mode), palette(b.theme, b.mode));
    return el("div", "page-scene-box", painted);
  };
}

/** The theme change between two themes, in the mode picked. */
function themeMove(lit: () => Element, from: string, to: string): Move {
  const between = { from: themeName(from), to: themeName(to) };
  return themeChange(`Theme change: ${between.from} to ${between.to}`, between, scene(lit, from, to, "through grey"));
}

/**
 * The power changes a control takes part in, played on `lit`: two theme changes, then dark to
 * light and back. Lit controls flicker to their signs-off look and back; resting ones just recolour.
 */
export function powerMoves(lit: () => Element): Move[] {
  return [
    themeMove(lit, "neon", "rose"),
    themeMove(lit, "rose", "lagoon"),
    toLight(
      { from: paletteName("neon", "dark"), to: paletteName("neon", "light") },
      scene(lit, { theme: "neon", mode: "dark" }, { theme: "neon", mode: "light" }, "dawn"),
    ),
    toDark(
      { from: paletteName("neon", "light"), to: paletteName("neon", "dark") },
      scene(lit, { theme: "neon", mode: "light" }, { theme: "neon", mode: "dark" }, "dawn"),
    ),
  ];
}
