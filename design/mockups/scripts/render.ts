// Renders the built mockup site (`dist/`) in headless Chromium: every page, or the PAGEs named, in
// light and dark at every scale, as PNG shots and geometry frames, or each page's motion recipes.
//
//   node scripts/render.ts OUT [--shots] [--frames] [--scales 1,1.5] [--modes dark,light] [PAGE...]
//   node scripts/render.ts OUT --motion [PAGE...]
//
// OUT gets PAGE-MODE-SCALEx.png and, for the geometry check, PAGE-MODE-SCALEx.json; with --shots
// or --frames, only those. With --motion it gets PAGE.json alone, every move's recipe on the page
// (`site/recipe.ts`), which hold the same in either mode and at any scale; the build writes them
// into `dist/motion/`. A page is drawn alone (`?chrome=off`), 1280 logical pixels wide as the
// specimen's are, at a device pixel ratio of 1, so the shot's pixels are the page's device pixels.
// The browser is nixpkgs' Playwright Chromium (`PLAYWRIGHT_BROWSERS_PATH`, set by `nix develop`
// and `just shots`). An error the page throws or logs fails the run.

import { createReadStream, existsSync, mkdirSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";

const WIDTH = 1280;
const SCALES = ["1", "1.25", "1.5", "2"];
const MODES = ["dark", "light"];
const TYPES: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".ttf": "font/ttf",
  ".svg": "image/svg+xml",
};

function usage(): never {
  process.stderr.write("usage: node scripts/render.ts OUT [--shots] [--frames] [--scales 1,1.5] [--modes dark,light] [--motion] [PAGE...]\n");
  process.exit(2);
}

function chromiumPath(): string {
  const browsers = process.env.PLAYWRIGHT_BROWSERS_PATH;
  if (!browsers) throw new Error("PLAYWRIGHT_BROWSERS_PATH is unset: run me in `nix develop`");
  const dir = readdirSync(browsers).find((name) => name.startsWith("chromium-"));
  if (!dir) throw new Error(`no Chromium in ${browsers}`);
  return path.join(browsers, dir, "chrome-linux64", "chrome");
}

/** Serves `root` on a free local port, for as long as the process runs. */
async function serve(root: string): Promise<string> {
  const server = createServer((request, response) => {
    const url = new URL(request.url ?? "/", "http://localhost");
    const file = path.join(root, decodeURIComponent(url.pathname === "/" ? "/index.html" : url.pathname));
    if (!file.startsWith(root) || !existsSync(file) || !statSync(file).isFile()) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { "content-type": TYPES[path.extname(file)] ?? "application/octet-stream" });
    createReadStream(file).pipe(response);
  });
  await new Promise<void>((done) => server.listen(0, "127.0.0.1", done));
  server.unref();
  return `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
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
    for (const mode of modes) {
      for (const scale of scales) {
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
        const failOnErrors = () => {
          if (errors.length > 0) throw new Error(`the page failed: ${errors.join("; ")}`);
        };
        const query = `chrome=off&mode=${mode}&scale=${scale}&text=short&reduced=off&speed=1&stutter=on&window=Default`;
        await page.goto(`${origin}/index.html?${query}`);
        await page.evaluate(() => window.kubaos.ready);
        failOnErrors();
        const pages = await page.evaluate(() => window.kubaos.pages);
        const unknown = named.filter((slug) => !pages.some((p) => p.slug === slug));
        if (unknown.length > 0) throw new Error(`no page ${unknown.join(", ")}: one of ${pages.map((p) => p.slug).join(", ")}`);
        for (const { level, slug } of pages.filter((p) => named.length === 0 || named.includes(p.slug))) {
          // A new hash draws the page again without loading it, so the wait is for that draw.
          await page.goto(`${origin}/index.html?${query}#/${level}/${slug}`);
          await page.evaluate(() => window.kubaos.ready);
          await page.waitForFunction((want) => document.getElementById("page")?.dataset.page === want, slug);
          failOnErrors();
          const name = `${slug}-${mode}-${scale}x`;
          if (shots) {
            await page.locator("#page").screenshot({ path: path.join(out, `${name}.png`), animations: "disabled" });
            process.stdout.write(`${path.join(out, `${name}.png`)}\n`);
          }
          if (frames) {
            const frame = await page.evaluate(() => window.kubaos.frame());
            writeFileSync(path.join(out, `${name}.json`), JSON.stringify(frame));
          }
          if (motion) {
            const recipes = await page.evaluate(() => window.kubaos.motion());
            writeFileSync(path.join(out, `${slug}.json`), `${JSON.stringify(recipes, null, 2)}\n`);
            process.stdout.write(`${path.join(out, `${slug}.json`)}\n`);
          }
        }
        await context.close();
      }
    }
  } finally {
    await browser.close();
  }
}

main().catch((error: unknown) => {
  process.stderr.write(`render: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
