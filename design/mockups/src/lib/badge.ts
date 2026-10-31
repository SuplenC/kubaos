// What is new about something, over its icon's top right corner: `kubaos-ui`'s `badge::badge` and
// `badge::accent_badge`, `KubaOS.Components`' `Badge` and `BadgedIcon`
// (`docs/design/components/badge.md`). A count in a pill `size.badge` tall in `label` type, a
// circle for one digit and `space.s4` either side for more, 99+ past 99; or a dot, `size.badge_dot`.
// In `error` with `on_error` on it, or in `primary` with `on_primary` where it marks something
// unread. In a fixed meaning (success, warning, sharing, microphone, recording) it is only a dot:
// those meanings have no `on_*` role a count could be drawn in. It has no states: it is a mark, not
// a control.

import { icon } from "./icon.ts";
import { FIXED_HEIGHT, probe, variant } from "./probe.ts";
import { advance, text } from "./text.ts";

/** The most a badge counts to; past it, it reads as this and a plus. */
export const MOST = 99;

/** The fixed meanings a badge can be in besides the error's, each only as a dot. */
export const MEANINGS = ["success", "warning", "sharing", "microphone", "recording"] as const;
export type Meaning = (typeof MEANINGS)[number];

export type BadgeOptions =
  | {
      /** How many, or none for a dot. */
      readonly count?: number;
      /** In the accent: something unread rather than something wrong. */
      readonly accent?: boolean;
      readonly meaning?: never;
    }
  | {
      /** A dot in a fixed meaning's colour: a meaning with no `on_*` role holds no count. */
      readonly meaning: Meaning;
      readonly count?: never;
      readonly accent?: never;
    };

/** What a badge with `count` says: the count, 99+ past 99. */
export function badgeWords(count: number): string {
  return count > MOST ? `${MOST}+` : String(count);
}

/**
 * The badge's width as CSS: a dot's side, a one-digit circle's, or its words' width rounded up to
 * a device pixel and `space.s4` either side.
 */
export function badgeWidth(count: number | undefined): string {
  if (count === undefined) return "var(--size-badge_dot)";
  const words = badgeWords(count);
  if (words.length === 1) return "var(--size-badge)";
  return `calc(round(up, calc(${advance("label", words)} * var(--px) - 0.001px), 1px) + 2 * var(--space-s4))`;
}

export function badge(options: BadgeOptions = {}): HTMLElement {
  const pill = document.createElement("span");
  pill.className = "k-badge";
  pill.toggleAttribute("data-accent", options.accent ?? false);
  pill.style.setProperty("--badge-width", badgeWidth(options.count));
  if (options.meaning) pill.style.setProperty("--badge-tone", `var(--color-${options.meaning})`);
  const tone = options.accent ? "accent" : options.meaning;
  if (options.count === undefined) {
    pill.dataset.dot = "";
    return probe(pill, { role: "component", name: "Badge", variant: variant("dot", tone), fixed: FIXED_HEIGHT });
  }
  const words = badgeWords(options.count);
  pill.style.setProperty("--natural", String(advance("label", words)));
  pill.append(text(words, "label", options.accent ? "on_primary" : "on_error"));
  return probe(pill, { role: "component", name: "Badge", variant: variant("count", tone), fixed: FIXED_HEIGHT });
}

export interface BadgedOptions {
  /** The symbol, by its name. */
  readonly name: string;
  /** Whether the badge shows. */
  readonly badged?: boolean;
  /** How many, or none for a dot. */
  readonly count?: number;
}

/**
 * The bar's symbol, 16 in `on_surface_variant`, with a badge centred on its top right corner while
 * `badged`: QML's `BadgedIcon`. While it shows, the box takes the half of the badge past the
 * symbol on both sides, so the badge keeps the row's gap and the symbol stays centred.
 */
export function badgedIcon(options: BadgedOptions): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-badged";
  box.append(icon(options.name, "var(--icon-s16)", "var(--color-on_surface_variant)"));
  if (options.badged) {
    box.dataset.badged = "";
    box.style.setProperty("--badge-width", badgeWidth(options.count));
    box.append(badge(options.count === undefined ? {} : { count: options.count }));
  }
  return box;
}
