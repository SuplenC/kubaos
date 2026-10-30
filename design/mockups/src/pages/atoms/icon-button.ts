// The IconButton page: `docs/design/components/icon-button.md`, in the state model (`docs/DESIGN.md`,
// "States"): rest, lit, pressed and disabled, for six kinds at three sizes, by night and by day.

import { filled, labelGlow, METRICS, type Size, SIZES, TONES } from "../../lib/button.ts";
import { fixed, shown, type Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { hintRoom } from "../../lib/hint.ts";
import { ICON_KINDS, type IconKind, iconButton } from "../../lib/icon-button.ts";
import { row } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The symbol and the label each kind shows here, the label as long as it gets. */
const SAMPLES: Record<IconKind, { icon: string; words: Words }> = {
  Filled: { icon: "add", words: { short: "New", long: "New snapshot of the home folder" } },
  Tonal: { icon: "settings", words: { short: "Settings", long: "Settings for notifications and sounds" } },
  Text: { icon: "close", words: { short: "Close", long: "Close the panel and go back" } },
  Danger: { icon: "power_settings_new", words: { short: "Shut down", long: "Shut down the computer now" } },
  Success: { icon: "check", words: { short: "Finish", long: "Finish and restart the computer" } },
  Warning: { icon: "restart_alt", words: { short: "Restart now", long: "Restart now and lose the open windows" } },
};

/** An icon button of `kind` and `size` in `state`; its hint drawn unless `hint` is false. */
function sample(kind: IconKind, size: Size, state: Shown = "rest", hint = true): HTMLButtonElement {
  const { icon, words } = SAMPLES[kind];
  return iconButton({ icon, label: shown(words), kind, size, hint, ...stateOptions(state) });
}

/** `control`, an icon button of `size` labelled `words`, with the room its hint floats in. */
function roomy(control: HTMLButtonElement, size: Size): HTMLElement {
  return hintRoom(control, control.getAttribute("aria-label") ?? "", `var(--size-${METRICS[size].height})`);
}

const FOR: Record<IconKind, string> = {
  Filled: "The main action of a toolbar or a card when a symbol says it plainly: Add, New. As a Filled button: at night the unlit fill, `primary_fill`, with its symbol near white (`on_primary_fill`); by day the accent lifted to lightness 0.80 with a dark symbol.",
  Tonal: "A quieter action beside a main one: Settings in a panel's header. On `secondary_container`, its symbol dimmed at night at rest (`on_secondary_container_rest`) and full lit.",
  Text: "Chrome and minor actions: Close, Back, a tile's detail arrow. No fill, the symbol in the tone: unlit at rest by night (`primary_rest`), burning as the core lit (`primary_lit`), with a faint glow round its strokes.",
  Danger: "An action that destroys something or takes the machine down, behind a confirmation: Shut down. It burns, always lit, on its own bright red (`burn_error`) with a dark symbol (`on_burn_error`).",
  Success: "The action that completes something good, where a symbol says it plainly: Finish. It burns, always lit, on its own bright green (`burn_success`) with a dark symbol (`on_burn_success`).",
  Warning: "The action with a cost the user should weigh, where a symbol says it plainly: Restart now. It burns, always lit, on its own bright amber (`burn_warning`) with a dark symbol (`on_burn_warning`).",
};

const SIZE_FOR: Record<Size, string> = {
  Sm: "`size.control_sm` across, a sm button's height, with `button_small`'s icon: chrome in a dense row, beside a field.",
  Md: "`size.control_md` across, a field's height, with `button`'s icon: a panel's or a card's actions.",
  Lg: "`size.control_lg` across, with `button_large`'s icon: beside a big page's lg button.",
};

/** Every kind at md, at rest: what each move plays on. */
function kinds(): HTMLElement {
  return row("s16", "start", ...ICON_KINDS.map((kind) => sample(kind, "Md", "rest", false)));
}

/** Every kind at md, lit: what a press plays on, and the lights a sequence flickers and strikes. */
function litKinds(): HTMLElement {
  return row("s16", "start", ...ICON_KINDS.map((kind) => sample(kind, "Md", "lit", false)));
}

const ON = "every kind";

/** The state matrix's rows: every kind at every size. */
const ROWS = SIZES.flatMap((size) => ICON_KINDS.map((kind) => ({ kind, size })));

export default page({
  level: "atoms",
  slug: "icon-button",
  title: "IconButton",
  summary: "A circle with a symbol, as tall as the label button of its size, in every one of the button's kinds. Its label is required: it is its name out loud and shows in its hint. Its states are the button's.",
  live: () => roomy(sample("Filled", "Md"), "Md"),
  description: [
    "An icon button is a circle (`radius.full`) as wide and as tall as the label button of its size, its symbol the icon of that button's label style, centred on a whole device pixel. It is for an action a symbol says plainly, where a label would crowd the row: Close, Settings, Add.",
    "Its label is required, so an icon button can't be built without one: an icon alone says nothing out loud. The label is its accessible name, and shows in its `Hint` while the pointer is over it or the keyboard is on it, a bubble under it in the inverse colours (the Hint page). One a `Tooltip` explains (`explained`) shows the tooltip's card instead, which already says its name.",
    "Its kinds, colours and states are the button's (the Button page): Filled, Tonal, Text, Danger, Success and Warning, each resting, lit (hovered or focused, one state, with its two looks, signs on and signs off), pressed and disabled. At night it rests unlit and lights up: the line inside its edge burns (the tone's core, `stroke.thin` wide, on Filled's fill and the burning kinds'; the tube, `focus.tube` wide, on Tonal and Text) with the glow outside it, the only thing that reaches past the circle. Its symbol brightens as a label does: Filled's to `on_primary_fill_lit` with a tiny glow, Text's burns as the core with a faint glow round its strokes, Tonal's goes full. Lighting up, going back to rest and pressing are fades; the flicker is for power changes only.",
    "Disabled, it has no action: grey, with no tone, no line and no glow, and it takes no focus. Its hint still shows while the pointer is over it, so a disabled icon still says what it is.",
  ],
  variants: nonEmpty(ICON_KINDS.map((kind) => ({ name: kind, for: FOR[kind], sample: () => roomy(sample(kind, "Md"), "Md") }))),
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => roomy(sample("Filled", size), size) }))),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "an icon button acts and holds no choice; a choice that stays on is a `Switch`, a `Chip` or a `Tile`.",
      loading: "it doesn't wait: what it starts shows its own progress.",
      empty: "it always has its symbol and its label; with words to show, it is a `Button`.",
      error: "it has no value to be wrong; what its action failed at shows where its result would.",
    },
    rows: nonEmpty(
      ROWS.map(({ kind, size }) => ({
        name: `${kind} ${size.toLowerCase()}`,
        cell: (state: "rest" | "lit" | "pressed" | "disabled") => sample(kind, size, state, false),
      })),
    ),
    described: {
      rest: "Unlit at night, as the button of its kind rests: Filled on `primary_fill` with an `on_primary_fill` symbol; Tonal on `secondary_container` with its symbol dimmed (`on_secondary_container_rest`); Text with no fill and its symbol the tone unlit (`primary_rest`); Danger, Success and Warning burning, `burn_error`, `burn_success` and `burn_warning` with an `on_burn_error`, `on_burn_success` and `on_burn_warning` symbol. By day: Filled the accent lifted to lightness 0.80 with a dark symbol, Tonal its container, Text the tone, the burning kinds their deeper daylight tones. The hints aren't drawn here: each floats over the row below; the samples above and the Hint page show them.",
      lit: "Hovered or focused, here with the signs on, and its hint shows. The line inside the circle's edge burns and the glow is outside it in the kind's tone: Filled's and the burning kinds' line is the core, `stroke.thin` wide (`primary_fill_line`, `burn_error_line`, `burn_success_line`, `burn_warning_line`), Tonal's and Text's the tube, `primary` at `focus.tube`. Filled's symbol brightens to `on_primary_fill_lit` with a tiny glow, Tonal's goes full, Text's burns as the core (`primary_lit`) with a faint glow round its strokes; the burning kinds' fill moves half way to the core (`burn_error_lit`, `burn_success_lit`, `burn_warning_lit`). By day lit is its signs-off look in daylight colours: no glow, a dark line inside the edge of Filled and the burning kinds and the tone's line inside Tonal's and Text's.",
      pressed: "Overdriven, under the pointer or Space and Enter, so lit as well: the glow `focus.pressed_glow` times the lit one; Filled's fill lifts 0.035 (`primary_fill_pressed`), Tonal's container lifts 0.035, Text's tube runs white-hot to the core, the burning kinds' lit fill goes 8% further toward white (`burn_error_pressed` and the rest). By day: Filled and Tonal 0.04 darker, Text a tint of the tone under it at `opacity.press_tint`, the burning kinds 0.06 darker than rest.",
      disabled: "Grey, with no tone, no line and no glow: its symbol at `opacity.disabled_content` of `on_surface`, its circle at `opacity.disabled_container` of it (none on Text). No action and no focus; the pointer over it still shows its hint.",
    },
  },
  focus: {
    tubes: nonEmpty(ICON_KINDS.map((kind) => ({ name: kind, tone: TONES[kind], filled: filled(kind), content: labelGlow(kind) === "content" }))),
    looks: {
      lit: "At night at rest, the signs burn: the line inside the circle's edge (the core on Filled and the burning kinds, the tube on Tonal and Text) with the glow outside it, all that reaches past the circle. The symbol lights as the label of its kind's button does.",
      signsOff: "The same icon button still hovered or focused with the neon unpowered: by day always, and at night while a theme change or dark ⇄ light has the lights out. The line stays inside the edge as unlit glass (`primary_fill_glass`, `primary_glass`, `burn_error_glass`, `burn_success_glass`, `burn_warning_glass`), with no glow, and the symbol at its rest colour. Its hint shows either way: the bubble isn't neon.",
    },
    signsOff: (index: number) => {
      const { kind, size } = ROWS[index]!;
      return sample(kind, size, "signs-off", false);
    },
  },
  content: {
    text: ICON_KINDS.map((kind) => ({
      name: `${kind}: its label, in its hint`,
      words: SAMPLES[kind].words,
      sample: (words: string) => roomy(iconButton({ icon: SAMPLES[kind].icon, label: words, kind, size: "Md", pinned: "lit" }), "Md"),
    })),
    missingPicture: [
      {
        name: "No picture to miss",
        for: "Its symbol is one of KubaOS Symbols, named in code, so it is never missing; a button with no symbol has words, and is a `Button`. The label can't be missing either: the types require it.",
        sample: () => roomy(iconButton({ icon: "more_time", label: shown(fixed("Add five minutes")), kind: "Tonal", size: "Md", pinned: "lit" }), "Md"),
      },
    ],
    counts: doesntApply("an icon button holds one symbol; icon buttons side by side are a row of the layout."),
  },
  motion: {
    arrives: doesntApply("an icon button comes with the surface or the page it is on, and moves as that does."),
    leaves: doesntApply("an icon button goes with the surface or the page it is on."),
    between: [
      {
        name: "Focus arrives",
        what: "Hovered or focused, it lights up: a fade over `short` on `standard`, never the flicker. Its fill, symbol, line and glow move together from their rest colours to their lit ones as `--lit` goes from 0 to 1, and its hint fades in over the same `short` (the Hint page).",
        tracks: lights(".k-button", ON, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: kinds,
      },
      {
        name: "Focus leaves",
        what: "The pointer leaves or focus moves on: it fades back to rest over `short` on `standard`, and its hint fades out with it.",
        tracks: lights(".k-button", ON, 1, 0),
        reduced: "same",
        back: "cuts",
        sample: litKinds,
      },
      {
        name: "Press",
        what: "A lit icon button goes down under the pointer, or with Space or Enter, over `micro`, and back as it comes up: its fill lifts (Filled, Tonal and the burning kinds) or its tube runs white-hot (Text), and its glow grows to `focus.pressed_glow` times the lit one.",
        tracks: press(".k-button", ON),
        reduced: "same",
        back: "plays",
        sample: litKinds,
      },
      ...powerMoves(litKinds),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Focus to the icon button, or on to the next or previous control, in reading order; its hint shows while it has focus." },
      { keys: ["Enter", "Space"], does: "Presses the focused icon button, once per press, never on key repeat." },
      { keys: ["Esc"], does: "Nothing of the icon button's: it goes back one step, as the surface says, and never presses a Danger, Success or Warning one." },
    ],
    order: "One stop in the surface's reading order; a disabled one is skipped. Focus moving to it or away never moves anything: the line, the glow and the hint take no room.",
    lands: "It takes focus when the surface gives it; a surface never starts on a Danger, Success or Warning one.",
  },
  measures: {
    samples: nonEmpty(SIZES.map((size) => ({ name: size.toLowerCase(), for: SIZE_FOR[size], sample: () => sample("Filled", size, "rest", false) }))),
    measures: [
      { what: "sm's side, a sm button's height.", token: "size-control_sm" },
      { what: "md's side.", token: "size-control_md" },
      { what: "lg's side.", token: "size-control_lg" },
      { what: "sm's symbol, `button_small`'s icon.", token: "type-button_small-icon" },
      { what: "md's symbol, `button`'s icon.", token: "type-button-icon" },
      { what: "lg's symbol, `button_large`'s icon.", token: "type-button_large-icon" },
      { what: "The circle.", token: "radius-full" },
      { what: "The lit tube on Tonal and Text, rounded half up to device pixels.", token: "focus-tube" },
      { what: "The lit line inside the fill of Filled and the burning kinds, rounded half up.", token: "stroke-thin" },
      { what: "The near glow's reach.", token: "focus-near" },
      { what: "The far glow's reach.", token: "focus-far" },
      { what: "A Text icon button's lit symbol's glow's reach.", token: "focus-content_near" },
      { what: "A Filled icon button's lit symbol's tiny glow's reach.", token: "focus-label_near" },
      { what: "The hint's gap below it.", token: "space-s8" },
    ],
    reflow: doesntApply("an icon button is its size's side at every window size: it never stretches, and a layout that wants more gives it room around it."),
  },
});
