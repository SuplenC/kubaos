// The Colour foundation (`docs/DESIGN.md`, "Colour", is its full reference): every colour role in
// both modes and every theme, the fixed palette, the derived roles as their recipes, the
// opacities, and every contrast pair with its ratio in the palette picked.

import { button, KINDS, type Kind, type Pinned } from "../../lib/button.ts";
import { row, stack } from "../../lib/layout.ts";
import { nested, role as corner } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { names, number, words } from "../../lib/tokens.ts";
import { decorative, type Pair, pairs, ratioIn } from "../../site/contrast.ts";
import { described } from "../../site/derive.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { bandedGrid } from "../../site/parts/bands.ts";
import { MODES, modeName, painted, swatch, swatchWith, table, themes, token } from "../../site/parts/swatches.ts";
import { type ColourPath, paintBetween, palette, paletteName, themeName } from "../../site/recolour.ts";
import { themeChange, toDark, toLight } from "../../site/sequences.ts";
import { type Mode, settings } from "../../site/settings.ts";

/** The roles matugen sets from the wallpaper, and the one each is read from. */
function sourced(): { role: string; from: string }[] {
  return names("source-")
    .map((name) => ({ role: name.slice("source-".length), from: words(name) }))
    .filter((each) => each.from !== "");
}

/** The roles of our own, which keep their value whatever the wallpaper. */
function fixed(): string[] {
  return names("source-")
    .filter((name) => words(name) === "")
    .map((name) => name.slice("source-".length));
}

/**
 * The fixed palette, a meaning a row: its burning tone, the text role fitted from it, its core and
 * its unlit form, what it is for and words it shows them with. The error family's labels and
 * container come from its tone too.
 */
const PALETTE: readonly {
  meaning: string;
  tone: string;
  text: string;
  core: string;
  unlit: string;
  tiles?: readonly (readonly [string, string])[];
  for: string;
  shows: string;
}[] = [
  {
    meaning: "Error",
    tone: "burn_error",
    text: "error",
    core: "error_core",
    unlit: "error_unlit",
    tiles: [["error", "on_error"], ["error_container", "on_error_container"]],
    for: "Red: something failed, and the action that destroys something. A Danger button burns on the tone; `error` is the words and marks of a failure; `error_container` a critical notification's surface, deep at night and pale by day.",
    shows: "Wrong password",
  },
  {
    meaning: "Success",
    tone: "burn_success",
    text: "success",
    core: "success_core",
    unlit: "success_unlit",
    for: "Green: done and installed. A Success button burns on the tone; `success` is a done mark and the words saying something is installed.",
    shows: "Installed",
  },
  {
    meaning: "Warning",
    tone: "burn_warning",
    text: "warning",
    core: "warning_core",
    unlit: "warning_unlit",
    for: "Amber, between fine and an error: a reading close to its limit. A Warning button, an action with a cost, burns on the tone.",
    shows: "Battery at 9%",
  },
  {
    meaning: "Sharing",
    tone: "burn_sharing",
    text: "sharing",
    core: "sharing_core",
    unlit: "sharing_unlit",
    for: "Violet: anything shared off the machine, the screen above all. Its tube marks what is shared, with `sharing_core` along its middle.",
    shows: "Sharing your screen",
  },
  {
    meaning: "Microphone",
    tone: "burn_microphone",
    text: "microphone",
    core: "microphone_core",
    unlit: "microphone_unlit",
    for: "Orange, clear of the warning's amber and the error's red: the privacy dot of a microphone that is listening, and its words.",
    shows: "Microphone on",
  },
  {
    meaning: "Recording",
    tone: "burn_recording",
    text: "recording",
    core: "recording_core",
    unlit: "recording_unlit",
    for: "Pure red at night, the REC light, a step deeper and purer than the error's red: the privacy dot of anything recording, the camera watching or the screen being recorded, and its words.",
    shows: "Recording",
  },
];

/** What each fixed role outside the palette is for, and words it shows them with. */
const FIXED_FOR: Record<string, { for: string; shows: string }> = {
  logo_core_warm: { for: "The logo's own core along the middle of the クバ mark's warm tubes: brand art, paler than a control's, the same in both modes. No control takes it.", shows: "クバ" },
  logo_core_cold: { for: "The logo's own core along the middle of the mark's cold tube, the dakuten. No control takes it.", shows: "クバ" },
};

/** The tone a fixed role's words are drawn in: a logo core is drawn along its tube, in that tube's tone. */
const LOGO_TONE: Record<string, string> = { logo_core_warm: "primary", logo_core_cold: "cold" };

/**
 * What each opacity is for, and the role it lays over which background where it is used, so its
 * sample is the opacity as it is seen.
 */
const OPACITY_FOR: Record<string, { for: string; over: string; under: string }> = {
  hover: { for: "The state layer on hover: the content colour over the fill.", over: "on_surface", under: "surface_container_high" },
  focus: { for: "The state layer of focus.", over: "on_surface", under: "surface_container_high" },
  press: { for: "The state layer on press.", over: "on_surface", under: "surface_container_high" },
  held: { for: "A fill lit while its control is held: a slider pressed or dragged, its fill brightened toward the palette's bright end, as a progress bar's glows.", over: "on_primary", under: "primary" },
  disabled_content: { for: "A disabled control's content, of `on_surface`.", over: "on_surface", under: "surface" },
  disabled_container: { for: "A disabled control's container, of `on_surface`.", over: "on_surface", under: "surface" },
  well: { for: "A well's surface over the panel.", over: "surface_container_low", under: "surface_container" },
  outline_soft: { for: "The soft hairline: a well's or a scroll panel's sunken edge, a divider.", over: "outline_variant", under: "surface" },
  placeholder: { for: "A shape that is only a suggestion of one: a workspace's ghost, a scroller at rest under the pointer.", over: "on_surface_variant", under: "surface" },
  selection: { for: "Selected text's highlight.", over: "primary", under: "surface" },
  locked: { for: "A read-only field's outline and surface.", over: "outline", under: "surface" },
  placeholder_text: { for: "A field's placeholder text: text, so it keeps 4.5:1 on every fill a field takes, the focused one too.", over: "on_surface_variant", under: "surface_container_high" },
  tint: { for: "An accent as a faint fill under itself: a tag, a done step.", over: "primary", under: "surface" },
  subdued: { for: "A picture that sits back while another is picked; a slider's step the fill has passed, on the fill.", over: "on_primary", under: "primary" },
  subdued_text: { for: "Secondary text on a selected row or a checked tile, over its container.", over: "on_secondary_container", under: "secondary_container" },
  scrim: { for: "What darkens the screen behind a dialog.", over: "scrim", under: "surface" },
  dialog_edge: { for: "A question dialog's hairline in its tint.", over: "primary", under: "surface_container_high" },
  danger_edge: { for: "A danger dialog's hairline in its tint.", over: "error", under: "surface_container_high" },
  track: { for: "A progress bar's track.", over: "surface_container_highest", under: "surface" },
  spinner_track: { for: "A spinner's faint full ring.", over: "primary", under: "surface" },
  error_fill: { for: "An error's surface, of the error container.", over: "error_container", under: "surface" },
  error_edge: { for: "An error's hairline, of the error.", over: "error", under: "surface" },
  log: { for: "The log's surface, over the page.", over: "surface_container_lowest", under: "surface" },
  frame: { for: "The shell's frame and its panels: glass, which Hyprland blurs behind.", over: "surface", under: "on_surface_variant" },
  recede: { for: "A list behind what floats over it, turned toward the surface so it sits back: the notifications under a group's fanned cards.", over: "surface_container", under: "on_surface_variant" },
  panel_scrim: { for: "What darkens the desktop behind an open panel, lighter than a dialog's scrim.", over: "scrim", under: "surface" },
  frame_blur: { for: "Where Hyprland starts blurring behind the frame's layer: only the glass.", over: "surface", under: "on_surface_variant" },
  glass: { for: "A panel's surface over the blurred wallpaper it floats on: Kōbō's panel.", over: "surface", under: "on_surface_variant" },
  backdrop: { for: "The furthest the wallpaper is turned toward the surface behind full-screen text, so every text role keeps 4.5:1 over any photo.", over: "surface", under: "on_surface" },
  window_border: { for: "An inactive window's border, the highest container over what is behind it.", over: "surface_container_highest", under: "surface" },
  window_shadow: { for: "An inactive window's neutral shadow, of the `shadow` role.", over: "shadow", under: "surface" },
};

/** The theme picked in the toolbar, and its palette in `mode`. */
function picked(mode: Mode) {
  const theme = settings().theme;
  return { theme, palette: palette(theme, mode) };
}

/** A scene of the roles at work: a card with its words, its actions, one focused, and the fixed roles' words. */
function scene(): HTMLElement {
  return el(
    "div",
    "f-card",
    stack(
      "s4",
      "start",
      text("Wi-Fi", "heading", "on_surface"),
      text("Home network · secured", "body", "on_surface_variant"),
    ),
    row(
      "s8",
      "start",
      button({ label: "Connect", kind: "Filled", size: "Md", pinned: "lit" }),
      button({ label: "Forget", kind: "Tonal", size: "Md" }),
      button({ label: "Details", kind: "Text", size: "Md" }),
    ),
    row(
      "s16",
      "start",
      text("Wrong password", "body_strong", "error"),
      text("Battery at 9%", "body_strong", "warning"),
      text("Installed", "body_strong", "success"),
      text("Sharing", "body_strong", "sharing"),
    ),
  );
}

/** The scene in `theme` and `mode`, under a name. */
function sceneIn(theme: string, mode: Mode): HTMLElement {
  return el("div", "f-swatch-cell", label(paletteName(theme, mode), "label"), painted(theme, mode, scene()));
}

/**
 * The matugen roles by family, each a tile: a role with the role drawn on it, or a role alone
 * where nothing is drawn on it. A role the families don't name lands in the last, so a new one
 * still shows.
 */
const FAMILIES: readonly { name: string; tiles: readonly (readonly [string, string | null])[] }[] = [
  { name: "Primary", tiles: [["primary", "on_primary"], ["primary_container", "on_primary_container"]] },
  { name: "Secondary", tiles: [["secondary", "on_secondary"], ["secondary_container", "on_secondary_container"]] },
  { name: "Tertiary", tiles: [["tertiary", "on_tertiary"], ["tertiary_container", "on_tertiary_container"]] },
  { name: "Cold", tiles: [["cold", "on_cold"], ["cold_container", "on_cold_container"]] },
  {
    name: "Surfaces",
    tiles: [
      ["surface_dim", "on_surface"],
      ["surface", "on_surface"],
      ["surface_bright", "on_surface"],
      ["surface_container_lowest", "on_surface"],
      ["surface_container_low", "on_surface"],
      ["surface_container", "on_surface"],
      ["surface_container_high", "on_surface"],
      ["surface_container_highest", "on_surface"],
      ["surface_variant", "on_surface_variant"],
    ],
  },
  { name: "Inverse", tiles: [["inverse_surface", "inverse_on_surface"], ["inverse_surface", "inverse_primary"]] },
  { name: "Lines and shade", tiles: [["outline", null], ["outline_variant", null], ["shadow", null], ["scrim", null]] },
];

/** The families, with every matugen role they leave out under one more. */
function families(): { name: string; tiles: (readonly [string, string | null])[] }[] {
  const named = new Set(FAMILIES.flatMap((family) => family.tiles.flatMap(([bg, fg]) => (fg ? [bg, fg] : [bg]))));
  const rest = sourced().filter((each) => !named.has(each.role));
  return [
    ...FAMILIES.map((family) => ({ name: family.name, tiles: [...family.tiles] })),
    ...(rest.length ? [{ name: "Not in a family yet", tiles: rest.map((each) => [each.role, null] as const) }] : []),
  ];
}

/** `bg` with `fg` drawn on it, each named with its value in `colours`; or `bg` alone. */
function tile(bg: string, fg: string | null, colours: Readonly<Record<string, string>>): HTMLElement {
  if (fg === null) {
    return el("div", "f-tile", swatch(bg), stack("s4", "start", text(bg, "label", "on_surface"), text(colours[bg] ?? "", "mono", "on_surface_variant")));
  }
  const box = el(
    "div",
    "f-tile",
    stack("s4", "start", text(bg, "label", fg), text(colours[bg] ?? "", "mono", fg), text(`${fg} ${colours[fg] ?? ""}`, "mono", fg)),
  );
  box.dataset.filled = "";
  box.style.setProperty("--tile", `var(--color-${bg})`);
  return box;
}

/** Every matugen role by family, in `theme` and `mode`, on that palette's surface. */
function rolesIn(theme: string, mode: Mode): HTMLElement {
  const colours = palette(theme, mode);
  return el(
    "div",
    "f-swatch-cell f-roles",
    label(paletteName(theme, mode), "label"),
    painted(
      theme,
      mode,
      el("div", "f-families", ...families().map((family) => stack("s8", "start", text(family.name, "label", "on_surface_variant"), ...family.tiles.map(([bg, fg]) => tile(bg, fg, colours))))),
    ),
  );
}

/** Every matugen role in the theme picked, dark beside light. */
function sourcedRoles(): HTMLElement {
  return el("div", "f-row", ...MODES.map((mode) => rolesIn(settings().theme, mode)));
}

/** Every matugen role in every theme, in the mode picked, side by side. */
function sourcedEverywhere(): HTMLElement {
  return el("div", "f-row", ...themes().map((theme) => rolesIn(theme.slug, settings().mode)));
}

/** Where each matugen role is read from. */
function sources(): HTMLElement {
  return table(
    ["Role", "Matugen's"],
    sourced().map(({ role, from }) => [token(role), token(from)]),
  );
}

/** `role` as a swatch over its name and its value in `colours`. */
function named(role: string, colours: Readonly<Record<string, string>>): HTMLElement {
  return swatchWith(role, `${role} ${colours[role] ?? ""}`);
}

/** The fixed palette, a meaning a row: its tone, text role, core and unlit form in each mode. */
function fixedPalette(): HTMLElement {
  return table(
    ["Meaning", ...MODES.map(modeName), "For"],
    PALETTE.map((each) => [
      label(each.meaning, "body_strong", "on_surface"),
      ...MODES.map((mode) => {
        const { theme, palette: colours } = picked(mode);
        const roles = el("div", "f-row", ...[each.tone, each.text, each.core, each.unlit].map((role) => named(role, colours)));
        const tiles = (each.tiles ?? []).map(([bg, fg]) => tile(bg, fg, colours));
        return painted(theme, mode, roles, text(each.shows, "body_strong", each.text), ...tiles);
      }),
      prose(each.for, "body", "on_surface"),
    ]),
  );
}

/** The fixed roles outside the palette, a row each: their value in each mode and how they are drawn. */
function fixedRoles(): HTMLElement {
  const inPalette = new Set(PALETTE.flatMap((each) => [each.tone, each.text, each.core, each.unlit, ...(each.tiles ?? []).flat()]));
  return table(
    ["Role", ...MODES.map(modeName), "For"],
    fixed().filter((role) => !inPalette.has(role)).map((role) => [
      token(role),
      ...MODES.map((mode) => {
        const { theme, palette: colours } = picked(mode);
        const shows = FIXED_FOR[role];
        const sample = role in LOGO_TONE ? tube(LOGO_TONE[role]!, role) : text(shows?.shows ?? role, "body_strong", role);
        const box = painted(theme, mode, swatchWith(role, colours[role] ?? ""), sample);
        box.dataset.tight = "";
        return box;
      }),
      prose(FIXED_FOR[role]?.for ?? "No words for this role: add them to `FIXED_FOR`.", "body", FIXED_FOR[role] ? "on_surface" : "error"),
    ]),
  );
}

/** A lit tube of `tone` with `core` along its middle: how the logo's mark draws one. */
function tube(tone: string, core: string): HTMLElement {
  const line = el("div", "f-tube");
  line.style.setProperty("--tone", `var(--color-${tone})`);
  line.style.setProperty("--core", `var(--color-${core})`);
  return line;
}

/** A control filled with `tone`, its line inside the fill's edge in `role`. */
function filled(tone: string, role: string): HTMLElement {
  const box = el("div", "f-filled");
  box.style.setProperty("--tone", `var(--color-${tone})`);
  box.style.setProperty("--line-colour", `var(--color-${role})`);
  return box;
}

/** `role` as text on `ground`, as a label fitted to it is drawn. */
function textOn(role: string, ground: string): HTMLElement {
  const box = el("div", "f-ground", text("Text", "body_strong", role));
  box.style.setProperty("--ground", `var(--color-${ground})`);
  return box;
}

/** Every derived role as its recipe, then drawn from its tone in every theme, in `mode`. */
function derivedIn(mode: Mode): HTMLElement {
  return bandedGrid(
    ["Recipe", ...themes().map((theme) => paletteName(theme.slug, mode))],
    described(mode).map((each) => ({
      name: each.role,
      cells: [
        el("div", "page-cell", prose(each.words, "small")),
        ...themes().map((theme) => {
          const shown = each.textOn[0] ? textOn(each.role, each.textOn[0]) : filled(each.tone, each.role);
          const box = painted(theme.slug, mode, shown, swatch(each.role));
          box.dataset.tight = "";
          return el("div", "page-cell", box);
        }),
      ],
    })),
  );
}

/** Every opacity at its value, as the role it lays is seen over its background. */
function opacities(): HTMLElement {
  const { theme } = picked(settings().mode);
  return table(
    ["Opacity", "Seen", "For"],
    names("opacity-").map((name) => {
      const short = name.slice("opacity-".length);
      const use = OPACITY_FOR[short];
      const layer = el("div", "f-layer", el("div", "", el("div", "")));
      layer.style.setProperty("--nested", nested(corner("control"), "s8").css);
      layer.style.setProperty("--under", `var(--color-${use?.under ?? "surface"})`);
      layer.style.setProperty("--over", `var(--color-${use?.over ?? "on_surface"})`);
      layer.style.setProperty("--alpha", `var(--${name})`);
      const box = painted(theme, settings().mode, layer);
      box.dataset.tight = "";
      return [
        token(`opacity.${short}`, String(number(name))),
        box,
        prose(
          use ? `${use.for} \`${use.over}\` over \`${use.under}\`.` : "No words for this opacity: add them to `OPACITY_FOR`.",
          "body",
          use ? "on_surface" : "error",
        ),
      ];
    }),
  );
}

/** A button state's column and how it is pinned: none for rest and disabled. */
const BUTTON_STATES: readonly { name: string; pinned?: Pinned; disabled?: boolean }[] = [
  { name: "Rest" },
  { name: "Lit · signs on", pinned: "lit" },
  { name: "Lit · signs off", pinned: "signs-off" },
  { name: "Pressed", pinned: "pressed" },
  { name: "Disabled", disabled: true },
];

/** Every kind of button in each state, in the mode picked: each state its own roles. */
function buttonStates(): HTMLElement {
  const sample = (kind: Kind, state: (typeof BUTTON_STATES)[number]) =>
    button({ label: kind, kind, size: "Md", disabled: state.disabled ?? false, ...(state.pinned ? { pinned: state.pinned } : {}) });
  return bandedGrid(
    BUTTON_STATES.map((state) => state.name),
    KINDS.map((kind) => ({ name: kind, cells: BUTTON_STATES.map((state) => el("div", "page-cell", sample(kind, state))) })),
  );
}

/** `pair`'s foreground on its background, as it is drawn: text, large text or a mark. */
function pairSample(pair: Pair, theme: string, mode: Mode): HTMLElement {
  const colour = pair.opacity ? `rgb(from var(--color-${pair.fg}) r g b / var(--opacity-${pair.opacity}))` : `var(--color-${pair.fg})`;
  const shown = pair.need === "graphic" ? swatch(pair.fg) : text(pair.need === "large_text" ? "Aa" : "Text", pair.need === "large_text" ? "headline" : "body", pair.fg);
  if (pair.need === "graphic") shown.style.setProperty("--swatch", colour);
  else shown.style.color = colour;
  const ground = el("div", "f-ground", shown);
  ground.style.setProperty("--ground", `var(--color-${pair.bg})`);
  const ratio = ratioIn(pair, palette(theme, mode));
  const words = ratio === null ? "no such role" : `${(Math.round(ratio * 100) / 100).toFixed(2)}:1`;
  const reading = label(words, "mono", ratio !== null && ratio >= pair.minimum ? "on_surface_variant" : "error");
  const box = painted(theme, mode, ground, reading);
  box.dataset.tight = "";
  return box;
}

const NEEDS: Record<Pair["need"], string> = { text: "text", large_text: "large text", graphic: "graphic" };

/** Every contrast pair, in the theme picked, dark and light, with its ratio and its minimum. */
function contrastPairs(): HTMLElement {
  return table(
    ["Foreground on background", "Needs", ...MODES.map(modeName), "Drawn"],
    pairs().map((pair) => [
      token(`${pair.fg}${pair.opacity ? ` at ${pair.opacity}` : ""} on ${pair.bg}`),
      label(`${NEEDS[pair.need]}, ${pair.minimum}:1`, "body", "on_surface"),
      ...MODES.map((mode) => pairSample(pair, settings().theme, mode)),
      prose(pair.use),
    ]),
  );
}

/** How many pairs fall under their minimum in each theme and mode. */
function failures(): HTMLElement {
  const all = pairs();
  return table(
    ["Theme", ...MODES.map(modeName)],
    themes().map((theme) => [
      label(theme.name, "body_strong", "on_surface"),
      ...MODES.map((mode) => {
        const colours = palette(theme.slug, mode);
        const under = all.filter((pair) => {
          const ratio = ratioIn(pair, colours);
          return ratio !== null && ratio < pair.minimum;
        });
        if (under.length === 0) return label("Every pair holds", "body", "on_surface");
        return el(
          "div",
          "f-swatch-cell",
          label(`${under.length} under their minimum`, "body_strong", "error"),
          ...under.map((pair) => token(`${pair.fg}${pair.opacity ? ` at ${pair.opacity}` : ""} on ${pair.bg}: ${(Math.round(ratioIn(pair, colours)! * 100) / 100).toFixed(2)}:1`)),
        );
      }),
    ]),
  );
}

/** A scene that a sequence plays on, painted on its way from one palette to another. */
function moving(from: { theme: string; mode: Mode } | string, to: { theme: string; mode: Mode } | string, path: ColourPath): () => Element {
  return () => {
    const mode = settings().mode;
    const [a, b] = [from, to].map((each) => (typeof each === "string" ? { theme: each, mode } : each)) as [{ theme: string; mode: Mode }, { theme: string; mode: Mode }];
    const every = new Set([...sourced().map((each) => each.role), ...fixed(), ...described(a.mode).map((each) => each.role)]);
    const roles = el("div", "f-row", ...[...every].map((role) => swatch(role)));
    const playing = el("div", "page-scene", label("Every colour role, on one clock", "body_strong", "on_surface"), scene(), roles);
    playing.dataset.mode = a.mode;
    paintBetween(playing, path, palette(a.theme, a.mode), palette(b.theme, b.mode));
    return el("div", "page-scene-box", playing);
  };
}

export default foundation({
  level: "foundations",
  slug: "colour",
  title: "Colour",
  lede: "Every colour is a role: matugen's from the wallpaper, a fixed palette for meanings, and recipes over both.",
  stage: { live: scene, hint: "the top bar's theme repaints it", daylight: true },
  tokens: [
    {
      name: "Roles from the wallpaper",
      note: "Each role matugen sets, by family, dark beside light, in the theme picked.",
      sample: sourcedRoles,
    },
    {
      name: "Where each comes from",
      note: "The matugen colour each role reads at runtime; `cold` is the tertiary.",
      sample: sources,
    },
    {
      name: "The fixed palette",
      note: "Each meaning's burning tone, text role, core and unlit form, the same in every theme.",
      sample: fixedPalette,
    },
    {
      name: "The logo's cores",
      note: "Fixed values outside the palette, one per mode, along the logo's tubes.",
      sample: fixedRoles,
    },
    {
      name: "Derived roles",
      note: "Every derived role as its recipe, drawn from what it is computed from, in each mode.",
      sample: () => el("div", "page-part", ...MODES.flatMap((mode) => [label(modeName(mode), "body_strong", "on_surface"), derivedIn(mode)])),
    },
    {
      name: "Opacities",
      note: "Each opacity at its value, laying its role over its background.",
      sample: opacities,
    },
    {
      name: "A button's states, by role",
      note: "No state layer: each state is its own derived roles from the live tone.",
      sample: buttonStates,
    },
  ],
  rules: [
    {
      name: "Every pair holds its minimum",
      note: "Every foreground on its background, its ratio live in the theme picked; red is under.",
      sample: contrastPairs,
    },
    {
      name: "The themes, pair by pair",
      note: "How many pairs fall under their minimum in each theme.",
      sample: failures,
    },
    {
      name: "Decoration needs no pair",
      note: "Roles drawn only as decoration carry no meaning, so no pair holds them.",
      sample: () => el("div", "f-row", ...decorative().map((role) => swatchWith(role, role))),
    },
  ],
  modes: [
    {
      name: "The same scene, dark and light",
      note: "By night the signs burn; by day nothing glows and fills lift to 0.80.",
      sample: () => el("div", "f-row", ...MODES.map((mode) => sceneIn(settings().theme, mode))),
    },
  ],
  themes: [
    {
      name: "The scene in every theme",
      note: "The same card in each theme; the fixed palette keeps its tones.",
      sample: () => el("div", "f-row", ...themes().map((theme) => sceneIn(theme.slug, settings().mode))),
    },
    {
      name: "Every role in every theme",
      note: "Each matugen role's value in each theme and mode.",
      sample: sourcedEverywhere,
    },
  ],
  scales: doesntApply("a colour is the same at every scale."),
  content: doesntApply("a colour holds no words; the Type page shows text's extremes."),
  motion: {
    arrives: doesntApply("a colour arrives with what it paints."),
    leaves: doesntApply("a colour leaves with what it paints."),
    between: [
      themeChange(`Theme change: ${themeName("neon")} to ${themeName("rose")}`, { from: themeName("neon"), to: themeName("rose") }, moving("neon", "rose", "through grey")),
      themeChange(`Theme change: ${themeName("lagoon")} to ${themeName("moss")}`, { from: themeName("lagoon"), to: themeName("moss") }, moving("lagoon", "moss", "through grey")),
      toLight({ from: paletteName("neon", "dark"), to: paletteName("neon", "light") }, moving({ theme: "neon", mode: "dark" }, { theme: "neon", mode: "light" }, "dawn")),
      toDark({ from: paletteName("neon", "light"), to: paletteName("neon", "dark") }, moving({ theme: "neon", mode: "light" }, { theme: "neon", mode: "dark" }, "dawn")),
    ],
  },
  checks: [
    { check: "kubaos-tokens", holds: "refuses a pair under its minimum, in either palette" },
    { check: "kubaos-design contrast", holds: "lists every pair; fails a drawn role in none" },
    { check: "kubaos-design lint", holds: "fails a colour written out in UI code" },
    { check: "Matugen's palettes", holds: "not checked yet: a wallpaper's own palette" },
  ],
});
