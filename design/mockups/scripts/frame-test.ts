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
const ROW = `${probe({ role: "layout", axis: "x", gap: 8, align: "start" })} data-layout`;

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
  {
    name: "a box at no opacity or hidden is left out, keeping its place in a layout as a plain box",
    html: `<span ${ROW} style="display: flex; gap: 8px">
      <span ${BOX} style="width: 40px; height: 40px; opacity: 0"></span>
      <span ${TEXT} style="width: 40px; height: 20px; visibility: hidden">Gone</span>
      <span ${BOX} style="width: 40px; height: 40px"></span>
    </span>`,
    check: (root) => {
      const row = one(root, "layout");
      const kinds = row.children.map((c) => `${c.role}${c.unseen ? `(${String(c.unseen)})` : ""}`).join(" ");
      return kinds === "box(opacity) box(hidden) component" ? null : `the row holds ${kinds}`;
    },
  },
  {
    name: "a box scaled to nothing is left out, and one scaled to a little is still a picture",
    html: `<span ${BOX} style="display: block; scale: 0; width: 40px; height: 40px"></span>
    <span ${BOX} style="display: block; scale: 0.5; width: 40px; height: 40px"></span>`,
    check: (root) => {
      const boxes = all(root, "Box");
      return boxes.length === 1 && boxes[0]!.picture === 0.5 ? null : `${boxes.length} boxes are reported`;
    },
  },
  {
    name: "content placed or moved wholly outside the box that cuts it off is left out",
    html: `<div style="position: relative; overflow: hidden; width: 200px; height: 40px">
      <span ${BOX} style="position: absolute; top: 60px; width: 40px; height: 40px"></span>
      <span ${TEXT} style="display: block; translate: 0 -80px; width: 40px; height: 20px">Slid</span>
    </div>`,
    check: (root) => (root.children.length === 0 ? null : `${root.children.length} nodes are reported`),
  },
  {
    name: "content a layout pushes wholly out of the box that cuts it off is still reported",
    html: `<div style="overflow: hidden; width: 200px; height: 40px">
      <span ${BOX} style="display: block; width: 40px; height: 60px"></span>
      <span ${TEXT} style="display: block; width: 40px; height: 20px">Pushed</span>
    </div>`,
    check: (root) => (all(root, "text").length === 1 ? null : "the pushed text is left out"),
  },
  {
    name: "content in a part shut to nothing is left out",
    html: `<span ${BOX} style="display: block; overflow: hidden; width: 200px; height: 0">
      <span ${TEXT} style="display: block; width: 40px; height: 20px">Shut</span>
    </span>`,
    check: (root) => (all(root, "text").length === 0 ? null : "the shut text is reported"),
  },
  {
    name: "content in a box of the page shut to nothing is still reported",
    html: `<div style="overflow: hidden; width: 200px; height: 0">
      <span ${TEXT} style="display: block; width: 40px; height: 20px">Lost</span>
    </div>`,
    check: (root) => (all(root, "text").length === 1 ? null : "the hidden text is left out"),
  },
  {
    name: "content scrolled out of view is still reported",
    html: `<div style="overflow-y: auto; width: 200px; height: 40px">
      <span ${BOX} style="display: block; width: 40px; height: 60px"></span>
      <span ${TEXT} style="display: block; position: relative; top: 10px; width: 40px; height: 20px">Below</span>
    </div>`,
    check: (root) => (all(root, "text").length === 1 ? null : "the scrolled text is left out"),
  },
  {
    name: "a box placed outside a cut whose box it is placed in is outside the cut is still reported",
    html: `<div style="position: relative">
      <div style="overflow: hidden; width: 200px; height: 40px">
        <span ${BOX} style="position: absolute; top: 60px; width: 40px; height: 40px"></span>
      </div>
    </div>`,
    check: (root) => (all(root, "Box").length === 1 ? null : "the escaping box is left out"),
  },
  {
    name: "a box out of the flow is marked over, through a wrapper that says nothing",
    html: `<span ${ROW} style="display: flex; position: relative; gap: 8px">
      <span style="position: absolute; inset: 0"><span ${BOX} style="display: block; height: 40px"></span></span>
      <span ${TEXT} style="position: absolute; left: 0">Over</span>
      <span ${BOX} style="width: 40px; height: 40px"></span>
    </span>`,
    check: (root) => {
      const marks = one(root, "layout").children.map((c) => (c.over === true ? "over" : "flow")).join(" ");
      return marks === "over over flow" ? null : `the row's children are ${marks}`;
    },
  },
  {
    name: "a probe that floats floats only when placed out of the flow",
    html: `<span style="position: relative; display: block; width: 40px; height: 40px">
      <span ${probe({ role: "component", name: "Hint", variant: "", fixed: { width: false, height: true }, floats: true })} style="position: absolute; top: 48px; width: 60px; height: 24px"></span>
    </span>
    <span ${probe({ role: "component", name: "Hint", variant: "", fixed: { width: false, height: true }, floats: true })} style="display: block; width: 60px; height: 24px"></span>`,
    check: (root) => {
      const said = all(root, "Hint").map((hint) => String(hint.floats === true)).join(" ");
      return said === "true false" ? null : `the hints float: ${said}`;
    },
  },
  {
    name: "a box the site lets scroll sideways is a scroll across when its content runs past it",
    html: `<div style="overflow-x: auto; width: 200px">
      <span ${BOX} style="display: block; width: 400px; height: 40px"></span>
    </div>
    <div style="overflow-x: auto; width: 200px">
      <span ${BOX} style="display: block; width: 100px; height: 40px"></span>
    </div>`,
    check: (root) => {
      const scrolls = all(root, "scroll");
      return scrolls.length === 1 && scrolls[0]!.across === true ? null : `${scrolls.length} scrolls across`;
    },
  },
  {
    name: "a layout's room that scrolls sideways is a scroll across, round a box as wide as its content",
    html: `<span ${ROW} style="display: flex; width: 200px">
      <div style="overflow-x: auto; width: 200px">
        <span style="display: block; width: 400px; height: 40px"><span ${TEXT} style="display: block; width: 380px; height: 20px">One</span></span>
      </div>
    </span>`,
    check: (root) => {
      const scroll = root.children[0]!.children[0]!;
      if (scroll.role !== "scroll" || scroll.across !== true) return `the room is a ${scroll.role} across ${String(scroll.across)}`;
      return boxIs(scroll.children[0]!, 0, 0, 400, 40);
    },
  },
  {
    name: "a layout drawn in reverse lists its children in the order they are drawn",
    html: `<span ${ROW.replace('"axis":"x"', '"axis":"y"')} style="display: flex; flex-direction: column-reverse; gap: 8px; width: 200px">
      <span ${probe({ role: "component", name: "First", variant: "", fixed: { width: false, height: false } })} style="display: block; height: 20px"></span>
      <span ${probe({ role: "component", name: "Second", variant: "", fixed: { width: false, height: false } })} style="display: block; height: 20px"></span>
    </span>`,
    check: (root) => {
      const order = root.children[0]!.children.map((child) => String(child.name)).join(" ");
      return order === "Second First" ? null : `the children are listed ${order}`;
    },
  },
  {
    name: "a box marked as what a well's rows scroll in is a scroll, overflowing or not",
    html: `<span ${BOX} style="display: block; width: 200px">
      <span data-scroll-probe style="display: block; overflow-y: auto; height: 40px">
        <span ${TEXT} style="display: block; width: 40px; height: 20px">One</span>
      </span>
    </span>`,
    check: (root) => (one(root, "Box").children[0]?.role === "scroll" ? null : "the pane is no scroll"),
  },
  {
    name: "a box that says nothing is a scroll only while its content runs past it",
    html: `<div style="overflow-y: auto; width: 200px; height: 40px">
      <span ${BOX} style="display: block; width: 40px; height: 80px"></span>
    </div>
    <div style="overflow-y: auto; width: 200px; height: 40px">
      <span ${BOX} style="display: block; width: 40px; height: 20px"></span>
    </div>`,
    check: (root) => {
      const scrolls = all(root, "scroll");
      return scrolls.length === 1 && scrolls[0]!.across === undefined ? null : `${scrolls.length} scrolls`;
    },
  },
  {
    name: "a probed box that scrolls its own content is inside a scroll, as tall as its content",
    html: `<span ${ROW.replace('"axis":"x"', '"axis":"y"')} style="display: flex; flex-direction: column; gap: 8px; overflow-y: auto; width: 200px; height: 40px">
      <span ${BOX} style="flex: none; height: 40px"></span>
      <span ${BOX} style="flex: none; height: 40px"></span>
    </span>`,
    check: (root) => {
      const scroll = root.children[0]!;
      if (scroll.role !== "scroll") return `the stack is in a ${scroll.role}`;
      return boxIs(scroll, 0, 0, 200, 40) ?? boxIs(scroll.children[0]!, 0, 0, 200, 88);
    },
  },
  {
    name: "wrapped words need their widest line and every line they break onto, not one long line",
    html: `<span ${TEXT} data-wrapped style="display: block; width: 120px; font: 16px/20px sans-serif; --natural: 400">one two three four five six seven eight nine ten</span>`,
    check: (root) => {
      const words = one(root, "text");
      const need = words.natural as { width: number; height: number };
      if (need.width > 120 || need.width < 60) return `the words need ${need.width} across`;
      return need.height === words.bounds.height && need.height >= 60 ? null : `the words need ${need.height} down in a box ${words.bounds.height} tall`;
    },
  },
  {
    name: "wrapped words cut short by their box, or with a word wider than it, still need more room",
    html: `<span ${TEXT} data-wrapped style="display: block; overflow: hidden; width: 120px; height: 20px; font: 16px/20px sans-serif">one two three four five six</span>
    <span ${TEXT} data-wrapped style="display: block; width: 60px; font: 16px/20px sans-serif">unbreakablewordhere</span>`,
    check: (root) => {
      const [cut, wide] = all(root, "text");
      const needs = (node: Node) => node.natural as { width: number; height: number };
      if (needs(cut!).height <= cut!.bounds.height) return `the cut words need only ${needs(cut!).height} down`;
      return needs(wide!).width > wide!.bounds.width ? null : `the long word needs only ${needs(wide!).width} across`;
    },
  },
  {
    name: "words the CSS draws larger than their type style need their advance at that size",
    html: `<span ${TEXT} data-size="14" style="display: block; white-space: pre; width: 400px; font: 28px/34px sans-serif; --natural: 30">Clock</span>`,
    check: (root) => {
      const need = one(root, "text").natural as { width: number; height: number };
      return need.width > 40 && need.width < 400 ? null : `the words need ${need.width} across, not their style's 30`;
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
