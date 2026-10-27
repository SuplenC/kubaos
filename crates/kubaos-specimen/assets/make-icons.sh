#!/usr/bin/env bash
# Regenerates the icon comparison sheet's fonts in icons/: the 40 glyphs of
# glyphs.tsv from Material Symbols Rounded, Phosphor, Lucide and Tabler, each
# a subset under its own family name, plus icons.tsv with their codepoints.
#
#   crates/kubaos-specimen/assets/make-icons.sh
#
# Needs curl, tar, sha256sum and Nix; fontTools comes from the repo's pinned
# nixpkgs. Every source is pinned: Material Symbols by flake.lock's nixpkgs
# (its font, and its codepoints at the same upstream commit), the others by
# their npm tarball's hash.
# - Material Symbols Rounded (Apache 2.0): outlined, weight 400, optical size
#   24, as Kōbō draws it.
# - Phosphor 2.1.2, regular weight (MIT).
# - Lucide 1.61.0 (ISC).
# - Tabler 3.49.0, the 2 px stroke (MIT).
set -euo pipefail
cd "$(dirname "$0")"
repo=$(git rev-parse --show-toplevel)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# name url sha256
packages=(
  "phosphor https://registry.npmjs.org/@phosphor-icons/web/-/web-2.1.2.tgz 40e3096099ca818c047979cece6a3764944d5200c67b64092f7bcd8bcd1d2b08"
  "lucide https://registry.npmjs.org/lucide-static/-/lucide-static-1.61.0.tgz 62773cd2ee411ad2e37c1973d1e4c4b49f9a6c57da6b5d5d0abca08756ba75f1"
  "tabler https://registry.npmjs.org/@tabler/icons-webfont/-/icons-webfont-3.49.0.tgz 48c69c65a5f9013d482a17cab8899a09dcaec50750d1ac7324eb63a061fffb82"
)
for package in "${packages[@]}"; do
  read -r name url sha256 <<<"$package"
  curl -fsSL -o "$tmp/$name.tgz" "$url"
  echo "$sha256  $tmp/$name.tgz" | sha256sum -c --quiet
  mkdir "$tmp/$name"
  tar -xzf "$tmp/$name.tgz" -C "$tmp/$name"
done

# nixpkgs' material-symbols is google/material-design-icons at this commit.
material_rev=5a4e1b7fd26f11ce3d2abf7d7fcd13274f874e6c
material_codepoints_sha256=8e8698bf6cc7d4071f5af64c343109c72e3a43a3e4b2f49e02153de6fd95612a
material=$(nix build --no-link --print-out-paths --inputs-from "$repo" nixpkgs#material-symbols)
curl -fsSL -o "$tmp/material.codepoints" \
  "https://raw.githubusercontent.com/google/material-design-icons/$material_rev/variablefont/MaterialSymbolsRounded%5BFILL,GRAD,opsz,wght%5D.codepoints"
echo "$material_codepoints_sha256  $tmp/material.codepoints" | sha256sum -c --quiet

# nixpkgs' Python, with its fontTools on the path.
python=$(nix build --no-link --print-out-paths --inputs-from "$repo" nixpkgs#python3)
fonttools=$(nix build --no-link --print-out-paths --inputs-from "$repo" nixpkgs#python3Packages.fonttools)
PYTHONPATH=$(echo "$fonttools"/lib/python3*/site-packages) "$python/bin/python3" -P icons.py glyphs.tsv icons \
  "$material/share/fonts/truetype/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf" "$tmp/material.codepoints" \
  "$tmp/phosphor/package/src/regular/Phosphor.ttf" "$tmp/phosphor/package/src/regular/style.css" \
  "$tmp/lucide/package/font/lucide.ttf" "$tmp/lucide/package/font/lucide.css" \
  "$tmp/tabler/package/dist/fonts/tabler-icons.ttf" "$tmp/tabler/package/dist/tabler-icons.css"

cp "$repo/crates/kubaos-kobo/assets/fonts/MaterialSymbols-LICENSE.txt" icons/material-LICENSE.txt
cp "$tmp/phosphor/package/LICENSE" icons/phosphor-LICENSE.txt
cp "$tmp/lucide/package/LICENSE" icons/lucide-LICENSE.txt
cp "$tmp/tabler/package/LICENSE" icons/tabler-LICENSE.txt
