// A message that matters on the page itself, with what to do about it inside: `kubaos-ui`'s
// `banner::banner`, `KubaOS.Components`' `Banner` (`docs/design/components/banner.md`). A status
// message (`lib/status.ts`) in one of three tones, its rail in `primary`, `warning` or `error`.
// Every tone lies flat in the page as a card does: one container role above what it sits on,
// `surface_container` on a window's `surface` (a host on a container names the role above its own
// as `--card-fill`). In it the symbol in the tone at `subheading`'s 24 on the title's first line,
// `space.s16` from a `subheading` title in `on_surface` over a `body` line and any paragraphs more in
// `on_surface_variant`, `space.s4` apart, all wrapping; its button bar `space.s16` under the words,
// across their width; an optional foot `space.s16` under the rest (a held button's timed bar).

import { icon } from "./icon.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { type StatusTone, statusBox } from "./status.ts";
import { wrapped } from "./text.ts";

export const BANNER_TONES = ["Info", "Warning", "Danger"] as const;
export type BannerTone = (typeof BANNER_TONES)[number];

const TONES: Record<BannerTone, StatusTone> = { Info: "primary", Warning: "warning", Danger: "error" };

export interface BannerOptions {
  readonly tone: BannerTone;
  readonly icon: string;
  readonly title: string;
  /** The line under the title; empty, the title stands alone, a line tall. */
  readonly body: string;
  /** More of the body, a paragraph each: the steps of what to do. */
  readonly paragraphs?: readonly string[];
  /** What to do about it: its button bar, `lib/button-bar.ts`. */
  readonly actions?: Element;
  /** A line across its foot: how far a held button is. */
  readonly foot?: Element;
}

export function banner(options: BannerOptions): HTMLElement {
  const tone = TONES[options.tone];
  const lines = [
    wrapped(options.title, "subheading", "on_surface"),
    ...[...(options.body === "" ? [] : [options.body]), ...(options.paragraphs ?? [])].map((each) => wrapped(each, "body", "on_surface_variant")),
  ];
  const said = row("s16", "start", icon(options.icon, "var(--type-subheading-icon)", `var(--color-${tone})`), filling(stack("s4", "start", ...lines)));
  const parts = [said, ...(options.actions ? [options.actions] : []), ...(options.foot ? [options.foot] : [])];
  const content = stack("s16", "start", ...parts.map((part) => (part instanceof HTMLElement ? filling(part) : part)));
  const box = statusBox(tone, "var(--card-fill, var(--color-surface_container))", filling(content));
  box.classList.add("k-banner");
  box.dataset.tone = options.tone;
  return probe(box, { role: "component", name: "Banner", variant: options.tone, fixed: { width: false, height: false } });
}
