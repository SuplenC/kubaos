// Holds the centring pass (`src/lib/centre.ts`) to the builds' rounding on small pages drawn in
// headless Chromium: each case lays a page out, runs the pass, and reads where the boxes landed.
//
//   node scripts/centre-test.ts
//
// It fails naming every case that disagrees. The browser is nixpkgs' Playwright Chromium
// (`PLAYWRIGHT_BROWSERS_PATH`, set by `nix develop`); the pass is the site's own source with its
// types stripped, so no build is needed.

import { readFileSync } from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright-core";
import { chromiumPath } from "./browser.ts";

interface Box {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

interface Case {
  /** What it checks, as a sentence. */
  readonly name: string;
  /** The page's content, in a 400 px wide page box with no margins; boxes read by `id`. */
  readonly html: string;
  /** What is wrong once the pass has run, or null. */
  readonly check: (page: Page) => Promise<string | null>;
}

const PROBE = `data-probe='{"role":"box"}'`;

/** `id`'s box, from the page box's corner. */
async function box(page: Page, id: string): Promise<Box> {
  return page.evaluate((at) => {
    const origin = document.getElementById("page")!.getBoundingClientRect();
    const r = document.getElementById(at)!.getBoundingClientRect();
    return { x: r.left - origin.left, y: r.top - origin.top, width: r.width, height: r.height };
  }, id);
}

async function centre(page: Page): Promise<void> {
  await page.evaluate(() => (globalThis as unknown as { centreOnPixels: (root: ParentNode) => void }).centreOnPixels(document));
}

/** What differs between `id`'s box and the one expected, or null. */
async function boxIs(page: Page, id: string, x: number, y: number, width: number, height: number): Promise<string | null> {
  const b = await box(page, id);
  const near = (a: number, e: number) => Math.abs(a - e) < 1e-3;
  return near(b.x, x) && near(b.y, y) && near(b.width, width) && near(b.height, height)
    ? null
    : `${id} is ${b.width}×${b.height} at (${b.x}, ${b.y}), not ${width}×${height} at (${x}, ${y})`;
}

const CASES: Case[] = [
  {
    name: "a box centred down a row with an odd spare pixel takes it after itself, the row's size kept",
    html: `<div id="row" style="display: flex; align-items: center; height: 50px">
      <span id="a" ${PROBE} style="width: 40px; height: 25px"></span>
      <span id="b" ${PROBE} style="width: 40px; height: 50px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return (await boxIs(page, "a", 0, 12, 40, 25)) ?? (await boxIs(page, "row", 0, 0, 400, 50));
    },
  },
  {
    name: "a box with an even spare stays where CSS centres it",
    html: `<div style="display: flex; align-items: center; height: 50px">
      <span id="a" ${PROBE} style="width: 40px; height: 24px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return boxIs(page, "a", 0, 13, 40, 24);
    },
  },
  {
    name: "a line centred along a row moves together, the odd pixel after its last box",
    html: `<div style="display: flex; justify-content: center; gap: 10px; width: 400px">
      <span id="a" ${PROBE} style="width: 21px; height: 20px"></span>
      <span id="b" ${PROBE} style="width: 40px; height: 20px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return (await boxIs(page, "a", 164, 0, 21, 20)) ?? (await boxIs(page, "b", 195, 0, 40, 20));
    },
  },
  {
    name: "a stack's child centred across it lands on a whole pixel, the odd one after it",
    html: `<div style="display: flex; flex-direction: column; align-items: center; width: 400px">
      <span id="a" ${PROBE} style="width: 75px; height: 20px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return boxIs(page, "a", 162, 0, 75, 20);
    },
  },
  {
    name: "a grid cell's content centred both ways lands on whole pixels",
    html: `<div style="display: grid; place-items: center; grid-template-columns: 100px; grid-template-rows: 50px">
      <span id="a" ${PROBE} style="width: 25px; height: 25px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return boxIs(page, "a", 37, 12, 25, 25);
    },
  },
  {
    name: "a box centred inside a box that is centred lands on whole pixels too",
    html: `<div style="display: flex; align-items: center; height: 50px">
      <span id="outer" ${PROBE} style="display: flex; align-items: center; width: 60px; height: 35px">
        <span id="inner" ${PROBE} style="width: 20px; height: 20px"></span>
      </span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return (await boxIs(page, "outer", 0, 7, 60, 35)) ?? (await boxIs(page, "inner", 0, 14, 20, 20));
    },
  },
  {
    name: "the pass run again leaves what it centred, and lets a box go once its spare is even",
    html: `<div id="row" style="display: flex; align-items: center; height: 50px">
      <span id="a" ${PROBE} style="width: 40px; height: 25px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      await centre(page);
      const twice = await boxIs(page, "a", 0, 12, 40, 25);
      if (twice) return `run twice: ${twice}`;
      await page.evaluate(() => {
        document.getElementById("row")!.style.height = "51px";
      });
      await centre(page);
      return boxIs(page, "a", 0, 13, 40, 25);
    },
  },
  {
    name: "a margin the CSS gives the box is kept under the pixel it takes",
    html: `<div style="display: flex; align-items: center; height: 50px">
      <span id="a" ${PROBE} style="width: 40px; height: 21px; margin-bottom: 4px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      const margin = await page.evaluate(() => getComputedStyle(document.getElementById("a")!).marginBottom);
      if (margin !== "5px") return `its margin is ${margin}, not 5px`;
      return boxIs(page, "a", 0, 12, 40, 21);
    },
  },
  {
    name: "a box placed at its start, not centred, is left between pixels for the check to find",
    html: `<div style="display: flex; align-items: flex-start; padding-top: 0.5px; height: 50px">
      <span id="a" ${PROBE} style="width: 40px; height: 25px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return boxIs(page, "a", 0, 0.5, 40, 25);
    },
  },
  {
    name: "a box drawn scaled is left where the CSS centres it",
    html: `<div style="scale: 0.5; transform-origin: 0 0; display: flex; align-items: center; height: 50px; width: 400px">
      <span id="a" ${PROBE} style="width: 40px; height: 25px"></span>
    </div>`,
    check: async (page) => {
      await centre(page);
      return boxIs(page, "a", 0, 6.25, 20, 12.5);
    },
  },
];

async function main(): Promise<void> {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const source = readFileSync(path.join(here, "../src/lib/centre.ts"), "utf8");
  // The pass as a plain script: `centreOnPixels` becomes a global the cases call.
  const pass = stripTypeScriptTypes(source).replace(/^export /gm, "");
  const browser = await chromium.launch({ executablePath: chromiumPath(), args: ["--disable-gpu"] });
  const failed: string[] = [];
  try {
    const page = await browser.newPage({ viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 });
    for (const each of CASES) {
      await page.setContent(`<!doctype html><body style="margin: 0"><div id="page" style="width: 400px">${each.html}</div></body>`);
      await page.addScriptTag({ content: `${pass}\nglobalThis.centreOnPixels = centreOnPixels;` });
      let wrong: string | null;
      try {
        wrong = await each.check(page);
      } catch (error) {
        wrong = error instanceof Error ? error.message : String(error);
      }
      if (wrong) failed.push(`${each.name}: ${wrong}`);
    }
  } finally {
    await browser.close();
  }
  for (const line of failed) process.stderr.write(`centre-test: ${line}\n`);
  if (failed.length > 0) {
    process.stderr.write(`centre-test: ${failed.length} of ${CASES.length} failed\n`);
    process.exit(1);
  }
  process.stdout.write(`centre-test: ${CASES.length} cases centred as the builds centre\n`);
}

main().catch((error: unknown) => {
  process.stderr.write(`centre-test: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
