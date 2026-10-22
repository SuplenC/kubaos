# A tiny stand-in for the real payload: Ubuntu 26.04 with GRUB (EFI),
# snapper, zsh, locales, no users, an empty machine-id, a fake Nix store
# with a home-manager "generation" and a fake nix-daemon, and KubaOS's
# checkout with the user flake's template and lock where the image puts them.
FROM ubuntu:26.04
RUN set -eux; \
    case "$(dpkg --print-architecture)" in arm64) g=grub-efi-arm64-bin ;; amd64) g=grub-efi-amd64-bin ;; esac; \
    apt-get update; \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      "$g" grub2-common grub-common btrfs-progs snapper zsh sudo locales tzdata socat \
      util-linux passwd dbus; \
    rm -rf /var/lib/apt/lists/*; \
    userdel -r ubuntu 2>/dev/null || true; \
    : > /etc/machine-id
COPY activate /nix/store/0000000000000000000000000000test-home-manager-generation/activate
# Laid out like the real thing: the profile is a chain of links ending in an
# absolute /nix/store path, which only resolves inside the target.
COPY nix-daemon /nix/store/0000000000000000000000000000test-nix/bin/nix-daemon
RUN set -eux; \
    chmod 0755 /nix/store/*/activate /nix/store/*-nix/bin/nix-daemon; \
    mkdir -p /nix/var/nix/profiles/per-user /nix/var/nix/gcroots/per-user /nix/var/nix/daemon-socket; \
    ln -s /nix/store/0000000000000000000000000000test-nix /nix/var/nix/profiles/default-1-link; \
    ln -s default-1-link /nix/var/nix/profiles/default; \
    mkdir -p /usr/share/kubaos/user-flake /usr/share/kubaos-image; \
    echo 'homeConfigurations."@USER@@@HOST@" = mkHome "@SYSTEM@";' > /usr/share/kubaos/user-flake/flake.nix.in; \
    echo '{ }' > /usr/share/kubaos/user-flake/home.nix; \
    echo '{"version": 7}' > /usr/share/kubaos-image/user-flake.lock
