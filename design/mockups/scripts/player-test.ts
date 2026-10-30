// Drives the motion players of the built site (`dist/`) in headless Chromium and holds them to
// their recipes: seeking on the plot holds the exact frame, and each track's ball sits on its
// curve at the progress the browser is playing, for every easing and each of `neon.strike`'s
// steps, with the move's own tracks, reduced motion's and the stutter off's. Each part alone moves
// with the whole, a mark seeks to its moment, and the arrow keys step a 60 Hz frame.
//
//   node scripts/player-test.ts [PAGE...]
//
// It fails on the first player that disagrees, naming it. The browser is nixpkgs' Playwright
// Chromium (`PLAYWRIGHT_BROWSERS_PATH`, set by `nix develop`).

import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright-core";
import { chromiumPath, serve } from "./browser.ts";

const FRAME = 1000 / 60;

/** How far a ball may sit from where its progress says, in the plot's pixels. */
const PIXEL = 0.01;

/** How far apart the ball's progress and the browser's may be. */
const PROGRESS = 1e-3;

let failures = 0;
let balls = 0;

function check(ok: boolean, what: string): void {
  if (ok) return;
  failures++;
  process.stderr.write(`player-test: ${what}\n`);
}

/** What one player shows held at a time: per ball, its progress, where it sits, and what plays. */
interface Held {
  time: number;
  balls: { part: string; progress: number; cy: number; want: number; onCurve: boolean; played: number[]; alone: number[] }[];
  head: number;
  cx: number[];
}

/** Reads player `index`'s plot, balls and the animated values of its sample and its parts alone. */
async function held(page: Page, index: number): Promise<Held> {
  return page.evaluate((index) => {
    const player = document.querySelectorAll<HTMLElement>(".page-player")[index]!;
    const plot = player.querySelector<SVGSVGElement>(".page-plot")!;
    const { y0, y1 } = plot.dataset;
    const stage = player.querySelector(".page-player-stage")!;
    const alone = [...player.querySelectorAll(".page-part-stage")];
    const time = Number(player.querySelector(".page-player-stage")!.getAnimations({ subtree: true })[0]?.currentTime ?? 0);
    const value = (element: Element, property: string) => Number(getComputedStyle(element).getPropertyValue(property));
    const curves = [...plot.querySelectorAll<SVGPathElement>(".page-plot-curve")];
    const undashed = (curve: SVGPathElement, test: (curve: SVGPathElement) => boolean) => {
      const dash = curve.style.strokeDasharray;
      curve.style.strokeDasharray = "none";
      const result = test(curve);
      curve.style.strokeDasharray = dash;
      return result;
    };
    const balls = [...plot.querySelectorAll<SVGCircleElement>(".page-plot-ball")].map((ball, i) => {
      const target = ball.dataset.target!;
      const progress = Number(ball.dataset.progress);
      const centre = new DOMPoint(Number(ball.getAttribute("cx")), Number(ball.getAttribute("cy")));
      return {
        part: target,
        progress,
        cy: Number(ball.getAttribute("cy")),
        want: Number(y0) + (Number(y1) - Number(y0)) * progress,
        // On the curve as drawn, not only where the plot's formula says: on its whole stroke, a
        // dash of it or a gap between two.
        onCurve: undashed(curves[i]!, (curve) => curve.isPointInStroke(centre)),
        played: [...stage.querySelectorAll(target)].map((element) => value(element, "opacity")),
        alone: alone[i] ? [...alone[i]!.querySelectorAll(target)].map((element) => value(element, "opacity")) : [],
      };
    });
    const head = Number(plot.querySelector(".page-plot-head")!.getAttribute("x1"));
    return { time: time % Number(plot.dataset.total), balls, head, cx: balls.map((_, i) => Number(plot.querySelectorAll(".page-plot-ball")[i]!.getAttribute("cx"))) };
  }, index);
}

/** Clicks player `index`'s plot where time `t` is drawn. */
async function seek(page: Page, index: number, t: number): Promise<void> {
  const plot = page.locator(".page-plot").nth(index);
  await plot.scrollIntoViewIfNeeded();
  const { x0, x1, total } = await plot.evaluate((element) => ({
    x0: Number(element.dataset.x0),
    x1: Number(element.dataset.x1),
    total: Number(element.dataset.total),
  }));
  const box = (await plot.boundingBox())!;
  const zoom = box.width / (await plot.evaluate((element) => element.clientWidth));
  const x = box.x + (x0 + (t / total) * (x1 - x0)) * zoom;
  await page.mouse.click(x, box.y + box.height / 3);
}

/** The times worth seeking a move to: inside each of its tracks' spans and each strike's steps. */
async function moments(page: Page, index: number): Promise<number[]> {
  return page.evaluate((index) => {
    const player = document.querySelectorAll<HTMLElement>(".page-player")[index]!;
    const marks = [...player.querySelectorAll(".page-plot-mark")].map((mark) => Number(mark.getAttribute("data-at"))).sort((a, b) => a - b);
    // Between each two marks: a quarter, half and three quarters of the way, so each eased span and
    // each strike step is sampled away from the keyframes' edges.
    const times: number[] = [];
    for (let i = 1; i < marks.length; i++) {
      for (const share of [0.25, 0.5, 0.75]) times.push(marks[i - 1]! + share * (marks[i]! - marks[i - 1]!));
    }
    return times;
  }, index);
}

/** The values each player's tracks go between, from the page's own recipes, by move and set. */
async function ranges(page: Page): Promise<Map<string, { from: number; to: number }[]>> {
  const recipes = (await page.evaluate(() => window.kubaos.motion())) as Record<string, unknown> | null;
  const found = new Map<string, { from: number; to: number }[]>();
  for (const list of Object.values(recipes ?? {})) {
    if (!Array.isArray(list)) continue;
    for (const move of list as { name: string; tracks: unknown; reduced: unknown; unstuttered?: unknown }[]) {
      const of = (tracks: unknown) =>
        (tracks as { from: { value: number }; to: { value: number } }[]).map((track) => ({ from: track.from.value, to: track.to.value }));
      found.set(`${move.name}|tracks`, of(move.tracks));
      if (move.reduced !== "same") found.set(`${move.name}|reduced`, of(move.reduced));
      if (move.unstuttered) found.set(`${move.name}|unstuttered`, of(move.unstuttered));
    }
  }
  return found;
}

async function testPage(page: Page, origin: string, level: string, slug: string, query: string, set: string): Promise<void> {
  await page.goto(`${origin}/index.html?${query}#/${level}/${slug}`);
  await page.evaluate(() => window.kubaos.ready);
  await page.waitForFunction((want) => document.getElementById("page")?.dataset.page === want, slug);
  // Every glow on the page, the players' parts alone among them, is drawn: masked, never a block.
  const bare = await page.evaluate(() => [...document.querySelectorAll<HTMLElement>(".k-glow")].filter((glow) => !glow.style.maskImage).length);
  check(bare === 0, `${slug} (${query}): ${bare} glows drawn without their mask`);
  const values = await ranges(page);
  const names = await page.locator(".page-player-words > :first-child").allTextContents();
  for (const [index, name] of names.entries()) {
    const where = `${slug}, ${name} (${query})`;
    const tracks = values.get(`${name}|${set}`) ?? values.get(`${name}|tracks`)!;
    let checked = 0;
    for (const t of await moments(page, index)) {
      await seek(page, index, t);
      const shown = await held(page, index);
      check((await page.locator(".page-player .site-control").nth(index * 4).textContent()) === "Play", `${where}: seeking didn't pause`);
      shown.balls.forEach((ball, i) => {
        const range = tracks[i]!;
        const playing = ball.played.map((v) => (v - range.from) / (range.to - range.from));
        check(Math.abs(ball.cy - ball.want) <= PIXEL, `${where} at ${shown.time} ms: ${ball.part}'s ball at y ${ball.cy}, its progress ${ball.progress} is drawn at ${ball.want}`);
        check(ball.onCurve, `${where} at ${shown.time} ms: ${ball.part}'s ball is off its curve`);
        check(Math.abs(shown.cx[i]! - shown.head) <= 1, `${where} at ${shown.time} ms: ${ball.part}'s ball isn't on the playhead`);
        for (const p of playing) {
          check(Math.abs(p - ball.progress) <= PROGRESS, `${where} at ${shown.time} ms: ${ball.part} plays at ${p}, its ball says ${ball.progress}`);
        }
        for (const v of ball.alone) {
          check(Math.abs(v - (ball.played[0] ?? v)) <= 1e-6, `${where} at ${shown.time} ms: ${ball.part} alone plays ${v}, the whole ${ball.played[0]}`);
        }
        checked++;
        balls++;
      });
    }
    check(checked > 0, `${where}: nothing checked`);

    // A mark goes to its exact moment, and → steps one frame from there.
    const mark = page.locator(".page-player").nth(index).locator(".page-plot-mark").nth(1);
    const at = Number(await mark.getAttribute("data-at"));
    await mark.locator("rect").dispatchEvent("pointerdown", { pointerId: 1, button: 0, isPrimary: true });
    const marked = await held(page, index);
    check(Math.abs(marked.time - at) < 1e-6, `${where}: the mark at ${at} ms held ${marked.time} ms`);
    await page.locator(".page-plot-box").nth(index).focus();
    await page.keyboard.press("ArrowRight");
    const stepped = await held(page, index);
    const frame = Math.floor(at / FRAME + 1e-6) + 1;
    check(Math.abs(stepped.time - frame * FRAME) < 1e-6, `${where}: → from ${at} ms held ${stepped.time} ms, not frame ${frame}'s ${frame * FRAME}`);
    await page.keyboard.press("Shift+ArrowLeft");
    const back = await held(page, index);
    check(Math.abs(back.time - Math.max(0, frame - 10) * FRAME) < 1e-6, `${where}: Shift+← held ${back.time} ms`);

    // Pointing at a part, by its stage, its curve, its legend key or its row, picks it everywhere:
    // its row, its curve, its ball, its stage, and an outline of its layers on the whole.
    const player = page.locator(".page-player").nth(index);
    const alone = player.locator(".page-part-alone");
    // The parts alone stack one under another, each a full row, in the legend's order.
    const stacked = await player.evaluate((element) => {
      const keys = [...element.querySelectorAll(".page-plot-legend .page-plot-key")].map((key) => key.textContent);
      const rows = [...element.querySelectorAll<HTMLElement>(".page-part-alone")];
      // A move of one part has no parts alone; a move of more has one per legend key.
      if (rows.length !== (keys.length > 1 ? keys.length : 0)) return false;
      const width = element.querySelector<HTMLElement>(".page-plot-box")?.getBoundingClientRect().width ?? 0;
      return rows.every((row, i) => {
        const box = row.getBoundingClientRect();
        const above = rows[i - 1]?.getBoundingClientRect();
        return (!above || box.top >= above.bottom) && Math.abs(box.width - width) < 0.5 && row.querySelector(".page-plot-key")?.textContent === keys[i];
      });
    });
    check(stacked, `${where}: the parts alone don't stack one under another in the legend's order`);
    const pointers = (i: number) => [
      alone.nth(i),
      player.locator(".page-plot-legend .page-plot-key").nth(i),
      player.locator(`tr[data-pick="${i}"][data-pick-set="${set}"]`).first(),
    ];
    for (let i = 0; i < (await alone.count()); i++) {
      for (const pointer of pointers(i)) {
        if ((await pointer.count()) === 0) continue;
        await pointer.hover();
        const picked = await player.evaluate((element, i) => {
          const of = (selector: string) => [...element.querySelectorAll(`${selector}[data-picked]`)].map((item) => item.getAttribute("data-pick"));
          return {
            curves: of(".page-plot-curve"),
            balls: of(".page-plot-ball"),
            stages: of(".page-part-alone"),
            rows: of("tr"),
            outlines: element.querySelectorAll(".page-player-outline").length,
            want: String(i),
          };
        }, i);
        const one = (list: (string | null)[]) => list.length === 1 && list[0] === picked.want;
        check(one(picked.curves) && one(picked.balls) && one(picked.stages) && one(picked.rows) && picked.outlines > 0, `${where}: part ${i} picked ${JSON.stringify(picked)}`);
        await page.mouse.move(0, 0);
      }
    }
    // A curve is found by the pointer too: the last track's, drawn over any it shares a curve with.
    if ((await alone.count()) > 0) {
      await player.locator(".page-plot").scrollIntoViewIfNeeded();
      const spot = await player.evaluate((element) => {
        const hit = [...element.querySelectorAll<SVGPathElement>(".page-plot-curve-hit")].at(-1)!;
        const plot = hit.ownerSVGElement!;
        const length = hit.getTotalLength();
        // Along the curve, at its middle: inside the move, away from the axes.
        const point = hit.getPointAtLength(length / 2);
        const box = plot.getBoundingClientRect();
        const zoom = box.width / plot.clientWidth;
        return { x: box.left + point.x * zoom, y: box.top + point.y * zoom, want: hit.getAttribute("data-pick") };
      });
      await page.mouse.move(spot.x, spot.y);
      const curve = await player.evaluate((element) => element.querySelector(".page-plot-curve[data-picked]")?.getAttribute("data-pick") ?? null);
      check(curve === spot.want, `${where}: hovering curve ${spot.want} picked ${curve}`);
      await page.mouse.move(0, 0);
    }
  }
}

async function main(): Promise<void> {
  const named = process.argv.slice(2);
  const dist = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../dist");
  if (!existsSync(path.join(dist, "index.html"))) throw new Error(`no built site in ${dist}: run \`just mockups-build\``);
  const origin = await serve(dist);
  const browser = await chromium.launch({ executablePath: chromiumPath() });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1, reducedMotion: "no-preference" });
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    const base = "text=short&window=Default";
    await page.goto(`${origin}/index.html?${base}&mode=dark&scale=1&speed=1&reduced=off&stutter=on`);
    await page.evaluate(() => window.kubaos.ready);
    const pages = (await page.evaluate(() => window.kubaos.pages)).filter((p) => named.length === 0 || named.includes(p.slug));
    for (const { level, slug } of pages) {
      for (const [settings, set] of [
        ["mode=dark&scale=1&speed=1&reduced=off&stutter=on", "tracks"],
        ["mode=light&scale=1.5&speed=1&reduced=off&stutter=on", "tracks"],
        ["mode=dark&scale=1&speed=2&reduced=off&stutter=on", "tracks"],
        ["mode=dark&scale=1&speed=1&reduced=on&stutter=on", "reduced"],
        ["mode=dark&scale=1&speed=1&reduced=off&stutter=off", "unstuttered"],
      ] as const) {
        await testPage(page, origin, level, slug, `${base}&${settings}`, set);
      }
    }
    check(errors.length === 0, `the page failed: ${errors.join("; ")}`);
  } finally {
    await browser.close();
  }
  if (failures > 0) {
    process.stderr.write(`player-test: ${failures} failed\n`);
    process.exit(1);
  }
  process.stdout.write(`player-test: every player holds its recipe, ${balls} balls checked\n`);
}

main().catch((error: unknown) => {
  process.stderr.write(`player-test: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
