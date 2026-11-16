// Renders the wallpaper picker's tour as numbered PNG frames, one every 1/60 s of the page's own
// time, for ffmpeg to join into a film (`README.md`, "The Gallery's tour").
//
//   node scripts/tour.ts OUT [--size 2560x1440] [--scale 1.5] [--from A] [--frames B] [--port 5183]
//
// OUT gets 00000.png, 00001.png, …, beats.txt, each beat's first frame and time, and trace-A.tsv,
// each frame's page time, the front picture's box, the step out and the room's light, to show the
// motion even. Only frames A to B (B left out) are written: the tour still plays from its start on
// the same clock, so a run in segments draws the frames one whole run would, and `--frames` alone is
// a trial. The live Gallery is opened on the whole viewport through the site's own Full screen
// control, at night, and driven by key presses alone: no pointer, so no hover and no tooltip. Vite
// serves the site on PORT for the run and stops with it.
//
// The page's time is its own (`scripts/clock.ts`), so a frame is exactly 1/60 s after the last
// whatever it takes to draw.

import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type CDPSession, type Page } from "playwright-core";
import { createServer } from "vite";
import { chromiumPath } from "./browser.ts";
import { installClock } from "./clock.ts";

const FPS = 60;
/** How long the picker rests between moves, once every animation has ended and no timer is near. */
const REST = 1.8;
/** The longest a move may take, in seconds: past it, something never stops. */
const LONGEST = 8;

/** A key pressed and what it shows, or a rest. */
interface Beat {
  readonly name: string;
  readonly key?: string;
  /** Seconds held after the move ends. */
  readonly rest: number;
}

const TOUR: readonly Beat[] = [
  { name: "the desktop at rest", rest: 2 },
  { name: "opening: the desktop steps back into the gallery, the lamp strikes", key: "Control+Meta+Space", rest: REST },
  { name: "the next picture", key: "ArrowRight", rest: REST },
  { name: "the one after", key: "ArrowRight", rest: REST },
  { name: "the floor above", key: "ArrowUp", rest: REST },
  { name: "back down", key: "ArrowDown", rest: REST },
  { name: "the Desktop view", key: "2", rest: REST },
  { name: "the Lock view", key: "3", rest: REST },
  { name: "the lock wakes", key: "w", rest: REST },
  { name: "the lock rests", key: "w", rest: REST },
  { name: "depth off", key: "d", rest: REST },
  { name: "depth on", key: "d", rest: REST },
  { name: "full screen", key: "f", rest: REST },
  { name: "back to the wall", key: "f", rest: REST },
  { name: "applied: the picture grows into the desktop", key: "Enter", rest: 2.5 },
];

function usage(): never {
  process.stderr.write("usage: node scripts/tour.ts OUT [--size 2560x1440] [--scale 1.5] [--from A] [--frames B] [--port 5183]\n");
  process.exit(2);
}

/** Writes one frame of the page as it is now. */
async function shoot(cdp: CDPSession, file: string): Promise<void> {
  const shot = await cdp.send("Page.captureScreenshot", { format: "png", optimizeForSpeed: true });
  writeFileSync(file, Buffer.from(shot.data, "base64"));
}

/** Moves the page's time until `done` holds, at most `limit` frames. */
async function pump(page: Page, done: () => Promise<boolean>, limit: number): Promise<void> {
  for (let i = 0; i < limit; i++) {
    if (await done()) return;
    await page.evaluate(() => window.pageClock.step());
  }
  throw new Error("the page never got there");
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const out = args.shift();
  if (!out || out.startsWith("-")) usage();
  let [width, height] = [2560, 1440];
  let scale = "1.5";
  let from = 0;
  let limit = Number.POSITIVE_INFINITY;
  let port = 5183;
  while (args.length > 0) {
    const arg = args.shift()!;
    const value = args.shift() ?? usage();
    if (arg === "--size") [width, height] = value.split("x").map(Number) as [number, number];
    else if (arg === "--scale") scale = value;
    else if (arg === "--from") from = Number(value);
    else if (arg === "--frames") limit = Number(value);
    else if (arg === "--port") port = Number(value);
    else usage();
  }
  mkdirSync(out, { recursive: true });

  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const vite = await createServer({ root, logLevel: "error", server: { port, strictPort: true, hmr: false } });
  await vite.listen();
  // Software drawing alone: a GPU process on the desktop's own GPU has frozen the machine before.
  const browser = await chromium.launch({ executablePath: chromiumPath(), args: ["--disable-gpu"] });
  try {
    const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1, reducedMotion: "no-preference", colorScheme: "dark" });
    await context.addInitScript(installClock, 1000 / FPS);
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    const cdp = await context.newCDPSession(page);

    await page.goto(`http://localhost:${port}/index.html?mode=dark&scale=${scale}&text=short&reduced=off&speed=1&flicker=on#/shell/wallpaper`);
    // Software drawing reads a backdrop filter under a scaled ancestor from the wrong place and
    // fills the rest with the blur's mirrored edge, so the awake lock on the wall shows its city
    // mirrored. The wall's screens are drawn at the same size and place by `zoom` instead of
    // `scale`: no transform for the backdrop to be read through, and the picture the page means.
    await page.addStyleTag({ content: ".gl-frame > .k-lock { scale: none; zoom: var(--share); }" });
    // The site's modules load on real time, then draw the page on the page's.
    await pump(page, () => page.evaluate(() => "kubaos" in window), 600);
    let ready = false;
    void page.evaluate(() => window.kubaos.ready).then(() => (ready = true), (error: unknown) => errors.push(String(error)));
    await pump(page, async () => ready || errors.length > 0, 600);
    await page.evaluate(() => {
      const control = document.querySelector<HTMLButtonElement>(".page-full-screen:not([hidden])");
      if (!control) throw new Error("the stage has no Full screen control");
      control.click();
    });
    const shown = () => page.evaluate(() => document.activeElement?.matches(".site-full-screen .gl-screen") ?? false);
    await pump(page, shown, 60);
    await pump(page, () => page.evaluate(() => window.pageClock.quiet()), 600);

    const beats: string[] = [];
    const trace: string[] = ["frame\tms\tleft\twidth\tout\tlit"];
    let index = 0;
    const capture = async (): Promise<boolean> => {
      if (index >= limit) return false;
      await page.evaluate(() => window.pageClock.step());
      if (index < from) {
        index++;
        return true;
      }
      const row = await page.evaluate(() => {
        const screen = document.querySelector<HTMLElement>(".site-full-screen .gl-screen");
        const front = screen?.querySelector(".gl-art[data-on] .gl-frame")?.getBoundingClientRect();
        const style = screen ? getComputedStyle(screen) : undefined;
        return [performance.now().toFixed(3), front?.left.toFixed(2), front?.width.toFixed(2), style?.getPropertyValue("--out"), style?.getPropertyValue("--room-lit")].join("\t");
      });
      trace.push(`${index}\t${row}`);
      await shoot(cdp, path.join(out, `${String(index).padStart(5, "0")}.png`));
      index++;
      if (index % 60 === 0) process.stdout.write(`${index} frames\n`);
      return true;
    };
    for (const beat of TOUR) {
      beats.push(`${index}\t${(index / FPS).toFixed(2)}s\t${beat.name}`);
      if (beat.key) {
        await page.keyboard.press(beat.key);
        // The move plays out, then the rest: each move starts from a picker at rest.
        for (let i = 0; !(await page.evaluate(() => window.pageClock.quiet())); i++) {
          if (i === LONGEST * FPS) throw new Error(`${beat.name} still moves after ${LONGEST} s`);
          if (!(await capture())) break;
        }
      }
      for (let i = 0; i < Math.round(beat.rest * FPS); i++) if (!(await capture())) break;
      if (errors.length > 0) throw new Error(`the page failed: ${errors.join("; ")}`);
    }
    writeFileSync(path.join(out, "beats.txt"), `${beats.join("\n")}\n`);
    writeFileSync(path.join(out, `trace-${from}.tsv`), `${trace.join("\n")}\n`);
    process.stdout.write(`${index} frames, ${(index / FPS).toFixed(2)} s\n${beats.join("\n")}\n`);
  } finally {
    await browser.close();
    await vite.close();
  }
}

main().catch((error: unknown) => {
  process.stderr.write(`tour: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
