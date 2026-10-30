// The mockup site's build: plain TypeScript and CSS, with the tokens' CSS generated from
// `design/tokens.toml` by `kubaos-tokens` before the site is served or built, and again whenever
// the tokens change, so a page follows them live.
import { spawnSync } from "node:child_process";
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

export default defineConfig({
  root: here,
  // Relative, so the built site opens from any directory it is served from.
  base: "./",
  plugins: [tokensPlugin()],
  // The fonts and the symbols are `kubaos-ui`'s own files, read where they are.
  server: { fs: { allow: [repo] } },
  build: { outDir: "dist", emptyOutDir: true, assetsInlineLimit: 0 },
});
