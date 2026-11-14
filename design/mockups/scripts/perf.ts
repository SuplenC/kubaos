// Measures how heavy the mockup pages named are: their load, their rest, a scroll through them and
// a press of their stage's live sample, each held to `scripts/perf-budgets.ts`.
//
//   npm run perf -- PAGE [PAGE...]
//
// It builds the site once, unminified so a culprit is named by its function and module, into
// `~/.cache/kubaos-mockups-perf/site`, serves it with `vite preview` on port 5184, and loads each
// page as a reader sees it, with the top bar and the navigation: `?chrome=off`, the shots' view,
// holds every player, spinner and live sample still. One headless Chromium drawing in software
// loads one page and one run at a time, three runs a page, each in a fresh context under a trace
// with the V8 sampler on. It prints the median of the runs per measure beside its budget, the
// culprits behind a measure past it, and the machine's load average; it exits 1 when a budget is
// broken. It checks only the pages named, never the whole site, and closes the browser and the
// server whatever fails.
//
// The browser is nixpkgs' Playwright Chromium (`PLAYWRIGHT_BROWSERS_PATH`, set by `nix develop`).

import { mkdirSync } from "node:fs";
import { loadavg } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { type Browser, chromium, type Page } from "playwright-core";
import { build, preview } from "vite";
import { chromiumPath } from "./browser.ts";
import { budgetFor, MEASURES, type Measure } from "./perf-budgets.ts";
import { type Frame, Trace } from "./perf-trace.ts";

declare global {
  interface Window {
    /** When the page's `kubaos.ready` settled, in the page's milliseconds since navigation. */
    perfReady?: number;
  }
}

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "..");
const SITE = path.join(process.env.HOME ?? ROOT, ".cache/kubaos-mockups-perf/site");
const PORT = 5184;
const ORIGIN = `http://127.0.0.1:${PORT}`;
const RUNS = 3;
const VIEWPORT = { width: 1280, height: 800 };
/** A wheel notch's travel, as a mouse wheel sends it. */
const NOTCH = 120;
/** At most this many notches: a longer page takes longer strides through the same steps. */
const MOST_NOTCHES = 150;
const LONG_TASK = 50;
const FRAME = 1000 / 60;
const REST_WAIT = 1000;
const REST = 2000;
const PRESS = 1000;

// `v8.execute` gives code run from a microtask or an observer a script event above it, so the
// layouts it forces are told from the frame's own.
const CATEGORIES = [
  "devtools.timeline",
  "disabled-by-default-devtools.timeline",
  "disabled-by-default-devtools.timeline.stack",
  "disabled-by-default-v8.cpu_profiler",
  "blink.user_timing",
  "v8.execute",
  "toplevel",
];

/** What a measure's value was in a run, and what the reader would act on when it is too high. */
interface Reading {
  readonly value: number;
  readonly culprits?: readonly string[];
}

type Run = Partial<Record<Measure, Reading>> & { notes: string[] };

function usage(): never {
  process.stderr.write("usage: npm run perf -- PAGE [PAGE...]\n  PAGE is a page's slug, as in `#/atoms/button`: button\n");
  process.exit(2);
}

function load(): string {
  return loadavg()
    .map((value) => value.toFixed(2))
    .join(" ");
}

function median(values: readonly number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)]!;
}

function mark(page: Page, name: string): Promise<void> {
  return page.evaluate((label) => console.timeStamp(label), name);
}

/** Marks when the site's `ready` settles, before any of its scripts run. */
function watchReady(): void {
  let site: Window["kubaos"] | undefined;
  Object.defineProperty(window, "kubaos", {
    configurable: true,
    get: () => site,
    set(value: Window["kubaos"]) {
      site = value;
      void value.ready.then(() => {
        window.perfReady = performance.now();
        console.timeStamp("perf:ready");
      });
    },
  });
}

/** The animations playing on the page, each as its element and what it animates. */
function runningAnimations(): string[] {
  return document
    .getAnimations()
    .filter((animation) => animation.playState === "running")
    .map((animation) => {
      const target = animation.effect instanceof KeyframeEffect ? animation.effect.target : null;
      const where = target ? `${target.tagName.toLowerCase()}${[...target.classList].map((name) => `.${name}`).join("")}` : "(no element)";
      const what = animation instanceof CSSAnimation ? animation.animationName : animation instanceof CSSTransition ? `transition ${animation.transitionProperty}` : (animation.id || "script");
      return `${where} ${what}`;
    });
}

/** How far the page scrolls, and the middle of its box in the window, where the wheel turns. */
function pageScroll(): { height: number; x: number; y: number } {
  const box = document.getElementById("page")!;
  let height = 0;
  for (let at: Element | null = box; at; at = at.parentElement) height = Math.max(height, at.scrollHeight - at.clientHeight);
  const scroller = document.scrollingElement;
  if (scroller) height = Math.max(height, scroller.scrollHeight - innerHeight);
  const rect = box.getBoundingClientRect();
  return { height, x: rect.left + rect.width / 2, y: (Math.max(0, rect.top) + Math.min(innerHeight, rect.bottom)) / 2 };
}

/**
 * Marks the first control in the stage's first live pane for a press, and says what it is; null
 * when the sample has none. A control, by its role, before anything merely focusable, and never
 * a link, which would leave the page.
 */
function markPressTarget(): string | null {
  const flow = document.querySelector(".page-pane .page-stage-flow");
  if (!flow) return null;
  const visible = (element: Element): boolean => element.getClientRects().length > 0 && !element.closest("a[href], [inert], [disabled], [aria-disabled=true]");
  const groups = [
    "button, [role=button], [role=switch], [role=checkbox], [role=radio], [role=tab], [role=option], [role=menuitem], [role=menuitemcheckbox], [role=slider]",
    "input, select, textarea",
    "[tabindex]:not([tabindex='-1'])",
  ];
  for (const selector of groups) {
    const target = [...flow.querySelectorAll<HTMLElement>(selector)].find(visible);
    if (!target) continue;
    target.dataset.perfTarget = "";
    const name = (target.getAttribute("aria-label") ?? target.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 30);
    return `${target.tagName.toLowerCase()}${target.getAttribute("role") ? `[role=${target.getAttribute("role")}]` : ""} "${name}"`;
  }
  return null;
}

function framesOver(frames: readonly Frame[]): number {
  return frames.filter((frame) => frame.busy > FRAME).length;
}

/** The long tasks in a window, each named by what it did and the code that ran longest in it. */
function longTasks(trace: Trace, from: number, to: number): { count: number; time: number; culprits: string[] } {
  const tasks = trace.topTasks(from, to).filter((task) => (task.end - task.start) / 1000 > LONG_TASK);
  const culprits = [...tasks]
    .sort((a, b) => b.end - b.start - (a.end - a.start))
    .slice(0, 3)
    .map((task) => `${((task.end - task.start) / 1000).toFixed(0)} ms task (${trace.breakdown(task)}): ${trace.hottest(task.start, task.end, 2).join("; ") || "no script sampled"}`);
  return { count: tasks.length, time: tasks.reduce((sum, task) => sum + (task.end - task.start) / 1000, 0), culprits };
}

/** What a run saw in the page itself, beside its trace. */
interface Drove {
  readonly readyAt: number;
  readonly elements: number;
  readonly animations: readonly string[];
  readonly notches: number;
  readonly stride: number;
  /** The control pressed, or null when the stage's sample has none. */
  readonly target: string | null;
}

/**
 * Loads the page at `url`, leaves it at rest, scrolls it through and presses its sample's first
 * control, marking each phase on the trace.
 */
async function drive(page: Page, url: string, slug: string): Promise<Drove> {
  await page.goto(url);
  await page.waitForFunction(() => window.perfReady !== undefined);
  await page.waitForFunction((want) => document.getElementById("page")?.dataset.page === want, slug);
  const readyAt = (await page.evaluate(() => window.perfReady))!;
  const elements = await page.evaluate(() => document.getElementsByTagName("*").length);

  await page.waitForTimeout(REST_WAIT);
  await mark(page, "perf:rest");
  await page.waitForTimeout(REST);
  await mark(page, "perf:rest-end");
  const animations = await page.evaluate(runningAnimations);

  const { height, x, y } = await page.evaluate(pageScroll);
  const notches = Math.min(MOST_NOTCHES, Math.ceil(height / NOTCH));
  const stride = notches > 0 ? Math.ceil(height / notches) : 0;
  await page.mouse.move(x, y);
  await mark(page, "perf:scroll");
  for (let i = 0; i < notches; i++) {
    await page.mouse.wheel(0, stride);
    await page.evaluate(() => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))));
  }
  await mark(page, "perf:scroll-end");

  await page.evaluate(() => {
    for (let at: Element | null = document.getElementById("page"); at; at = at.parentElement) at.scrollTop = 0;
    scrollTo(0, 0);
  });
  await page.waitForTimeout(REST_WAIT);
  const target = await page.evaluate(markPressTarget);
  if (target) {
    await page.focus("[data-perf-target]");
    await mark(page, "perf:press");
    await page.keyboard.press("Space");
    await page.waitForTimeout(PRESS);
    await mark(page, "perf:press-end");
  }
  return { readyAt, elements, animations, notches, stride, target };
}

function readLoad(trace: Trace, drove: Drove, run: Run): void {
  const start = trace.navigationStart;
  const ready = trace.mark("perf:ready");
  const loaded = trace.times(start, ready);
  const long = longTasks(trace, start, ready);
  const forced = trace.forced(start, ready).by.map((by) => `forced by ${by}`);
  run["load.ready"] = { value: drove.readyAt };
  run["load.script"] = { value: loaded.script, culprits: trace.hottest(start, ready) };
  run["load.style"] = { value: loaded.style, culprits: forced };
  run["load.layout"] = { value: loaded.layout, culprits: forced };
  run["load.paint"] = { value: loaded.paint };
  run["load.longTasks"] = { value: long.count, culprits: long.culprits };
  run["load.longTaskTime"] = { value: long.time, culprits: long.culprits };
  run["load.elements"] = { value: drove.elements };
  run["load.animations"] = { value: drove.animations.length, culprits: drove.animations.slice(0, 5) };
}

function readRest(trace: Trace, drove: Drove, run: Run): void {
  const rest = [trace.mark("perf:rest"), trace.mark("perf:rest-end")] as const;
  const seconds = (rest[1] - rest[0]) / 1e6;
  const busy = Object.values(trace.times(...rest)).reduce((sum, ms) => sum + ms, 0);
  const culprits = [...trace.hottest(...rest), ...trace.invalidatedBy(...rest).map((by) => `restyled by ${by}`), ...drove.animations.slice(0, 3).map((what) => `animation ${what}`)];
  run["rest.busy"] = { value: busy / seconds, culprits };
  run["rest.styles"] = { value: trace.count("UpdateLayoutTree", ...rest) / seconds, culprits };
  run["rest.layouts"] = { value: trace.count("Layout", ...rest) / seconds, culprits };
  run["rest.callbacks"] = { value: (trace.count("TimerFire", ...rest) + trace.count("FireAnimationFrame", ...rest)) / seconds, culprits: trace.hottest(...rest) };
}

function readScroll(trace: Trace, drove: Drove, run: Run): void {
  const scroll = [trace.mark("perf:scroll"), trace.mark("perf:scroll-end")] as const;
  const frames = trace.frames(...scroll);
  const forced = trace.forced(...scroll);
  const long = longTasks(trace, ...scroll);
  const worst = [...frames].sort((a, b) => b.busy - a.busy)[0];
  const each = (pick: (frame: Frame) => number): number => (frames.length > 0 ? frames.reduce((sum, frame) => sum + pick(frame), 0) / frames.length : 0);
  const forcedBy = forced.by.map((by) => `forced by ${by}`);
  run["scroll.median"] = { value: frames.length > 0 ? median(frames.map((frame) => frame.busy)) : 0, culprits: trace.hottest(...scroll) };
  run["scroll.worst"] = { value: worst?.busy ?? 0, culprits: worst ? trace.hottest(worst.start, worst.end) : [] };
  run["scroll.styleLayout"] = { value: each((frame) => frame.times.style + frame.times.layout), culprits: trace.invalidatedBy(...scroll).map((by) => `restyled by ${by}`) };
  run["scroll.paint"] = { value: each((frame) => frame.times.paint) };
  run["scroll.forced"] = { value: forced.count, culprits: forcedBy };
  run["scroll.forcedTime"] = { value: frames.length > 0 ? forced.time / frames.length : 0, culprits: forcedBy };
  run["scroll.longTasks"] = { value: long.count, culprits: long.culprits };
  run.notes.push(`scroll: ${drove.notches} wheel steps of ${drove.stride} px, ${frames.length} frames with main-thread work`);
}

function readPress(trace: Trace, drove: Drove, run: Run): void {
  if (!drove.target) {
    run.notes.push("press: skipped, the stage's live sample has no control to press");
    return;
  }
  const press = [trace.mark("perf:press"), trace.mark("perf:press-end")] as const;
  const longest = [...trace.topTasks(...press)].sort((a, b) => b.end - b.start - (a.end - a.start))[0];
  const busy = Object.values(trace.times(...press)).reduce((sum, ms) => sum + ms, 0);
  run["press.longest"] = { value: longest ? (longest.end - longest.start) / 1000 : 0, culprits: longest ? [trace.breakdown(longest), ...trace.hottest(longest.start, longest.end, 2)] : [] };
  run["press.over"] = { value: framesOver(trace.frames(...press)), culprits: trace.hottest(...press) };
  run["press.busy"] = { value: busy, culprits: trace.hottest(...press) };
  run.notes.push(`press: Space on ${drove.target}`);
}

/** One run of `slug` in a fresh context: load, rest, scroll and press, under one trace. */
async function measure(browser: Browser, level: string, slug: string): Promise<Run> {
  const context = await browser.newContext({ viewport: VIEWPORT, deviceScaleFactor: 1, reducedMotion: "no-preference" });
  try {
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.addInitScript(watchReady);
    const url = `${ORIGIN}/index.html#/${level}/${slug}`;
    await browser.startTracing(page, { categories: CATEGORIES });
    let drove: Drove;
    let traced: Buffer;
    try {
      drove = await drive(page, url, slug);
    } finally {
      // Stopped whatever failed, so the next run can start a trace.
      traced = await browser.stopTracing();
    }
    if (errors.length > 0) throw new Error(`the page failed: ${errors.join("; ")}`);
    const trace = new Trace(traced.toString("utf8"), SITE, url);
    const run: Run = { notes: [] };
    readLoad(trace, drove, run);
    readRest(trace, drove, run);
    readScroll(trace, drove, run);
    readPress(trace, drove, run);
    return run;
  } finally {
    await context.close();
  }
}

function format(value: number, unit: string): string {
  const shown = Number.isInteger(value) ? String(value) : value >= 100 ? value.toFixed(0) : value.toFixed(1);
  return unit ? `${shown} ${unit}` : shown;
}

/** Prints one page's table and returns whether every budget held. */
function report(slug: string, runs: readonly Run[]): boolean {
  let held = true;
  const rows: string[][] = [["measure", "median", `runs (${runs.length})`, "budget", ""]];
  const failures: string[] = [];
  for (const measure of Object.keys(MEASURES) as Measure[]) {
    const readings = runs.map((run) => run[measure]).filter((reading): reading is Reading => reading !== undefined);
    if (readings.length === 0) continue;
    const { label, unit } = MEASURES[measure];
    const values = readings.map((reading) => reading.value);
    const value = median(values);
    const budget = budgetFor(slug, measure);
    const pass = budget === null || value <= budget.budget;
    rows.push([label, format(value, unit), values.map((v) => format(v, "")).join(" "), budget ? format(budget.budget, unit) + (budget.reason ? " *" : "") : "—", budget === null ? "" : pass ? "ok" : "OVER"]);
    if (pass) continue;
    held = false;
    // The culprits of the run whose value is the median: what a typical run shows.
    const typical = readings.find((reading) => reading.value === value) ?? readings[0]!;
    failures.push(`  ${label}: ${format(value, unit)}, budget ${format(budget!.budget, unit)}`);
    for (const culprit of typical.culprits ?? []) failures.push(`    ${culprit}`);
  }
  const widths = rows[0]!.map((_, column) => Math.max(...rows.map((row) => row[column]!.length)));
  process.stdout.write(`\n${slug}\n`);
  for (const row of rows) process.stdout.write(`  ${row.map((cell, column) => (column === 0 ? cell.padEnd(widths[column]!) : cell.padStart(widths[column]!))).join("  ")}\n`);
  // Every run presses and scrolls alike; their frame counts differ by a few.
  for (const note of runs[0]?.notes ?? []) process.stdout.write(`  ${note}\n`);
  for (const [measure, override] of Object.entries(MEASURES)) {
    const budget = budgetFor(slug, measure as Measure);
    if (budget?.reason) process.stdout.write(`  * ${override.label}: ${budget.reason}\n`);
  }
  if (failures.length > 0) process.stdout.write(`  over budget, and what it points at:\n${failures.join("\n")}\n`);
  return held;
}

async function main(): Promise<void> {
  const slugs = process.argv.slice(2);
  if (slugs.length === 0 || slugs.some((slug) => slug.startsWith("-"))) usage();
  mkdirSync(SITE, { recursive: true });
  process.stdout.write(`load average before: ${load()}\nbuilding the site into ${SITE}\n`);
  await build({ root: ROOT, configFile: path.join(ROOT, "vite.config.ts"), logLevel: "warn", build: { outDir: SITE, emptyOutDir: true, minify: false } });
  const server = await preview({ root: ROOT, configFile: path.join(ROOT, "vite.config.ts"), logLevel: "warn", build: { outDir: SITE }, preview: { port: PORT, strictPort: true, host: "127.0.0.1" } });
  let browser: Browser | null = null;
  let held = true;
  try {
    // One browser, headless and drawing in software, whatever the machine has.
    browser = await chromium.launch({ executablePath: chromiumPath(), headless: true, args: ["--disable-gpu"] });
    const probe = await browser.newPage();
    await probe.goto(`${ORIGIN}/index.html?chrome=off`);
    const pages = await probe.evaluate(async () => {
      await window.kubaos.ready;
      return window.kubaos.pages;
    });
    await probe.close();
    const unknown = slugs.filter((slug) => !pages.some((page) => page.slug === slug));
    if (unknown.length > 0) throw new Error(`no page ${unknown.join(", ")}: one of ${pages.map((page) => page.slug).join(", ")}`);
    for (const slug of slugs) {
      const { level } = pages.find((page) => page.slug === slug)!;
      const runs: Run[] = [];
      for (let i = 0; i < RUNS; i++) runs.push(await measure(browser, level, slug));
      held = report(slug, runs) && held;
    }
  } finally {
    await browser?.close();
    await server.close();
  }
  process.stdout.write(`\nload average after: ${load()} (1, 5 and 15 minutes; this machine has its other work too)\n`);
  if (!held) process.exit(1);
}

main().catch((error: unknown) => {
  process.stderr.write(`perf: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
