<p align="center">
  <img src="branding/kubaos-horizontal.svg" alt="クバ KubaOS" width="420">
</p>

# KubaOS

**Shinjuku**, work in progress. Nothing is released yet.

Kuba's personal OS: Ubuntu 26.04 underneath, Nix and home-manager for everything you touch, Hyprland and a Quickshell desktop on top. It's built for one laptop, a TUXEDO InfinityBook Pro 14 Gen10, and for one person. It updates when you say so, never nags, and if you break it there's a snapshot to go back to.

It exists because macOS got worse with every update. Nothing stops you from reading the code and stealing the parts you like. Nobody's promising it'll work on your machine.

_Screenshots are being retaken._

## Install it

The install is offline: the whole system is on the USB stick. Write the ISO with `dd` or Etcher, boot it (UEFI, Secure Boot works), click through language, keyboard, disk and account, and hold the button to erase the disk. It's LUKS2-encrypted, with a `factory` snapshot to roll back to. Git name and email aren't in the repo: Kōbō's Git screen or the Git card in first-boot setup asks for them and writes `~/.config/git-identity/identity.gitconfig`.

Flashing, checks and what to expect: [`docs/RELEASE-shinjuku.md`](docs/RELEASE-shinjuku.md). The details: [`docs/INSTALL.md`](docs/INSTALL.md) and [`docs/INSTALL-INTERNALS.md`](docs/INSTALL-INTERNALS.md).

## Build it yourself

You need Docker and [`just`](https://github.com/casey/just). On macOS you also need QEMU (`brew install qemu`) to boot the result. `arch` is `aarch64` or `x86_64` and defaults to the machine you're on. Amd64 builds are emulated on an Apple Silicon Mac, so they're slow.

```sh
just                       # list everything
just payload x86_64        # the system image the install lays down
just iso x86_64            # the USB-stick ISO into iso/
just iso aarch64 vm        # the bench ISO (ssh, no firmware) for QEMU
just vm-install aarch64    # install it headless in QEMU, boot it, look around
just vm aarch64            # boot an ISO in QEMU
just test                  # cargo tests plus the Docker and engine checks
just release               # tests, checks and all four ISOs
just clean                 # remove iso/ and the build outputs
```

One payload build at a time, they eat the RAM. How it all fits together: [`install/image/README.md`](install/image/README.md), [`install/iso/README.md`](install/iso/README.md) and [`test/vm/README.md`](test/vm/README.md).

## More

- [`docs/USING.md`](docs/USING.md): keys, updates and rollbacks, daily commands, the repo map, known limitations.
- [`docs/BRAND.md`](docs/BRAND.md): name, look and voice.
- [`docs/RELEASE-shinjuku.md`](docs/RELEASE-shinjuku.md): the Shinjuku release.
- [`docs/TESTING.md`](docs/TESTING.md): the test harnesses.

## Licence

MIT OR Apache-2.0, except the wallpapers and fonts. See [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE) and the wallpapers' credits in [SuplenC/kubaos-wallpapers](https://github.com/SuplenC/kubaos-wallpapers).
