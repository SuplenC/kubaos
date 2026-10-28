# Design PR loop

How a PR that changes how something looks gets its screenshots and its review. The tool is `just shots` (`test/shots/shots`, pinned by the flake as `nix run .#shots`).

## The loop

1. **Render while you iterate: `just shots`.** With no arguments it renders only the surfaces whose files changed against `origin/main` (the table in `test/shots/surfaces`), at 1× and 1.5×, dark. It renders your working tree as AFTER, uncommitted files included (and, for the shell, untracked ones), and the merge base with `origin/main` as BEFORE.
2. **Look at `test/out/shots/changed.txt`.** It lists only the pairs that differ; the pairs are in `test/out/shots/pairs/` (before left, after right, cropped round the surface, at device pixels). Fix any QML warning it prints: they are the AFTER side's, in `warnings.txt`.
3. **Run `just shots --full` once before the PR is ready**: every surface at 1, 1.25, 1.5 and 2, light and dark. Run `just check` in `nix develop` too: the formatting, clippy, the tests and the design checks, as the PR's CI runs them (`docs/agents/code-quality.md`). CI leaves the shots out, and the end-to-end checks: run `just e2e` on the machine (`docs/TESTING.md`), or `just e2e quickshell notifications menu` for the checks the change reaches, and put its results table in the PR.
4. **Review the changed pairs against `docs/agents/design-review.md`** and `docs/DESIGN.md`. Run your reviewer sub-agent in the foreground (`run_in_background: false` on the Agent tool), so its report comes back to you and not to the coordinator. Fix what it finds and render again.
5. **Attach the pairs: `just shots publish chore/shots-<your-branch>`.** It pushes `changed.txt`'s pairs to that image-only branch and prints the Markdown table; paste it into the PR body, with each visible change listed as a decision for Kuba.

## Commands

| Command | What |
|---|---|
| `just shots` | The surfaces changed against `origin/main`, 1× and 1.5×, dark |
| `just shots controlcenter menu` | Those surfaces, whatever changed |
| `just shots --scales 1,2 --modes dark,light` | Other scales or modes |
| `just shots --ref REV` | BEFORE is the merge base with REV |
| `just shots --full` | Every scale and mode, of the surfaces named or, without names, of those that changed |
| `just shots plymouth`, `just shots grub`, `just shots cursor` | The boot splash and the disk prompt (plymouthd's X11 renderer in Xvfb, in a bubblewrap sandbox), the GRUB menu and shell (QEMU under OVMF), and the pointer's XCursor sheets on dark and light (`kubaos cursor` in a home of its own), from `test/shots/boot`. Plymouth and GRUB render at 1 (1920x1080) and 2 (2880x1800), dark; the pointer at every scale |
| `just shots kobo`, `just shots specimen` | Kōbō's tour (`KOBO_SHOTS`, `cargo test -p kubaos-kobo --release shots`) and the specimen pages (`kubaos-specimen shots`, `qml-specimen shots`), into the same pairs and `changed.txt`. They draw every scale and mode in release builds, a few minutes; the pairs keep the scales and modes asked for. `qml-specimen` reads the tree as a git flake: `git add` a new QML file first. The repo keeps no pictures of the specimen, so these pairs are how a PR shows its change; CI publishes main's to `chore/shots-specimen` (`docs/DESIGN.md`, "Screenshot diffs") |
| `just shots publish BRANCH` | Push the changed pairs to BRANCH, print the table (raw.githubusercontent URLs, pinned to the commit) |
| `just shots self-test` | Check a render can't reach the desktop session |

The shell's surfaces: `bar` (the tray holds Adwaita's unfilled symbolic icon, one filled #2e3436 and a colour one), `controlcenter` (Wi-Fi tab with the three notifications scrolled to the middle, the Bluetooth tab with a device pairing, its passkey prompt, the Keyboard tab), `clock` (with an answer to a typed question), `sysmon` (and a process's details), `wallpaper` (and its settings), `powermenu`, `osd` (volume and brightness), `notifications`, `lock` (and a wrong password, the security key asked for, and the idle screen over a bright and over a busy wallpaper), `polkit` (and a failed password), `menu` ("fi" typed, and its Action panel). Each is a line in `test/shots/scenarios`.

## How it renders

- **The real QML, stand-in services.** The frame driver loads the tree's own `Frame.qml` (bar, border, shadow, scrim and every panel) and opens a panel through `Panels`, as the shell does. Quickshell's modules, Kotodama and the shell's services are stand-ins with fixed data (`test/shots/fakes`): a frozen clock (Tuesday 27 October 2026, 21:47), Wi-Fi networks, Bluetooth devices, audio, battery, power, notifications, tray, workspaces, proxy and the system monitor. A change to `config/quickshell/services` doesn't show: those files aren't loaded.
- **Drivers only use what a panel exposes** (`Panels.current`, `tab`, `show()`, `openActions()`, the services), so they work across versions of the components. A new state is a new line in `scenarios`, and, if it needs one, a branch in its driver (`test/shots/drivers`).
- **Headless and private.** Each render is Qt's `qml` runtime in its own headless sway (pixman, Mesa's llvmpipe for Qt), on its own runtime dir, started from an emptied environment: no `WAYLAND_DISPLAY`, `HYPRLAND_INSTANCE_SIGNATURE`, `DISPLAY` or `QS_*` of the desktop reaches it, and neither `qs` nor `quickshell` is on its `PATH`. It never touches the live session. `just shots self-test` checks this.
- **Pinned.** Qt, sway, Mesa, ImageMagick, the fonts (kubaos-ui's Geist and Geist Mono, M PLUS 1, Noto Color Emoji, DejaVu for symbols) and the wallpapers come from the flake's lock (`test/shots/package.nix`).
- **Parallel**, one render per core. Animations run on the clock, and a shot waits until its window stops drawing, so a busy machine gives the same pixels; a window still drawing after 20 s fails its shot.
- **Cached BEFORE.** BEFORE shots are kept in `~/.cache/kubaos/shots/<rev>/<harness>/`, keyed by the base commit and a hash of `test/shots` and the pinned tools' store paths, and reused; a later run only renders what is missing. A shot is written whole or not at all, so agents rendering the same base at once share the cache safely. Kōbō's and the specimen's BEFORE depend on the commit alone (`<rev>/iced/`) and build in `~/.cache/kubaos/shots/target`.
- **The diff** counts the pixels where a channel differs by more than 0.5%. A pair is in `changed.txt` when its size changed or more than `--threshold` logical pixels differ (default 32; a text caret's blink is about 20).
- **Not deterministic:** Kōbō's installing screens show its tour's progress, which runs on the clock, so they can differ between two runs of the same tree.

## Compile cache

`nix develop` pins the toolchain (`flake.lock`'s rustc, cargo, clippy, nextest and the nightly rustfmt) and runs cargo through sccache (`RUSTC_WRAPPER`, cache in `~/.cache/sccache`), shared by every checkout, so a fresh worktree reuses the crates.io dependencies another one built. The workspace's own crates still compile in each worktree: cargo builds them incrementally, which sccache doesn't cache, and their hash includes the checkout's path. Crane and the flake's packages build in the sandbox, without it.
