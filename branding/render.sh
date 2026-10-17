#!/usr/bin/env bash
# Renders branding/svg/*.svg to branding/png/ and writes fastfetch.txt.
# Needs python3, resvg (brew install resvg) and ImageMagick (magick).
set -euo pipefail
cd "$(dirname "$0")"
python3 make.py
rm -rf png && mkdir -p png
for s in 16 32 64 128 256 512 1024; do
  # The tile is the app icon; 16 and 32 use the flat, fat-stroke drawing.
  if [ "$s" -le 32 ]; then src=svg/kubaos-icon-small.svg; else src=svg/kubaos-icon.svg; fi
  resvg -w "$s" "$src" "png/kubaos-icon-$s.png"
  resvg -w "$s" svg/kubaos-mark.svg "png/kubaos-mark-$s.png"
  resvg -w "$s" svg/kubaos-mark-white.svg "png/kubaos-mark-white-$s.png"
  resvg -w "$s" svg/kubaos-mark-black.svg "png/kubaos-mark-black-$s.png"
done
for f in horizontal horizontal-white stacked wordmark; do
  resvg -h 256 "svg/kubaos-$f.svg" "png/kubaos-$f.png"
done
resvg -h 256 svg/kubaos-horizontal-dark.svg png/kubaos-horizontal-dark.png
resvg -h 512 --background '#120d0a' svg/kubaos-stacked.svg png/kubaos-stacked-dark.png
resvg -w 1920 svg/kubaos-splash.svg png/kubaos-splash-1920x1080.png
python3 fastfetch.py > fastfetch.txt
echo "rendered png/ and fastfetch.txt"
