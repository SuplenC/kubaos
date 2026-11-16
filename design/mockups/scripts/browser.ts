// What the scripts that drive the site in headless Chromium share: the browser, a server for
// `dist/`, and a way round the host's network changes.

import { createReadStream, existsSync, readdirSync, statSync } from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import type { BrowserContext } from "playwright-core";

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

/**
 * Fetches every request `context` makes to `origin` in Node and hands Chromium the response, so
 * Chromium opens no socket for it. Chromium aborts every load in flight with
 * `ERR_NETWORK_CHANGED` when the host's interfaces change, which containers starting and stopping
 * do every few seconds; a page served this way loads whatever they do. Each response is fetched
 * once for the context, so pages loaded side by side ask a dev server for its modules once.
 */
export async function throughNode(context: BrowserContext, origin: string): Promise<void> {
  const fetched = new Map<string, Promise<{ status: number; headers: Record<string, string>; body: Buffer }>>();
  await context.route(`${origin}/**`, async (route) => {
    const request = route.request();
    const key = `${request.method()} ${request.url()}`;
    let response = fetched.get(key);
    if (!response) {
      response = fetch(request.url(), { method: request.method(), headers: request.headers() }).then(async (each) => ({
        status: each.status,
        headers: Object.fromEntries(each.headers),
        body: Buffer.from(await each.arrayBuffer()),
      }));
      fetched.set(key, response);
    }
    await route.fulfill(await response);
  });
}
