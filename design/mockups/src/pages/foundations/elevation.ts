// The Elevation foundation: the planes every surface lies on and the shadow each casts, the ladder
// a part rises on inside a window or a panel, the hairline, the thin stroke and the symbol stroke,
// each in device pixels at every scale (`docs/DESIGN.md`, "Space and shape" and "Iconography").

import { button } from "../../lib/button.ts";
import { raise, type Step } from "../../lib/elevation.ts";
import { focusRing, windowRing } from "../../lib/focus.ts";
import { icon } from "../../lib/icon.ts";
import { stack } from "../../lib/layout.ts";
import { nested, role } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { popover, tooltip } from "../../lib/tooltip.ts";
import { length, names, number } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, themes, token } from "../../site/parts/swatches.ts";
import { type Mode, SCALES, settings } from "../../site/settings.ts";

/** Every shadow of `[shadow]`, by name: what the shell and the apps cast today. */
function shadows(): string[] {
  return names("shadow-")
    .filter((name) => name.endsWith("-y"))
    .map((name) => name.slice("shadow-".length, -"-y".length));
}

/** What each `[shadow]` is for, and whether it is cast in its fill's own colour rather than `shadow`. */
const SHADOW_FOR: Record<string, { for: string; colour: string }> = {
  panel: { for: "The shell plane over the windows and the wallpaper: the bar, a panel, a notification, the OSD, a system dialog.", colour: "shadow" },
  dialog: { for: "A dialog over its scrim, as the shell and the apps draw it until they take the ladder's step 3.", colour: "shadow" },
  tucked: { for: "A notification group's card tucked under the one above it, as the shell draws it until it takes the ladder's step 1.", colour: "shadow" },
  lifted: { for: "A notification group's card lifted over the list, as the shell draws it until it takes the ladder's step 2.", colour: "shadow" },
  glow: { for: "The glow under a progress bar's fill, in the fill's own colour: light, not a shadow.", colour: "primary" },
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
  1: { what: "A card; a notification tucked under another.", day: "surface_container_high", night: "surface_container_highest" },
  2: { what: "A menu, a popover; a notification lifted over the list.", day: "surface_container", night: "surface_container_high" },
  3: { what: "A dialog inside an app or a panel.", day: "surface_container_high", night: "surface_container_highest" },
  4: { what: "A tooltip.", day: "surface_container", night: "surface_container_high" },
};

/** `element` raised to `step` on its step's fill. */
function raised<E extends HTMLElement>(element: E, step: Step): E {
  element.style.setProperty("--fill", `var(--color-${STEP_FOR[step].day})`);
  element.style.setProperty("--fill-night", `var(--color-${STEP_FOR[step].night})`);
  return raise(element, step);
}

/** A card at step 1: what the scene puts in a window and in a panel alike. */
function card(): HTMLElement {
  return raised(el("div", "f-elev-card", text("Projects", "body_strong", "on_surface"), text("12 folders · 3.4 GB", "small", "on_surface_variant")), 1);
}

/** An open menu at step 2: the library's popover holding a few rows. */
function menu(): HTMLElement {
  const rows = ["Open", "Rename", "Move to trash"].map((words) => el("div", "f-elev-menu-row", text(words, "body", "on_surface")));
  const open = popover(2, stack(0, "start", ...rows));
  open.classList.add("f-elev-menu");
  return open;
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
 * The desktop in `mode`: two windows on the wallpaper, the front one active with a card, an open
 * menu and a tooltip over a button inside it; the shell's bar along the top and a panel growing
 * out of it over the window, the same card in it.
 */
function scene(theme: string, mode: Mode): HTMLElement {
  const tip = tooltip("Sort by name", "Folders first, then files.");
  tip.classList.add("f-elev-tip");
  const sort = button({ label: "Sort", kind: "Tonal", size: "Sm" });
  sort.classList.add("f-elev-sort");
  const front = appWindow("front", "Files", true, card(), menu(), tip, sort);
  const back = appWindow("back", "Terminal", false, text("~/Projects/kubaos", "small", "on_surface_variant"));
  const bar = el("div", "f-elev-bar", text("14:05", "label", "on_surface"));
  const panel = el("div", "f-elev-panel", text("Recent", "subheading", "on_surface"), card());
  const box = painted(theme, mode, el("div", "f-scroll", el("div", "f-elev-desktop", back, front, bar, panel)));
  return el("div", "f-swatch-cell", label(modeName(mode), "label"), box);
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
      plane("Lock screen", "Above everything; casts nothing.", slab("lock", "lock screen")),
      plane(
        "Shell",
        `Above every window: \`shadow.panel\`, ${length("shadow-panel-y")} down, ${length("shadow-panel-blur")} blur, over a window or the wallpaper alike.`,
        slab("bar", "bar"),
        slab("panel", "panel"),
        slab("osd", "OSD"),
        slab("prompt", "password prompt"),
      ),
      plane(
        "Desktop",
        `The wallpaper and the windows: one neutral shadow on every window, ${length("shadow-panel-y")} down, ${length("shadow-panel-blur")} blur at \`opacity.window_shadow\`.`,
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
  return table(
    ["Step", "Stands there", "Contact", "Ambient", "Day", "Night", "Sample"],
    [
      [label("0", "mono", "on_surface"), prose("The window's or the panel's own surface: its plane's shadow alone.", "body", "on_surface"), "—", "—", "—", "—", painted(settings().theme, settings().mode, stepSample(0))],
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
  );
}

/** The layers' and the rim's opacities, night and day. */
function opacityTable(): HTMLElement {
  return table(
    ["Token", "Night", "Day", "Of"],
    [
      ["elevation-contact-opacity", "shadow"],
      ["elevation-ambient-opacity", "shadow"],
      ["elevation-rim", "on_surface"],
    ].map(([name, of]) => [token(name!.replace("-", ".").replaceAll("-", "_")), label(inMode("dark", name!), "mono", "on_surface"), label(inMode("light", name!), "mono", "on_surface"), token(of!)]),
  );
}

/** Every `[shadow]`'s measures by token and its opacity in each mode. */
function shadowTable(): HTMLElement {
  return table(
    ["Shadow", "Down", "Blur", "Opacity", "Of", "For"],
    shadows().map((shadow) => [
      token(`shadow.${shadow}`),
      label(String(length(`shadow-${shadow}-y`)), "mono", "on_surface"),
      label(String(length(`shadow-${shadow}-blur`)), "mono", "on_surface"),
      label(bothModes(`shadow-${shadow}-opacity`), "mono", "on_surface"),
      token(SHADOW_FOR[shadow]?.colour ?? "shadow"),
      prose(SHADOW_FOR[shadow]?.for ?? "No words for this shadow: add them to `SHADOW_FOR`.", "body", SHADOW_FOR[shadow] ? "on_surface" : "error"),
    ]),
  );
}

/** The same card in a window and in a panel: the ladder restarts in each, so they are one. */
function sameInBoth(): HTMLElement {
  const { theme, mode } = settings();
  const inWindow = el("div", "f-elev-host f-elev-host-window", label("In a window", "label"), card());
  const inPanel = el("div", "f-elev-host f-elev-host-panel", label("In a panel", "label"), card());
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
  return table(
    ["Icon", "Drawn", ...SCALES.map((scale) => `${scale}×`)],
    sizes.map((name) => [
      token(name.replace("-", "."), String(length(name))),
      icon("settings", `var(--${name})`, "var(--color-on_surface)"),
      ...SCALES.map((scale) => label(String(symbolStroke(length(name), scale)), "mono", "on_surface")),
    ]),
  );
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
  return table(["Length", ...SCALES.map((scale) => `${scale}×`)], rows);
}

export default foundation({
  level: "foundations",
  slug: "elevation",
  title: "Elevation, hairlines and strokes",
  summary: "Two planes, the desktop and the shell, each casting one fixed shadow; inside every window and panel a ladder of four steps, each one shadow from the plane's surface; hairlines one device pixel and strokes whole device pixels at every scale.",
  live: () => scene(settings().theme, settings().mode),
  description: [
    "Every surface lies on a plane. The desktop plane is the wallpaper and the windows: every window casts one constant neutral shadow, and the active window's glow is the focus family's, laid over it, never changing it. The shell plane is the bar and the frame, the panels, the notifications, the OSD and the system dialogs (the password prompt, the share-screen prompt): always above every window, so it casts the same shadow over a window as over the wallpaper. The lock screen is above everything and casts nothing.",
    "Inside every window and every panel a part rises on the ladder, which restarts from 0 in each, so a card in a window and a card in a panel are one: 0 the window's or the panel's own surface, 1 a card or a tucked notification, 2 a menu, a popover or a lifted notification, 3 a dialog inside an app or a panel, 4 a tooltip. Each step casts one shadow measured from the plane's surface, not from what it lies on, in two layers of `shadow`: a tight contact layer, 4 blur at every step, and an ambient one 4 down and 16 blur per step (`[elevation]`). A menu or a tooltip that reaches past its window's edge keeps its step's shadow.",
    "At night the layers are stronger (0.70 and 0.80) and a raised part takes the container role one above its day one, a card `surface_container_highest` where by day it is `surface_container_high`; by day they are 0.24 and 0.28. Every raised part draws a rim, a hairline of `on_surface` inside its edge, 8 % at night and 6 % by day. The surfaces come from the wallpaper, so the roles are named, never a colour.",
    "A hairline is one device pixel at every scale (`stroke.hairline_device_px`): dividers, the outlines of cards, fields and wells, a switch's track, the rim. It is the one length not on the grid.",
    "The thin stroke, 2, is drawn as whole device pixels rounded half up: a spinner's arc, a field's caret, a partial checkbox's dash, the line where a dragged row will land. A symbol's stroke is 2 of its 24 units scaled to its size, rounded half up and never under one, so no line of an icon falls between device pixels.",
  ],
  tokens: [
    {
      name: "The ladder",
      described: "Each step's two layers as the tokens give them, the container role it takes by day and at night, and a part at that step on a window's surface, in the mode picked.",
      sample: ladderTable,
    },
    {
      name: "The ladder's opacities",
      described: "The contact and ambient layers' opacities of `shadow` and the rim's of `on_surface`, at night and by day.",
      sample: opacityTable,
    },
    {
      name: "The shadows the shell and the apps cast",
      described: "`[shadow]`: the shell plane's `panel`, the progress bar's `glow`, and the dialog's and the notification cards' shadows the shell and `kubaos-ui` still read until they take the ladder.",
      sample: shadowTable,
    },
    {
      name: "The hairline",
      described: "One device pixel at every scale: a divider in a list, a field's outline, a switch's track.",
      sample: hairlines,
    },
    {
      name: "The thin stroke",
      described: "`stroke.thin` rounded half up to device pixels: a caret beside the words typed, a partial checkbox's dash, a spinner's arc over its faint ring, a drop line.",
      sample: thinStrokes,
    },
    {
      name: "The symbol stroke",
      described: "Each icon size's stroke in device pixels at each scale, and the symbol drawn at the page's scale. 16 at 1× stays 1: 1.33 rounds to 1, and 2 would make the small symbols heavier than the text beside them.",
      sample: symbolStrokes,
    },
  ],
  rules: [
    {
      name: "The planes, from the side",
      described: "Top to bottom: the lock screen over everything, casting nothing; the shell's surfaces, one fixed shadow wherever they lie; the windows on the wallpaper, one neutral shadow each.",
      sample: planes,
    },
    {
      name: "The ladder restarts in every window and panel",
      described: "The same card at step 1 in a window and in a panel: its shadow is measured from the surface it rises from, so it is the same in both.",
      sample: sameInBoth,
    },
  ],
  modes: [
    {
      name: "The desktop, night and day",
      described: "Two windows on the wallpaper, the front one active, its glow over its constant shadow; in it a card (1), an open menu (2) and a tooltip over its button (4); the bar along the top and a panel over the window, the same card in it. At night the parts sit a role lighter, with stronger layers and a brighter rim.",
      sample: () => el("div", "f-elev-scenes", ...MODES.map((mode) => scene(settings().theme, mode))),
    },
  ],
  themes: [
    {
      name: "The ladder in every theme",
      described: "Steps 1 to 4 on a window's surface: the fills are the theme's own containers, so the night's step up follows the wallpaper; the shadows are the `shadow` role in every theme.",
      sample: () => el("div", "f-row", ...themes().map(ladderIn)),
    },
  ],
  scales: [
    {
      name: "Every length in device pixels",
      described: "The hairline stays one device pixel; the thin stroke rounds half up, 2, 3, 3, 4; every layer's offset and blur are multiples of 4, whole at every scale.",
      sample: deviceTable,
    },
    {
      name: "The lines at every scale",
      described: "Each box below is drawn at its own scale, whatever the toolbar's: the hairlines stay one device pixel, the strokes thicken in whole ones.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, hairlines(), thinStrokes())))),
    },
  ],
  content: doesntApply("a shadow or a line holds no words."),
  motion: doesntApply("a shadow moves only with what casts it: a notification eases from tucked to lifted over its group's spread, which that group's page plays."),
  checks: [
    { check: "kubaos-tokens", holds: "Refuses a shadow's or a ladder layer's offset or blur off the grid at any step, and an opacity outside 0 to 1; the hairline is device pixels by name and the thin stroke a whole number of logical pixels, rounded where it is drawn." },
    { check: "The geometry check", holds: "Holds every box, outline included, to whole device pixels at every scale." },
    { check: "kubaos-ui and KubaOS.Components", holds: "Draw a symbol's stroke with `icon::device_stroke` and the thin stroke rounded half up, so no line is fractional." },
  ],
});
