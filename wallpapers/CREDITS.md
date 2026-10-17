# Wallpapers

Tokyo, for `dot-theme` and the Quickshell picker. `home/theme.nix` links this
directory to `~/Pictures/wallpapers`; drop more images or videos into any
folder and they show up in the picker (no rebuild). Three folders:
`shinjuku/` (KubaOS Shinjuku's own set, five stills),
`japanese/` and `cyberpunk/`. The live scenes of
[ukiyo](../docs/UKIYO.md) aren't files in here: they come with the ukiyo
package, and the picker lists them first in the Shinjuku tab.

Stills are the photographer's original, downscaled to 3840 px wide (Lanczos)
and re-encoded as progressive JPEG, quality 90, 4:4:4 chroma, metadata
stripped. All are 3:2 (or close), so they crop cleanly to both 16:10
(2880x1800 laptop) and 16:9 (1920x1080 externals).

The default (first boot, and the source of `config/matugen/default-colors.json`)
is the live scene `ukiyo:kabukicho-gate`, through its poster. Without ukiyo,
`dot-theme` falls back to the first wallpaper of the folder. The installer
GUI still uses `cyberpunk/shinjuku-neon-canyon.jpg`.

## Loops

A wallpaper can still be a video: `<name>.mp4` (or webm, mkv, mov) next to
`<name>.poster.jpg`. Drop one in a folder and the picker plays it. The set
shipped here has no videos any more: the old HEVC loops and their tooling
live in the kubaos-media archive, and the live scene below replaced them.

## The live scene

| Scene | Photographer | Source | License |
|---|---|---|---|
| `ukiyo:kabukicho-gate` (Kabukichō Gate) | Basile Morin | [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Kabukicho_red_gate_and_colorful_neon_street_signs_at_night,_Shinjuku,_Tokyo,_Japan.jpg) ("Kabukicho red gate and colorful neon street signs at night, Shinjuku, Tokyo, Japan"). The depth, masks and poster are derived from it by ukiyo's `tools/`; the rain, light and reactions are ukiyo's | [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) |

CC BY-SA means: credit the photographer (done, here and in the scene's
`scene.toml`), and anything derived from the photo, the scene's textures and
poster included, stays CC BY-SA. That's fine for an OS and for a public
scene. The files live in the ukiyo repository, not here.

## Credits

| File | Size | Photographer | Source | License |
|---|---|---|---|---|
| `shinjuku/kabukicho-red-gate.jpg` | 3840x2560 | Manuel Cosentino | [unsplash.com/photos/d92I1Of2ofQ](https://unsplash.com/photos/d92I1Of2ofQ) ("Kabukicho, Shinjuku, Japan") | [Unsplash License](https://unsplash.com/license) |
| `shinjuku/omoide-yokocho-lanterns.jpg` | 3840x2560 | Pema G. Lama | [unsplash.com/photos/6cfK0SEtpbY](https://unsplash.com/photos/6cfK0SEtpbY) ("Omoide Yokochō, Nishishinjuku") | [Unsplash License](https://unsplash.com/license) |
| `shinjuku/skyline-fuji-dusk.jpg` | 3840x2560 | LoboStudio Hamburg | [unsplash.com/photos/qKZzNC5mqTw](https://unsplash.com/photos/qKZzNC5mqTw) ("Shinjuku, Tokyo, Japan") | [Unsplash License](https://unsplash.com/license) |
| `shinjuku/nishi-shinjuku-violet-rain.jpg` | 3840x2560 | ayumi kubo | [unsplash.com/photos/zIja6vTunL4](https://unsplash.com/photos/zIja6vTunL4) ("1 Chome-11-1 Nishishinjuku") | [Unsplash License](https://unsplash.com/license) |
| `shinjuku/train-neon-streak.jpg` | 3840x2556 | Letian Zhang | [unsplash.com/photos/UvXATPmGkbc](https://unsplash.com/photos/UvXATPmGkbc) ("Shinjuku, Tokyo, Japan") | [Unsplash License](https://unsplash.com/license) |
| `cyberpunk/shinjuku-neon-canyon.jpg` | 3840x2560 | Pema G. Lama | [unsplash.com/photos/Lfh0wUAJXpA](https://unsplash.com/photos/people-walking-on-street-during-night-time-Lfh0wUAJXpA) | [Unsplash License](https://unsplash.com/license) |
| `cyberpunk/shibuya-rainy-corner.jpg` | 3840x2561 | Alex Knight | [unsplash.com/photos/DpPutJwgyW8](https://unsplash.com/photos/people-walking-near-buildings-at-night-DpPutJwgyW8) | [Unsplash License](https://unsplash.com/license) |
| `cyberpunk/akihabara-after-rain.jpg` | 3840x2491 | Jezael Melgoza | [unsplash.com/photos/layMbSJ3YOE](https://unsplash.com/photos/people-walking-on-road-near-well-lit-buildings-layMbSJ3YOE) | [Unsplash License](https://unsplash.com/license) |
| `japanese/tokyo-tower-blue-hour.jpg` | 3840x2563 | Marek Okon | [unsplash.com/photos/3-esk8zPwPY](https://unsplash.com/photos/tokyo-tower-at-night-in-tokyo-3-esk8zPwPY) | [Unsplash License](https://unsplash.com/license) |
| `japanese/shinjuku-purple-hour.jpg` | 3840x2561 | Jezael Melgoza | [unsplash.com/photos/2FiXtdnVhjQ](https://unsplash.com/photos/tokyo-skyline-at-purple-hour-2FiXtdnVhjQ) | [Unsplash License](https://unsplash.com/license) |

Quoted text is each page's location or description, which is how every
Shinjuku picture was checked to be Shinjuku. Cinemagraphs are labelled: the
motion is ours, the photo is theirs.

The Unsplash License allows free personal use without attribution; credit is
given here anyway. Kept for personal, non-commercial use.
