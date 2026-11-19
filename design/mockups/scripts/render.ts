// Renders the built mockup site (`dist/`) in headless Chromium: every page, or the PAGEs named, in
// light and dark at every scale, as PNG shots and geometry frames, or each page's motion recipes.
//
//   node scripts/render.ts OUT [--shots] [--frames] [--scales 1,1.5] [--modes dark,light] [PAGE...]
//   node scripts/render.ts OUT --motion [PAGE...]
//
// OUT gets PAGE-MODE-SCALEx.png and, for the geometry check, PAGE-MODE-SCALEx.json; with --shots
// or --frames, only those. With --motion it gets PAGE.json alone, every move's recipe on the page
// (`site/recipe.ts`), which hold the same in either mode and at any scale; the build writes them
// into `dist/motion/`. A page is drawn alone (`?chrome=off`) with every sample its stages and
// motion lists hold (`?all`), 1280 logical pixels wide as the specimen's are, at a device pixel ratio of 1, so the shot's pixels are the page's device pixels.
// The browser is nixpkgs' Playwright Chromium (`PLAYWRIGHT_BROWSERS_PATH`, set by `nix develop`
// and `just shots`). An error the page throws or logs fails the run.
//
// It draws `MOCKUPS_JOBS` pages at once, one per CPU unless set, each worker in a browser
// context of its own; what it prints keeps the order of modes, scales and pages, whatever
// finishes first.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { type Browser, chromium, type Page } from "playwright-core";
import { chromiumPath, inOrder, jobs, serve } from "./browser.ts";

const WIDTH = 1280;
const SCALES = ["1", "1.25", "1.5", "2"];
const MODES = ["dark", "light"];

function usage(): never {
  process.stderr.write("usage: node scripts/render.ts OUT [--shots] [--frames] [--scales 1,1.5] [--modes dark,light] [--motion] [PAGE...]\n");
  process.exit(2);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const out = args.shift();
  if (!out || out.startsWith("-")) usage();
  let shots = true;
  let frames = true;
  let motion = false;
  let scales = SCALES;
  let modes = MODES;
  const named: string[] = [];
  while (args.length > 0) {
    const arg = args.shift()!;
    if (arg === "--shots") frames = false;
    else if (arg === "--frames") shots = false;
    else if (arg === "--motion") motion = true;
    else if (arg === "--scales") scales = (args.shift() ?? usage()).split(",");
    else if (arg === "--modes") modes = (args.shift() ?? usage()).split(",");
    else if (arg.startsWith("-")) usage();
    else named.push(arg);
  }
  const dist = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../dist");
  if (!existsSync(path.join(dist, "index.html"))) throw new Error(`no built site in ${dist}: run \`just mockups-build\``);
  mkdirSync(out, { recursive: true });
  if (motion) {
    shots = false;
    frames = false;
    scales = ["1"];
    modes = ["dark"];
  }
  const origin = await serve(dist);
  const browser = await chromium.launch({ executablePath: chromiumPath() });
  try {
    const settings = modes.flatMap((mode) => scales.map((scale) => ({ mode, scale })));
    // A worker keeps one page, loaded at one mode and scale, for as long as its items stay there.
    const tabs: (Tab | undefined)[] = [];
    const tab = async (worker: number, mode: string, scale: string): Promise<Tab> => {
      const kept = tabs[worker];
      if (kept?.mode === mode && kept.scale === scale) return kept;
      await kept?.page.context().close();
      tabs[worker] = await open(browser, origin, mode, scale);
      return tabs[worker]!;
    };
    const first = await tab(0, settings[0]!.mode, settings[0]!.scale);
    const pages = await first.page.evaluate(() => window.kubaos.pages);
    const unknown = named.filter((slug) => !pages.some((p) => p.slug === slug));
    if (unknown.length > 0) throw new Error(`no page ${unknown.join(", ")}: one of ${pages.map((p) => p.slug).join(", ")}`);
    const chosen = pages.filter((p) => named.length === 0 || named.includes(p.slug));
    const items = settings.flatMap(({ mode, scale }) => chosen.map(({ level, slug }) => ({ mode, scale, level, slug })));
    await inOrder(
      items,
      jobs(),
      async ({ mode, scale, level, slug }, worker) => {
        const { page, query, errors } = await tab(worker, mode, scale);
        const printed: string[] = [];
        // A new hash draws the page again without loading it, so the wait is for that draw.
        await page.goto(`${origin}/index.html?${query}#/${level}/${slug}`);
        await page.evaluate(() => window.kubaos.ready);
        await page.waitForFunction((want) => document.getElementById("page")?.dataset.page === want, slug);
        await pictures(page);
        failOnErrors(errors);
        const name = `${slug}-${mode}-${scale}x`;
        if (shots) {
          await page.locator("#page").screenshot({ path: path.join(out, `${name}.png`), animations: "disabled" });
          printed.push(path.join(out, `${name}.png`));
        }
        if (frames) {
          const frame = await page.evaluate(() => window.kubaos.frame());
          writeFileSync(path.join(out, `${name}.json`), JSON.stringify(frame));
        }
        if (motion) {
          const recipes = await page.evaluate(() => window.kubaos.motion());
          writeFileSync(path.join(out, `${slug}.json`), `${JSON.stringify(recipes, null, 2)}\n`);
          printed.push(path.join(out, `${slug}.json`));
        }
        return printed;
      },
      (printed) => {
        for (const line of printed) process.stdout.write(`${line}\n`);
      },
      // Loading the site at another mode or scale takes longer than drawing most pages.
      ({ mode, scale }) => `${mode} ${scale}`,
    );
  } finally {
    await browser.close();
  }
}

/** A worker's page, with the errors it has thrown or logged since its last item. */
interface Tab {
  readonly mode: string;
  readonly scale: string;
  readonly query: string;
  readonly page: Page;
  readonly errors: string[];
}

/** Opens a page of its own context at `mode` and `scale`, loaded and ready. */
async function open(browser: Browser, origin: string, mode: string, scale: string): Promise<Tab> {
  const context = await browser.newContext({
    viewport: { width: Math.round(WIDTH * Number(scale)), height: 800 },
    deviceScaleFactor: 1,
    reducedMotion: "no-preference",
  });
  const page = await context.newPage();
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  // `all` draws every sample a stage or a motion list shows one at a time.
  const query = `chrome=off&all&mode=${mode}&scale=${scale}&text=short&reduced=off&speed=1&flicker=on&window=Default`;
  await page.goto(`${origin}/index.html?${query}`);
  await page.evaluate(() => window.kubaos.ready);
  failOnErrors(errors);
  return { mode, scale, query, page, errors };
}

/**
 * Waits for every picture on the page to load, and each icon tile's to be measured and placed or
 * taken away. A tile shows its symbol until then, after the draw the page says is done, so a page
 * drawn while the CPU is busy would otherwise be measured with the symbol, and a quiet one without.
 */
async function pictures(page: Page): Promise<void> {
  await page.waitForFunction(() =>
    [...document.querySelectorAll<HTMLImageElement>("#page img[src]")].every(
      (picture) => picture.complete && (!picture.matches(".k-icon-tile > img") || picture.hasAttribute("data-loaded")),
    ),
  );
}

function failOnErrors(errors: string[]): void {
  if (errors.length > 0) throw new Error(`the page failed: ${errors.join("; ")}`);
}

main().catch((error: unknown) => {
  process.stderr.write(`render: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
