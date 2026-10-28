# Design PR loop

How a PR that changes how something looks gets its screenshots and its review. The tool is `just shots` (`test/shots/shots`, pinned by the flake as `nix run .#shots`).

## The loop

1. **Render while you iterate: `just shots`.** With no arguments it renders only the surfaces and specimen pages whose sources changed against `origin/main`, at 1× and 1.5×, dark. A change to a shell module renders the surfaces it draws, a change to one page of the specimen that page; a change to a shared input renders everything (below). It renders your working tree as AFTER, uncommitted files included (and, for the shell, untracked ones), and the merge base with `origin/main` as BEFORE.
2. **Look at `test/out/shots/changed.txt`.** It lists only the pairs that differ; the pairs are in `test/out/shots/pairs/` (before left, after right, cropped round the surface, at device pixels). Fix any QML warning it prints: they are the AFTER side's, in `warnings.txt`.
3. **Run `just shots --full` once before the PR is ready**: every surface at 1, 1.25, 1.5 and 2, light and dark. Run `just check` in `nix develop` too: the formatting, clippy, the tests and the design checks, as the PR's CI runs them (`docs/agents/code-quality.md`). CI leaves the shots out, and the end-to-end checks: run `just e2e` on the machine (`docs/TESTING.md`), or `just e2e quickshell notifications menu` for the checks the change reaches, and put its results table in the PR.
4. **Review the changed pairs against `docs/agents/design-review.md`**, `docs/DESIGN.md` and the pages of the components it touches (`docs/design/components/`). Run your reviewer sub-agent in the foreground (`run_in_background: false` on the Agent tool), so its report comes back to you and not to the coordinator. Fix what it finds and render again.
5. **Attach the pairs: `just shots publish chore/shots-<your-branch>`.** It pushes `changed.txt`'s pairs to that image-only branch and prints the Markdown table; paste it into the PR body, with each visible change listed as a decision for Kuba.

## Commands

| Command | What |
|---|---|
| `just shots` | The surfaces and specimen pages changed against `origin/main`, 1× and 1.5×, dark |
| `just shots --all` | Every surface and page, whatever changed |
| `just shots controlcenter menu` | Those surfaces, whatever changed |
| `just shots --scales 1,2 --modes dark,light` | Other scales or modes |
| `just shots --ref REV` | BEFORE is the merge base with REV |
| `just shots --full` | Every scale and mode, of the surfaces named or, without names, of those that changed |
| `just shots plymouth`, `just shots grub`, `just shots cursor` | The boot splash and the disk prompt (plymouthd's X11 renderer in Xvfb, in a bubblewrap sandbox), the GRUB menu and shell (QEMU under OVMF), and the pointer's XCursor sheets on dark and light (`kubaos cursor` in a home of its own), from `test/shots/boot`. Plymouth and GRUB render at 1 (1920x1080) and 2 (2880x1800), dark; the pointer at every scale |
| `just shots kobo`, `just shots specimen`, `just shots specimen/components` | Kōbō's tour (`KOBO_SHOTS`, `cargo test -p kubaos-kobo --release shots`) and the specimen's pages (`kubaos-specimen shots DIR PAGE...`, `qml-specimen shots`), every page or the one named: `specimen/type`, `colour`, `space`, `components`, `icons`, `motion`, or `specimen/qml` for qml-specimen's components page. They go into the same pairs and `changed.txt`. They draw every scale and mode in release builds, from under a minute for one page to a few for the tour; the pairs keep the scales and modes asked for. `qml-specimen` reads the tree as a git flake: `git add` a new QML file first. The repo keeps no pictures of the specimen, so these pairs are how a PR shows its change; CI publishes main's to `chore/shots-specimen` (`docs/DESIGN.md`, "Screenshot diffs") |
| `just shots publish BRANCH` | Push the changed pairs to BRANCH, print the table (raw.githubusercontent URLs, pinned to the commit) |
| `just shots self-test` | Check a render can't reach the desktop session |

The shell's surfaces: `bar` (the tray holds Adwaita's unfilled symbolic icon, one filled #2e3436 and a colour one; and the bell crossed out, with and without notifications, and without a bell), `controlcenter` (Wi-Fi tab, the Bluetooth tab with a device pairing, its passkey prompt, the Keyboard tab), `clock` (with an answer to a typed question), `sysmon` (and a process's details), `wallpaper` (and its settings), `powermenu`, `osd` (volume and brightness), `notifications` (two popups, the bell's peek, and the panel: with a stack folded and open, its keys tooltip, empty, and Do not disturb on while the screen is shared with its tooltip), `lock` (and a wrong password, the security key asked for, and the idle screen over a bright and over a busy wallpaper), `polkit` (and a failed password), `menu` ("fi" typed, and its Action panel). Each is a line in `test/shots/scenarios`.

## What a change renders

`test/shots/surfaces` is the one table of what each surface and page is drawn from: a line per surface, `SURFACE PATH...`. A changed file renders the lines that list its most specific match, so `crates/kubaos-specimen/src/pages/components/buttons.rs` renders `specimen/components` alone and the rest of `crates/kubaos-specimen` every page. Three lines are groups: `shell` (every shell surface: the shared shell components and theme, the stand-ins, the drivers), `specimen` (every page), and `*`, the shared inputs, which render everything: `design/tokens.toml` and its generator, `crates/kubaos-ui` with its fonts and symbols, `crates/kubaos-geometry`, `qml/KubaOS/Components`, `Cargo.lock`, `flake.lock`, and the harness itself (`shots`, `lib.sh`, `surfaces`, `package.nix`). A file no line lists renders nothing, as the shell's services do: they are stand-ins.

A new surface (a line in `scenarios`, a page of the specimen) gets its own line, or `just shots-test` fails: every surface has sources, every source exists, and every page `kubaos-specimen pages` lists is in the table. A path under a `*` path listed by another line fails it too, as it would render that line alone.

## How it renders

- **The real QML, stand-in services.** The frame driver loads the tree's own `Frame.qml` (bar, border, shadow, scrim and every panel) and opens a panel through `Panels`, as the shell does. Quickshell's modules, Kotodama and the shell's services are stand-ins with fixed data (`test/shots/fakes`): a frozen clock (Tuesday 27 October 2026, 21:47), Wi-Fi networks, Bluetooth devices, audio, battery, power, notifications, tray, workspaces, proxy and the system monitor. A change to `config/quickshell/services` doesn't show: those files aren't loaded.
- **Drivers only use what a panel exposes** (`Panels.current`, `tab`, `show()`, `openActions()`, the services), so they work across versions of the components. A new state is a new line in `scenarios`, and, if it needs one, a branch in its driver (`test/shots/drivers`).
- **Headless and private.** Each render is Qt's `qml` runtime in its own headless sway (pixman, Mesa's llvmpipe for Qt), on its own runtime dir, started from an emptied environment: no `WAYLAND_DISPLAY`, `HYPRLAND_INSTANCE_SIGNATURE`, `DISPLAY` or `QS_*` of the desktop reaches it, and neither `qs` nor `quickshell` is on its `PATH`. It never touches the live session. `just shots self-test` checks this.
- **Pinned.** Qt, sway, Mesa, ImageMagick, the fonts (kubaos-ui's Geist and Geist Mono, M PLUS 1, Noto Color Emoji, DejaVu for symbols) and the wallpapers come from the flake's lock (`test/shots/package.nix`).
- **Parallel**, one render per core. Animations run on the clock, and a shot waits until its window stops drawing, so a busy machine gives the same pixels; a window still drawing after 20 s fails its shot.
- **Cached BEFORE.** BEFORE shots are kept in `~/.cache/kubaos/shots/<rev>/<harness>/`, keyed by the base commit and a hash of `test/shots` and the pinned tools' store paths, and reused; a later run only renders what is missing, down to the specimen's page. A shot is written whole or not at all, so agents rendering the same base at once share the cache safely. Kōbō's and the specimen's BEFORE depend on the commit alone (`<rev>/iced/`) and build in `~/.cache/kubaos/shots/target`.
- **Pruned after each run** (`test/shots/lib.sh`, `prune`). The cache keeps the trees of the four base commits last used: each agent renders against the main it branched from, and the coordinator runs about four at once. It drops the rest, and the trees of a base no longer on `main`, `origin/main` or a local branch. In a base it keeps, it keeps the BEFORE shots of the two harness versions last used (main's, and a branch's that changes the harness). A run holds its base under a lock for as long as it renders, and a base held is never dropped. Scratch a killed run left in `tmp/` goes after a day, and Qt's compiled QML (`xdg/QtProject`), which only the renders of one run read, after an hour. The shared cargo `target/` stays: the next base builds on it.
- **The diff** counts the pixels where a channel differs by more than 0.5%. A pair is in `changed.txt` when its size changed or more than `--threshold` logical pixels differ (default 32; a text caret's blink is about 20).
- **Not deterministic:** Kōbō's installing screens show its tour's progress, which runs on the clock, so they can differ between two runs of the same tree.

## Compile cache

`nix develop` pins the toolchain (`flake.lock`'s rustc, cargo, clippy, nextest and the nightly rustfmt) and runs cargo through sccache (`RUSTC_WRAPPER`, cache in `~/.cache/sccache`), shared by every checkout, so a fresh worktree reuses the crates.io dependencies another one built. The workspace's own crates still compile in each worktree: cargo builds them incrementally, which sccache doesn't cache, and their hash includes the checkout's path. Crane and the flake's packages build in the sandbox, without it.
