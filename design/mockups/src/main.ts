// The mockup site: the top bar, the navigation and the rail around the page picked by the address,
// `#/LEVEL/PAGE`. `?chrome=off` shows the page alone, as the shots draw it.
//
// `window.kubaos` lets a script that drives the site (`scripts/render.ts`) wait until the page is
// drawn and read its geometry frame and its moves' recipes.

import "./tokens.css";
import "./fonts.css";
import "./motion.css";
import "./lib/lib.css";
import "./site/site.css";
import { forgetMasks } from "./lib/focus.ts";
import { forgetWidths, shareRows } from "./lib/text.ts";
import { forgetLengths } from "./lib/tokens.ts";
import { type Painted, paintPage, playChange } from "./site/change.ts";
import { forgetPalettes } from "./site/recolour.ts";
import { PAGES } from "./pages/index.ts";
import { el, RELAYOUT } from "./site/dom.ts";
import { type Frame, frame } from "./site/frame.ts";
import { type Nav, nav } from "./site/nav.ts";
import { type AnyPage, applies, isFoundation } from "./site/page.ts";
import { recipes, stopPlayers } from "./site/parts/motion.ts";
import { forgetRail, rail } from "./site/rail.ts";
import { render } from "./site/render.ts";
import { apply, CHROME, onChange, settings } from "./site/settings.ts";
import { holdWheel, releaseWheel } from "./site/scroll.ts";
import { settle } from "./site/settle.ts";
import { type Toolbar, toolbar } from "./site/toolbar.ts";

declare global {
  interface Window {
    kubaos: {
      ready: Promise<void>;
      frame(): Frame;
      motion(): Record<string, unknown> | null;
      pages: { level: string; slug: string }[];
    };
  }
}

const site = document.getElementById("site")!;
const pageBox = el("main", "page mock");
pageBox.id = "page";

function picked(): AnyPage | undefined {
  const [, level, slug] = location.hash.split("/");
  return PAGES.find((page) => page.level === level && page.slug === slug) ?? PAGES[0];
}

/**
 * Zooms the page's box out by the display's own ratio, so a CSS pixel inside it is one device
 * pixel, whatever the browser's zoom or the screen: hairlines and the grid land as in the builds.
 */
function fitDevicePixels(): void {
  pageBox.style.zoom = String(1 / devicePixelRatio);
  matchMedia(`(resolution: ${devicePixelRatio}dppx)`).addEventListener("change", refit, { once: true });
}

/** The display's ratio changed: the browser's zoom, or the window moved to another screen. */
function refit(): void {
  fitDevicePixels();
  draw();
}

/**
 * The top bar, and the navigation, kept as pages change so its search and its scroll stay, beside
 * the page and the rail of the page drawn now. Both are drawn once the fonts have loaded: a label's
 * width is measured from its font and kept, and one measured in the fallback font would leave its
 * box, and what is laid from it (a tab's lit stretch), short of the words drawn.
 */
let chrome: { readonly bar: Toolbar; readonly navigation: Nav; readonly body: HTMLElement } | null = null;
let shownRail: HTMLElement | null = null;

function drawChrome(): NonNullable<typeof chrome> {
  if (chrome) return chrome;
  const navigation = nav();
  chrome = { bar: toolbar(), navigation, body: el("div", "site-body", navigation.element) };
  return chrome;
}

/**
 * The top bar's height, which wraps onto more lines in a narrow window, kept in
 * `--site-toolbar-height` so the navigation sticks just under it, as tall as the window below it.
 */
const toolbarHeight = new ResizeObserver(([entry]) => {
  const height = entry?.borderBoxSize[0]?.blockSize ?? 0;
  document.documentElement.style.setProperty("--site-toolbar-height", `${height}px`);
  // The navigation is the window's height under the bar: its current row is brought into the room
  // it has now.
  chrome?.navigation.select(picked());
});

function draw(): void {
  const page = picked();
  apply(pageBox);
  stopPlayers();
  releaseWheel();
  // The page measures its samples as it draws them, so its box is in its place on the document
  // first: moved there after, the whole page would be styled and laid out again.
  if (CHROME) {
    const { bar, body } = drawChrome();
    if (pageBox.parentElement !== body) body.insertBefore(pageBox, shownRail);
    if (bar.element.parentElement !== site || body.parentElement !== site) site.replaceChildren(bar.element, body);
  } else if (pageBox.parentElement !== site || site.childElementCount !== 1) site.replaceChildren(pageBox);
  const drawn = page ? render(page) : null;
  pageBox.replaceChildren(drawn?.container ?? el("p", "page-prose", "No pages yet."));
  if (CHROME) {
    const { bar, navigation, body } = drawChrome();
    bar.windows(page !== undefined && !isFoundation(page) && applies(page.measures.reflow));
    const nextRail = rail(drawn?.sections ?? []);
    if (shownRail) shownRail.replaceWith(nextRail);
    else body.append(nextRail);
    shownRail = nextRail;
    navigation.select(page);
    toolbarHeight.disconnect();
    toolbarHeight.observe(bar.element);
  } else forgetRail();
  document.title = page ? `${page.title} · KubaOS mockups` : "KubaOS mockups";
  settle(pageBox);
  pageBox.dataset.page = page?.slug ?? "";
}

/** The theme and mode the page is painted in now, so a change knows where it starts. */
let painted: Painted = { theme: settings().theme, mode: settings().mode };

/**
 * Lines side by side share a baseline (`shareRows`): set once a frame after anything is drawn, so a
 * sample a player or a live part draws later shares it too.
 */
let sharing = 0;
const rowsDrawn = new MutationObserver(() => {
  if (sharing) return;
  sharing = requestAnimationFrame(() => {
    sharing = 0;
    shareRows(document);
  });
});

async function start(): Promise<void> {
  await Promise.all([...document.fonts].map((face) => face.load()));
  // Anything measured while the fonts loaded was measured in the fallback.
  forgetWidths();
  fitDevicePixels();
  rowsDrawn.observe(document.body, { childList: true, subtree: true });
  holdWheel();
  draw();
  paintPage([document.documentElement, pageBox], painted);
  addEventListener("hashchange", draw);
  addEventListener("resize", () => settle(pageBox));
  pageBox.addEventListener(RELAYOUT, () => settle(pageBox));
  // A change to the tokens reaches the page as new CSS; what script measured from the old CSS is
  // measured again.
  import.meta.hot?.on("vite:afterUpdate", () => {
    forgetWidths();
    forgetLengths();
    forgetMasks();
    forgetPalettes();
    draw();
    paintPage([document.documentElement, pageBox], painted);
  });
  onChange((now, changed) => {
    if (changed.has("theme") || changed.has("mode")) {
      const from = painted;
      painted = { theme: now.theme, mode: now.mode };
      // A page's samples are drawn for the mode they show in, its scenes in the theme's palettes.
      draw();
      playChange([document.documentElement, pageBox], from, painted);
      return;
    }
    if (changed.has("text") || changed.has("window")) {
      draw();
      return;
    }
    if (changed.has("both") || changed.has("scale")) {
      // The stage's panes follow Both, and the anatomy's values the scale.
      draw();
      return;
    }
    apply(pageBox);
    settle(pageBox);
  });
  await new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));
  settle(pageBox);
}

window.kubaos = {
  ready: start(),
  frame: () => frame(pageBox, picked()?.slug ?? "", settings().mode, settings().scale),
  motion: () => {
    const page = picked();
    return page && applies(page.motion) ? recipes(page.motion) : null;
  },
  pages: PAGES.map((page) => ({ level: page.level, slug: page.slug })),
};
