# The media player's visualisers

A song playing in the media player can show a visualiser behind its controller: Water, Silk, Tube or Embers. This page says how one is calculated, from the song's samples to the pixels, so it works the same with every song: a quiet old master and a loud new one, a drum and bass track and a solo piano, a podcast, a drone, an hour-long mix. The mockup (`design/mockups/src/pages/apps/media-player-*.ts`) is the reference implementation; its constants are the ones named here, from one table in the code each:

| Table | File | Holds |
|---|---|---|
| `ANALYSIS` | `media-player-analysis.worker.ts` | Every constant of the analysis |
| `FOLLOW` | `media-player-analysis.worker.ts` | Every feature's attack and release |
| `FALL` | `media-player-song.ts` | How fast an onset's envelope falls |
| Water's tunables | `media-player-viz-water.ts` | Its mappings, curves and limits |

## The pipeline

1. **Decode** the file and resample it to 22 050 Hz (`rate`), two channels. Nothing above 10.5 kHz is read, so the rate is the least that keeps the hats.
2. **Frame** it: windows of 1024 samples (`size`, 46.4 ms), a Hann window each, a step of 512 samples (`hop`, 23.2 ms): 43.07 frames a second. Each frame's DC offset (the mean of both channels) is taken out first.
3. **Spectrum:** an FFT of the frame's mid ((left + right) / 2), each bin's power scaled so a full-scale sine reads 0 dBFS.
4. **Bands:** the power averaged over each band's bins, in dB (the bands are fixed in Hz, below).
5. **Loudness:** the K-weighted power of the frame (ITU-R BS.1770's shelf and high-pass, applied to the bins as magnitudes), in LUFS.
6. **Onset fluxes:** each onset band's rise in dB since the last frame, and the share of its bins that rose.
7. **Normalisation** (once the whole song is in): every level spread over the song's own range.
8. **Smoothing:** each feature followed as a meter, with its own attack and release.
9. **Onsets:** each band's flux against its own neighbourhood; kicks, snares and hats told apart by where the flux is.
10. **Tempo and beat grid:** a grid fitted to the drum onsets window by window, tracked through the song, with a confidence.
11. **Structure:** the song's sections, from where it stops resembling itself; each one's weight; its build and its drop.
12. **The signal:** for any moment of the song, every feature read off at that moment and handed to the visualiser each frame, at the audio clock's time.

The analysis runs as a stream (`Analyser.push`): it keeps each frame's few numbers, never the samples, so its memory grows with the song's length in frames. Steps 7 to 11 need the whole song and run once it is in (`Analyser.finish`).

## The bands

| Band | Hz | Used for |
|---|---|---|
| Sub | 30–60 | `sub`: the held glow and Water's swell |
| Low | 30–150 | `bass`, `bassSlow`; the kick's onsets |
| Middle | 250–2000 | `mid` |
| Top | 4000–10 500 | `high` |
| Spectrum | 40–10 500 in 32 bands, spaced evenly in pitch | `spectrum` |
| Snare | 150–700 and 1500–5000 | The snare's onsets: its body and its crack |
| Hats | 6000–10 500 | The hats' onsets |

## Normalisation: a song against itself

Every level is in decibels and is read against the song's own range, so the same song at −30 dB and at 0 dB gives the same picture, and a quiet master uses the whole picture as a loud one does.

- **A level's range.** Over the song's sounding frames, its quiet end is the 10th percentile (`quietShare`) and its loud end the 97th (`loudShare`); the value is (level − quiet end) / (loud end − quiet end), 0 to 1.2.
- **The least range.** The range is never under 12 dB (`leastRangeDb`), widened downward from the loud end: a steady drone in a band reads full, and a band that only flickers by a dB or two never has its flicker blown up to the whole range.
- **A band against the song.** A band's loud end is never under the song's loudest band's less 40 dB (`spanDb`): a band holding only hiss or a filter's skirt sits far under the music and stays dark, however steady it is; a band-light song still lights its bass with what low end it has, because each band is read against its own history.
- **Floors.** Under −140 dBFS a bin counts as nothing at all (`floorDb`): digital silence. Above it, what counts is relative: a bin more than 80 dB (`depthDb`) under the frame's loudest bin, or a band more than 80 dB under the song's loudest, counts as silent, so the floors move with the song's level.
- **Silence.** A frame whose loudness is under −70 LUFS (`gateLufs`, BS.1770's absolute gate) is silent: every band reads 0, and no onset is found in it.
- **Why the whole song, offline.** A stretch is judged against the song, so a break stays dark against its drop. A running follower would brighten a long break to the drop's level; the live mode, which has no song to look at, uses one (below).

## The features

Each is a number a frame, 0 to about 1 unless it says; `time` is the song's clock in seconds.

| Feature | Definition | Range | Attack / release (`FOLLOW`) | With no signal |
|---|---|---|---|---|
| `level` | Momentary loudness: the K-weighted power over 400 ms, in LUFS, normalised | 0–1 | 30 ms / 250 ms | 0 |
| `loud` | Short-term loudness: the same over 3 s, normalised | 0–1 | 500 ms / 1.5 s | 0 |
| `sub` | The sub band's level, normalised | 0–1.2 | 4 ms / 160 ms | 0 |
| `bass` | The low band's level, normalised | 0–1.2 | 4 ms / 160 ms | 0 |
| `bassSlow` | `bass` followed slowly: what holds, not what hits | 0–1.2 | 300 ms / 900 ms | 0 |
| `mid` | The middle band's level, normalised | 0–1.2 | 30 ms / 250 ms | 0 |
| `high` | The top band's level, normalised | 0–1.2 | 30 ms / 250 ms | 0 |
| `spectrum` | Each of the 32 bands' levels, each normalised on its own history | 0–1.2 each | 60 ms / 300 ms | 0 |
| `centroid` | The sound's brightness: where the spectrum's weight sits in pitch, each band weighed by how far it stands over 50 dB (`brightDepthDb`) under the frame's loudest band; 0 at 40 Hz, 1 at 10.5 kHz | 0–1 | 150 ms / 400 ms | Holds the last sound's; 0.5 if none |
| `balance` | (right − left) / (right + left) of the channels' RMS | −1 to 1 | 200 ms / 500 ms | 0 |
| `kick`, `snare`, `hat` | Each kind's last onset: its strength × e^(−fall × seconds since), the fall 8, 9 and 14 a second (`FALL`) | 0–1.4 | Instant on the onset | 0 |
| `onset` | Whether a kick struck in the last 50 ms | true / false | — | false |
| `bpm` | The local tempo: over the eight beats around the moment | BPM | — | 120 |
| `confidence` | How far the beat grid can be trusted here | 0–1 | — | 0 |
| `phase`, `beatIndex` | Where between this beat and the next, and which beat from the downbeat | 0–1, integer | — | 0 |
| `section`, `into` | Which section, from 0, and how far through it | integer, 0–1 | — | 0, 0.5 |
| `weight` | The section's weight, eased into the next one's over the last bar | 0.2–1 | Over a bar | 0.2 |
| `build` | How far into the section's build: ((time − build start) / (section end − build start))² | 0–1 | — | 0 |
| `wave` | 128 points of the waveform around the moment, every fourth sample, × 2.5 | −1 to 1 | — | 0 |

The sound's brightness weighs bands by their presence, not their power: power alone always sits in the bass, and would read every song as dark.

## Onsets

A kick, a snare or a hat is an onset in its band, found against that band's own neighbourhood, so a dense mix and a sparse one find theirs alike.

- **Flux.** Each frame, a band's flux is the mean over its bins of the rise in dB since the last frame, each bin held at the frame's floor (`depthDb` under its loudest bin), and the share of its bins that rose by 3 dB or more (`binRiseDb`).
- **Adaptive threshold.** Each band's flux is scored as (flux − median) / spread over the 1 s either side (`onsetWindow`), the spread being the median absolute deviation × 1.4826 and never under 0.5 dB (half `leastRiseDb`). An onset is a peak of the score over the two frames either side, at least 2.5 spreads (`onsetSpreads`) over the median, with a mean rise of at least 1 dB (`leastRiseDb`), across at least 40 % of the band's bins (`leastSpread`): a tone gliding from bin to bin is no drum.
- **Classification.**
  - A **kick**: the low band peaks, and its score is at least 0.8 (`kickOver`) of the snare bands' (a kick's beater clicks in them too). Where the low band has been noise over the second around (its flatness, the geometric mean of its bins' power over their mean, over 0.55, `noiseFlatness`) or a pure tone (under 0.05, `toneFlatness`), a kick must also lift at least 60 % of the band's bins (`strictSpread`) by 9 dB or more (`strictRiseDb`) at once: noise's random peaks and a gliding tone's entry into the next bins are no kicks, while a song's low end (tonal bass notes under a kit, its flatness 0.1 to 0.5) keeps the plain test.
  - A **snare**: the snare bands peak, their score over the low's (`snareOver`, 1), and the top band's score at least 1 (the crack).
  - A **hat**: the top band peaks while neither the low nor the snare bands do.
- **Refractory time.** Two onsets of a kind are at least 90 ms (kicks), 120 ms (snares) or 45 ms (hats) apart (`kickGap`, `snareGap`, `hatGap`).
- **Strength.** The onset's score over the 90th percentile of its kind's scores in the song, at most 1.4: the song's strong hits read about 1.
- **No drums.** Music without drums yields few onsets or none; every visualiser stays alive from the envelopes alone.

## Tempo and the beat grid

- **The fit.** The tempo is read from the drums (kicks and snares; every onset, hats too, where there are under one a second). In windows of 8 s every 2 s (`tempoWindow`, `tempoStep`), each tempo from 55 to 200 BPM (`slowest`, `fastest`) in steps of 0.5 is scored by how well a grid of beats at that tempo, at its best phase, fits the onsets: the F-measure of the onsets that land within 35 ms (`onBeat`) of a beat and the beats that have an onset. A grid twice as fast catches the same onsets but leaves half its beats empty; a grid half as fast misses half the onsets; so neither a multiple nor a fraction of the tempo wins by being denser or sparser. This is what settles half and double time (drum and bass at 87 or 174) and the two-thirds and three-quarters errors an autocorrelation makes.
- **The prior.** Each fit is weighed by a prior over tempos, a Gaussian in octaves round 120 BPM (`likeliest`) 1.5 octaves wide (`priorOctaves`): it only breaks near ties.
- **Tracking.** A window with under 0.5 onsets a second (`leastOnsets`) says nothing. A path through the windows keeps to the fits with the fewest changes: each window's say is its fit against its own best, sharpened 4 times (`tempoSharpness`), and a change costs 3 per octave (`tempoInertia`), so an octave jump takes three windows' clear word, while a real change of tempo is followed within a window or two.
- **Beats.** The beats are the path through the frames that lands on onsets while keeping each step within 0.8 to 1.25 of the local period, a step off the period costing 100 × ln(step / period)² (`beatTightness`; Ellis's dynamic programme). Before the first beat found and after the last, the grid goes on at the period.
- **Downbeat.** Of a bar's four beats, the downbeat is the one the kicks fall on most.
- **The tempo.** The song's tempo is the median of each beat's tempo over the eight beats around it.
- **Confidence.** A window's confidence is how far its best fit stands over the best fit of a tempo unrelated to it (not within a ratio of e^0.04, `related`, of the best tempo or its half, double, two thirds, three halves, three quarters or four thirds), as a share of the best, over 0.3 (`confidenceSpan`), 0 to 1. Onsets at random fit some tempo nearly as well as any other; a beat fits its own family alone. Each beat carries its window's confidence; the song's is the median.
- **Low confidence** (free tempo, rubato, ambient, speech). From 0.5 (`trusted`) on the grid is trusted. Under it, sections are not snapped to bars, and a visualiser's beat-locked mappings fade to envelope-driven ones (Water's, below). The grid is never locked onto by force.

## Structure

- **Units.** Bars where the grid is trusted, 2 s units (`freeUnit`) where it is not.
- **Features.** Each unit's loudness (dB), sub, low, middle and top levels, brightness, kicks, snares and hats a second, and its spectrum in 8 coarse bands, each standardised over the song against its spread, never under a least spread (3 dB, 0.1 for the levels, 0.05 for the brightness, 0.5 onsets a second), so a feature that barely moves is not blown into a difference.
- **Self-similarity.** Two units are as alike as e^(−d / 2), d their distance in the standardised features.
- **Novelty.** A Gaussian-tapered checkerboard kernel 8 units each side (`kernel`) along the self-similarity's diagonal: likeness within each side less likeness across. Only where the kernel has two units at least either side.
- **Changes.** The novelty's peaks over its mean + 0.75 of its spread (`noveltySpreads`), and over 0.12 (`leastNovelty`), at least 8 units apart (`leastSection`), never in the first or last 4 units.
- **Weight.** A section's loudness (its units' power, averaged) against the loudest section's: 0.2 + 0.8 × (1 − (loudest − its) / 9 dB) (`breakDb`), clamped: a section 9 dB under the loudest is a break. A silent section weighs 0.2.
- **Build.** Before a change into a section at least 0.2 heavier: the run of units, back from the change, at most 16 (`longestBuild`), over which the sound brightens (brightness up by 0.05, `buildRise`) or swells (loudness up by 1.5 dB, `buildRiseDb`) while its low end holds or falls; at least two units, or none.
- **Drop.** A section whose low end over its first four units stands 0.2 (`dropRise`) over the four before it.

## The signal a visualiser receives

Each frame the visualiser is handed the analysis at the song's current time (`Analysis`, in `media-player-signal.ts`): every feature in the table above, plus the song's own: its sections (`section(time)`: index, start and end), and its onsets in a span (`kicks(from, to)`, `snares(from, to)`). Each value is the analysis interpolated between the two frames around the moment.

**The time sync rule.** The picture reads the analysis at the audio clock's current time, never at a clock of its own: in the mockup, the audio element's `currentTime`; in the build, mpv's playback position less the audio output's latency as the output reports it, so the picture shows the moment being heard. The offline analysis knows every onset's exact time, so an onset shows in the frame its sound is heard in. Paused, the clock stands: the picture holds the paused moment, drifting slowly, its light dimmed with the song's `energy`, and comes back over `long` as it plays.

**Nothing reads ahead.** A visualiser reads the analysis at the moment and before it: the features of this instant and the recent past, the onsets already heard, the section playing. None reads a later moment, so what a file, a stream and live sound show differs only in the analysis behind it (below), never in how the picture follows it.

A song not yet analysed reads as a calm, still signal (`QUIET`: every level 0, weight 0.2, confidence 0): the visualiser shows its paused look until the analysis is done, which runs in the background.

## Offline and live

Offline is the mode the mockup draws: a file, decoded and analysed once, every value known for every moment.

| | Offline (a file) | Live (a stream, system audio) |
|---|---|---|
| Levels, bands, spectrum | Exact at the moment | Causal: a frame (23 ms) late, plus the attack; normalised against a running range over the last 30 s instead of the song's |
| Loudness | Centred windows | Trailing windows: the momentary 200 ms late, the short-term 1.5 s late |
| Onsets | Exact, from a threshold over the second either side | The threshold over the past second alone; decided two frames after the peak, 46 ms late |
| Water's rings | Born on the onset's own frame | Born 46 ms late |
| Tempo and grid | From the whole song | From the last 8 s: no grid for the first 8 s, confidence rising as it locks |
| Sections | Exact: a drop's flash lands on the change | Known half a kernel (8 bars) after: a drop's flash lands once the change is confirmed |
| Builds and drops | Known | Unknown ahead; a drop is seen as the low end returns |
| Cost | Analysis about 0.005 × the song's length on one core in a worker (a 215 s song in about 1 s, an hour in about 15 s); about 130 MB of features an hour | One 1024-point FFT every 23 ms, well under 1 % of a core |

The mockup decodes a file whole before analysing it, as the browser can only decode a file whole: it analyses songs up to 30 minutes (`longestDecode`), and plays a longer one with its visualiser's calm look. The build decodes in chunks, so its offline mode has no such limit: the analysis itself is streamed and bounded.

## Water

Travelling down a tunnel of light under water. Its wall is bundles of strings of light; the tunnel bends in long, slow curves you travel through and rolls slowly all the time; specks of light hang in the water and stream past; the water behind is deep and dark, darkest down the tunnel, a little lighter far out, with a faint slow haze. The far end fades to dark.

**Words.** A string is one line of light; a bundle is a group of strings moving together; the undulation is the slow sideways S-shape running along a bundle; a ring is the swelling a bass hit sends along every string; the vibration is the fast shake on bass hits.

**Three families.** Each listens to its own band and sits at its own radius of the wall, so where they overlap one passes behind another; all turn the same way:

| Family | Listens to | Bundles × strings | Girth | Radius | Spacing, rad | Shade |
|---|---|---|---|---|---|---|
| Deep | `bass` | 6 × 7 | 1.55 | 1.14 | 0.052 | Deeper |
| Main | `mid` | 12 × 8 | 1.0 | 1.0 | 0.04 | The colour |
| Fine | `high` | 16 × 4 | 0.5 | 0.88 | 0.03 | Paler |

(`FAMILIES`, with each family's undulation height, wavelength and speed, ring and light: 0.9 deep, 1 main, 0.6 fine.) In a bundle the leading string is at one edge: the fattest and brightest, with a white-hot core 0.42 of its girth wide (`CORE_LEAD`); the followers trail to one side, the kᵗʰ 0.62 × 0.9ᵏ of the leader's girth and 0.78 × 0.9ᵏ of its light (`FOLLOWER_GIRTH`, `FOLLOWER_LIGHT`, `FOLLOWER_FADE`), lagging 0.045 rad each (`LAG`), so they read as separate but never cross.

**The bands.** Each family hears its band as it is at the moment: the deep the bass, felt from 0.25 of its range up to full at 0.95 (`BASS_FLOOR`, `BASS_RANGE`), and never under the kick's hit, so it is full on the kick's own frame; the main the mids; the fine the highs. The three share the picture (`shared`): the band that leads keeps its level, each other steps back by how far it trails it (its level squared over the leader's), and together they never pass 1.618 (`BAND_ROOM`), so a full passage is led by one family and never every string at its most.

**Girth.** A string's girth is 0.019 units at depth one (`GIRTH`) × its family's girth × its place in the bundle × its swell, and never under 0.55 px (`LEAST_PX`). The swell is its band, a ring and the hit multiplied: (0.45 + 0.6 band) × (1 + 1.9 × the family's ring × the ring) × (1 + 0.6 hit) (`GIRTH_BAND`, `RING_SWELL`, `HIT_GIRTH`). Up to 1.2 it is as it is; past that it bends toward the family's most, 1.8 deep, 1.5 main, 1.3 fine (`SWELL_KNEE`, `SWELL_MOST`), so a kick fattens the strings without blotting them out.

**Light.** A string's light is the family's light × its place in the bundle × (0.18 + 0.12 energy + 0.24 band + 0.18 hit × the family's step), and the deep family adds 0.15 hit (`LIT`, `HIT_LIGHT`); the steps go down by the golden ratio, 1 deep, 0.618 main, 0.382 fine (`STEP`). Where a ring is it is × (1 + 0.45 × the ring's light) (`RING_LIGHT`). Up to the family's knee, 0.8 deep, 0.5 main, 0.3 fine, it is as it is; past it it bends toward the family's most, 1.3, 0.8, 0.5 (`LIGHT_KNEE`, `LIGHT_MOST`), so a kick lands bright at once without burning the strings to white, and at the loudest the lows still read brighter than the mids and the mids than the highs. Then a drop's flash and hold multiply it (below), and the depth thins it: soft right beside you, full from about 1.4 units ahead, fading out from 9 units on, so the far end stays dark.

**The undulation.** Runs along each bundle on its own, toward you, faster than you travel, in two wavelengths of different speeds (the second 2.75 times the first's), so the shape never repeats. Its height grows with the loudness and the family's band, plus a kick on each bass hit (0.05 + 0.15 loudness + 0.2 band + 0.06 hit rad, `SWING`). Where a bundle bends is its own: its phase is part random, part winding round the tunnel, and its wavelength differs by up to 12 % (`k`), so bends never line up into circles. Strings are never straight and never still, paused included.

**The vibration.** Only an answer to sound: a bass hit shakes every string at 13 Hz (`SHAKE_HZ`), 0.015 rad at full (`SHAKE`), and the highs add a finer shake (0.005); a bundle shakes as one. At silence and paused there is none: it eases out with the song's energy as the music stops.

**The rings.** A bass ring is born the instant its kick sounds, 11 units down the tunnel where it can be seen (`RING_FROM`), at full strength at once, and rushes in past you at 17 units a second (`RING_SPEED`), about two thirds of a second. As it passes it swells the strings' girth (Girth, above; the family's ring is 1.25 deep, 1 main, 0.7 fine) and pulls them in by 0.17 (`RING_PULL`). A kick rings by the bass of its passage, the most of it over the 0.4 s up to the kick (`RING_BASS`, `PASSAGE`), so a light kick barely rings. Its birth is the kick seen: on the kick's own frame the ring's light is 1 + 5 times its strength, over 3 units more of the tunnel's depth than the ring itself (`RING_BIRTH`, `RING_BIRTH_WIDTH`), both falling over 0.1 s (`RING_BIRTH_FALL`) as it travels in (Light, above); its arrival is the aftermath.

**The hit.** A kick answers across the whole tunnel on its own frame, at full strength with no rise: every string's girth and light (Girth and Light, above), the shake and the swing, falling over 0.15 s (`HIT_FALL`). The hit is the kick's strength from its stamp, under the bass band itself, which rises over a frame or so after the attack.

**The flight.** The travel follows the energy and the tempo: (2.2 + 4.4 × energy) units a second (`TRAVEL`) at 175 BPM, scaled by the square root of the tempo against it where the beat grid is trusted. The bends deepen with the energy (`BEND`); the roll is 0.045 rad a second (`ROLL`). Nothing steers to the beat.

**The motes.** 110 tiny specks (`MOTES`) hanging in the water over 30 units (`MOTE_SPAN`), streaming past, glinting with the highs. Each is a soft round dot with a glow; only when you travel fast (from 2.5 to 6 units a second, `TRAIL_FROM`, `TRAIL_FULL`) does it draw out a soft trail behind it, at most three of its radii (`TRAIL_MOST`), narrowing and fading to nothing at its tail.

**The drop.** A section at least 0.85 heavy and 0.05 heavier than the one before (`DROP_HEAVY`, `DROP_STEP`), each weight read at its section's start. It lands on its section's first kick within 0.06 s of the start (`DROP_ALIGN`), the moment it is heard, as a flash: the strings' light 2.6 times on that frame, with no rise, settling over 0.9 s (`FLASH_FALL`, `FLASH_LIGHT`). Until the section ends the wave stays bright and big: light × 1.35, undulation × 1.3 (`HOLD_LIGHT`, `HOLD_HEIGHT`). The flash and the hold multiply the light past its ceiling, so the drop is the brightest moment of a song.

**Colour.** The theme's colours only, as live values that ease over a third of a second when the theme changes, mid-song too. The cold one leads; one bundle in four (`SECOND_EVERY`) takes the primary. Both are worn 0.35 of the way to white (`PALE`), as light rather than paint; the deep family in deeper shades of them, the fine in paler. The water behind is the cold colour at a few hundredths. White only in the strings' cores.

**Glow and white.** Each string is a crisp core one girth wide with a soft glow round it and the engine's bloom on the frame. The frame is exposed at 0.85 and brought into range by its brightest channel (`exposure`, `tone: "hue"`), so strings piling up near the middle in a loud passage stay their colour instead of burning to white.

**The signal.** Water reads the analysis at the moment and the onsets already heard, nothing else: the bands as they are at the moment, which the analysis's followers raise with the sound at once and let fall smoothly; the loudness over the last second and the energy over the last four, each weighted less the further back; the last eight kicks of the last 1.2 s. Nothing reads ahead, so a file, a stream and live sound draw alike; offline, the analysis behind the features is exact (see Offline and live).

**States.** Paused or silent: thin, dim and slowly wavy, never straight, never still, perfectly smooth. Reduced motion: calm, travelling at 0.6 units a second (`CALM_TRAVEL`), with no vibration, rings or flash.

**Cost.** One draw: a quad of background, 202 strings of 160 segments each (`SEGMENTS`, evenly in log-depth from 0.3 to 34 units, `NEAR`, `FAR`), and the motes; each string's quad is shaded by its own distance from the middle line, so wide strings stay smooth. About 4 ms a frame at 1920 × 1080 on the test machine's GPU, drawn and finished (median 3.7 ms, 95th percentile 6.1 ms), the per-frame analysis reads included: well inside the 16.7 ms of 60 Hz.

## The others

They read the same signal through the engine's uniforms, each the feature × the song's `energy`.

| Visualiser | Music → picture |
|---|---|
| Silk | `bass` widens the sheets; `high` thickens the threads; the spectrum lights each sheet; `kick` brightens the front sheets and pushes the trails out; `level` swells the waves; `weight` spreads the sheets apart |
| Tube | `bass` brightens the tube's glow; `level` bends it; `mid` ripples it; `high` frays its edge and lights its threads; `kick` lights it and pushes the trails |
| Embers | `level` swells the arms and lifts the embers; `build` draws them into a shell; `kick` throws them out and brightens them; the spectrum lights each ember |

## Flashing and reduced motion

**Flashing.** A visualiser answers the music as hard as it is drawn to: a bass ring, a hit and a drop light the frame as they come, the whole frame at once if the music does. Who would rather not see light flash turns the visualiser off: Off, in its list, leaves the song's cover and words.

**Reduced motion.** With reduce motion on, Water stays calm, travelling slowly, with no vibration, rings or flash; the others give way to the song's cover and words in the middle, as with Off.

## Guarantees and their checks

The same set runs on the build: each a file generated with ffmpeg (`lavfi`) or a song, analysed, and held to its expected result.

| Case | Expected |
|---|---|
| Digital silence, 30 s | Every feature 0; no onsets; confidence 0; one section, weight 0.2: dark, calm water |
| Noise at −90 dBFS | As silence: under the loudness gate |
| A 60 Hz sine | `bass` and `level` full and steady; `mid`, `high` 0; no onsets; confidence 0 |
| A 20 Hz–20 kHz sweep, 30 s | The spectrum's loudest band walks from the first to the last; few kicks (under one in four seconds) |
| Pink noise | Every band near full; confidence under 0.3; few kicks (under one in four seconds) |
| Clicks at 60, 120, 174 BPM | That tempo within 1 %; confidence 1; a kick on every click |
| Clicks at 100 then 140 BPM | Each half's tempo; a section at the change |
| The same song at −30 dB and 0 dB | The same features (mean difference under 0.001), onsets, tempo and sections |
| A clipped copy (+12 dB) | The same tempo and sections, near the same onsets |
| A DC offset, a mono file, a 192 kHz file | As the original |
| One channel only | As the original; `balance` near −1 or 1 |
| A 6 s file | A tempo and onsets; one section |
| Fade-in and fade-out | The fades read as rising and falling levels; the faded end its own quiet section |
| Silence before and after | The silent stretches their own sections, weight 0.2 |
| A drone | Full steady levels; no onsets; confidence 0 |
| Speech | Low confidence: no grid; its syllables' attacks are onsets |
| Solo piano, rubato | Notes as onsets; confidence low where the timing is free |
| An hour-long mix | Analysed in a bounded time and memory, its sections throughout |
