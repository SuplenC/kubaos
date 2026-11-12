// The stage, which leads a page: the thing itself, live and large, in a night pane and a day pane
// side by side (or the one mode the top bar picks), under one bar of chips that drives both: Live,
// then each state the page shows, pinned on every row. A mono hint at the bar's end says how to try
// the live sample, or what the pinned state looks like. A sample wider than its pane is scaled down
// to fit it, and its pane says by how much; with `?all`, the panes stack and nothing is scaled.

import { el, holdScaled, label, relayout, sampleBox } from "../dom.ts";
import { fullScreenControl, isScreen } from "../full-screen.ts";
import { applies, type DoesntApply, type Focus, LOOK_NAMES, STATE_NAMES, type Stage, type State, type States } from "../page.ts";
import { ALL, type Mode, settings } from "../settings.ts";
import { picker } from "./picker.ts";
import { painted } from "./swatches.ts";

const MODE_NAMES: Record<Mode, string> = { dark: "Night", light: "Day" };

/** One chip of the bar: what each pane draws for it, and the hint it shows. */
interface Pick {
  readonly name: string;
  readonly hint: string;
  readonly draw: (mode: Mode) => Element[];
}

/**
 * A pane: its samples wrapping in `inner`, scaled by a transform, which lays nothing out, where one
 * is wider than the pane; `outer` takes the height they are drawn at; `scale` says by how much.
 */
interface Pane {
  readonly box: HTMLElement;
  readonly outer: HTMLElement;
  readonly inner: HTMLElement;
  readonly scale: HTMLElement;
  /** Shows the pick drawn on the whole screen, where it is a screen. */
  readonly full: HTMLButtonElement | null;
}

/**
 * How wide `inner`'s samples are, in its own pixels. A lit control's glow reaches past the control's
 * edge into the pane's padding, and a browser counts it as overflow, but it asks for no room: only
 * the boxes the samples lay out count. A box that clips keeps what is inside it to itself.
 */
function samplesWidth(inner: HTMLElement): number {
  if (inner.scrollWidth <= inner.clientWidth) return inner.scrollWidth;
  const box = inner.getBoundingClientRect();
  let right = box.left;
  const reach = (parent: Element): void => {
    for (const child of parent.children) {
      if (child.matches(".k-ring, .k-field-caret-glow")) continue;
      right = Math.max(right, child.getBoundingClientRect().right);
      if (getComputedStyle(child).overflowX === "visible") reach(child);
    }
  };
  reach(inner);
  // The page's zoom shrinks its CSS pixels in the viewport's.
  return Math.ceil(((right - box.left) * inner.offsetWidth) / box.width);
}

/** Every stage's fitting, again: after the page is laid out anew. */
const fitting = new Set<() => void>();

/** Fits every stage's samples to their panes again, as the page was laid out anew. */
export function refitStages(): void {
  for (const fit of fitting) fit();
}

/** The states' chips: each state shown, and Lit's signs-off look by night after Lit. */
function statePicks(states: States<State>, focus: Focus | DoesntApply): Pick[] {
  const off = applies(focus) && states.shown.includes("lit") ? focus : null;
  const pinned = (state: State) => (): Element[] => states.rows.map((row) => row.cell(state));
  return states.shown.flatMap((state): Pick[] => {
    const pick = { name: STATE_NAMES[state], hint: states.looks[state], draw: pinned(state) };
    if (!(off && state === "lit")) return [pick];
    // By day the signs are off, so the signs-off look is Lit's.
    const signsOff = (mode: Mode) => (mode === "light" ? pinned("lit")() : states.rows.map((_, index) => off.signsOff(index)));
    return [pick, { name: LOOK_NAMES.signsOff, hint: off.looks.signsOff, draw: signsOff }];
  });
}

/** The stage for `spec`; with `states`, the bar's chips pin each of them. */
export function stage(spec: Stage, states?: { readonly states: States<State>; readonly focus: Focus | DoesntApply }): HTMLElement {
  const picks: Pick[] = [{ name: "Live", hint: `live · ${spec.hint}`, draw: () => [spec.live()] }, ...(states ? statePicks(states.states, states.focus) : [])];
  const now = settings();
  const day = applies(spec.daylight);
  const modes: Mode[] = !day ? ["dark"] : ALL || now.both ? ["dark", "light"] : [now.mode];
  let shown = 0;
  const panes = modes.map((mode): Pane => {
    const inner = el("div", "page-stage-flow");
    const outer = el("div", "page-pane-fit", inner);
    const scale = el("span", "page-pane-scale");
    const full = fullScreenControl(() => ({ make: () => picks[shown]!.draw(mode)[0]!, mode }));
    const head = el("div", "page-pane-head", label(day ? MODE_NAMES[mode] : `${MODE_NAMES[mode]}, in either mode`, "label", "on_surface_variant"), scale, ...(full ? [full] : []));
    const box = sampleBox(painted(now.theme, mode, head, outer));
    box.classList.add("page-pane");
    box.dataset.paneMode = mode;
    return { box, outer, inner, scale, full };
  });
  const hint = el("div", "page-stage-hint");

  const show = (index: number): void => {
    const pick = picks[index]!;
    shown = index;
    panes.forEach((pane, i) => {
      const drawn = pick.draw(modes[i]!);
      pane.inner.replaceChildren(...drawn);
      if (pane.full) pane.full.hidden = !isScreen(drawn);
    });
    // A label is measured for its words, so each pick's hint is a label of its own.
    hint.replaceChildren(label(pick.hint, "mono", "on_surface_variant"));
    if (box.isConnected) relayout(box);
  };

  const fit = (): void => {
    if (!box.isConnected) {
      fitting.delete(fit);
      return;
    }
    for (const { outer, inner } of panes) {
      outer.style.height = "";
      inner.style.width = "";
      inner.style.transform = "";
    }
    // Every box is read, then every one sized: one layout, not one a box. The samples wrap in the
    // pane's room, so one wider than it is what overflows it.
    const read = panes.map(({ inner }) => ({ natural: samplesWidth(inner), room: inner.clientWidth }));
    panes.forEach(({ outer, inner, scale }, i) => {
      const { natural, room } = read[i]!;
      // `?all` draws every sample at its own size for the shots and the geometry frames, which
      // measure each box as drawn: one wider than its pane scrolls in it instead.
      const by = natural > room && room > 0 && !ALL ? room / natural : 1;
      scale.replaceChildren(...(by < 1 ? [label(`shown at ${Math.floor(by * 100)}%`, "label", "on_surface_variant")] : []));
      if (by < 1) {
        inner.style.width = `${natural}px`;
        inner.style.transform = `scale(${by})`;
      }
      holdScaled(outer, inner, by);
    });
  };
  fitting.add(fit);

  const chips = picker(
    picks.map((pick) => pick.name),
    0,
    show,
    "What the stage shows",
  );
  const bar = el("div", "page-stage-bar", chips.element, hint);
  const box = el(
    "div",
    "page-hero",
    el("div", "page-panes", ...panes.map((pane) => pane.box)),
    bar,
  );
  box.dataset.panes = String(panes.length);
  show(0);
  return box;
}
