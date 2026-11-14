// Holds the geometry frame's walk (`src/site/frame.ts`) to what a person sees, on small pages of
// probed boxes drawn in headless Chromium: each case lays a page out, walks it into a frame, and
// checks the boxes the check will read.
//
//   node scripts/frame-test.ts
//
// It fails naming every case that disagrees. The browser is nixpkgs' Playwright Chromium
// (`PLAYWRIGHT_BROWSERS_PATH`, set by `nix develop`); the walk is the site's own source with its
// types stripped, so no build is needed.

import { readFileSync } from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { chromiumPath } from "./browser.ts";

interface Node {
  role: string;
  bounds: { x: number; y: number; width: number; height: number };
  children: Node[];
  [key: string]: unknown;
}

interface Case {
  /** What it checks, as a sentence. */
  readonly name: string;
  /** The page's content, in a 400 px wide page box with no margins. */
  readonly html: string;
  /** The scale the frame is walked at. */
  readonly scale?: number;
  /** What is wrong with the frame's root, or null. */
  readonly check: (root: Node) => string | null;
}

/** A probe's attribute. */
function probe(role: Record<string, unknown>): string {
  return `data-probe='${JSON.stringify(role)}'`;
}

const BOX = probe({ role: "component", name: "Box", variant: "", fixed: { width: false, height: false } });
const TEXT = probe({ role: "text" });

/** Every node under `node` whose probe names it `name`, or whose role is `name`, depth first. */
function all(node: Node, name: string): Node[] {
  const found = node.name === name || node.role === name ? [node] : [];
  return [...found, ...node.children.flatMap((child) => all(child, name))];
}

function one(root: Node, name: string): Node {
  const found = all(root, name);
  if (found.length !== 1) throw new Error(`${found.length} nodes named ${name}`);
  return found[0]!;
}

/** What differs between a box and the one expected, or null. */
function boxIs(node: Node, x: number, y: number, width: number, height: number): string | null {
  const b = node.bounds;
  const near = (a: number, e: number) => Math.abs(a - e) < 1e-3;
  return near(b.x, x) && near(b.y, y) && near(b.width, width) && near(b.height, height)
    ? null
    : `${node.name ?? node.role} is ${b.width}×${b.height} at (${b.x}, ${b.y}), not ${width}×${height} at (${x}, ${y})`;
}

const CASES: Case[] = [
  {
    name: "a box drawn scaled is reported at the size it is laid out at, marked as a picture",
    html: `<div style="scale: 0.25; transform-origin: 0 0; width: 400px; height: 200px">
      <span ${BOX} style="display: block; margin-left: 100px; width: 80px; height: 40px"></span>
    </div>`,
    check: (root) => {
      const picture = root.children[0]!;
      if (picture.picture !== 0.25) return `the picture says ${String(picture.picture)}, not 0.25`;
      return boxIs(picture, 0, 0, 400, 200) ?? boxIs(one(picture, "Box"), 100, 0, 80, 40);
    },
  },
  {
    name: "a probed box scaled about its centre keeps its own role and is placed where it is laid out",
    html: `<div style="padding: 20px">
      <span ${BOX} style="display: flow-root; scale: 0.96; width: 200px; height: 100px">
        <span ${TEXT} style="display: block; margin: 10px; width: 50px; height: 20px">Hi</span>
      </span>
    </div>`,
    check: (root) => {
      const picture = one(root, "Box");
      if (picture.picture !== 0.96) return `the window says ${String(picture.picture)}, not 0.96`;
      return boxIs(picture, 20, 20, 200, 100) ?? boxIs(one(picture, "text"), 30, 30, 50, 20);
    },
  },
  {
    name: "a picture inside a picture is reported at its own size too",
    html: `<div style="scale: 0.5; transform-origin: 0 0; width: 400px; height: 400px">
      <div style="scale: 0.5; transform-origin: 0 0; margin-left: 40px; width: 200px; height: 200px">
        <span ${BOX} style="display: block; margin-left: 12px; width: 40px; height: 40px"></span>
      </div>
    </div>`,
    check: (root) => boxIs(one(root, "Box"), 52, 0, 40, 40),
  },
  {
    name: "a picture's layout off the device grid stays off it",
    scale: 2,
    html: `<div style="scale: 0.3; transform-origin: 0 0; width: 400px; height: 200px">
      <span ${BOX} style="display: block; margin-left: 9.5px; width: 40px; height: 40px"></span>
    </div>`,
    check: (root) => boxIs(one(root, "Box"), 4.75, 0, 20, 20),
  },
  {
    name: "a picture laid out between device pixels is measured from its own corner, on a whole pixel",
    html: `<div style="padding-left: 10.4px">
      <div style="scale: 0.5; transform-origin: 0 0; width: 400px; height: 200px">
        <span ${BOX} style="display: block; margin-left: 12px; width: 40px; height: 40px"></span>
      </div>
    </div>`,
    check: (root) => boxIs(one(root, "Box"), 22, 0, 40, 40),
  },
  {
    name: "a box moved without scaling is reported where it is drawn, with no mark",
    html: `<span ${BOX} style="display: block; translate: 12px 4px; width: 40px; height: 40px"></span>`,
    check: (root) => {
      const moved = one(root, "Box");
      return moved.picture !== undefined ? "a moved box is marked as a picture" : boxIs(moved, 12, 4, 40, 40);
    },
  },
];

async function main(): Promise<void> {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const source = readFileSync(path.join(here, "../src/site/frame.ts"), "utf8");
  // The walk as a plain script: `frame` becomes a global the cases call.
  const walk = stripTypeScriptTypes(source).replace(/^export /gm, "");
  const browser = await chromium.launch({ executablePath: chromiumPath() });
  const failed: string[] = [];
  try {
    const page = await browser.newPage({ viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 });
    for (const each of CASES) {
      await page.setContent(`<!doctype html><body style="margin: 0"><div id="page" style="width: 400px">${each.html}</div></body>`);
      await page.addScriptTag({ content: walk });
      const scale = each.scale ?? 1;
      const root = (await page.evaluate((at) => (globalThis as unknown as { frame: (...a: unknown[]) => { root: unknown } }).frame(document.getElementById("page"), "case", "dark", at).root, scale)) as Node;
      let wrong: string | null;
      try {
        wrong = each.check(root);
      } catch (error) {
        wrong = error instanceof Error ? error.message : String(error);
      }
      if (wrong) failed.push(`${each.name}: ${wrong}`);
    }
  } finally {
    await browser.close();
  }
  for (const line of failed) process.stderr.write(`frame-test: ${line}\n`);
  if (failed.length > 0) {
    process.stderr.write(`frame-test: ${failed.length} of ${CASES.length} failed\n`);
    process.exit(1);
  }
  process.stdout.write(`frame-test: ${CASES.length} cases walked as drawn\n`);
}

main().catch((error: unknown) => {
  process.stderr.write(`frame-test: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
