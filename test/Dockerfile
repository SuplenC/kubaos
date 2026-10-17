# syntax=docker/dockerfile:1
# Test image for this repo: Ubuntu 26.04 put together by the repo's own
# system/*.sh, plus Nix and the test tools. docs/TESTING.md explains the
# layers; build it with `test/run build`.
#
# Only system/ files are copied in, one stage at a time, so editing config/,
# home/ or scripts/ never rebuilds the image. Those come in at run time: the
# repo is bind-mounted read-only and copied to ~/Projects/dotfiles, and
# home-manager switches against that copy (test/lib/harness.sh).
#
# /nix is a named volume at run time (dotfiles-nix). Docker seeds an empty
# volume from this image's /nix (the bare Nix install from the `nix` stage),
# so anything a later stage put into /nix would be hidden by the volume.
# Nix packages the tests need are realised into the volume by `test/run warm`.

ARG UBUNTU=ubuntu:26.04

# ---------------------------------------------------------------------------
# base: Ubuntu + system/10-strip.sh + system/20-base.sh, run for real as kuba.
FROM ${UBUNTU} AS base
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive LANG=C.UTF-8 TZ=Etc/UTC

# Keep downloaded .debs in the BuildKit cache mount (the image's docker-clean
# hook would delete them after every install).
RUN rm -f /etc/apt/apt.conf.d/docker-clean \
 && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' >/etc/apt/apt.conf.d/10keep-cache \
 && touch /.dockerenv

# The image ships user ubuntu with uid 1000; the laptop's user is kuba (1000).
# sudo-rs is 26.04's sudo. NOPASSWD because the scripts call sudo themselves;
# PAM is tested through a separate user (test/lib/e2e.sh).
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
 && apt-get install -y --no-install-recommends sudo-rs ca-certificates \
 && userdel -r ubuntu \
 && useradd -m -u 1000 -U -G sudo -s /bin/bash kuba \
 && echo 'kuba:kuba' | chpasswd \
 && echo 'kuba ALL=(ALL:ALL) NOPASSWD: ALL' >/etc/sudoers.d/kuba \
 && chmod 0440 /etc/sudoers.d/kuba

COPY system/lib.sh system/10-strip.sh system/20-base.sh /opt/dotfiles/system/
COPY system/files/apt/ /opt/dotfiles/system/files/apt/
COPY system/files/netplan/ /opt/dotfiles/system/files/netplan/
COPY system/files/logind.conf.d/ /opt/dotfiles/system/files/logind.conf.d/
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    runuser -u kuba -- /opt/dotfiles/system/10-strip.sh \
 && runuser -u kuba -- /opt/dotfiles/system/20-base.sh \
 && usermod -s /usr/bin/zsh kuba

# ---------------------------------------------------------------------------
# nix: the official installer exactly as system/60-nix.sh runs it (version,
# URL and sha256 read from that script), multi-user, flakes on. The flake-
# dependent half of 60-nix.sh (home-manager switch, PAM module links, GPU
# setup) runs at test time against the volume, by the real script.
FROM base AS nix
COPY system/60-nix.sh /opt/dotfiles/system/60-nix.sh
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends curl xz-utils
RUN eval "$(grep -E '^NIX_(VERSION|INSTALL_URL|INSTALL_SHA256)=' /opt/dotfiles/system/60-nix.sh)" \
 && tmp="$(mktemp -d)" && chmod 755 "$tmp" \
 && curl --proto '=https' --tlsv1.2 -fsSL "$NIX_INSTALL_URL" -o "$tmp/install" \
 && sha256sum --quiet -c <<<"$NIX_INSTALL_SHA256  $tmp/install" \
 && runuser -u kuba -- sh "$tmp/install" --daemon --yes </dev/null \
 && rm -rf "$tmp" \
 && printf '\n# dotfiles/system/60-nix.sh\nexperimental-features = nix-command flakes\n' >>/etc/nix/nix.conf \
 && /nix/var/nix/profiles/default/bin/nix --version

# ---------------------------------------------------------------------------
# tooling: test-only tools from apt. Nix-only ones (Chromium for arm64,
# Nix's pamtester for the lock screen's libpam) come from the flake's
# nixpkgs at warm time. gpu-screen-recorder: not in resolute's archive, and
# it needs a GPU anyway; wf-recorder records instead. nodejs/npm stand in
# for the Node that proto installs on the laptop (Mason's npm packages).
FROM nix AS tooling
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
 && apt-get install -y --no-install-recommends \
      sway swaybg grim slurp wf-recorder wayland-utils webp imagemagick \
      mesa-utils libgl1-mesa-dri libegl-mesa0 libegl1 libgles2 \
      dbus-daemon dbus-user-session dbus-x11 gnome-keyring libsecret-tools \
      xdg-utils xdg-user-dirs fonts-noto-core fonts-dejavu-core \
      pamtester shellcheck lua5.5 hyperfine jq expect procps psmisc less file \
      python3 python3-requests python3-cryptography openssl nodejs npm

# ---------------------------------------------------------------------------
# desktop: the system halves of system/50-hyprland.sh (Ubuntu's arm64
# hyprland 0.53, lock-screen PAM files, unix_chkpwd link, tty1 autologin) and
# system/70-bitwarden.sh (bw CLI, pam_u2f lines for sudo/sudo-i/polkit-1,
# qs-lock-u2f). The pam_u2f.so link into /nix is made at test time.
FROM tooling AS desktop
COPY system/50-hyprland.sh system/70-bitwarden.sh /opt/dotfiles/system/
COPY system/files/getty/ /opt/dotfiles/system/files/getty/
COPY system/files/pam.d/ /opt/dotfiles/system/files/pam.d/
COPY system/files/tmpfiles.d/ /opt/dotfiles/system/files/tmpfiles.d/
COPY system/files/systemd/ /opt/dotfiles/system/files/systemd/
COPY system/files/hyprland-autostart.zsh /opt/dotfiles/system/files/
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update \
 && runuser -u kuba -- /opt/dotfiles/system/50-hyprland.sh \
 && runuser -u kuba -- /opt/dotfiles/system/70-bitwarden.sh

# ---------------------------------------------------------------------------
# hyprland: Hyprland 0.56.2 for aarch64, headless on llvmpipe.
#
# TODO(lead): recipe from screenshot agent.
# Contract for test/lib/shots.sh (`test/run shots --hyprland`): this stage
# installs an executable /usr/local/lib/dotfiles-test/hyprland-session that,
# run as kuba with XDG_RUNTIME_DIR set, starts Hyprland with the repo's
# config/hypr (live-linked to ~/.config/hypr) on a headless output, exports
# WAYLAND_DISPLAY and HYPRLAND_INSTANCE_SIGNATURE to the file named by its
# first argument (KEY=VALUE lines), and keeps running until killed.
# Ubuntu's arm64 hyprland is 0.53 (no Lua config) and the PPA is amd64 only;
# nixpkgs' nixos-unstable has 0.56.2 (prefer hyprland.cachix.org). Anything
# it puts in /nix must be realised at run time (`test/run warm --hyprland`),
# because the dotfiles-nix volume hides the image's /nix.
FROM desktop AS hyprland

# ---------------------------------------------------------------------------
FROM hyprland AS harness
# Mount points for the caches, owned by kuba so the volumes start that way.
RUN install -d -o kuba -g kuba -m 0755 /var/cache/dotfiles-test /out \
 && install -d -o kuba -g kuba -m 0700 /home/kuba/.cache \
 && install -d -o kuba -g kuba -m 0755 /home/kuba/.local /home/kuba/.local/share /home/kuba/Projects
CMD ["sleep", "infinity"]
