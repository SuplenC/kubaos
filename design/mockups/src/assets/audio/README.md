# The media player's songs

The media player's mockup plays two songs that come with the repository, `drum-and-bass-jungle.opus` and `into-the-light.opus` (`CREDITS.md`), and after them any song in `local/` beside this file, a folder git ignores, to try more on one machine. A song's licence often allows playing it but not passing the file on, and a public repository passes on every file in it, so a song goes in `local/` until its licence is known to allow committing it.

## Trying a song

1. Drop the audio file into `design/mockups/src/assets/audio/local/`: Opus, Ogg, MP3, FLAC, WAV, M4A, AAC or WebM.
2. Reload the page. The song is in the playlist after the two that come with it, plays with its real sound, and its visualiser reads the offline analysis of it.

The songs in `local/` go in file-name order. A leading number sets the order and is left out of the title: `01-night-swim.opus` shows as "night swim". To show a proper title and artist, put a JSON file of the same name beside it:

```json
{ "title": "Night Swim", "by": "An Artist", "length": 201.4 }
```

Every field is optional. `length` is in seconds; without it the player reads the file's own as the page loads.

To try a song once, drop it on the player itself instead.

A song is decoded whole before it is analysed, so songs over 30 minutes play with their visualiser's calm look instead (`docs/design/visualiser.md`).
