// The measures overlay: every probed box of a sample outlined and its size written in logical
// pixels, a component's padding and a layout's gaps shaded and labelled with the tokens they come
// from; beside it the measures by their tokens, then the reflow in a window of the toolbar's size.

import { length } from "../../lib/tokens.ts";
import { settings, windowSize } from "../settings.ts";
import type { Measures } from "../page.ts";
import { el, label, mockCell, prose } from "../dom.ts";
import { partOr, subheading } from "./section.ts";

const overlays = new Set<() => boolean>();

/**
 * Lays every overlay on the page out again: after a change of scale, or of the window. Whether one
 * took another line of names above its sample, so the page's mocks are centred again.
 */
export function redrawOverlays(): boolean {
  let moved = false;
  for (const draw of overlays) moved = draw() || moved;
  return moved;
}

function role(element: Element): string {
  const probe = JSON.parse((element as HTMLElement).dataset.probe ?? "{}") as { role?: string };
  return probe.role ?? "box";
}

/** The token a custom property names, `var(--space-s24)` read as `space.s24`, or "". */
function tokenOf(element: Element, property: string): string {
  const value = (element as HTMLElement).style.getPropertyValue(property);
  const name = /var\(--([a-z0-9_]+)-([a-z0-9_-]+)\)/.exec(value);
  return name ? `${name[1]}.${name[2]!.replaceAll("-", ".")}` : "";
}

/** `sample` with its boxes outlined over it, in a box the overlay is laid out in. */
function overlaid(sample: Element): HTMLElement {
  const marks = el("div", "page-overlay");
  const holder = el("div", "page-measured", sample, marks);
  const draw = (): boolean => {
    marks.replaceChildren();
    if (!holder.isConnected) {
      overlays.delete(draw);
      return false;
    }
    const origin = holder.getBoundingClientRect();
    const zoom = holder.offsetWidth / origin.width || 1;
    const scale = settings().scale;
    const place = (mark: HTMLElement, left: number, top: number, width: number, height: number) => {
      mark.style.left = `${(left - origin.left) * zoom}px`;
      mark.style.top = `${(top - origin.top) * zoom}px`;
      mark.style.width = `${width * zoom}px`;
      mark.style.height = `${height * zoom}px`;
      marks.append(mark);
    };
    const names: HTMLElement[] = [];
    const band = (left: number, top: number, width: number, height: number, token: string, kind: string) => {
      const name = el("span", "page-band-name", token);
      const shade = el("div", "page-band", name);
      shade.dataset.kind = kind;
      place(shade, left, top, width, height);
      names.push(name);
    };
    for (const element of holder.querySelectorAll("[data-probe]")) {
      const box = element.getBoundingClientRect();
      const mark = el("div", "page-mark");
      mark.dataset.role = role(element);
      place(mark, box.left, box.top, box.width, box.height);
      if (mark.dataset.role === "component") {
        const height = tokenOf(element, "--height");
        const size = `${round((box.width * zoom) / scale)} × ${round((box.height * zoom) / scale)}${height ? ` · ${height}` : ""}`;
        mark.append(el("span", "page-mark-size", size));
        const content = element.querySelector<HTMLElement>(":scope > [data-layout]");
        const padding = tokenOf(element, "--padding");
        if (content && padding) {
          const inner = content.getBoundingClientRect();
          band(box.left, box.top, inner.left - box.left, box.height, padding, "padding-start");
          band(inner.right, box.top, box.right - inner.right, box.height, padding, "padding-end");
        }
      }
      if (mark.dataset.role === "layout") {
        const gap = tokenOf(element, "--gap");
        // A row's gaps span the component it is in, so their names line up with its padding's.
        const holder = element.parentElement?.closest<HTMLElement>("[data-probe]");
        const span = holder && role(holder) === "component" ? holder.getBoundingClientRect() : box;
        const kids = [...element.children].map((kid) => kid.getBoundingClientRect());
        for (let i = 1; gap && i < kids.length; i++) {
          const before = kids[i - 1]!;
          const after = kids[i]!;
          if (element.classList.contains("k-row")) band(before.right, span.top, after.left - before.right, span.height, gap, "gap");
          else band(box.left, before.bottom, box.width, after.top - before.bottom, gap, "gap");
        }
      }
    }
    // A line of names and the gap above it, in the viewport's pixels.
    const rise = ((length("type-mono-line") + length("space-s4")) * scale) / zoom;
    // And the room kept between two names on a line, so they read as two.
    const apart = (length("space-s8") * scale) / zoom;
    return stack(holder, names, rise, apart);
  };
  overlays.add(draw);
  return holder;
}

/**
 * Lifts each of `names` a line at a time until it comes no nearer than `apart` to a name before
 * it, paddings first and then gaps, so two names never meet however narrow the box; and gives `holder` room above for
 * the lines used. Whether that room changed.
 */
function stack(holder: HTMLElement, names: readonly HTMLElement[], rise: number, apart: number): boolean {
  const placed: DOMRect[] = [];
  let lines = 1;
  for (const name of names) {
    const box = name.getBoundingClientRect();
    let line = 0;
    const at = (n: number) => new DOMRect(box.left, box.top - n * rise, box.width, box.height);
    const meets = (a: DOMRect) => placed.some((b) => a.left < b.right + apart && b.left < a.right + apart && a.top < b.bottom && b.top < a.bottom);
    while (meets(at(line))) line++;
    name.style.setProperty("--line", String(line));
    placed.push(at(line));
    lines = Math.max(lines, line + 1);
  }
  const before = holder.style.getPropertyValue("--lines");
  holder.style.setProperty("--lines", String(lines));
  return before !== String(lines);
}

function round(value: number): string {
  return String(Math.round(value * 100) / 100);
}

export function measures(spec: Measures): HTMLElement {
  const samples = el("div", "page-variants");
  for (const sample of spec.samples) {
    samples.append(el("div", "page-cell", label(sample.name, "label")), mockCell(overlaid(sample.sample())), prose(sample.for));
  }
  const table = el("dl", "page-list");
  for (const measure of spec.measures) {
    const token = measure.token.replace(/^--/, "");
    const name = token.replaceAll("-", ".");
    table.append(el("dt", "", el("code", "page-code", `${name} = ${round(length(token))}`)), el("dd", "", prose(measure.what)));
  }
  const reflow = partOr(spec.reflow, (r) => {
    const size = windowSize();
    const frame = el("div", "page-window", r.sample());
    frame.style.width = size.width;
    frame.style.height = size.height;
    return el("div", "", prose(r.described), frame);
  });
  return el("div", "page-part", samples, subheading("By token"), table, subheading("Reflow"), reflow);
}
