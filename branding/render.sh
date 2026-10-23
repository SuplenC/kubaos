#!/usr/bin/env bash
# Renders every generated branding asset: the vector marks (make.py, then
# branding/png/), the terminal logo (fastfetch.txt), the neon-sign photo assets
# (neon.py) and the boot assets (boot.py, into system/files/).
#
#   branding/render.sh
#
# Needs resvg, ImageMagick (magick) and a python3 with Pillow and numpy;
# PYTHON picks the interpreter (default python3).
set -euo pipefail
cd "$(dirname "$0")"
python="${PYTHON:-python3}"
"$python" make.py
rm -rf png && mkdir -p png
for s in 16 32 64 128 256 512 1024; do
  # The tile is the app icon; 16 and 32 use the flat, fat-stroke drawing.
  if [[ $s -le 32 ]]; then src=svg/kubaos-icon-small.svg; else src=svg/kubaos-icon.svg; fi
  resvg -w "$s" "$src" "png/kubaos-icon-$s.png"
  resvg -w "$s" svg/kubaos-mark.svg "png/kubaos-mark-$s.png"
  resvg -w "$s" svg/kubaos-mark-white.svg "png/kubaos-mark-white-$s.png"
  resvg -w "$s" svg/kubaos-mark-black.svg "png/kubaos-mark-black-$s.png"
done
for f in horizontal horizontal-white stacked wordmark; do
  resvg -h 256 "svg/kubaos-$f.svg" "png/kubaos-$f.png"
done
resvg -h 256 svg/kubaos-horizontal-dark.svg png/kubaos-horizontal-dark.png
# surface_container_lowest of config/matugen/default-colors.json.
resvg -h 512 --background '#140d07' svg/kubaos-stacked.svg png/kubaos-stacked-dark.png
"$python" fastfetch.py > fastfetch.txt
"$python" neon.py
"$python" boot.py
echo "rendered png/, fastfetch.txt and the boot assets"
