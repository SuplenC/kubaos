// The Command menu button at the bar's start: the クバ mark (`branding/svg/kubaos-mark-flat.svg`),
// `size.bar_mark` in a `size.bar_row` square target, drawn as the sign switched off. At rest its
// tubes are unlit glass, the warm ones `primary_unlit` and the dakuten `cold_unlit`, a glint of
// `on_surface` along each; hovered the glass warms to `primary_glass` and `cold_glass` over `short`
// on `standard`; pressed a disc of `on_surface` at `opacity.press` comes up behind it over `micro`
// and the mark dips to `scale.press_dip`. Opening the Command menu, by a click or by its key,
// lights the strokes in writing order (ク, バ, the dakuten), each over `short` on `standard`,
// `duration.swap_lag` after the one before: the tube in its tone, its core and its glow. Closing,
// the sign goes off at once over `short`. It never takes the edge light.

import markSvg from "../../../../../branding/svg/kubaos-mark-flat.svg?raw";
import type { Shown } from "../../lib/control.ts";
import { pin } from "../../lib/control.ts";
import { withHint } from "../../lib/hint.ts";
import { probe } from "../../lib/probe.ts";

/** A stroke of the mark: its path, and whether it is a warm tube or the dakuten's cold one. */
interface Stroke {
  readonly d: string;
  readonly cold: boolean;
}

/** The mark's strokes in writing order, as the brand's flat mark draws them: warm, then the dakuten. */
const STROKES: readonly Stroke[] = [...markSvg.matchAll(/<path d="([^"]+)"[^>]*stroke="(#[0-9a-f]+)"/g)].map((found) => ({ d: found[1]!, cold: found[2] !== "#ffb86c" }));

/** The brand mark's own frame: its viewBox and the offset its strokes are drawn at. */
const FRAME = /viewBox="([^"]+)"/.exec(markSvg)?.[1] ?? "";
const OFFSET = /<g transform="([^"]+)"/.exec(markSvg)?.[1] ?? "";

// The brand's small-size weights (`branding/make.py`, "16 and 32 px: no glow, no core, fatter
// strokes"), in the mark's own 120 units: a warm tube 14.5 wide, the dakuten a fifth thinner. Lit,
// a core along each tube's middle and a glow round it, as the full mark draws them. These are the
// drawing's units, scaled with the mark, not lengths of the page.
const WARM = 14.5;
const COLD = WARM * 0.8;
const CORE = 0.22;
const GLOW = 1.6;
const GLINT = 0.18;
/** The glint's offset up and to the left of its tube's middle, where the light catches the glass. */
const CATCH = "translate(-1.2 -1.4)";

/** The button's state: the pointer's, or open while the Command menu is. */
export type CommandState = Shown | "open";

export interface CommandButtonOptions {
  readonly state?: CommandState;
  /** A press opens and closes the Command menu's light, as the menu itself would. */
  readonly live?: boolean;
}

/** The mark as SVG: each stroke its glow, its tube, its core and its glint, numbered in writing order. */
function mark(): SVGSVGElement {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", FRAME);
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.classList.add("shell-cmd-mark", "k-bar-dip");
  const id = `shell-cmd-glow-${Math.random().toString(36).slice(2)}`;
  const strokes = STROKES.map((stroke, index) => {
    const width = stroke.cold ? COLD : WARM;
    const at = `d="${stroke.d}"`;
    return `<g class="shell-cmd-stroke" data-tone="${stroke.cold ? "cold" : "warm"}" style="--order: ${index}">` +
      `<path ${at} class="shell-cmd-glow" stroke-width="${width * GLOW}" filter="url(#${id})"/>` +
      `<path ${at} class="shell-cmd-tube" stroke-width="${width}"/>` +
      `<path ${at} class="shell-cmd-core" stroke-width="${width * CORE}"/>` +
      `<path ${at} class="shell-cmd-glint" stroke-width="${width * GLINT}" transform="${CATCH}"/>` +
      "</g>";
  }).join("");
  // The glow's blur, in the mark's units: the full mark's.
  svg.innerHTML = `<defs><filter id="${id}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="5"/></filter></defs><g transform="${OFFSET}">${strokes}</g>`;
  return svg;
}

/** Lights the button's sign in writing order, or puts it out. */
export function setCommandOpen(button: HTMLElement, open: boolean): void {
  button.toggleAttribute("data-open", open);
}

/** The Command menu button, `size.bar_row` square at the bar's start. */
export function commandButton(options: CommandButtonOptions = {}): HTMLElement {
  const element = document.createElement("span");
  element.className = "shell-cmd k-control";
  element.setAttribute("role", "button");
  element.setAttribute("aria-label", "Command menu");
  const state = options.state ?? "rest";
  if (state === "open") element.dataset.open = "";
  else if (state !== "rest" && state !== "disabled") pin(element, state === "hovered" ? "hover" : state);
  element.append(mark());
  withHint(element, "Command menu  ·  SUPER+Space");
  if (options.live) element.addEventListener("click", () => setCommandOpen(element, !element.hasAttribute("data-open")));
  return probe(element, { role: "component", name: "CommandButton", variant: "", fixed: { width: true, height: true } });
}
