#!/usr/bin/env bash
# Makes neon-sign-front@2x.jpg, the master every photo asset is drawn from
# (branding/neon.py): the photo upscaled 4x with Upscayl's remacri model, then
# halved. The large uses (the boot splash at 2x, the README hero) come out
# sharp instead of soft.
#
#   branding/photo/upscale.sh
#
# Needs Upscayl and ImageMagick (magick). UPSCAYL points at the directory that
# holds upscayl-bin and models/ (default: the macOS app's).
set -euo pipefail
cd "$(dirname "$0")"
upscayl="${UPSCAYL:-/Applications/Upscayl.app/Contents/Resources}"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

"$upscayl/bin/upscayl-bin" -i neon-sign-front.png -o "$tmp/4x.png" \
  -m "$upscayl/models" -n remacri-4x -s 4 -f png
magick "$tmp/4x.png" -filter Lanczos -resize 50% -strip -sampling-factor 4:4:4 -quality 92 neon-sign-front@2x.jpg
echo "wrote branding/photo/neon-sign-front@2x.jpg"
