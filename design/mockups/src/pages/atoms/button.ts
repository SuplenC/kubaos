// The Button page: `docs/design/components/button.md`, as the specimens draw it today.

import { button, contentInTone, filledWithTone, KINDS, type Kind, type Pinned, type Size, SIZES, TONES } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row } from "../../lib/layout.ts";
import { el, label } from "../../site/dom.ts";
import { type Curve, type Duration, doesntApply, type Easing, type NonEmpty, nonEmpty, page, type Track, type Value } from "../../site/page.ts";
import { type ColourPath, paintBetween, paintGlass, palette, paletteName, themeName } from "../../site/recolour.ts";
import { themeChange, toDark, toLight } from "../../site/sequences.ts";
import { type Mode, settings } from "../../site/settings.ts";

/** The words and symbol each kind shows in the specimens, and how long they get. */
const SAMPLES: Record<Kind, { words: Words; icon: string; trailing: boolean }> = {
  Filled: { words: { short: "Continue", long: "Continue to the disks" }, icon: "arrow_forward", trailing: true },
  Tonal: { words: { short: "Save log", long: "Save the installation log" }, icon: "save", trailing: false },
  Text: { words: { short: "Back", long: "Back to the keyboard layout" }, icon: "arrow_back", trailing: false },
  Danger: { words: { short: "Erase disk", long: "Erase the disk and install" }, icon: "warning", trailing: false },
};

function sample(kind: Kind, size: Size, state: Pinned | "rest" | "disabled" = "rest"): HTMLButtonElement {
  const { words, icon, trailing } = SAMPLES[kind];
  return button({
    label: shown(words),
    kind,
    size,
    icon,
    trailing,
    disabled: state === "disabled",
    ...(state === "rest" || state === "disabled" ? {} : { pinned: state }),
  });
}

const FOR: Record<Kind, string> = {
  Filled: "The one thing to do next on a screen: `primary` filled, `on_primary` content. One per screen, so the next step is never in doubt.",
  Tonal: "An alternative to the filled one: `secondary_container` filled, `on_secondary_container` content.",
  Text: "Back, Skip, Cancel, Show log: no fill, `primary` content, and less padding, since it has no edge to keep the label from.",
  Danger: "The action that destroys something, after a confirmation: `error` filled, `on_error` content, naming the action (Erase and install).",
};

const SIZE_FOR: Record<Size, string> = {
  Sm: "`size.control_sm` tall, a `button_small` label and its icon, `space.s4` between them, `space.s12` at the sides (`space.s12` on Text): chrome around a screen, beside a field's label.",
  Md: "`size.control_md` tall, as tall as a field, a `button` label and its icon, `space.s8` between them, `space.s24` at the sides (`space.s12` on Text): a screen's own actions.",
  Lg: "`size.control_lg` tall, a `button_large` label and its icon, `space.s8` between them, `space.s32` at the sides (`space.s16` on Text): the one main action of a big page, Kōbō's Install.",
};

/** Every kind side by side at md, at rest: what each move plays on, so each kind's tone shows. */
function kinds(): HTMLElement {
  return row("s16", "start", ...KINDS.map((kind) => sample(kind, "Md")));
}

/** Every kind at md, focused: the lights a sequence dims and strikes. */
function focusedKinds(): HTMLElement {
  return row("s16", "start", ...KINDS.map((kind) => sample(kind, "Md", "focused")));
}

/** The state matrix's rows: every kind at every size. */
const ROWS = SIZES.flatMap((size) => KINDS.map((kind) => ({ kind, size })));

/** A palette: a theme in a mode. */
interface Painted {
  readonly theme: string;
  readonly mode: Mode;
}

/**
 * A scene on its own surface, painted on the way from `from` to `to` along `path`, its focused
 * kinds lit as `from`'s mode lights them: what a theme change or a switch plays on. A palette
 * without a mode is in the mode picked when the scene is drawn.
 */
function scene(from: Painted | string, to: Painted | string, path: ColourPath): () => Element {
  return () => {
    const mode = settings().mode;
    const [a, b] = [from, to].map((each) => (typeof each === "string" ? { theme: each, mode } : each)) as [Painted, Painted];
    const painted = el("div", "page-scene", label("Every light and every colour, on one clock", "body_strong", "on_surface"), focusedKinds());
    painted.dataset.mode = a.mode;
    paintBetween(painted, path, palette(a.theme, a.mode), palette(b.theme, b.mode));
    if (a.mode === "dark" && b.mode === "light") paintGlass(painted, palette(b.theme, b.mode), b.mode);
    return el("div", "page-scene-box", painted);
  };
}

/** The theme change between two themes, in the mode picked. */
function themeMove(from: string, to: string): ReturnType<typeof themeChange> {
  const between = { from: themeName(from), to: themeName(to) };
  return themeChange(`Theme change: ${between.from} to ${between.to}`, between, scene(from, to, "through grey"));
}

/** The kinds of a list, as the recipe says them. */
function on(list: readonly Kind[]): string {
  return list.length === KINDS.length ? "every kind" : list.join(" and ");
}

const FILLED = KINDS.filter(filledWithTone);
const EDGED = KINDS.filter((kind) => !filledWithTone(kind));
const LIT_CONTENT = KINDS.filter(contentInTone);

/**
 * The neon's parts from `from` to `to`: the glow on every kind, the tube inside the edge of a kind
 * not filled with its tone, the core inside the fill's edge of one that is, and the label of one
 * whose content is drawn in its tone, core-hot with its own glow.
 */
function neon<C extends Curve>(from: number, to: number, duration: Duration, curve: C): NonEmpty<Track<C>> {
  const part = (name: string, list: readonly Kind[], target: string): Track<C> => ({
    part: name,
    on: on(list),
    target,
    property: "opacity",
    from,
    to,
    duration,
    curve,
  });
  return [
    part("glow", KINDS, ".k-glow"),
    part("tube", EDGED, ".k-ring:not([data-filled]) > .k-tube"),
    part("core", FILLED, ".k-ring[data-filled] > .k-tube"),
    part("label", LIT_CONTENT, ".k-lit"),
  ];
}

/**
 * In daylight the neon is off: the unlit glass of the tube and of the core comes and goes over
 * `short`, never struck. The glow and the lit label keep their tracks, so the player is the same in
 * both modes, but daylight puts them out.
 */
function glass(from: number, to: number): NonEmpty<Track<Easing>> {
  const unseen: Record<string, string> = {
    glow: "every kind, unseen: daylight puts the glow out",
    label: `${on(LIT_CONTENT)}, unseen: daylight leaves the label its own tone`,
  };
  const [first, ...rest] = neon(from, to, "short", "standard").map((track) => ({ ...track, on: unseen[track.part] ?? track.on }));
  return [first!, ...rest];
}

/** The state layer from `from` to `to` over `micro`. */
function layer(from: Value, to: Value): NonEmpty<Track<"standard">> {
  return [{ part: "state layer", on: on(KINDS), target: ".k-layer", property: "opacity", from, to, duration: "micro", curve: "standard" }];
}

export default page({
  level: "atoms",
  slug: "button",
  title: "Button",
  summary: "A button with a label: four kinds by what it is for, in three sizes, with an icon before or after the label or none.",
  live: () => sample("Filled", "Md"),
  description: [
    "A button names its action in sentence case, as the user would say it: Erase and install, never OK. Its label is its size's button style on a line as tall as the button's content box, so the label sits in the middle of the button on the grid, with the size's padding at its sides.",
    "Its corners take the control radius (`radius.control`); an icon-only button is an `IconButton`, a circle. An optional icon leads the label, or trails it on a button that goes forward (Continue →). The icon is the label's style's icon size, in a box the label's line tall, centred on a whole device pixel, so an icon never changes a button's height.",
    "Hovering it lays its content colour over the fill at `opacity.hover`, pressing it at `opacity.press`, each fading over `micro`. Focused from the keyboard, its edge lights as the neon tube with its glow outside it, in the kind's tone: red (`error`) on Danger, the accent on the others. A Filled or a Danger button is filled with its tone's colour, so its fill is the tube and the tone's hot core (`primary_core`, `error_core`) is the line inside the fill's edge, the thin stroke wide, at the button's corner. A Text button's label and icon are drawn in its tone with nothing under them, so they light as a tube too: `primary_core` letters and icon with the tone's near and far glow round them, scaled by the label's stroke over the tube's width, burning, striking and going out with the line. Nothing but the glow reaches past the button's box, so its shape never seems to grow. That is its lit look; its unlit look is the same line as unlit glass (the tone's `_unlit`, 35% toward black; on Tonal and Text at `focus.glass` over the button's own container), with no glow, and a Text button's label its own `primary`. Light mode is always unlit, and dark mode is unlit while a sequence has the lights out: the scene picks the look, never the user. Disabled, it has no action: its content dims to `opacity.disabled_content` of `on_surface`, its fill to `opacity.disabled_container`, and it takes no pointer and no focus.",
    "A held button (`Button::on_hold`) is pressed for a while instead of clicked, and a timed bar beside it shows how far along the hold is: Kōbō's Install, held with the mouse or Enter. It looks like any button of its kind.",
  ],
  variants: nonEmpty(KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => sample(kind, "Md") }))),
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size) }))),
  states: {
    shown: ["rest", "hovered", "pressed", "focused", "disabled"],
    absent: {
      selected: "a button acts and holds no choice; a choice that stays on is a `Switch`, a `Chip` or a `Tile`.",
      loading: "a button doesn't wait: what it starts shows its own progress (a `ProgressBar`, a `Spinner`), and a held button's timed bar sits beside it.",
      empty: "a button always has its label; one with only an icon is an `IconButton`.",
      error: "a button has no value to be wrong; what its action failed at shows where its result would, as an `ErrorBlock` or a `Note`.",
    },
    rows: nonEmpty(
      ROWS.map(({ kind, size }) => ({
        name: `${kind} ${size.toLowerCase()}`,
        cell: (state: "rest" | "hovered" | "pressed" | "focused" | "disabled") => sample(kind, size, state),
      })),
    ),
    described: {
      rest: "Its fill and content, nothing over them.",
      hovered: "The content colour over the fill at `opacity.hover`, faded in over `micro`.",
      pressed: "The content colour at `opacity.press`, over `micro`.",
      focused: "Focused from the keyboard, in the look the scene gives it, never one the user picks: lit in dark mode at rest, unlit in light mode and in dark mode while a sequence has the lights out (the next column). Lit: the neon tube inside its edge in the kind's tone, `focus.tube` wide rounded half up to device pixels, and the glow outside it in the same tone: the near glow over `focus.near`, the far one over `focus.far`. On Filled and Danger, the fill is the tube: the tone's core (`primary_core`, `error_core`, half way to white) is the line inside the fill's edge, `stroke.thin` wide, and the glow is outside. Danger's is red. On Text, the label and icon are a tube too: `primary_core`, with the near and far glow round them, scaled by the label's stroke over `focus.tube`. In light mode this column is the unlit look. Hover and press still lay their layer under it.",
      disabled: "Content at `opacity.disabled_content` of `on_surface`, fill at `opacity.disabled_container` of it (none on Text); no layer, no focus, no action.",
    },
  },
  focus: {
    tubes: nonEmpty(KINDS.map((kind) => ({ name: kind, tone: TONES[kind], filled: filledWithTone(kind), content: contentInTone(kind) }))),
    looks: {
      lit: "The neon burning, in dark mode at rest: the tube inside the edge in the kind's tone, or on Filled and Danger the tone's core half way to white inside the fill's edge, with the glow outside. On Text the label and icon burn too: `primary_core`, with the glow round them.",
      unlit:
        "The same line as unlit glass, with no glow: on Filled and Danger the tone's `_unlit` core (35% toward black) inside the fill's edge, `stroke.thin` wide; on Tonal and Text, `primary_unlit` at `focus.glass` over the button's own container, `focus.tube` wide; a Text button's label is its own `primary`, with no glow. Light mode is always this look, so there this column is the same as Focused; dark mode shows it while a sequence has the lights out (the theme change, the switch to light or dark), in the dark palette's `_unlit`. Nothing the user does picks it: the scene does.",
    },
    unlit: (index: number) => sample(ROWS[index]!.kind, ROWS[index]!.size, "unlit"),
  },
  content: {
    text: KINDS.map((kind) => ({
      name: kind,
      words: SAMPLES[kind].words,
      sample: (words: string) => button({ label: words, kind, size: "Md", icon: SAMPLES[kind].icon, trailing: SAMPLES[kind].trailing }),
    })),
    missingPicture: [
      {
        name: "No icon",
        for: "The icon is optional: without one the label is alone between the paddings, and the button keeps its height. Every symbol is one of KubaOS Symbols, so a named symbol is never missing.",
        sample: () => button({ label: "Authenticate", kind: "Filled", size: "Md" }),
      },
    ],
    counts: doesntApply("a button holds one label; buttons side by side are a row of the layout, which the template's page shows."),
  },
  motion: {
    arrives: doesntApply("a button comes with the surface or the page it is on, and moves as that does."),
    leaves: doesntApply("a button goes with the surface or the page it is on."),
    between: [
      {
        name: "Hover",
        what: "The state layer, the content colour over the fill, fades in as the pointer comes onto the button, and out as it leaves.",
        tracks: layer(0, { opacity: "hover" }),
        reduced: "same",
        back: "plays",
        sample: kinds,
      },
      {
        name: "Press",
        what: "The layer deepens from hover's to `opacity.press` as the button goes down under the pointer, and back as it comes up.",
        tracks: layer({ opacity: "hover" }, { opacity: "press" }),
        reduced: "same",
        back: "plays",
        sample: kinds,
      },
      {
        name: "Focus arrives",
        what: "The tube strikes on in the kind's tone as the boot sign's tubes do, `neon.strike`'s steps over `medium`, and the glow with it: red on Danger, the accent on the others. On Filled and Danger the fill is the tube, so the core strikes inside the fill's edge; on Text the label strikes with the tube, core-hot with its glow. In light mode the neon is off: the unlit glass fades in over `short`, never struck, and the glow and the lit label stay out.",
        tracks: neon(0, 1, "medium", "neon.strike"),
        reduced: neon(0, 1, "short", "standard"),
        unstuttered: neon(0, 1, "short", "standard"),
        daylight: glass(0, 1),
        back: "cuts",
        sample: kinds,
      },
      {
        name: "Focus leaves",
        what: "The tube, the core, the glow and Text's lit label fade out together, the label back to its own `primary`; in light mode the unlit glass alone.",
        tracks: neon(1, 0, "short", "standard"),
        reduced: "same",
        daylight: glass(1, 0),
        back: "cuts",
        sample: kinds,
      },
      themeMove("neon", "rose"),
      themeMove("rose", "lagoon"),
      toLight(
        { from: paletteName("neon", "dark"), to: paletteName("neon", "light") },
        scene({ theme: "neon", mode: "dark" }, { theme: "neon", mode: "light" }, "dawn"),
      ),
      toDark(
        { from: paletteName("neon", "light"), to: paletteName("neon", "dark") },
        scene({ theme: "neon", mode: "light" }, { theme: "neon", mode: "dark" }, "dawn"),
      ),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to the button, or on to the next or previous control, in reading order." },
      { keys: ["Enter"], does: "Presses the focused button, once per press, never on key repeat. With no control focused that takes Enter, it is the screen's Filled action. On a held button, Enter held is the hold." },
      { keys: ["Space"], does: "Presses the focused button." },
      { keys: ["Esc"], does: "Nothing of the button's: it goes back one step, as the surface says, and never presses a Danger button." },
    ],
    order: "A button is one stop in the surface's reading order; a disabled one is skipped. Focus moving to it or away never moves anything: the tube and the glow take no room.",
    lands: "A button takes focus when the surface gives it: a screen with no field starts on its Filled button, a dialog on its last action, a Danger dialog on Cancel, so Enter twice never destroys anything.",
  },
  measures: {
    samples: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size) }))),
    measures: [
      { what: "sm's height.", token: "size-control_sm" },
      { what: "md's height, a field's too.", token: "size-control_md" },
      { what: "lg's height.", token: "size-control_lg" },
      { what: "sm's label line, which is its content box; the rest of the height is split above and below.", token: "type-button_small-line" },
      { what: "md's label line.", token: "type-button-line" },
      { what: "lg's label line.", token: "type-button_large-line" },
      { what: "sm's icon.", token: "type-button_small-icon" },
      { what: "md's icon.", token: "type-button-icon" },
      { what: "lg's icon.", token: "type-button_large-icon" },
      { what: "sm's gap between icon and label.", token: "space-s4" },
      { what: "md's and lg's gap.", token: "space-s8" },
      { what: "sm's padding at the sides, Text's at sm and md.", token: "space-s12" },
      { what: "lg's Text padding.", token: "space-s16" },
      { what: "md's padding.", token: "space-s24" },
      { what: "lg's padding.", token: "space-s32" },
      { what: "The corners.", token: "radius-control" },
      { what: "The focus tube, rounded half up to device pixels.", token: "focus-tube" },
      { what: "The core inside a Filled or Danger button's fill's edge, rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
    ],
    reflow: doesntApply("a button is as wide as its label and paddings at every window size: it never stretches or wraps, and a layout that wants it wider gives it room around it."),
  },
});
