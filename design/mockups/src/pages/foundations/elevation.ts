// The Elevation foundation: the shadows, the container ladder a surface rises on instead, the
// hairline, the thin stroke and the symbol stroke, each in device pixels at every scale
// (`docs/DESIGN.md`, "Space and shape" and "Iconography").

import { icon } from "../../lib/icon.ts";
import { stack } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { length, names, number } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, MODES, modeName, painted, table, themes, token } from "../../site/parts/swatches.ts";
import { type Mode, SCALES, settings } from "../../site/settings.ts";

/** Every shadow, by name. */
function shadows(): string[] {
  return names("shadow-")
    .filter((name) => name.endsWith("-y"))
    .map((name) => name.slice("shadow-".length, -"-y".length));
}

/** What each shadow is for, and whether it is cast in its fill's own colour rather than `shadow`. */
const SHADOW_FOR: Record<string, { for: string; colour: string }> = {
  panel: { for: "A panel of the shell over the windows.", colour: "shadow" },
  dialog: { for: "A dialog over its scrim.", colour: "shadow" },
  tucked: { for: "A card in a stack, cast onto the edge of the card tucked under it, cut to that edge: close and tight, lighter in light mode.", colour: "shadow" },
  lifted: { for: "A card lifted over a list, lying flat beside the others it fans out with, straight under it: a notification group's preview.", colour: "shadow" },
  glow: { for: "The glow under a progress bar's fill, in the fill's own colour.", colour: "primary" },
};

/** A shape casting `shadow`: a card, or for the glow, a progress bar's fill on its track. */
function casting(shadow: string): HTMLElement {
  const colour = SHADOW_FOR[shadow]?.colour ?? "shadow";
  const shape = colour === "shadow" ? el("div", "f-raised") : el("div", "f-track", el("div", "f-fill"));
  const caster = colour === "shadow" ? shape : (shape.firstElementChild as HTMLElement);
  caster.style.setProperty("--y", `var(--shadow-${shadow}-y)`);
  caster.style.setProperty("--blur", `var(--shadow-${shadow}-blur)`);
  caster.style.setProperty("--alpha", `var(--shadow-${shadow}-opacity)`);
  caster.style.setProperty("--cast", `var(--color-${colour})`);
  return shape;
}

/** Every shadow cast on the surface of the theme picked in `mode`, under its name. */
function shadowsIn(mode: Mode): HTMLElement {
  const box = painted(
    settings().theme,
    mode,
    el("div", "f-shadows", ...shadows().map((shadow) => el("div", "f-swatch-cell", label(shadow, "label"), casting(shadow)))),
  );
  return el("div", "f-swatch-cell", label(modeName(mode), "label"), box);
}

/** Every shadow's measures by token, in the mode picked. */
function shadowTable(): HTMLElement {
  return table(
    ["Shadow", "Down", "Blur", "Opacity", "Of", "For"],
    shadows().map((shadow) => [
      token(`shadow.${shadow}`),
      label(String(length(`shadow-${shadow}-y`)), "mono", "on_surface"),
      label(String(length(`shadow-${shadow}-blur`)), "mono", "on_surface"),
      label(opacityIn(shadow), "mono", "on_surface"),
      token(SHADOW_FOR[shadow]?.colour ?? "shadow"),
      prose(SHADOW_FOR[shadow]?.for ?? "No words for this shadow: add them to `SHADOW_FOR`.", "body", SHADOW_FOR[shadow] ? "on_surface" : "error"),
    ]),
  );
}

/** A shadow's opacity in each mode, as the tokens' CSS gives it there. */
function opacityIn(shadow: string): string {
  const read = (mode: Mode) => {
    const probe = el("div", "");
    probe.dataset.mode = mode;
    document.body.append(probe);
    const value = getComputedStyle(probe).getPropertyValue(`--shadow-${shadow}-opacity`).trim();
    probe.remove();
    return value;
  };
  const [dark, light] = MODES.map(read);
  return dark === light ? dark! : `${dark} dark, ${light} light`;
}

/**
 * The surface containers side by side, lowest to highest: a surface rises by its role, not a
 * shadow. They sit beside each other rather than inside, so no corner has to nest.
 */
function ladder(): HTMLElement {
  const steps = ["surface_container_lowest", "surface_container_low", "surface_container", "surface_container_high", "surface_container_highest"];
  return painted(
    settings().theme,
    settings().mode,
    el(
      "div",
      "f-row",
      ...steps.map((step) => {
        const rung = el("div", "f-rung", text(step, "label", "on_surface"));
        rung.style.setProperty("--rung", `var(--color-${step})`);
        return rung;
      }),
    ),
  );
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

/** Every length of the family in device pixels at every scale: hairline, thin stroke, shadows. */
function deviceTable(): HTMLElement {
  const thin = (scale: number) => Math.max(1, Math.round(length("stroke-thin") * scale));
  const rows: (Element | string)[][] = [
    [token("stroke.hairline_device_px"), ...SCALES.map(() => label(String(length("stroke-hairline_device_px")), "mono", "on_surface"))],
    [token("stroke.thin"), ...SCALES.map((scale) => label(String(thin(scale)), "mono", "on_surface"))],
    ...shadows().flatMap((shadow) =>
      (["y", "blur"] as const).map((part) => [
        token(`shadow.${shadow}.${part}`),
        ...SCALES.map((scale) => {
          const value = length(`shadow-${shadow}-${part}`) * scale;
          return label(String(value), "mono", Number.isInteger(value) ? "on_surface" : "error");
        }),
      ]),
    ),
  ];
  return table(["Length", ...SCALES.map((scale) => `${scale}×`)], rows);
}

export default foundation({
  level: "foundations",
  slug: "elevation",
  title: "Elevation, hairlines and strokes",
  summary: "Five shadows and nothing else casts one; a surface rises by its container role; hairlines are one device pixel and strokes whole device pixels at every scale.",
  live: () => painted(settings().theme, settings().mode, el("div", "f-shadows", casting("panel"), casting("glow"))),
  description: [
    "Elevation is two shadows, `panel` and `dialog`, each down and blurred in the `shadow` role; `glow` under a progress bar's fill in the fill's own colour; and a notification group's two, `tucked` and `lifted`, as cards lying on cards. Nothing else casts a shadow: a surface sits higher by its container role, from `surface_container_lowest` up to `surface_container_highest`, not by a stronger shadow.",
    "A hairline is one device pixel at every scale (`stroke.hairline_device_px`): dividers, the outlines of cards, fields and wells, a switch's track. It is the one length not on the grid.",
    "The thin stroke, 2, is drawn as whole device pixels rounded half up: a spinner's arc, a field's caret, a partial checkbox's dash, the line where a dragged row will land. A symbol's stroke is 2 of its 24 units scaled to its size, rounded half up and never under one, so no line of an icon falls between device pixels.",
  ],
  tokens: [
    {
      name: "The shadows",
      described: "Each shadow's measures by token and its opacity in each mode, then each cast on the surface: the cards in `shadow`, the glow under a fill in the fill's colour.",
      sample: () => el("div", "page-part", shadowTable(), el("div", "f-row", shadowsIn(settings().mode))),
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
      name: "A surface rises by its role, not a shadow",
      described: "The containers side by side on the surface, lowest to highest: the step up is the colour, so nothing else needs a shadow to sit higher.",
      sample: ladder,
    },
  ],
  modes: [
    {
      name: "Every shadow, dark and light",
      described: "The `shadow` role is black in both modes; `tucked` takes its own lighter opacity in light, where the dark one's strength turns the edges into grey bands rather than paper.",
      sample: () => el("div", "f-row", ...MODES.map(shadowsIn)),
    },
  ],
  themes: [
    {
      name: "The glow in every theme",
      described: "The glow is the fill's own colour, so it follows the accent; the other shadows are the `shadow` role in every theme.",
      sample: () =>
        el(
          "div",
          "f-row",
          ...themes().map((theme) => el("div", "f-swatch-cell", label(theme.name, "label"), painted(theme.slug, settings().mode, casting("glow")))),
        ),
    },
  ],
  scales: [
    {
      name: "Every length in device pixels",
      described: "The hairline stays one device pixel; the thin stroke rounds half up, 2, 3, 3, 4; a shadow's offset and blur are multiples of 4, whole at every scale.",
      sample: deviceTable,
    },
    {
      name: "The lines at every scale",
      described: "Each box below is drawn at its own scale, whatever the toolbar's: the hairlines stay one device pixel, the strokes thicken in whole ones.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, hairlines(), thinStrokes())))),
    },
  ],
  content: doesntApply("a shadow or a line holds no words."),
  motion: doesntApply("a shadow moves only with what casts it: a notification group's cards ease from tucked to lifted over their spread's duration and curve, which that group's page plays."),
  checks: [
    { check: "kubaos-tokens", holds: "Refuses a shadow's offset or blur off the grid; the hairline is device pixels by name and the thin stroke a whole number of logical pixels, rounded where it is drawn." },
    { check: "The geometry check", holds: "Holds every box, outline included, to whole device pixels at every scale." },
    { check: "kubaos-ui and KubaOS.Components", holds: "Draw a symbol's stroke with `icon::device_stroke` and the thin stroke rounded half up, so no line is fractional." },
  ],
});
