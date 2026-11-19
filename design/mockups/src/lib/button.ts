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
//
// A quiet kind can wear the sharing colour instead of the accent (`colour: "sharing"`): the same
// kind, the same weight and states, in the sharing roles and lighting in violet, for the controls
// that share something off the machine (the share picker's).

import { type Pinned, pin } from "./control.ts";
import { focusRing, type Tone, toneColour } from "./focus.ts";
import { heldByKeys, holdable } from "./hold.ts";
import { icon } from "./icon.ts";
import { row } from "./layout.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { advance, type TypeStyle, text } from "./text.ts";

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

/** The colours a kind wears: the accent's, or the sharing violet's. */
export type ButtonColour = "accent" | "sharing";

/** The kinds that can wear the sharing colour, and their colours in it, by the accent kinds' recipes. */
export const SHARING_COLOURS: Partial<Record<Kind, Colours>> = {
  Tonal: {
    fill: "sharing_container",
    fillLit: "sharing_container",
    fillOff: "sharing_container",
    fillPressed: "sharing_container_pressed",
    label: "on_sharing_container_rest",
    labelLit: "on_sharing_container",
    labelOff: "on_sharing_container_rest",
    line: "sharing",
    lineOff: "sharing_glass",
  },
  Text: {
    fill: null,
    fillLit: null,
    fillOff: null,
    fillPressed: null,
    label: "sharing_rest",
    labelLit: "sharing_lit",
    labelOff: "sharing_rest",
    line: "sharing",
    lineOff: "sharing_glass",
  },
};

/** `kind`'s colours in `colour`; a kind with no sharing colours can't be asked for one. */
export function coloursOf(kind: Kind, colour: ButtonColour = "accent"): Colours {
  if (colour === "accent") return COLOURS[kind];
  const colours = SHARING_COLOURS[kind];
  if (!colours) throw new Error(`a ${kind} button has no sharing colour`);
  return colours;
}

/** The tone `kind` lights in when it wears `colour`. */
export function toneOf(kind: Kind, colour: ButtonColour = "accent"): Tone {
  return colour === "sharing" ? "sharing" : TONES[kind];
}

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
  /**
   * The labels it changes between (Show and Hide): it holds the widest one's width whichever it
   * says, so it keeps its place and size as it flips.
   */
  readonly alternatives?: readonly string[];
  /**
   * It discloses something: an `expand_more` chevron after the label, turned a half while what it
   * discloses is open (`--disclosed`, 0 to 1), with it over `medium` on `standard`.
   */
  readonly disclosure?: boolean;
  /**
   * Held, not pressed: what it does once held for `duration.hold` (`lib/hold.ts`). Its body fills
   * from its start meanwhile, in its kind's hold fill, its label and symbol readable either side of
   * the fill's edge.
   */
  readonly hold?: () => void;
  /** A held button drawn this far through its hold, 0 to 1: a sample's. */
  readonly held?: number;
  /** The accent's colours, or, for Tonal and Text, the sharing violet's. */
  readonly colour?: ButtonColour;
}

/**
 * What a held button fills with, and its label over the fill: the accent for the quiet kinds, the
 * core for the filled and burning ones, so the fill always stands off the button's own fill.
 */
const HOLD_FILL: Record<Kind, { readonly fill: string; readonly label: string }> = {
  Filled: { fill: "primary_fill_line", label: "on_primary_fill" },
  Tonal: { fill: "primary", label: "on_primary" },
  Text: { fill: "primary", label: "on_primary" },
  Danger: { fill: "burn_error_line", label: "on_burn_error" },
  Success: { fill: "burn_success_line", label: "on_burn_success" },
  Warning: { fill: "burn_warning_line", label: "on_burn_warning" },
};

/** What a sharing button fills with as it is held: the violet, as the quiet kinds fill with the accent. */
const SHARING_HOLD = { fill: "sharing", label: "on_sharing" } as const;

/** A symbol in a box as wide as itself and as tall as the label's line beside it. */
export function iconInLine(name: string, style: TypeStyle, color: string): HTMLElement {
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
 * Sets `element`'s colours as `kind`'s in `colour`, its fill, label and line in every state and
 * look, as the custom properties `.k-kind` crossfades, and the tone a Text kind's day press tints
 * with.
 */
export function paintKind(element: HTMLElement, kind: Kind, colour: ButtonColour = "accent"): void {
  const colours = coloursOf(kind, colour);
  element.dataset.kind = kind;
  element.style.setProperty("--kind-tone", `var(--color-${toneColour(toneOf(kind, colour))})`);
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
 * Lights `element` as `kind` lights in `colour`, at the radius token `corner`: its ring in the
 * kind's tone, and its content, drawn in its tone or on a fill, glowing where the kind's does.
 * Returns the ring.
 */
export function lightKind(element: HTMLElement, kind: Kind, corner: string, content: HTMLElement, colour: ButtonColour = "accent"): HTMLElement {
  const glowing = labelGlow(kind);
  const ring = focusRing(element, corner, toneOf(kind, colour), filled(kind), glowing ? { content, glow: glowing } : undefined);
  ring.dataset.pressable = "";
  return ring;
}

export function button(options: ButtonOptions): HTMLButtonElement {
  const kind = options.kind ?? "Filled";
  const colour = options.colour ?? "accent";
  const size = options.size ?? "Md";
  const m = METRICS[size];
  const element = document.createElement("button");
  element.className = "k-button k-kind k-control";
  element.type = "button";
  element.disabled = options.disabled ?? false;
  pin(element, options.pinned);
  paintKind(element, kind, colour);
  element.style.setProperty("--height", `var(--size-${m.height})`);
  element.style.setProperty("--line", `var(--type-${m.label}-line)`);
  element.style.setProperty("--padding", `var(--space-${kind === "Text" ? m.textPadding : m.padding})`);
  const shown = "var(--shown)";
  const label = text(options.label, m.label);
  label.classList.add("k-button-label");
  label.style.color = shown;
  if (options.alternatives?.length) {
    element.dataset.held = "";
    const widest = Math.max(...[options.label, ...options.alternatives].map((words) => advance(m.label, words)));
    label.style.setProperty("--natural", String(widest));
  }
  const disclosing = options.disclosure !== undefined;
  if (disclosing) element.style.setProperty("--disclosed", options.disclosure ? "1" : "0");
  const glyph = disclosing ? iconInLine("expand_more", m.label, shown) : options.icon ? iconInLine(options.icon, m.label, shown) : null;
  glyph?.classList.toggle("k-button-chevron", disclosing);
  const children = glyph ? (options.trailing || disclosing ? [label, glyph] : [glyph, label]) : [label];
  const content = row(m.gap, "start", ...children);
  element.append(content);
  if (options.hold || options.held !== undefined) {
    // The fill holds the same content in the fill's own label colour, cut to how far it has gone,
    // so the words read on either side of its edge, as a held menu item's sweep does.
    const fill = document.createElement("span");
    fill.className = "k-button-hold";
    fill.setAttribute("aria-hidden", "true");
    const holding = colour === "sharing" ? SHARING_HOLD : HOLD_FILL[kind];
    fill.style.setProperty("--hold-fill", role(holding.fill));
    fill.style.setProperty("--shown", role(holding.label));
    const copy = content.cloneNode(true) as HTMLElement;
    // The button's own content is what the geometry check reads; the fill's copy is paint.
    for (const each of [copy, ...copy.querySelectorAll<HTMLElement | SVGElement>("[data-probe]")]) delete each.dataset.probe;
    fill.append(copy);
    element.append(fill);
    element.dataset.hold = "";
    if (options.held !== undefined) element.style.setProperty("--hold", String(options.held));
    if (options.hold) heldByKeys(element, holdable(element, options.hold, { empties: true }));
  }
  lightKind(element, kind, "control", content, colour);
  const placement = disclosing ? "disclosure" : !options.icon ? "" : options.trailing ? "trailing-icon" : "leading-icon";
  return probe(element, { role: "component", name: "Button", variant: variant(kind, size, placement), fixed: FIXED_HEIGHT });
}

/** The colour role a kind's glow is drawn in. */
export function glowColour(kind: Kind): string {
  return toneColour(TONES[kind]);
}
