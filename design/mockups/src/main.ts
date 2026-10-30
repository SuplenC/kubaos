// The mockup site: the toolbar and the navigation around the page picked by the address,
// `#/LEVEL/PAGE`. `?chrome=off` shows the page alone, as the shots draw it.
//
// `window.kubaos` lets a script that drives the site (`scripts/render.ts`) wait until the page is
// drawn and read its geometry frame and its moves' recipes.

import "./tokens.css";
import "./fonts.css";
import "./motion.css";
import "./lib/lib.css";
import "./site/site.css";
import { forgetWidths } from "./lib/text.ts";
import { forgetLengths } from "./lib/tokens.ts";
import { PAGES } from "./pages/index.ts";
import { el } from "./site/dom.ts";
import { type Frame, frame } from "./site/frame.ts";
import { nav } from "./site/nav.ts";
import type { Page } from "./site/page.ts";
import { applies } from "./site/page.ts";
import { recipes, stopPlayers } from "./site/parts/motion.ts";
import { render } from "./site/render.ts";
import { apply, CHROME, onChange, settings } from "./site/settings.ts";
import { settle } from "./site/settle.ts";
import { toolbar, updateToolbar } from "./site/toolbar.ts";

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

function picked(): Page | undefined {
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
 * The toolbar's height, which wraps onto more lines in a narrow window, kept in
 * `--site-toolbar-height` so the navigation sticks just under it instead of sliding beneath it.
 */
const toolbarHeight = new ResizeObserver(([entry]) => {
  const height = entry?.borderBoxSize[0]?.blockSize ?? 0;
  document.documentElement.style.setProperty("--site-toolbar-height", `${height}px`);
});

function draw(): void {
  const page = picked();
  apply(pageBox);
  stopPlayers();
  pageBox.replaceChildren(...(page ? render(page) : [el("p", "page-prose", "No pages yet.")]));
  if (CHROME) {
    const hasWindow = page !== undefined && applies(page.measures.reflow);
    const bar = toolbar(hasWindow);
    site.replaceChildren(bar, el("div", "site-body", nav(page), pageBox));
    toolbarHeight.disconnect();
    toolbarHeight.observe(bar);
  } else {
    site.replaceChildren(pageBox);
  }
  document.title = page ? `${page.title} · KubaOS mockups` : "KubaOS mockups";
  settle(pageBox);
  pageBox.dataset.page = page?.slug ?? "";
}

async function start(): Promise<void> {
  await Promise.all([...document.fonts].map((face) => face.load()));
  fitDevicePixels();
  draw();
  addEventListener("hashchange", draw);
  addEventListener("resize", () => settle(pageBox));
  // A change to the tokens reaches the page as new CSS; what script measured from the old CSS is
  // measured again.
  import.meta.hot?.on("vite:afterUpdate", () => {
    forgetWidths();
    forgetLengths();
    draw();
  });
  onChange((_, changed) => {
    if (changed.has("text") || changed.has("window")) {
      draw();
      return;
    }
    apply(pageBox);
    const bar = site.querySelector(".site-toolbar");
    if (bar) updateToolbar(bar);
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
    return page ? recipes(page.motion) : null;
  },
  pages: PAGES.map((page) => ({ level: page.level, slug: page.slug })),
};
