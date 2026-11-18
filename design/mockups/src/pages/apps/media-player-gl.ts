// The visualisers' engine, a DRAFT: WebGL2 drawing a song's visualiser behind the player's
// controller, at the display's rate, from the time alone (`media-player-signal.ts` says what the
// music does at it). Every frame is three passes:
//
// 1. Feedback: the last frame drawn again, a little zoomed, turned, shifted, blurred and faded, so
//    light leaves trails: the trick behind the old players' visualisations.
// 2. The scene, added on top: a full-screen fragment shader, thousands of points, or tubes of
//    light, each visualiser's own (`media-player-viz-*.ts`).
// 3. Present: the frame to the screen with its bloom, its tone, a vignette and a dither that keeps
//    dark gradients from banding.
//
// Its colours are the wallpaper's (the night scheme's `primary` and `cold`) or the cover's. Paused,
// the song stops but the light keeps breathing, slowly; playing again brings it back over `long`.
// A still (a sample, `?all`, a shot held still) is drawn at a fixed moment after a short run-in, so
// the same still comes out every time; with no WebGL it is the cover alone.

import { type Analysis, BANDS, QUIET, type Signal, WAVE } from "./media-player-signal.ts";
import { raw } from "../../lib/tokens.ts";
import { settings } from "../../site/settings.ts";

export type VizKind = "water" | "silk" | "tube" | "embers";

/** Two colours in 0 … 1 per channel: the light's main one and its cold one. */
export interface Palette {
  readonly primary: readonly [number, number, number];
  readonly cold: readonly [number, number, number];
}

/** How long a theme's change of colours takes to ease in, s: its time constant. */
const PALETTE_EASE = 0.35;

/** A palette `share` of the way from `from` to `to`, so a theme's change of colours eases in. */
function blended(from: Palette, to: Palette, share: number): Palette {
  const mix = (a: readonly [number, number, number], b: readonly [number, number, number]): [number, number, number] => [a[0] + (b[0] - a[0]) * share, a[1] + (b[1] - a[1]) * share, a[2] + (b[2] - a[2]) * share];
  return { primary: mix(from.primary, to.primary), cold: mix(from.cold, to.cold) };
}

/** How the last frame comes back under the new one. */
export interface Feedback {
  /** Scale toward the middle, a little over 1 to stream outward. */
  readonly zoom: number;
  /** Radians turned a frame. */
  readonly turn: number;
  /** How much of the last frame stays. */
  readonly fade: number;
  /** The blur's reach, in pixels. */
  readonly blur: number;
  /** Drift a frame, in the screen's heights. */
  readonly shift: readonly [number, number];
  /**
   * Where the zoom and the turn are centred, from the screen's middle in its heights: where a
   * flight leads on screen, so its streaks pour out of its far end. The middle unless given.
   */
  readonly centre?: readonly [number, number];
}

/** Where the song is for a visualiser that needs more than the moment: how far it has flown, its song. */
export interface Flight {
  readonly travel: number;
  readonly spin: number;
  readonly signal: Signal;
  /** What the song's sections count from (`Analysis.section` already counts so): songs played one after another go on counting. */
  readonly offset: number;
  /** How much the song sounds: 1 playing, a breath paused. */
  readonly energy: number;
}

/** How many numbers a visualiser may hand its shader of its own (`extras`, `uExtra`). */
const EXTRAS = 32;

/**
 * The music's recent past, for a visualiser whose picture carries it: `rows` moments back from now,
 * `columns` numbers each, filled each frame into `out` (row after row) and handed to the shader as
 * the texture `uHistory`, filtered, so a depth can read the music of a moment ago.
 */
export interface History {
  readonly rows: number;
  readonly columns: number;
  readonly fill: (a: Analysis, flight: Flight, out: Float32Array) => void;
}

/** A visualiser: what it is called, how it feeds back, and what it draws. */
export interface Visual {
  readonly kind: VizKind;
  readonly name: string;
  readonly feedback: (a: Analysis, time: number, flight: Flight) => Feedback;
  /** What the scene pass draws: a full-screen fragment shader, points, or strips. */
  readonly scene:
    | { readonly draw: "fragment"; readonly fragment: string }
    | { readonly draw: "points"; readonly vertex: string; readonly fragment: string; readonly count: number }
    | { readonly draw: "strips"; readonly vertex: string; readonly fragment: string; readonly count: number };
  /** How the scene lands on the frame: added, or the brighter of the two (no doubled joints). */
  readonly blend: "add" | "max";
  /**
   * How fast it travels for the music at a moment, in its own units a second of the song: a
   * visualiser that flies (Water) is handed the distance travelled as `uTravel`.
   */
  readonly speed?: (a: Analysis) => number;
  /**
   * How fast it turns for the music at a moment, in radians a second of the song, handed over
   * integrated as `uSpin`: a change of rate turns it faster or slower, never jumps it.
   */
  readonly spin?: (a: Analysis) => number;
  /** How much it twists along its length for the music at a moment, radians a unit: `uTwist`. */
  readonly twist?: (a: Analysis) => number;
  /**
   * Where a new song's sections start counting when one is skipped to mid-section, given the
   * section shown: the next one unless given. A visualiser whose sections look different picks one
   * that looks the same, so a skip never changes the picture at once.
   */
  readonly continueAt?: (section: number) => number;
  /** Numbers of its own for its shader, up to `EXTRAS`, worked out each frame: `uExtra`. */
  readonly extras?: (a: Analysis, flight: Flight) => readonly number[];
  /** The music's recent past, handed over as `uHistory`. */
  readonly history?: History;
  /** The present pass's bloom strength and exposure. */
  readonly bloom: number;
  readonly exposure: number;
  /** How much the frame's edges darken, 0 to 1; 0.9 unless given. */
  readonly vignette?: number;
  /**
   * How light past white is brought into range: each channel on its own (the default), so light
   * piling up burns to white; or by its brightest channel, so piled-up colour stays its colour and
   * only what is white (a white-hot core) is white.
   */
  readonly tone?: "channel" | "hue";
}

/** A number written as GLSL's float: `32` as `32.0`, so a tunable never lands as an integer. */
export function glsl(value: number): string {
  return Number.isInteger(value) ? value.toFixed(1) : String(value);
}

/** What every scene shader may read: the moment, the music and the colours. */
export const COMMON = /* glsl */ `#version 300 es
precision highp float;
uniform vec2 uRes;
uniform float uTime;
uniform float uMusic;
uniform float uEnergy;
uniform float uKick;
uniform float uBass;
uniform float uMid;
uniform float uHigh;
uniform float uLevel;
uniform float uBuild;
uniform float uWeight;
uniform float uSince;
uniform float uTravel;
uniform float uSpin;
uniform float uTwist;
uniform float uExtra[${EXTRAS}];
uniform float uSection;
uniform float uInto;
uniform vec3 uPrimary;
uniform vec3 uCold;
uniform float uSpec[${BANDS}];
uniform float uWave[${WAVE}];
uniform sampler2D uHistory;
const float PI = 3.14159265;

float hash11(float p) { p = fract(p * 0.1031); p *= p + 33.33; p *= p + p; return fract(p); }
float hash21(vec2 p) { vec3 q = fract(vec3(p.xyx) * 0.1031); q += dot(q, q.yzx + 33.33); return fract((q.x + q.y) * q.z); }
float vnoise(vec2 p) {
  vec2 i = floor(p); vec2 f = fract(p); vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash21(i), hash21(i + vec2(1, 0)), u.x), mix(hash21(i + vec2(0, 1)), hash21(i + vec2(1, 1)), u.x), u.y);
}
float fbm(vec2 p) { float s = 0.0; float a = 0.5; for (int i = 0; i < 4; i++) { s += a * vnoise(p); p = p * 2.03 + 17.1; a *= 0.5; } return s; }
float spec(float at) { float x = clamp(at, 0.0, 1.0) * float(${BANDS - 1}); int i = int(floor(x)); int j = min(i + 1, ${BANDS - 1}); return mix(uSpec[i], uSpec[j], fract(x)); }
float wave(float at) { float x = clamp(at, 0.0, 1.0) * float(${WAVE - 1}); int i = int(floor(x)); int j = min(i + 1, ${WAVE - 1}); return mix(uWave[i], uWave[j], fract(x)); }
`;

const FULLSCREEN = /* glsl */ `#version 300 es
out vec2 vUv;
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  vUv = p;
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
`;

const FEEDBACK = /* glsl */ `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 o;
uniform sampler2D uPrev;
uniform vec2 uRes;
uniform float uZoom, uTurn, uFade, uBlur;
uniform vec2 uShift;
uniform vec2 uCentre;
void main() {
  float aspect = uRes.x / uRes.y;
  vec2 p = vUv - 0.5;
  p.x *= aspect;
  float c = cos(uTurn), s = sin(uTurn);
  p = mat2(c, -s, s, c) * (p - uCentre) / uZoom + uCentre - uShift;
  p.x /= aspect;
  vec2 uv = p + 0.5;
  vec2 d = uBlur / uRes;
  vec4 acc = texture(uPrev, uv) * 0.4
    + (texture(uPrev, uv + vec2(d.x, 0.0)) + texture(uPrev, uv - vec2(d.x, 0.0))
    + texture(uPrev, uv + vec2(0.0, d.y)) + texture(uPrev, uv - vec2(0.0, d.y))) * 0.15;
  float inside = step(0.0, uv.x) * step(uv.x, 1.0) * step(0.0, uv.y) * step(uv.y, 1.0);
  o = max(acc * uFade - 0.0015, 0.0) * inside;
}
`;

const PRESENT = /* glsl */ `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 o;
uniform sampler2D uImage;
uniform vec2 uRes;
uniform float uBloom, uExposure, uVignette, uHue;
uniform vec3 uPrimary, uCold;
float hash21(vec2 p) { vec3 q = fract(vec3(p.xyx) * 0.1031); q += dot(q, q.yzx + 33.33); return fract((q.x + q.y) * q.z); }
void main() {
  vec3 c = texture(uImage, vUv).rgb;
  vec3 glow = vec3(0.0);
  float h = uRes.y;
  for (int ring = 1; ring <= 3; ring++) {
    float r = float(ring * ring) * 0.004 * h;
    for (int k = 0; k < 8; k++) {
      float a = float(k) * 0.7854 + float(ring) * 0.39;
      glow += texture(uImage, vUv + vec2(cos(a), sin(a)) * r / uRes).rgb / float(ring);
    }
  }
  c += glow / 14.7 * uBloom;
  // The night's floor: the two colours, barely, so black is never dead.
  c += (uPrimary * 0.012 + uCold * 0.018);
  if (uHue > 0.5) {
    // By the brightest channel: linear to the knee, then eased toward full, every channel alike.
    float m = max(max(c.r, c.g), max(c.b, 1e-5)) * uExposure;
    float knee = 0.8;
    float mapped = m < knee ? m : knee + (1.0 - knee) * (1.0 - exp(-(m - knee) / (1.0 - knee)));
    c = c * uExposure * mapped / m;
  } else c = 1.0 - exp(-c * uExposure);
  vec2 q = vUv - 0.5;
  c *= 1.0 - dot(q, q) * uVignette;
  c += (hash21(gl_FragCoord.xy) - 0.5) / 255.0;
  o = vec4(c, 1.0);
}
`;

/** A compiled program and its uniforms by name. */
interface Program {
  readonly program: WebGLProgram;
  readonly uniforms: Map<string, WebGLUniformLocation | null>;
}

function compile(gl: WebGL2RenderingContext, vertex: string, fragment: string): Program {
  const make = (type: number, source: string) => {
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw new Error(`visualiser shader: ${gl.getShaderInfoLog(shader)}`);
    return shader;
  };
  const program = gl.createProgram()!;
  gl.attachShader(program, make(gl.VERTEX_SHADER, vertex));
  gl.attachShader(program, make(gl.FRAGMENT_SHADER, fragment));
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(`visualiser program: ${gl.getProgramInfoLog(program)}`);
  return { program, uniforms: new Map() };
}

function where(gl: WebGL2RenderingContext, made: Program, name: string): WebGLUniformLocation | null {
  if (!made.uniforms.has(name)) made.uniforms.set(name, gl.getUniformLocation(made.program, name));
  return made.uniforms.get(name)!;
}

/** A colour role of the night scheme as 0 … 1 channels, read through a canvas so any colour syntax works. */
export function roleColour(element: Element, role: string): [number, number, number] {
  const value = getComputedStyle(element).getPropertyValue(`--color-${role}`).trim() || "#808080";
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const pen = canvas.getContext("2d", { willReadFrequently: true })!;
  pen.fillStyle = value;
  pen.fillRect(0, 0, 1, 1);
  const [r, g, b] = pen.getImageData(0, 0, 1, 1).data;
  return [r! / 255, g! / 255, b! / 255];
}

/**
 * A colour as light burns it: its own hue, its saturation brought up and its lightness to a tube's
 * middle, so a pale night-scheme accent still glows as neon rather than as cream.
 */
export function neon(colour: readonly [number, number, number]): [number, number, number] {
  const [r, g, b] = colour;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const light = (max + min) / 2;
  const delta = max - min;
  let hue = 0;
  if (delta > 1e-5) {
    if (max === r) hue = ((g - b) / delta) % 6;
    else if (max === g) hue = (b - r) / delta + 2;
    else hue = (r - g) / delta + 4;
  }
  hue = (hue * 60 + 360) % 360;
  const saturation = delta < 1e-5 ? 0 : Math.max(0.85, delta / (1 - Math.abs(2 * light - 1)));
  const lightness = Math.min(0.58, Math.max(0.48, light));
  const chroma = (1 - Math.abs(2 * lightness - 1)) * Math.min(1, saturation);
  const x = chroma * (1 - Math.abs(((hue / 60) % 2) - 1));
  const m = lightness - chroma / 2;
  const [rr, gg, bb] = hue < 60 ? [chroma, x, 0] : hue < 120 ? [x, chroma, 0] : hue < 180 ? [0, chroma, x] : hue < 240 ? [0, x, chroma] : hue < 300 ? [x, 0, chroma] : [chroma, 0, x];
  return [rr + m, gg + m, bb + m];
}

/** The engine: one WebGL2 context drawing visualisers into its canvas. */
export class Engine {
  readonly canvas: HTMLCanvasElement;
  private readonly gl: WebGL2RenderingContext;
  private readonly feedbackProgram: Program;
  private readonly presentProgram: Program;
  private readonly scenes = new Map<VizKind, Program>();
  private readonly float: boolean;
  /** Whether it draws in software (SwiftShader): a still is then drawn small. */
  readonly software: boolean;
  private targets: { texture: WebGLTexture; buffer: WebGLFramebuffer }[] = [];
  private history: { texture: WebGLTexture; data: Float32Array } | null = null;
  private front = 0;
  private width = 0;
  private height = 0;

  constructor(canvas: HTMLCanvasElement, preserve = false) {
    this.canvas = canvas;
    const gl = canvas.getContext("webgl2", { alpha: false, antialias: false, premultipliedAlpha: false, preserveDrawingBuffer: preserve });
    if (!gl) throw new Error("no WebGL2");
    this.gl = gl;
    this.float = !!gl.getExtension("EXT_color_buffer_float");
    const about = gl.getExtension("WEBGL_debug_renderer_info");
    this.software = /swiftshader|llvmpipe|software/i.test(String(about ? gl.getParameter(about.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER)));
    gl.bindVertexArray(gl.createVertexArray());
    this.feedbackProgram = compile(gl, FULLSCREEN, FEEDBACK);
    this.presentProgram = compile(gl, FULLSCREEN, PRESENT);
  }

  /** Sizes the canvas and its two frames to `width` × `height` device pixels, clearing them. */
  resize(width: number, height: number): void {
    const gl = this.gl;
    width = Math.max(1, Math.round(width));
    height = Math.max(1, Math.round(height));
    if (width === this.width && height === this.height) return;
    this.width = width;
    this.height = height;
    this.canvas.width = width;
    this.canvas.height = height;
    for (const each of this.targets) {
      gl.deleteTexture(each.texture);
      gl.deleteFramebuffer(each.buffer);
    }
    this.targets = [0, 1].map(() => {
      const texture = gl.createTexture()!;
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, this.float ? gl.RGBA16F : gl.RGBA8, width, height, 0, gl.RGBA, this.float ? gl.HALF_FLOAT : gl.UNSIGNED_BYTE, null);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      const buffer = gl.createFramebuffer()!;
      gl.bindFramebuffer(gl.FRAMEBUFFER, buffer);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
      gl.clearColor(0, 0, 0, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);
      return { texture, buffer };
    });
  }

  /** Clears the trails: a new visualiser, or a still's run-in, starts from the night. */
  clear(): void {
    const gl = this.gl;
    for (const each of this.targets) {
      gl.bindFramebuffer(gl.FRAMEBUFFER, each.buffer);
      gl.clearColor(0, 0, 0, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);
    }
  }

  private scene(visual: Visual): Program {
    let made = this.scenes.get(visual.kind);
    if (!made) {
      const scene = visual.scene;
      made = scene.draw === "fragment" ? compile(this.gl, FULLSCREEN, scene.fragment) : compile(this.gl, scene.vertex, scene.fragment);
      this.scenes.set(visual.kind, made);
    }
    return made;
  }

  /**
   * Draws one frame of `visual` for the music `a` at `time`, with the song's `energy` (1 playing,
   * a breath paused) and `since` seconds since the last kick.
   */
  frame(visual: Visual, a: Analysis, time: number, energy: number, since: number, palette: Palette, travel = 0, spin = 0, signal: Signal = QUIET, offset = 0): void {
    const gl = this.gl;
    const from = this.targets[this.front]!;
    const to = this.targets[1 - this.front]!;
    gl.viewport(0, 0, this.width, this.height);

    // 1. The last frame, carried.
    const flight: Flight = { travel, spin, signal, offset, energy };
    const back = visual.feedback(a, time, flight);
    gl.bindFramebuffer(gl.FRAMEBUFFER, to.buffer);
    gl.disable(gl.BLEND);
    gl.useProgram(this.feedbackProgram.program);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, from.texture);
    const fb = this.feedbackProgram;
    gl.uniform1i(where(gl, fb, "uPrev"), 0);
    gl.uniform2f(where(gl, fb, "uRes"), this.width, this.height);
    gl.uniform1f(where(gl, fb, "uZoom"), back.zoom);
    gl.uniform1f(where(gl, fb, "uTurn"), back.turn);
    gl.uniform1f(where(gl, fb, "uFade"), back.fade);
    gl.uniform1f(where(gl, fb, "uBlur"), back.blur * (this.height / 1080));
    gl.uniform2f(where(gl, fb, "uShift"), back.shift[0], back.shift[1]);
    gl.uniform2f(where(gl, fb, "uCentre"), back.centre?.[0] ?? 0, back.centre?.[1] ?? 0);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // 2. The scene, on top.
    const made = this.scene(visual);
    gl.useProgram(made.program);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE);
    gl.blendEquation(visual.blend === "max" ? gl.MAX : gl.FUNC_ADD);
    const set1 = (name: string, value: number) => gl.uniform1f(where(gl, made, name), value);
    gl.uniform2f(where(gl, made, "uRes"), this.width, this.height);
    set1("uTime", time);
    set1("uMusic", a.time);
    set1("uEnergy", energy);
    set1("uKick", a.kick * energy);
    set1("uBass", a.bass * energy);
    set1("uMid", a.mid * energy);
    set1("uHigh", a.high * energy);
    set1("uLevel", a.level * energy);
    set1("uBuild", a.build * energy);
    set1("uWeight", a.weight);
    set1("uSince", since);
    set1("uTravel", travel);
    set1("uSpin", spin);
    set1("uTwist", visual.twist?.(a) ?? 0);
    if (visual.extras) {
      const extra = new Float32Array(EXTRAS);
      extra.set(visual.extras(a, flight).slice(0, EXTRAS));
      gl.uniform1fv(where(gl, made, "uExtra"), extra);
    }
    set1("uSection", a.section);
    set1("uInto", a.into);
    gl.uniform3f(where(gl, made, "uPrimary"), ...palette.primary);
    gl.uniform3f(where(gl, made, "uCold"), ...palette.cold);
    gl.uniform1fv(where(gl, made, "uSpec"), a.spectrum.map((value) => value * energy));
    gl.uniform1fv(where(gl, made, "uWave"), a.wave.map((value) => value * energy));
    if (visual.history) {
      const { rows, columns } = visual.history;
      if (!this.history || this.history.data.length !== rows * columns) {
        if (this.history) gl.deleteTexture(this.history.texture);
        const texture = gl.createTexture()!;
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        this.history = { texture, data: new Float32Array(rows * columns) };
      }
      visual.history.fill(a, flight, this.history.data);
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, this.history.texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.R16F, columns, rows, 0, gl.RED, gl.FLOAT, this.history.data);
      gl.uniform1i(where(gl, made, "uHistory"), 1);
      gl.activeTexture(gl.TEXTURE0);
    }
    const scene = visual.scene;
    if (scene.draw === "fragment") gl.drawArrays(gl.TRIANGLES, 0, 3);
    else if (scene.draw === "points") gl.drawArrays(gl.POINTS, 0, scene.count);
    else gl.drawArrays(gl.TRIANGLES, 0, scene.count * 6);
    gl.blendEquation(gl.FUNC_ADD);
    gl.disable(gl.BLEND);

    // 3. To the screen.
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    const present = this.presentProgram;
    gl.useProgram(present.program);
    gl.bindTexture(gl.TEXTURE_2D, to.texture);
    gl.uniform1i(where(gl, present, "uImage"), 0);
    gl.uniform2f(where(gl, present, "uRes"), this.width, this.height);
    gl.uniform1f(where(gl, present, "uBloom"), visual.bloom);
    gl.uniform1f(where(gl, present, "uExposure"), visual.exposure);
    gl.uniform1f(where(gl, present, "uVignette"), visual.vignette ?? 0.9);
    gl.uniform1f(where(gl, present, "uHue"), visual.tone === "hue" ? 1 : 0);
    gl.uniform3f(where(gl, present, "uPrimary"), ...palette.primary);
    gl.uniform3f(where(gl, present, "uCold"), ...palette.cold);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    this.front = 1 - this.front;
  }
}

/** `rate` summed over `time` seconds of `signal`'s song: how far it has travelled or turned, the same every time. */
export function summed(rate: ((a: Analysis) => number) | undefined, signal: Signal, time: number): number {
  if (!rate) return 0;
  let sum = 0;
  const step = 1 / 20;
  for (let at = 0; at < time; at += step) sum += rate(signal.analyse(at)) * Math.min(step, time - at);
  return sum;
}

/** Seconds since the last kick's onset at `time`, looking back up to two beats. */
function sinceKick(signal: Signal, time: number): number {
  const reach = (60 / signal.bpm) * 2;
  const last = signal.kicks(time - reach, time).at(-1);
  return last ? time - last.time : reach;
}

/** A duration token in seconds under the speed setting. */
function seconds(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / 1000 / settings().speed;
}

/**
 * A visualiser running live in `canvas`: it draws while it is in view, at the display's rate. The
 * song's time runs while `playing()` says so; paused, it slows to a breath, and playing again
 * brings it back over `duration.long`.
 */
export class Live {
  private readonly engine: Engine;
  private visual: Visual;
  private signal: Signal;
  private palette: Palette | null = null;
  private target: Palette | null = null;
  private frames = 0;
  private frameId = 0;
  private last = -1;
  private music: number;
  private energy = 1;
  private seen = true;
  private lastOnset = -10;
  private travel = 0;
  private spin = 0;
  private offset = 0;
  private readonly paletteOf: () => Palette;
  private readonly playing: () => boolean;
  private readonly clock: (() => number | null) | undefined;

  constructor(
    canvas: HTMLCanvasElement,
    visual: Visual,
    signal: Signal,
    paletteOf: () => Palette,
    playing: () => boolean,
    start: number,
    clock?: () => number | null,
  ) {
    this.paletteOf = paletteOf;
    this.playing = playing;
    this.clock = clock;
    this.engine = new Engine(canvas);
    this.visual = visual;
    this.signal = signal;
    this.music = start;
    this.travel = summed(visual.speed, signal, start);
    this.spin = summed(visual.spin, signal, start);
    new IntersectionObserver((entries) => {
      this.seen = entries.some((entry) => entry.isIntersecting);
      if (this.seen) this.run();
    }).observe(canvas);
    this.run();
  }

  /**
   * Plays `visual` for `signal`'s song from now on, its colours read again. `song` says whether
   * `signal` is another song, from `start` seconds into it (else the same song's signal, its
   * analysis arrived): the flight goes on, and the new song's sections go on counting from the
   * section shown, the next one where the old song ended on its section's end, so the stretch
   * arriving was its first; else one the visualiser picks to go on from (`continueAt`).
   */
  set(visual: Visual, signal: Signal, song?: { readonly start: number }): void {
    if (visual.kind !== this.visual.kind) this.engine.clear();
    if (song) {
      const span = this.signal.section(this.music);
      const shown = span.index + this.offset;
      const ended = span.end - this.music < 0.5;
      this.offset = ended ? shown + 1 : (visual.continueAt?.(shown) ?? shown + 1);
      this.music = song.start;
    }
    this.visual = visual;
    this.signal = signal;
    this.target = null;
  }

  private run(): void {
    if (this.frameId) return;
    const tick = (now: number) => {
      this.frameId = 0;
      if (!this.engine.canvas.isConnected || !this.seen) return;
      const box = this.engine.canvas.getBoundingClientRect();
      this.engine.resize(box.width * devicePixelRatio, box.height * devicePixelRatio);
      const dt = this.last < 0 ? 1 / 60 : Math.min(0.1, (now - this.last) / 1000);
      this.last = now;
      // Toward 1 playing over `long`, toward a breath paused over `long`.
      const target = this.playing() ? 1 : 0;
      const rate = dt / Math.max(0.001, seconds("long"));
      this.energy += Math.max(-rate, Math.min(rate, target - this.energy));
      const breath = 0.18 + 0.06 * Math.sin(now / 1000 * 0.9);
      const energy = Math.max(breath, this.energy);
      // The song's time: the audio's own, where it plays out loud, or its own clock, creeping while
      // paused. Either way what it flies and turns follows, and paused it still drifts on, slowly.
      const heard = this.clock?.() ?? null;
      const was = this.music;
      this.music = heard ?? this.music + dt * (0.12 + 0.88 * this.energy);
      const moved = this.music - was;
      const step = moved > 0 && moved < 0.25 ? Math.max(moved, dt * 0.12) : dt * (0.12 + 0.88 * this.energy);
      // The colours read again now and then, and eased toward over a moment: a theme's change
      // reaches the light within a second, never at a cut.
      if (!this.target || this.frames++ % 60 === 0) this.target = this.paletteOf();
      this.palette = this.palette ? blended(this.palette, this.target, 1 - Math.exp(-dt / PALETTE_EASE)) : this.target;
      const read = this.signal.analyse(this.music);
      const a = this.offset ? { ...read, section: read.section + this.offset } : read;
      this.travel += (this.visual.speed?.(a) ?? 0) * step;
      this.spin += (this.visual.spin?.(a) ?? 0) * step;
      this.lastOnset = this.music - sinceKick(this.signal, this.music);
      this.engine.frame(this.visual, a, now / 1000, energy, this.music - this.lastOnset, this.palette, this.travel, this.spin, this.signal, this.offset);
      this.frameId = requestAnimationFrame(tick);
    };
    this.frameId = requestAnimationFrame(tick);
  }

  stop(): void {
    cancelAnimationFrame(this.frameId);
    this.frameId = 0;
  }
}

/** The one context every still is drawn in, kept: a page of samples never holds a context each. */
let shared: Engine | null = null;
let sharedFailed = false;

/**
 * `visual` drawn still at `time` seconds into `signal`'s song, `width` × `height` CSS pixels, after a
 * second's run-in so its trails are there: a picture, the same every time. Null with no WebGL.
 */
export function still(visual: Visual, signal: Signal, palette: Palette, time: number, width: number, height: number, playing = true): HTMLCanvasElement | null {
  if (sharedFailed) return null;
  try {
    shared ??= new Engine(document.createElement("canvas"), true);
  } catch {
    sharedFailed = true;
    return null;
  }
  // Drawn at most a full-HD's pixels, a sixteenth of them in software: a sample is a picture,
  // scaled to its room, and a page of them must draw in seconds with no GPU.
  const most = shared.software ? 480 : 1920;
  const scale = Math.min(devicePixelRatio, most / Math.max(1, width), (most * 9) / 16 / Math.max(1, height));
  shared.resize(width * scale, height * scale);
  shared.clear();
  const steps = 30;
  let travel = summed(visual.speed, signal, time - steps / 60);
  let spin = summed(visual.spin, signal, time - steps / 60);
  for (let i = steps; i >= 0; i--) {
    const at = time - i / 60;
    const energy = playing ? 1 : 0.2;
    const a = signal.analyse(playing ? at : time);
    travel += (visual.speed?.(a) ?? 0) * (playing ? 1 : 0.15) / 60;
    spin += (visual.spin?.(a) ?? 0) * (playing ? 1 : 0.15) / 60;
    shared.frame(visual, a, at, energy, sinceKick(signal, at), palette, travel, spin, signal);
  }
  const picture = document.createElement("canvas");
  picture.width = shared.canvas.width;
  picture.height = shared.canvas.height;
  picture.getContext("2d")!.drawImage(shared.canvas, 0, 0);
  return picture;
}
