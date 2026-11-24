// Captures states of one mockup page, each drawn alone (solo mode, `src/site/solo.ts`), as PNGs:
// the quick pictures of a draft (`README.md`, "Draft, then harden").
//
//   node scripts/shot.ts PAGE [STATE...] [--out DIR] [--scale 1] [--mode dark] [--theme SLUG]
//     [--text short] [--reduced] [--still] [--size WxH] [--at WxH] [--keys K,K] [--click CSS]
//     [--frames T,T] [--sheet] [--port N]
//
// PAGE is a slug, as `setup-yours`, or `LEVEL/SLUG`; each STATE a name or slug the page lists
// (`?solo#/LEVEL/SLUG` shows them); with none, every state of the page. Each is drawn at its own
// size, unless `--size` lays it out in a window of that size, which is then the shot; `--at` lays
// a whole screen out at W by H logical pixels. `--keys` (comma-separated, as Playwright names them:
// `Tab,ArrowDown,Enter`) and `--click` (a CSS selector) act on the state before its shot, in the
// order given, and can repeat. `--frames 0,120,240` shoots each state at those milliseconds after
// its actions instead of once it is still: the page runs on a clock of its own (`scripts/clock.ts`),
// so a frame is the same however long it takes to draw, and a move's state plays its loop from 0.
// `--sheet` also tiles every shot into one PNG, a row a state when there are frames. `--still`
// holds spinners, clocks and live samples, as the shots do (`?chrome=off`).
//
// OUT is `~/.cache/kubaos-mockups/shots/` unless given; each file is SLUG-STATE-MODE-SCALEx.png,
// with `-Tms` for a frame. It prints each path it writes, and every error the page throws or logs,
// and exits 1 on any. It captures from the dev server on `--port` or `$PORT` when one runs there,
// so an agent mid-work captures what its own server serves; otherwise it starts Vite on a free port
// from 5188 up and stops it after. Never 5174: that is the review server's.
//
// One headless Chromium (nixpkgs' Playwright's, `PLAYWRIGHT_BROWSERS_PATH`), several states at a
// time, each in a page of its own.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer as listen } from "node:net";
import { availableParallelism, homedir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { type Browser, type BrowserContext, chromium, type Page } from "playwright-core";
import { createServer, type ViteDevServer } from "vite";
import { chromiumPath, throughNode } from "./browser.ts";
import { installClock } from "./clock.ts";
import { findState, type StateName } from "../src/site/state-names.ts";

const FRAME = 1000 / 60;
/** The review server's port, which no script of an agent's may use. */
const REVIEW_PORT = 5174;
const FIRST_FREE_PORT = 5188;
/** The longest a state may move after its actions before it is shot anyway: an endless move never stops. */
const SETTLE_FRAMES = 300;
/** A sheet's tiles are at most this wide, in the sheet's pixels. */
const TILE = 640;
/**
 * A page the dev server reloaded while it was captured, as it does when a file it serves changes:
 * the capture is tried once more, on the files as they are now.
 */
const RELOADED = /Execution context was destroyed|navigat/;

type Action = { readonly keys: readonly string[] } | { readonly click: string };

interface Options {
  page: string;
  states: string[];
  out: string;
  scale: string;
  mode: string;
  theme: string | null;
  text: string | null;
  reduced: boolean;
  still: boolean;
  size: { width: number; height: number } | null;
  at: string | null;
  actions: Action[];
  frames: number[] | null;
  sheet: boolean;
  port: number | null;
}

type State = StateName;

interface Shot {
  readonly state: State;
  readonly frames: { readonly at: number | null; readonly file: string }[];
  readonly errors: string[];
}

function usage(): never {
  process.stderr.write(
    "usage: node scripts/shot.ts PAGE [STATE...] [--out DIR] [--scale 1] [--mode dark|light] [--theme SLUG] [--text short|long|translated]\n" +
      "         [--reduced] [--still] [--size WxH] [--at WxH] [--keys K,K] [--click CSS] [--frames T,T] [--sheet] [--port N]\n",
  );
  process.exit(2);
}

function size(value: string): { width: number; height: number } {
  const found = /^(\d+)x(\d+)$/.exec(value);
  if (!found) usage();
  return { width: Number(found[1]), height: Number(found[2]) };
}

function options(args: string[]): Options {
  const env = process.env.PORT;
  const found: Options = {
    page: "",
    states: [],
    out: path.join(homedir(), ".cache/kubaos-mockups/shots"),
    scale: "1",
    mode: "dark",
    theme: null,
    text: null,
    reduced: false,
    still: false,
    size: null,
    at: null,
    actions: [],
    frames: null,
    sheet: false,
    port: env ? Number(env) : null,
  };
  const value = () => args.shift() ?? usage();
  while (args.length > 0) {
    const arg = args.shift()!;
    if (arg === "--out") found.out = path.resolve(value());
    else if (arg === "--scale") found.scale = value();
    else if (arg === "--mode") found.mode = value();
    else if (arg === "--theme") found.theme = value();
    else if (arg === "--text") found.text = value();
    else if (arg === "--reduced") found.reduced = true;
    else if (arg === "--still") found.still = true;
    else if (arg === "--size") found.size = size(value());
    else if (arg === "--at") {
      const { width, height } = size(value());
      found.at = `${width}x${height}`;
    }
    else if (arg === "--keys") found.actions.push({ keys: value().split(",") });
    else if (arg === "--click") found.actions.push({ click: value() });
    else if (arg === "--frames") found.frames = value().split(",").map(Number).sort((a, b) => a - b);
    else if (arg === "--sheet") found.sheet = true;
    else if (arg === "--port") found.port = Number(value());
    else if (arg.startsWith("-")) usage();
    else if (!found.page) found.page = arg.replace(/^#?\//, "");
    else found.states.push(arg);
  }
  if (!found.page || !["dark", "light"].includes(found.mode)) usage();
  if (found.frames?.some((t) => !Number.isFinite(t) || t < 0)) usage();
  if (found.port !== null && !Number.isInteger(found.port)) usage();
  return found;
}

/** A port free on this machine, from 5188 up. */
async function freePort(): Promise<number> {
  for (let port = FIRST_FREE_PORT; port < FIRST_FREE_PORT + 100; port++) {
    const free = await new Promise<boolean>((done) => {
      const probe = listen();
      probe.once("error", () => done(false));
      probe.listen(port, "127.0.0.1", () => probe.close(() => done(true)));
    });
    if (free) return port;
  }
  throw new Error(`no free port from ${FIRST_FREE_PORT}`);
}

/** Whether a server answers on `origin`. */
async function answers(origin: string): Promise<boolean> {
  try {
    return (await fetch(origin, { signal: AbortSignal.timeout(2000) })).ok;
  } catch {
    return false;
  }
}

/** The site's origin: the server on `port` when one runs, or Vite started for the run. */
async function server(port: number | null): Promise<{ origin: string; vite: ViteDevServer | null }> {
  if (port === REVIEW_PORT) throw new Error(`${REVIEW_PORT} is the review server's: start your own, or leave --port out`);
  if (port !== null) {
    const origin = `http://127.0.0.1:${port}`;
    if (await answers(origin)) return { origin, vite: null };
    process.stderr.write(`shot: nothing answers on ${port}; starting Vite there\n`);
  }
  const at = port ?? (await freePort());
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const vite = await createServer({ root, logLevel: "error", server: { port: at, strictPort: true, hmr: false } });
  await vite.listen();
  return { origin: `http://127.0.0.1:${at}`, vite };
}

/** The address of `state` (or of the page's list, with "") at the run's settings. */
function address(origin: string, options: Options, level: string, slug: string, state: string): string {
  const query = new URLSearchParams({ solo: state, mode: options.mode, scale: options.scale });
  if (options.theme) query.set("theme", options.theme);
  if (options.text) query.set("text", options.text);
  if (options.reduced) query.set("reduced", "on");
  if (options.still) query.set("chrome", "off");
  if (options.at) query.set("at", options.at);
  return `${origin}/index.html?${query.toString()}#/${level}/${slug}`;
}

/** Moves the page's time a frame at a time until `done` holds, at most `limit` frames. */
async function pump(page: Page, done: () => Promise<boolean>, limit: number, what: string): Promise<void> {
  for (let i = 0; i < limit; i++) {
    if (await done()) return;
    await page.evaluate(() => window.pageClock.step());
  }
  throw new Error(`${what}: the page never got there`);
}

/** Opens `url` in a new page on the clock, and waits until the site has drawn it. */
async function open(context: BrowserContext, url: string, errors: string[]): Promise<Page> {
  const page = await context.newPage();
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await page.goto(url);
  // The site's modules load on real time, then draw the page on the page's.
  await pump(page, () => page.evaluate(() => "kubaos" in window), 3000, "loading");
  let ready = false;
  void page.evaluate(() => window.kubaos.ready).then(
    () => (ready = true),
    (error: unknown) => errors.push(String(error)),
  );
  await pump(page, async () => ready || errors.length > 0, 600, "drawing");
  return page;
}

/** The page's level and slug, and its states, from its list in solo mode. */
async function listed(context: BrowserContext, origin: string, options: Options): Promise<{ level: string; slug: string; states: State[] }> {
  const errors: string[] = [];
  const parts = options.page.split("/");
  const given = parts.length > 1 ? parts[0]! : "";
  const slug = parts.at(-1)!;
  const page = await open(context, address(origin, options, given, slug, ""), errors);
  try {
    const pages = await page.evaluate(() => window.kubaos.pages);
    const found = pages.find((each) => each.slug === slug && (!given || each.level === given));
    if (!found) throw new Error(`no page ${options.page}: one of ${pages.map((each) => `${each.level}/${each.slug}`).join(", ")}`);
    if (found.level !== given) {
      await page.evaluate((hash) => (location.hash = hash), `#/${found.level}/${found.slug}`);
      await pump(page, () => page.evaluate((want) => document.getElementById("page")?.dataset.page === want, found.slug), 60, "the page's list");
    }
    const states = await page.evaluate(() => window.kubaos.states());
    if (errors.length > 0) throw new Error(`the page failed: ${errors.join("; ")}`);
    return { level: found.level, slug: found.slug, states };
  } finally {
    await page.close();
  }
}

/** The states asked for, by name or slug, as solo mode finds them; every one for none. */
function picked(states: readonly State[], wanted: readonly string[]): State[] {
  if (wanted.length === 0) return [...states];
  const unknown = wanted.filter((want) => !findState(states, want));
  if (unknown.length > 0) throw new Error(`no state ${unknown.join(", ")}: one of ${states.map((each) => each.slug).join(", ")}`);
  return wanted.map((want) => findState(states, want)!);
}

/** The box shot: the state at its own size, or the whole window it was laid out in. */
async function box(page: Page, options: Options): Promise<{ x: number; y: number; width: number; height: number }> {
  if (options.size) return { x: 0, y: 0, ...options.size };
  const rect = await page.evaluate(() => {
    const drawn = document.getElementById("page")!.getBoundingClientRect();
    return { x: drawn.x, y: drawn.y, width: Math.ceil(drawn.width), height: Math.ceil(drawn.height) };
  });
  return rect;
}

async function shoot(page: Page, clip: { x: number; y: number; width: number; height: number }, file: string): Promise<void> {
  const cdp = await page.context().newCDPSession(page);
  // The clip is painted afresh rather than read from the compositor's last frame: just after the
  // window grows to the state's size, with other captures drawing beside it, that frame can hold
  // only part of the state, and a capture came back cut off at its right or its foot.
  const shot = await cdp.send("Page.captureScreenshot", { format: "png", clip: { ...clip, scale: 1 }, captureBeyondViewport: true });
  await cdp.detach();
  writeFileSync(file, Buffer.from(shot.data, "base64"));
}

/** Draws `state` alone, acts on it, and shoots it once still, or at each of the frames' times. */
async function capture(context: BrowserContext, origin: string, options: Options, level: string, slug: string, state: State): Promise<Shot> {
  const errors: string[] = [];
  const shot: Shot = { state, frames: [], errors };
  const page = await open(context, address(origin, options, level, slug, state.slug), errors);
  try {
    if (!options.size) {
      // The window grows to the state's own size, so all of it is in view and under the pointer.
      const { width, height } = await box(page, options);
      await page.setViewportSize({ width: Math.max(width, 1), height: Math.max(height, 1) });
      await page.evaluate(() => window.pageClock.step());
    }
    const quiet = () => page.evaluate(() => window.pageClock.quiet());
    for (let i = 0; i < SETTLE_FRAMES && !(await quiet()); i++) await page.evaluate(() => window.pageClock.step());
    for (const action of options.actions) {
      if ("keys" in action) for (const key of action.keys) await page.keyboard.press(key);
      else await page.click(action.click, { timeout: 5000 });
      await page.evaluate(() => window.pageClock.step());
    }
    const name = `${slug}-${state.slug}-${options.mode}-${options.scale}x`;
    if (options.frames) {
      let at = 0;
      for (const t of options.frames) {
        for (; at < Math.round(t / FRAME); at++) await page.evaluate(() => window.pageClock.step());
        const file = path.join(options.out, `${name}-${t}ms.png`);
        await shoot(page, await box(page, options), file);
        shot.frames.push({ at: t, file });
      }
    } else {
      for (let i = 0; i < SETTLE_FRAMES && !(await quiet()); i++) await page.evaluate(() => window.pageClock.step());
      const file = path.join(options.out, `${name}.png`);
      await shoot(page, await box(page, options), file);
      shot.frames.push({ at: null, file });
    }
  } catch (error) {
    errors.push(error instanceof Error ? error.message : String(error));
  } finally {
    await page.close();
  }
  return shot;
}

/** Every shot tiled into one PNG, a row a state when there are frames, each under its name. */
async function sheet(browser: Browser, shots: readonly Shot[], options: Options, slug: string): Promise<string> {
  const tiles = shots.flatMap((shot) =>
    shot.frames.map((frame) => ({
      words: frame.at === null ? shot.state.name : `${shot.state.name} · ${frame.at} ms`,
      src: `data:image/png;base64,${readFileSync(frame.file).toString("base64")}`,
    })),
  );
  const columns = options.frames ? options.frames.length : Math.ceil(Math.sqrt(tiles.length));
  const escaped = (words: string) => words.replaceAll("&", "&amp;").replaceAll("<", "&lt;");
  const html = `<!doctype html><style>
    body { margin: 0; padding: 16px; background: #111; color: #ddd; font: 14px sans-serif; width: max-content; }
    .grid { display: grid; grid-template-columns: repeat(${columns}, auto); gap: 16px; align-items: start; }
    figure { margin: 0; display: flex; flex-direction: column; gap: 6px; }
    img { max-width: ${TILE}px; height: auto; outline: 1px solid #333; }
  </style><div class="grid">${tiles.map((tile) => `<figure><figcaption>${escaped(tile.words)}</figcaption><img src="${tile.src}"></figure>`).join("")}</div>`;
  const context = await browser.newContext({ deviceScaleFactor: 1 });
  try {
    const page = await context.newPage();
    await page.setContent(html);
    await page.evaluate(() => Promise.all([...document.images].map((image) => image.decode())));
    const file = path.join(options.out, `${slug}-sheet-${options.mode}-${options.scale}x.png`);
    await page.screenshot({ path: file, fullPage: true });
    return file;
  } finally {
    await context.close();
  }
}

/** Runs `work` on each of `items`, `width` at a time, keeping their order. */
async function eachAtOnce<T, R>(items: readonly T[], width: number, work: (item: T) => Promise<R>): Promise<R[]> {
  const results: R[] = new Array(items.length);
  let next = 0;
  const lane = async () => {
    for (let i = next++; i < items.length; i = next++) results[i] = await work(items[i]!);
  };
  await Promise.all(Array.from({ length: Math.min(width, items.length) }, lane));
  return results;
}

async function main(): Promise<void> {
  const run = options(process.argv.slice(2));
  mkdirSync(run.out, { recursive: true });
  const { origin, vite } = await server(run.port);
  const browser = await chromium.launch({ executablePath: chromiumPath() });
  let failed = false;
  try {
    const context = await browser.newContext({
      viewport: run.size ?? { width: 1920, height: 1080 },
      deviceScaleFactor: 1,
      reducedMotion: run.reduced ? "reduce" : "no-preference",
      colorScheme: run.mode === "light" ? "light" : "dark",
    });
    await context.addInitScript(installClock, FRAME);
    await throughNode(context, origin);
    const { level, slug, states } = await listed(context, origin, run).catch((error: unknown) => {
      if (RELOADED.test(String(error))) return listed(context, origin, run);
      throw error;
    });
    const wanted = picked(states, run.states);
    const shots = await eachAtOnce(wanted, Math.max(1, Math.min(8, Math.floor(availableParallelism() / 4))), async (state) => {
      const shot = await capture(context, origin, run, level, slug, state);
      return shot.errors.some((error) => RELOADED.test(error)) ? capture(context, origin, run, level, slug, state) : shot;
    });
    for (const shot of shots) {
      for (const frame of shot.frames) process.stdout.write(`${frame.file}\n`);
      for (const error of shot.errors) process.stderr.write(`shot: ${slug} ${shot.state.slug}: ${error}\n`);
      failed ||= shot.errors.length > 0;
    }
    if (run.sheet && shots.some((shot) => shot.frames.length > 0)) process.stdout.write(`${await sheet(browser, shots, run, slug)}\n`);
  } finally {
    await browser.close();
    await vite?.close();
  }
  if (failed) process.exit(1);
}

main().catch((error: unknown) => {
  process.stderr.write(`shot: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
