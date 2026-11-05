// What is laid out in script once the page's CSS has laid it out: the page's container's padding
// for its width, the sample grids' bands for it, the samples wider than it let out of it, the
// stage's samples fitted to their panes, each mock centred down its row on a whole device pixel,
// the focus rings' glows for their controls' sizes, then what the measures draw over their samples. CSS centres by half the spare
// room, which is half a device pixel whenever that is odd; a mock is placed as the builds place a
// child, the odd device pixel after it (`kubaos-ui`'s `centre`).

import { redrawRings } from "../lib/focus.ts";
import { raw } from "../lib/tokens.ts";
import { reflowBands } from "./parts/bands.ts";
import { redrawOverlays } from "./parts/measures.ts";
import { refitStages } from "./parts/stage.ts";
import { settings } from "./settings.ts";

/** Centres every mock cell's content on `page` down its row, on whole device pixels. */
function centreMocks(page: HTMLElement): void {
  // The page's CSS pixels are device pixels: its zoom shrinks them in the viewport's.
  const devicePerViewport = page.offsetWidth / page.getBoundingClientRect().width || 1;
  const mocks = [...page.querySelectorAll<HTMLElement>(".page-cell[data-mock]")]
    .map((cell) => ({ cell, mock: cell.firstElementChild as HTMLElement | null }))
    .filter((pair): pair is { cell: HTMLElement; mock: HTMLElement } => pair.mock !== null);
  // Every margin is cleared, then every height read, then every margin set: one layout, not one a
  // cell.
  for (const { mock } of mocks) mock.style.marginTop = "0px";
  const spares = mocks.map(
    ({ cell, mock }) => (cell.getBoundingClientRect().height - mock.getBoundingClientRect().height) * devicePerViewport,
  );
  mocks.forEach(({ mock }, i) => {
    mock.style.marginTop = `${Math.floor(Math.round(spares[i]!) / 2)}px`;
  });
}

/**
 * Gives `page`'s container, which fills the room between the navigation and the rail, its padding
 * for the width it takes in logical pixels: the padding of the widest step of `[container]` in the
 * tokens it reaches (`Tokens.containerPadding`).
 */
function fitContainer(page: HTMLElement): void {
  const container = page.querySelector<HTMLElement>(":scope > .page-container");
  if (!container) return;
  const width = container.offsetWidth / settings().scale;
  let step = 0;
  for (let i = 0; raw(`container-${i}-from`) !== ""; i++) {
    if (width >= Number(raw(`container-${i}-from`))) step = i;
  }
  container.style.setProperty("--page-padding", `var(--container-${step}-padding)`);
}

/** Puts every sample's box back in the container, to be measured there again. */
function keepIn(page: HTMLElement): void {
  for (const view of page.querySelectorAll<HTMLElement>(".page-view")) {
    view.style.width = "";
    view.style.marginLeft = "";
    view.removeAttribute("data-scrolls");
  }
}

/**
 * Lets each sample's box wider than the container out of it, centred on it as far as the page's
 * padding allows, on a whole device pixel; one wider than that scrolls inside its own box. The
 * page never scrolls sideways.
 */
function breakOut(page: HTMLElement): void {
  const container = page.querySelector<HTMLElement>(":scope > .page-container");
  if (!container) return;
  const views = [...page.querySelectorAll<HTMLElement>(".page-view")];
  // Every box is read, then every one placed: one layout, not one a box.
  const perViewport = page.offsetWidth / page.getBoundingClientRect().width || 1;
  const box = container.getBoundingClientRect();
  const padding = Number.parseFloat(getComputedStyle(container).paddingLeft) / perViewport;
  const left = box.left + padding;
  const right = box.right - padding;
  const pageBox = page.getBoundingClientRect();
  const roomLeft = pageBox.left + padding;
  const roomRight = pageBox.right - padding;
  // A sample's width is its own box's, not what overflows it: a lit control's glow reaches past it.
  const widthOf = (view: HTMLElement) => Math.max(0, ...[...view.children].map((child) => (child as HTMLElement).offsetWidth ?? 0));
  const placed = views
    .filter((view) => view.offsetParent !== null && widthOf(view) > view.clientWidth)
    .map((view) => {
      const natural = widthOf(view) / perViewport;
      const want = Math.min(natural, roomRight - roomLeft);
      const at = Math.min(Math.max((left + right) / 2 - want / 2, roomLeft), roomRight - want);
      return { view, want, by: at - view.getBoundingClientRect().left, scrolls: natural > roomRight - roomLeft };
    });
  for (const { view, want, by, scrolls } of placed) {
    view.style.width = `${Math.floor(want * perViewport)}px`;
    view.style.marginLeft = `${Math.round(by * perViewport)}px`;
    view.toggleAttribute("data-scrolls", scrolls);
  }
}

/** Lays out what script lays out on `page`, after its CSS has. */
export function settle(page: HTMLElement): void {
  fitContainer(page);
  keepIn(page);
  // A grid of bands is split for the container's room before a box is let out for what it holds.
  reflowBands(page);
  breakOut(page);
  refitStages();
  centreMocks(page);
  redrawRings();
  redrawOverlays();
}
