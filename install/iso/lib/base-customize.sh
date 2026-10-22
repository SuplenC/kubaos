#!/usr/bin/env bash
# Runs in the chroot at the end of the base stage (install/iso/lib/build-iso.sh).
# Changing this file invalidates the cached base rootfs.
set -euo pipefail

echo kubaos-live >/etc/hostname
printf '127.0.0.1 localhost\n127.0.1.1 kubaos-live\n' >/etc/hosts

# casper: the live user and hostname (read from the initramfs copy)
cat >/etc/casper.conf <<'EOF'
export USERNAME="ubuntu"
export USERFULLNAME="Live session user"
export HOST="kubaos-live"
export BUILD_SYSTEM="Ubuntu"
export FLAVOUR="KubaOS"
EOF

sed -i 's/^# *\(en_US.UTF-8 UTF-8\)/\1/' /etc/locale.gen
locale-gen >/dev/null
echo 'LANG=C.UTF-8' >/etc/default/locale

# boot fast: nothing waits for the network, no timers, no motd
systemctl set-default multi-user.target
for u in NetworkManager-wait-online.service systemd-networkd-wait-online.service \
  apt-daily.timer apt-daily-upgrade.timer e2scrub_all.timer e2scrub_reap.service \
  fstrim.timer motd-news.timer dpkg-db-backup.timer man-db.timer \
  ssh.service ssh.socket; do
  systemctl disable "$u" 2>/dev/null || true
done
systemctl mask systemd-networkd-wait-online.service NetworkManager-wait-online.service 2>/dev/null || true
chmod -x /etc/update-motd.d/* 2>/dev/null || true

mkdir -p /etc/systemd/journald.conf.d
printf '[Journal]\nStorage=volatile\nRuntimeMaxUse=64M\n' >/etc/systemd/journald.conf.d/volatile.conf

# The `default` XCursor theme is Bibata-Modern-Classic, the installed system's cursor.
# bibata-cursor-theme registers the x-cursor-theme alternative only when another theme
# already has, and its own cursor.theme quotes the Inherits= value, which the XCursor
# loaders of wlroots and libwayland-cursor keep as part of the theme name. So the file is
# written here, not linked.
mkdir -p /usr/share/icons/default
rm -f /usr/share/icons/default/index.theme
printf '[Icon Theme]\nName=Default\nInherits=Bibata-Modern-Classic\n' >/usr/share/icons/default/index.theme

# GRUB/os-prober and kernel hooks must not run in the live system
rm -f /etc/kernel/postinst.d/zz-update-grub

update-initramfs -u -k all
