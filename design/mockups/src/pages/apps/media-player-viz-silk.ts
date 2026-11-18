// Silk, a DRAFT visualiser: sheets of light in the wallpaper's colours folding through each other,
// each a soft veil with bright filaments drawn along it, the far ones dimmer and slower, so there
// is depth. The bass swells the sheets, the highs brighten their threads, a drop opens them out.
// The last frame lingers softly behind, so the folds blur like silk moving.

import { COMMON, glsl, type Visual } from "./media-player-gl.ts";

// Its tunables.
/** How many sheets, front to back. */
const SHEETS = 6;
/** A sheet's width at rest and on the bass, in the screen's heights. */
const WIDTH = 0.09;
const WIDTH_BASS = 0.16;
/** A filament's width, at rest and on the highs. */
const THREAD = 0.0035;
const THREAD_HIGH = 0.004;
/** How far a sheet folds, and how fast it drifts. */
const FOLD = 0.9;
const DRIFT = 0.035;
/** How much of the last frame stays, and how it drifts. */
const LINGER = 0.86;

const fragment = /* glsl */ `${COMMON}
in vec2 vUv;
out vec4 o;
void main() {
  vec2 p = (vUv - 0.5) * vec2(uRes.x / uRes.y, 1.0) * 2.0;
  float t = uTime;
  vec3 col = vec3(0.0);
  for (int k = 0; k < ${SHEETS}; k++) {
    float fk = float(k);
    float depth = 1.0 + fk * 0.28;
    vec2 q = p * depth;
    float x = q.x + t * ${glsl(DRIFT)} * (1.0 + fk * 0.4) * (mod(fk, 2.0) < 0.5 ? 1.0 : -0.7);
    // Its centre line: long waves, a slow fold from the noise, and the song's own sway.
    float y = sin(x * 0.8 + t * 0.19 + fk * 1.9) * 0.32
      + sin(x * 1.9 - t * 0.11 + fk * 2.7) * 0.14
      + (fbm(vec2(x * 0.45 + fk * 4.0, t * 0.04)) - 0.5) * ${glsl(FOLD)}
      + (fk - float(${SHEETS} - 1) * 0.5) * (0.16 + 0.1 * uWeight)
      + wave(fract(x * 0.05 + 0.5)) * 0.06 * uLevel;
    float band = spec(0.05 + fk / float(${SHEETS}) * 0.85);
    float w = ${glsl(WIDTH)} + ${glsl(WIDTH_BASS)} * uBass * (1.0 - fk * 0.1) + 0.05 * fbm(vec2(x * 1.6, t * 0.3 + fk));
    float d = q.y - y;
    // The veil, brighter on the side the light comes from.
    float veil = exp(-pow(d / w, 2.0)) * (0.5 + 0.5 * smoothstep(-w, w, d));
    // Its threads: lines running along it, each its own wobble.
    float threads = 0.0;
    for (int j = 0; j < 4; j++) {
      float fj = float(j);
      float off = (fj - 1.5) * w * 0.45 + sin(x * (3.0 + fj) + t * (0.6 + fj * 0.2) + fk) * w * 0.25;
      float tw = ${glsl(THREAD)} + ${glsl(THREAD_HIGH)} * uHigh;
      threads += exp(-pow((d - off) / tw, 2.0)) * (0.4 + 0.6 * vnoise(vec2(x * 3.0 + fj * 7.0, t * 0.5)));
    }
    float hue = 0.5 + 0.5 * sin(fk * 1.37 + x * 0.35 + t * 0.06);
    vec3 c = mix(uPrimary, uCold, hue);
    float bright = (0.3 + 0.9 * band + 0.5 * uKick * (fk < 2.0 ? 1.0 : 0.3)) / (depth * depth);
    col += c * (veil * 0.28 + threads * 0.55) * bright;
  }
  o = vec4(col, 1.0);
}
`;

export const SILK: Visual = {
  kind: "silk",
  name: "Silk",
  scene: { draw: "fragment", fragment },
  blend: "add",
  feedback: (a) => ({ zoom: 1.0015 + a.kick * 0.004, turn: 0, fade: LINGER, blur: 2.2, shift: [0, 0.0006] }),
  bloom: 1.25,
  exposure: 1.05,
};
