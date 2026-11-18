// A real song's signal, a DRAFT: the file decoded once at the analysis's rate and analysed offline
// in a worker (`media-player-analysis.worker.ts`) into what the visualisers read, indexed by the
// song's time. Offline, so a still or a paused frame at any time is the same every time; a visualiser
// knows the next section change and the kicks to come; and the picture follows the audio's own
// clock with no analyser's lag. Until it is ready the song reads as `QUIET`, its visualiser's calm
// look. `docs/design/visualiser.md` specifies it.

import { type Analysis, BANDS, type Kick, type SectionSpan, type Signal, WAVE } from "./media-player-signal.ts";
import { ANALYSIS, HOP, SIZE, type SongData, WAVE_STEP } from "./media-player-analysis.worker.ts";

/** How fast an onset's envelope falls, a share a second, by kind; how long a kick's onset counts as one, s. */
const FALL = { kick: 8, snare: 9, hat: 14 } as const;
const ONSET = 0.05;

/** The index of the last of `sorted` at or before `value`, or -1. */
function lastAtOrBefore(sorted: Float32Array, value: number): number {
  let lo = 0;
  let hi = sorted.length - 1;
  let found = -1;
  while (lo <= hi) {
    const middle = (lo + hi) >> 1;
    if (sorted[middle]! <= value) {
      found = middle;
      lo = middle + 1;
    } else hi = middle - 1;
  }
  return found;
}

/** The signal of a song analysed into `data`, `length` seconds long. */
export function analysed(data: SongData, length: number): Signal {
  const rate = ANALYSIS.rate;
  const frameAt = (time: number) => Math.max(0, Math.min(data.frames - 1, (time * rate - SIZE / 2) / HOP));
  const read = (values: Float32Array, at: number, stride = 1, offset = 0) => {
    const i = Math.floor(at);
    const j = Math.min(data.frames - 1, i + 1);
    const t = at - i;
    return values[i * stride + offset]! * (1 - t) + values[j * stride + offset]! * t;
  };
  const beats = data.beatTimes;
  const section = (time: number): SectionSpan => {
    const index = Math.max(0, lastAtOrBefore(data.sectionStarts, time));
    return { index, start: data.sectionStarts[index]!, end: data.sectionStarts[index + 1] ?? Math.max(length, time + 1) };
  };
  const envelope = (times: Float32Array, strengths: Float32Array, time: number, fall: number) => {
    const k = lastAtOrBefore(times, time);
    return k >= 0 ? { level: strengths[k]! * Math.exp(-(time - times[k]!) * fall), since: time - times[k]! } : { level: 0, since: Infinity };
  };
  const analyse = (time: number): Analysis => {
    const at = frameAt(time);
    const span = section(time);
    const into = Math.max(0, Math.min(1, (time - span.start) / Math.max(1e-6, span.end - span.start)));
    // The beat: where between this beat and the next, its tempo from their distance, its confidence.
    const k = Math.max(0, Math.min(beats.length - 2, lastAtOrBefore(beats, time)));
    const from = beats[k] ?? 0;
    const beat = Math.max(1e-3, (beats[k + 1] ?? from + 0.5) - from);
    const counted = k + (time - from) / beat;
    const confidence = data.beatConfidence[k] ?? 0;
    // The weight eases into the next section's over its last bar: four beats, or the free unit.
    const bar = confidence >= ANALYSIS.trusted ? beat * 4 : ANALYSIS.freeUnit;
    const now = data.sectionWeights[span.index]!;
    const next = data.sectionWeights[span.index + 1] ?? now;
    const ease = Math.max(0, Math.min(1, (time - (span.end - bar)) / bar));
    const buildFrom = data.sectionBuilds[span.index]!;
    const kick = envelope(data.kickTimes, data.kickStrengths, time, FALL.kick);
    const spectrum = new Float32Array(BANDS);
    for (let b = 0; b < BANDS; b++) spectrum[b] = read(data.spectrum, at, BANDS, b);
    // The waveform: a few hundredths of a second around the moment.
    const wave = new Float32Array(WAVE);
    const middle = Math.round((time * rate) / WAVE_STEP);
    for (let i = 0; i < WAVE; i++) wave[i] = Math.max(-1, Math.min(1, ((data.wave[middle + i - WAVE / 2] ?? 0) / 127) * 2.5));
    return {
      time,
      // The tempo over the eight beats around, so a beat landing a frame early never jolts it.
      bpm: (60 * (Math.min(beats.length - 1, k + 4) - Math.max(0, k - 4))) / Math.max(1e-3, beats[Math.min(beats.length - 1, k + 4)]! - beats[Math.max(0, k - 4)]!),
      confidence,
      phase: counted - Math.floor(counted),
      beatIndex: Math.floor(counted) - data.downbeat,
      kick: kick.level,
      onset: kick.since >= 0 && kick.since < ONSET,
      snare: envelope(data.snareTimes, data.snareStrengths, time, FALL.snare).level,
      hat: envelope(data.hatTimes, data.hatStrengths, time, FALL.hat).level,
      sub: read(data.sub, at),
      bass: read(data.bass, at),
      mid: read(data.mid, at),
      high: read(data.high, at),
      level: read(data.level, at),
      bassSlow: read(data.bassSlow, at),
      loud: read(data.loud, at),
      centroid: read(data.centroid, at),
      balance: read(data.balance, at),
      build: buildFrom >= 0 && time >= buildFrom ? Math.pow((time - buildFrom) / Math.max(1e-6, span.end - buildFrom), 2) : 0,
      weight: now + (next - now) * ease * ease,
      section: span.index,
      into,
      spectrum,
      wave,
    };
  };
  const onsets = (times: Float32Array, strengths: Float32Array) => (from: number, to: number): Kick[] => {
    const found: Kick[] = [];
    for (let i = lastAtOrBefore(times, from) + 1; i < times.length && times[i]! <= to; i++) found.push({ time: times[i]!, strength: strengths[i]! });
    return found;
  };
  return { bpm: data.bpm, analyse, section, kicks: onsets(data.kickTimes, data.kickStrengths), snares: onsets(data.snareTimes, data.snareStrengths) };
}

/** A song being analysed: its signal once ready, and the analysis's own figures. */
export interface Song {
  readonly ready: Promise<Signal>;
  /** Its signal, or none until the analysis is done. */
  readonly now: () => Signal | null;
  /** The analysis, once done: its tempo, its sections, its onsets. */
  readonly data: () => SongData | null;
}

const songs = new Map<string, Song>();

/** The song at `url`, `length` seconds long (0 where it isn't known): decoded and analysed once, however often it is asked for. */
export function song(url: string, length: number): Song {
  const known = songs.get(url);
  if (known) return known;
  let signal: Signal | null = null;
  let data: SongData | null = null;
  const ready = (async () => {
    // The browser decodes a file whole before any of it can be read, a long mix to hundreds of
    // megabytes: past `longestDecode` the song plays with its visualiser's calm look instead.
    if (length > ANALYSIS.longestDecode) throw new Error("too long to decode whole");
    const bytes = await (await fetch(url)).arrayBuffer();
    // An offline context decodes at its own rate and plays nothing: no sound, no gesture needed.
    const audio = await new OfflineAudioContext(2, 1, ANALYSIS.rate).decodeAudioData(bytes);
    const worker = new Worker(new URL("./media-player-analysis.worker.ts", import.meta.url), { type: "module" });
    data = await new Promise<SongData>((resolve, reject) => {
      worker.addEventListener("message", (event: MessageEvent<SongData>) => resolve(event.data));
      worker.addEventListener("error", (event) => reject(new Error(event.message)));
      // A minute at a time: the worker keeps the analysis's own numbers, never the whole song.
      const chunk = ANALYSIS.rate * 60;
      for (let at = 0; at < Math.max(1, audio.length); at += chunk) {
        const end = Math.min(audio.length, at + chunk);
        const left = audio.getChannelData(0).slice(at, end);
        const right = (audio.numberOfChannels > 1 ? audio.getChannelData(1) : audio.getChannelData(0)).slice(at, end);
        worker.postMessage({ left, right, rate: ANALYSIS.rate, done: end >= audio.length }, [left.buffer, right.buffer]);
      }
    });
    worker.terminate();
    // A song whose length isn't known yet takes the decoded sound's own.
    signal = analysed(data, length || audio.duration);
    return signal;
  })();
  const made: Song = { ready, now: () => signal, data: () => data };
  songs.set(url, made);
  return made;
}
