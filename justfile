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

# Build the installer payload (the whole system as a squashfs) into installer/out/payload-<arch>. One at a time.
payload arch=host:
    installer/image/build --arch {{arch}} {{dirty}}

# Check a built payload (unpack + chroot checks).
check arch=host:
    installer/image/check --arch {{arch}}

# Build the live ISO into iso/ (builds the payload first if there isn't one). `just iso aarch64 vm` makes the bench ISO (ssh, no firmware).
iso arch=host profile="release":
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{profile}}" in release|vm) ;; *) echo "profile is 'vm' or empty (release)" >&2; exit 2 ;; esac
    p=installer/out/payload-{{arch}}
    [[ -f $p/manifest.json ]] || just payload {{arch}}
    if [[ "{{profile}}" == vm ]]; then flags=(--profile vm); else flags=(--release); fi
    installer/iso/build --arch {{arch}} --payload "$p" "${flags[@]}"
    ls -lh iso/kubaos-{{codename}}-{{ver}}-{{arch}}*.iso

# All four ISOs (aarch64 and x86_64, release and vm), after the tests and payload checks. Needs a clean tree.
release: test
    just payload aarch64
    just check aarch64
    just payload x86_64
    just check x86_64
    just iso aarch64 vm
    just iso aarch64
    just iso x86_64 vm
    just iso x86_64
    ls -lh iso/kubaos-{{codename}}-{{ver}}-*.iso

# Cargo tests, the shell harness (Docker) and the engine end-to-end test.
test:
    cargo test --manifest-path installer/Cargo.toml
    test/run build
    test/run e2e
    installer/test/engine-e2e

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

# Screenshot session in Docker (test/run shots): `just shots`, then `test/run shots shot NAME`.
shots *flags="--hyprland":
    test/run shots {{flags}}

# Remove iso/ and the build outputs. Never touches Docker volumes or images.
clean:
    rm -rf iso installer/out installer/target test/out
    test/vm/run clean
