// The mockup site's build: plain TypeScript and CSS, with the tokens' CSS generated from
// `design/tokens.toml` by `kubaos-tokens` before the site is served or built, and again whenever
// the tokens change, so a page follows them live. The dev server also serves the media player's
// film from the cache `just mockup-film` writes it to, outside every checkout.
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin, type ViteDevServer } from "vite";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const tokens = path.join(repo, "design/tokens.toml");

/**
 * Writes the token outputs git ignores into the checkout, `src/tokens.css` among them, as `just
 * tokens` does for an editor. Returns the generator's error, or null.
 */
function generate(): string | null {
  const run = spawnSync("cargo", ["run", "-q", "-p", "kubaos-tokens", "--", "build", repo, "--root", repo], {
    cwd: repo,
    encoding: "utf8",
  });
  if (run.error) return `kubaos-tokens didn't run: ${run.error.message}; run the site in \`nix develop\``;
  return run.status === 0 ? null : run.stderr || `kubaos-tokens exited ${run.status}`;
}

function tokensPlugin(): Plugin {
  return {
    name: "kubaos-tokens",
    buildStart() {
      const failed = generate();
      if (failed) this.error(failed);
    },
    configureServer(server: ViteDevServer) {
      server.watcher.add(tokens);
      server.watcher.on("change", (file) => {
        if (path.resolve(file) !== tokens) return;
        const failed = generate();
        if (failed) server.ws.send({ type: "error", err: { message: failed, stack: "" } });
      });
    },
  };
}

/** Where `just mockup-film` writes the film, as `scripts/film.sh` says. */
const filmDir = process.env.KUBAOS_MOCKUP_FILM || path.join(process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache"), "kubaos-mockups/film");
const filmManifest = path.join(filmDir, "sintel.json");
/** Where the dev server serves the film's files. */
const FILM_URL = "/@kubaos-film/";
const FILM_MODULE = "virtual:kubaos-film";
const FILM_RESOLVED = `\0${FILM_MODULE}`;

/** The film's manifest, with where its files are served; null when it isn't fetched or can't be read. */
function filmModule(): string {
  let manifest: unknown = null;
  try {
    manifest = { ...JSON.parse(fs.readFileSync(filmManifest, "utf8")), base: FILM_URL };
  } catch {
    manifest = null;
  }
  return `export default ${JSON.stringify(manifest)};`;
}

/**
 * Serves the film's files by range, as a video seeks: the cut and its subtitles, nothing else of
 * the folder.
 */
function serveFilm(req: IncomingMessage, res: ServerResponse, next: () => void): void {
  const name = (req.url ?? "").split("?")[0]!.replace(/^\//, "");
  if (!/^sintel(\.webm|\.[a-z]{3}\.vtt)$/.test(name)) return next();
  const file = path.join(filmDir, name);
  let size: number;
  try {
    size = fs.statSync(file).size;
  } catch {
    return next();
  }
  res.setHeader("Content-Type", name.endsWith(".webm") ? "video/webm" : "text/vtt; charset=utf-8");
  res.setHeader("Accept-Ranges", "bytes");
  res.setHeader("Cache-Control", "no-cache");
  const range = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range ?? "");
  let start = 0;
  let end = size - 1;
  if (range && (range[1] || range[2])) {
    if (range[1]) {
      start = Number(range[1]);
      if (range[2]) end = Math.min(end, Number(range[2]));
    } else start = Math.max(0, size - Number(range[2]));
    if (start > end) {
      res.statusCode = 416;
      res.setHeader("Content-Range", `bytes */${size}`);
      return void res.end();
    }
    res.statusCode = 206;
    res.setHeader("Content-Range", `bytes ${start}-${end}/${size}`);
  }
  res.setHeader("Content-Length", String(end - start + 1));
  if (req.method === "HEAD") return void res.end();
  fs.createReadStream(file, { start, end }).pipe(res);
}

/**
 * The media player's film (`src/pages/apps/media-player-parts.ts`): `virtual:kubaos-film` is its
 * manifest where the dev server serves it, and null in a static build, which never carries the
 * film. Fetching it while the site is served reloads the page with it.
 */
function filmPlugin(): Plugin {
  let serving = false;
  return {
    name: "kubaos-film",
    configResolved(config) {
      serving = config.command === "serve";
    },
    resolveId(id) {
      return id === FILM_MODULE ? FILM_RESOLVED : null;
    },
    load(id) {
      if (id !== FILM_RESOLVED) return null;
      return serving ? filmModule() : "export default null;";
    },
    configureServer(server: ViteDevServer) {
      server.middlewares.use(FILM_URL, serveFilm);
      server.watcher.add(filmManifest);
      const changed = (file: string) => {
        if (path.resolve(file) !== filmManifest) return;
        const graph = server.environments.client.moduleGraph;
        const made = graph.getModuleById(FILM_RESOLVED);
        if (made) graph.invalidateModule(made);
        server.ws.send({ type: "full-reload" });
      };
      for (const event of ["add", "change", "unlink"] as const) server.watcher.on(event, changed);
    },
  };
}

export default defineConfig({
  root: here,
  // Relative, so the built site opens from any directory it is served from.
  base: "./",
  plugins: [tokensPlugin(), filmPlugin()],
  // The fonts and the symbols are `kubaos-ui`'s own files, read where they are.
  server: { fs: { allow: [repo] } },
  build: { outDir: "dist", emptyOutDir: true, assetsInlineLimit: 0 },
});
