# Alternatives review

Checked 2026-10-15 against every tool in `home/*.nix`, `system/*.sh`, `config/` and `docs/CONTRACTS.md`. The question: is there a newer rewrite or alternative that is simply faster or lighter, and also actively maintained and actually used?

## The bar

An alternative is recommended only if all of these hold:

- **Maintained:** a commit in the last ~3 months and a release in the last ~6 months, with issues and PRs being handled.
- **Used:** packaged in nixpkgs 26.05 and/or Ubuntu 26.04, Arch or Debian, with a real user base (Repology package count is quoted as "repos").
- **Not vibecoded:** at least ~2 years old or clearly established, several human contributors, a real issue tracker, CI and tests.
- **Better for this machine:** measurably faster or lighter, or much simpler, with no lost features, and installable from apt or nixpkgs 26.05 on Ubuntu 26.04 with Hyprland.

Decisions in `docs/PLAN.md` (tmux, oh-my-posh config, Docker Engine, power-profiles-daemon, snapper, GRUB, own Quickshell shell) were treated as fixed unless something was strictly better.

## Sources and method

- Maintenance: GitHub API on 2026-10-15 (last commit on the default branch, latest release or tag, contributor count, last merged PR).
- Adoption and versions: Repology API (`repology.org/api/v1/project/<name>`), packages.ubuntu.com for `resolute`, and the `nixos-26.05` branch of nixpkgs.
- Ubuntu 26.04 kernel facts: `/boot/config-7.0.0-38-generic` from the `linux-modules-7.0.0-38-generic` package (arm64 build, same config family as amd64).
- Own measurements: Docker on an Apple Silicon Mac (arm64, 8 vCPU, `nixos/nix` image, nixpkgs `nixos-26.05`, hyperfine 1.20). Other benchmark containers were running at the same time, so treat absolute numbers as noisy and the ratios as the signal. Nothing was measured on the TUXEDO itself.

### Measurements

| Test | Result |
|---|---|
| zsh `zsh -i -c exit`, 150 runs, all plugins of `home/shell.nix` (OMZ `git`, autosuggestions, fzf, direnv, atuin, oh-my-posh, zoxide) | 33.0 ms ± 5.3 |
| Same without oh-my-zsh (compinit + OMZ `lib/*.zsh` and `plugins/git` sourced directly) | 29.1 ms ± 11.6 (22 ms in a quieter run) |
| Same, with the `init` outputs of fzf/direnv/atuin/oh-my-posh/zoxide pre-generated and sourced (no forks) | 22.3 ms ± 3.1 (15 ms in a quieter run) |
| Bare zsh with compinit only | 5.8 ms |
| Prompt render in a dirty git repo: `oh-my-posh print primary` (repo's `config/ohmyposh/config.toml`) vs `starship prompt` (defaults) | 11.3 ms vs 11.4 ms |
| Rust relink after `touch main.rs`, clang + mold 2.41 vs clang + wild 0.8.0 (both from nixpkgs 26.05), debug build | ripgrep 14.1.1: 294 ms vs 256 ms (1.15x). helix 25.07.1 (350 MB debug binary): 2.02 s vs 1.60 s (1.26x) |
| jq 1.8.2 vs jaq 3.1.0 vs gojq 0.12.19, 72 MB JSON, `length` | 1.09 s vs 0.48 s vs 0.48 s |
| Same, `[.[]\|select(.score>50)\|.nested.y]\|add` | 1.17 s vs 0.76 s vs 0.59 s |
| Same, tiny file `.b` | 0.74 ms vs 0.57 ms vs 0.64 ms |
| fzf 0.72 vs skim 4.7, `--filter` over 1.01 M lines | 125 ms vs 107 ms wall, but fzf used 230 ms CPU vs skim 340 ms |

Takeaways: oh-my-zsh costs a few milliseconds once its cache exists, and the whole shell starts well below the 50 ms first-prompt threshold that zsh-bench treats as imperceptible. oh-my-posh and starship render equally fast. wild gives a real but moderate relink gain. jq is 2x slower than jaq/gojq only on large files.

## Shell layer

| Current | Verdict | Alternative | Evidence | Migration cost / risks |
|---|---|---|---|---|
| oh-my-zsh (`git` plugin) | Keep | antidote, zinit, plain zsh + OMZ libs | OMZ: commit 2026-10-13, 399 contributors, nixpkgs 26.05 `2026-02-19`. Measured cost over plain zsh: ~4 to 7 ms. home-manager already skips its own `compinit` when OMZ is on (`modules/programs/zsh/default.nix`, release-26.05). antidote: release 2.3.0 2026-08, but nixpkgs 26.05 has 1.10.3 and Ubuntu 1.10.0. zinit: active (v3.17.1 2026-10-10) but complex turbo-mode config | Not worth it: the gain is below perception |
| oh-my-posh | Keep | starship | oh-my-posh v31.8.0 2026-10-12, 425 contributors, nixpkgs 26.05 29.14.0. starship v1.26.0 2026-06-28, 78 repos. Measured 11.3 vs 11.4 ms per prompt | Switching would mean porting the config and the matugen template for zero gain |
| zsh-autosuggestions | Keep (watch) | none that pass | Last release v0.7.1 2024-11, last commit 2025-06-24: fails the "maintained" bar, but it is feature-complete and packaged in 74 repos (nixpkgs 0.7.1, Ubuntu 0.7.1). No maintained drop-in exists | None |
| fzf | Keep | skim | fzf v0.74.5 2026-10-11, 110 repos. skim v5.7.4 2026-10-04 but nixpkgs 26.05 ships 4.7.0 and Ubuntu has none. Measured: equal wall time, fzf uses ~30% less CPU | skim would lose `fzf --zsh` integration and home-manager wiring |
| zoxide | Keep | none | v0.10.0 2026-07-04, 98 repos | None |
| atuin | Keep | none | v18.23.0 2026-09-22, 323 contributors, nixpkgs 18.15.2 | None |
| direnv + nix-direnv | Keep | mise (`[env]`) | direnv v2.38.2 2026-10-10; nix-direnv 3.2.0 2026-07-28. mise v2026.10.11 is very active but does not load Nix flake devshells, which is what nix-direnv is for; proto already owns team toolchains | Overlap with proto |
| Shell init forks | Config tweak (optional) | Pre-generated init scripts | Measured 29 ms to 22 ms by sourcing cached `init` output instead of five `eval "$(tool init zsh)"` forks | Must regenerate on every tool update; home-manager modules for fzf/atuin/direnv emit the evals themselves. Low value |

## CLI tools

| Current | Verdict | Alternative | Evidence | Migration cost / risks |
|---|---|---|---|---|
| eza | Keep (watch) | lsd | eza v0.23.5 2026-07-09, last merged PR 2026-07-09, issues still closed (2026-09-23), 91 repos. Slowing but alive. lsd is not faster | None |
| bat | Keep | none | Last release v0.26.1 2025-12-02 but commits 2026-10-14, 111 repos | None |
| fd | Keep | none | v10.5.0 2026-08-26, 109 repos | None |
| ripgrep | Keep | none | 15.2.0 2026-07-15, 122 repos | None |
| delta | Keep | difftastic (addition, not replacement) | delta 0.20.1 2026-10-04. difftastic 0.71.0 2026-09-18, 368 contributors, nixpkgs 0.69.0, not in Ubuntu. difftastic is a structural diff, slower and not a pager; complementary as `git difftool` | Optional addition only |
| dust | Keep | none | v1.2.6 2026-09-16, 84 repos | None |
| duf | Switch (low priority) | dysk | duf: last release v0.9.1 2025-09-08, last merged PR 2025-09-30: fails the maintenance bar. dysk (by Canop, author of bacon and broot): v3.7.1 2026-09-27, 47 repos, nixpkgs 26.05 3.6.1, Ubuntu 3.6.0. Both are instant; this is about maintenance, not speed | Different column layout; `dysk --json` exists. Trivial swap |
| btop | Keep | bottom | btop v1.4.7 2026-05-01, commits 2026-10-15, 107 repos. bottom 0.14.9 is active but not in Ubuntu, nixpkgs 0.14.2; no measured advantage | Config in `config/btop` would be lost |
| jq | Keep | jaq, gojq | Measured: jaq/gojq ~2x faster on a 72 MB file, equal (sub-ms) on normal API-sized input. jaq 3.1.0 has documented semantic differences from jq; gojq has no Ubuntu-current build (0.12.13). jq 1.8.2 is maintained (2026-06-20, 164 repos) | Scripts and muscle memory target jq |
| yq (yq-go) | Keep | none | v4.54.1 2026-09-29 | None |
| xh | Keep | none | v0.26.2 2026-07-26 | None |
| tealdeer | Keep | none | v1.9.0 2026-08-24 | None |
| just | Keep | none | 1.59.0 2026-10-15 | None |
| glow | Keep | none | v3.0.0 2026-08-11 | None |
| gh | Keep | none | v2.102.0 2026-09-30 (nixpkgs has the same; Ubuntu 2.46.0 is old, so keep it in Nix) | None |
| fastfetch | Keep | none | 2.70.0 2026-10-14 | None |
| lazydocker | Keep (watch) | none that pass | Last release and last commit v0.25.2 2026-04-19, last issue closed 2026-01: borderline on the maintenance bar. No maintained TUI alternative with Ubuntu or nixpkgs adoption | None |
| lazygit | Keep | gitui | lazygit v0.66.1 2026-10-11, 360 contributors. gitui last release v0.28.1 2026-03-24 (over 6 months), and a smaller feature set | Would lose `config/lazygit` |
| yazi | Keep | none | v26.9.1 2026-09-01, 185 contributors | None |
| tms | Keep | sesh | tms v0.6.1 2026-05-29, merged PR 2026-10-12, 24 contributors, nixpkgs 0.5.0. sesh v2.33.0 2026-10-12, 66 contributors, nixpkgs 2.26.2, not in Ubuntu. Both pick a session instantly; no speed case | Rewrite `config/tms` for no gain |

## Terminal, multiplexer, editor

| Current | Verdict | Alternative | Evidence | Migration cost / risks |
|---|---|---|---|---|
| Ghostty | Keep | none | Commits daily (2026-10-15), 422 contributors; last tag v1.3.1 2026-03-13, so the 6-month release clock is close, but tip builds are the documented channel. nixpkgs 1.3.1 | None |
| tmux | Keep (user choice) | none tmux-compatible | tmux 3.8 2026-09-09 upstream, nixpkgs 26.05 3.6a | None |
| Neovim 0.12 + LazyVim | Keep | none | Neovim v0.12.6 2026-10-09; LazyVim v16.0.1 2026-09-08 | None |
| Completion | Keep | already blink.cmp | `config/nvim/lazy-lock.json` has blink.cmp and no nvim-cmp. blink.cmp commits 2026-10-15 | None |
| Picker | Keep | already snacks picker | `lazyvim.json` enables `editor.snacks_picker`. telescope.nvim is still installed only because `lua/plugins/jsonfly.lua` and scretch.nvim (`lua/plugins/scratch.lua`) use it; it lazy-loads, so no startup cost. snacks.nvim last release v2.31.0 2026-03-20, last commit 2026-05-25 (slow, but issues are still handled, last closed 2026-10-13) | Optional cleanup only |
| lazy.nvim | Keep | none | v11.17.5 2025-11, last commit 2025-12; quiet but stable and required by LazyVim | None |

## Rust and dev

| Current | Verdict | Alternative | Evidence | Migration cost / risks |
|---|---|---|---|---|
| mold (apt 2.40.4) | Consider | wild | wild: 0.10.0 2026-08-04, releases every ~3 months (0.5 to 0.10 since 2025-05), commits daily, 59 contributors including Martin Liska (SUSE toolchain), Rust Foundation Rust Innovation Lab project, repo age 2.7 years, CI and a large test suite. nixpkgs 26.05 has `wild-unwrapped` 0.8.0 plus the `useWildLinker` stdenv adapter; not in Ubuntu or Debian. Upstream benchmark (Ryzen 9 9955HX, linker only): mold 2.41 takes 2.5x as long as wild 0.10 on rust-analyzer and 2.3x on zed-debug. Measured here (whole incremental cargo rebuild): 1.15x on ripgrep, 1.26x on helix (0.4 s saved per rebuild) | Pre-1.0, no incremental linking yet, smaller user base than mold. Keep mold installed as the fallback |
| mold upstream | Note | mold 3.0 | mold 3.0.0 (2026-10-05) is a Rust rewrite, "performance on par with 2.42.1". Not in Ubuntu or nixpkgs 26.05 yet. Nothing to do | None |
| sccache | Keep | none | v0.18.0 2026-09-14, 83 repos, nixpkgs 0.15.0 | None |
| cargo-nextest | Keep | none | 0.9.148 2026-10-08 | None |
| bacon | Keep | none | v3.26.0 2026-09-26 | None |
| hyperfine | Keep | none | v2.0.0 2026-10-07 (nixpkgs 1.20.0) | None |
| clang (apt) | Keep | none | Ubuntu 26.04 ships LLVM 21 | None |

## Desktop

| Current | Verdict | Alternative | Evidence | Migration cost / risks |
|---|---|---|---|---|
| fuzzel, foot (fallbacks) | Keep | none | fuzzel 1.15.0, foot 1.28.0 upstream; Ubuntu 1.12.0 and 1.25.0. Fallbacks must stay on apt | None |
| grim, slurp | Keep | none | grim 1.5.0 / slurp 1.6.0 in Ubuntu 26.04, 96 and 93 repos; wrappers like grimblast still call them | None |
| gpu-screen-recorder (added 2026-10-16) | Adopt | wf-recorder | gsr: tag 6.1.4 2026-10-12, 126 commits since 2026-07-16 (git.dec05eba.com), mostly one author (dec05eba) since 2020; nixpkgs 26.05 5.13.8 (2026-05), not in Ubuntu 26.04. Encodes on the GPU (VA-API on the Radeon 890M), has a RAM replay buffer. wf-recorder: last commit 2026-04-12, last release v0.6.0 2025-10-17: fails the maintenance bar; kept only as the script's no-GPU path (the test harness) | KMS capture needs a setcap'd helper (`system/65-recorder.sh`); the portal route waits for an xdg-desktop-portal-hyprland release with commit ba31964 (2026-08-29) |
| satty (added 2026-10-16) | Adopt | swappy | Satty-org/Satty v0.22.0 2026-08-03, commit 2026-10-13, 2.4k stars; nixpkgs 26.05 0.20.1. swappy is slower moving | None |
| wl-clipboard | Keep | none | v2.3.0 2026-03-22, commits 2026-09 | None |
| cliphist (not installed) | Keep as the planned backend | none | Release v0.7.0 2025-10-11 is a year old, but commits 2026-10-07 and 54 repos. If clipboard history is added, take nixpkgs 0.7.0, not Ubuntu's 0.5.0 | None |
| hypridle, hyprlock, hyprpolkitagent | Keep | none | hyprwm: hypridle v0.1.8 2026-07, hyprlock v0.9.6 2026-07, hyprpolkitagent v0.2.0 2026-09. Ubuntu has older (0.1.7, 0.9.2, 0.1.3); the PPA provides them | None |
| matugen | Keep | none | v4.2.0 2026-08-17, nixpkgs 4.0.0 | None |
| keyd (added 2026-10-24, Mac-style accents on Left Alt) | Adopt | us(mac) layout, kanata, xremap, an input method (fcitx5) | keyd 2.5.0 in Ubuntu 26.04 universe, a small C daemon on evdev and uinput, below the compositor. us(mac) changes the user's layout, which the feature must not. kanata and xremap aren't in Ubuntu. An input method is a second daemon in every app. keyd's own unicode output types a compose sequence with the US layout's keys, so KubaOS doesn't use it: keyd sends a dead key on a keycode no layout uses, and the `kubaos:mac-accents` XKB option gives it its symbol in every layout | One root daemon over every keyboard; turning the accents off stops it (a polkit rule for `keyd.service`) |
| Quickshell, PipeWire/WirePlumber | Keep | n/a | User's own shell; PipeWire is the standard | None |

## System

| Current | Verdict | Alternative | Evidence | Migration cost / risks |
|---|---|---|---|---|
| Nix (official installer) | Keep | Lix | Nix 2.35.3 2026-10-13. Lix 2.96.0 2026-10-13 (git.lix.systems, GitHub mirror), nixpkgs 26.05 ships Lix 2.94.2, not in Ubuntu. Lix publishes no benchmark numbers for its performance work | Different installer and daemon, flake behaviour differences; no measured gain |
| GRUB + grub-btrfs | Keep | systemd-boot | systemd-boot is in Ubuntu 26.04, but it has no equivalent of grub-btrfs's snapshot menu, which the rollback plan depends on. grub-btrfs: last release v4.14 2025-09, last merge 2026-08-24, 25 repos (not in Ubuntu) | Would break snapshot boot |
| snapper | Keep | btrbk, Timeshift | snapper v0.13.2 2026-09-07 upstream, Ubuntu 0.10.6. btrbk (0.32.6 in Ubuntu) is for send/receive backups, not a replacement; useful later for off-disk copies. Timeshift expects its own `@`/`@home`-only layout | None |
| NetworkManager + wpa_supplicant | Keep | iwd backend | iwd 3.10 in Ubuntu. Reports of MT7921/MT7922 (`mt7921e`) failing 5 GHz 4-way handshakes under iwd while wpa_supplicant works ([NixOS Discourse](https://discourse.nixos.org/t/unsolved-mt7921-iwd-4-way-handshake-timeout-reason-15-on-5ghz-wpa-supplicant-works-fine/77470), [Arch forum](https://bbs.archlinux.org/viewtopic.php?id=305030)); one Framework MT7922 owner reports faster boot-time connect with iwd ([Framework forum](https://community.frame.work/t/responded-mt7922-wifi-slow-to-start-on-boot/42605/12)). Gain is seconds at connect; risk is a broken link | Profiles must be recreated |
| power-profiles-daemon | Keep | tuned-ppd | Ubuntu 26.04 ships ppd 0.30 (default) and tuned-ppd 2.25.1. A Fedora 44 test on Panther Lake measured ~19% lower benchmark scores with tuned-ppd than with ppd ([comss.ru](https://www.comss.ru/page.php?id=21569), secondary source, Intel). ppd drives `platform_profile` plus amd-pstate EPP, which is what tuxedo-drivers exposes | None |
| Docker Engine | Keep (user choice) | Podman | Noted only | n/a |
| 8 GB swapfile, zswap off | Config tweak (optional) | enable zswap | Ubuntu 26.04 kernel: `# CONFIG_ZSWAP_DEFAULT_ON is not set`, default compressor `lzo`, `CONFIG_ZRAM=m`; Ubuntu does not set up zram by default. With 64 GB RAM swap is rarely touched; zswap compresses pages in RAM in front of the existing swapfile, which matters only under pressure (big Rust builds + Docker). zram would duplicate the swapfile's role | Kernel parameter only; easy to revert |
| systemd-oomd | Keep off | enable it | Hyprland is started from tty1 without uwsm, so apps are not in per-app scopes; oomd acts on cgroups and would kill the whole session scope. The kernel OOM killer picks single processes | n/a |
| Kernel | Keep | HWE | `linux-image-generic` and `linux-image-generic-hwe-26.04` both point to 7.0.0-38 today; Strix Point is supported. `30-hardware.sh` already handles HWE headers if you switch later | None now |
| I/O scheduler | Keep | none | NVMe uses `none` by default; nothing to gain | None |
| irqbalance | Keep as is | none | Not installed by this repo; single-socket laptop | None |
| Btrfs options `noatime,compress=zstd:1` | Keep | add `discard=async`, `ssd`, `space_cache=v2` | btrfs docs: `discard=async` is the default since kernel 6.2 when the device supports it, `ssd` is autodetected, `space_cache` defaults to v2 ([btrfs mount options](https://btrfs.readthedocs.io/en/latest/ch-mount-options.html)). Adding them changes nothing | None |

## Recommended changes

Ranked by benefit over effort. None of these is urgent; the current set already passes the bar almost everywhere.

### 1. Pilot wild as the Rust linker (Consider)

Expected gain: about 20 to 25% faster incremental debug rebuilds on large binaries in this test (0.4 s per rebuild on helix); upstream reports 2.3 to 2.5x faster linking than mold on linker-only benchmarks. Effort: small. Risk: pre-1.0 linker bugs; fall back to mold.

- `home/dev.nix`: in `linuxTarget`, replace `"link-arg=-fuse-ld=mold"` with `"link-arg=--ld-path=${lib.getExe pkgs.wild-unwrapped}"`. Use `wild-unwrapped`, not `wild`: the wrapped variant is a nixpkgs bintools wrapper meant for Nix stdenv, and on Ubuntu the pure binary is what clang should call. Keep `linker = "clang"` (clang from apt).
- Keep `mold` in `system/20-base.sh` as the fallback. For a one-off build with mold: `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build`.
- Update the comment at the top of `home/dev.nix` ("clang and mold come from apt") and add `wild` to the Nix list in `docs/CONTRACTS.md`, "Packages: who installs what" (lead owns that file).
- Verify on the laptop: `readelf --string-dump .comment target/debug/<bin> | grep -i wild`, then `cargo build --timings` on `arms` with both linkers.

### 2. Replace duf with dysk (Switch, low priority)

Gain: a maintained tool instead of one with no release or merge for over a year. No speed difference. Effort: trivial.

- `home/cli.nix`: replace `duf` with `dysk`.
- `docs/CONTRACTS.md` and `docs/PLAN.md` ("Tools in `cli.nix`"): rename duf to dysk.

### 3. Enable zswap (Config tweak, optional)

Gain: smoother behaviour only under memory pressure; nothing in normal use. Effort: small.

- `system/20-base.sh`: write `/etc/default/grub.d/90-zswap.cfg` containing `GRUB_CMDLINE_LINUX_DEFAULT="$GRUB_CMDLINE_LINUX_DEFAULT zswap.enabled=1"` (use the existing `write_root_file` helper), then `update-grub` (the script already runs it later in the snapper section, so place this step before that).
- Check after reboot: `cat /sys/module/zswap/parameters/enabled` prints `Y`. Leave the compressor at the kernel default (`lzo`) unless you have confirmed that `zstd` or `lz4` is built into the kernel and not a module; a missing module makes zswap fall back or stay off at boot.

### 4. Add difftastic as a difftool (optional addition)

Gain: syntax-aware diffs on demand; delta stays the pager. Effort: trivial.

- `home/cli.nix`: add `difftastic` to `home.packages`, and in the `git-nix/delta.gitconfig` attrset add `diff.tool = "difftastic"`, `difftool.prompt = false`, `"difftool \"difftastic\"".cmd = "difft \"$LOCAL\" \"$REMOTE\""`. Use with `git difftool`.

### Not recommended despite a measured gain

- Removing oh-my-zsh or caching init scripts: 4 to 7 ms each, well under the threshold anyone notices, and caching adds a regeneration step to every tool update.
- jq to jaq: only faster on files of tens of MB; semantics differ.

## Rejected candidates

| Candidate | Reason |
|---|---|
| starship | Same render time as oh-my-posh (11.4 vs 11.3 ms); would mean porting the prompt and its matugen template |
| antidote, zinit | OMZ costs only a few ms here; nixpkgs 26.05 antidote is 1.10.3 (upstream 2.3.0); zinit's turbo mode adds complexity |
| skim | No speed gain over fzf, more CPU; nixpkgs 26.05 lags (4.7.0 vs 5.7.4); not in Ubuntu |
| jaq, gojq | Faster only on huge inputs; jq-compat differences; gojq in Ubuntu is 0.12.13 |
| bottom | No advantage over btop for this use; not in Ubuntu |
| gitui | Last release 2026-03-24 (over 6 months); fewer features than lazygit |
| sesh | No speed case over tms; would mean rewriting `config/tms` |
| lsd | Not faster than eza |
| mise env | Does not replace nix-direnv for flake devshells; proto already pins toolchains |
| Lix | No published benchmarks; replaces the Nix installer and daemon for no measured gain |
| tuned-ppd | Not Ubuntu's default; a reported 19% regression vs ppd on Fedora 44 |
| iwd backend | Reports of 5 GHz handshake failures on the same MediaTek driver family |
| systemd-boot | No snapshot boot menu; the rollback plan needs grub-btrfs |
| Timeshift | Assumes its own two-subvolume layout; snapper already fits |
| zram | Would duplicate the swapfile; zswap is the better fit with an existing swapfile and 64 GB |
| systemd-oomd | Would kill the whole tty-launched Hyprland session scope |
| mold 3.0 | Not packaged yet; same speed as 2.x by its own release notes |

## Could not verify

- Any number on the real machine (x86_64 Strix Point): all timings above are arm64 Docker on a Mac, with other containers running.
- That the TUXEDO's Wi-Fi card is actually an MT7922, and iwd behaviour on it.
- Which zswap compressors are built in (not modules) in the amd64 Ubuntu 7.0 kernel.
- Whether wild 0.8.0 (nixpkgs 26.05) handles every crate in `arms`; upstream 0.10.0 is newer and is what its benchmarks use.
