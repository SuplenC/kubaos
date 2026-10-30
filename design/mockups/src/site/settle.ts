// What is laid out in script once the page's CSS has laid it out: the page's padding for its
// width, each mock centred down its row on a whole device pixel, the focus rings' glows for their
// controls' sizes, then the measures overlays drawn over the result. CSS centres by half the spare
// room, which is half a device pixel whenever that is odd; a mock is placed as the builds place a
// child, the odd device pixel after it (`kubaos-ui`'s `centre`).

import { redrawRings } from "../lib/focus.ts";
import { raw } from "../lib/tokens.ts";
import { redrawOverlays } from "./parts/measures.ts";
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
 * Gives `page` the container's padding at its width in logical pixels: the padding of the widest
 * step its width reaches (`[container]` in the tokens, `Tokens.containerPadding`).
 */
function padPage(page: HTMLElement): void {
  const width = page.offsetWidth / settings().scale;
  let step = 0;
  for (let i = 0; raw(`container-${i}-from`) !== ""; i++) {
    if (width >= Number(raw(`container-${i}-from`))) step = i;
  }
  page.style.setProperty("--page-padding", `var(--container-${step}-padding)`);
}

/** Lays out what script lays out on `page`, after its CSS has. */
export function settle(page: HTMLElement): void {
  padPage(page);
  centreMocks(page);
  redrawRings();
  // An overlay that took another line of names above its sample is taller: centre again.
  if (redrawOverlays()) {
    centreMocks(page);
    redrawOverlays();
  }
}
