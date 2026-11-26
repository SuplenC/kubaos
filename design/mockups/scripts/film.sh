#!/usr/bin/env bash
# design/mockups/scripts/film.sh: the media player mockup's film, a five-minute cut of Sintel
# encoded for the browser, made once for every checkout of the repo on this machine.
#
#   film.sh [SOURCE]   SOURCE: Sintel.2010.720p.mkv, or the .zip it comes in; downloaded
#                      from the cut file's `url` when none is given
#
# The cut file, `src/assets/video/sintel.json`, says what is cut: the source's frames from
# `start` to `end` at `rate`, the chapters by the cut's frames, the tracks' names. Into
# $KUBAOS_MOCKUP_FILM, ${XDG_CACHE_HOME:-$HOME/.cache}/kubaos-mockups/film by default, it writes
# `sintel.webm` (AV1 and stereo Opus), `sintel.LANG.vtt` for each subtitle track of the mkv, and
# last `sintel.json`, the manifest the mockup site reads. Nothing goes into the work tree.
#
# It does nothing when the manifest is there for the same cut file and encoding. The source's
# mkv must match the cut file's md5, or nothing is written. Every file is written beside its name
# and renamed into place, so a half-written one never shows; a downloaded source is kept, checked,
# in `source/` for the next cut.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cut="$here/../src/assets/video/sintel.json"
out="${KUBAOS_MOCKUP_FILM:-${XDG_CACHE_HOME:-$HOME/.cache}/kubaos-mockups/film}"
manifest="$out/sintel.json"

# The picture keeps its own 1280 × 544; 10-bit, which keeps the dark scenes' gradients from
# banding at this rate, and a keyframe every second, so a seek or a frame preview decodes at
# most 24 frames. The 5.1 sound is mixed down to stereo by FFmpeg's own matrix.
video=(-c:v libsvtav1 -preset 5 -crf 30 -g 24 -pix_fmt yuv420p10le -svtav1-params "tune=0:enable-variance-boost=1:irefresh-type=2")
audio=(-c:a libopus -b:a 160k -ac 2)

usage() { sed -n '2,/^set -euo/{/^set -euo/d; s/^# \{0,1\}//p;}' "$0"; }

fail() {
  echo "mockup-film: $*" >&2
  exit 1
}

case "${1:-}" in
  -h | --help)
    usage
    exit 0
    ;;
  -*)
    usage >&2
    exit 2
    ;;
esac
(($# <= 1)) || {
  usage >&2
  exit 2
}
source_given="${1:-}"

for tool in ffmpeg ffprobe jq md5sum sha256sum unzip curl; do
  command -v "$tool" >/dev/null || fail "$tool isn't on PATH; the dev shell has jq and curl, and FFmpeg (with libsvtav1 and libopus) and unzip come from the system"
done

key="$({
  cat "$cut"
  printf '%s\n' "${video[*]}" "${audio[*]}"
} | sha256sum | cut -d' ' -f1)"

if [[ -f $manifest ]] && [[ $(jq -r '.key // ""' "$manifest") == "$key" ]]; then
  echo "mockup-film: $manifest is up to date"
  exit 0
fi

encoders="$(ffmpeg -hide_banner -encoders 2>/dev/null)"
for encoder in libsvtav1 libopus; do
  [[ $encoders == *" $encoder "* ]] || fail "this FFmpeg ($(command -v ffmpeg)) has no $encoder encoder"
done

mkdir -p "$out"
work="$(mktemp -d "$out/.work.XXXXXX")"
trap 'rm -rf -- "$work"' EXIT

mkv_name="$(jq -r .mkv "$cut")"
md5="$(jq -r .md5 "$cut")"

# The source's mkv: the one given, out of its zip if it is one; else the one kept from a download;
# else downloaded and kept.
kept="$out/source/$mkv_name"
if [[ -n $source_given ]]; then
  [[ -f $source_given ]] || fail "no file at $source_given"
  if [[ $source_given == *.zip ]]; then
    echo "mockup-film: unzipping $source_given"
    unzip -p "$source_given" "$mkv_name" >"$work/$mkv_name" || fail "$source_given holds no $mkv_name"
    mkv="$work/$mkv_name"
  else
    mkv="$source_given"
  fi
elif [[ -f $kept ]]; then
  mkv="$kept"
else
  url="$(jq -r .url "$cut")"
  echo "mockup-film: downloading $url"
  curl -fL --retry 3 -o "$work/source.zip" "$url" || fail "the download of $url failed"
  unzip -p "$work/source.zip" "$mkv_name" >"$work/$mkv_name" || fail "the download holds no $mkv_name"
  rm -f -- "$work/source.zip"
  mkv="$work/$mkv_name"
fi

echo "mockup-film: checking $mkv"
[[ $(md5sum "$mkv" | cut -d' ' -f1) == "$md5" ]] || fail "$mkv doesn't match md5 $md5; nothing written"
if [[ $mkv == "$work/$mkv_name" && -z $source_given ]]; then
  mkdir -p "$out/source"
  mv -f -- "$mkv" "$kept"
  mkv="$kept"
fi

rate="$(jq -r .rate "$cut")"
start="$(jq -r .start "$cut")"
end="$(jq -r .end "$cut")"
# A millisecond before the first frame, so a seek that rounds never drops it; the length in
# whole frames, so the cut ends before the frame at `end`.
from="$(awk -v f="$start" -v r="$rate" 'BEGIN { printf "%.6f", f / r - 0.001 }')"
length="$(awk -v a="$start" -v b="$end" -v r="$rate" 'BEGIN { printf "%.6f", (b - a) / r }')"

echo "mockup-film: encoding ${length}s from ${from}s"
began=$SECONDS
ffmpeg -hide_banner -nostdin -loglevel error -stats -ss "$from" -i "$mkv" -t "$length" \
  -map 0:v:0 -map 0:a:0 "${video[@]}" "${audio[@]}" -sn -map_metadata -1 -map_chapters -1 \
  -cues_to_front 1 -f webm "$work/sintel.webm"
echo "mockup-film: encoded in $((SECONDS - began)) s"

# Each subtitle track the mkv holds, re-timed to the cut by the same seek, named by the cut file.
mapfile -t langs < <(ffprobe -v error -select_streams s -show_entries stream_tags=language -of csv=p=0 "$mkv")
subtitles=()
for i in "${!langs[@]}"; do
  lang="${langs[$i]}"
  [[ $lang =~ ^[a-z]{3}$ ]] || fail "subtitle track $i has no language code"
  ffmpeg -hide_banner -nostdin -loglevel error -ss "$from" -i "$mkv" -t "$length" \
    -map "0:s:$i" -c:s webvtt -f webvtt "$work/sintel.$lang.vtt"
  subtitles+=("$lang")
done

measured="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$work/sintel.webm")"
jq -n --slurpfile cut "$cut" --arg key "$key" --argjson length "$measured" \
  --args '
  $cut[0] as $c
  | {
      key: $key,
      title: $c.title,
      by: $c.by,
      length: $length,
      video: "sintel.webm",
      chapters: [$c.chapters[] | { at: (.at / $c.rate), name }],
      audio: [{ name: $c.audio }],
      subtitles: [
        $ARGS.positional[] as $lang
        | { lang: $lang, name: (($c.subtitles[] | select(.lang == $lang) | .name) // $lang), file: "sintel.\($lang).vtt" }
      ]
    }' "${subtitles[@]}" >"$work/manifest.json"

# The old manifest goes first and the new one last: a manifest names only files already there.
rm -f -- "$manifest"
mv -f -- "$work/sintel.webm" "$out/sintel.webm"
for lang in "${subtitles[@]}"; do mv -f -- "$work/sintel.$lang.vtt" "$out/sintel.$lang.vtt"; done
mv -f -- "$work/manifest.json" "$manifest"
echo "mockup-film: wrote $out ($(du -h "$out/sintel.webm" | cut -f1) film, ${#subtitles[@]} subtitle tracks)"
