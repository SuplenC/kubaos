# KubaOS build and test commands. `just` lists them.
# Needs Docker; the vm recipes take QEMU from PATH or the flake's nixpkgs (test/vm/README.md).
# arch defaults to this machine's: x86_64 or aarch64.

host := `uname -m`
dirty := if env("ALLOW_DIRTY", "") == "" { "" } else { "--allow-dirty" }
# The release (docs/adr/0001-release-and-update-model.md, "Versions").
ver := trim(read(justfile_directory() / "VERSION"))
codename := trim(read(justfile_directory() / "CODENAME"))

# List the recipes.
default:
    @just --list

# Build the payload (the whole system as a squashfs) into install/out/payload-<arch>, then check it. One at a time.
payload arch=host:
    install/image/build --arch {{arch}} {{dirty}}
    install/image/check --arch {{arch}}

# Check a built payload (unpack + chroot checks).
payload-check arch=host:
    install/image/check --arch {{arch}}

# Build the live ISO into iso/ (builds the payload first if there isn't one, and checks it if it hasn't passed). `just iso x86_64 vm` makes the bench ISO (ssh, no firmware).
iso arch=host profile="release":
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{profile}}" in release|vm) ;; *) echo "profile is 'vm' or empty (release)" >&2; exit 2 ;; esac
    p=install/out/payload-{{arch}}
    [[ -f $p/manifest.json ]] || just payload {{arch}}
    [[ "$(cat "$p/check.ok" 2>/dev/null)" == "$(sha256sum "$p/manifest.json" | cut -d' ' -f1)" ]] || just payload-check {{arch}}
    if [[ "{{profile}}" == vm ]]; then flags=(--profile vm); else flags=(--release); fi
    install/iso/build --arch {{arch}} --payload "$p" "${flags[@]}"
    ls -lh iso/kubaos-{{codename}}-{{ver}}-{{arch}}*.iso

# All four ISOs (aarch64 and x86_64, release and vm), after the tests and payload checks. Needs a clean tree.
release: test
    just payload aarch64
    just payload x86_64
    just iso aarch64 vm
    just iso aarch64
    just iso x86_64 vm
    just iso x86_64
    ls -lh iso/kubaos-{{codename}}-{{ver}}-*.iso

# What a pull request's CI runs (.github/workflows/check.yml), in `nix develop`: the formatting, both clippy passes, the workspace's tests, the shell's logic tests, the screenshot harness's tests, the design checks, the system files' check and shellcheck. CI runs the mockups' part of the design checks only when their inputs change, and adds `apps-verify` for the apps a pull request changes.
check: fmt-check lint nextest qml-test shots-test design-check system-files-check shellcheck

# A changed file under system/files/ comes with a new system migration that installs it again, and system/files.map says what installs every file there (test/system-files/check, docs/agents/code-quality.md, "Shell"). Compares against the merge base with `base`; runs its own tests first.
system-files-check base="origin/main":
    test/system-files/check-test .
    test/system-files/check --base {{base}} .

# The shell's logic tests (test/qml) in Qt's test runner, headless, as `nix flake check` runs them (crates/checks.nix, qml-logic). A message from the QML engine fails them, as it fails the e2e checks. The flake sees the files git knows about: `git add` a new test first.
qml-test:
    nix build --no-link -L .#checks.{{host}}-linux.qml-logic

# The workspace's tests, with nextest; arguments go to `cargo nextest run`, as `just nextest -E 'package(kubaos-kobo)'`. With a package named (`-p kubaos-core`, `--package=…`), only that package builds and runs; with none, the whole workspace. Every test process gets GIT_DIR pointing at a decoy repository, as `git rebase --exec` sets it, and the run fails if the decoy changed (test/git-decoy): no test, and no git KubaOS runs, may act on a repository its environment names.
[positional-arguments]
nextest *args:
    #!/usr/bin/env bash
    set -euo pipefail
    scope=(--workspace)
    for arg in "$@"; do
      case "$arg" in
        -p | -p* | --package | --package=*) scope=() ;;
      esac
    done
    test/git-decoy cargo nextest run ${scope[@]+"${scope[@]}"} "$@"

# Hold the Apps catalog's pins to their upstreams (crates/kubaos-apps-verify, docs/TESTING.md): each vendor key's fingerprint and the InRelease it signs, each package's candidate in its repository or Ubuntu's archive, each pinned download's sha256 as served and as published, each nixpkgs package at flake.lock for both systems, each web app's URL. Every app with no ID; `just apps-verify --changed origin/main` the apps a branch changed. Network only: it installs nothing and reads no apt state.
[positional-arguments]
apps-verify *args:
    cargo run -q -p kubaos-apps-verify -- "$@"

# shellcheck over the repo's shell scripts (test/shellcheck/check): the Apps catalog's, the system scripts, migrations, scripts/, the test harness, the hooks. A warning or an error fails it; info and style notes are listed.
shellcheck:
    test/shellcheck/check .

# Start this worktree's cargo build from the main checkout's target/: a reflink copy, so the first build rebuilds only what differs from main (test/warm-target, docs/TESTING.md). Claude Code's agent worktrees get it from the WorktreeCreate hook; this is for a worktree made another way. Where reflinks aren't supported, it changes nothing.
warm:
    test/warm-target

# Format the workspace with the nightly rustfmt and rustfmt.toml (`nix develop` has it; outside, point RUSTFMT at a nightly rustfmt), then turn the `use foo::{self};` it writes into `use foo;` (crates/self-imports) and sort again.
fmt: _nightly-rustfmt
    cargo fmt --all
    crates/self-imports crates
    cargo fmt --all

# Check the formatting, rewriting nothing.
fmt-check: _nightly-rustfmt
    cargo fmt --all --check
    crates/self-imports --check crates

# A stable rustfmt skips rustfmt.toml's unstable options with a warning and formats the rest with the defaults, so the fmt recipes refuse one that warns.
_nightly-rustfmt:
    #!/usr/bin/env bash
    set -euo pipefail
    rustfmt="${RUSTFMT:-rustfmt}"
    if ! warnings="$(printf 'fn main() {}\n' | "$rustfmt" --edition 2024 --config-path rustfmt.toml 2>&1 >/dev/null)" || [[ -n $warnings ]]; then
      printf '%s\n' "$warnings" >&2
      echo "$rustfmt isn't a nightly rustfmt reading rustfmt.toml whole: run this in \`nix develop\`, or set RUSTFMT to a nightly rustfmt" >&2
      exit 1
    fi

# Clippy, both passes: the code with -D warnings, then every target (tests, examples, benches) with unwrap allowed, since tests may unwrap. The lints are `[workspace.lints]` in Cargo.toml. It runs `cargo-clippy` from PATH rather than `cargo clippy`, which takes the one in ~/.cargo/bin first: rustup's, on a machine or a runner with rustup.
lint: _pinned-clippy
    cargo-clippy clippy --workspace --locked -- -D warnings
    cargo-clippy clippy --workspace --locked --all-targets -- -D warnings -A clippy::unwrap_used

# Each clippy release adds and changes lints, so the recipes refuse one at another Rust than rust-toolchain.toml's: what passes here passes in CI.
_pinned-clippy:
    #!/usr/bin/env bash
    set -euo pipefail
    pinned="$(sed -n 's/^channel = "\([0-9]*\.[0-9]*\)\.[0-9]*"$/\1/p' rust-toolchain.toml)"
    # Clippy numbers its releases 0.1.N for Rust 1.N.
    clippy="$(cargo-clippy clippy --version)"
    minor="${clippy#clippy 0.1.}"
    if [[ ${minor%% *} != "${pinned#1.}" ]]; then
      echo "$(command -v cargo-clippy) is $clippy, but rust-toolchain.toml names Rust $pinned: run this in \`nix develop\`, or set its \`channel\` to the dev shell's \`rustc --version\`" >&2
      exit 1
    fi

# Cargo tests, the shell harness (Docker) and the engine end-to-end test.
test:
    cargo test --workspace
    just e2e
    install/test/engine-e2e

# The end-to-end checks in the Docker harness (test/run, docs/TESTING.md): every check, or `just e2e pins suites` for those. Builds the image and warms the store first; both are cached, so a run after a config change goes straight to the switch.
[positional-arguments]
e2e *checks:
    #!/usr/bin/env bash
    set -euo pipefail
    test/run build
    test/run warm
    only=()
    (($#)) && only=(--only "$(IFS=,; echo "$*")")
    test/run e2e ${only[@]+"${only[@]}"}

# The design checks (docs/DESIGN.md, "Checks"): the libraries' (`design-check-ui`), then the mockup pages that must pass (`mockups-checked`).
design-check: design-check-ui mockups-checked

# The libraries' design checks: the lint for literal design values against design/lint-baseline.toml, every colour pair's WCAG contrast in both modes, every specimen page at every scale, light and dark, held to the geometry rules, then QML's parity with iced. `just test` runs the first three as cargo tests; `nix flake check` runs them and parity.
design-check-ui: && design-parity
    cargo run -q -p kubaos-design -- lint
    cargo run -q -p kubaos-design -- contrast >/dev/null
    cargo run -q -p kubaos-specimen -- check

# The components page drawn with KubaOS.Components, laid out headless by Qt (qml-specimen, from the flake) and held to the iced page's boxes, node for node, at every scale, light and dark. The flake sees the files git knows about: `git add` a new QML file first, or it is left out of the page.
design-parity:
    #!/usr/bin/env bash
    set -euo pipefail
    frames="$(mktemp -d)"
    trap 'rm -rf "$frames"' EXIT
    nix run .#checks.{{host}}-linux.qml-specimen -- frames "$frames" >/dev/null
    cargo run -q -p kubaos-specimen -- parity "$frames"

# The mockup site (design/mockups, docs/agents/building-ui.md), served with hot reload at the address Vite prints: every page, the toolbar and the navigation. A change to design/tokens.toml regenerates its CSS live. Arguments go to Vite, as `just mockups --port 5174`.
[positional-arguments]
mockups *args: _mockups-deps
    cd design/mockups && npx vite "$@"

# The mockup site built static into design/mockups/dist, after the type check and the check of its registered custom properties (`scripts/property-check.ts`): open it from any static server.
mockups-build: _mockups-deps
    cd design/mockups && npm run -s build

# The mockup site built into design/mockups/dist for the checks, as `mockups-build` builds it but without the moves' recipes, which it draws every page for.
_mockups-site: _mockups-deps
    cd design/mockups && npm run -s typecheck && node scripts/property-check.ts && npx vite build --logLevel warn

# The mockups' check (docs/DESIGN.md, "The mockups"): the walk that reads a page's boxes and the pass that centres them on device pixels held to small pages first (`scripts/frame-test.ts`, `scripts/centre-test.ts`), then each page laid out in headless Chromium at every scale, light and dark, held to the geometry rules, and each mocked component to its twin on the specimen's pages (`kubaos-specimen mockup`), wherever each page puts it; then every motion player seeked and held to its recipe (`scripts/player-test.ts`). With PAGEs (slugs, as `#/atoms/button`), those pages alone, together: the `identical` rule compares their instances with each other's only. With none, the whole site, which not every page passes yet: the site is drawn ahead of the libraries.
[positional-arguments]
mockups-check *pages: _mockups-site
    #!/usr/bin/env bash
    set -euo pipefail
    frames="$(mktemp -d)"
    trap 'rm -rf "$frames"' EXIT
    cd design/mockups
    node --disable-warning=ExperimentalWarning scripts/frame-test.ts
    node --disable-warning=ExperimentalWarning scripts/centre-test.ts
    node scripts/render.ts "$frames" --frames "$@"
    cargo run -q -p kubaos-specimen -- mockup "$frames"
    node scripts/player-test.ts "$@"

# The mockup pages the mockups' check holds every change to their inputs to, by `design-check` (CI's `mockups` job, when a pull request changes them): those listed in design/mockups/checked-pages, checked together. A page joins the list once `just mockups-check PAGE` passes it with no finding and no difference.
mockups-checked:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t pages < <(grep -v -e '^#' -e '^$' design/mockups/checked-pages)
    if ((${#pages[@]} == 0)); then
      echo "mockups-checked: design/mockups/checked-pages names no page" >&2
      exit 1
    fi
    just mockups-check "${pages[@]}"

# One mockup page's states, each drawn alone (solo mode, `?solo=STATE`), captured as PNGs into ~/.cache/kubaos-mockups/shots/ by headless Chromium (design/mockups/scripts/shot.ts): every state with none named, or `just mockup-shot setup-yours go rest --mode light --scale 1.5 --keys Tab,Enter --frames 0,120,240 --at 1920x1080 --sheet --out DIR`. It captures from your own dev server with `--port N` or PORT=N, or starts Vite on a free port from 5188 up for the run; never 5174, the review server's. In `nix develop`.
[positional-arguments]
mockup-shot page *args: _mockups-deps
    cd design/mockups && node scripts/shot.ts "$@"

# A checkout's mockup site served with hot reload on PORT as a transient user unit, so the site follows an agent's edits and commits there live: `just mockup-preview ~/.t3/worktrees/kubaos/NAME`, this checkout by default. The unit is kubaos-mockups-preview on 5175 (kubaos-mockups-preview-PORT on another port), and a preview already running on that port is replaced. It prints the address once the site answers. Never 5174, the review server's.
mockup-preview worktree=justfile_directory() port="5175":
    #!/usr/bin/env bash
    set -euo pipefail
    port={{port}}
    if [[ $port == 5174 ]]; then
      echo "mockup-preview: 5174 is the review server's" >&2
      exit 2
    fi
    site="$(realpath "{{worktree}}")"
    [[ -f $site/vite.config.ts ]] || site="$site/design/mockups"
    if [[ ! -f $site/vite.config.ts ]]; then
      echo "mockup-preview: no mockup site in {{worktree}}" >&2
      exit 2
    fi
    unit=kubaos-mockups-preview
    [[ $port == 5175 ]] || unit="kubaos-mockups-preview-$port"
    (cd "$site" && { [[ node_modules/.package-lock.json -nt package-lock.json ]] || npm ci --no-audit --no-fund; })
    systemctl --user stop "$unit" 2>/dev/null || true
    systemctl --user reset-failed "$unit" 2>/dev/null || true
    systemd-run --user --quiet --unit="$unit" --collect --working-directory="$site" --setenv=PATH="$PATH" \
      npx vite --port "$port" --strictPort
    # The first start generates the tokens' CSS, which builds kubaos-tokens in that checkout.
    for _ in $(seq 300); do
      if curl -sf -o /dev/null "http://127.0.0.1:$port/"; then
        echo "http://localhost:$port/ serves $site ($unit)"
        exit 0
      fi
      if ! systemctl --user is-active --quiet "$unit"; then break; fi
      sleep 1
    done
    echo "mockup-preview: $unit didn't start:" >&2
    journalctl --user -u "$unit" -n 20 --no-pager >&2 || true
    exit 1

# Stops the preview `mockup-preview` started on PORT.
mockup-preview-stop port="5175":
    #!/usr/bin/env bash
    set -euo pipefail
    unit=kubaos-mockups-preview
    [[ {{port}} == 5175 ]] || unit="kubaos-mockups-preview-{{port}}"
    systemctl --user stop "$unit"

# The mockup site's npm dependencies, from its package-lock.json, when node_modules is missing or older than the lock.
_mockups-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    cd design/mockups
    if [[ ! node_modules/.package-lock.json -nt package-lock.json ]]; then
      npm ci --no-audit --no-fund
    fi

# Write the token outputs a checkout keeps (docs/DESIGN.md, "Where the outputs are generated"): the checked-in boot files after a change to design/tokens.toml, and the ones git ignores (Tokens.qml, the terminals' tokens, Hyprland's tokens.lua), which the Nix build writes for the home, for an editor or a test reading them from the checkout. The crates' build scripts write their own.
tokens:
    cargo run -q -p kubaos-tokens -- write

# Render the logo and every picture made from it (branding/README.md, "Regenerating"): the vector marks, the neon sign's assets and boot layers, the boot screens' pictures, then the token outputs that measure them. In the flake's `branding` shell.
branding:
    nix develop .#branding -c branding/render.sh
    cargo run -q -p kubaos-tokens -- write

# After fixing literal design values: lower design/lint-baseline.toml to what the lint finds. It never raises a count.
design-baseline:
    cargo run -q -p kubaos-design -- baseline

# Boot the newest ISO for <arch> in QEMU (test/vm/run boot-iso). Add --gui for a window.
vm arch=host *flags="":
    test/vm/run boot-iso --arch {{arch}} {{flags}}

# Headless install of the bench ISO onto a fresh disk, then boot the installed system and check it (user kuba, password bench, LUKS passphrase bench-luks). `just vm-install aarch64 test/vm/examples/install-alice.toml` installs as alice, whose generation the engine builds.
vm-install arch=host config="":
    #!/usr/bin/env bash
    set -euo pipefail
    iso="$(ls -t iso/kubaos-*-{{arch}}-vm.iso 2>/dev/null | head -1)"
    [[ -n $iso ]] || { echo "no bench ISO for {{arch}}: just iso {{arch}} vm" >&2; exit 1; }
    cfg=test/vm/examples/install.toml
    sb=()
    if [[ {{arch}} == x86_64 ]]; then cfg=test/vm/examples/install-x86_64.toml; sb=(--secure-boot); fi
    [[ -z "{{config}}" ]] || cfg="{{config}}"
    test/vm/run install "$cfg" test/vm/examples/secrets.toml --iso "$iso" --arch {{arch}} ${sb[@]+"${sb[@]}"} --ssh --timeout 3000
    test/vm/run boot-disk
    sleep 45
    test/vm/run type 'bench-luks\n'
    test/vm/run wait-ssh --timeout 600
    test/vm/run ssh 'id -un; systemctl --failed --no-legend; echo "layout: $(grep XKBLAYOUT /etc/default/keyboard)"'
    test/vm/run shot desktop

# After `just vm-install`: the user-layer promise (issue #9) on the installed system. Fills every file in ~/.config/kubaos, commits newer defaults to a clone of /usr/share/kubaos, switches the user flake onto it, and checks the user's files are untouched and the new defaults live.
vm-user-layer user="kuba":
    test/vm/run ssh "runuser -l {{user}} -c '. /etc/default/locale; export LANG XDG_RUNTIME_DIR=/run/user/\$(id -u) DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/\$(id -u)/bus; /usr/share/kubaos/test/user-layer-check'"

# Before/after screenshots for a design PR (test/shots/shots, docs/agents/design-pr-loop.md): the surfaces and specimen pages whose sources changed against origin/main at 1× and 1.5× dark, `just shots --all` every one, or `just shots SURFACE… [--scales 1,1.5] [--modes dark,light] [--ref REV] [--full]`; `just shots kobo`, `just shots specimen`, `just shots specimen/components`; `just shots publish BRANCH` prints the PR's table. Writes test/out/shots/, and keeps the before-trees of the last four base commits in ~/.cache/kubaos/shots.
[positional-arguments]
shots *args:
    nix run .#shots -- "$@"

# The screenshot harness's tests (test/shots/shots-test): which surfaces a change renders, the table of their sources, and the pruning of the before-trees, on a throwaway repository and cache.
shots-test:
    test/shots/shots-test .

# Screenshot session in Docker (test/run shots): `just shots-session`, then `test/run shots shot NAME`.
shots-session *flags="--hyprland":
    test/run shots {{flags}}

# New migration: migrations/<system|user>/<unix time>-<slug>.sh, which kubaos update runs once per machine or user (docs/CONTRACTS.md, "Migrations").
migration kind slug:
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{kind}}" in
      system) who="as root, after apt and before the home-manager switch" ;;
      user) who="as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake" ;;
      *) echo "just migration: the kind is system or user, not {{kind}}" >&2; exit 2 ;;
    esac
    [[ "{{slug}}" =~ ^[a-z0-9][a-z0-9-]*$ ]] || { echo "just migration: the slug is a-z, 0-9 and -, not {{slug}}" >&2; exit 2; }
    f="migrations/{{kind}}/$(date +%s)-{{slug}}.sh"
    printf '%s\n' \
      "#!/usr/bin/env bash" \
      "# {{slug}}: what this changes, and why the release needs it." \
      "# kubaos update runs it once, $who," \
      "# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting" \
      "# non-zero stops the update; it runs again on the next one, so make it safe to rerun." \
      '# shellcheck source=migrations/lib.sh' \
      'source "$KUBAOS_DIR/migrations/lib.sh"' >"$f"
    echo "$f"

# Remove iso/ and the build outputs. Never touches Docker volumes or images.
clean:
    rm -rf iso install/out install/target test/out
    test/vm/run clean
