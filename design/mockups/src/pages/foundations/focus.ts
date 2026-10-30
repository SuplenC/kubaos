// The Focus foundation: the neon tube and its glow, the line inside the edge of every kind of
// control, lit with the signs on and off, content lit in its tone, the flicker, where a glow is
// cut and the active window's tube (`docs/DESIGN.md`, "States").
//
// The controls other than the button are their shapes alone, drawn for their focus line: each
// gets its own page, and its own mock, at its level.

import { button, filled, labelGlow, type Kind, type Size, SIZES, TONES } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { focusRing, glow, type Tone, toneColour, windowRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { nested, role } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { length, names, number, raw } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { type Curve, type Duration, type Easing, foundation, type Move, type NonEmpty, type Track } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, swatch, table, themes, token } from "../../site/parts/swatches.ts";
import { type ColourPath, paintBetween, palette, paletteName, themeName } from "../../site/recolour.ts";
import { strikeSteps, milliseconds } from "../../site/recipe.ts";
import { themeChange, toDark, toLight } from "../../site/sequences.ts";
import { type Mode, SCALES, settings } from "../../site/settings.ts";

/**
 * How a control is shown: at rest, lit in the look the scene gives it, or lit with the signs off
 * whatever the scene. These are the pinned states the button takes too.
 */
type Shown = "rest" | "lit" | "signs-off";

/** Every tone a focus line lights in. */
const ALL_TONES: readonly Tone[] = ["accent", "cold", "error", "success", "warning", "sharing"];

/** A kind of control, as its focus line lights it. */
interface Control {
  readonly name: string;
  readonly tone: Tone;
  /** Filled with its tone's colour: the line inside the fill's edge is the tone's core. */
  readonly filled: boolean;
  /** Its content is drawn in its tone, so lit, it is a tube too. */
  readonly content: boolean;
  readonly draw: (shown: Shown) => HTMLElement;
}

/** The words each kind of button shows here. */
const BUTTON_WORDS: Record<Kind, string> = { Filled: "Continue", Tonal: "Save log", Text: "Back", Danger: "Erase disk", Success: "Finish", Warning: "Restart now" };

/** A button pinned as this page shows it. */
function pinnedButton(shown: Shown): { pinned?: "lit" | "signs-off" } {
  return shown === "rest" ? {} : { pinned: shown };
}

function buttonControl(kind: Kind, size: Size = "Md"): Control {
  return {
    name: `${kind} button`,
    tone: TONES[kind],
    filled: filled(kind),
    content: labelGlow(kind) === "content",
    draw: (shown) => button({ label: BUTTON_WORDS[kind], kind, size, ...pinnedButton(shown) }),
  };
}

/** A shape's measures and fill, each a token's custom property. */
interface ShapeSpec {
  readonly shape: string;
  readonly corner: "small" | "control" | "surface" | "full";
  readonly width?: string;
  readonly height?: string;
  readonly padBlock?: string;
  readonly padInline?: string;
  readonly fill?: string;
  readonly outlined?: boolean;
  readonly justify?: string;
  readonly children?: () => Element[];
}

/** A control's shape, focusable, with the library's focus ring in `tone`. */
function shape(spec: ShapeSpec, tone: Tone, filled: boolean, shown: Shown): HTMLElement {
  const drawn = el("div", "f-control", ...(spec.children?.() ?? []));
  drawn.dataset.shape = spec.shape;
  drawn.tabIndex = 0;
  drawn.toggleAttribute("data-outlined", spec.outlined ?? false);
  drawn.style.setProperty("--corner", `var(--radius-${spec.corner})`);
  const set = (property: string, value: string | undefined) => {
    if (value) drawn.style.setProperty(property, value);
  };
  set("--width", spec.width);
  set("--height", spec.height);
  set("--pad-block", spec.padBlock);
  set("--pad-inline", spec.padInline);
  set("--fill", spec.fill);
  set("--justify", spec.justify);
  focusRing(drawn, spec.corner, tone, filled);
  if (shown !== "rest") drawn.dataset.pinned = shown;
  return drawn;
}

function shapeControl(name: string, spec: ShapeSpec, tone: Tone, filled: boolean): Control {
  return { name, tone, filled, content: false, draw: (shown) => shape(spec, tone, filled, shown) };
}

const FIELD: ShapeSpec = {
  shape: "field",
  corner: "control",
  width: "var(--size-menu)",
  height: "var(--size-control_md)",
  padInline: "var(--space-s16)",
  fill: "var(--color-surface_container_high)",
  outlined: true,
  children: () => [text("Passphrase", "body_large", "on_surface_variant")],
};

const CHECKBOX: ShapeSpec = { shape: "checkbox", corner: "small", width: "var(--size-check)", height: "var(--size-check)", outlined: true };
const RADIO: ShapeSpec = { shape: "radio", corner: "full", width: "var(--size-check)", height: "var(--size-check)", outlined: true };

const CHIP: ShapeSpec = {
  shape: "chip",
  corner: "control",
  height: "var(--size-chip)",
  padInline: "var(--space-s8)",
  outlined: true,
  children: () => [text("Wi-Fi", "label", "on_surface")],
};

const CARD: ShapeSpec = {
  shape: "card",
  corner: "surface",
  width: "var(--size-menu)",
  padBlock: "var(--space-s16)",
  padInline: "var(--space-s16)",
  fill: "var(--color-surface_container_high)",
  outlined: true,
  children: () => [text("Erase the disk", "body_strong", "on_surface"), text("Everything on it goes", "small", "on_surface_variant")],
};

/** The role a symbol is drawn in on a fill of each tone: its `on_` role, or for a fixed role, which has none, the surface. */
const ON_TONE: Record<Tone, string> = {
  accent: "on_primary",
  cold: "on_cold",
  error: "on_error",
  success: "surface",
  warning: "surface",
  sharing: "surface",
  burn_danger: "on_burn_danger",
  burn_success: "on_burn_success",
  burn_warning: "on_burn_warning",
};

/** A tile that is on, filled with its tone, its symbol on it. */
function tileOn(tone: Tone): ShapeSpec {
  const colour = toneColour(tone);
  return {
    shape: "tile",
    corner: "control",
    width: "var(--size-control_lg)",
    height: "var(--size-control_lg)",
    fill: `var(--color-${colour})`,
    justify: "center",
    children: () => [icon("wifi", "var(--type-heading-icon)", `var(--color-${ON_TONE[tone]})`)],
  };
}

/** A tile that is off: outlined, its symbol in the quiet content colour. */
const TILE_OFF: ShapeSpec = {
  shape: "tile",
  corner: "control",
  width: "var(--size-control_lg)",
  height: "var(--size-control_lg)",
  fill: "var(--color-surface_container_highest)",
  outlined: true,
  justify: "center",
  children: () => [icon("wifi", "var(--type-heading-icon)", "var(--color-on_surface_variant)")],
};

const SWITCH_ON: ShapeSpec = {
  shape: "switch",
  corner: "full",
  width: "var(--size-switch_width)",
  height: "var(--size-switch_height)",
  padInline: "var(--space-s4)",
  fill: "var(--color-primary)",
  justify: "flex-end",
  children: () => [el("div", "f-knob")],
};

/** Every kind of control the focus line is drawn on, unfilled first, then filled with its tone. */
const CONTROLS: readonly Control[] = [
  buttonControl("Tonal"),
  buttonControl("Text"),
  shapeControl("Field", FIELD, "accent", false),
  shapeControl("Invalid field", FIELD, "error", false),
  shapeControl("Checkbox", CHECKBOX, "accent", false),
  shapeControl("Radio", RADIO, "accent", false),
  shapeControl("Cold chip", CHIP, "cold", false),
  shapeControl("Picked card", CARD, "cold", false),
  buttonControl("Filled"),
  buttonControl("Danger"),
  shapeControl("Tile, on", tileOn("accent"), "accent", true),
  shapeControl("Switch, on", SWITCH_ON, "accent", true),
];

/** What the line is on each control, in words. */
function lineWords(control: Control): string {
  return control.filled ? "the core" : "the tube";
}

/** The controls a move plays on: one of each line, the lit label among them. */
const MOVING: readonly Control[] = [CONTROLS[0]!, CONTROLS[1]!, CONTROLS[2]!, CONTROLS[8]!, CONTROLS[11]!];

/** `controls` side by side, each `shown`. */
function controlsRow(controls: readonly Control[], shown: Shown): HTMLElement {
  return row("s16", "centre", ...controls.map((control) => control.draw(shown)));
}

/** `children` on a surface in `theme` and `mode`, with room for the whole glow round them. */
function room(theme: string, mode: Mode, ...children: Element[]): HTMLElement {
  const box = painted(theme, mode, ...children);
  box.dataset.room = "";
  return box;
}

/** A cell of a sample: its words over it. */
function cell(words: string, ...children: Element[]): HTMLElement {
  return el("div", "f-swatch-cell", label(words, "label"), ...children);
}

/** Every focus token, its value and what it is for. */
const FOCUS_FOR: Record<string, string> = {
  "focus-tube": "The tube's width on a control not filled with its tone, drawn rounded half up to whole device pixels.",
  "focus-near": "How far out the near glow reaches, fading from its opacity at the edge to nothing.",
  "focus-near_opacity": "The near glow's opacity at the control's edge.",
  "focus-far": "How far out the far, faint glow reaches; nothing of the glow is past it.",
  "focus-far_opacity": "The far glow's opacity at the control's edge.",
  "focus-content_near": "How far the faint glow round lit content (a Text button's label and icon) reaches, the same at every size.",
  "focus-content_glow": "That glow's opacity at the letters' and strokes' edge.",
  "focus-offset": "The band a picture keeps round itself inside its own box where its neighbours leave no more room: a wallpaper thumbnail, the boot prompt's field. A focused one's glow is cut past it.",
  "focus-label_near": "How far the tiny glow round a lit label on a fill (a Filled button's, at night) reaches.",
  "focus-label_glow": "That glow's opacity at the letters' edge.",
  "focus-pressed_glow": "A pressed control's glow: the lit glow this many times brighter.",
  "focus-glass": "The signs-off tube of a control not filled with its tone and with no glass role of its own (a field, a checkbox, a chip): the tone's `_unlit` at this opacity over the control's own container. A button's glass is a solid role of its kind (`primary_glass`, `primary_fill_glass`, `burn_danger_glass`).",
};

/** A focus token's value as the tokens write it: a length in logical pixels, or a number. */
function valueOf(name: string): string {
  const written = raw(name);
  return /^[\d.]+$/.test(written) ? String(number(name)) : String(length(name));
}

function focusTokens(): HTMLElement {
  return table(
    ["Token", "Value", "For"],
    names("focus-").map((name) => [
      token(name.replace("-", ".")),
      label(valueOf(name), "mono", "on_surface"),
      prose(FOCUS_FOR[name] ?? "No words for this token: add them to `FOCUS_FOR`.", "body", FOCUS_FOR[name] ? "on_surface" : "error"),
    ]),
  );
}

/** What lights in each tone, as `docs/DESIGN.md` lists it. */
const TONE_LIGHTS: Record<Tone, string> = {
  accent: "Every control without a tone of its own: Filled, Tonal and Text buttons, fields, switches, checkboxes, radios, sliders, chips, lists, tiles.",
  cold: "A ChoiceCard and a picked PictureChoice, where the accent already means picked; a Cold chip.",
  error: "A Danger button or icon button; an Invalid field.",
  success: "A control in the success tone.",
  warning: "A control in the warning tone.",
  sharing: "A control marking what is shared off the machine.",
  burn_danger: "A Danger button, burning.",
  burn_success: "A Success button, burning.",
  burn_warning: "A Warning button, burning.",
};

/** Every tone's tube, its core and its unlit tone, in the theme and mode picked. */
function toneTable(): HTMLElement {
  return table(
    ["Tone", "Tube and glow", "Core, signs on", "Unlit, signs off", "Lights"],
    ALL_TONES.map((tone) => {
      const colour = toneColour(tone);
      return [
        label(tone, "body_strong", "on_surface"),
        el("div", "f-swatch-cell", swatch(colour), token(colour)),
        el("div", "f-swatch-cell", swatch(`${colour}_core`), token(`${colour}_core`)),
        el("div", "f-swatch-cell", swatch(`${colour}_unlit`), token(`${colour}_unlit`)),
        prose(TONE_LIGHTS[tone], "body", "on_surface"),
      ];
    }),
  );
}

/** The glow's opacity at the middle of each logical pixel out from the tube, as squares. */
function glowStrip(): HTMLElement {
  const reach = length("focus-far");
  const squares = Array.from({ length: reach }, (_, d) => {
    const square = el("div", "");
    square.style.setProperty("--alpha", String(glow(d + 0.5)));
    return square;
  });
  const values = Array.from({ length: reach / 4 + 1 }, (_, i) => i * 4).map((d) => [label(`${d}`, "mono", "on_surface"), label(glow(d).toFixed(3), "mono", "on_surface")]);
  return el(
    "div",
    "page-part",
    room(settings().theme, "dark", el("div", "f-glow-strip", el("div", ""), ...squares)),
    table(["Logical pixels out", "Opacity"], values),
  );
}

/** `neon.strike`'s steps as bars, and as a table. */
function strikeBars(): HTMLElement {
  const steps = strikeSteps();
  const duration = milliseconds("medium");
  const bars = el(
    "div",
    "f-steps",
    ...steps.map((step) => {
      const bar = el("div", "f-step");
      bar.style.setProperty("--step", String(step));
      return bar;
    }),
  );
  return el(
    "div",
    "page-part",
    room(settings().theme, "dark", bars),
    table(
      ["Step", "From", "Brightness"],
      steps.map((step, i) => [label(String(i + 1), "mono", "on_surface"), label(`${(i * duration) / steps.length} ms`, "mono", "on_surface"), label(String(step), "mono", "on_surface")]),
    ),
  );
}

/** Every control, lit in the look the mode gives it, each in a box drawn round its own. */
function everyControl(mode: Mode): HTMLElement {
  return room(
    settings().theme,
    mode,
    el(
      "div",
      "f-row",
      ...CONTROLS.map((control) => {
        const boxed = el("div", "f-swatch-cell", label(`${control.name}: ${lineWords(control)}`, "label"));
        const holder = el("div", "", control.draw("lit"));
        holder.dataset.boxed = "";
        boxed.append(holder);
        return boxed;
      }),
    ),
  );
}

/** A row per control: lit at night with the signs on and off, and by day, each a painted box. */
function looks(): HTMLElement {
  const theme = settings().theme;
  return table(
    ["Control", "Dark · signs on", "Dark · signs off", "Light · signs off"],
    CONTROLS.map((control) => [
      label(control.name, "body_strong", "on_surface"),
      room(theme, "dark", control.draw("lit")),
      room(theme, "dark", control.draw("signs-off")),
      room(theme, "light", control.draw("lit")),
    ]),
  );
}

/** Every tone on a tile that is off and on one filled with it, lit with the signs on and off at night, and by day. */
function toneLooks(): HTMLElement {
  const theme = settings().theme;
  const pair = (tone: Tone, mode: Mode, shown: Shown) => room(theme, mode, row("s16", "start", shape(TILE_OFF, tone, false, shown), shape(tileOn(tone), tone, true, shown)));
  return table(
    ["Tone", "Dark · signs on", "Dark · signs off", "Light · signs off"],
    ALL_TONES.map((tone) => [label(tone, "body_strong", "on_surface"), pair(tone, "dark", "lit"), pair(tone, "dark", "signs-off"), pair(tone, "light", "lit")]),
  );
}

/** A Text button at every size, with `words`, each under its size's name. */
function textButtons(words: string, shown: Shown): HTMLElement {
  return el(
    "div",
    "f-row",
    ...SIZES.map((size) => cell(size.toLowerCase(), button({ label: words, kind: "Text", size, icon: "arrow_back", ...pinnedButton(shown) }))),
  );
}

/** Content lit as a tube: a Text button lit with the signs on and off, at the page's scale and at 2. */
function litContent(): HTMLElement {
  const theme = settings().theme;
  return el(
    "div",
    "f-row",
    cell("Dark · signs on", room(theme, "dark", textButtons("Back", "lit"))),
    cell("Dark · signs off", room(theme, "dark", textButtons("Back", "signs-off"))),
    cell("Light · signs off", room(theme, "light", textButtons("Back", "lit"))),
    atScale(2, room(theme, "dark", button({ label: "Back", kind: "Text", size: "Md", icon: "arrow_back", pinned: "lit" }))),
  );
}

/** Where a glow is cut and where it isn't, each in the theme and mode picked. */
function cuts(): HTMLElement {
  const { theme, mode } = settings();
  const field = (shown: Shown) => shape(FIELD, "accent", false, shown);
  const scroll = (scrolled: boolean) => {
    const inside = el("div", "f-cut-scroll", field("lit"), field("rest"), field("rest"));
    inside.toggleAttribute("data-scrolled", scrolled);
    return el("div", "f-cut-panel", inside);
  };
  const card = el("div", "f-cut-card", button({ label: "Erase disk", kind: "Danger", size: "Md", pinned: "lit" }), text("Everything on the disk goes.", "small", "on_surface_variant"));
  const tile = (shown: Shown) => shape({ shape: "tile", corner: "control", width: "var(--size-control_lg)", height: "var(--size-control_lg)", fill: "var(--color-surface_container_highest)" }, "accent", false, shown);
  return el(
    "div",
    "f-row",
    cell("A scroll with no edge of its own", room(theme, mode, scroll(false))),
    cell("A field scrolled half out of it", room(theme, mode, scroll(true))),
    cell("A card's edge", room(theme, mode, card)),
    cell("Tiles drawn in order", room(theme, mode, row("s8", "centre", tile("rest"), tile("lit"), tile("rest")))),
  );
}

/** The window's border: Hyprland's `border_size`, drawn the thin stroke wide. */
const BORDER = "stroke-thin";

/** An app's window on the desktop, `active` or not: its corner the frame's surface less its gap. */
function appWindow(title: string, active: Shown, which?: string): HTMLElement {
  const corner = nested(role("surface"), "s8");
  const drawn = el("div", "f-window", text(title, "body_strong", "on_surface"), text("~/Projects/kubaos", "small", "on_surface_variant"));
  drawn.style.setProperty("--corner", corner.css);
  drawn.tabIndex = 0;
  if (which) drawn.dataset.window = which;
  windowRing(focusRing(drawn, "surface", "accent", false), BORDER, corner.css);
  if (active !== "rest") drawn.dataset.pinned = active;
  return drawn;
}

/** Two windows on the desktop, the first active. */
function desktop(theme: string, mode: Mode, active: Shown = "lit"): HTMLElement {
  const box = painted(theme, mode, el("div", "f-desktop", appWindow("Files", active, "from"), appWindow("Terminal", "rest", "to")));
  box.dataset.room = "";
  return box;
}

/** A control by its name. */
function byName(name: string): Control {
  const found = CONTROLS.find((control) => control.name === name);
  if (!found) throw new Error(`no control ${name}`);
  return found;
}

/**
 * The live sample's rows, each of controls of one height, so none sits off its row's line, and none
 * wider than a field, so the words beside it keep their room at every scale.
 */
const LIVE_ROWS: readonly (readonly string[])[] = [
  ["Tonal button", "Text button"],
  ["Filled button", "Danger button"],
  ["Field"],
  ["Invalid field"],
  ["Checkbox", "Radio"],
  ["Cold chip", "Switch, on"],
  ["Tile, on"],
  ["Picked card"],
];

/** The live sample: every kind of control at rest, to Tab through. */
function live(): HTMLElement {
  const { theme, mode } = settings();
  return room(
    theme,
    mode,
    stack(
      "s16",
      "start",
      prose("Tab through these: each lights up as focus arrives, a fade, and goes back to rest as it leaves.", "small", "on_surface_variant"),
      ...LIVE_ROWS.map((names) => row("s16", "start", ...names.map((name) => byName(name).draw("rest")))),
    ),
  );
}

/** The kinds of a list, as a recipe says them. */
function on(list: readonly Control[]): string {
  return list.map((control) => control.name).join(", ");
}

const EDGED = MOVING.filter((control) => !control.filled);
const FILLED = MOVING.filter((control) => control.filled);
const LIT = MOVING.filter((control) => control.content);

/** The neon's parts from `from` to `to` on every moving control. */
function neon<C extends Curve>(from: number, to: number, duration: Duration, curve: C): NonEmpty<Track<C>> {
  const part = (name: string, list: readonly Control[], target: string): Track<C> => ({ part: name, on: on(list), target, property: "opacity", from, to, duration, curve });
  return [
    part("glow", MOVING, ".k-glow"),
    part("tube", EDGED, ".k-ring:not([data-filled]) > .k-tube"),
    part("core", FILLED, ".k-ring[data-filled] > .k-tube"),
    part("label", LIT, ".k-lit"),
  ];
}

/** By day the signs are off: the glass comes and goes over `short`, the glow and label out. */
function glass(from: number, to: number): NonEmpty<Track<Easing>> {
  const unseen: Record<string, string> = {
    glow: `${on(MOVING)}, unseen: by day the signs are off`,
    label: `${on(LIT)}, unseen: with the signs off the label is its own tone`,
  };
  const [first, ...rest] = neon(from, to, "short", "standard").map((track) => ({ ...track, on: unseen[track.part] ?? track.on }));
  return [first!, ...rest];
}

const moving = () => controlsRow(MOVING, "lit");

/** Focus moving from one field to the next: the line and glow fading out of one as they light the next. */
function focusMoves(): Move {
  const two = () => {
    const [a, b] = [shape(FIELD, "accent", false, "lit"), shape(FIELD, "accent", false, "lit")];
    a.dataset.focus = "from";
    b.dataset.focus = "to";
    return row("s16", "centre", a, b);
  };
  const track = (part: string, which: "from" | "to", target: string, from: number, to: number): Track<Easing> => ({
    part: `${part}, ${which === "from" ? "leaving" : "arriving"}`,
    on: which === "from" ? "the field focus leaves" : "the field focus arrives on",
    target: `[data-focus="${which}"] ${target}`,
    property: "opacity",
    from,
    to,
    duration: "short",
    curve: "standard",
  });
  return {
    name: "Focus moves",
    what: "Tab from one field to the next: the field focus leaves goes back to rest as the next lights up, both fades over `short` on `standard`, so for a moment both are half lit. Focus moving is never a power change, so nothing flickers. By day only the glass moves, the glow out.",
    tracks: [
      track("tube", "from", ".k-tube", 1, 0),
      track("glow", "from", ".k-glow", 1, 0),
      track("tube", "to", ".k-tube", 0, 1),
      track("glow", "to", ".k-glow", 0, 1),
    ],
    reduced: "same",
    daylight: [track("tube", "from", ".k-tube", 1, 0), track("tube", "to", ".k-tube", 0, 1)],
    back: "cuts",
    sample: two,
  };
}

/** Focus moving to another open window: the one that had it goes back to rest as the other lights up. */
function windowLights(): Move {
  const two = () => el("div", "f-desktop", appWindow("Files", "lit", "from"), appWindow("Terminal", "lit", "to"));
  const lit = (which: string) => `[data-window="${which}"] > .k-ring > :is(.k-tube, .k-glow)`;
  const shade = (which: string) => `[data-window="${which}"] > .k-ring > .k-shade`;
  const track = (part: string, on: string, target: string, from: number, to: number): Track<Easing> => ({
    part,
    on,
    target,
    property: "opacity",
    from,
    to,
    duration: "short",
    curve: "standard",
  });
  return {
    name: "The active window lights up",
    what: "Focus moving to another window that is already open: a light-up, not a power change, so it fades as focus between controls does. The window that had it goes back to its inactive look over `short` on `standard` (its neutral border, `surface_container_highest` at `opacity.window_border`, and its neutral shadow, `shadow` at `opacity.window_shadow`), while the border of the one it moves to lights up as the tube, `primary`, its shadow becoming the glow in `primary` with it. The window's content never dims. Only a window appearing strikes its tube on, and a window closing goes off for good, through `neon.strike`. By day only the border's glass moves, the glow out.",
    tracks: [
      track("tube and glow, leaving", "the window focus leaves: its border's tube and the glow its shadow draws", lit("from"), 1, 0),
      track("neutral shadow, back", "the window focus leaves: its shadow in `shadow` at `opacity.window_shadow`", shade("from"), 0, 1),
      track("tube and glow, arriving", "the window focus moves to: its border's tube over the neutral border, its shadow's glow", lit("to"), 0, 1),
      track("neutral shadow, going", "the window focus moves to: the neutral shadow the glow takes the place of", shade("to"), 1, 0),
    ],
    reduced: "same",
    daylight: [
      track("tube and glow, leaving", "the window focus leaves: its border's glass, the glow unseen: by day the signs are off", lit("from"), 1, 0),
      track("tube and glow, arriving", "the window focus moves to: its border's glass, the glow unseen", lit("to"), 0, 1),
    ],
    back: "cuts",
    sample: two,
  };
}

/** A palette: a theme in a mode. */
interface Painted {
  readonly theme: string;
  readonly mode: Mode;
}

/**
 * The scene a sequence plays on: every kind of line lit and an active window, on its own
 * surface painted on its way from `from` to `to` along `path`.
 */
function scene(from: Painted | string, to: Painted | string, path: ColourPath): () => Element {
  return () => {
    const mode = settings().mode;
    const [a, b] = [from, to].map((each) => (typeof each === "string" ? { theme: each, mode } : each)) as [Painted, Painted];
    const painted = el(
      "div",
      "page-scene",
      label("Every lit line and the active window, on one clock", "body_strong", "on_surface"),
      controlsRow(MOVING, "lit"),
      el("div", "f-desktop", appWindow("Files", "lit"), appWindow("Terminal", "rest")),
    );
    painted.dataset.mode = a.mode;
    paintBetween(painted, path, palette(a.theme, a.mode), palette(b.theme, b.mode));
    return el("div", "page-scene-box", painted);
  };
}

function themeMove(from: string, to: string): Move {
  const between = { from: themeName(from), to: themeName(to) };
  return themeChange(`Theme change: ${between.from} to ${between.to}`, between, scene(from, to, "through grey"));
}

/** Each focus length in device pixels at each scale. */
function deviceTable(): HTMLElement {
  const halfUp = (logical: number, scale: number) => Math.max(1, Math.round(logical * scale));
  const rows: (Element | string)[][] = [
    [token("focus.tube"), label("the tube, rounded half up", "body", "on_surface"), ...SCALES.map((scale) => label(String(halfUp(length("focus-tube"), scale)), "mono", "on_surface"))],
    [token("stroke.thin"), label("the core inside a fill's edge, rounded half up", "body", "on_surface"), ...SCALES.map((scale) => label(String(halfUp(length("stroke-thin"), scale)), "mono", "on_surface"))],
    [token("focus.near"), label("the near glow's reach", "body", "on_surface"), ...SCALES.map((scale) => label(String(length("focus-near") * scale), "mono", "on_surface"))],
    [token("focus.far"), label("the far glow's reach, rounded up: the glow's room", "body", "on_surface"), ...SCALES.map((scale) => label(String(Math.ceil(length("focus-far") * scale)), "mono", "on_surface"))],
    [token("focus.content_near"), label("lit content's glow, rounded up", "body", "on_surface"), ...SCALES.map((scale) => label(String(Math.ceil(length("focus-content_near") * scale - 1e-3)), "mono", "on_surface"))],
  ];
  return table(["Length", "What", ...SCALES.map((scale) => `${scale}×`)], rows);
}

/** The text a long-content sample shows, from the toolbar's length. */
const LONG: Words = { short: "Back", long: "Back to the keyboard layout" };

export default foundation({
  level: "foundations",
  slug: "focus",
  title: "Focus",
  summary: "Hovered or focused, a control is lit: a neon line inside its edge, in its tone, with a glow outside it while the signs are on, and the same line as unlit glass while they're off. It lights up with a fade; only a power change flickers.",
  live,
  description: [
    "Hovering a control and focusing it from the keyboard are one state, lit, so focus is always seen: the lit control's edge is a neon line. The line is drawn inside the control's box, at its own corner, so nothing on it reaches past the box but the glow, and its shape never seems to grow. On a control not filled with its tone the line is the tube, `focus.tube` wide rounded half up to device pixels, over the control's own outline, so a field, a checkbox or a card has one line, recoloured. A control filled with its tone (a Filled or Danger button, a tile or a switch that is on) can't show a tube of its own colour, so the line inside the fill's edge is the tone's core, `stroke.thin` wide.",
    "Lit has two looks, and the scene picks one, never the user. Signs on is the neon burning, at night while the lights burn: the tube in its tone, the core half way to white (`_core`), and the glow outside, a near one over `focus.near` and a faint far one over `focus.far`. Signs off is the same line as unlit glass with no glow, by day always and at night while a power change has the lights out. The glass is the tone unlit (`_unlit`, its OKLCH lightness × 0.6 and chroma × 1.2, fitted into sRGB by its chroma alone): a button's is a solid role of its kind (Filled's the unlit fill lifted 0.12 in lightness, Tonal's and Text's the accent unlit, the burning kinds' the tone unlit on their lit fill), and a control with no glass role of its own takes the `_unlit` at `focus.glass` over its container. A lit control is lit with the signs on or off, never gone.",
    "The neon takes the control's tone: a Danger button's line and glow are its burning red, a Cold chip's the cold accent, a control with no tone of its own the accent. Content drawn in the control's own tone with nothing under it (a Text button's label and icon) lights with the line while the signs are on: its letters and strokes are the tone's core with a faint glow round them over `focus.content_near` from `focus.content_glow`; with the signs off, they are their own tone.",
    "A control lights up as focus arrives, a fade over `short` on `standard`, and goes back to rest over `short` as it leaves; focus moving from one control to the next, or to another open window, is the same fade on both. The flicker, `neon.strike`, is for power changes only: the theme change, dark ⇄ light, a surface appearing and a definite off. In those, a lit control flickers down to its signs-off look and strikes back, while a control at rest just recolours. The glow is the control's, so whatever clips the control clips it, but it is cut only at an edge you can see. The active window is in the same family: Hyprland draws its border as the tube and its shadow as the glow.",
  ],
  tokens: [
    { name: "The focus tokens", described: "Every token of the family, its value as the tokens write it, and what it is for.", sample: focusTokens },
    {
      name: "The tones and their cores",
      described: "Each tone's tube and glow, its core with the signs on (the tone 50% toward white) and its unlit tone with them off (OKLCH lightness × 0.6, chroma × 1.2, gamut fitted), in the theme and mode picked: formulas of the live tone, never values. What lights in each.",
      sample: toneTable,
    },
    {
      name: "The glow, pixel by pixel",
      described: "Outward from the tube, a square for each logical pixel at the glow's opacity at its middle: the near and far glows laid over one another, `1 − (1 − near)(1 − far)`, each `opacity × (1 − d / reach)²`, nothing past `focus.far`. The libraries draw it as bands a device pixel apart, so it lands on device pixels at every scale; here it is one mask per control, worked out per device pixel.",
      sample: glowStrip,
    },
    {
      name: "The flicker: neon.strike",
      described: "A sign switching on: its brightness through ten even steps over `medium`, each held until the next. It stutters, then catches. Only a power change plays it: the theme change's and dark ⇄ light's lights striking on (their flicker out plays the steps backwards), a surface or window appearing, and a definite off. Hover, focus, press and focus moving never flicker: they light up with a fade.",
      sample: strikeBars,
    },
  ],
  rules: [
    {
      name: "One line, inside the edge, on every kind of control",
      described: "Every kind of control lit, in the look the mode picked gives it, its own box drawn dashed: the tube inside the edge of an unfilled one (Tonal and Text buttons, a field, an invalid field in red, a checkbox, a radio, a Cold chip, a picked card), the core inside the fill's edge of one filled with its tone (Filled and Danger buttons, a tile and a switch that are on). Nothing but the glow is past the dashed box. Tab through the controls in the description to light them live.",
      sample: () => everyControl("dark"),
    },
    {
      name: "The neon takes the control's tone",
      described: "Each tone on a tile that is off (its tube) and on one filled with it (its core): with the signs on, the tube and glow in the tone, the core its tone half way to white; with them off, the tube is the tone's `_unlit` at `focus.glass` over the tile and the core the `_unlit` itself, with no glow.",
      sample: toneLooks,
    },
    {
      name: "Content in its tone lights as a tube",
      described: "A Text button at sm, md and lg. Lit with the signs on, its label and icon burn as `primary_core` under a faint near glow, `focus.content_near` from `focus.content_glow`, the same at every size, and no far glow; with the signs off, they are its rest label, `primary_rest`, and by day `primary`. At 2, the glow's three logical pixels are six device pixels round every letter. A Filled button's label takes only a tiny glow (`focus.label_near` from `focus.label_glow`); Tonal's and the burning kinds' labels don't glow.",
      sample: litContent,
    },
    {
      name: "Where a glow is cut, and where it isn't",
      described: "The glow is the control's, so whatever clips the control clips it, but it is cut only at an edge you can see. A scroll with no visible edge of its own (a page's, a panel's: its box dashed) reaches `focus.far` past its box, so a resting field at its top keeps its whole glow; a field scrolled half out of it shows its glow trimmed where the scroll cuts. A card's, a well's, a menu's or a panel's own edge cuts it. In a grid, the tile drawn after the focused one covers its glow, and the one before shows under it.",
      sample: cuts,
    },
    {
      name: "The active window's tube",
      described: "Hyprland draws the active window's border as the tube, `primary` alone, over the neutral border (`surface_container_highest` at `opacity.window_border`), and its shadow as the glow in `primary`, falling off over `focus.far`; an inactive window keeps its neutral border and its shadow, `shadow` at `opacity.window_shadow`. The border is Hyprland's `border_size`, 2, drawn here the thin stroke wide, at the window's outer corner, the frame's surface radius less its 8 gap. Its content never dims.",
      sample: () => el("div", "f-row", ...MODES.map((mode) => cell(modeName(mode), desktop(settings().theme, mode)))),
    },
  ],
  modes: [
    {
      name: "Signs on and off, in dark and light",
      described: "Every kind of control lit: at night the signs are on while the lights burn and off while a power change has them out; by day they are always off. The geometry is the same in every look.",
      sample: looks,
    },
  ],
  themes: [
    {
      name: "The lines in every theme",
      described: "One of each kind of line in each theme the toolbar paints in, in the mode picked: the tube, the lit label, the core on a fill, a burning Danger, a cold chip. Every core is a formula of the theme's tone.",
      sample: () =>
        el(
          "div",
          "page-part",
          ...themes().map((theme) =>
            cell(
              theme.name,
              room(
                theme.slug,
                settings().mode,
                el("div", "f-row", ...["Tonal button", "Text button", "Field", "Filled button", "Danger button"].map((name) => byName(name).draw("lit"))),
                el("div", "f-row", ...["Cold chip", "Switch, on"].map((name) => byName(name).draw("lit"))),
              ),
            ),
          ),
        ),
    },
    {
      name: "The active window in every theme",
      described: "The tube and its glow are the theme's `primary`; the neutral border and shadow follow the theme's containers.",
      sample: () => el("div", "f-row", ...themes().map((theme) => cell(theme.name, desktop(theme.slug, settings().mode)))),
    },
  ],
  scales: [
    {
      name: "Every length in device pixels",
      described: "The tube rounds half up to whole device pixels, 2, 2, 2, 3; the core is the thin stroke, 2, 3, 3, 4; the glow's reach is its room, rounded up, and its mask is worked out per device pixel, so it lands on them at every scale.",
      sample: deviceTable,
    },
    {
      name: "The line at every scale",
      described: "Each box is drawn at its own scale, whatever the toolbar's: the tube, the core and the lit label, with the signs on.",
      sample: () =>
        el(
          "div",
          "f-scroll",
          el("div", "f-row", ...SCALES.map((scale) => atScale(scale, room(settings().theme, "dark", controlsRow([CONTROLS[0]!, CONTROLS[1]!, CONTROLS[8]!], "lit"))))),
        ),
    },
  ],
  content: [
    {
      name: "Long words, lit",
      described: "A Text button's lit label in the toolbar's text length: the glow follows every letter, however many, and the tube keeps to the button's box as it grows. Pick long or translated in the toolbar.",
      sample: () => cell("Dark · signs on", room(settings().theme, "dark", textButtons(shown(LONG), "lit"))),
    },
  ],
  motion: {
    arrives: [
      {
        name: "Focus arrives",
        what: "Hovered or focused, the control lights up: a fade over `short` on `standard`, never the flicker, which is for power changes. The tube, the core, the glow and the lit label fade in together: on a fill the core inside the fill's edge, on a Text button the label burning up to the core with its glow. By day the signs are off: only the glass fades in, and the glow and the lit label stay out.",
        tracks: neon(0, 1, "short", "standard"),
        reduced: "same",
        daylight: glass(0, 1),
        back: "cuts",
        sample: moving,
      },
    ],
    leaves: [
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: the tube, the core, the glow and the lit label fade out together over `short` on `standard`, the label back to its own tone; by day the glass alone.",
        tracks: neon(1, 0, "short", "standard"),
        reduced: "same",
        daylight: glass(1, 0),
        back: "cuts",
        sample: moving,
      },
    ],
    between: [
      focusMoves(),
      windowLights(),
      themeMove("neon", "rose"),
      themeMove("lagoon", "moss"),
      toLight({ from: paletteName("neon", "dark"), to: paletteName("neon", "light") }, scene({ theme: "neon", mode: "dark" }, { theme: "neon", mode: "light" }, "dawn")),
      toDark({ from: paletteName("neon", "light"), to: paletteName("neon", "dark") }, scene({ theme: "neon", mode: "light" }, { theme: "neon", mode: "dark" }, "dawn")),
    ],
  },
  checks: [
    { check: "The contrast check", holds: "Holds the tube against every surface it is drawn on (the page, a well, a panel, a card or a dialog, a field's fill, a picked card's), in the accent, the cold accent and the error colour (`[contrast]`)." },
    { check: "The geometry check", holds: "Compares a focused control's box with its resting one: the tube and the glow take no room, so focus never moves layout." },
    { check: "kubaos-tokens", holds: "Derives every `_core`, `_unlit` and `_glass` from its live tone." },
    { check: "kubaos-ui and KubaOS.Components", holds: "Draw the glow as bands from the formula, one device pixel apart. Not built yet: the libraries still strike focus through `neon.strike`, draw the core outside a fill, light the neon in both modes, light Danger in the accent and leave a Text button's label as it is, and Hyprland strikes the active window's border; the mockups draw the rule, and the libraries take it with the rest of the neon change." },
  ],
});
