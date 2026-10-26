#!/usr/bin/env bash
# Regenerates the fonts kubaos-ui bundles, so an iced app draws with them on a
# machine that has none installed (Kōbō on the live ISO).
#
#   crates/kubaos-ui/assets/make-fonts.sh
#
# Needs curl, unzip, sha256sum and Nix; fontTools comes from the repo's pinned
# nixpkgs. Every source is pinned: Geist by its release zip's hash, M PLUS 1
# by flake.lock's nixpkgs.
# - Geist and Geist Mono 1.8.1 (SIL OFL 1.1): the weights the type styles use.
# - M PLUS 1 Medium (SIL OFL 1.1): a subset with only the characters of `jp`
#   below, the Japanese accents the apps show.
# KubaOS Symbols are drawings, not a font: make-symbols.sh fetches them.
set -euo pipefail
cd "$(dirname "$0")/fonts"
repo=$(git rev-parse --show-toplevel)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

geist_url=https://github.com/vercel/geist-font/releases/download/v1.8.1/geist-font-v1.8.1.zip
geist_sha256=70f42c1452fb6df12ea183e20a65999157354f91d11dac18b4c713e383808ec2

# Every Japanese character an app shows: Kōbō's accents, then the specimen's.
jp="ようこそ言語と入力時間帯接続アカウントディスク署名確認インストール中完了失敗初期設定壁紙"
jp+="書体間隔色部品動き記号見本"

curl -fsSL -o "$tmp/geist.zip" "$geist_url"
echo "$geist_sha256  $tmp/geist.zip" | sha256sum -c --quiet
unzip -qo "$tmp/geist.zip" -d "$tmp/geist"
for w in Regular Medium SemiBold; do
  cp "$tmp/geist/geist-font/Geist/ttf/Geist-$w.ttf" .
done
for w in Regular Medium; do
  cp "$tmp/geist/geist-font/GeistMono/ttf/GeistMono-$w.ttf" .
done
cp "$tmp/geist/geist-font/OFL.txt" Geist-OFL.txt

mplus=$(nix build --no-link --print-out-paths --inputs-from "$repo" nixpkgs#mplus-outline-fonts.githubRelease)
nix shell --inputs-from "$repo" nixpkgs#python3Packages.fonttools -c \
  pyftsubset "$mplus/share/fonts/truetype/mplus-outline-fonts/Mplus1-Medium.ttf" \
  --text="$jp" --layout-features='*' --output-file=MPLUS1-subset.ttf
curl -fsSL -o MPLUS1-OFL.txt https://raw.githubusercontent.com/google/fonts/main/ofl/mplus1/OFL.txt

