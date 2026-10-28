# 3. KubaOS is developed and tested on its own Linux machine

Status: accepted (2026-10-28). Supersedes the parts of ADR 0001 that put the binary cache's push and the release builds on the Mac.

## Context

ADR 0001 assumed a Mac as the development machine: the binary cache "signed with our own key, pushed from the Mac", and "building and signing stay on the Mac until CI exists". The test harness was built around the same Mac: Docker Desktop's arm64 VM, an aarch64 `kuba@vm` home, and a QEMU bench on HVF. x86_64 builds and boots there were emulated, and some checks never ran there at all. Since then the cache moved to CI (#10, `docs/RESEARCH-updates.md`, "What shipped instead"), and development moved to an x86_64 Linux machine running KubaOS's own stack: Ubuntu 26.04, Nix and Hyprland.

## Decision

**The development and test machine is the x86_64 Linux machine.** No Mac is part of building, testing or releasing KubaOS.

**The binary cache** is the public Cachix cache `kuba-os`, signed by Cachix, not by a key of our own. `.github/workflows/cache.yml` builds `packages.<system>.cache` on every push to `main`, on x86_64 and aarch64 runners, and pushes it with `cachix push kuba-os`. Nothing is pushed from a developer's machine. Machines read it through `extra-substituters` and `extra-trusted-public-keys` in `/etc/nix/nix.conf`, as ADR 0001 decided, and users are still not added to `trusted-users`.

**The test harness runs natively there.** `test/run` (`just e2e`) builds an x86_64 image, switches the x86_64 `kuba@vm` home, and reuses the machine's own Nix store read-only as a substituter, ahead of the binary caches. `test/vm/run` runs the bench on KVM. Release builds (`just release`) and the release tag's signature are made on the same machine.

**aarch64 stays a product target.** KubaOS still ships an aarch64 ISO and payload, and CI still builds `packages.aarch64-linux`. On the x86_64 machine they build and boot emulated.

## Consequences

- Every e2e check runs on every machine that develops KubaOS, including the ones that needed hardware the Mac's VM lacked; "needs a run on the Mac" is no longer a state a PR can be in.
- Signing a release needs the SSH signing key on the Linux machine, through Bitwarden's agent there.
- An aarch64 build or boot is slow (QEMU binfmt and TCG), so the aarch64 artefacts get checked less often than the x86_64 ones.
