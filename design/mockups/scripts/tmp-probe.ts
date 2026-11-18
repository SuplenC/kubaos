// Temporary: reports the camera panel's boxes in solo mode.
import { chromium } from "playwright-core";
import { chromiumPath, throughNode } from "./browser.ts";
const origin = "http://localhost:5212";
const browser = await chromium.launch({ executablePath: chromiumPath(), args: ["--mute-audio"] });
const context = await browser.newContext({ viewport: { width: 1600, height: 1000 } });
await throughNode(context, origin);
const page = await context.newPage();
page.on("pageerror", (e) => console.log("pageerror", e.message));
page.on("console", (m) => console.log("console", m.type(), m.text()));
await page.goto(`${origin}/?solo=the-camera-panel#/shell/on-air-pills`);
await page.waitForTimeout(2500);
console.log(await page.evaluate(() => [".k-screen", ".camera-panel", ".k-media", ".k-sheet", ".k-panel-page"].map((s) => { const e = document.querySelector(s); if (!e) return `${s}: none`; const r = e.getBoundingClientRect(); return `${s}: ${r.x},${r.y} ${r.width}x${r.height} op=${getComputedStyle(e).opacity}`; }).join("\n")));
await browser.close();
