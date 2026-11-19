// The stage, which leads a page: the thing itself, live and large, in a night pane and a day pane
// (or the one mode the top bar picks), under one bar of chips that drives both: Live, then each
// state the page shows, pinned on every row. A mono hint at the bar's end says how to try the live
// sample, or what the pinned state looks like. The panes stand side by side where the sample fits a
// half-width pane at its own size, and one under the other, each across the stage, where side by
// side would shrink it (`standPanes`). A sample wider than its pane is scaled down to fit it, and
// its pane says by how much; with `?all`, the panes stack and nothing is scaled. A live sample's own
// controls (its state switches, `stageControls`) are no part of what it mocks: each pane lifts them
// out of the sample into a row of their own above it, drawn at the site's size whatever the scale.

import { el, holdScaled, label, relayout, sampleBox } from "../dom.ts";
import { fullScreenControl, isScreen } from "../full-screen.ts";
import { applies, type DoesntApply, type Focus, LOOK_NAMES, STATE_NAMES, type Stage, type State, type States } from "../page.ts";
import { ALL, type Mode, settings } from "../settings.ts";
import { picker } from "./picker.ts";
import { painted } from "./swatches.ts";

const MODE_NAMES: Record<Mode, string> = { dark: "Night", light: "Day" };

/**
 * Marks `element`, part of a live sample, as the sample's own control (a switch between the states
 * it shows): the stage lifts it out of the sample into its pane's controls row, so it is drawn at
 * the site's size and never scaled with the sample or read as part of what it mocks. It is moved,
 * not copied, so it keeps its listeners.
 */
export function stageControls<E extends HTMLElement>(element: E): E {
  element.dataset.stageControls = "";
  return element;
}

/** One chip of the bar: what each pane draws for it, and the hint it shows. */
export interface Pick {
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
  /** The row the sample's own controls are lifted into, above the scaled sample. */
  readonly controls: HTMLElement;
  readonly outer: HTMLElement;
  readonly inner: HTMLElement;
  readonly scale: HTMLElement;
  /** Shows the pick drawn on the whole screen, where it is a screen. */
  readonly full: HTMLButtonElement | null;
}

/** What a lit control draws past its own edge: a glow asks for no room. */
const GLOWS = ".k-ring, .k-field-caret-glow";

/**
 * The right edge of the boxes `parent`'s children lay out, in the viewport's pixels, `from` at
 * least. A lit control's glow reaches past the control's edge into the pane's padding, and a browser
 * counts it as overflow, but it asks for no room: only the boxes the samples lay out count. A box
 * that clips keeps what is inside it to itself.
 */
function reach(parent: Element, from: number): number {
  let right = from;
  for (const child of parent.children) {
    if (child.matches(GLOWS)) continue;
    right = Math.max(right, child.getBoundingClientRect().right);
    if (getComputedStyle(child).overflowX === "visible") right = reach(child, right);
  }
  return right;
}

/** How wide `inner`'s samples are, as they wrap in its room, in its own pixels. */
function samplesWidth(inner: HTMLElement): number {
  if (inner.scrollWidth <= inner.clientWidth) return inner.scrollWidth;
  const box = inner.getBoundingClientRect();
  // The page's zoom shrinks its CSS pixels in the viewport's.
  return Math.ceil(((reach(inner, box.left) - box.left) * inner.offsetWidth) / box.width);
}

/**
 * How wide the widest of `inner`'s samples must be to lay out without overflowing, in its own
 * pixels: each at its min-content width, `inner` as narrow as they allow. A sample keeps its own
 * width (a screen's, a toast's) and its least (an app's window's), while one that wraps (a row of
 * icons, cards that fall under each other) or fills its room asks only for its widest piece that
 * cannot wrap. It is the same whatever room the pane has. `inner` is held at it for the one layout
 * that reads it.
 */
function widestSample(inner: HTMLElement): number {
  inner.style.width = "min-content";
  const box = inner.getBoundingClientRect();
  let widest = 0;
  for (const sample of inner.children) {
    if (sample.matches(GLOWS)) continue;
    const own = sample.getBoundingClientRect();
    const right = getComputedStyle(sample).overflowX === "visible" ? reach(sample, own.right) : own.right;
    widest = Math.max(widest, right - own.left);
  }
  const width = box.width > 0 ? Math.ceil((widest * inner.offsetWidth) / box.width) : 0;
  inner.style.width = "";
  return width;
}

/** Every stage's fitting, again: after the page is laid out anew. */
const fitting = new Set<() => void>();

/**
 * The least scale side by side may show a sample at and still stand the panes so: a little under
 * its size still reads as its own size, and a sample somewhat wider than a half-width pane is not
 * worth the stage's whole height twice over, each pane half empty.
 */
const SIDE_BY_SIDE_LEAST = 0.85;

/** The scale that fits a sample `natural` wide into `room`, 1 where it fits as it is. */
function fitScale(natural: number, room: number): number {
  return natural > room && room > 0 ? room / natural : 1;
}

/**
 * How each pane's samples measure, every box read before any is sized: one layout, not one a box.
 * The samples wrap in the pane's room, so one wider than it is what overflows it.
 */
function measure(panes: readonly Pane[]): { natural: number; room: number }[] {
  return panes.map(({ inner }) => ({ natural: samplesWidth(inner), room: inner.clientWidth }));
}

/**
 * Stands `box`'s two panes side by side where each sample fits a half-width pane at its own size
 * (`widestSample`, down to `SIDE_BY_SIDE_LEAST`), else one under the other, each across the stage,
 * so a wide sample is drawn about twice as large. A sample's own size does not depend on the room it
 * is given, so a width near the line gives one answer whichever way the panes stand now, and the
 * stage never flickers between the two while a window is resized. Returns how the panes measure as
 * they stand.
 */
function standPanes(box: HTMLElement, panes: readonly Pane[], stacked: boolean): { natural: number; room: number }[] {
  // Side by side, a pane is half the row the panes stand in, less the same padding; read before
  // any pane is measured, as measuring lays them out again.
  const halves = panes.map(({ box: pane, inner }) => (pane.parentElement?.clientWidth ?? 0) / 2 - (pane.clientWidth - inner.clientWidth));
  const fits = !stacked && panes.every(({ inner }, i) => fitScale(widestSample(inner), halves[i]!) >= SIDE_BY_SIDE_LEAST);
  box.toggleAttribute("data-stacked", !fits);
  return measure(panes);
}

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

/** What the stage's chips pin, in the bar's order: Live, then each state `states` shows. */
export function stagePicks(spec: Stage, states?: { readonly states: States<State>; readonly focus: Focus | DoesntApply }): Pick[] {
  return [{ name: "Live", hint: `live · ${spec.hint}`, draw: () => [spec.live()] }, ...(states ? statePicks(states.states, states.focus) : [])];
}

/** The stage for `spec`; with `states`, the bar's chips pin each of them. */
export function stage(spec: Stage, states?: { readonly states: States<State>; readonly focus: Focus | DoesntApply }): HTMLElement {
  const picks = stagePicks(spec, states);
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
    const controls = el("div", "page-pane-controls");
    const box = sampleBox(painted(now.theme, mode, head, controls, outer));
    box.classList.add("page-pane");
    box.dataset.paneMode = mode;
    return { box, controls, outer, inner, scale, full };
  });
  const hint = el("div", "page-stage-hint");

  const show = (index: number): void => {
    const pick = picks[index]!;
    shown = index;
    resized.disconnect();
    panes.forEach((pane, i) => {
      const drawn = pick.draw(modes[i]!);
      // The sample's own controls leave it for the pane's row above it, where nothing scales them.
      const lifted = drawn.flatMap((sample) => [...(sample.matches("[data-stage-controls]") ? [sample] : []), ...sample.querySelectorAll<HTMLElement>("[data-stage-controls]")]);
      pane.controls.replaceChildren(...lifted);
      pane.inner.replaceChildren(...drawn.filter((sample) => !lifted.includes(sample as HTMLElement)));
      for (const sample of drawn) resized.observe(sample);
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
    // `?all` stacks every stage's panes already, and the shots keep its corners; one pane spans the
    // stage whatever it holds.
    const read = ALL || panes.length === 1 ? measure(panes) : standPanes(box, panes, spec.stacked === true);
    panes.forEach(({ outer, inner, scale }, i) => {
      const { natural, room } = read[i]!;
      // `?all` draws every sample at its own size for the shots and the geometry frames, which
      // measure each box as drawn: one wider than its pane scrolls in it instead.
      const by = ALL ? 1 : fitScale(natural, room);
      scale.replaceChildren(...(by < 1 ? [label(`shown at ${Math.floor(by * 100)}%`, "label", "on_surface_variant")] : []));
      if (by < 1) {
        inner.style.width = `${natural}px`;
        inner.style.transform = `scale(${by})`;
      }
      holdScaled(outer, inner, by);
    });
  };
  fitting.add(fit);
  // A sample that changes its own width (a slider sets it, a row opens) may stand the panes anew.
  // The panes are stood by each sample's min-content width, which the room it is given does not
  // change, so a sample that only fills its room leaves the answer as it was and the fit settles
  // after one. It runs on the next frame: fitting inside the observer would resize what it watches
  // in the same frame.
  const widths = new WeakMap<Element, number>();
  let refit = 0;
  const resized = new ResizeObserver((entries) => {
    for (const { target, borderBoxSize } of entries) {
      const width = borderBoxSize[0]?.inlineSize ?? 0;
      const before = widths.get(target);
      widths.set(target, width);
      if (before !== undefined && before !== width && refit === 0) {
        refit = requestAnimationFrame(() => {
          refit = 0;
          fit();
        });
      }
    }
  });

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
