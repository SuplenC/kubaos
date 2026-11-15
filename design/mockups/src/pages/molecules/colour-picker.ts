// The ColourPicker page: `docs/design/components/colour-picker.md`. A colour picked for something
// drawn over a picture: Auto, a role of the scheme as a swatch, or Custom by its hue and tone. Its
// three controls are shown in a setting's well, as the wallpaper picker's clock editor lays them.

import { type ColourPickerOptions, colourPicker, type PickedColour, type Swatch } from "../../lib/colour-picker.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

type Role = "primary" | "secondary" | "cold" | "on_surface" | "tertiary" | "error" | "success" | "warning";

/** A role's swatch, in the colour the pane it is drawn in gives it. */
const swatchOf = (role: Role, name: string): Swatch<Role> => ({ role, name, colour: `var(--color-${role})` });

/** The scheme's roles the clock offers: `primary` and `cold` follow the wallpaper. */
const ROLES: readonly Swatch<Role>[] = [swatchOf("primary", "Primary"), swatchOf("secondary", "Secondary"), swatchOf("cold", "Cold"), swatchOf("on_surface", "Text")];

const AUTO: PickedColour<Role> = { mode: "auto" };
const SECONDARY: PickedColour<Role> = { mode: "role", role: "secondary" };
const CUSTOM: PickedColour<Role> = { mode: "custom", h: 40, l: 0.8 };

type Options = Partial<ColourPickerOptions<Role>>;

/**
 * The picker's controls in a setting's well: Colour, and under it Hue and Tone. On `surface`, a
 * colour role, the well is that role and says so to the picker (`--k-colour-surface`).
 */
function well(options: Options = {}, surface?: string): HTMLElement {
  const made = colourPicker<Role>({ picked: AUTO, swatches: ROLES, ...options });
  // Each row is its label's slot and its control, `space.s12` apart; the slots line up as the
  // well's label column, as wide as its widest label.
  const line = (words: string, sub: boolean, control: HTMLElement) => {
    const said = text(words, "label", "on_surface_variant");
    said.toggleAttribute("data-sub", sub);
    const slot = document.createElement("span");
    slot.className = "f-colour-label";
    slot.append(said);
    const made = row("s12", "centre", slot, control);
    made.classList.add("f-colour-line");
    return made;
  };
  const box = stack("s4", "start", line("Colour", false, made.choice), line("Hue", true, made.hue), line("Tone", true, made.tone));
  box.classList.add("f-colour-well");
  if (surface) {
    box.style.background = `var(--color-${surface})`;
    box.style.setProperty("--k-colour-surface", `var(--color-${surface})`);
  }
  return within("var(--size-panel)", box);
}

/** `options`' well in `state`. */
function inState(options: Options, state: Shown): HTMLElement {
  return well({ ...options, ...stateOptions(state) });
}

/** Live: a pick draws the picker again with it, the keyboard kept on the choice. */
function live(): HTMLElement {
  let picked: PickedColour<Role> = SECONDARY;
  const room = document.createElement("span");
  const draw = () => {
    const focused = room.querySelector(".k-colour")?.matches(":focus-visible") ?? false;
    room.replaceChildren(
      well({
        picked,
        onPick: (colour) => {
          picked = colour;
          draw();
        },
        onTune: (h, l) => {
          picked = { mode: "custom", h, l };
        },
      }),
    );
    if (focused) room.querySelector<HTMLElement>(".k-colour")?.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
  };
  draw();
  return room;
}

const ROWS = [
  { name: "Auto picked", picked: AUTO },
  { name: "A role picked", picked: SECONDARY },
  { name: "Custom picked", picked: CUSTOM },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

const CHOICE = "the choice";

export default page({
  level: "molecules",
  slug: "colour-picker",
  title: "ColourPicker",
  lede: "A colour for something over a picture: Auto, a role of the scheme, or Custom by its hue and tone.",
  code: { unbuilt: "ColourPicker: not built yet" },
  from: ["chip", "slider"],
  stage: { live, hint: "click a swatch, or Tab and ← →", daylight: true },
  variants: [
    { name: "Auto, roles and Custom", for: "the lock clock's colour, in the clock editor", sample: () => well({ picked: SECONDARY }) },
    { name: "On another surface", for: "`--k-colour-surface` set: the ring's gap in `surface_container`", sample: () => well({ picked: SECONDARY }, "surface_container") },
  ],
  sizes: doesntApply("one size: swatches `size.chip`, the hue track `size.slider_compact`."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "the pick's `primary` ring, or Auto `Selected`, in every row",
      loading: "its colours are the scheme's, there at once",
      empty: "something is always picked",
      error: "it has no wrong pick",
    },
    rows: nonEmpty(ROWS.map(({ name, picked }) => ({ name, cell: (state: MatrixState) => inState({ picked }, state) }))),
    looks: {
      rest: "the pick ringed in `primary`; Hue and Tone live with Custom",
      lit: "the choice's focus tube; a hovered pick or track, its own",
      pressed: "next pick's tube white-hot; track held, hues up `opacity.held`",
      disabled: "everything at `opacity.disabled_content`, the hues' chroma lowered too",
    },
  },
  focus: {
    tubes: [
      { name: "The choice, keyboard on it", tone: "accent", filled: false, content: false },
      { name: "The hue track, keyboard on it", tone: "accent", filled: false, content: false },
      { name: "A swatch, the pointer on it", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "tube round the choice or the track, glowing",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => inState({ picked: ROWS[index]!.picked }, "signs-off"),
  },
  content: {
    text: doesntApply("its words are its user's: Auto, Custom and Hue by default, and the hue in degrees."),
    missingPicture: doesntApply("no pictures."),
    counts: [
      { name: "No roles", for: "Auto and Custom alone", sample: () => well({ picked: AUTO, swatches: [] }) },
      { name: "Four roles", for: "the clock's: Primary, Secondary, Cold, Text", sample: () => well({ picked: SECONDARY }) },
      {
        name: "More than the row holds",
        for: "the choice wraps onto a second line",
        sample: () =>
          well({
            picked: SECONDARY,
            swatches: [...ROLES, swatchOf("tertiary", "Tertiary"), swatchOf("error", "Error"), swatchOf("success", "Success"), swatchOf("warning", "Warning")],
          }),
      },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with the form it is in."),
    leaves: doesntApply("it goes with the form it is in."),
    between: [
      {
        name: "The keyboard comes to the choice",
        what: "The tube round the choice lights over `short` on `standard`; a pick moves the ring at once.",
        tracks: lights(".k-colour > .k-ring", CHOICE, 0, 1),
        reduced: "same",
        back: "cuts",
        sample: () => well({ picked: SECONDARY }),
      },
      ...powerMoves(() => inState({ picked: SECONDARY }, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "h"], does: "on the choice: picks the one before, round from the first" },
      { keys: ["→", "l"], does: "on the choice: picks the one after, round from the last" },
      { keys: ["←", "→", "h", "l"], does: "on Hue: a twentieth of the wheel; Shift, a degree" },
      { keys: ["Home", "End"], does: "on Hue: the first or the last degree" },
      { keys: ["Tab", "Shift+Tab"], does: "the choice, then Hue and Tone while Custom is picked" },
    ],
    order: "The choice is one stop; Hue and Tone are stops only while Custom is picked.",
    lands: "A pick keeps the keyboard on the choice; its form says where it lands first.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Custom picked", for: "the choice, the swatches, the hue track and Tone", sample: () => well({ picked: CUSTOM }) },
      { name: "A role picked", for: "the picked swatch's ring", sample: () => well({ picked: SECONDARY }) },
    ]),
    measures: [
      { part: "Choice padding", token: "space-s4" },
      { part: "Choice ends, past the ring", token: "stroke-thin" },
      { part: "Between picks", token: "space-s8" },
      { part: "Swatch", token: "size-chip" },
      { part: "Picked ring and its gap", token: "stroke-thin" },
      { part: "Swatch rim", token: "stroke-hairline_device_px" },
      { part: "Hue track height", token: "size-slider_compact" },
      { part: "Hue knob, inside its ring", token: "size-slider_stop" },
      { part: "Hue value from the end", token: "space-s12" },
      { part: "Choice and track corners", token: "radius-full" },
    ],
    reflow: doesntApply("the choice wraps in a narrow row; Hue and Tone fill theirs."),
  },
});
