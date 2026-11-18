// What a music analyser hands the visualisers, as one interface (`Signal`) with two sources: a real
// song analysed offline from its decoded audio (`media-player-song.ts`), and a synthesised one for
// the songs that have no sound, a tempo, a groove and its sections worked out for any moment
// (`synthetic`). Either way `analyse` says, for a moment of the song, what an analyser would give
// at it: a spectrum of `BANDS` bands, a stretch of waveform, the kick's and the snare's onsets and
// envelopes, the hats' rush, the sub, low, middle and high energies, the level, the low and the
// level over seconds, the sound's brightness and balance, the tempo, and where the song is in its
// sections. It is a pure
// function of the song and the time, so a still drawn at a time is the same every time.

/** How many bands the spectrum has, low to high. */
export const BANDS = 32;
/** How many points of waveform it hands over, oldest first. */
export const WAVE = 128;

/** A song's character, as the synthesis plays it. */
export interface Mood {
  /** Beats a minute. */
  readonly bpm: number;
  /** Its own randomness: two songs with different seeds never move alike. */
  readonly seed: number;
  /** Driving (kicks on every beat, builds and drops) or ambient (soft pulses, long swells). */
  readonly character: "drive" | "ambient";
  /** Bars a section lasts. */
  readonly bars: number;
}

/** What a real analyser would say about one moment. */
export interface Analysis {
  readonly time: number;
  /** The song's tempo here, beats a minute, and how far its beat grid can be trusted here, 0 to 1. */
  readonly bpm: number;
  readonly confidence: number;
  /** Where in the beat it is, 0 to 1, and which beat. */
  readonly phase: number;
  readonly beatIndex: number;
  /** The kick's envelope, 1 on its onset falling away; and whether a kick has just struck. */
  readonly kick: number;
  readonly onset: boolean;
  /** The snare's envelope, 1 on its onset falling away quickly. */
  readonly snare: number;
  /** The hats' rush: the top of the spectrum rising, followed quickly. */
  readonly hat: number;
  /** The sub, under 80 Hz, and the low, under 150 Hz: both at once on their attack. */
  readonly sub: number;
  readonly bass: number;
  readonly mid: number;
  readonly high: number;
  /** The whole level, 0 to about 1. */
  readonly level: number;
  /** The low and the level over seconds: what holds, not what hits. */
  readonly bassSlow: number;
  readonly loud: number;
  /** The sound's brightness, 0 dark to 1 bright, and its balance, -1 left to 1 right. */
  readonly centroid: number;
  readonly balance: number;
  /** How far into a build it is, 0 outside one, rising to 1 at the drop. */
  readonly build: number;
  /** The section's weight: 0 in a break, about 0.6 grooving, 1 in a drop. */
  readonly weight: number;
  /** Which section it is in, counting from 0, and how far through it, 0 to 1. */
  readonly section: number;
  readonly into: number;
  readonly spectrum: Float32Array;
  /** The waveform, -1 to 1. */
  readonly wave: Float32Array;
}

/** A section's kind, in the order a song moves through them. */
type Section = "break" | "groove" | "build" | "drop";

/** A number from 0 to 1 for `a` and `b`, the same every time. */
function hash(a: number, b = 0): number {
  const x = Math.sin(a * 127.1 + b * 311.7) * 43758.5453;
  return x - Math.floor(x);
}

/** Smooth value noise in one dimension, 0 to 1. */
function noise(x: number, seed: number): number {
  const i = Math.floor(x);
  const f = x - i;
  const u = f * f * (3 - 2 * f);
  return hash(i, seed) * (1 - u) + hash(i + 1, seed) * u;
}

/** Each section's kind for a song, worked out once from its seed: a build always lands on a drop. */
const plans = new Map<string, Section[]>();

function sectionOf(mood: Mood, index: number): Section {
  const key = `${mood.seed}:${mood.character}`;
  let plan = plans.get(key);
  if (!plan) {
    plan = [];
    plans.set(key, plan);
  }
  while (plan.length <= index) {
    const at = plan.length;
    const before = plan[at - 1];
    let kind: Section;
    if (at === 0) kind = "break";
    else if (before === "build") kind = "drop";
    else if (mood.character === "ambient") kind = hash(at, mood.seed) < 0.55 ? "groove" : hash(at, mood.seed + 1) < 0.5 ? "break" : "build";
    else {
      const roll = hash(at, mood.seed);
      kind = before === "drop" ? (roll < 0.5 ? "break" : "groove") : roll < 0.45 ? "build" : roll < 0.8 ? "groove" : "break";
    }
    plan.push(kind);
  }
  return plan[index]!;
}

const WEIGHT: Record<Section, number> = { break: 0.2, groove: 0.62, build: 0.75, drop: 1 };

/** What the analyser says about `mood`'s song at `time` seconds. */
export function analyse(mood: Mood, time: number): Analysis {
  const seed = mood.seed;
  const beatLength = 60 / mood.bpm;
  const beats = time / beatLength;
  const beatIndex = Math.floor(beats);
  const phase = beats - beatIndex;
  const bar = Math.floor(beatIndex / 4);
  const sectionIndex = Math.floor(bar / mood.bars);
  const section = sectionOf(mood, sectionIndex);
  const into = (beats / 4 - sectionIndex * mood.bars) / mood.bars;
  const next = sectionOf(mood, sectionIndex + 1);
  // The weight eases from one section's into the next over its last bar.
  const ease = Math.max(0, Math.min(1, (into - (1 - 1 / mood.bars)) * mood.bars));
  const weight = WEIGHT[section] + (WEIGHT[next] - WEIGHT[section]) * ease * ease;
  const build = section === "build" ? into * into : 0;
  const drive = mood.character === "drive";

  // The kick: every beat while grooving or dropping, the last bar of a build doubling it; an
  // ambient song's is a soft heart, every other beat.
  const kickOn = section !== "break" || (drive && into > 0.75);
  const doubled = section === "build" && into > 1 - 1 / mood.bars;
  const kickPhase = doubled ? (beats * 2) % 1 : drive ? phase : ((beats / 2) % 1);
  const kick = kickOn ? Math.exp(-kickPhase * (drive ? 7 : 4)) * (drive ? 1 : 0.6) : 0;
  const onset = kickOn && kickPhase < 0.08;

  // The bass: a note an eighth, its pattern the song's own, ducking under the kick.
  const eighth = Math.floor(beats * 2);
  const note = hash(eighth % 16, seed + 3);
  const bassEnv = Math.exp(-((beats * 2) % 1) * 3) * (note > 0.25 ? 1 : 0.3);
  const bass = Math.min(1, bassEnv * (0.35 + weight * 0.7) * (1 - kick * 0.4) + noise(time * 0.3, seed) * 0.15);

  // The middle: a snare on two and four, and pads swelling slowly.
  const snare = drive && kickOn && beatIndex % 2 === 1 ? Math.exp(-phase * 9) : 0;
  const pads = 0.3 + 0.7 * noise(time * 0.12, seed + 5);
  const mid = Math.min(1, snare * 0.8 + pads * (0.25 + weight * 0.4));

  // The highs: hats on the sixteenths, rising through a build; an ambient song's a shimmer.
  const sixteenth = (beats * 4) % 1;
  const hat = Math.exp(-sixteenth * 16) * (hash(Math.floor(beats * 4), seed + 7) > (drive ? 0.15 : 0.6) ? 1 : 0.25);
  const shimmer = noise(time * 3.1, seed + 9);
  const high = Math.min(1, (drive ? hat * (0.25 + weight * 0.5 + build * 0.6) : shimmer * 0.5 + hat * 0.25) + build * 0.35);

  const level = Math.min(1, kick * 0.35 + bass * 0.3 + mid * 0.2 + high * 0.15 + weight * 0.15);

  const spectrum = new Float32Array(BANDS);
  for (let band = 0; band < BANDS; band++) {
    const at = band / (BANDS - 1);
    const low = Math.exp(-at * 18) * (kick * 1.1 + bass * 0.6);
    const body = Math.exp(-Math.pow((at - 0.18) / 0.1, 2)) * bass * 0.7;
    const middle = Math.exp(-Math.pow((at - 0.45) / 0.16, 2)) * mid;
    const top = Math.exp(-Math.pow((at - 0.82) / 0.18, 2)) * high;
    const grain = 0.75 + 0.5 * noise(time * 5 + band * 1.7, seed + band);
    spectrum[band] = Math.min(1, (low + body + middle + top) * grain);
  }

  const wave = new Float32Array(WAVE);
  const bassHz = 1.5 + Math.floor(note * 4);
  for (let i = 0; i < WAVE; i++) {
    const x = i / WAVE;
    const value =
      Math.sin((x * bassHz + time * 0.9) * Math.PI * 2) * (bass * 0.7 + kick * 0.5) +
      Math.sin((x * 7 + time * 2.3) * Math.PI * 2) * mid * 0.35 +
      (noise(x * 40 + time * 30, seed + 11) - 0.5) * high * 0.6;
    wave[i] = Math.max(-1, Math.min(1, value));
  }

  const sub = bass * (0.6 + 0.4 * weight);
  const bassSlow = (0.35 + weight * 0.6) * 0.8 + noise(time * 0.3, seed) * 0.1;
  const loud = 0.2 + 0.6 * weight + 0.2 * build;
  const centroid = Math.max(0, Math.min(1, 0.35 + high * 0.4 - bass * 0.2 + noise(time * 0.07, seed + 13) * 0.3));
  const balance = (noise(time * 0.2, seed + 15) - 0.5) * 0.4;
  return { time, bpm: mood.bpm, confidence: 1, phase, beatIndex, kick, onset, snare, hat: hat * (drive ? 1 : 0.4), sub, bass, mid, high, level, bassSlow, loud, centroid, balance, build, weight, section: sectionIndex, into, spectrum, wave };
}

/** The sample songs' characters. */
export const MOODS = {
  drive: { bpm: 118, seed: 7, character: "drive", bars: 8 },
  ambient: { bpm: 72, seed: 31, character: "ambient", bars: 12 },
} as const satisfies Record<string, Mood>;

/** A kick: when its onset is, in the song's seconds, and how hard it hit, about 0 to 1.4. */
export interface Kick {
  readonly time: number;
  readonly strength: number;
}

/** A section of a song: its index, from 0, and when it starts and ends, in the song's seconds. */
export interface SectionSpan {
  readonly index: number;
  readonly start: number;
  readonly end: number;
}

/** A song's music, as the visualisers read it: what it is at a moment, its sections, its kicks. */
export interface Signal {
  /** Its tempo, beats a minute. */
  readonly bpm: number;
  analyse(time: number): Analysis;
  /** The section `time` is in. */
  section(time: number): SectionSpan;
  /** The kicks whose onsets fall after `from` and up to `to`, oldest first; and the snares'. */
  kicks(from: number, to: number): readonly Kick[];
  snares(from: number, to: number): readonly Kick[];
}

const synthetics = new Map<Mood, Signal>();

/** A synthesised song's signal: `mood`'s, worked out from its tempo and its seed. */
export function synthetic(mood: Mood): Signal {
  const known = synthetics.get(mood);
  if (known) return known;
  const sectionLength = (mood.bars * 4 * 60) / mood.bpm;
  const half = 30 / mood.bpm;
  const made: Signal = {
    bpm: mood.bpm,
    analyse: (time) => analyse(mood, time),
    section: (time) => {
      const index = Math.max(0, Math.floor(time / sectionLength));
      return { index, start: index * sectionLength, end: (index + 1) * sectionLength };
    },
    kicks: (from, to) => {
      // Its kicks fall on the half beats where it has one.
      const found: Kick[] = [];
      for (let k = Math.ceil(from / half); k * half <= to; k++) {
        const at = k * half;
        if (at <= from || at < 0) continue;
        const then = analyse(mood, at + 0.004);
        if (then.onset) found.push({ time: at, strength: Math.min(1.4, then.kick * (0.55 + 0.6 * then.weight + 0.4 * then.build)) });
      }
      return found;
    },
    snares: (from, to) => {
      // Its snares fall on two and four where it drives.
      const beat = 60 / mood.bpm;
      const found: Kick[] = [];
      for (let k = Math.ceil(from / beat); k * beat <= to; k++) {
        const at = k * beat;
        if (at <= from || at < 0) continue;
        const then = analyse(mood, at + 0.004);
        if (then.snare > 0.9) found.push({ time: at, strength: 0.6 + 0.4 * then.weight });
      }
      return found;
    },
  };
  synthetics.set(mood, made);
  return made;
}

/** The signal of a song not yet analysed: still and calm, so its visualiser shows its paused look. */
export const QUIET: Signal = {
  bpm: 120,
  analyse: (time) => ({ time, bpm: 120, confidence: 0, phase: 0, beatIndex: 0, kick: 0, onset: false, snare: 0, hat: 0, sub: 0, bass: 0, mid: 0, high: 0, level: 0, bassSlow: 0, loud: 0, centroid: 0.5, balance: 0, build: 0, weight: 0.2, section: 0, into: 0.5, spectrum: new Float32Array(BANDS), wave: new Float32Array(WAVE) }),
  section: () => ({ index: 0, start: 0, end: 1e9 }),
  kicks: () => [],
  snares: () => [],
};
