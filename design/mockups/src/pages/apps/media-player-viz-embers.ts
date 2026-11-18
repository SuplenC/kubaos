// Embers, a DRAFT visualiser: tens of thousands of sparks in a slow current. At rest they drift and
// rise like embers off a fire; the music stirs them into spiral arms whose count drifts over
// minutes; a kick throws them outward; a build pulls them into rings that burst on the drop. Each
// spark is lit by its own band, in the accent, a few in the cold accent, and leaves a glowing trail.

import { COMMON, glsl, type Visual } from "./media-player-gl.ts";

// Its tunables.
/** How many sparks. */
const SPARKS = 16000;
/** A spark's size, in pixels at 1080, at rest and with its band. */
const SIZE = 1.8;
const SIZE_BAND = 3.6;
/** The kick's throw outward, a share of a spark's distance from the middle. */
const THROW = 0.2;
/** How far the current carries a spark. */
const CURRENT = 0.3;
/** How much of the last frame stays, the trails' length. */
const TRAIL = 0.93;

const vertex = /* glsl */ `${COMMON}
out vec3 vColour;
void main() {
  float id = float(gl_VertexID);
  float s1 = hash11(id * 1.31 + 0.7);
  float s2 = hash11(id * 7.77 + 3.1);
  float s3 = hash11(id * 3.71 + 9.2);
  float t = uTime;
  float way = s3 > 0.5 ? 1.0 : -1.0;
  float angle = s1 * 2.0 * PI + t * (0.03 + 0.1 * s2) * way;
  float r = 0.12 + 1.15 * pow(s2, 0.75);
  // Spiral arms, their count and their tightness drifting over minutes.
  float arms = 2.0 + floor(fbm(vec2(t * 0.008, 9.0)) * 5.0);
  float twist = 3.0 + 6.0 * fbm(vec2(t * 0.01, 2.0));
  r += sin(angle * arms + r * twist - t * 0.45) * (0.1 + 0.14 * uLevel);
  vec2 p = vec2(cos(angle), sin(angle)) * r;
  // The current: a slow noise field carrying each spark off its orbit.
  vec2 flow = vec2(fbm(p * 1.2 + vec2(t * 0.04, 0.0)), fbm(p * 1.2 + vec2(0.0, t * 0.04) + 5.0)) - 0.5;
  p += flow * ${glsl(CURRENT)};
  // At rest the embers rise.
  p.y += (fract(s3 + t * 0.02) - 0.4) * 0.35 * (1.0 - min(1.0, uLevel * 1.5));
  // A build pulls them into rings; the kick throws them out.
  float shell = (floor(length(p) * 5.0) + 0.5) / 5.0;
  p = mix(p, normalize(p + 1e-5) * shell, uBuild * 0.75);
  p *= 1.0 + uKick * ${glsl(THROW)} * (0.35 + s3);
  gl_Position = vec4(p.x * uRes.y / uRes.x, p.y, 0.0, 1.0);
  float band = spec(s1);
  vColour = (s3 > 0.78 ? uCold : uPrimary) * (0.35 + 1.4 * band + 0.6 * uKick * s3);
  gl_PointSize = (${glsl(SIZE)} + ${glsl(SIZE_BAND)} * s3 * s3 * (0.4 + band)) * uRes.y / 1080.0;
}
`;

const fragment = /* glsl */ `${COMMON}
in vec3 vColour;
out vec4 o;
void main() {
  float d = length(gl_PointCoord - 0.5) * 2.0;
  float a = exp(-d * d * 3.5);
  o = vec4(vColour * a, 1.0);
}
`;

export const EMBERS: Visual = {
  kind: "embers",
  name: "Embers",
  scene: { draw: "points", vertex, fragment, count: SPARKS },
  blend: "add",
  feedback: (a, time) => ({ zoom: 1.0015 + a.kick * 0.006, turn: 0.004 * Math.sin(time * 0.04) + 0.003, fade: TRAIL, blur: 0.8, shift: [0, 0.0003] }),
  bloom: 1.4,
  exposure: 1.4,
};
