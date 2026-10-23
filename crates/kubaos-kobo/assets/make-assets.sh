#!/usr/bin/env bash
# Regenerates the bundled assets. Needs curl, unzip, ImageMagick (magick)
# and fontTools (pip install fonttools).
# Fonts: Geist and JetBrains Mono (SIL OFL 1.1), M PLUS 1 (SIL OFL 1.1, a
# subset with only the Japanese accents Kōbō shows), Material Symbols
# Rounded (Apache 2.0, a subset with only the icons in src/theme.rs).
# Wallpaper: cyberpunk/shinjuku-neon-canyon.jpg from a SuplenC/kubaos-wallpapers
# checkout (WALLPAPERS, default ../kubaos-wallpapers next to this repo), shrunk, plus
# a heavily blurred copy for the glass panels.
set -euo pipefail
cd "$(dirname "$0")"
repo=$(git rev-parse --show-toplevel)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Keep in sync with JP_ACCENTS in src/theme.rs.
jp="ようこそ言語と入力時間帯接続アカウントディスク署名確認インストール中完了失敗初期設定壁紙"
# Keep in sync with the Icon enum in src/theme.rs (Google wants them sorted).
icons="account_circle arrow_back arrow_forward badge bolt check check_circle chevron_right circle close code computer deployed_code download draft error expand_less expand_more folder_open hard_drive key keyboard language lock lock_open mail network_wifi_1_bar network_wifi_2_bar network_wifi_3_bar person power_settings_new progress_activity public radio_button_checked radio_button_unchecked restart_alt rocket_launch save schedule search signal_wifi_4_bar storage terminal timer visibility visibility_off wallpaper warning wifi wifi_lock wifi_off"

gfont() { # css url -> first font url in it
  curl -fsSL "$1" | grep -o 'https://fonts.gstatic.com[^)]*' | head -1
}

curl -fsSL -o "$tmp/geist.zip" https://github.com/vercel/geist-font/releases/download/v1.8.1/geist-font-v1.8.1.zip
unzip -qo "$tmp/geist.zip" -d "$tmp/geist"
for w in Regular Medium SemiBold Bold Light; do
  cp "$tmp/geist/geist-font/Geist/ttf/Geist-$w.ttf" fonts/
done
cp "$tmp/geist/geist-font/OFL.txt" fonts/Geist-OFL.txt

curl -fsSL -o "$tmp/jb.zip" https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip
unzip -qo "$tmp/jb.zip" -d "$tmp/jb"
cp "$tmp/jb/fonts/ttf/JetBrainsMono-Regular.ttf" fonts/
cp "$tmp/jb/OFL.txt" fonts/JetBrainsMono-OFL.txt

text=$(python3 -c 'import sys,urllib.parse;print(urllib.parse.quote(sys.argv[1]))' "$jp")
curl -fsSL -o fonts/MPLUS1-subset.ttf "$(gfont "https://fonts.googleapis.com/css2?family=M+PLUS+1:wght@500&text=$text")"
curl -fsSL -o fonts/MaterialSymbolsRounded-subset.ttf \
  "$(gfont "https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@24,400,0,0&icon_names=$(echo $icons | tr ' ' ,)")"
# Its own family name, so the installed system's full Material Symbols
# (different codepoints for some icons) can't shadow it. Needs fontTools.
python3 rename-icon-font.py fonts/MaterialSymbolsRounded-subset.ttf

wp="${WALLPAPERS:-$repo/../kubaos-wallpapers}/cyberpunk/shinjuku-neon-canyon.jpg"
magick "$wp" -resize 2560x -strip -quality 84 wallpaper.jpg
magick "$wp" -resize 640x -blur 0x18 -strip -quality 90 wallpaper-blur.jpg

curl -fsSL -o fonts/MPLUS1-OFL.txt https://raw.githubusercontent.com/google/fonts/main/ofl/mplus1/OFL.txt
curl -fsSL -o fonts/MaterialSymbols-LICENSE.txt https://raw.githubusercontent.com/google/material-design-icons/master/LICENSE

# The KubaOS logo (branding/, see its README): the vector mark and wordmark for
# small sizes, the neon sign for the welcome screen. The crate carries copies,
# so it builds from its own directory.
cp "$repo/branding/svg/kubaos-mark.svg" "$repo/branding/svg/kubaos-wordmark.svg" \
  "$repo/branding/png/kubaos-sign-1024.png" logo/
