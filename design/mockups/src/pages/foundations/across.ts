// What the foundations' tables share: a table as wide as the page, so its lines run across it and
// leave no empty room at its right.

/**
 * `scroll`, a table from `site/parts/swatches.ts`'s `table`, drawn across the room it has. Its last
 * column takes all the spare room: shared between the columns, it would put their edges between
 * device pixels.
 */
export function across(scroll: HTMLElement): HTMLElement {
  scroll.style.width = "100%";
  scroll.querySelector("table")?.style.setProperty("width", "100%");
  scroll.querySelector("th:last-child")?.setAttribute("style", "width: 100%");
  return scroll;
}
