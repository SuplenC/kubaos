#!/usr/bin/env bash
# Regenerates locales.tsv and keymaps.tsv: display names for every UTF-8
# locale glibc supports and every XKB layout and variant, taken from the same
# Ubuntu release the live ISO and the payload use, so the names match what
# `kubaos probe` lists. Needs Docker.
#
#   locales.tsv  en_SE.UTF-8<TAB>English (Sweden)
#   keymaps.tsv  us<TAB>English (US)     us(intl)<TAB>English (US, intl., with dead keys)
set -euo pipefail
cd "$(dirname "$0")"
docker run --rm -i ubuntu:26.04 bash -s >"$PWD/tables.out" <<'EOF'
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq >/dev/null
apt-get install -y -qq --no-install-recommends locales xkb-data >/dev/null
# glibc: LC_IDENTIFICATION's language and territory, per SUPPORTED UTF-8 entry.
while read -r name charset; do
  [ "$charset" = UTF-8 ] || continue
  case "$name" in \#*) continue ;; esac
  base=${name%%.*}; mod=
  case "$name" in *@*) mod=${name##*@} ;; esac
  f=/usr/share/i18n/locales/$base${mod:+@$mod}
  [ -f "$f" ] || f=/usr/share/i18n/locales/$base
  [ -f "$f" ] || continue
  ident=$(sed -n '/^LC_IDENTIFICATION/,/^END LC_IDENTIFICATION/p' "$f")
  lang=$(sed -n 's/^language *"\(.*\)"/\1/p' <<<"$ident" | head -1)
  terr=$(sed -n 's/^territory *"\(.*\)"/\1/p' <<<"$ident" | head -1)
  printf 'L\t%s\t%s\t%s\t%s\n' "$name" "$lang" "$terr" "$mod"
done </usr/share/i18n/SUPPORTED
# xkb: layouts and variants with their descriptions.
awk '
  /^! /{ sec=$2; next }
  sec=="layout" && NF { n=$1; $1=""; sub(/^ +/,""); print "K\t" n "\t" $0 }
  sec=="variant" && NF { v=$1; l=$2; sub(/:$/,"",l); $1=""; $2=""; sub(/^ +/,""); print "K\t" l "(" v ")\t" $0 }
' /usr/share/X11/xkb/rules/base.lst
EOF
python3 - "$PWD/tables.out" <<'PY'
import sys
loc, key = {}, {}
# glibc leaves these without a language or calls it something odd.
fix_lang = {"American English": "English", "British English": "English", "C": "Plain C"}
missing = {"C.UTF-8": "None (plain C, English messages)", "bi_VU": "Bislama (Vanuatu)",
           "crh_RU": "Crimean Tatar (Russia)", "crh_UA": "Crimean Tatar (Ukraine)"}
for line in open(sys.argv[1], encoding="utf-8"):
    f = line.rstrip("\n").split("\t")
    if f[0] == "L":
        name, lang, terr, mod = f[1], f[2], f[3], f[4]
        if not lang:
            if name in missing:
                loc[name] = missing[name]
            continue
        lang = fix_lang.get(lang, lang)
        # "Canadian French (Canada)" says it twice; the territory is enough.
        for base in ("English", "French", "German", "Spanish", "Portuguese"):
            if lang.endswith(" " + base) and lang.split()[0] != "Low":
                lang = base
        lang = lang[:1].upper() + lang[1:]
        extra = [t for t in [terr, mod.capitalize() if mod else ""] if t]
        loc[name] = f"{lang} ({', '.join(extra)})" if extra else lang
    elif f[0] == "K":
        key[f[1]] = f[2]
with open("locales.tsv", "w", encoding="utf-8") as o:
    for k in sorted(loc):
        o.write(f"{k}\t{loc[k]}\n")
with open("keymaps.tsv", "w", encoding="utf-8") as o:
    for k in sorted(key):
        o.write(f"{k}\t{key[k]}\n")
print(f"{len(loc)} locales, {len(key)} keymaps")
PY
rm -f tables.out
