// Water, a DRAFT visualiser: travelling down a tunnel of light under water. Its wall is bundles of
// strings of light, each bundle a leading string with followers trailing beside it, in three
// families that each listen to their own part of the sound: a few broad deep ones for the bass, the
// main ones for the mids, many fine ones for the highs. An undulation runs along every bundle toward
// us, its height from the loudness and the family's band; a bass hit shakes every string and sends a
// ring rushing in past us that swells them; the tunnel bends in long slow curves and rolls. Specks
// of light hang in the water. A drop lands as a flash and holds bright and big.
// Everything is the sound of this instant and the onsets already heard: nothing reads ahead, so a
// file, a stream and live sound draw alike.
//
// Drawn in one pass of strips: a quad of background, then every string as a ribbon of quads worked
// out in the vertex shader, and the specks. `docs/design/visualiser.md` specifies it.

import { COMMON, type Flight, glsl, type Visual } from "./media-player-gl.ts";
import type { Analysis, Signal } from "./media-player-signal.ts";
import { settings } from "../../site/settings.ts";

// Its tunables.
/** The view: screen heights per world unit at a depth of one, and the depths drawn, units. */
const FOCAL = 1.6667;
const NEAR = 0.3;
const FAR = 34;
/** Segments along each string, evenly in log-depth. */
const SEGMENTS = 160;
/**
 * The three families, deep (bass), main (mids) and fine (highs): bundles, strings a bundle, the
 * angle between neighbours, radians, their girth, the wall's radius they sit at, the undulation's
 * height, its wavelength's and its speed's scale, how far a ring swells them, and their light.
 */
const FAMILIES = [
  { bundles: 6, strings: 7, gap: 0.052, girth: 1.55, radius: 1.14, height: 1.1, length: 0.86, run: 0.9, ring: 1.25, light: 0.9 },
  { bundles: 12, strings: 8, gap: 0.04, girth: 1.0, radius: 1.0, height: 1.0, length: 1.0, run: 1.0, ring: 1.0, light: 1.0 },
  { bundles: 16, strings: 4, gap: 0.03, girth: 0.5, radius: 0.88, height: 0.9, length: 1.18, run: 1.12, ring: 0.7, light: 0.6 },
] as const;
/** The theme's colours as the strings wear them: this share of the way to white, so light, not paint. */
const PALE = 0.35;
/** One bundle in this many takes the second colour. */
const SECOND_EVERY = 4;
/** A string's girth, world units at depth one before its swell; the least, pixels. */
const GIRTH = 0.019;
/** A string's swell in girth from its family's band: at rest and what the band adds. */
const GIRTH_BAND = [0.45, 0.6] as const;
const LEAST_PX = 0.55;
/** A follower's girth and light against the leader's, and how each further one thins and dims; its lag, radians. */
const FOLLOWER_GIRTH = 0.62;
const FOLLOWER_LIGHT = 0.78;
const FOLLOWER_FADE = 0.9;
const LAG = 0.045;
/** The white-hot core: its width in girths and its light, the leader's and a follower's. */
const CORE_LEAD = [0.42, 0.8] as const;
const CORE_FOLLOW = [0.3, 0.3] as const;
/** The undulation's height: at rest, from the loudness, the family's band and a bass hit, radians. */
const SWING = [0.05, 0.15, 0.2, 0.06] as const;
/**
 * The vibration: its rate, Hz, and its depth on a bass hit and from the highs, radians. Only an
 * answer to sound: at silence it is none, and as the music stops it eases out with the song's energy.
 */
const SHAKE_HZ = 13;
const SHAKE = [0.015, 0.005] as const;
/** The wall breathing in and out with the family's band: at rest and what the band adds, units. */
const BREATHE = [0.03, 0.11] as const;
/** The strings' light: at rest, from the energy, the family's band, and a bass hit for the deep family. */
const LIT = [0.18, 0.12, 0.24, 0.15] as const;
/** A ring: born this deep at the kick, rushing in at this speed, units a second; how long it lives, s; how far it pulls the strings in. */
const RING_FROM = 11;
const RING_SPEED = 17;
const RING_LIFE = 70 / 60;
const RING_PULL = 0.17;
/** How far a ring swells the strings' girth where it is, at its full strength, times their family's `ring`. */
const RING_SWELL = 1.9;
/**
 * A ring's light: the strings where it is are this much brighter at its full strength; at its birth
 * this much more again, over this much more of the tunnel's depth, units, both falling over
 * `RING_BIRTH_FALL` s. The kick reads as the ring's birth, wide and bright deep in the tunnel, on the
 * kick's own frame; its travel in is the aftermath.
 */
const RING_LIGHT = 0.45;
const RING_BIRTH = 5;
const RING_BIRTH_WIDTH = 3;
const RING_BIRTH_FALL = 0.1;
/**
 * A string's swell in girth and its light before a drop's flash: each as it is up to its knee,
 * bending past it toward the most it may reach. The deep, main and fine families each have their
 * own most, so at the loudest the lows still read heavier and brighter than the mids, and the mids
 * than the highs.
 */
const SWELL_KNEE = 1.2;
const SWELL_MOST = [1.8, 1.5, 1.3] as const;
const LIGHT_KNEE = [0.8, 0.5, 0.3] as const;
const LIGHT_MOST = [1.3, 0.8, 0.5] as const;
/** The golden ratio's steps down: what the main and the fine families take of what the deep one does, and the room the three bands share. */
const STEP = [1, 0.618, 0.382] as const;
const BAND_ROOM = 1.618;
/**
 * A kick's hit across the whole tunnel: at its full strength on the kick's own frame, falling over
 * `HIT_FALL` s; it swells every string's girth by `HIT_GIRTH`, adds `HIT_LIGHT` to the deep family's
 * light and the golden steps of it to the others' (`STEP`), and shakes them. Nothing rises: only
 * the fall is shaped.
 */
const HIT_FALL = 0.15;
const HIT_GIRTH = 0.6;
const HIT_LIGHT = 0.18;
/** A kick's ring and hit by the bass of its passage: the most of it over the moments before it and at it, s. */
const PASSAGE = [0, 0.1, 0.2, 0.3, 0.4] as const;
/** The tunnel's bends: how deep at rest and what the energy adds; its roll, radians a second. */
const BEND = [0.22, 0.42] as const;
const ROLL = 0.045;
/** The travel, units a second at rest and from the energy, at the tempo below; the undulation's speed, Hz, at rest and from the energy. */
const TRAVEL = [2.2, 4.4] as const;
const TEMPO = 175;
const UNDULATE = [0.4, 0.7, 0.5] as const;
/** With reduce motion on: the travel, units a second, and the undulation's speed, radians a second. */
const CALM_TRAVEL = 0.6;
const CALM_UNDULATE = 0.6;
/** Specks of light in the water: how many, over how deep a stretch they stream past. */
const MOTES = 110;
const MOTE_SPAN = 30;
/**
 * A speck is a soft round dot with a glow; travelling fast it draws out a soft tapered trail behind
 * it, from nothing at this speed, units a second, to its longest, in its own radii, at this one.
 */
const TRAIL_FROM = 2.5;
const TRAIL_FULL = 6;
const TRAIL_MOST = 3;
/**
 * The drop: what lands as one (a section this heavy, this much heavier than the one before, of 0 to
 * 1, and the step that lands at full); where it lands, on its section's first kick within
 * `DROP_ALIGN` s of the start, or the start; its flash, at full on that frame and falling over
 * `FLASH_FALL` s, and its light; the hold's light and height.
 */
const DROP_HEAVY = 0.85;
const DROP_STEP = 0.05;
const DROP_FULL = 0.25;
const DROP_ALIGN = 0.06;
const FLASH_FALL = 0.9;
const FLASH_LIGHT = 1.6;
const HOLD_LIGHT = 0.35;
const HOLD_HEIGHT = 0.3;
/** The kicks the shader is handed: the last this many of the ring's life. */
const KICKS = 8;
/** The song's energy (1 playing, a breath paused) over which the answers to sound ease in. */
const SOUNDING = [0.3, 0.9] as const;
const DEEP_STRINGS = FAMILIES[0].bundles * FAMILIES[0].strings;
const MAIN_STRINGS = FAMILIES[1].bundles * FAMILIES[1].strings;
const STRINGS = DEEP_STRINGS + MAIN_STRINGS + FAMILIES[2].bundles * FAMILIES[2].strings;
const STRING_QUADS = STRINGS * SEGMENTS;
const FIRST_STRING = 1;
const FIRST_MOTE = FIRST_STRING + STRING_QUADS;
const QUADS = FIRST_MOTE + MOTES;

/** Three numbers, one a family, written as GLSL's vec3. */
const vec3 = (values: readonly [number, number, number]) => `vec3(${values.map(glsl).join(", ")})`;

const reduced = () => settings().reduced;
/** The song's loudness, 0 to 1, as its own range has it. */
const loudness = (a: Analysis) => Math.max(0, Math.min(1, (a.loud - 0.2) / 0.65));
/** The tempo against the one the travel is set for, where the beat grid is trusted. */
const tempo = (a: Analysis) => {
  const trusted = Math.max(0, Math.min(1, (a.confidence - 0.3) / 0.3));
  return Math.sqrt(Math.max(0.6, Math.min(1.25, (TEMPO + (a.bpm - TEMPO) * trusted) / TEMPO)));
};
const speed = (a: Analysis) => (reduced() ? CALM_TRAVEL : (TRAVEL[0] + TRAVEL[1] * loudness(a)) * tempo(a));
const spin = (a: Analysis) => (reduced() ? CALM_UNDULATE : 2 * Math.PI * UNDULATE[0] * (UNDULATE[1] + UNDULATE[2] * loudness(a)));

/**
 * The moments back the slow features (the loudness, the energy) are smoothed over, s, and each
 * one's window: only what has sounded, each weighted less the further back (a triangle). The bands
 * are read as they are now: the analysis's followers rise with the sound at once and fall smoothly,
 * so a kick lights the strings on its own frame.
 */
const BACK = [0, 1 / 30, 2 / 30, 0.1, 0.15, 0.2, 0.3, 0.45, 0.6, 0.8, 1.0, 1.3, 1.6, 2.0, 2.5, 3.0, 3.5, 4.0];
const WINDOW = { level: 1.0, energy: 4.0 } as const;
/** The bass as the strings feel it: nothing under this share of the song's range, full this far above it. */
const BASS_FLOOR = 0.25;
const BASS_RANGE = 0.7;
/** A kick rings by the bass under it, felt as above: from this to this a ring grows from nothing to its strength. */
const RING_BASS = [0.1, 0.5] as const;
const felt = (bass: number) => Math.max(0, Math.min(1, (bass - BASS_FLOOR) / BASS_RANGE));
const between = (x: number, [from, to]: readonly [number, number]) => {
  const t = Math.max(0, Math.min(1, (x - from) / (to - from)));
  return t * t * (3 - 2 * t);
};

/** The loudness and the energy over the moments before now, each over its own window. */
function slow(signal: Signal, a: Analysis): { level: number; energy: number } {
  const sums = { level: 0, energy: 0 };
  const weights = { level: 0, energy: 0 };
  for (const back of BACK) {
    const then = back === 0 ? a : signal.analyse(Math.max(0, a.time - back));
    const value = { level: then.level, energy: loudness(then) };
    for (const key of Object.keys(sums) as (keyof typeof sums)[]) {
      const weight = Math.max(0, 1 - back / WINDOW[key]);
      sums[key] += value[key] * weight;
      weights[key] += weight;
    }
  }
  return { level: sums.level / weights.level, energy: sums.energy / weights.energy };
}

/** A drop landing at the section playing now, as far as it has played: its flash and its hold. */
function drop(signal: Signal, time: number): { flash: number; hold: number } {
  const span = signal.section(time);
  if (span.start <= 0) return { flash: 0, hold: 0 };
  // Each section's weight read at its own start, which has sounded.
  const now = signal.analyse(span.start).weight;
  const before = signal.analyse(signal.section(span.start - 0.01).start).weight;
  const step = now - before;
  if (now < DROP_HEAVY || step < DROP_STEP) return { flash: 0, hold: 0 };
  const strength = 0.5 + 0.5 * Math.min(1, step / DROP_FULL);
  // It lands on the section's first kick, the moment the drop is heard, where one is that near.
  const kick = signal.kicks(span.start, span.start + DROP_ALIGN)[0];
  const since = time - (kick?.time ?? span.start);
  if (since < 0) return { flash: 0, hold: 0 };
  return { flash: strength * Math.exp(-since / FLASH_FALL), hold: strength };
}

/**
 * The three bands as the families wear them, so a full passage is not every string at its most: the
 * band that leads keeps its level and each other one steps back by how far it trails it, and
 * together they never pass `BAND_ROOM`.
 */
function shared(bands: readonly [number, number, number]): number[] {
  const lead = Math.max(...bands);
  if (lead <= 0) return [0, 0, 0];
  const stepped = bands.map((each) => (each * each) / lead);
  const all = stepped.reduce((sum, each) => sum + each, 0);
  return stepped.map((each) => each * Math.min(1, BAND_ROOM / all));
}

/**
 * What the shader is handed each frame: the energy, the loudness, the three bands as the families
 * share them, a bass hit, a drop's flash and hold, whether to keep calm, the travel's speed, the
 * last kicks' ages and strengths.
 */
function extras(a: Analysis, flight: Flight): number[] {
  // What sounds: none paused, where the song's energy is only a breath, easing in and out with it,
  // so nothing answers to sound (the shake, the rings) when nothing plays.
  const live = reduced() ? 0 : between(flight.energy, SOUNDING);
  const heard = slow(flight.signal, a);
  const { flash, hold } = drop(flight.signal, a.time);
  const recent = flight.signal.kicks(a.time - RING_LIFE, a.time).slice(-KICKS).reverse();
  const ages = recent.map((k) => a.time - k.time);
  // A ring's strength, and its kick's hit: the kick's, by the bass of its passage up to it, so a light
  // kick barely rings; known at the kick's own moment, so both are at full on its frame.
  const strengths = recent.map((k) => k.strength * between(Math.max(...PASSAGE.map((back) => felt(flight.signal.analyse(Math.max(0, k.time - back)).bass))), RING_BASS) * live);
  const hit = recent.reduce((most, k, i) => Math.max(most, strengths[i]! * Math.exp(-(a.time - k.time) / HIT_FALL)), 0);
  while (ages.length < KICKS) {
    ages.push(-1);
    strengths.push(0);
  }
  const energy = heard.energy * flight.energy;
  return [
    energy,
    heard.level * flight.energy,
    // The bass, and the hit: at least the kick's own from its moment, as the band itself rises over a
    // frame or so after the kick's attack.
    ...shared([Math.max(felt(a.bass) * live, hit), a.mid * live, a.high * live]),
    Math.max(felt(a.bass) * live, hit),
    flash * live,
    hold * live,
    reduced() ? 1 : 0,
    // The speed travelled now: paused, the song's clock creeps, and the travel with it.
    speed(a) * (0.12 + 0.88 * flight.energy),
    ...ages,
    ...strengths,
  ];
}

const vertex = /* glsl */ `${COMMON}
flat out int vKind;
out vec4 vA;
out vec4 vB;
out vec3 vColour;

const float TAU = 6.2832;
float h1(float n) { return fract(sin(n * 127.1 + 311.7) * 43758.5453); }

// The music as extras() hands it over.
float E, LOUD, BASS, MID, HIGH, HIT, FLASH, HOLD;
float T, S, PH1, PH2, ROLLED, BENDK;
vec2 P0, DP;

vec2 pathAt(float w) { return vec2(sin(w / 9.0 + 1.0) + 0.6 * sin(w / 5.3 + 2.0), sin(w / 7.7 + 4.0) + 0.6 * sin(w / 4.6 + 0.5)); }
// Where the tunnel's middle is at a depth ahead, against the line we look along.
vec2 offAt(float z) { return BENDK * (pathAt(S + z) - P0 - z * DP); }
// A string's light by depth: soft beside us, full a little ahead, gone long before the far end.
float depthLight(float z) { return min(1.0, 0.45 + 0.4 * z) * min(1.0, exp(-(z - 9.0) / 7.5)); }
vec2 toClip(vec2 p, float z) { return vec2(p.x * uRes.y / uRes.x, p.y) * ${glsl(FOCAL)} / z; }
float pixel() { return 2.0 / uRes.y; }

// The rings at a depth, each born at its kick a way down the tunnel and rushing in past us: how
// they swell the strings there, and how they light them, brightest at birth.
vec2 ringAt(float z) {
  vec2 sum = vec2(0.0);
  for (int k = 0; k < ${KICKS}; k++) {
    float age = uExtra[10 + k];
    if (age < 0.0) continue;
    float at = ${glsl(RING_FROM)} - ${glsl(RING_SPEED)} * age;
    float width = 0.5 + 0.1 * max(0.0, at);
    float d = (z - at) / width;
    float birth = exp(-age / ${glsl(RING_BIRTH_FALL)});
    float lit = (z - at) / (width + ${glsl(RING_BIRTH_WIDTH)} * birth);
    sum += uExtra[18 + k] * vec2(exp(-d * d), exp(-lit * lit) * (1.0 + ${glsl(RING_BIRTH)} * birth));
  }
  return min(sum, vec2(1.3, 8.0));
}

// A string: its family, bundle and place in it, and the bundle's own numbers.
struct Str { int family; float b; float s; float n; float gap; float girth; float radius; float height; float len; float run; float ring; float light; float th0; float p1; float p2; float k; float p3; float own; float rate; bool second; };
Str stringOf(int index) {
  Str c;
  int f = index < ${DEEP_STRINGS} ? 0 : index < ${DEEP_STRINGS + MAIN_STRINGS} ? 1 : 2;
  int local = index - (f == 0 ? 0 : f == 1 ? ${DEEP_STRINGS} : ${DEEP_STRINGS + MAIN_STRINGS});
  int per = f == 0 ? ${FAMILIES[0].strings} : f == 1 ? ${FAMILIES[1].strings} : ${FAMILIES[2].strings};
  c.family = f;
  c.b = float(local / per);
  c.s = float(local % per);
  ${FAMILIES.map((F, i) => `if (f == ${i}) { c.n = ${glsl(F.bundles)}; c.gap = ${glsl(F.gap)}; c.girth = ${glsl(F.girth)}; c.radius = ${glsl(F.radius)}; c.height = ${glsl(F.height)}; c.len = ${glsl(F.length)}; c.run = ${glsl(F.run)}; c.ring = ${glsl(F.ring)}; c.light = ${glsl(F.light)}; }`).join("\n  ")}
  float seed = float(f) * 97.0 + c.b * 7.31;
  float share = c.b / c.n;
  // Where it bends is its own: scattered, part random and part winding round the tunnel.
  c.th0 = share * TAU + h1(seed) * 0.1 + c.radius * 2.0;
  c.p1 = 2.0 * share * TAU + h1(seed + 1.0) * 1.8;
  c.p2 = -3.0 * share * TAU + h1(seed + 2.0) * 2.2;
  c.k = 0.88 + h1(seed + 3.0) * 0.24;
  c.p3 = 0.5 + h1(seed + 4.0) * 0.4;
  c.own = 0.8 + h1(seed + 5.0) * 0.4;
  c.rate = 0.7 + h1(seed + 6.0) * 0.6;
  c.second = mod(c.b, ${glsl(SECOND_EVERY)}) == 2.0;
  return c;
}
float bandOf(int f) { return f == 0 ? BASS : f == 1 ? MID : HIGH; }
// A family's own of three numbers, the deep's, the main's and the fine's.
float byFamily(int f, vec3 v) { return f == 0 ? v.x : f == 1 ? v.y : v.z; }

// A string's point at a depth, on screen, and its girth there, pixels-high units of the clip.
vec3 stringAt(Str c, float z, out float width) {
  float own = bandOf(c.family);
  float near = 1.0 - exp(-z / 1.4);
  vec2 ring = ringAt(z);
  float pulse = ring.x;
  float lag = c.s * ${glsl(LAG)};
  float swing = c.own * c.height * (${glsl(SWING[0])} + ${glsl(SWING[1])} * LOUD + ${glsl(SWING[2])} * own + ${glsl(SWING[3])} * HIT) * (1.0 + ${glsl(HOLD_HEIGHT)} * HOLD)
    * near * min(1.0, exp(-(z - 9.0) / 11.0)) * (0.78 + 0.22 * sin(0.33 * c.k * z - 0.9 * PH1 + c.p2 * 1.7));
  float drift = 0.5 * sin(T * 0.45 * c.rate + c.p3 * 9.0);
  float calm = uExtra[8];
  float shake = (1.0 - calm) * near * (${glsl(SHAKE[0])} * HIT + ${glsl(SHAKE[1])} * HIGH)
    * sin(TAU * ${glsl(SHAKE_HZ)} * T + z * 3.0 + c.s * 0.22 + c.p3 * 7.0) * (0.6 + 0.4 * sin(z * 0.9 + c.p1 * 5.0));
  float th = c.th0 + c.s * c.gap + ROLLED + 0.035 * z
    + swing * sin(0.55 * c.len * c.k * z + 1.7 * c.run * PH1 + drift - lag + c.p1)
    + swing * 0.5 * sin(1.45 * c.len * (2.0 - c.k) * z + 1.15 * c.run * PH2 + drift * 1.3 - lag * 1.5 + c.p2)
    + shake;
  float r = c.radius + (${glsl(BREATHE[0])} + ${glsl(BREATHE[1])} * own) * sin(0.7 * c.len * c.k * z + 1.4 * c.run * PH1 + drift - lag + c.p3)
    - ${glsl(RING_PULL)} * c.ring * pulse * c.own + shake * 0.8;
  float girth = c.girth * (c.s == 0.0 ? 1.0 : ${glsl(FOLLOWER_GIRTH)} * pow(${glsl(FOLLOWER_FADE)}, c.s));
  // The band, the hit and the rings multiply up on a kick as the light does: past the knee the
  // swell bends toward the most a string may fatten, so the strings stay strings.
  float swell = (${glsl(GIRTH_BAND[0])} + ${glsl(GIRTH_BAND[1])} * own) * (1.0 + ${glsl(RING_SWELL)} * c.ring * pulse) * (1.0 + ${glsl(HIT_GIRTH)} * HIT);
  float fatMost = byFamily(c.family, ${vec3(SWELL_MOST)}) - ${glsl(SWELL_KNEE)};
  if (swell > ${glsl(SWELL_KNEE)}) swell = ${glsl(SWELL_KNEE)} + fatMost * (1.0 - exp(-(swell - ${glsl(SWELL_KNEE)}) / fatMost));
  width = max(${glsl(LEAST_PX)} * pixel(), ${glsl(GIRTH)} * girth * swell * ${glsl(FOCAL)} / z);
  vec2 world = vec2(cos(th), sin(th)) * r + offAt(z);
  return vec3(toClip(world, z), ring.y);
}

// A mote's place and the place it streams from a moment later, at a depth.
vec2 moteAt(float a, float r, float z) {
  float ang = a + ROLLED + 0.035 * z;
  return toClip(vec2(cos(ang), sin(ang)) * r + offAt(z), z);
}

// A soft speck at its head on screen, its radius in the clip's heights, its trail so many radii long
// toward where it was; the quad reaches out to its glow.
void speck(vec2 head, vec2 behind, float radius, float trail, vec2 k) {
  vec2 px = vec2(uRes.x / uRes.y, 1.0);
  vec2 d = (behind - head) * px;
  vec2 dir = length(d) > 1e-7 ? normalize(d) : vec2(1.0, 0.0);
  vec2 n = vec2(-dir.y, dir.x);
  float reach = 3.0;
  float along = k.x < 0.5 ? -reach : trail + reach;
  vec2 at = head * px + dir * along * radius + n * reach * radius * k.y;
  gl_Position = vec4(at / px, 0.0, 1.0);
  // Its place behind the head and across it, in radii, the trail's length, and its radius in pixels.
  vB = vec4(along, reach * k.y, trail, radius / pixel());
}

void main() {
  E = uExtra[0]; LOUD = uExtra[1]; BASS = uExtra[2]; MID = uExtra[3]; HIGH = uExtra[4]; HIT = uExtra[5]; FLASH = uExtra[6]; HOLD = uExtra[7];
  T = uTime;
  S = uTravel;
  PH1 = uSpin;
  PH2 = uSpin * 2.75;
  ROLLED = ${glsl(ROLL)} * T;
  BENDK = ${glsl(BEND[0])} + ${glsl(BEND[1])} * E;
  P0 = pathAt(S);
  DP = (pathAt(S + 0.01) - P0) / 0.01;
  int quad = gl_VertexID / 6;
  int corner = gl_VertexID % 6;
  const vec2 CORNERS[6] = vec2[6](vec2(0.0, -1.0), vec2(0.0, 1.0), vec2(1.0, -1.0), vec2(1.0, -1.0), vec2(0.0, 1.0), vec2(1.0, 1.0));
  vec2 k = CORNERS[corner];
  vA = vec4(0.0);
  vB = vec4(0.0);
  vColour = vec3(0.0);

  if (quad == 0) {
    // The water behind: the whole frame, darkest where the tunnel leads.
    vKind = 0;
    gl_Position = vec4(k.x * 2.0 - 1.0, k.y, 0.0, 1.0);
    vA = vec4(toClip(offAt(${glsl(FAR)}), ${glsl(FAR)}), 0.0, 0.0);
    return;
  }

  if (quad < ${FIRST_MOTE}) {
    vKind = 1;
    int q = quad - ${FIRST_STRING};
    Str c = stringOf(q / ${SEGMENTS});
    float i = float(q % ${SEGMENTS});
    // This corner's depth, its point, and the string's heading there from a step further: the
    // neighbouring segment works out the same corner the same way, so they meet without a seam.
    float z = ${glsl(NEAR)} * pow(${glsl(FAR / NEAR)}, (i + k.x) / ${glsl(SEGMENTS)});
    float w;
    float wNext;
    vec3 a = stringAt(c, z, w);
    vec3 b = stringAt(c, z * ${glsl(Math.pow(FAR / NEAR, 0.5 / SEGMENTS))}, wNext);
    vec2 px = vec2(uRes.x / uRes.y, 1.0);
    vec2 dir = normalize((b.xy - a.xy) * px + 1e-7);
    vec2 n = vec2(-dir.y, dir.x);
    // The ribbon reaches out to the string's glow, two and a half girths each side.
    float half_ = w * 2.6;
    vec2 at = a.xy * px + n * half_ * k.y;
    gl_Position = vec4(at / px, 0.0, 1.0);
    float own = bandOf(c.family);
    float follower = c.s == 0.0 ? 1.0 : ${glsl(FOLLOWER_LIGHT)} * pow(${glsl(FOLLOWER_FADE)}, c.s);
    float lit = c.light * (${glsl(LIT[0])} + ${glsl(LIT[1])} * E + ${glsl(LIT[2])} * own + ${glsl(HIT_LIGHT)} * HIT * byFamily(c.family, ${vec3(STEP)}) + (c.family == 0 ? ${glsl(LIT[3])} * HIT : 0.0)) * follower;
    lit *= 1.0 + ${glsl(RING_LIGHT)} * a.z;
    // The band, the hit and the rings multiply up on a kick: past the knee their sum bends toward a
    // ceiling, so a kick lands at once and bright without burning the strings out to white.
    float knee = byFamily(c.family, ${vec3(LIGHT_KNEE)});
    float most = byFamily(c.family, ${vec3(LIGHT_MOST)}) - knee;
    if (lit > knee) lit = knee + most * (1.0 - exp(-(lit - knee) / most));
    lit *= (1.0 + ${glsl(FLASH_LIGHT)} * FLASH + ${glsl(HOLD_LIGHT)} * HOLD) * depthLight(z);
    vec3 base = mix(c.second ? uPrimary : uCold, vec3(1.0), ${glsl(PALE)});
    // Deep and fine families in deeper and paler shades of the same colours.
    vec3 shade = c.family == 0 ? pow(base, vec3(1.4)) : c.family == 2 ? mix(base, vec3(1.0), 0.45) : base;
    vColour = shade * lit;
    float coreWidth = c.s == 0.0 ? ${glsl(CORE_LEAD[0])} : ${glsl(CORE_FOLLOW[0])};
    float coreLight = c.s == 0.0 ? ${glsl(CORE_LEAD[1])} : ${glsl(CORE_FOLLOW[1])};
    // The fragment measures its own distance from the middle line, in pixels: a coordinate
    // interpolated across a long thin quad would kink at its diagonal. The core is one girth wide,
    // the white-hot middle a share of one.
    vA = vec4(w / pixel(), coreWidth * 0.5, coreLight * lit, 0.0);
    vB = vec4((a.xy * 0.5 + 0.5) * uRes, n);
    return;
  }

  // A speck of light hanging in the water, streaming past; it glints with the highs.
  vKind = 2;
  float m = float(quad - ${FIRST_MOTE});
  float w0 = h1(m * 3.1) * ${glsl(MOTE_SPAN)};
  float a = h1(m * 5.7) * TAU;
  float r = 0.25 + h1(m * 7.3) * 1.5;
  float size = 0.5 + h1(m * 9.1);
  float l = 0.4 + h1(m * 11.9) * 0.6;
  float f = 0.2 + h1(m * 13.3) * 0.5;
  bool warm = h1(m * 17.7) < 0.2;
  float z = mod(w0 - S, ${glsl(MOTE_SPAN)}) + 0.35;
  float rr = r + 0.05 * sin(T * f + a * 7.0);
  vec2 p0 = moteAt(a, rr, z);
  vec2 p1 = moteAt(a, rr, z + 0.5);
  float radius = clamp(size * 0.012 * ${glsl(FOCAL)} / z, 1.2 * pixel(), 5.0 * pixel()) * 0.5;
  float fast = clamp((uExtra[9] - ${glsl(TRAIL_FROM)}) / ${glsl(TRAIL_FULL - TRAIL_FROM)}, 0.0, 1.0);
  speck(p0, p1, radius, ${glsl(TRAIL_MOST)} * fast * (1.0 - uExtra[8]), k);
  vColour = (warm ? mix(uPrimary, vec3(1.0), 0.45) : mix(uCold, vec3(1.0), 0.5)) * depthLight(z) * (0.32 + 0.6 * HIGH) * l;
}
`;

const fragment = /* glsl */ `${COMMON}
flat in int vKind;
in vec4 vA;
in vec4 vB;
in vec3 vColour;
out vec4 o;

void main() {
  if (vKind == 0) {
    // Deep dark water: darkest down the tunnel, a little lighter far out, a slow haze in it.
    vec2 p = gl_FragCoord.xy / uRes.y * 2.0 - vec2(uRes.x / uRes.y, 1.0);
    vec2 lead = vA.xy * vec2(uRes.x / uRes.y, 1.0);
    float d = length(p - lead) / 2.407;
    float t = clamp(d, 0.0, 1.0);
    float depth = t < 0.35 ? mix(0.012, 0.042, t / 0.35) : mix(0.042, 0.085, (t - 0.35) / 0.65);
    vec3 c = uCold * depth;
    for (int i = 0; i < 5; i++) {
      float fi = float(i);
      float a = fi * 1.9 + uTime * 0.05 * (mod(fi, 2.0) < 0.5 ? -1.0 : 1.0);
      float rr = (520.0 + 90.0 * sin(uTime * 0.07 + fi)) / 540.0;
      vec2 at = vec2(cos(a) * rr * 1.5, sin(a) * rr);
      c += uCold * 0.5 * 0.055 * max(0.0, 1.0 - length(p - at) / (520.0 / 540.0));
    }
    o = vec4(c, 1.0);
    return;
  }
  if (vKind == 1) {
    // A string: its colour's crisp core, a soft glow round it, a white-hot middle.
    float girthPx = max(vA.x, 1e-3);
    float u = abs(dot(gl_FragCoord.xy - vB.xy, normalize(vB.zw))) / girthPx;
    float aa = 0.75 / girthPx;
    float core = 1.0 - smoothstep(0.5 - aa, 0.5 + aa, u);
    float glow = 0.12 * exp(-u * u / 0.6) + 0.035 * exp(-u * u / 3.0);
    float white = 1.0 - smoothstep(vA.y - aa, vA.y + aa, u);
    // A string thinner than a pixel keeps its light, spread over the pixel.
    float thin = min(1.0, girthPx);
    o = vec4(vColour * (0.85 * core * thin + glow) + vec3(1.0) * white * vA.z * thin, 1.0);
    return;
  }
  if (vKind == 2) {
    // A speck: a round dot of light its whole width, its edge smoothed over a pixel so the
    // smallest still shows, a soft glow round it; behind it a trail that narrows and fades to
    // nothing at its tail.
    float x = vB.x;
    float trail = vB.z;
    float radiusPx = max(vB.w, 0.5);
    float t = trail > 0.0 ? clamp(x / trail, 0.0, 1.0) : 1.0;
    float width = x > 0.0 ? mix(1.0, 0.35, t) : 1.0;
    float d = length(vec2(x > 0.0 ? 0.0 : x, vB.y / width));
    float fade = x > 0.0 ? (trail > 0.0 ? (1.0 - t) * (1.0 - t) : 0.0) : 1.0;
    float dot_ = length(vB.xy);
    float disc = clamp(radiusPx + 0.5 - dot_ * radiusPx, 0.0, 1.0);
    float tail = clamp(radiusPx * width + 0.5 - d * radiusPx, 0.0, 1.0) * fade;
    float light = max(disc, tail) + 0.3 * exp(-dot_ * 1.2);
    o = vec4(vColour * light, 1.0);
    return;
  }
}
`;

export const WATER: Visual = {
  kind: "water",
  name: "Water",
  scene: { draw: "strips", vertex, fragment, count: QUADS },
  blend: "add",
  speed,
  spin,
  extras,
  // Nothing carried from the last frame: every string is drawn where it is now.
  feedback: () => ({ zoom: 1, turn: 0, fade: 0, blur: 0, shift: [0, 0] }),
  vignette: 0,
  bloom: 0.25,
  exposure: 0.85,
  tone: "hue",
};
