# KubaOS build and test commands. `just` lists them.
# Needs Docker, and on macOS QEMU (brew install qemu) for the vm recipes.
# arch defaults to this machine's: aarch64 or x86_64.

host := `case "$(uname -m)" in arm64|aarch64) echo aarch64 ;; *) echo x86_64 ;; esac`
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
check arch=host:
    install/image/check --arch {{arch}}

# Build the live ISO into iso/ (builds the payload first if there isn't one, and checks it if it hasn't passed). `just iso aarch64 vm` makes the bench ISO (ssh, no firmware).
iso arch=host profile="release":
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{profile}}" in release|vm) ;; *) echo "profile is 'vm' or empty (release)" >&2; exit 2 ;; esac
    p=install/out/payload-{{arch}}
    [[ -f $p/manifest.json ]] || just payload {{arch}}
    [[ "$(cat "$p/check.ok" 2>/dev/null)" == "$(shasum -a 256 "$p/manifest.json" | cut -d' ' -f1)" ]] || just check {{arch}}
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

# Cargo tests, the shell harness (Docker) and the engine end-to-end test.
test:
    cargo test --workspace
    test/run build
    test/run e2e
    install/test/engine-e2e

# The design checks (docs/DESIGN.md, "Checks"): the lint for literal design values against design/lint-baseline.toml, every colour pair's WCAG contrast in both modes, every specimen page at every scale, light and dark, held to the geometry rules, then QML's parity with iced. `just test` runs the first three as cargo tests; `nix flake check` runs them all.
design-check: && design-parity
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

# Before/after screenshots for a design PR (test/shots/shots, docs/agents/design-pr-loop.md): the surfaces changed against origin/main at 1× and 1.5× dark, or `just shots SURFACE… [--scales 1,1.5] [--modes dark,light] [--ref REV] [--full]`; `just shots kobo`, `just shots specimen`; `just shots publish BRANCH` prints the PR's table. Writes test/out/shots/.
[positional-arguments]
shots *args:
    nix run .#shots -- "$@"

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
