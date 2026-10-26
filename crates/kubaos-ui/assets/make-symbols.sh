#!/usr/bin/env bash
# Fetches KubaOS Symbols' Tabler drawings into symbols/: for every entry of
# design/tokens.toml's [symbols] that names a Tabler icon, its outline SVG as
# symbols/<name>.svg, plus Tabler's licence. The `drawn` ones are ours and
# live in symbols/ already; this never touches them.
#
#   crates/kubaos-ui/assets/make-symbols.sh
#
# Needs curl, tar, sha256sum and python3 (3.11 or later, for tomllib). The
# package is pinned by its hash: @tabler/icons 3.49.0 (MIT).
set -euo pipefail
cd "$(dirname "$0")"
repo=$(git rev-parse --show-toplevel)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

url=https://registry.npmjs.org/@tabler/icons/-/icons-3.49.0.tgz
sha256=aa3042907eb0bdb079fd70733e2d75f785683d4726279128a0ca2343099f7074

curl -fsSL -o "$tmp/icons.tgz" "$url"
echo "$sha256  $tmp/icons.tgz" | sha256sum -c --quiet
tar -xzf "$tmp/icons.tgz" -C "$tmp"

mkdir -p symbols
python3 -I - "$repo/design/tokens.toml" "$tmp/package/icons/outline" symbols <<'PY'
import shutil, sys, tomllib
tokens, outline, out = sys.argv[1:]
for name, symbol in tomllib.load(open(tokens, "rb"))["symbols"].items():
    if "tabler" in symbol:
        shutil.copyfile(f"{outline}/{symbol['tabler']}.svg", f"{out}/{name}.svg")
PY
cp "$tmp/package/LICENSE" symbols/TABLER-LICENSE.txt
