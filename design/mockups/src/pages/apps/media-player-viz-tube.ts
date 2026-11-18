// Tube, a DRAFT visualiser: the waveform as a neon tube, its core white-hot, its glass the
// accent and its glow round it, bending along a slow figure that never closes the same way twice;
// a second tube in the cold accent shivers with the highs. The last frame streams outward behind
// them, so each tube leaves a tunnel of its own afterglow, pushed on every kick.

import { COMMON, glsl, type Visual } from "./media-player-gl.ts";

// Its tunables.
/** Segments along each tube. */
const SEGMENTS = 220;
/** How wide a tube's glow reaches, in the screen's heights, at rest and on the bass. */
const GLOW = 0.022;
const GLOW_BASS = 0.016;
/** The core's width, a share of the glow's. */
const CORE = 0.07;
/** How far the waveform bends the tube, at rest and with the level. */
const BEND = 0.03;
const BEND_LEVEL = 0.13;
/** The tunnel: the stream outward a frame, and the kick's push. */
const STREAM = 1.008;
const PUSH = 0.016;
/** How much of the last frame stays. */
const AFTERGLOW = 0.9;

const vertex = /* glsl */ `${COMMON}
out float vAcross;
out float vTube;
out float vGlow;

// The figure a tube follows: two slow Lissajous loops drifting between their ratios, so it never
// closes the same way twice. 'tube' 0 is the main one, 1 the cold one inside it.
vec2 figure(float s, float tube) {
  float t = uTime;
  float a = 1.0 + floor(fbm(vec2(t * 0.012, 1.0 + tube)) * 3.0 + 0.5);
  float b = 2.0 + floor(fbm(vec2(t * 0.009, 7.0 + tube)) * 3.0 + 0.5);
  float ease = smoothstep(0.0, 1.0, fract(t * 0.02));
  float th = s * 2.0 * PI;
  vec2 loop = vec2(sin(th * a + t * 0.11 + tube * 2.0), sin(th * b + t * 0.07));
  vec2 ring = vec2(cos(th + t * 0.05), sin(th + t * 0.05));
  vec2 p = mix(ring, loop, 0.55 + 0.25 * sin(t * 0.03 + tube)) * vec2(0.95, 0.62) * (tube > 0.5 ? 0.62 : 1.0);
  float c = cos(t * (tube > 0.5 ? -0.04 : 0.025)), si = sin(t * (tube > 0.5 ? -0.04 : 0.025));
  return mat2(c, -si, si, c) * p;
}

vec2 bent(float s, float tube) {
  vec2 p = figure(s, tube);
  vec2 q = figure(s + 0.002, tube);
  vec2 n = normalize(vec2(-(q - p).y, (q - p).x));
  // The waveform read there and back, so the loop closes without a seam.
  float w = wave(abs(fract(s * 2.0) * 2.0 - 1.0));
  float amount = tube > 0.5
    ? (fbm(vec2(s * 40.0, uTime * 6.0)) - 0.5) * uHigh * 0.22
    : w * (${glsl(BEND)} + ${glsl(BEND_LEVEL)} * uLevel) + sin(s * 2.0 * PI * 9.0 + uTime * 2.0) * 0.02 * uMid;
  return p + n * amount;
}

void main() {
  int segment = gl_VertexID / 6;
  int corner = gl_VertexID % 6;
  float tube = segment >= ${SEGMENTS} ? 1.0 : 0.0;
  float i = float(segment % ${SEGMENTS});
  float s0 = i / float(${SEGMENTS});
  float s1 = (i + 1.0) / float(${SEGMENTS});
  vec2 p0 = bent(s0, tube);
  vec2 p1 = bent(s1, tube);
  vec2 dir = normalize(p1 - p0 + 1e-6);
  vec2 n = vec2(-dir.y, dir.x);
  float glow = (${glsl(GLOW)} + ${glsl(GLOW_BASS)} * uBass) * (tube > 0.5 ? 0.6 : 1.0);
  // Two triangles over the segment, long enough past its ends to round the joints.
  const vec2 CORNERS[6] = vec2[6](vec2(0.0, -1.0), vec2(0.0, 1.0), vec2(1.0, -1.0), vec2(1.0, -1.0), vec2(0.0, 1.0), vec2(1.0, 1.0));
  vec2 k = CORNERS[corner];
  vec2 along = k.x < 0.5 ? p0 - dir * glow : p1 + dir * glow;
  float side = k.y;
  vec2 p = along + n * glow * side;
  vAcross = side;
  vTube = tube;
  vGlow = glow;
  gl_Position = vec4(p.x * uRes.y / uRes.x, p.y, 0.0, 1.0);
}
`;

const fragment = /* glsl */ `${COMMON}
in float vAcross;
in float vTube;
in float vGlow;
out vec4 o;
void main() {
  float d = abs(vAcross);
  vec3 tone = vTube > 0.5 ? uCold : uPrimary;
  float lit = vTube > 0.5 ? 0.25 + 1.1 * uHigh : 0.45 + 0.6 * uLevel + 0.5 * uKick;
  float core = smoothstep(${glsl(CORE)}, 0.0, d);
  float glass = exp(-pow(d / 0.28, 2.0));
  float glow = exp(-d * 4.0) * 0.55;
  vec3 c = mix(tone, vec3(1.0), 0.45) * core * 1.3 + tone * glass * 0.9 + tone * glow * 0.8;
  o = vec4(c * lit, 1.0);
}
`;

export const TUBE: Visual = {
  kind: "tube",
  name: "Tube",
  scene: { draw: "strips", vertex, fragment, count: SEGMENTS * 2 },
  blend: "max",
  feedback: (a, time) => ({ zoom: STREAM + a.kick * PUSH, turn: 0.0025 * Math.sin(time * 0.07), fade: AFTERGLOW, blur: 1, shift: [0, 0] }),
  bloom: 1.2,
  exposure: 1.05,
};
