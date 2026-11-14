// The ColourPicker page: `docs/design/components/colour-picker.md`. A colour picked for something
// drawn over a picture: Auto, a role of the scheme as a swatch, or Custom by its hue and tone. Its
// three controls are shown in a setting's well, as the wallpaper picker's clock editor lays them.

import { type ColourPickerOptions, colourPicker, type PickedColour, type Swatch } from "../../lib/colour-picker.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { within } from "../../lib/layout.ts";
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

/** The picker's controls in a setting's well: Colour, and under it Hue and Tone when offered. */
function well(options: Options = {}): HTMLElement {
  const made = colourPicker<Role>({ picked: AUTO, swatches: ROLES, ...options });
  const box = document.createElement("span");
  box.className = "f-colour-well";
  const label = (words: string, sub: boolean) => {
    const said = text(words, "label", "on_surface_variant");
    said.toggleAttribute("data-sub", sub);
    return said;
  };
  box.append(label("Colour", false), made.choice);
  if (made.hue && made.tone) box.append(label("Hue", true), made.hue, label("Tone", true), made.tone);
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
    { name: "Without Auto", for: "a colour with no automatic choice", sample: () => well({ picked: SECONDARY, auto: false }) },
    { name: "Roles only", for: "the scheme's roles alone: no Custom, no Hue or Tone", sample: () => well({ picked: SECONDARY, auto: false, custom: false }) },
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
      lit: "focus tube round the choice",
      pressed: "Auto's chip pressed; a swatch has no pressed look",
      disabled: "everything at `opacity.disabled_content`, the hues' chroma lowered too",
    },
  },
  focus: {
    tubes: [
      { name: "The choice, keyboard on it", tone: "accent", filled: false, content: false },
      { name: "The hue track, keyboard on it", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "tube round the choice or the track, glowing",
      signsOff: "the tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => inState({ picked: ROWS[index]!.picked }, "signs-off"),
  },
  content: {
    text: doesntApply("its words are fixed: Auto, and the hue in degrees."),
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
      { keys: ["←", "h"], does: "on the choice: picks the one before, stopping at the first" },
      { keys: ["→", "l"], does: "on the choice: picks the one after, stopping at the last" },
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
