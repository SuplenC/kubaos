# The media player's film

The media player's mockup plays "Sintel", a short film by the Blender Foundation, so the player has a real video with sound and subtitles to play, seek and preview.

"Sintel": © copyright Blender Foundation | [durian.blender.org](https://durian.blender.org), licensed under [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/). Source: [durian.blender.org](https://durian.blender.org), downloaded from <https://download.blender.org/durian/movies/Sintel.2010.720p.mkv.zip>.

| File | What it is | Where it lives |
|---|---|---|
| `sintel-poster.jpg` | The film's frame at 3:10.71 (the cut's 1:42.17), saved as a JPEG | Committed here |
| `sintel.json` | What the cut takes: its frames, chapters and tracks' names, and a line of each subtitle track | Committed here |
| `sintel.webm`, `sintel.LANG.vtt` | The cut and its subtitles, made by `just mockup-film` | Outside the repository, in `~/.cache/kubaos-mockups/film/` |

Changes made: the poster is one frame of the film, unchanged. `just mockup-film` (`design/mockups/scripts/film.sh`) cuts the film from 1:28.54 to 6:20.54 (four minutes 52 seconds), re-encodes the picture to AV1 in WebM at its own 1280 × 544, mixes the 5.1 sound down to stereo Opus, and converts the ten subtitle tracks from SubRip to WebVTT, re-timed to the cut. The cut is never committed or published.

The player shows the film's title and its maker wherever the film shows.
