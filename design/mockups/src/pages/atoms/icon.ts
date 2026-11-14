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

/** What each icon size sits beside, from `docs/DESIGN.md`, "Icons by type". */
const BESIDE: Record<(typeof SIZES)[number], string> = {
  s16: "beside `small`, `label`, `caption`: a chip's, the bar's",
  s20: "beside `body`, `button`: a row's, a field's",
  s24: "beside `heading`, `button_large`, two lines",
  s32: "beside `title`; above an `EmptyState`'s words",
  s48: "beside `display`; a page's one big symbol",
};

export default page({
  level: "atoms",
  slug: "icon",
  title: "Icon",
  lede: "A symbol of KubaOS Symbols in a square box, the size of the text beside it.",
  code: { rust: "icon::icon(look, glyph, size, color)", qml: "Icon" },
  stage: {
    live: () =>
      row(
        "s48",
        "centre",
        stack("s16", "start", beside("wifi", "Wi-Fi", "body_strong"), beside("warning", "Couldn't connect", "subheading", "error"), beside("keyboard", "Mac accents", "label", "on_surface_variant")),
        stack("s24", "start", row("s16", "end", ...SIZES.map((size) => icon("wifi_lock", `var(--icon-${size})`, "var(--color-on_surface)"))), roles()),
      ),
    hint: "try Scale: strokes stay on whole pixels",
    daylight: true,
  },
  variants: [
    { name: "Beside text", for: "the style's icon size, centred down its line", sample: () => beside("wifi", "Wi-Fi", "body_strong") },
    {
      name: "Beside two lines",
      for: "a title over a subtitle: `icon.two_line`",
      sample: () => row("s12", "centre", icon("hard_drive", "var(--icon-two_line)", "var(--color-on_surface_variant)"), stack(0, "start", text("Samsung 990 Pro", "body_strong"), text("2 TB · NVMe", "small", "on_surface_variant"))),
    },
    { name: "Above text", for: "an empty state's, a page's one big symbol", sample: () => stack("s12", "centre", icon("search_off", "var(--icon-s32)", "var(--color-on_surface_variant)"), text("Nothing found", "subheading")) },
    { name: "Alone in a slot", for: "an icon button's, on a whole device pixel", sample: () => iconButton({ icon: "settings", kind: "Tonal", size: "Md", label: "Settings" }) },
  ],
  sizes: nonEmpty(SIZES.map((size) => ({ name: size.slice(1), for: BESIDE[size], sample: () => icon("wifi_lock", `var(--icon-${size})`, "var(--color-on_surface)") }))),
  states: {
    shown: ["rest", "disabled"],
    absent: {
      lit: "in a lit control it takes the lit content colour",
      pressed: "it moves with its control",
      selected: "a picked control recolours it",
      loading: "drawn at once; a picture on its way is `Media`'s",
      empty: "a name with no drawing is under Content",
      error: "an error's symbol is `error` in its role",
    },
    rows: [
      { name: "16, on_surface_variant", cell: (state) => shown("keyboard", "var(--icon-s16)", state === "disabled", "on_surface_variant") },
      { name: "20, on_surface", cell: (state) => shown("wifi", "var(--icon-s20)", state === "disabled") },
      { name: "24, error", cell: (state) => shown("warning", "var(--icon-s24)", state === "disabled", "error") },
      { name: "48, primary", cell: (state) => shown("rocket_launch", "var(--icon-s48)", state === "disabled", "primary") },
    ],
    looks: {
      rest: "in its role; the stroke `stroke.symbol`, on whole device pixels",
      disabled: "its control's: `on_surface` at `opacity.disabled_content`",
    },
  },
  focus: doesntApply("not a control: no line and no glow; a lit control's icon lights with its label."),
  content: {
    text: doesntApply("no words; a label beside it is a `Text`."),
    missingPicture: [
      { name: "A name with no drawing", for: "QML draws its box, empty; Rust can't name one", sample: () => row("s8", "centre", missingIcon("not_a_symbol", "var(--type-body_strong-icon)"), text("Wi-Fi", "body_strong")) },
      { name: "No icon", for: "starts with its words, as tall as one with", sample: () => text("Wi-Fi", "body_strong") },
    ],
    counts: [
      { name: "One", for: "beside its words", sample: () => beside("wifi", "Wi-Fi", "body_strong") },
      { name: "Many", for: "side by side at one size: the bar's", sample: roles },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface."),
    leaves: doesntApply("it goes with its surface."),
    between: powerMoves(() => stack("s12", "start", roles(), beside("warning", "Couldn't connect", "subheading", "error"))),
  },
  keyboard: doesntApply("no focus and no key."),
  measures: {
    samples: nonEmpty([
      { name: "Every size", for: "the square boxes", sample: () => row("s16", "end", ...SIZES.map((size) => icon("wifi_lock", `var(--icon-${size})`, "var(--color-on_surface)"))) },
      { name: "Beside its text", for: "its size, its gap", sample: () => beside("wifi", "Wi-Fi", "body_strong") },
    ]),
    measures: [
      { part: "16", token: "icon-s16" },
      { part: "20", token: "icon-s20" },
      { part: "24", token: "icon-s24" },
      { part: "32", token: "icon-s32" },
      { part: "48", token: "icon-s48" },
      { part: "Icon → words", token: "space-s8" },
    ],
    reflow: doesntApply("its size at every window size."),
  },
});
