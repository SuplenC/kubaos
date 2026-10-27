#!/usr/bin/env bash
# Regenerates Kōbō's own pictures. Needs ImageMagick (magick). Its fonts and
# symbols are kubaos-ui's (crates/kubaos-ui/assets).
# Wallpaper: cyberpunk/shinjuku-neon-canyon.jpg from a SuplenC/kubaos-wallpapers
# checkout (WALLPAPERS, default ../kubaos-wallpapers next to this repo), shrunk, plus
# a heavily blurred copy for the glass panels.
set -euo pipefail
cd "$(dirname "$0")"
repo=$(git rev-parse --show-toplevel)

wp="${WALLPAPERS:-$repo/../kubaos-wallpapers}/cyberpunk/shinjuku-neon-canyon.jpg"
magick "$wp" -resize 2560x -strip -quality 84 wallpaper.jpg
magick "$wp" -resize 640x -blur 0x18 -strip -quality 90 wallpaper-blur.jpg

# The KubaOS logo (branding/, see its README): the vector mark and wordmark for
# small sizes, the neon sign for the welcome screen. The crate carries copies,
# so it builds from its own directory.
cp "$repo/branding/svg/kubaos-mark.svg" "$repo/branding/svg/kubaos-wordmark.svg" \
  "$repo/branding/png/kubaos-sign-1024.png" logo/
