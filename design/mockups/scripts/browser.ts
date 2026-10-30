// What the scripts that drive the built site in headless Chromium share: the browser, and a
// server for `dist/`.

import { createReadStream, existsSync, readdirSync, statSync } from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";

const TYPES: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".ttf": "font/ttf",
  ".svg": "image/svg+xml",
};

export function chromiumPath(): string {
  const browsers = process.env.PLAYWRIGHT_BROWSERS_PATH;
  if (!browsers) throw new Error("PLAYWRIGHT_BROWSERS_PATH is unset: run me in `nix develop`");
  const dir = readdirSync(browsers).find((name) => name.startsWith("chromium-"));
  if (!dir) throw new Error(`no Chromium in ${browsers}`);
  return path.join(browsers, dir, "chrome-linux64", "chrome");
}

/** Serves `root` on a free local port, for as long as the process runs. */
export async function serve(root: string): Promise<string> {
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
