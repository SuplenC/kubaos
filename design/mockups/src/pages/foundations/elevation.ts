// The Elevation foundation: the planes every surface lies on and the shadow each casts, the ladder
// a part rises on inside a window or a panel, the hairline, the thin stroke and the symbol stroke,
// each in device pixels at every scale (`docs/DESIGN.md`, "Space and shape" and "Iconography").

import { button } from "../../lib/button.ts";
import { card as libraryCard } from "../../lib/card.ts";
import { raise, type Step } from "../../lib/elevation.ts";
import { focusRing, windowRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { stack } from "../../lib/layout.ts";
import { menu } from "../../lib/menu.ts";
import { nested, role } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { popover, tooltip } from "../../lib/tooltip.ts";
import { length, names, number } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, themes, token } from "../../site/parts/swatches.ts";
import { type Mode, SCALES, settings } from "../../site/settings.ts";
import { across } from "./across.ts";

/** Every shadow of `[shadow]`, by name: what the shell and the apps cast today. */
function shadows(): string[] {
  return names("shadow-")
    .filter((name) => name.endsWith("-y"))
    .map((name) => name.slice("shadow-".length, -"-y".length));
}

/** What each `[shadow]` is for, and whether it is cast in its fill's own colour rather than `shadow`. */
const SHADOW_FOR: Record<string, { for: string; colour: string }> = {
  panel: { for: "The shell plane: the bar, a panel, a notification, the OSD", colour: "shadow" },
  dialog: { for: "A dialog, until it takes step 3", colour: "shadow" },
  tucked: { for: "A tucked notification, until it takes step 1", colour: "shadow" },
  lifted: { for: "A lifted notification, until it takes step 2", colour: "shadow" },
  glow: { for: "A progress bar's glow: light, not a shadow", colour: "primary" },
};

/** A step's own value of the token `--name`, read in `mode` off a box painted in it. */
function inMode(mode: Mode, name: string): string {
  const probe = el("div", "");
  probe.dataset.mode = mode;
  document.body.append(probe);
  const value = getComputedStyle(probe).getPropertyValue(`--${name}`).trim();
  probe.remove();
  return value;
}

/** A token's value in each mode, once when they agree. */
function bothModes(name: string): string {
  const [dark, light] = MODES.map((mode) => inMode(mode, name));
  return dark === light ? dark! : `${dark} night, ${light} day`;
}

/** Every step of the ladder over the plane's surface, as the tokens list them. */
function steps(): Step[] {
  return Array.from({ length: number("elevation-steps") }, (_, index) => (index + 1) as Step);
}

/** What stands at each step, and its container role by day; at night it takes the role above. */
const STEP_FOR: Record<Step, { what: string; day: string; night: string }> = {
  1: { what: "A tucked notification; a card being dragged", day: "surface_container_high", night: "surface_container_highest" },
  2: { what: "A menu, a popover, a toast, a lifted notification", day: "surface_container", night: "surface_container_high" },
  3: { what: "A dialog inside an app or a panel", day: "surface_container_high", night: "surface_container_highest" },
  4: { what: "A tooltip", day: "surface_container", night: "surface_container_high" },
};

/** `element` raised to `step` on its step's fill. */
function raised<E extends HTMLElement>(element: E, step: Step): E {
  element.style.setProperty("--fill", `var(--color-${STEP_FOR[step].day})`);
  element.style.setProperty("--fill-night", `var(--color-${STEP_FOR[step].night})`);
  return raise(element, step);
}

/** A card, flat on the surface it is in: the library's, a container role above that surface. */
function card(): HTMLElement {
  const flat = libraryCard({ body: [stack("s4", "start", text("Projects", "body_strong", "on_surface"), text("12 folders · 3.4 GB", "small", "on_surface_variant"))] });
  flat.classList.add("f-elev-card");
  return flat;
}

/** An open menu at step 2: the library's popover holding a few items. */
function openMenu(): HTMLElement {
  return popover(2, menu([{ label: "Open" }, { label: "Rename" }, { label: "Move to trash" }], { current: -1 }));
}

/** The window's border: Hyprland's `border_size`, drawn the thin stroke wide. */
const BORDER = "stroke-thin";

/**
 * A window on the desktop plane, with its constant neutral shadow; `active` lights its border and
 * glow, the focus family, over that shadow.
 */
function appWindow(which: "front" | "back", title: string, active: boolean, ...content: Element[]): HTMLElement {
  const corner = nested(role("surface"), "s8");
  const drawn = el("div", "f-elev-window", text(title, "body_strong", "on_surface"), ...content);
  drawn.dataset.window = which;
  drawn.style.setProperty("--corner", corner.css);
  if (active) {
    windowRing(focusRing(drawn, "surface", "accent", false), BORDER, corner.css);
    drawn.dataset.pinned = "lit";
  }
  return drawn;
}

/**
 * The desktop: two windows on the wallpaper, the front one active with a card, an open menu and a
 * tooltip over a button inside it; the shell's bar along the top and a panel growing out of it over
 * the window, the same card in it. It takes the mode of the surface it is drawn on.
 */
function desktop(): HTMLElement {
  const tip = tooltip("Sort by name", "Folders first, then files.");
  tip.classList.add("f-elev-tip");
  const sort = button({ label: "Sort", kind: "Tonal", size: "Sm" });
  sort.classList.add("f-elev-sort");
  const open = openMenu();
  open.classList.add("f-elev-menu");
  const front = appWindow("front", "Files", true, card(), open, tip, sort);
  const back = appWindow("back", "Terminal", false, text("~/Projects/kubaos", "small", "on_surface_variant"));
  const bar = el("div", "f-elev-bar", text("14:05", "label", "on_surface"));
  const panel = el("div", "f-elev-panel", text("Recent", "subheading", "on_surface"), card());
  return el("div", "f-elev-desktop", back, front, bar, panel);
}

/** The desktop painted in `theme` and `mode`, under the mode's name. */
function scene(theme: string, mode: Mode): HTMLElement {
  return el("div", "f-swatch-cell", label(modeName(mode), "label"), painted(theme, mode, el("div", "f-scroll", desktop())));
}

/** A slab seen from its edge, on its plane's row, casting its plane's shadow. */
function slab(kind: string, words: string): HTMLElement {
  return el("div", `f-elev-slab f-elev-slab-${kind}`, label(words, "caption", "on_surface"));
}

/** The planes seen from the side, top to bottom: the lock screen, the shell, the desktop. */
function planes(): HTMLElement {
  const plane = (name: string, casts: string, ...slabs: HTMLElement[]) =>
    el("div", "f-elev-plane", stack("s4", "start", text(name, "body_strong", "on_surface"), prose(casts, "small", "on_surface_variant")), el("div", "f-elev-slabs", ...slabs));
  return painted(
    settings().theme,
    settings().mode,
    el(
      "div",
      "f-elev-planes",
      plane("Lock screen", "Casts nothing.", slab("lock", "lock screen")),
      plane(
        "Shell",
        `\`shadow.panel\`, ${length("shadow-panel-y")} down, ${length("shadow-panel-blur")} blur, over anything.`,
        slab("bar", "bar"),
        slab("panel", "panel"),
        slab("osd", "OSD"),
        slab("prompt", "password prompt"),
      ),
      plane(
        "Desktop",
        `One neutral shadow a window, ${length("shadow-panel-y")} down, ${length("shadow-panel-blur")} blur.`,
        slab("window", "window"),
        slab("window", "window"),
        slab("wallpaper", "wallpaper"),
      ),
    ),
  );
}

/** One step's sample: a part at that step on the plane's surface. */
function stepSample(step: Step | 0): HTMLElement {
  const part = el("div", "f-elev-step", text(String(step), "label", "on_surface"));
  return step === 0 ? part : raised(part, step);
}

/** The ladder: each step's layers, read from the tokens, its fill by day and at night, and a sample. */
function ladderTable(): HTMLElement {
  const layer = (step: Step, which: "contact" | "ambient") =>
    label(`${length(`elevation-${step}-${which}-y`)} down, ${length(`elevation-${step}-${which}-blur`)} blur`, "mono", "on_surface");
  return across(table(
    ["Step", "Stands there", "Contact", "Ambient", "Day", "Night", "Sample"],
    [
      [label("0", "mono", "on_surface"), prose("The surface itself: its plane's shadow alone", "body", "on_surface"), "—", "—", "—", "—", painted(settings().theme, settings().mode, stepSample(0))],
      ...steps().map((step) => [
        label(String(step), "mono", "on_surface"),
        prose(STEP_FOR[step].what, "body", "on_surface"),
        layer(step, "contact"),
        layer(step, "ambient"),
        token(STEP_FOR[step].day),
        token(STEP_FOR[step].night),
        painted(settings().theme, settings().mode, stepSample(step)),
      ]),
    ],
  ));
}

/** The layers' and the rim's opacities, night and day. */
function opacityTable(): HTMLElement {
  return across(table(
    ["Token", "Night", "Day", "Of"],
    [
      ["elevation-contact-opacity", "shadow"],
      ["elevation-ambient-opacity", "shadow"],
      ["elevation-rim", "on_surface"],
    ].map(([name, of]) => [token(name!.replace("-", ".").replaceAll("-", "_")), label(inMode("dark", name!), "mono", "on_surface"), label(inMode("light", name!), "mono", "on_surface"), token(of!)]),
  ));
}

/** Every `[shadow]`'s measures by token and its opacity in each mode. */
function shadowTable(): HTMLElement {
  return across(table(
    ["Shadow", "Down", "Blur", "Opacity", "Of", "For"],
    shadows().map((shadow) => [
      token(`shadow.${shadow}`),
      label(String(length(`shadow-${shadow}-y`)), "mono", "on_surface"),
      label(String(length(`shadow-${shadow}-blur`)), "mono", "on_surface"),
      label(bothModes(`shadow-${shadow}-opacity`), "mono", "on_surface"),
      token(SHADOW_FOR[shadow]?.colour ?? "shadow"),
      prose(SHADOW_FOR[shadow]?.for ?? "No words for this shadow: add them to `SHADOW_FOR`.", "body", SHADOW_FOR[shadow] ? "on_surface" : "error"),
    ]),
  ));
}

/** The same open menu in a window and in a panel: the ladder restarts in each, so they are one. */
function sameInBoth(): HTMLElement {
  const { theme, mode } = settings();
  const inWindow = el("div", "f-elev-host f-elev-host-window", label("In a window", "label"), openMenu());
  const inPanel = el("div", "f-elev-host f-elev-host-panel", label("In a panel", "label"), openMenu());
  return painted(theme, mode, el("div", "f-elev-hosts", inWindow, inPanel));
}

/** Each step in every theme, so the night's lighter fill is judged in every accent. */
function ladderIn(theme: { slug: string; name: string }): HTMLElement {
  return el("div", "f-swatch-cell", label(theme.name, "label"), painted(theme.slug, settings().mode, el("div", "f-elev-steps", ...steps().map(stepSample))));
}

/** Lines one device pixel thick: a divider in a card, a field's outline, a switch's track. */
function hairlines(): HTMLElement {
  return stack(
    "s16",
    "start",
    el("div", "f-divided", text("Wi-Fi", "body_strong", "on_surface"), el("div", "f-divider"), text("Bluetooth", "body_strong", "on_surface")),
    el("div", "f-outlined", text("Passphrase", "body_large", "on_surface_variant")),
    el("div", "f-track-outline"),
  );
}

/** Lines `stroke.thin` thick, rounded half up to device pixels: a caret, a partial checkbox's dash, a spinner's arc, a drop line. */
function thinStrokes(): HTMLElement {
  return el(
    "div",
    "f-row",
    el("div", "f-swatch-cell", label("caret", "label"), el("div", "f-caret-field", text("hunter", "body_large", "on_surface"), el("div", "f-caret"))),
    el("div", "f-swatch-cell", label("partial checkbox", "label"), el("div", "f-check", el("div", "f-dash"))),
    el("div", "f-swatch-cell", label("spinner", "label"), el("div", "f-spinner")),
    el("div", "f-swatch-cell", label("drop line", "label"), el("div", "f-drop")),
  );
}

/** A symbol's stroke in device pixels at `size` logical pixels and `scale`: 2 of its 24 units, rounded half up, never under one. */
function symbolStroke(size: number, scale: number): number {
  return Math.max(1, Math.round((size * scale * number("stroke-symbol")) / 24));
}

/** Every icon size's symbol stroke at every scale, beside the symbol drawn at the page's. */
function symbolStrokes(): HTMLElement {
  const sizes = names("icon-s");
  return across(table(
    ["Icon", "Drawn", ...SCALES.map((scale) => `${scale}×`)],
    sizes.map((name) => [
      token(name.replace("-", "."), String(length(name))),
      icon("settings", `var(--${name})`, "var(--color-on_surface)"),
      ...SCALES.map((scale) => label(String(symbolStroke(length(name), scale)), "mono", "on_surface")),
    ]),
  ));
}

/** Every length of the family in device pixels at every scale: hairline, thin stroke, the ladder's layers, the shadows. */
function deviceTable(): HTMLElement {
  const thin = (scale: number) => Math.max(1, Math.round(length("stroke-thin") * scale));
  const atEvery = (name: string, shown: string) => [
    token(shown),
    ...SCALES.map((scale) => {
      const value = length(name) * scale;
      return label(String(value), "mono", Number.isInteger(value) ? "on_surface" : "error");
    }),
  ];
  const rows: (Element | string)[][] = [
    [token("stroke.hairline_device_px"), ...SCALES.map(() => label(String(length("stroke-hairline_device_px")), "mono", "on_surface"))],
    [token("stroke.thin"), ...SCALES.map((scale) => label(String(thin(scale)), "mono", "on_surface"))],
    ...steps().flatMap((step) =>
      (["contact", "ambient"] as const).flatMap((layer) =>
        (["y", "blur"] as const).map((part) => atEvery(`elevation-${step}-${layer}-${part}`, `elevation.${layer}.${part} at step ${step}`)),
      ),
    ),
    ...shadows().flatMap((shadow) => (["y", "blur"] as const).map((part) => atEvery(`shadow-${shadow}-${part}`, `shadow.${shadow}.${part}`))),
  ];
  return across(table(["Length", ...SCALES.map((scale) => `${scale}×`)], rows));
}

export default foundation({
  level: "foundations",
  slug: "elevation",
  title: "Elevation, hairlines and strokes",
  lede: "Two planes, each casting one fixed shadow; inside every window and panel a ladder of four steps; hairlines and strokes on device pixels.",
  stage: { live: desktop, hint: "compare the panes: at night raised parts sit a role lighter", daylight: true },
  tokens: [
    { name: "The ladder", note: "Only what floats rises: each step's two layers, its fill by day and night, a part on it.", sample: ladderTable },
    { name: "The ladder's opacities", note: "The layers' of `shadow`, the rim's of the colour of its words.", sample: opacityTable },
    { name: "The shell's and apps' shadows", note: "`[shadow]`: the shell plane's, the progress bar's glow, and those still to take the ladder.", sample: shadowTable },
    { name: "The hairline", note: "One device pixel at every scale, the one length off the grid.", sample: hairlines },
    { name: "The thin stroke", note: "`stroke.thin`, rounded half up to whole device pixels.", sample: thinStrokes },
    { name: "The symbol stroke", note: "2 of 24 units, rounded half up; 16 at 1× stays 1, so small symbols stay light.", sample: symbolStrokes },
  ],
  rules: [
    { name: "The planes, from the side", note: "The lock over everything, the shell over every window, the windows on the wallpaper.", sample: planes },
    { name: "The ladder restarts", note: "A menu at step 2 is the same in a window and a panel.", sample: sameInBoth },
  ],
  modes: [
    {
      name: "The desktop, night and day",
      note: "At night raised parts sit a role lighter, with stronger layers and a brighter rim.",
      sample: () => el("div", "f-elev-scenes", ...MODES.map((mode) => scene(settings().theme, mode))),
    },
  ],
  themes: [
    {
      name: "The ladder",
      note: "The fills are the theme's containers; the shadow is `shadow` in every theme.",
      sample: () => el("div", "f-row", ...themes().map(ladderIn)),
    },
  ],
  scales: [
    { name: "Every length", note: "The hairline stays one; the thin stroke goes 2, 3, 3, 4; every layer stays whole.", sample: deviceTable },
    {
      name: "The lines",
      note: "Each box at its own scale, whatever the top bar picks.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, hairlines(), thinStrokes())))),
    },
  ],
  content: doesntApply("a shadow or a line holds no words."),
  motion: doesntApply("a shadow moves only with what casts it; a notification group's page plays that."),
  checks: [
    { check: "kubaos-tokens", holds: "refuses offsets or blurs off the grid, opacities outside 0 to 1" },
    { check: "The geometry check", holds: "holds every box, outline included, to whole device pixels" },
    { check: "kubaos-ui and KubaOS.Components", holds: "draw a symbol's stroke with `icon::device_stroke`, no line fractional" },
  ],
});
