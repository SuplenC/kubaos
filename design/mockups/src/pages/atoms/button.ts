// The Button page: `docs/design/components/button.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, for six kinds by night and by day.

import { button, filled, KINDS, type Kind, labelGlow, type Pinned, type Size, SIZES, TONES } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row } from "../../lib/layout.ts";
import { el, label } from "../../site/dom.ts";
import { doesntApply, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";
import { type ColourPath, paintBetween, paintGlass, palette, paletteName, themeName } from "../../site/recolour.ts";
import { themeChange, toDark, toLight } from "../../site/sequences.ts";
import { type Mode, settings } from "../../site/settings.ts";

/** The words and symbol each kind shows in the specimens, and how long they get. */
const SAMPLES: Record<Kind, { words: Words; icon: string; trailing: boolean }> = {
  Filled: { words: { short: "Continue", long: "Continue to the disks" }, icon: "arrow_forward", trailing: true },
  Tonal: { words: { short: "Save log", long: "Save the installation log" }, icon: "save", trailing: false },
  Text: { words: { short: "Back", long: "Back to the keyboard layout" }, icon: "arrow_back", trailing: false },
  Danger: { words: { short: "Erase disk", long: "Erase the disk and install" }, icon: "warning", trailing: false },
  Success: { words: { short: "Finish", long: "Finish and restart the computer" }, icon: "check", trailing: false },
  Warning: { words: { short: "Restart now", long: "Restart now and lose the open windows" }, icon: "warning", trailing: false },
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
  Filled:
    "The one main action on a screen, the next step: the most emphasis. One per screen, so the next step is never in doubt. At night its fill rests unlit (`primary_fill`, the accent at 60% of its lightness and 1.2 times its chroma) and its letters light: near white, tinted 18% with the tone (`on_primary_fill`). By day the fill is the accent lifted to lightness 0.80 with a dark label.",
  Tonal: "Medium emphasis, the accent's quieter sibling: `secondary_container` with its label, dimmed at night at rest (`on_secondary_container_rest`) and full lit.",
  Text: "Minor actions, Back, Skip, Cancel, Show log: no fill, a label in the tone, and less padding, since it has no edge to keep the label from. At night it rests unlit (`primary_rest`) and burns lit (`primary_lit`, the core).",
  Danger:
    "The action that destroys something, after a confirmation, naming it (Erase and install). It carries a meaning, not an emphasis, and burns: always lit, its own bright red (`burn_danger`) with a dark label (`on_burn_danger`), so it can't be taken for any wallpaper's accent.",
  Success: "The action that completes something good, naming it (Finish). It burns in its own bright green (`burn_success`) with a dark label (`on_burn_success`).",
  Warning: "The action with a cost the user should weigh, naming it (Restart now). It burns in its own bright amber (`burn_warning`) with a dark label (`on_burn_warning`).",
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

/** Every kind at md, lit: what a press plays on, and the lights a sequence flickers and strikes. */
function litKinds(): HTMLElement {
  return row("s16", "start", ...KINDS.map((kind) => sample(kind, "Md", "lit")));
}

/** The state matrix's rows: every kind at every size. */
const ROWS = SIZES.flatMap((size) => KINDS.map((kind) => ({ kind, size })));

/** A palette: a theme in a mode. */
interface Painted {
  readonly theme: string;
  readonly mode: Mode;
}

/**
 * A scene on its own surface, painted on the way from `from` to `to` along `path`, its lit kinds
 * as `from`'s mode shows them: what a theme change or a switch plays on. A palette
 * without a mode is in the mode picked when the scene is drawn.
 */
function scene(from: Painted | string, to: Painted | string, path: ColourPath): () => Element {
  return () => {
    const mode = settings().mode;
    const [a, b] = [from, to].map((each) => (typeof each === "string" ? { theme: each, mode } : each)) as [Painted, Painted];
    const painted = el("div", "page-scene", label("Every light and every colour, on one clock", "body_strong", "on_surface"), litKinds());
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
  return list.length === KINDS.length ? "every kind" : list.join(", ");
}

/**
 * A button lighting up or going back to rest, `--lit` from `from` to `to` over `short`: its fill,
 * label, line and glow all follow it, each from its rest colour to its lit one. A fade, never the
 * stutter.
 */
function lights(from: 0 | 1, to: 0 | 1): NonEmpty<Track<"standard">> {
  return [{ part: "lit", on: on(KINDS), target: ".k-button", property: "--lit", from, to, duration: "short", curve: "standard" }];
}

/** A lit button pressed, `--press` from 0 to 1 over `micro`. */
function press(): NonEmpty<Track<"standard">> {
  return [{ part: "pressed", on: on(KINDS), target: ".k-button", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" }];
}

export default page({
  level: "atoms",
  slug: "button",
  title: "Button",
  summary: "A button with a label: six kinds by what it is for, in three sizes, with an icon before or after the label or none. Its states follow the neon: it rests, lights up when hovered or focused, and is pressed.",
  live: () => sample("Filled", "Md"),
  description: [
    "A button names its action in sentence case, as the user would say it: Erase and install, never OK. Its label is its size's button style on a line as tall as the button's content box, so the label sits in the middle of the button on the grid, with the size's padding at its sides.",
    "Its corners take the control radius (`radius.control`); an icon-only button is an `IconButton`, a circle. An optional icon leads the label, or trails it on a button that goes forward (Continue →). The icon is the label's style's icon size, in a box the label's line tall, centred on a whole device pixel, so an icon never changes a button's height.",
    "Its states follow the neon (`docs/DESIGN.md`, \"States\"): rest, lit, pressed and disabled. Hovering it and focusing it from the keyboard are one state, lit: the same look whether the pointer or the keyboard got there. At night a button rests unlit and lights up: the line inside its edge burns (the tone's core on a fill, the tube elsewhere) with the glow outside it, and its label brightens. Pressed, it is overdriven: its fill goes a touch brighter and its glow a touch stronger. Every colour is a role computed from the live tone, and every label holds at least 4.5:1 on every fill it sits on, on any wallpaper. Nothing but the glow reaches past the button's box, so its shape never seems to grow. Lighting up, going back to rest and pressing are fades; the neon's stutter is for power changes only.",
    "The kinds make an emphasis ladder: Filled is the one main action, Tonal medium emphasis, Text minor actions. Danger, Success and Warning carry a meaning, not an emphasis, and burn: always lit, on their own fixed bright tones with dark labels, so none can be taken for a wallpaper's accent, even one of the same hue.",
    "Disabled, it has no action: grey, with no tone, no line and no glow. Its content dims to `opacity.disabled_content` of `on_surface`, its fill to `opacity.disabled_container`, and it takes no pointer and no focus.",
    "A held button (`Button::on_hold`) is pressed for a while instead of clicked, and a timed bar beside it shows how far along the hold is: Kōbō's Install, held with the mouse or Enter. It looks like any button of its kind.",
  ],
  variants: nonEmpty(KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => sample(kind, "Md") }))),
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size) }))),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "a button acts and holds no choice; a choice that stays on is a `Switch`, a `Chip` or a `Tile`.",
      loading: "a button doesn't wait: what it starts shows its own progress (a `ProgressBar`, a `Spinner`), and a held button's timed bar sits beside it.",
      empty: "a button always has its label; one with only an icon is an `IconButton`.",
      error: "a button has no value to be wrong; what its action failed at shows where its result would, as an `ErrorBlock` or a `Note`.",
    },
    rows: nonEmpty(
      ROWS.map(({ kind, size }) => ({
        name: `${kind} ${size.toLowerCase()}`,
        cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(kind, size, state),
      })),
    ),
    described: {
      rest: "Unlit at night. Filled: the deep unlit fill, `primary_fill`, the accent at 60% of its OKLCH lightness and 1.2 times its chroma, lowered in chroma alone until sRGB holds it, so its hue never shifts; its label `on_primary_fill`, near white tinted 18% with the tone. Tonal: `secondary_container`, its label dimmed, `on_secondary_container_rest`. Text: no fill, its label the tone unlit, `primary_rest`, the accent's `_unlit` lifted only as far as it holds 4.5:1 on every surface. Danger, Success and Warning burn even at rest: the bright fixed tone (`burn_danger`, `burn_success`, `burn_warning`) with a dark label, the tone at a quarter of its lightness, darker still where it must be to hold on every fill.",
      lit: "Hovered or focused from the keyboard, one state. The line inside the edge burns and the glow is outside it in the kind's tone: the near glow over `focus.near`, the far one over `focus.far`. Filled: the fill stays unlit; the core (`primary_fill_line`, the accent half way to white) is the line, `stroke.thin` wide, and the label brightens to `on_primary_fill_lit`, tinted 8%, with a tiny glow, the tone at `focus.label_glow` over `focus.label_near`. Tonal: the tube, `primary` at `focus.tube`, and the label full, `on_secondary_container`. Text: the tube, and the label burns as the core, `primary_lit`, with a faint near glow (`focus.content_glow` over `focus.content_near`). The burning kinds: the fill moves half way to the core (`burn_danger_lit` and the rest), with the core inside the edge; the label doesn't change.",
      pressed: "Overdriven, under the pointer or Space and Enter, so always lit as well. The glow is `focus.pressed_glow` times the lit one. Filled: the fill lifts 0.035 in lightness (`primary_fill_pressed`); its label and the label's glow don't change. Tonal: the container lifts 0.035 (`secondary_container_pressed`). Text: the tube runs white-hot, to the core. The burning kinds: the lit fill 8% further toward white (`burn_danger_pressed` and the rest).",
      disabled: "Grey, with no tone, no line and no glow: content at `opacity.disabled_content` of `on_surface`, fill at `opacity.disabled_container` of it (none on Text); no action, no focus.",
    },
  },
  focus: {
    tubes: nonEmpty(KINDS.map((kind) => ({ name: kind, tone: TONES[kind], filled: filled(kind), content: labelGlow(kind) === "content" }))),
    lit: "A button lights when it is hovered or focused, one state. At night its line burns inside its edge, the tone's core on a fill, the tube elsewhere, with the glow outside it, and only the glow reaches past the box. Each kind's line is its own colour role (`primary_fill_line`, `primary`, `burn_danger_line` and the rest), and the glow is its tone: the accent on Filled, Tonal and Text, the fixed tone on the burning kinds. A sequence that turns the lights out flickers a lit button down to its rest look and strikes it back: a control is always lit or at rest.",
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
        name: "Focus arrives",
        what: "Hovered or focused, the button lights up: a fade over `short` on `standard`, never the stutter, which is for power changes. Its fill, label, line and glow move together from their rest colours to their lit ones, as `--lit` goes from 0 to 1: on Filled the core and the glow fade in and the letters brighten, on Text the label burns up to the core, on the burning kinds the fill moves half way to the core.",
        tracks: lights(0, 1),
        reduced: "same",
        back: "cuts",
        sample: kinds,
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: the button fades back to rest over `short` on `standard`, every colour back to its rest one and the line and glow out.",
        tracks: lights(1, 0),
        reduced: "same",
        back: "cuts",
        sample: litKinds,
      },
      {
        name: "Press",
        what: "A lit button goes down under the pointer, or with Space or Enter, over `micro`, and back as it comes up: Filled's fill lifts 0.035 and the burning kinds' 8% further toward white, Tonal's container lifts, Text's tube runs white-hot, and every glow grows to `focus.pressed_glow` times the lit one.",
        tracks: press(),
        reduced: "same",
        back: "plays",
        sample: litKinds,
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
      { keys: ["Esc"], does: "Nothing of the button's: it goes back one step, as the surface says, and never presses a Danger, Success or Warning button." },
    ],
    order: "A button is one stop in the surface's reading order; a disabled one is skipped. Focus moving to it or away never moves anything: the line and the glow take no room, and focus lights it as hovering does.",
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
      { what: "The lit tube on Tonal and Text, rounded half up to device pixels.", token: "focus-tube" },
      { what: "The lit line inside a filled kind's edge (Filled, Danger, Success, Warning), rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
      { what: "A Text button's lit label's near glow's reach.", token: "focus-content_near" },
      { what: "A Filled button's lit label's tiny glow's reach.", token: "focus-label_near" },
    ],
    reflow: doesntApply("a button is as wide as its label and paddings at every window size: it never stretches or wraps, and a layout that wants it wider gives it room around it."),
  },
});
