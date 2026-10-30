// A button with a label: `kubaos-ui`'s `button::button`, `KubaOS.Components`' `Button`. Six kinds
// by what it is for, three sizes; an optional icon leads the label, or trails it on a button that
// goes forward. Its measures are the specimens' (`docs/design/components/button.md`).
//
// Its states are the neon's (`docs/DESIGN.md`, "States"): rest, lit (hovered or focused, one
// state), pressed and disabled. Lit has two looks: with the signs on, the neon burns; with them
// off, by day always and at night while a power change has the lights out, the line stays inside
// the edge as unlit glass, with no glow. Each kind names the colour roles of its fill, label and
// line in each state and look, every one a derived role computed from the live tone, and the CSS
// crossfades them as the control lights, as the signs go off and on, and as it is pressed.

import { type Pinned, pin } from "./control.ts";
import { focusRing, type Tone, toneColour } from "./focus.ts";
import { icon } from "./icon.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { type TypeStyle, text } from "./text.ts";

export const KINDS = ["Filled", "Tonal", "Text", "Danger", "Success", "Warning"] as const;
export type Kind = (typeof KINDS)[number];

/** The kinds that carry a meaning rather than an emphasis, and burn: always lit. */
export const BURNING: readonly Kind[] = ["Danger", "Success", "Warning"];

export const SIZES = ["Sm", "Md", "Lg"] as const;
export type Size = (typeof SIZES)[number];

export type { Pinned } from "./control.ts";

/** A size's measures, every one a token. */
export interface Metrics {
  readonly height: string;
  readonly label: TypeStyle;
  readonly gap: string;
  /** Inside the sides of every kind with a fill or a container. */
  readonly padding: string;
  /** Inside a text button, which has no edge to keep the label from. */
  readonly textPadding: string;
}

export const METRICS: Record<Size, Metrics> = {
  Sm: { height: "control_sm", label: "button_small", gap: "s4", padding: "s12", textPadding: "s12" },
  Md: { height: "control_md", label: "button", gap: "s8", padding: "s24", textPadding: "s12" },
  Lg: { height: "control_lg", label: "button_large", gap: "s8", padding: "s32", textPadding: "s16" },
};

/**
 * A kind's colours in each state, by colour role: its fill and its label at rest, lit and pressed,
 * and the line inside its edge, lit and pressed; and lit with the signs off, its fill, label and
 * glass line. A null fill is none. Pressed, a label is its lit one, and a kind whose pressed fill or
 * line changes only by day or by night says so in CSS. By day the signs-off roles are the lit ones,
 * so daylight lit and signs off are one look.
 */
export interface Colours {
  readonly fill: string | null;
  readonly fillLit: string | null;
  readonly fillOff: string | null;
  readonly fillPressed: string | null;
  readonly label: string;
  readonly labelLit: string;
  readonly labelOff: string;
  readonly line: string;
  readonly lineOff: string;
}

/** Each kind's colours (`docs/design/components/button.md`). */
export const COLOURS: Record<Kind, Colours> = {
  Filled: {
    fill: "primary_fill",
    fillLit: "primary_fill",
    fillOff: "primary_fill",
    fillPressed: "primary_fill_pressed",
    label: "on_primary_fill",
    labelLit: "on_primary_fill_lit",
    labelOff: "on_primary_fill",
    line: "primary_fill_line",
    lineOff: "primary_fill_glass",
  },
  Tonal: {
    fill: "secondary_container",
    fillLit: "secondary_container",
    fillOff: "secondary_container",
    fillPressed: "secondary_container_pressed",
    label: "on_secondary_container_rest",
    labelLit: "on_secondary_container",
    labelOff: "on_secondary_container_rest",
    line: "primary",
    lineOff: "primary_glass",
  },
  Text: {
    fill: null,
    fillLit: null,
    fillOff: null,
    fillPressed: null,
    label: "primary_rest",
    labelLit: "primary_lit",
    labelOff: "primary_rest",
    line: "primary",
    lineOff: "primary_glass",
  },
  Danger: burning("burn_error"),
  Success: burning("burn_success"),
  Warning: burning("burn_warning"),
};

/** A burning kind's colours: signs off, its fill stays lit, since a fill isn't a light, and its line is glass. */
function burning(tone: string): Colours {
  return {
    fill: tone,
    fillLit: `${tone}_lit`,
    fillOff: `${tone}_lit`,
    fillPressed: `${tone}_pressed`,
    label: `on_${tone}`,
    labelLit: `on_${tone}`,
    labelOff: `on_${tone}`,
    line: `${tone}_line`,
    lineOff: `${tone}_glass`,
  };
}

/** The tone each kind's line and glow light in: the burning kinds their own, the others the accent. */
export const TONES: Record<Kind, Tone> = {
  Filled: "accent",
  Tonal: "accent",
  Text: "accent",
  Danger: "burn_error",
  Success: "burn_success",
  Warning: "burn_warning",
};

/**
 * Whether a kind's label glows when it is lit at night, and how: a Text button's label is drawn
 * in its tone with nothing under it, so it burns as a tube with the content glow; a Filled
 * button's near-white letters light with a tiny glow. The burning kinds' and Tonal's don't glow.
 */
export function labelGlow(kind: Kind): "content" | "label" | null {
  if (kind === "Text") return "content";
  return kind === "Filled" ? "label" : null;
}

/** Whether a kind has a fill, so the line inside its edge is the tone's core, `stroke.thin` wide. */
export function filled(kind: Kind): boolean {
  return kind === "Filled" || BURNING.includes(kind);
}

export interface ButtonOptions {
  readonly label: string;
  readonly kind?: Kind;
  readonly size?: Size;
  readonly icon?: string;
  readonly trailing?: boolean;
  readonly disabled?: boolean;
  readonly pinned?: Pinned;
}

/** A symbol in a box as wide as itself and as tall as the label's line beside it. */
function iconInLine(name: string, style: TypeStyle, color: string): HTMLElement {
  const slot = document.createElement("span");
  slot.className = "k-icon-line";
  slot.style.setProperty("--line", `var(--type-${style}-line)`);
  slot.style.setProperty("--icon-size", `var(--type-${style}-icon)`);
  slot.append(icon(name, `var(--type-${style}-icon)`, color));
  return slot;
}

/** A colour role as CSS, or none. */
const role = (name: string | null) => (name ? `var(--color-${name})` : "transparent");

/**
 * Sets `element`'s colours as `kind`'s, its fill, label and line in every state and look, as the
 * custom properties `.k-kind` crossfades.
 */
export function paintKind(element: HTMLElement, kind: Kind): void {
  const colours = COLOURS[kind];
  element.dataset.kind = kind;
  element.toggleAttribute("data-hot", kind === "Text");
  element.style.setProperty("--fill", role(colours.fill));
  element.style.setProperty("--fill-lit", role(colours.fillLit));
  element.style.setProperty("--fill-off", role(colours.fillOff));
  element.style.setProperty("--fill-pressed", role(colours.fillPressed));
  element.style.setProperty("--label", role(colours.label));
  element.style.setProperty("--label-lit", role(colours.labelLit));
  element.style.setProperty("--label-off", role(colours.labelOff));
  element.style.setProperty("--edge", role(colours.line));
  element.style.setProperty("--edge-off", role(colours.lineOff));
}

/**
 * Lights `element` as `kind` lights, at the radius token `corner`: its ring in the kind's tone,
 * and its content, drawn in its tone or on a fill, glowing where the kind's does. Returns the ring.
 */
export function lightKind(element: HTMLElement, kind: Kind, corner: string, content: HTMLElement): HTMLElement {
  const glowing = labelGlow(kind);
  const ring = focusRing(element, corner, TONES[kind], filled(kind), glowing ? { content, glow: glowing } : undefined);
  ring.dataset.pressable = "";
  return ring;
}

export function button(options: ButtonOptions): HTMLButtonElement {
  const kind = options.kind ?? "Filled";
  const size = options.size ?? "Md";
  const m = METRICS[size];
  const element = document.createElement("button");
  element.className = "k-button k-kind k-control";
  element.type = "button";
  element.disabled = options.disabled ?? false;
  pin(element, options.pinned);
  paintKind(element, kind);
  element.style.setProperty("--height", `var(--size-${m.height})`);
  element.style.setProperty("--line", `var(--type-${m.label}-line)`);
  element.style.setProperty("--padding", `var(--space-${kind === "Text" ? m.textPadding : m.padding})`);
  const shown = "var(--shown)";
  const label = text(options.label, m.label);
  label.style.color = shown;
  const glyph = options.icon ? iconInLine(options.icon, m.label, shown) : null;
  const children = glyph ? (options.trailing ? [label, glyph] : [glyph, label]) : [label];
  const content = row(m.gap, "start", ...children);
  element.append(content);
  lightKind(element, kind, "control", content);
  const placement = !options.icon ? "" : options.trailing ? "trailing-icon" : "leading-icon";
  return probe(element, { role: "component", name: "Button", variant: variant(kind, size, placement), fixed: FIXED_HEIGHT });
}

/** The colour role a kind's glow is drawn in. */
export function glowColour(kind: Kind): string {
  return toneColour(TONES[kind]);
}
