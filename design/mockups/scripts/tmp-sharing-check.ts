// Temporary: loads the sharing pages whole and with `?all`, and prints every error they raise.
import { chromium } from "playwright-core";
import { chromiumPath, throughNode } from "./browser.ts";

const origin = "http://localhost:5212";
const pages = process.argv.slice(2);
const browser = await chromium.launch({ executablePath: chromiumPath(), args: ["--mute-audio"] });
const context = await browser.newContext({ viewport: { width: 1600, height: 1000 } });
await throughNode(context, origin);
for (const slug of pages) {
  for (const query of ["", "?all"]) {
    const page = await context.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));
    page.on("console", (message) => {
      if (message.type() === "error" || message.type() === "warning") errors.push(`${message.type()}: ${message.text()}`);
    });
    await page.goto(`${origin}/${query}#/shell/${slug}`);
    await page.evaluate(() => (window as unknown as { kubaos: { ready: Promise<void> } }).kubaos.ready).catch((error: Error) => errors.push(`ready: ${error.message}`));
    await page.waitForTimeout(1500);
    const sections = await page.evaluate(() => `${document.querySelector("h1")?.textContent ?? "no h1"}, ${document.querySelectorAll("section, h2").length}`);
    console.log(`${slug} ${query || "whole"}: ${sections} sections, ${errors.length} errors`);
    for (const error of errors) console.log(`  ${error}`);
    await page.close();
  }
}
await browser.close();
