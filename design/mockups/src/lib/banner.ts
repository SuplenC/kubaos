// A message that matters on the page itself, with what to do about it inside: `kubaos-ui`'s
// `banner::banner`, `KubaOS.Components`' `Banner` (`docs/design/components/banner.md`). A status
// message (`lib/status.ts`) in one of three tones: a symbol at `subheading`'s 24, `space.s16` from
// a `subheading` title over a `body` line and any paragraphs more, `space.s4` apart, all wrapping;
// its actions `space.s12` apart at its end beside the words, the one it is about last; an optional
// foot `space.s16` under the rest (a held button's timed bar).
//
// Info and Warning lie on `surface_container_high`, their edge and symbol in `primary` and
// `warning`. Danger is really red, so the step that destroys something can't be skimmed:
// `error_container`, its edge and symbol in `error`, its title in `on_error_container` and its body
// in it at `opacity.subdued_text`; on its red a Text button takes `on_error_container` and the
// foot's bar `error`, the banner's own colours, not the accent (`banner::inside`).

import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { type StatusTone, statusBox } from "./status.ts";
import { wrapped } from "./text.ts";

export const BANNER_TONES = ["Info", "Warning", "Danger"] as const;
export type BannerTone = (typeof BANNER_TONES)[number];

interface Colours {
  readonly tone: StatusTone;
  readonly fill: string;
  readonly title: string;
  readonly body: string;
}

const COLOURS: Record<BannerTone, Colours> = {
  Info: { tone: "primary", fill: "var(--color-surface_container_high)", title: "var(--color-on_surface)", body: "var(--color-on_surface_variant)" },
  Warning: { tone: "warning", fill: "var(--color-surface_container_high)", title: "var(--color-on_surface)", body: "var(--color-on_surface_variant)" },
  Danger: {
    tone: "error",
    fill: "var(--color-error_container)",
    title: "var(--color-on_error_container)",
    body: "rgb(from var(--color-on_error_container) r g b / var(--opacity-subdued_text))",
  },
};

export interface BannerOptions {
  readonly tone: BannerTone;
  readonly icon: string;
  readonly title: string;
  /** The line under the title; empty, the title stands alone, a line tall. */
  readonly body: string;
  /** More of the body, a paragraph each: the steps of what to do. */
  readonly paragraphs?: readonly string[];
  /** What to do about it, the one it is about last: buttons. */
  readonly actions?: readonly Element[];
  /** A line across its foot: how far a held button is. */
  readonly foot?: Element;
}

/**
 * `button`, a Text button, in a Danger banner's own colours: its label, line and glow in
 * `on_error_container`, so the accent doesn't read as a stray colour on the red.
 */
export function onRed<E extends HTMLElement>(button: E): E {
  const ink = "var(--color-on_error_container)";
  for (const property of ["--label", "--label-lit", "--label-off", "--edge", "--edge-off"]) button.style.setProperty(property, ink);
  const ring = button.querySelector<HTMLElement>(":scope > .k-ring");
  for (const property of ["--tube-lit", "--core-lit", "--tube-unlit", "--core-unlit"]) ring?.style.setProperty(property, ink);
  return button;
}

/** A body line or paragraph in `colour`. */
function said(words: string, colour: string): HTMLElement {
  const line = wrapped(words, "body");
  line.style.color = colour;
  return line;
}

export function banner(options: BannerOptions): HTMLElement {
  const colours = COLOURS[options.tone];
  const title = wrapped(options.title, "subheading");
  title.style.color = colours.title;
  const lines = [title, ...(options.body === "" ? [] : [said(options.body, colours.body)]), ...(options.paragraphs ?? []).map((each) => said(each, colours.body))];
  const words = filling(stack("s4", "start", ...lines));
  const parts: Element[] = [icon(options.icon, "var(--type-subheading-icon)", `var(--color-${colours.tone})`), words];
  if (options.actions && options.actions.length > 0) parts.push(row("s12", "centre", ...options.actions));
  const content = stack("s16", "start", filling(row("s16", "centre", ...parts)), ...(options.foot ? [filling(stack(0, "start", options.foot))] : []));
  const box = statusBox(colours.tone, colours.fill, filling(content));
  box.classList.add("k-banner");
  box.dataset.tone = options.tone;
  return probe(box, { role: "component", name: "Banner", variant: options.tone, fixed: { width: false, height: false } });
}
