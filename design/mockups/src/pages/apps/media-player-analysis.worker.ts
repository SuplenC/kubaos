// The media player's song analysis, a DRAFT, in a worker so the page never hitches: a song's two
// channels in at `ANALYSIS.rate`, and out, frame by frame, what the visualisers read. It is the
// offline mode of the analysis `docs/design/visualiser.md` specifies; every constant it uses is in
// `ANALYSIS` and `FOLLOW` below, named as the document names them.
//
// Nothing in it is tuned to a song or a genre. Every level is measured in decibels and read against
// the song's own range, so a quiet old master and a loud new one fill the picture alike; every band
// is read against its own history, with a floor in absolute level under which it stays dark; onsets
// are found against their own neighbourhood's spread; the tempo is tracked with a confidence that
// says how far the beat grid can be trusted; sections are where the song stops resembling itself.
//
// It runs as a stream: `Analyser.push` takes the samples a chunk at a time and keeps only each
// frame's few numbers, so memory grows with the song's length in frames, never in samples, and
// `finish` works out what needs the whole song (its range, its onsets, its grid, its sections).

import { BANDS } from "./media-player-signal.ts";

/** Every constant of the analysis, in one place: `docs/design/visualiser.md` lists them all. */
export const ANALYSIS = {
  /** The rate the song is analysed at, Hz: nothing above 10.5 kHz is read. */
  rate: 22050,
  /** The longest song the mockup decodes whole in the browser, s: 30 minutes is 160 MB of samples at the rate. */
  longestDecode: 1800,
  /** A frame's window and its step, samples: 46.4 ms and 23.2 ms at the rate. */
  size: 1024,
  hop: 512,
  /** Under this a bin's power counts as nothing at all, dBFS (a full-scale sine is 0): digital silence. Above it, what counts is relative to the song (`depthDb`). */
  floorDb: -140,
  /** Under this the loudness counts as silence, LUFS: BS.1770's absolute gate. */
  gateLufs: -70,
  /** How far under a frame's loudest bin a bin still counts for the onsets, and under the song's loudest band a band still counts as sounding, dB. */
  depthDb: 80,
  /** The least range a feature is spread over, dB: a steady drone reads full, steady hiss stays dim. */
  leastRangeDb: 12,
  /** How far under the song's loudest band a band's top may sit before it counts as quiet, dB. */
  spanDb: 40,
  /** The percentiles a feature's quiet and loud ends are read at, over the song's sounding frames. */
  quietShare: 0.1,
  loudShare: 0.97,
  /** The bands, Hz: the sub, the low, the middle, the top; the spectrum's edges, spaced in pitch. */
  sub: [30, 60],
  bass: [30, 150],
  mid: [250, 2000],
  high: [4000, 10500],
  spectrum: [40, 10500],
  /** Where the onsets are looked for, Hz: the kick's, the snare's body and crack, the hats'. */
  kickBand: [30, 150],
  snareBands: [
    [150, 700],
    [1500, 5000],
  ],
  hatBand: [6000, 10500],
  /** Onsets: the neighbourhood their threshold is the median and spread of, s either side; how many spreads over it. */
  onsetWindow: 1.0,
  onsetSpreads: 2.5,
  /** The least mean rise across a band that counts, dB a frame: smaller is noise, however steady the song. */
  leastRiseDb: 1.0,
  /** An onset rises across its band: at least this share of its bins rise by `binRiseDb`, so a tone gliding from bin to bin is no drum. */
  leastSpread: 0.4,
  /**
   * Where the low band has been noise (its flatness over the second around over `noiseFlatness`) or a
   * pure tone (under `toneFlatness`), a kick must lift most of the band at once and by much: at
   * least `strictSpread` of its bins by `strictRiseDb`. Noise's random peaks and a gliding tone's
   * entry into the next bins are no kicks; a song's low end, tonal bass notes under a kit, is
   * neither, and keeps the plain test.
   */
  noiseFlatness: 0.55,
  toneFlatness: 0.05,
  strictRiseDb: 9,
  strictSpread: 0.6,
  /** A kick's low band rises at least this share of what the snare's bands rise, in spreads (a kick's beater clicks in them too); a snare's bands rise at least this share of the low's. */
  kickOver: 0.8,
  snareOver: 1.0,
  binRiseDb: 3,
  /** The least time between two onsets of a kind, s. */
  kickGap: 0.09,
  snareGap: 0.12,
  hatGap: 0.045,
  /** The tempo: its range, BPM; the prior's middle, BPM, and width, octaves; the window it is found over and its step, s. */
  slowest: 55,
  fastest: 200,
  likeliest: 120,
  priorOctaves: 1.5,
  tempoWindow: 8,
  tempoStep: 2,
  /** What a tempo change costs between two windows, per octave, against how much a window's fit counts: an octave jump takes three windows' clear word. */
  tempoInertia: 3,
  tempoSharpness: 4,
  /** How far the best grid's fit must stand over an unrelated tempo's, a share of its own, for full confidence; and how near, as a ratio's log, a tempo is related. */
  confidenceSpan: 0.3,
  related: 0.04,
  /** How near a beat an onset must land to count as on it, s. */
  onBeat: 0.035,
  /** The fewest onsets a tempo window needs, a second, for any confidence: a drone or a sweep has none. */
  leastOnsets: 0.5,
  /** The beat tracker: how hard it holds a beat to the tempo against an onset off it. */
  beatTightness: 100,
  /** From this confidence on the grid is trusted: sections snap to its bars and the beat-locked mappings play. */
  trusted: 0.5,
  /** Sections: the unit when the grid is not trusted, s; the novelty kernel's half-width and the least section, in units; the least novelty. */
  freeUnit: 2,
  kernel: 8,
  leastSection: 8,
  leastNovelty: 0.12,
  /** A change stands this many spreads over the song's mean novelty. */
  noveltySpreads: 0.75,
  /** A section this far under the loudest one is a break, weighed 0.2, dB. */
  breakDb: 9,
  /** A build: the most units it runs back from a louder section; how much the brightness, or the loudness in dB, must rise over it. */
  longestBuild: 16,
  buildRise: 0.05,
  buildRiseDb: 1.5,
  /** The brightness weighs each band by how far it stands over this far under the frame's loudest band, dB. */
  brightDepthDb: 50,
  /** A drop: a section whose low end comes back this far over the units before it, normalised. */
  dropRise: 0.2,
} as const;

/** Each feature's meter, s: how fast it rises, and how fast it falls. `docs/design/visualiser.md` lists them. */
const FOLLOW = {
  level: [0.03, 0.25],
  loud: [0.5, 1.5],
  sub: [0.004, 0.16],
  bass: [0.004, 0.16],
  bassSlow: [0.3, 0.9],
  mid: [0.03, 0.25],
  high: [0.03, 0.25],
  spectrum: [0.06, 0.3],
  centroid: [0.15, 0.4],
  balance: [0.2, 0.5],
} as const;

export const SIZE = ANALYSIS.size;
export const HOP = ANALYSIS.hop;

/** What the worker hands back. Every array is per frame unless it says otherwise; every feature 0 to about 1. */
export interface SongData {
  readonly rate: number;
  readonly frames: number;
  readonly level: Float32Array;
  readonly loud: Float32Array;
  readonly sub: Float32Array;
  readonly bass: Float32Array;
  readonly bassSlow: Float32Array;
  readonly mid: Float32Array;
  readonly high: Float32Array;
  /** `BANDS` a frame, one frame after another. */
  readonly spectrum: Float32Array;
  readonly centroid: Float32Array;
  readonly balance: Float32Array;
  /** The beat grid: each beat's time, s, and how far it can be trusted, 0 to 1; which of the first four beats is a downbeat. */
  readonly beatTimes: Float32Array;
  readonly beatConfidence: Float32Array;
  readonly downbeat: number;
  /** The song's tempo, BPM, and the grid's confidence over the whole song, 0 to 1. */
  readonly bpm: number;
  readonly confidence: number;
  readonly kickTimes: Float32Array;
  readonly kickStrengths: Float32Array;
  readonly snareTimes: Float32Array;
  readonly snareStrengths: Float32Array;
  readonly hatTimes: Float32Array;
  readonly hatStrengths: Float32Array;
  /** Each section's start, s, its weight, where its build starts (-1 for none), and whether it is a drop. */
  readonly sectionStarts: Float32Array;
  readonly sectionWeights: Float32Array;
  readonly sectionBuilds: Float32Array;
  readonly sectionDrops: Uint8Array;
  /** The song in one channel, every `WAVE_STEP`th sample, -127 to 127, for the waveform. */
  readonly wave: Int8Array;
}

/** The waveform keeps every this many samples. */
export const WAVE_STEP = 4;

/** An in-place radix-2 FFT of `re` and `im`, their length a power of two. */
function fft(re: Float64Array, im: Float64Array): void {
  const n = re.length;
  for (let i = 1, j = 0; i < n; i++) {
    let bit = n >> 1;
    for (; j & bit; bit >>= 1) j ^= bit;
    j ^= bit;
    if (i < j) {
      [re[i], re[j]] = [re[j]!, re[i]!];
      [im[i], im[j]] = [im[j]!, im[i]!];
    }
  }
  for (let len = 2; len <= n; len <<= 1) {
    const angle = (-2 * Math.PI) / len;
    const wr = Math.cos(angle);
    const wi = Math.sin(angle);
    for (let i = 0; i < n; i += len) {
      let cr = 1;
      let ci = 0;
      for (let k = 0; k < len / 2; k++) {
        const a = i + k;
        const b = a + len / 2;
        const tr = re[b]! * cr - im[b]! * ci;
        const ti = re[b]! * ci + im[b]! * cr;
        re[b] = re[a]! - tr;
        im[b] = im[a]! - ti;
        re[a] = re[a]! + tr;
        im[a] = im[a]! + ti;
        const next = cr * wr - ci * wi;
        ci = cr * wi + ci * wr;
        cr = next;
      }
    }
  }
}

/** The `share` quantile of `values`. */
function quantile(values: ArrayLike<number>, share: number): number {
  if (values.length === 0) return 0;
  const sorted = Float64Array.from(values).sort();
  return sorted[Math.min(sorted.length - 1, Math.floor(share * sorted.length))]!;
}

/** `values` followed as a meter: rising over `attack` seconds, falling over `release`, at `rate` frames a second. */
function meter(values: Float32Array, rate: number, [attack, release]: readonly [number, number], stride = 1, offset = 0): void {
  const up = 1 - Math.exp(-1 / (rate * attack));
  const down = 1 - Math.exp(-1 / (rate * release));
  let now = values[offset] ?? 0;
  for (let i = offset; i < values.length; i += stride) {
    const target = values[i]!;
    now += (target - now) * (target > now ? up : down);
    values[i] = now;
  }
}

/** Power to decibels. */
const db = (power: number) => 10 * Math.log10(Math.max(power, 1e-16));

/**
 * Levels in dB spread over the song's own range: its quiet end (`quietShare` of its sounding frames)
 * reads 0 and its loud end (`loudShare`) 1, over `leastRangeDb` at least, its top never under
 * `songTop - spanDb`; anything at or under `floor` (`depthDb` under the song's top) reads 0.
 */
function ranged(levels: Float32Array, songTop: number, stride = 1, offset = 0, floor: number = Math.max(ANALYSIS.floorDb, songTop - ANALYSIS.depthDb)): Float32Array {
  const sounding: number[] = [];
  for (let i = offset; i < levels.length; i += stride) if (levels[i]! > floor) sounding.push(levels[i]!);
  const out = new Float32Array(Math.ceil((levels.length - offset) / stride));
  if (sounding.length === 0) return out;
  const top = Math.max(quantile(sounding, ANALYSIS.loudShare), songTop - ANALYSIS.spanDb);
  const bottom = Math.max(floor, Math.min(quantile(sounding, ANALYSIS.quietShare), top - ANALYSIS.leastRangeDb));
  for (let i = offset, j = 0; i < levels.length; i += stride, j++) {
    const value = levels[i]!;
    out[j] = value <= floor ? 0 : Math.max(0, Math.min(1.2, (value - bottom) / Math.max(1e-6, top - bottom)));
  }
  return out;
}

/**
 * The K-weighting of ITU-R BS.1770 as a power gain at `hz`: its high shelf, +4 dB above about
 * 1.5 kHz, and its high-pass at 38 Hz, second order, both as magnitudes.
 */
function kWeight(hz: number): number {
  const shelf = 1 + (Math.pow(10, 0.4) - 1) * ((hz * hz) / (hz * hz + 1500 * 1500));
  const f4 = hz ** 4;
  return (shelf * f4) / (f4 + 38 ** 4);
}

/** Medians and spreads (the median absolute deviation, as a standard deviation) of `values` over `reach` frames either side, every `step` frames. */
function neighbourhood(values: Float32Array, reach: number, step = 4): { median: Float32Array; spread: Float32Array } {
  const median = new Float32Array(values.length);
  const spread = new Float32Array(values.length);
  for (let f = 0; f < values.length; f += step) {
    const window = Float64Array.from(values.subarray(Math.max(0, f - reach), Math.min(values.length, f + reach + 1))).sort();
    const m = window[window.length >> 1]!;
    const deviations = window.map((v) => Math.abs(v - m)).sort();
    const mad = deviations[deviations.length >> 1]! * 1.4826;
    for (let g = f; g < Math.min(values.length, f + step); g++) {
      median[g] = m;
      spread[g] = mad;
    }
  }
  return { median, spread };
}

/** A list of numbers growing as the song comes in, four bytes each, doubling its room as it fills. */
class Grow {
  private data = new Float32Array(4096);
  length = 0;
  push(value: number): void {
    if (this.length === this.data.length) {
      const more = new Float32Array(this.data.length * 2);
      more.set(this.data);
      this.data = more;
    }
    this.data[this.length++] = value;
  }
  /** What it holds, as one array. */
  all(): Float32Array {
    return this.data.subarray(0, this.length);
  }
}

/** The waveform growing as the song comes in, a byte a sample. */
class GrowWave {
  private data = new Int8Array(1 << 16);
  length = 0;
  push(value: number): void {
    if (this.length === this.data.length) {
      const more = new Int8Array(this.data.length * 2);
      more.set(this.data);
      this.data = more;
    }
    this.data[this.length++] = value;
  }
  all(): Int8Array {
    return this.data.slice(0, this.length);
  }
}

/** A song's analysis, fed its samples a chunk at a time and finished once. */
class Analyser {
  private readonly rate: number;
  private readonly frameRate: number;
  private readonly bins = SIZE / 2;
  private readonly window = Float64Array.from({ length: SIZE }, (_, i) => 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (SIZE - 1)));
  /** Half the window's sum: dividing a bin's magnitude by it makes a full-scale sine read 0 dB. */
  private readonly gain: number;
  private readonly weights: Float64Array;
  private readonly edges: number[];
  private pendingL = new Float32Array(0);
  private pendingR = new Float32Array(0);
  private readonly re = new Float64Array(SIZE);
  private readonly im = new Float64Array(SIZE);
  private last = new Float64Array(SIZE / 2);
  private now = new Float64Array(SIZE / 2);
  // Each frame's numbers, grown as the song comes in.
  private loudness = new Grow();
  private subDb = new Grow();
  private bassDb = new Grow();
  private midDb = new Grow();
  private highDb = new Grow();
  private bandDb = new Grow();
  private centroidOf = new Grow();
  private balanceOf = new Grow();
  private kickFlux = new Grow();
  private snareFlux = new Grow();
  private hatFlux = new Grow();
  private kickSpread = new Grow();
  private lowFlatness = new Grow();
  private kickStrict = new Grow();
  private snareSpread = new Grow();
  private hatSpread = new Grow();
  private heldLast: number = ANALYSIS.floorDb;
  private waveOut = new GrowWave();
  private waveAt = 0;

  constructor(rate: number) {
    this.rate = rate;
    this.frameRate = rate / HOP;
    this.gain = this.window.reduce((a, b) => a + b, 0) / 2;
    this.weights = Float64Array.from({ length: this.bins }, (_, k) => kWeight(this.hz(k)));
    const [low, high] = ANALYSIS.spectrum;
    this.edges = Array.from({ length: BANDS + 1 }, (_, i) => this.bin(low * Math.pow(high / low, i / BANDS)));
  }

  private hz(k: number): number {
    return (k * this.rate) / SIZE;
  }

  private bin(hz: number): number {
    return Math.max(1, Math.min(this.bins - 1, Math.round((hz * SIZE) / this.rate)));
  }

  private range([from, to]: readonly number[]): [number, number] {
    return [this.bin(from!), Math.max(this.bin(from!), this.bin(to!))];
  }

  /** Takes the next stretch of the song's two channels. */
  push(left: Float32Array, right: Float32Array): void {
    const l = new Float32Array(this.pendingL.length + left.length);
    l.set(this.pendingL);
    l.set(left, this.pendingL.length);
    const r = new Float32Array(this.pendingR.length + right.length);
    r.set(this.pendingR);
    r.set(right, this.pendingR.length);
    for (; this.waveAt < l.length; this.waveAt += WAVE_STEP) this.waveOut.push(Math.round(Math.max(-1, Math.min(1, (l[this.waveAt]! + r[this.waveAt]!) / 2)) * 127));
    let start = 0;
    for (; start + SIZE <= l.length; start += HOP) this.frame(l, r, start);
    this.waveAt -= start;
    this.pendingL = l.slice(start);
    this.pendingR = r.slice(start);
  }

  private frame(l: Float32Array, r: Float32Array, start: number): void {
    const { re, im, window } = this;
    let mean = 0;
    for (let i = 0; i < SIZE; i++) mean += l[start + i]! + r[start + i]!;
    // The DC offset, taken out before the window: it would otherwise read as a low it isn't.
    mean /= SIZE * 2;
    let left = 0;
    let right = 0;
    for (let i = 0; i < SIZE; i++) {
      const a = l[start + i]! - mean;
      const b = r[start + i]! - mean;
      left += a * a;
      right += b * b;
      re[i] = ((a + b) / 2) * window[i]!;
      im[i] = 0;
    }
    fft(re, im);
    // Each bin's power, a full-scale sine reading 0 dB.
    let weighted = 0;
    for (let k = 1; k < this.bins; k++) {
      const power = (re[k]! ** 2 + im[k]! ** 2) / (this.gain * this.gain);
      this.now[k] = power;
      weighted += power * this.weights[k]!;
    }
    const band = (edges: readonly number[]) => {
      const [from, to] = this.range(edges);
      let sum = 0;
      for (let k = from; k <= to; k++) sum += this.now[k]!;
      return db(sum / (to - from + 1));
    };
    // The loudness: the K-weighted power of both channels summed, as BS.1770 sums them (here their mid, twice).
    this.loudness.push(-0.691 + db(2 * weighted));
    this.subDb.push(band(ANALYSIS.sub));
    this.bassDb.push(band(ANALYSIS.bass));
    this.midDb.push(band(ANALYSIS.mid));
    this.highDb.push(band(ANALYSIS.high));
    for (let b = 0; b < BANDS; b++) {
      let sum = 0;
      const from = this.edges[b]!;
      const to = Math.max(from, this.edges[b + 1]! - 1);
      for (let k = from; k <= to; k++) sum += this.now[k]!;
      this.bandDb.push(db(sum / (to - from + 1)));
    }
    // The brightness: where the spectrum's weight sits in pitch, each band weighed by how far it
    // stands over `brightDepthDb` under the frame's loudest band, as the ear hears a band's presence
    // rather than its power (power alone always sits in the bass): 0 at the spectrum's low edge, 1
    // at its top.
    let loudestBand = -Infinity;
    const frameBands = this.bandDb.all().subarray(this.bandDb.length - BANDS);
    for (const v of frameBands) loudestBand = Math.max(loudestBand, v);
    let place = 0;
    let weight = 0;
    frameBands.forEach((v, b) => {
      const w = Math.max(0, v - (loudestBand - ANALYSIS.brightDepthDb));
      place += (b / (BANDS - 1)) * w;
      weight += w;
    });
    this.centroidOf.push(weight > 0 && loudestBand > ANALYSIS.floorDb ? place / weight : 0.5);
    const sum = Math.sqrt(left) + Math.sqrt(right);
    this.balanceOf.push(sum > 1e-4 ? (Math.sqrt(right) - Math.sqrt(left)) / sum : 0);
    // The onsets' fluxes: how much each band rose since the last frame, in dB, each bin held at the
    // floor, so a rise out of silence counts no more than the floor allows.
    // Each also says what share of its bins rose by `binRiseDb`: a drum rises across its band. A bin
    // is held at `depthDb` under the frame's loudest, never under the floor, so the fluxes are the
    // same whatever the song's level.
    let loudest = 0;
    for (let k = 1; k < this.bins; k++) loudest = Math.max(loudest, this.now[k]!);
    const held = Math.max(ANALYSIS.floorDb, db(loudest) - ANALYSIS.depthDb);
    const heldLast = this.heldLast;
    this.heldLast = held;
    const rise = (edges: readonly number[], binRise: number = ANALYSIS.binRiseDb): [number, number] => {
      const [from, to] = this.range(edges);
      let sum = 0;
      let rising = 0;
      for (let k = from; k <= to; k++) {
        const up = Math.max(0, Math.max(db(this.now[k]!), held) - Math.max(db(this.last[k]!), heldLast));
        sum += up;
        if (up >= binRise) rising++;
      }
      return [sum / (to - from + 1), rising / (to - from + 1)];
    };
    const kick = rise(ANALYSIS.kickBand);
    this.kickStrict.push(rise(ANALYSIS.kickBand, ANALYSIS.strictRiseDb)[1]);
    // The low band's flatness: its bins' geometric mean over their mean, 1 for noise, near 0 for a tone.
    {
      const [from, to] = this.range(ANALYSIS.kickBand);
      let logs = 0;
      let sum = 0;
      for (let k = from; k <= to; k++) {
        logs += Math.log(this.now[k]! + 1e-20);
        sum += this.now[k]!;
      }
      const count = to - from + 1;
      this.lowFlatness.push(sum > 0 ? Math.exp(logs / count) / (sum / count) : 0);
    }
    const body = rise(ANALYSIS.snareBands[0]);
    const crack = rise(ANALYSIS.snareBands[1]);
    const hat = rise(ANALYSIS.hatBand);
    this.kickFlux.push(kick[0]);
    this.kickSpread.push(kick[1]);
    this.snareFlux.push((body[0] + crack[0]) / 2);
    this.snareSpread.push((body[1] + crack[1]) / 2);
    this.hatFlux.push(hat[0]);
    this.hatSpread.push(hat[1]);
    [this.last, this.now] = [this.now, this.last];
  }

  /** Works out what needs the whole song, and hands it all back. */
  finish(): SongData {
    const rate = this.frameRate;
    const frames = this.loudness.length;
    const [loudnessDb, subDb, bassDb, midDb, highDb, bandDb, centroidOf, balanceOf] = [this.loudness, this.subDb, this.bassDb, this.midDb, this.highDb, this.bandDb, this.centroidOf, this.balanceOf].map((each) => each.all());
    const [kickFlux, snareFlux, hatFlux, kickSpread, snareSpread, hatSpread] = [this.kickFlux, this.snareFlux, this.hatFlux, this.kickSpread, this.snareSpread, this.hatSpread].map((each) => each.all());
    const time = (f: number) => (f * HOP + SIZE / 2) / this.rate;
    const f32 = (values: number[]) => Float32Array.from(values);

    // The loudness, momentary over 400 ms and short-term over 3 s, then spread over the song's range.
    const average = (values: Float32Array, seconds: number) => {
      const reach = Math.max(1, Math.round((seconds * rate) / 2));
      const sums = new Float64Array(frames + 1);
      for (let f = 0; f < frames; f++) sums[f + 1] = sums[f]! + Math.pow(10, values[f]! / 10);
      return Float32Array.from({ length: frames }, (_, f) => {
        const from = Math.max(0, f - reach);
        const to = Math.min(frames, f + reach);
        return db((sums[to]! - sums[from]!) / Math.max(1, to - from));
      });
    };
    const momentary = average(loudnessDb!, 0.4);
    const shortTerm = average(loudnessDb!, 3);
    const songTop = Math.max(...[subDb!, bassDb!, midDb!, highDb!].map((each) => quantile(each, ANALYSIS.loudShare)));
    const loudTop = quantile(momentary, ANALYSIS.loudShare);
    const level = ranged(momentary, loudTop, 1, 0, ANALYSIS.gateLufs);
    const loud = ranged(shortTerm, loudTop, 1, 0, ANALYSIS.gateLufs);
    const sub = ranged(subDb!, songTop);
    const bass = ranged(bassDb!, songTop);
    const mid = ranged(midDb!, songTop);
    const high = ranged(highDb!, songTop);
    // Under the loudness's gate the song is silent, whatever its bands hold: dither and hiss stay dark.
    const silent = (f: number) => momentary[f]! <= ANALYSIS.gateLufs;
    for (const each of [sub, bass, mid, high]) for (let f = 0; f < frames; f++) if (silent(f)) each[f] = 0;
    const bands = bandDb!;
    const spectrum = new Float32Array(frames * BANDS);
    for (let b = 0; b < BANDS; b++) {
      const spread = ranged(bands, songTop, BANDS, b);
      for (let f = 0; f < frames; f++) spectrum[f * BANDS + b] = silent(f) ? 0 : spread[f]!;
      meter(spectrum, rate, FOLLOW.spectrum, BANDS, b);
    }
    const bassSlow = bass.slice();
    meter(bassSlow, rate, FOLLOW.bassSlow);
    meter(level, rate, FOLLOW.level);
    meter(loud, rate, FOLLOW.loud);
    meter(sub, rate, FOLLOW.sub);
    meter(bass, rate, FOLLOW.bass);
    meter(mid, rate, FOLLOW.mid);
    meter(high, rate, FOLLOW.high);
    const centroid = centroidOf!.slice();
    // Silence has no brightness of its own: it holds the last sound's.
    for (let f = 1; f < frames; f++) if (momentary[f]! <= ANALYSIS.gateLufs) centroid[f] = centroid[f - 1]!;
    meter(centroid, rate, FOLLOW.centroid);
    const balance = balanceOf!.slice();
    meter(balance, rate, FOLLOW.balance);

    // The onsets: each band's rise against the median and spread of the second around it.
    const reach = Math.round(ANALYSIS.onsetWindow * rate);
    const scored = (values: Float32Array) => {
      const { median, spread } = neighbourhood(values, reach);
      // A spread never under half the least rise, so a steady band never reads its noise as onsets.
      return values.map((v, f) => (v - median[f]!) / Math.max(spread[f]!, ANALYSIS.leastRiseDb / 2));
    };
    const kickZ = scored(kickFlux!);
    const snareZ = scored(snareFlux!);
    const hatZ = scored(hatFlux!);
    const peak = (z: Float32Array, f: number) => z[f]! >= ANALYSIS.onsetSpreads && z[f]! >= z[f - 1]! && z[f]! >= z[f + 1]! && z[f]! >= z[f - 2]! && z[f]! >= z[f + 2]!;
    type Kind = "kick" | "snare" | "hat";
    const found: Record<Kind, number[]> = { kick: [], snare: [], hat: [] };
    const strength: Record<Kind, number[]> = { kick: [], snare: [], hat: [] };
    const last: Record<Kind, number> = { kick: -Infinity, snare: -Infinity, hat: -Infinity };
    const gaps: Record<Kind, number> = { kick: ANALYSIS.kickGap, snare: ANALYSIS.snareGap, hat: ANALYSIS.hatGap };
    const add = (kind: Kind, f: number, z: number) => {
      const at = time(f);
      if (at - last[kind] < gaps[kind]) return;
      last[kind] = at;
      found[kind].push(at);
      strength[kind].push(z);
    };
    const flatAround = neighbourhood(this.lowFlatness.all(), Math.round(rate), 4).median;
    const strict = this.kickStrict.all();
    const transient = (f: number) => {
      const flat = flatAround[f]!;
      return (flat <= ANALYSIS.noiseFlatness && flat >= ANALYSIS.toneFlatness) || strict[f]! >= ANALYSIS.strictSpread;
    };
    for (let f = 2; f < frames - 2; f++) {
      if (silent(f)) continue;
      // A kick: the low band peaks, past the least rise, across its bins, rising at least `kickOver` of
      // what the snare's bands rose; where the low band is noise or a pure tone, lifting most of it by much.
      if (peak(kickZ, f) && kickFlux![f]! >= ANALYSIS.leastRiseDb && kickSpread![f]! >= ANALYSIS.leastSpread && kickZ[f]! >= snareZ[f]! * ANALYSIS.kickOver && transient(f)) add("kick", f, kickZ[f]!);
      // A snare: its bands peak, rising more than the low (`snareOver`), with the top rising too (its crack).
      else if (peak(snareZ, f) && snareFlux![f]! >= ANALYSIS.leastRiseDb && snareSpread![f]! >= ANALYSIS.leastSpread && snareZ[f]! > kickZ[f]! * ANALYSIS.snareOver && hatZ[f]! >= 1) add("snare", f, snareZ[f]!);
      // A hat: the top peaks while neither the low nor the snare's bands do.
      if (peak(hatZ, f) && hatFlux![f]! >= ANALYSIS.leastRiseDb && hatSpread![f]! >= ANALYSIS.leastSpread && kickZ[f]! < ANALYSIS.onsetSpreads && snareZ[f]! < ANALYSIS.onsetSpreads) add("hat", f, hatZ[f]!);
    }
    // Strengths against the song's own strong ones of each kind: its strong ones read about 1.
    const strengths = (values: number[]) => {
      const strong = Math.max(1e-6, quantile(values, 0.9));
      return Float32Array.from(values, (v) => Math.min(1.4, v / strong));
    };

    // The tempo and the beats, from the onsets' envelope.
    const envelope = new Float32Array(frames);
    for (let f = 0; f < frames; f++) envelope[f] = Math.min(10, Math.max(0, kickZ[f]!) + Math.max(0, snareZ[f]!) + 0.5 * Math.max(0, hatZ[f]!));
    // The tempo is read from the drums, kicks and snares, unless there are too few of them to read
    // (under one a second): then from every onset, hats too.
    const drums = [...found.kick, ...found.snare];
    const timed = drums.length >= time(frames - 1) ? drums : [...drums, ...found.hat];
    const onsetFrames = timed.map((t) => (t * this.rate - SIZE / 2) / HOP).sort((a, b) => a - b);
    const grid = this.tempo(envelope, rate, onsetFrames);
    const beats = this.beats(envelope, grid);
    const beatTimes = Float32Array.from(beats, (f) => time(f));
    // The downbeat: of a bar's four beats, the one the kicks fall on most.
    const kicksAt = [0, 0, 0, 0];
    let b = 0;
    for (const at of found.kick) {
      while (b + 1 < beatTimes.length && beatTimes[b + 1]! <= at) b++;
      const nearest = b + 1 < beatTimes.length && beatTimes[b + 1]! - at < at - beatTimes[b]! ? b + 1 : b;
      if (Math.abs(at - (beatTimes[nearest] ?? Infinity)) < 0.07) kicksAt[nearest % 4]!++;
    }
    const downbeat = kicksAt.indexOf(Math.max(...kicksAt));
    const beatConfidence = Float32Array.from(beats, (f) => grid.confidenceAt(f));
    const confidence = quantile(beatConfidence, 0.5);
    // The tempo: from the beats themselves, each beat's over the eight around it, their median; the
    // likeliest tempo where there is no grid to trust.
    const local = beats.map((_, i) => {
      const from = Math.max(0, i - 4);
      const to = Math.min(beats.length - 1, i + 4);
      return to > from ? (60 * (to - from)) / (beatTimes[to]! - beatTimes[from]!) : ANALYSIS.likeliest;
    });
    const bpm = confidence > 0 ? quantile(local, 0.5) : ANALYSIS.likeliest;

    const structure = this.structure({ frames, time, shortTerm, bass, sub, mid, high, centroid, bands, kicks: found.kick, snares: found.snare, hats: found.hat, beatTimes, downbeat, confidence });

    return {
      rate,
      frames,
      level,
      loud,
      sub,
      bass,
      bassSlow,
      mid,
      high,
      spectrum,
      centroid,
      balance,
      beatTimes,
      beatConfidence,
      downbeat,
      bpm,
      confidence,
      kickTimes: f32(found.kick),
      kickStrengths: strengths(strength.kick),
      snareTimes: f32(found.snare),
      snareStrengths: strengths(strength.snare),
      hatTimes: f32(found.hat),
      hatStrengths: strengths(strength.hat),
      ...structure,
      wave: this.waveOut.all(),
    };
  }

  /**
   * The tempo, window by window: how well a beat grid at each tempo fits the window's drum onsets,
   * weighed by a prior over tempos in octaves; the path through the windows that best keeps to the
   * fits with the fewest changes; and each window's confidence.
   */
  private tempo(envelope: Float32Array, rate: number, onsets: number[]): { periodAt: (f: number) => number; confidenceAt: (f: number) => number; median: number } {
    const frames = envelope.length;
    const width = Math.max(1, Math.min(frames, Math.round(ANALYSIS.tempoWindow * rate)));
    const step = Math.max(1, Math.round(ANALYSIS.tempoStep * rate));
    const starts: number[] = [];
    for (let s = 0; s + width <= frames; s += step) starts.push(s);
    if (starts.length === 0) starts.push(0);
    const tempos: number[] = [];
    for (let t = ANALYSIS.slowest; t <= ANALYSIS.fastest; t += 0.5) tempos.push(t);
    const prior = tempos.map((t) => Math.exp(-0.5 * (Math.log2(t / ANALYSIS.likeliest) / ANALYSIS.priorOctaves) ** 2));
    const scores: Float64Array[] = [];
    const confidences: number[] = [];
    const tolerance = ANALYSIS.onBeat * rate;
    for (const start of starts) {
      const end = start + width;
      const inside = onsets.filter((f) => f >= start && f < end);
      // How well a grid of beats at each tempo fits the window's onsets, at its best phase: the
      // F-measure of the onsets that land on a beat and the beats that have an onset. A grid twice
      // as fast catches the same onsets but leaves half its beats empty, so neither multiples nor
      // fractions of the tempo win by being denser or sparser.
      const fits = tempos.map((t) => {
        if (inside.length < 2) return 0;
        const period = (60 * rate) / t;
        let best = 0;
        for (let phase = 0; phase < period; phase += tolerance) {
          let landed = 0;
          for (const f of inside) {
            const x = (((f - start - phase) % period) + period) % period;
            if (Math.min(x, period - x) <= tolerance) landed++;
          }
          let beats = 0;
          let filled = 0;
          for (let at = start + phase, i = 0; at < end; at += period) {
            beats++;
            while (i < inside.length && inside[i]! < at - tolerance) i++;
            if (i < inside.length && Math.abs(inside[i]! - at) <= tolerance) filled++;
          }
          const recall = landed / inside.length;
          const precision = filled / Math.max(1, beats);
          best = Math.max(best, recall + precision > 0 ? (2 * recall * precision) / (recall + precision) : 0);
        }
        return best;
      });
      // Each window's say, against its own best: a window with too few onsets says nothing.
      const enough = inside.length >= ANALYSIS.leastOnsets * (width / rate);
      const weighed = fits.map((fit, i) => fit * prior[i]!);
      const top = Math.max(1e-6, ...weighed);
      scores.push(Float64Array.from(weighed, (w) => (enough ? w / top : 1)));
      // Confidence: how far the best fit stands over the best fit of a tempo unrelated to it (not
      // within `related` of it, nor of its half, double, two and three thirds or quarters): onsets
      // at random fit some tempo nearly as well as any other, a beat fits its own family alone.
      const best = Math.max(...fits);
      const at = tempos[fits.indexOf(best)]!;
      const family = [1, 0.5, 2, 2 / 3, 1.5, 0.75, 4 / 3].map((ratio) => at * ratio);
      let rival = 0;
      tempos.forEach((t, i) => {
        if (family.every((f) => Math.abs(Math.log(t / f)) > ANALYSIS.related)) rival = Math.max(rival, fits[i]!);
      });
      confidences.push(enough && best > 0 ? Math.max(0, Math.min(1, (best - rival) / best / ANALYSIS.confidenceSpan)) : 0);
    }
    // The path: each window's tempo, kept unless a change pays for itself.
    const n = tempos.length;
    let cost = Float64Array.from(scores[0]!, (s) => -ANALYSIS.tempoSharpness * Math.log(s + 1e-3));
    const back: Int16Array[] = [];
    for (let w = 1; w < starts.length; w++) {
      const next = new Float64Array(n);
      const from = new Int16Array(n);
      for (let i = 0; i < n; i++) {
        let best = Infinity;
        let at = i;
        for (let j = 0; j < n; j++) {
          const c = cost[j]! + ANALYSIS.tempoInertia * Math.abs(Math.log2(tempos[i]! / tempos[j]!));
          if (c < best) [best, at] = [c, j];
        }
        next[i] = best - ANALYSIS.tempoSharpness * Math.log(scores[w]![i]! + 1e-3);
        from[i] = at;
      }
      cost = next;
      back.push(from);
    }
    const path = new Array<number>(starts.length);
    path[starts.length - 1] = cost.indexOf(Math.min(...cost));
    for (let w = starts.length - 1; w > 0; w--) path[w - 1] = back[w - 1]![path[w]!]!;
    const chosen = path.map((i) => tempos[i]!);
    const middles = starts.map((s) => s + width / 2);
    const at = (values: number[], f: number) => {
      if (f <= middles[0]!) return values[0]!;
      for (let w = 1; w < middles.length; w++) if (f <= middles[w]!) return values[w - 1]! + ((values[w]! - values[w - 1]!) * (f - middles[w - 1]!)) / (middles[w]! - middles[w - 1]!);
      return values[values.length - 1]!;
    };
    return { periodAt: (f) => (60 * rate) / at(chosen, f), confidenceAt: (f) => at(confidences, f), median: quantile(chosen, 0.5) };
  }

  /**
   * The beats: the path through the frames that lands on onsets while keeping each step near the
   * local period (Ellis's dynamic programme), so a tempo change is followed and a missing beat
   * filled; before its first and after its last, the grid goes on at the period.
   */
  private beats(envelope: Float32Array, grid: { periodAt: (f: number) => number }): number[] {
    const frames = envelope.length;
    let mean = 0;
    for (const v of envelope) mean += v / frames;
    let deviation = 0;
    for (const v of envelope) deviation += (v - mean) ** 2 / frames;
    const scale = Math.sqrt(deviation) || 1;
    const score = new Float64Array(frames);
    const from = new Int32Array(frames).fill(-1);
    for (let f = 0; f < frames; f++) {
      const period = grid.periodAt(f);
      let best = -Infinity;
      let at = -1;
      for (let back = Math.round(period * 0.8); back <= Math.round(period * 1.25); back++) {
        const g = f - back;
        if (g < 0) break;
        const s = score[g]! - ANALYSIS.beatTightness * Math.log(back / period) ** 2;
        if (s > best) [best, at] = [s, g];
      }
      score[f] = envelope[f]! / scale + Math.max(0, best);
      from[f] = at >= 0 && best > 0 ? at : -1;
    }
    let end = frames - 1;
    for (let f = Math.max(0, frames - Math.round(grid.periodAt(frames - 1))); f < frames; f++) if (score[f]! > score[end]!) end = f;
    const beats: number[] = [];
    for (let f = end; f >= 0; f = from[f]!) beats.push(f);
    beats.reverse();
    const lead: number[] = [];
    for (let f = beats[0]! - grid.periodAt(beats[0]!); f >= 0; f -= grid.periodAt(f)) lead.unshift(Math.round(f));
    const tail: number[] = [];
    for (let f = beats.at(-1)! + grid.periodAt(beats.at(-1)!); f < frames; f += grid.periodAt(f)) tail.push(Math.round(f));
    return [...lead, ...beats, ...tail];
  }

  /**
   * The sections: units of a bar where the grid is trusted, of `freeUnit` seconds where it is not;
   * each unit's features, standardised; their self-similarity; its novelty under a checkerboard
   * kernel; and the novelty's peaks, `leastSection` units apart, as the changes. Each section is
   * weighed by its loudness against the loudest's; a build is the run of units before a louder
   * section over which the sound brightens or swells while its low end holds or falls; a drop is a
   * section whose low end comes back over the units before it.
   */
  private structure(song: {
    frames: number;
    time: (f: number) => number;
    shortTerm: Float32Array;
    sub: Float32Array;
    bass: Float32Array;
    mid: Float32Array;
    high: Float32Array;
    centroid: Float32Array;
    bands: Float32Array;
    kicks: number[];
    snares: number[];
    hats: number[];
    beatTimes: Float32Array;
    downbeat: number;
    confidence: number;
  }): { sectionStarts: Float32Array; sectionWeights: Float32Array; sectionBuilds: Float32Array; sectionDrops: Uint8Array } {
    const length = song.time(song.frames - 1) + SIZE / 2 / this.rate;
    const units: number[] = [];
    if (song.confidence >= ANALYSIS.trusted && song.beatTimes.length > 8) {
      for (let b = song.downbeat; b < song.beatTimes.length; b += 4) units.push(song.beatTimes[b]!);
      if (units[0]! > 0.05) units.unshift(0);
    } else for (let t = 0; t < length; t += ANALYSIS.freeUnit) units.push(t);
    if (length - units.at(-1)! > 0.05) units.push(length);
    const count = Math.max(1, units.length - 1);
    const frameOf = (t: number) => Math.max(0, Math.min(song.frames - 1, Math.round((t * this.rate - SIZE / 2) / HOP)));
    const span = (u: number): [number, number] => {
      const from = frameOf(units[u]!);
      return [from, Math.max(from + 1, frameOf(units[u + 1] ?? length))];
    };
    const meanOver = (values: Float32Array, u: number) => {
      const [from, to] = span(u);
      let sum = 0;
      for (let f = from; f < to; f++) sum += values[f]!;
      return sum / (to - from);
    };
    const rateOver = (times: number[], u: number) => times.filter((t) => t >= units[u]! && t < (units[u + 1] ?? length)).length / Math.max(0.1, (units[u + 1] ?? length) - units[u]!);
    const loudDb = Array.from({ length: count }, (_, u) => {
      const [from, to] = span(u);
      let power = 0;
      for (let f = from; f < to; f++) power += Math.pow(10, song.shortTerm[f]! / 10);
      return db(power / (to - from));
    });
    const coarse = (u: number) => {
      const [from, to] = span(u);
      const out = new Array<number>(8).fill(0);
      for (let f = from; f < to; f++) for (let b = 0; b < BANDS; b++) out[b >> 2]! += Math.max(ANALYSIS.floorDb, song.bands[f * BANDS + b]!) / (4 * (to - from));
      return out;
    };
    const vectors = Array.from({ length: count }, (_, u) => [loudDb[u]!, meanOver(song.sub, u), meanOver(song.bass, u), meanOver(song.mid, u), meanOver(song.high, u), meanOver(song.centroid, u), rateOver(song.kicks, u), rateOver(song.snares, u), rateOver(song.hats, u), ...coarse(u)]);
    // Standardised, each feature against its spread over the song, never under a least spread (dB,
    // shares, onsets a second), so a feature that barely moves is not blown up into a difference.
    const least = [3, 0.1, 0.1, 0.1, 0.1, 0.05, 0.5, 0.5, 0.5, 3, 3, 3, 3, 3, 3, 3, 3];
    const dims = least.length;
    for (let d = 0; d < dims; d++) {
      let mean = 0;
      for (const v of vectors) mean += v[d]! / count;
      let spread = 0;
      for (const v of vectors) spread += (v[d]! - mean) ** 2 / count;
      spread = Math.max(Math.sqrt(spread), least[d]!);
      for (const v of vectors) v[d] = (v[d]! - mean) / spread;
    }
    // Two units are as alike as they are near in the standardised features: 1 the same, falling with distance.
    const similar = (i: number, j: number) => {
      let distance = 0;
      for (let d = 0; d < dims; d++) distance += (vectors[i]![d]! - vectors[j]![d]!) ** 2;
      return Math.exp(-Math.sqrt(distance) / 2);
    };
    // The novelty: a Gaussian-tapered checkerboard kernel along the self-similarity's diagonal.
    const K = ANALYSIS.kernel;
    const novelty = new Float64Array(count);
    for (let u = 0; u < count; u++) {
      let sum = 0;
      let weight = 0;
      for (let i = -K; i < K; i++) {
        for (let j = -K; j < K; j++) {
          const a = u + i;
          const b = u + j;
          if (a < 0 || b < 0 || a >= count || b >= count) continue;
          const taper = Math.exp(-((i + 0.5) ** 2 + (j + 0.5) ** 2) / (K * K));
          sum += ((i < 0) === (j < 0) ? -1 : 1) * taper * similar(a, b);
          weight += taper;
        }
      }
      novelty[u] = weight > 0 ? (-2 * sum) / weight : 0;
    }
    // Only where the kernel has two units at least on either side: at the song's very ends one half
    // is empty, and the other's likeness alone would read as a change.
    const valid = Array.from(novelty, (_, u) => u >= 2 && u <= count - 2);
    const values = Array.from(novelty).filter((_, u) => valid[u]);
    const mean = values.reduce((a, v) => a + v, 0) / Math.max(1, values.length);
    const spread = Math.sqrt(values.reduce((a, v) => a + (v - mean) ** 2, 0) / Math.max(1, values.length));
    const threshold = Math.max(ANALYSIS.leastNovelty, mean + ANALYSIS.noveltySpreads * spread);
    const changes: number[] = [];
    const edge = Math.floor(ANALYSIS.leastSection / 2);
    for (const [value, u] of Array.from(novelty, (v, u) => [v, u] as const).sort((a, b) => b[0] - a[0])) {
      if (value < threshold) break;
      if (!valid[u] || u < edge || u > count - edge) continue;
      if (changes.every((other) => Math.abs(other - u) >= ANALYSIS.leastSection)) changes.push(u);
    }
    changes.sort((a, b) => a - b);
    const bounds = [0, ...changes, count];
    const sectionLoud = bounds.slice(0, -1).map((from, i) => {
      let power = 0;
      let seconds = 0;
      for (let u = from; u < bounds[i + 1]!; u++) {
        const s = (units[u + 1] ?? length) - units[u]!;
        power += Math.pow(10, loudDb[u]! / 10) * s;
        seconds += s;
      }
      return db(power / Math.max(1e-6, seconds));
    });
    const loudest = Math.max(...sectionLoud);
    const weights = sectionLoud.map((l) => (l <= ANALYSIS.gateLufs ? 0.2 : 0.2 + 0.8 * Math.max(0, Math.min(1, 1 - (loudest - l) / ANALYSIS.breakDb))));
    const unitBass = Array.from({ length: count }, (_, u) => meanOver(song.bass, u));
    const unitCentroid = Array.from({ length: count }, (_, u) => meanOver(song.centroid, u));
    const builds = bounds.slice(0, -1).map((from, i) => {
      const to = bounds[i + 1]!;
      if (i + 1 >= weights.length || weights[i + 1]! < weights[i]! + 0.2) return -1;
      // Back from the change, while the sound brightens or swells and its low end holds or falls.
      let start = to;
      for (let u = to - 1; u >= Math.max(from, to - ANALYSIS.longestBuild); u--) {
        const brighter = unitCentroid[to - 1]! - unitCentroid[u]! >= ANALYSIS.buildRise;
        const swells = loudDb[to - 1]! - loudDb[u]! >= ANALYSIS.buildRiseDb;
        const holds = unitBass[to - 1]! <= unitBass[u]! + ANALYSIS.buildRise;
        if ((brighter || swells) && holds) start = u;
        else if (start !== to) break;
      }
      return to - start >= 2 ? units[start]! : -1;
    });
    const average = (values: number[]) => values.reduce((a, v) => a + v, 0) / Math.max(1, values.length);
    const drops = bounds.slice(0, -1).map((from) => (from > 0 && average(unitBass.slice(from, from + 4)) - average(unitBass.slice(Math.max(0, from - 4), from)) >= ANALYSIS.dropRise ? 1 : 0));
    return {
      sectionStarts: Float32Array.from(bounds.slice(0, -1), (u) => units[u]!),
      sectionWeights: Float32Array.from(weights),
      sectionBuilds: Float32Array.from(builds),
      sectionDrops: Uint8Array.from(drops),
    };
  }
}

// As a worker: the song's channels in, a chunk at a time, then its analysis out, its arrays handed over.
type Chunk = { readonly left: Float32Array; readonly right: Float32Array; readonly rate: number; readonly done?: boolean };
const scope = globalThis as unknown as {
  addEventListener?: (type: "message", listener: (event: MessageEvent<Chunk>) => void) => void;
  postMessage?: (message: unknown, transfer: Transferable[]) => void;
  document?: unknown;
};
if (scope.document === undefined && scope.addEventListener && scope.postMessage) {
  const post = scope.postMessage.bind(scope);
  let analyser: Analyser | null = null;
  scope.addEventListener("message", (event) => {
    analyser ??= new Analyser(event.data.rate);
    analyser.push(event.data.left, event.data.right);
    if (!event.data.done) return;
    const data = analyser.finish();
    analyser = null;
    const buffers = Object.values(data)
      .filter((each): each is Float32Array | Int8Array | Uint8Array => ArrayBuffer.isView(each))
      .map((each) => each.buffer as ArrayBuffer);
    post(data, buffers);
  });
}
