// What the scripts that drive the site in headless Chromium share: the browser, a server for
// `dist/`, and a way round the host's network changes.

import { createReadStream, existsSync, readdirSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { availableParallelism } from "node:os";
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

/** How many pages the checks draw at once: `MOCKUPS_JOBS`, or one per CPU. */
export function jobs(): number {
  const set = process.env.MOCKUPS_JOBS;
  if (set === undefined || set === "") return availableParallelism();
  const count = Number(set);
  if (!Number.isInteger(count) || count < 1) throw new Error(`MOCKUPS_JOBS is ${set}: a whole number of pages, 1 or more`);
  return count;
}

/**
 * Runs `work` on every item, `count` at a time, and hands each result to `done` in the items'
 * order, as soon as every item before it is done too, so what the run prints reads the same
 * whatever finishes first. `work` gets the index of the worker running it, below `count`, which
 * keeps a browser page of its own. Items of one `group` share how that page is set up: a free
 * worker takes the first item left in its group, else the first of the group the fewest workers
 * are in, so each sets its page up as few times as it can. The first item to throw, by order,
 * stops the items after it from starting; those already running finish, the results before it are
 * handed on, and its error is thrown.
 */
export async function inOrder<T, R>(
  items: readonly T[],
  count: number,
  work: (item: T, worker: number) => Promise<R>,
  done: (result: R, item: T) => void,
  group: (item: T) => string = () => "",
): Promise<void> {
  const results: ({ ok: true; value: R } | { ok: false; error: unknown } | undefined)[] = items.map(() => undefined);
  const started = items.map(() => false);
  const groups = items.map(group);
  const workers = Array.from({ length: Math.min(count, items.length) }, (): string | undefined => undefined);
  let handed = 0;
  let failed = items.length;
  const hand = () => {
    while (handed < failed && results[handed]?.ok) {
      const result = results[handed] as { value: R };
      done(result.value, items[handed]!);
      handed++;
    }
  };
  const take = (worker: number): number => {
    let best = -1;
    let crowd = Infinity;
    for (let index = 0; index < failed; index++) {
      if (started[index]) continue;
      if (groups[index] === workers[worker]) return index;
      const others = workers.filter((other) => other === groups[index]).length;
      if (others < crowd) {
        best = index;
        crowd = others;
      }
    }
    return best;
  };
  const run = async (worker: number) => {
    for (let index = take(worker); index >= 0; index = take(worker)) {
      started[index] = true;
      workers[worker] = groups[index];
      try {
        results[index] = { ok: true, value: await work(items[index]!, worker) };
      } catch (error) {
        results[index] = { ok: false, error };
        failed = Math.min(failed, index);
      }
      hand();
    }
  };
  await Promise.all(workers.map((_, worker) => run(worker)));
  hand();
  const first = results[failed];
  if (first && !first.ok) throw first.error;
}
