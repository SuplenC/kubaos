// The WeatherSky's drawing: one fragment shader for the whole sky (its colours, the sun, the moon
// in its phase, the stars, two cloud decks, haze, smog, fog and lightning), and the one WebGL
// context every sky on the page draws in. Each sky is drawn into the corner of that context's
// canvas and copied into its own 2D canvas, so a page of many skies holds one context, and the
// precipitation's library, which takes contexts of its own, never pushes the oldest out.
//
// What falls is not drawn here: the precipitation's library draws it, so it can land on the text.

const VERTEX = `attribute vec2 a; void main(){ gl_Position = vec4(a, 0., 1.); }`;

const FRAGMENT = `precision highp float;
uniform vec2 uRes; uniform vec4 uView; uniform float uT, uCover, uFog, uFlash, uStars, uDrift, uSunUp, uMoonUp, uHaze, uGlare, uHalo, uSmog;
uniform vec3 uTop, uBot, uLit, uShade, uSunCol; uniform vec2 uSun, uMoon;
float hash(vec2 p){ p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }
float noise(vec2 p){ vec2 i = floor(p), f = fract(p); f = f*f*(3. - 2.*f);
  return mix(mix(hash(i), hash(i + vec2(1,0)), f.x), mix(hash(i + vec2(0,1)), hash(i + vec2(1,1)), f.x), f.y); }
float fbm(vec2 p){ float a = .5, s = 0.; for (int i = 0; i < 5; i++){ s += a * noise(p); p = p * 2.03 + vec2(17.1, 9.2); a *= .5; } return s; }
float fbm3(vec2 p){ float a = .5, s = 0.; for (int i = 0; i < 3; i++){ s += a * noise(p); p = p * 2.03 + vec2(17.1, 9.2); a *= .5; } return s; }
vec2 deck(vec2 p, float scale, float speed, float seed, vec2 light){
  vec2 q = p * scale + vec2(uDrift * speed + seed, seed * .37);
  q += (fbm3(q * .6 + uT * .012) - .5) * .9;
  float n = fbm(q); float thr = mix(.78, .26, uCover);
  return vec2(smoothstep(thr, thr + .22, n), clamp((n - fbm3(q + light * .35)) * 2.6 + .55, 0., 1.));
}
vec3 scene(vec2 p){
  float y = clamp(p.y + .5, 0., 1.);
  vec3 col = mix(uBot, uTop, pow(y, .75));
  vec2 sp = p * 26.; vec2 si = floor(sp); vec2 sf = fract(sp) - .5;
  float sh = hash(si); vec2 so = vec2(hash(si + 3.), hash(si + 9.)) - .5;
  col += step(.9, sh) * smoothstep(.07, 0., length(sf - so * .7)) * (.55 + .45 * sin(uT * (1. + sh * 2.) + sh * 40.)) * uStars * (.5 + .5 * y);
  // The sun: a blown white core, a tight bloom and a wide veil, all warm, laid over the sky rather
  // than added to it so the blue never shows through as a cold ring. Strong UV widens the glare and
  // draws thin streaks through it; humid air widens the veil.
  vec2 dv = p - uSun; float ds = length(dv);
  vec3 warm = mix(uSunCol, vec3(1., .96, .86), .45);
  float bloom = 1. / (1. + pow(ds * (11. - 5. * uGlare), 2.)), veil = 1. / (1. + pow(ds * (3. - 1.5 * uHalo), 2.));
  float streak = (exp(-abs(dv.y) * 90.) * exp(-abs(dv.x) * 3.5) + exp(-abs(dv.x) * 120.) * exp(-abs(dv.y) * 6.)) * uGlare;
  col = mix(col, warm, clamp((bloom * (.8 + .5 * uGlare) + veil * (.28 + .3 * uHalo) + streak * .5) * uSunUp, 0., 1.));
  col = mix(col, vec3(1., .995, .97), smoothstep(.062, .048, ds) * uSunUp);
  // The moon: a lit ball with its seas, darker at the limb, the dark side barely there.
  vec2 m = (p - uMoon) / .062; float r2 = dot(m, m); float dm = length(p - uMoon);
  col += vec3(.62, .72, .95) * uMoonUp * (.16 + .25 * uHalo) / (1. + pow(dm * (13. - 8. * uHalo), 2.));
  if (r2 < 1.){
    vec3 n = vec3(m, sqrt(1. - r2)); float lit = smoothstep(-.03, .14, dot(n, normalize(vec3(.72, .22, .55))));
    float seas = smoothstep(.38, .62, fbm(m * 1.9 + vec2(3.3, 8.1)));
    float albedo = mix(.7, 1., seas) * (.95 + .1 * noise(m * 9.)) * mix(.7, 1., n.z);
    col = mix(col, vec3(.96, .95, .9) * albedo * (lit + .03), smoothstep(1., .9, r2) * uMoonUp);
  }
  vec2 light = normalize(mix(uMoon, uSun, uSunUp) - p + vec2(0., .4));
  vec2 far = deck(p + vec2(0., .2), 1.5, .010, 3.1, light); vec2 near = deck(p, .85, .022, 11.7, light);
  vec3 lit = uLit + warm * uSunUp * .3 / (1. + pow(ds * 3., 2.));
  col = mix(col, mix(uShade, lit, far.y) * .94, far.x * .8);
  col = mix(col, mix(uShade, lit, near.y), near.x * .95);
  // Poor visibility whitens the distance, most at the horizon; bad air lays a dull warm band there.
  vec3 hazeCol = mix(mix(uBot, uLit, .55), warm, .2 * uSunUp / (1. + ds * 2.));
  col = mix(col, hazeCol, clamp(uHaze * mix(1., .4, y), 0., 1.));
  float l = dot(hazeCol, vec3(.33));
  col = mix(col, vec3(.86, .7, .48) * (l * .9 + .08), uSmog * smoothstep(.75, 0., y) * .7);
  float fog = uFog * (.55 + .45 * fbm3(p * 1.2 + vec2(uT * .02, 0.))) * mix(1., .55, y);
  col = mix(col, mix(uShade, uLit, .7), clamp(fog, 0., 1.));
  col += uFlash * (.25 + .75 * near.x) * vec3(.75, .82, 1.) * (.4 + .6 * fbm3(p * 2. + 5.));
  return col;
}
void main(){
  // The sky is laid out on the part of the canvas in view: a box changing size is the same sky.
  vec2 p = (gl_FragCoord.xy - uView.xy - .5 * uView.zw) / uView.w;
  vec3 col = scene(p);
  // A dither under one step of 8 bits, so the dark gradients never band.
  col += (hash(gl_FragCoord.xy + fract(uT)) - .5) / 255.;
  gl_FragColor = vec4(col, 1.);
}`;

type Vec3 = readonly [number, number, number];
type Vec2 = readonly [number, number];

/** What one frame of a sky draws: every value the shader reads, colours in linear light. */
export interface SkyFrame {
  readonly top: Vec3;
  readonly bottom: Vec3;
  readonly lit: Vec3;
  readonly shade: Vec3;
  readonly sun: Vec3;
  readonly sunAt: Vec2;
  readonly sunUp: number;
  readonly moonAt: Vec2;
  readonly moonUp: number;
  readonly stars: number;
  readonly cover: number;
  readonly fog: number;
  readonly haze: number;
  readonly glare: number;
  readonly halo: number;
  readonly smog: number;
}

const NAMES = ["uRes", "uView", "uT", "uCover", "uFog", "uFlash", "uStars", "uDrift", "uSunUp", "uMoonUp", "uHaze", "uGlare", "uHalo", "uSmog", "uTop", "uBot", "uLit", "uShade", "uSunCol", "uSun", "uMoon"] as const;

/** The context every sky draws in, its program and its uniforms. */
class Painter {
  readonly canvas = document.createElement("canvas");
  private readonly gl: WebGLRenderingContext | null;
  private readonly at = new Map<(typeof NAMES)[number], WebGLUniformLocation | null>();

  constructor() {
    this.canvas.width = 1;
    this.canvas.height = 1;
    // `preserveDrawingBuffer`: a sky's frame is copied out after the draw, in the same task, and
    // left as it was it would be cleared once the page composites.
    this.gl = this.canvas.getContext("webgl", { antialias: false, alpha: false, powerPreference: "low-power", preserveDrawingBuffer: true });
    const gl = this.gl;
    if (!gl) return;
    const shader = (type: number, source: string) => {
      const made = gl.createShader(type);
      if (!made) throw new Error("the sky's shader wasn't made");
      gl.shaderSource(made, source);
      gl.compileShader(made);
      if (!gl.getShaderParameter(made, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(made) ?? "the sky's shader didn't compile");
      return made;
    };
    const program = gl.createProgram();
    gl.attachShader(program, shader(gl.VERTEX_SHADER, VERTEX));
    gl.attachShader(program, shader(gl.FRAGMENT_SHADER, FRAGMENT));
    gl.bindAttribLocation(program, 0, "a");
    gl.linkProgram(program);
    gl.useProgram(program);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    for (const name of NAMES) this.at.set(name, gl.getUniformLocation(program, name));
  }

  /**
   * Draws `frame` at `time` seconds, with the lightning at `flash` and the clouds `drift` along, in
   * `width` × `height` device pixels, and copies it into `into`. The sky is laid out on `view`, the
   * part of those pixels in view from their top left, in device pixels; all of them unless given.
   */
  paint(into: CanvasRenderingContext2D, frame: SkyFrame, time: number, drift: number, flash: number, width: number, height: number, view?: { readonly width: number; readonly height: number }): void {
    const gl = this.gl;
    if (!gl) return;
    // The canvas only grows, so skies of different sizes drawn one after another never reallocate it.
    if (width > this.canvas.width || height > this.canvas.height) {
      this.canvas.width = Math.max(width, this.canvas.width);
      this.canvas.height = Math.max(height, this.canvas.height);
    }
    gl.viewport(0, 0, width, height);
    const one = (name: (typeof NAMES)[number], value: number) => gl.uniform1f(this.at.get(name) ?? null, value);
    // The colours are mixed in linear light and drawn in sRGB, as the canvas shows them.
    const encoded = (c: Vec3) => c.map((v) => Math.max(v, 0) ** (1 / 2.2));
    gl.uniform2f(this.at.get("uRes") ?? null, width, height);
    const seen = view ?? { width, height };
    gl.uniform4f(this.at.get("uView") ?? null, 0, height - seen.height, seen.width, seen.height);
    one("uT", time);
    one("uCover", frame.cover);
    one("uFog", frame.fog);
    one("uFlash", flash);
    one("uStars", frame.stars);
    one("uDrift", drift);
    one("uSunUp", frame.sunUp);
    one("uMoonUp", frame.moonUp);
    one("uHaze", frame.haze);
    one("uGlare", frame.glare);
    one("uHalo", frame.halo);
    one("uSmog", frame.smog);
    gl.uniform3fv(this.at.get("uTop") ?? null, encoded(frame.top));
    gl.uniform3fv(this.at.get("uBot") ?? null, encoded(frame.bottom));
    gl.uniform3fv(this.at.get("uLit") ?? null, encoded(frame.lit));
    gl.uniform3fv(this.at.get("uShade") ?? null, encoded(frame.shade));
    gl.uniform3fv(this.at.get("uSunCol") ?? null, encoded(frame.sun));
    gl.uniform2fv(this.at.get("uSun") ?? null, frame.sunAt);
    gl.uniform2fv(this.at.get("uMoon") ?? null, frame.moonAt);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    // The frame sits in the canvas's bottom-left corner: WebGL counts rows from the bottom.
    into.drawImage(this.canvas, 0, this.canvas.height - height, width, height, 0, 0, width, height);
  }

  /** Lets the context go now, rather than whenever the page is collected. */
  free(): void {
    this.gl?.getExtension("WEBGL_lose_context")?.loseContext();
  }
}

let painter: Painter | null = null;
let holders = 0;

/** A hold on the shared context: made with the first, let go with the last. */
export interface PainterHold {
  paint: Painter["paint"];
  release(): void;
}

/** Takes a hold on the shared context, making it if no sky holds it. */
export function holdPainter(): PainterHold {
  painter ??= new Painter();
  holders++;
  const held = painter;
  let released = false;
  return {
    paint: (...args) => held.paint(...args),
    release() {
      if (released) return;
      released = true;
      holders--;
      if (holders > 0 || painter !== held) return;
      held.free();
      painter = null;
    },
  };
}
