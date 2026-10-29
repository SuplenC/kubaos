# syntax=docker/dockerfile:1
# The payload's system half: a bootable Ubuntu 26.04 put together by the
# repo's own system/*.sh, in image mode (DOTFILES_IMAGE_BUILD=1, system/lib.sh).
# install/image/build builds this, then lib/assemble.sh adds /nix (the
# home-manager generation) in a container and packs the tree.
# install/image/README.md explains the stages.
#
# Only system/ is in the build context (Dockerfile.dockerignore), and each
# stage copies just the files it runs, so editing home/, config/ or scripts/
# never rebuilds any of this.

ARG UBUNTU=ubuntu:26.04

# ---------------------------------------------------------------------------
# os: what the Ubuntu Server (minimized) installer would have put on disk
# before system/ runs, plus the boot chain the engine needs in the chroot.
FROM ${UBUNTU} AS os
ARG TARGETARCH
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive LANG=C.UTF-8 TZ=Etc/UTC

# Keep downloaded .debs in the cache mount. /var/lib/apt/lists is a cache
# mount too, so the image never carries apt lists; /var/lib/apt itself
# (extended_states: what's auto-installed) stays in the image.
RUN rm -f /etc/apt/apt.conf.d/docker-clean \
 && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' >/etc/apt/apt.conf.d/10kubaos-keep-cache

# The kernel flavour is linux-generic on both arches: the TUXEDO needs nothing
# else (tuxedo-drivers is DKMS, first boot), and a QEMU arm64 VM boots it.
# grub-efi-*-signed + shim-signed: Secure Boot works on the laptop.
RUN --mount=type=cache,id=kubaos-image-apt-cache-${TARGETARCH},target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=kubaos-image-apt-lists-${TARGETARCH},target=/var/lib/apt/lists,sharing=locked \
    case "$TARGETARCH" in \
      amd64) grub=(grub-efi-amd64 grub-efi-amd64-signed) ;; \
      arm64) grub=(grub-efi-arm64 grub-efi-arm64-signed) ;; \
      *) echo "unsupported arch $TARGETARCH" >&2; exit 1 ;; \
    esac \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      ubuntu-minimal systemd systemd-sysv systemd-resolved systemd-timesyncd udev dbus \
      libpam-systemd sudo-rs locales tzdata keyboard-configuration console-setup kbd \
      linux-generic initramfs-tools zstd cryptsetup cryptsetup-initramfs \
      btrfs-progs dosfstools e2fsprogs "${grub[@]}" shim-signed efibootmgr os-prober \
      openssh-server libcap2-bin ca-certificates less bash-completion \
 && ls /boot/vmlinuz-* /boot/initrd.img-*

# The scripts run as the desktop user, with sudo. kuba (uid 1000) exists only
# while building; lib/assemble.sh deletes it from the payload, the engine
# creates it again. The image ships user ubuntu with uid 1000: drop it.
RUN userdel -r ubuntu 2>/dev/null || true \
 && useradd -m -u 1000 -U -G sudo -s /bin/bash kuba \
 && echo 'kuba ALL=(ALL:ALL) NOPASSWD: ALL' >/etc/sudoers.d/kubaos-build \
 && chmod 0440 /etc/sudoers.d/kubaos-build

ENV DOTFILES_IMAGE_BUILD=1
WORKDIR /opt/dotfiles

# ---------------------------------------------------------------------------
# base: system/10-strip.sh and system/20-base.sh for real.
FROM os AS base
ARG TARGETARCH
COPY system/lib.sh system/manifest.sh system/packages.toml system/10-strip.sh system/20-base.sh /opt/dotfiles/system/
COPY system/files/apt/ /opt/dotfiles/system/files/apt/
COPY system/files/netplan/ /opt/dotfiles/system/files/netplan/
COPY system/files/logind.conf.d/ /opt/dotfiles/system/files/logind.conf.d/
COPY system/files/sysctl.d/ /opt/dotfiles/system/files/sysctl.d/
COPY system/files/systemd/system.conf.d/ /opt/dotfiles/system/files/systemd/system.conf.d/
COPY system/files/systemd/user.conf.d/ /opt/dotfiles/system/files/systemd/user.conf.d/
RUN --mount=type=cache,id=kubaos-image-apt-cache-${TARGETARCH},target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=kubaos-image-apt-lists-${TARGETARCH},target=/var/lib/apt/lists,sharing=locked \
    apt-get update \
 && runuser -u kuba -- /opt/dotfiles/system/10-strip.sh \
 && runuser -u kuba -- /opt/dotfiles/system/20-base.sh

# ---------------------------------------------------------------------------
# nix: the official installer exactly as system/60-nix.sh runs it (version,
# URL and sha256 read from that script), multi-user, flakes on, and
# KubaOS's binary cache, the lines nix_binary_cache (system/lib.sh) writes, so
# the payload's build downloads KubaOS's packages too. The half of 60-nix.sh
# that needs the flake runs in lib/assemble.sh.
FROM base AS nix
ARG TARGETARCH
COPY system/60-nix.sh /opt/dotfiles/system/60-nix.sh
RUN eval "$(grep -E '^NIX_(VERSION|INSTALL_URL|INSTALL_SHA256)=' /opt/dotfiles/system/60-nix.sh)" \
 && tmp="$(mktemp -d)" && chmod 755 "$tmp" \
 && curl --proto '=https' --tlsv1.2 -fsSL "$NIX_INSTALL_URL" -o "$tmp/install" \
 && sha256sum --quiet -c <<<"$NIX_INSTALL_SHA256  $tmp/install" \
 && runuser -u kuba -- sh "$tmp/install" --daemon --yes </dev/null \
 && rm -rf "$tmp" \
 && printf '\n# dotfiles/system/60-nix.sh\nexperimental-features = nix-command flakes\n' >>/etc/nix/nix.conf \
 && printf '\n# KubaOS: its binary cache (nix_binary_cache, system/lib.sh)\nextra-substituters = %s\nextra-trusted-public-keys = %s\n' \
      https://kuba-os.cachix.org kuba-os.cachix.org-1:BAhDPs8FW+W0SaOzsx7yS9sZmcb6ZjdLc09hBeKXIE0= >>/etc/nix/nix.conf \
 && /nix/var/nix/profiles/default/bin/nix --version

# ---------------------------------------------------------------------------
# apps: system/40-apps.sh (Docker CE, Chrome on amd64, the Chrome theme helper).
FROM nix AS apps
ARG TARGETARCH
COPY system/40-apps.sh /opt/dotfiles/system/
COPY system/files/chrome/ /opt/dotfiles/system/files/chrome/
COPY system/files/systemd/docker.service.d/ /opt/dotfiles/system/files/systemd/docker.service.d/
RUN --mount=type=cache,id=kubaos-image-apt-cache-${TARGETARCH},target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=kubaos-image-apt-lists-${TARGETARCH},target=/var/lib/apt/lists,sharing=locked \
    apt-get update \
 && runuser -u kuba -- /opt/dotfiles/system/40-apps.sh

# ---------------------------------------------------------------------------
# desktop: system/50-hyprland.sh (PPA on amd64, Ubuntu's 0.53 on arm64,
# autologin, lock-screen PAM, keyd for the accents), 70-bitwarden.sh
# (desktop .deb on amd64, bw, pam_u2f lines; the Nix half runs in
# lib/assemble.sh), 75-yubikey.sh.
FROM apps AS desktop
ARG TARGETARCH
COPY system/50-hyprland.sh system/70-bitwarden.sh system/75-yubikey.sh /opt/dotfiles/system/
COPY system/files/getty/ /opt/dotfiles/system/files/getty/
COPY system/files/pam.d/ /opt/dotfiles/system/files/pam.d/
COPY system/files/tmpfiles.d/ /opt/dotfiles/system/files/tmpfiles.d/
COPY system/files/systemd/ /opt/dotfiles/system/files/systemd/
COPY system/files/udev/ /opt/dotfiles/system/files/udev/
COPY system/files/keyd/ /opt/dotfiles/system/files/keyd/
COPY system/files/xkb/ /opt/dotfiles/system/files/xkb/
COPY system/files/polkit/ /opt/dotfiles/system/files/polkit/
COPY system/files/hyprland-autostart.zsh /opt/dotfiles/system/files/
RUN --mount=type=cache,id=kubaos-image-apt-cache-${TARGETARCH},target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=kubaos-image-apt-lists-${TARGETARCH},target=/var/lib/apt/lists,sharing=locked \
    apt-get update \
 && runuser -u kuba -- /opt/dotfiles/system/50-hyprland.sh \
 && runuser -u kuba -- /opt/dotfiles/system/70-bitwarden.sh \
 && runuser -u kuba -- /opt/dotfiles/system/75-yubikey.sh

# ---------------------------------------------------------------------------
# boot: system/25-boot.sh (Plymouth splash, GRUB theme, menu names and
# timeout). Its own stage, last, so changing the splash rebuilds nothing else.
# The initramfs and grub.cfg it would rebuild are made by the engine on the
# target (image mode skips them).
FROM desktop AS boot
ARG TARGETARCH
COPY system/25-boot.sh /opt/dotfiles/system/
COPY system/files/plymouth/ /opt/dotfiles/system/files/plymouth/
COPY system/files/grub/ /opt/dotfiles/system/files/grub/
COPY system/files/fonts/ /opt/dotfiles/system/files/fonts/
COPY system/files/initramfs-tools/ /opt/dotfiles/system/files/initramfs-tools/
RUN --mount=type=cache,id=kubaos-image-apt-cache-${TARGETARCH},target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=kubaos-image-apt-lists-${TARGETARCH},target=/var/lib/apt/lists,sharing=locked \
    apt-get update \
 && runuser -u kuba -- /opt/dotfiles/system/25-boot.sh

# ---------------------------------------------------------------------------
# system: what lib/assemble.sh starts from. The scripts in /opt/dotfiles are
# build-time copies; assemble.sh runs the ones it needs from the repo snapshot
# and leaves /opt/dotfiles out of the payload.
FROM boot AS system
CMD ["sleep", "infinity"]

# ---------------------------------------------------------------------------
# tools: what install/image/check runs in (unsquashfs, getcap, jq). Not part
# of the payload.
FROM ${UBUNTU} AS tools
ARG TARGETARCH
RUN --mount=type=cache,id=kubaos-image-apt-cache-${TARGETARCH},target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=kubaos-image-apt-lists-${TARGETARCH},target=/var/lib/apt/lists,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean \
 && apt-get update \
 && apt-get install -y --no-install-recommends squashfs-tools libcap2-bin jq file
