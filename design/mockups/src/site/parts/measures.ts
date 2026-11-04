// A page's measures: its samples, each clean at rest with its parts tree and spec list beside it;
// the measures by their tokens; then the reflow in a window of the toolbar's size. There are two
// ways in, and the section draws one thing at a time across its samples: pointing at a part (in a
// sample or its tree) inspects it, pointing at a token's row lights every place it is used. One
// replaces the other. A click pins either; clicking it again or Esc lets it go.
//
// Pinning a part by clicking it in its sample is clicking into the sample, so the sample takes the
// wheel as any clicked sample does (`site/scroll.ts`) and its lists scroll under the pinned part;
// letting the pin go by a click lets the wheel go with it, and Esc lets both go.

import { length, raw } from "../../lib/tokens.ts";
import { windowSize } from "../settings.ts";
import type { Measures } from "../page.ts";
import { el, prose } from "../dom.ts";
import { applies } from "../page.ts";
import { releaseWheel } from "../scroll.ts";
import { inspector, type Inspector, type Lit, litWords, tokenName } from "./inspector.ts";
import { subheading } from "./section.ts";
import { besideWords, type Group } from "./shelf.ts";

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

const AT_REST = "Point at a token to light every place the samples above use it; click to pin it.";

function round(value: number): string {
  return String(Math.round(value * 100) / 100);
}

/** The measures' parts: the samples with their trees, every measure by its token, then the reflow. */
export function measures(spec: Measures): (Group | Element)[] {
  const tokens = spec.measures.map((measure) => measure.token.replace(/^--/, ""));
  let hovered: Lit | null = null;
  let pinned: Lit | null = null;
  const same = (a: Lit | null, b: Lit | null) =>
    a !== null && b !== null && ("part" in a ? "part" in b && a.part === b.part : "token" in b && a.token === b.token);

  const inspectors: Inspector[] = [];
  const rows = new Map<string, { term: HTMLElement; words: HTMLElement }>();
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
      for (const part of [row.term, row.words]) {
        part.toggleAttribute("data-lit", each === token);
        part.toggleAttribute("data-pinned", each === token && lit === pinned);
      }
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
    small: besideWords,
    items: spec.samples.map((sample) => {
      const each = inspector(sample.name, sample.sample(), hands);
      inspectors.push(each);
      return { name: sample.name, words: sample.for, sample: each.holder, aside: each.aside };
    }),
  };

  const table = el("dl", "page-list page-tokens");
  for (const [i, measure] of spec.measures.entries()) {
    const token = tokens[i]!;
    // An aspect ratio is two numbers, not a length: shown as written.
    const value = token.startsWith("aspect-") ? raw(token) : round(length(token));
    const term = el("dt", "", el("code", "page-code", `${tokenName(token)} = ${value}`));
    const words = el("dd", "", prose(measure.what));
    rows.set(token, { term, words });
    for (const part of [term, words]) {
      part.addEventListener("pointerenter", () => point({ token }));
      part.addEventListener("pointerleave", () => point(null));
      part.addEventListener("click", () => click({ token }));
    }
    table.append(term, words);
  }

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
    title: "Reflow",
    items: applies(spec.reflow)
      ? [{ name: `In a window of the toolbar's size, ${size.name}`, words: spec.reflow.described, sample: windowed(spec.reflow.sample()) }]
      : spec.reflow,
  };
  return [samples, subheading("By token"), status, table, reflow];
}

/** `sample` in a window of the toolbar's size. */
function windowed(sample: Element): HTMLElement {
  const size = windowSize();
  const frame = el("div", "page-window", sample);
  frame.style.width = size.width;
  frame.style.height = size.height;
  return frame;
}
