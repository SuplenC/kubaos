// A page's measures: the anatomy table, each part by its token and the token's value at the scale
// picked; the samples, each clean at rest with its parts tree and spec list beside it; then the
// reflow in a window of the top bar's size. There are two
// ways in, and the section draws one thing at a time across its samples: pointing at a part (in a
// sample or its tree) inspects it, pointing at a token's row lights every place it is used. One
// replaces the other. A click pins either; clicking it again or Esc lets it go.
//
// Pinning a part by clicking it in its sample is clicking into the sample, so the sample takes the
// wheel as any clicked sample does (`site/scroll.ts`) and its lists scroll under the pinned part;
// letting the pin go by a click lets the wheel go with it, and Esc lets both go.

import { length, raw } from "../../lib/tokens.ts";
import { settings, windowSize } from "../settings.ts";
import type { Measures } from "../page.ts";
import { el } from "../dom.ts";
import { applies } from "../page.ts";
import { releaseWheel } from "../scroll.ts";
import { inspector, type Inspector, type Lit, litWords, tokenName } from "./inspector.ts";
import { type Group, inspected } from "./shelf.ts";
import { specTable } from "./table.ts";

/** Each measures section's drawing, again: after a change of scale or of the window. */
const sections = new Set<() => void>();

/** Draws every measures section's layer again, as the page was laid out anew. */
export function redrawOverlays(): void {
  for (const draw of sections) draw();
}

/** Every section's pin, let go on Esc. */
const unpins = new Set<() => void>();

addEventListener("keydown", (event) => {
  if (event.key === "Escape") for (const unpin of unpins) unpin();
});

const AT_REST = "Point at a row to light its token, or a part to read its measures; click pins, Esc lets go.";

function round(value: number): string {
  return String(Math.round(value * 100) / 100);
}

/** `token`'s value as the table shows it: logical pixels, and device pixels at another scale. */
function value(token: string): string {
  // An aspect ratio is two numbers, not a length: shown as written.
  if (token.startsWith("aspect-")) return raw(token);
  // Its number only stands for half the shorter side, which no table should read as a measure.
  if (token === "radius-full") return "full";
  const logical = length(token);
  const scale = settings().scale;
  return scale === 1 ? round(logical) : `${round(logical)} · ${round(logical * scale)} px`;
}

/** The measures' parts, each placed by the page. */
export interface Drawn {
  /** Part, token and value, a row per measure, lighting where the samples use it. */
  readonly table: HTMLElement;
  /** The line over the samples saying what is lit. */
  readonly status: HTMLElement;
  /** The samples with their trees. */
  readonly samples: Group;
  readonly reflow: Group;
}

/** The measures' parts: the table, the samples with their trees, and the reflow. */
export function measures(spec: Measures): Drawn {
  const tokens = spec.measures.map((measure) => measure.token.replace(/^--/, ""));
  let hovered: Lit | null = null;
  let pinned: Lit | null = null;
  const same = (a: Lit | null, b: Lit | null) =>
    a !== null && b !== null && ("part" in a ? "part" in b && a.part === b.part : "token" in b && a.token === b.token);

  const inspectors: Inspector[] = [];
  const rows = new Map<string, HTMLElement>();
  const status = el("p", "page-token-status", AT_REST);

  const draw = () => {
    const lit = pinned ?? hovered;
    let places = 0;
    for (const each of inspectors) {
      if (!each.holder.isConnected) continue;
      places += each.draw(lit, lit !== null && lit === pinned);
    }
    const token = lit && "token" in lit ? lit.token : null;
    for (const [each, row] of rows) {
      row.toggleAttribute("data-lit", each === token);
      row.toggleAttribute("data-pinned", each === token && lit === pinned);
    }
    // The count is a line of its own over the table, so lighting a row never moves the table.
    const unseen = token && places === 0 ? inspectors.filter((each) => each.uses(token)).map((each) => each.name) : [];
    status.textContent = token ? litWords(token, places, unseen) : AT_REST;
    status.toggleAttribute("data-lit", token !== null);
  };
  const point = (lit: Lit | null) => {
    if (same(lit, hovered) || (lit === null && hovered === null)) return;
    hovered = lit;
    if (!pinned) draw();
  };
  const click = (lit: Lit) => {
    if (same(lit, pinned)) {
      pinned = null;
      hovered = lit;
      if ("part" in lit) releaseWheel();
    } else pinned = lit;
    draw();
  };
  const unpin = () => {
    if (!pinned) return;
    pinned = null;
    draw();
  };

  const hands = {
    tokens,
    point: (part: Element | null, from: Inspector) => {
      if (part) point({ part, in: from });
      else if (hovered && "part" in hovered && hovered.in === from) point(null);
    },
    click: (part: Element, from: Inspector) => click({ part, in: from }),
  };
  const samples: Group = {
    title: "",
    small: inspected,
    items: spec.samples.map((sample) => {
      const each = inspector(sample.name, sample.sample(), hands);
      inspectors.push(each);
      return { name: sample.name, words: sample.for, sample: each.holder, aside: each.aside };
    }),
  };

  const drawn = specTable(
    { heads: ["Part", "Token", "Value"], numeric: [2] },
    spec.measures.map((measure, i) => ({ cells: [measure.part, `\`${tokenName(tokens[i]!)}\``, value(tokens[i]!)] })),
  );
  drawn.element.classList.add("page-tokens");
  drawn.rows.forEach((row, i) => {
    const token = tokens[i]!;
    rows.set(token, row);
    row.addEventListener("pointerenter", () => point({ token }));
    row.addEventListener("pointerleave", () => point(null));
    row.addEventListener("click", () => click({ token }));
  });

  const redraw = () => {
    if (!inspectors.some((each) => each.holder.isConnected)) {
      sections.delete(redraw);
      unpins.delete(unpin);
      return;
    }
    draw();
  };
  sections.add(redraw);
  unpins.add(unpin);

  const size = windowSize();
  const reflow: Group = {
    title: "",
    items: applies(spec.reflow)
      ? [{ name: `In a window, ${size.name}`, words: spec.reflow.note, sample: windowed(spec.reflow.sample()) }]
      : spec.reflow,
  };
  return { table: drawn.element, status, samples, reflow };
}

/** `sample` in a window of the toolbar's size. */
function windowed(sample: Element): HTMLElement {
  const size = windowSize();
  const frame = el("div", "page-window", sample);
  frame.style.width = size.width;
  // A part keeps its own height in the window, as it does in an app; a whole surface fills it.
  const whole = sample.matches("[data-whole]") || sample.querySelector("[data-whole]") !== null;
  frame.style[whole ? "height" : "maxHeight"] = size.height;
  return frame;
}
