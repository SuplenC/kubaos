// The Icon page: `docs/design/components/icon.md`. A symbol of KubaOS Symbols in a square box of
// an icon size, taking its size from the text beside it, in a colour role. It isn't a control: it
// has no states of its own and dims with the control it is in. Foundations, Icons shows the set.

import { iconButton } from "../../lib/icon-button.ts";
import { icon, missingIcon } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { text, type TypeStyle } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const SIZES = ["s16", "s20", "s24", "s32", "s48"] as const;

/** What each icon size sits beside, from `docs/DESIGN.md`, "Icons by type". */
const BESIDE: Record<(typeof SIZES)[number], string> = {
  s16: "Beside `caption`, `small`, `label`, `mono`, `keycap`, `accent` and `button_small`: a chip's, a hint's, the bar's.",
  s20: "Beside `body`, `body_strong`, `body_large`, `mono_large` and `button`: a row's, a field's, a regular button's.",
  s24: "Beside `subheading`, `heading` and `button_large`, and beside two lines (`icon.two_line`): an error's, a large button's, a two-line row's.",
  s32: "Beside `headline` and `title`, and above an `EmptyState`'s words.",
  s48: "Beside `display`, `hero`, `hero_number` and `clock`, and a page's one big symbol above its words.",
};

/** The colour roles an icon is drawn in, by what it says. */
const ROLES = ["on_surface", "on_surface_variant", "primary", "error", "success", "warning", "sharing"] as const;

/** `name` beside `words` in `style`, at the style's icon size, centred down the line. */
function beside(name: string, words: string, style: TypeStyle, color = "on_surface"): HTMLElement {
  return row("s8", "centre", icon(name, `var(--type-${style}-icon)`, `var(--color-${color})`), text(words, style, color));
}

/** A symbol in a disabled control's content colour, `on_surface` at `opacity.disabled_content`, when `disabled`. */
function shown(name: string, size: string, disabled: boolean, color = "on_surface"): SVGElement {
  return icon(name, size, disabled ? `rgb(from var(--color-${color}) r g b / var(--opacity-disabled_content))` : `var(--color-${color})`);
}

/** A row of symbols in every role. */
function roles(): HTMLElement {
  return row("s16", "centre", ...ROLES.map((role) => icon("wifi", "var(--icon-s20)", `var(--color-${role})`)));
}

const DESCRIBED =
  "Drawn from its SVG, Tabler's 24 unit grid, with the stroke set for its size and the scale it shows at: 2 units scaled to the size, rounded half up to whole device pixels and never under one (`stroke.symbol`), so no line blurs between two. The box is square, its ink 2 units clear inside.";

export default page({
  level: "atoms",
  slug: "icon",
  title: "Icon",
  summary: "A symbol of KubaOS Symbols in a square box of an icon size, the size of the text beside it, in a colour role.",
  live: () =>
    stack(
      "s16",
      "start",
      beside("wifi", "Wi-Fi", "body_strong"),
      beside("warning", "Couldn't connect", "subheading", "error"),
      beside("keyboard", "Mac accents", "label", "on_surface_variant"),
      roles(),
    ),
  description: [
    "Rust `icon::icon(look, glyph, size, color)`, QML `Icon` (`name`, `size`, `color`): a symbol by the Material Symbols name it replaces (`arrow_forward`), in a square box of one of the icon sizes, 16, 20, 24, 32 or 48. Foundations, Icons has the whole set, the five drawn by us and the stroke table.",
    "Beside text it takes the size of the text's style (`Type::icon()`, `Tokens.type.<style>.icon`), never a size of its own, and never taller than the line, so a row with an icon is as tall as a row without. Above text it takes the size its place names: an `EmptyState`'s 32, a page's one big symbol 48.",
    DESCRIBED,
    "Its colour is a role. The colour's alpha goes to the drawing's opacity, so an icon in a disabled control, or on a fading screen, dims with its label. It has no states of its own. Alone in its slot (an icon button's), its box is centred on a whole device pixel, and the geometry check holds it to the slot's centre (`icon-centre`).",
    "At 16 at 1× the stroke stays one device pixel, 1.33 rounding to 1, so the small symbols aren't heavier than the text beside them; they read a little grainy there, which is known and parked.",
  ],
  variants: [
    { name: "Beside text", for: "A row's, a button's, a chip's symbol: the style's icon size, centred down its line.", sample: () => beside("wifi", "Wi-Fi", "body_strong") },
    {
      name: "Beside two lines",
      for: "A title over a subtitle (`body_strong` over `small`): the icon spans both, `icon.two_line`.",
      sample: () => row("s12", "centre", icon("hard_drive", "var(--icon-two_line)", "var(--color-on_surface_variant)"), stack(0, "start", text("Samsung 990 Pro", "body_strong"), text("2 TB · NVMe", "small", "on_surface_variant"))),
    },
    { name: "Above text", for: "An empty state's symbol, 32, and a page's one big symbol, 48, over their words.", sample: () => stack("s12", "centre", icon("search_off", "var(--icon-s32)", "var(--color-on_surface_variant)"), text("Nothing found", "subheading")) },
    { name: "Alone in a slot", for: "An icon button's symbol: centred in the slot on a whole device pixel.", sample: () => iconButton({ icon: "settings", kind: "Tonal", size: "Md", label: "Settings" }) },
  ],
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.slice(1), for: BESIDE[size], sample: () => icon("wifi_lock", `var(--icon-${size})`, "var(--color-on_surface)") }))),
  states: {
    shown: ["rest", "disabled"],
    absent: {
      lit: "an icon takes no pointer and no focus; in a lit control it takes the control's lit content colour.",
      pressed: "it takes no press; it moves with its control.",
      selected: "it holds no choice: a picked control recolours its icon.",
      loading: "a symbol is drawn at once from the set; a picture on its way is a `Media` box's.",
      empty: "an icon always names a symbol; a name with no drawing is under Content.",
      error: "it has no value to be wrong: an error's symbol is `error` in its role.",
    },
    rows: [
      { name: "16, on_surface_variant", cell: (state) => shown("keyboard", "var(--icon-s16)", state === "disabled", "on_surface_variant") },
      { name: "20, on_surface", cell: (state) => shown("wifi", "var(--icon-s20)", state === "disabled") },
      { name: "24, error", cell: (state) => shown("warning", "var(--icon-s24)", state === "disabled", "error") },
      { name: "48, primary", cell: (state) => shown("rocket_launch", "var(--icon-s48)", state === "disabled", "primary") },
    ],
    described: {
      rest: `In its role. ${DESCRIBED} By day the same roles in daylight colours.`,
      disabled: "An icon has no disabled state of its own: in a disabled control it takes the control's disabled content, `on_surface` at `opacity.disabled_content`, the alpha going to the drawing's opacity so it dims with its label.",
    },
  },
  focus: doesntApply("an icon isn't a control: it has no line and no glow. A lit control's icon lights with its label, as its page says."),
  content: {
    text: doesntApply("an icon has no words; a control's label beside it is a `Text`."),
    missingPicture: [
      { name: "A name with no drawing", for: "QML draws its box, empty, so nothing beside it moves; Rust names a symbol by its enum, so it can't name one that isn't there.", sample: () => row("s8", "centre", missingIcon("not_a_symbol", "var(--type-body_strong-icon)"), text("Wi-Fi", "body_strong")) },
      { name: "No icon", for: "A control or a row without a symbol starts with its words, and is as tall as one with.", sample: () => text("Wi-Fi", "body_strong") },
    ],
    counts: [
      { name: "One", for: "A symbol beside its words.", sample: () => beside("wifi", "Wi-Fi", "body_strong") },
      { name: "Many", for: "Symbols side by side in a row of the same size: the bar's, a toolbar's.", sample: roles },
    ],
  },
  motion: {
    arrives: doesntApply("an icon comes with the surface it is on."),
    leaves: doesntApply("an icon goes with the surface it is on."),
    between: powerMoves(() => stack("s12", "start", roles(), beside("warning", "Couldn't connect", "subheading", "error"))),
  },
  keyboard: doesntApply("an icon takes no focus and no key."),
  measures: {
    samples: nonEmpty([
      { name: "Every size", for: "The square boxes, 16 to 48.", sample: () => row("s16", "end", ...SIZES.map((size) => icon("wifi_lock", `var(--icon-${size})`, "var(--color-on_surface)"))) },
      { name: "Beside its text", for: "The style's icon size, centred down the line, the gap to the words.", sample: () => beside("wifi", "Wi-Fi", "body_strong") },
    ]),
    measures: [
      { what: "The smallest box: beside small text, in chips and the bar.", token: "icon-s16" },
      { what: "Beside body text, in rows, fields and regular buttons.", token: "icon-s20" },
      { what: "Beside a heading, in large buttons and errors, and beside two lines.", token: "icon-s24" },
      { what: "Beside a title, and above an empty state's words.", token: "icon-s32" },
      { what: "A page's one big symbol.", token: "icon-s48" },
      { what: "The gap between an icon and its words in a row of text.", token: "space-s8" },
    ],
    reflow: doesntApply("an icon is its size at every window size."),
  },
});
