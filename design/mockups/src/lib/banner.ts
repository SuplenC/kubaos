// A message that matters on the page itself, with what to do about it inside: `kubaos-ui`'s
// `banner::banner`, `KubaOS.Components`' `Banner` (`docs/design/components/banner.md`). A status
// message (`lib/status.ts`) in one of four tones, its rail in `primary`, `warning`, `error` or `cold`.
// Every tone lies flat in the page as a card does: one container role above what it sits on,
// `surface_container` on a window's `surface` (a host on a container names the role above its own
// as `--card-fill`). In it the symbol in the tone at `subheading`'s 24 on the title's first line,
// `space.s16` from a `subheading` title in `on_surface` over a `body` line and any paragraphs more in
// `on_surface_variant`, `space.s4` apart, all wrapping; its button bar `space.s16` under the words,
// across their width, or one button beside them at the end, centred against them (`aside`); an
// optional foot `space.s16` under the rest (a held button's timed bar).
//
// Small, it stands outside a card, under a section's heading or alone on a page: the symbol at
// `label`'s 16 beside a `label` title in the tone, `space.s8` apart, the lines under them in
// `small`, `space.s4` apart, starting where the symbol does. Line, it lives inside a card, under the
// row it is about: no box and no title, the rail down the start of its one `small` line, the symbol
// before the words if it has one, wrapping under them.
//
// A tip is a `Tip` banner in `cold`: how to do something faster. Titled ("Tip", Small) outside a
// card; a line (Line) inside one, under its setting. Any line names keys as `[Key]`, each a keycap
// where it stands (`sentence` in `lib/keycap.ts`).

import { icon } from "./icon.ts";
import { sentence } from "./keycap.ts";
import { filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { type StatusTone, statusBox } from "./status.ts";
import { text } from "./text.ts";

export const BANNER_TONES = ["Info", "Warning", "Danger", "Tip"] as const;
export type BannerTone = (typeof BANNER_TONES)[number];

const TONES: Record<BannerTone, StatusTone> = { Info: "primary", Warning: "warning", Danger: "error", Tip: "cold" };

/** A page's banner; a small one outside a card; a line inside one, under its row. */
export type BannerSize = "Regular" | "Small" | "Line";

export interface BannerOptions {
  readonly tone: BannerTone;
  readonly size?: BannerSize;
  /** Its symbol; a Line may go without. */
  readonly icon?: string;
  /** Its title; a Line has none. */
  readonly title?: string;
  /** The line under the title (a Line's one line); empty, the title stands alone, a line tall. `[Key]` is a keycap. */
  readonly body: string;
  /** More of the body, a paragraph each: the steps of what to do. */
  readonly paragraphs?: readonly string[];
  /** What to do about it: its button bar, `lib/button-bar.ts`. */
  readonly actions?: Element;
  /**
   * One button beside the words instead of a bar under them (Regular only): at the end, `space.s16`
   * from them, centred against the title and its lines, so it takes no row of its own.
   */
  readonly aside?: Element;
  /** A line across its foot: how far a held button is. */
  readonly foot?: Element;
}

export function banner(options: BannerOptions): HTMLElement {
  const tone = TONES[options.tone];
  const size = options.size ?? "Regular";
  const box = size === "Line" ? lineBox(tone, options) : statusBox(tone, "var(--card-fill, var(--color-surface_container))", size === "Small" ? "small" : "regular", filling(boxed(tone, size === "Small", options)));
  box.classList.add("k-banner");
  box.dataset.tone = options.tone;
  return probe(box, { role: "component", name: "Banner", variant: size === "Regular" ? options.tone : `${options.tone} ${size}`, fixed: { width: false, height: false } });
}

/** What a Regular or a Small banner holds: its symbol and title over its lines, then its actions and its foot. */
function boxed(tone: StatusTone, small: boolean, options: BannerOptions): HTMLElement {
  const glyph = icon(options.icon ?? "info", small ? "var(--type-label-icon)" : "var(--type-subheading-icon)", `var(--color-${tone})`);
  const title = options.title ?? "";
  const more = [...(options.body === "" ? [] : [options.body]), ...(options.paragraphs ?? [])];
  const said = small
    ? stack("s4", "start", row("s8", "centre", glyph, text(title, "label", tone)), ...more.map((each) => filling(sentence(each, "small", "on_surface_variant"))))
    : row("s16", "start", glyph, filling(stack("s4", "start", sentence(title, "subheading", "on_surface"), ...more.map((each) => sentence(each, "body", "on_surface_variant")))));
  const beside = !small && options.aside ? row("s16", "centre", filling(said), options.aside) : said;
  const parts = [beside, ...(options.actions ? [options.actions] : []), ...(options.foot ? [options.foot] : [])];
  return stack(small ? "s12" : "s16", "start", ...parts.map((part) => (part instanceof HTMLElement ? filling(part) : part)));
}

/**
 * A Line: its one line in `small`, the symbol standing first in it if it has one, the lines it
 * wraps to starting where its words do, every line a keycap and `space.s4` tall whether it names a
 * key or not, so a tip is as tall as any other of as many lines.
 */
function lineBox(tone: StatusTone, options: BannerOptions): HTMLElement {
  // Every line a keyed sentence's height, whether it names a key or not.
  const said = sentence(options.body, "small", "on_surface_variant");
  said.classList.add("k-said");
  // The symbol in a slot a line tall beside the words, so the lines they wrap to start where they do.
  const lead: HTMLElement[] = [];
  if (options.icon) {
    const slot = document.createElement("span");
    slot.className = "k-said-symbol";
    slot.append(icon(options.icon, "var(--type-small-icon)", `var(--color-${tone})`));
    lead.push(probe(slot, { role: "box" }));
  }
  return statusBox(tone, "transparent", "line", filling(row("s8", "start", ...lead, filling(said))));
}

/** Whether a tip's line leads with its symbol: the one switch between the two looks. */
export const TIP_SYMBOL = true;

/**
 * A tip outside a card, under a section's heading or alone on a page: a Small `Tip` banner, its
 * lightbulb and "Tip" over how to do something faster, one whole sentence naming its keys as
 * `[Key]` where they fall.
 */
export function tip(said: string): HTMLElement {
  return banner({ tone: "Tip", size: "Small", icon: "lightbulb", title: "Tip", body: said });
}

/** A tip inside a card, under the setting it is about: a `Tip` Line, its lightbulb first if `symbol`. */
export function tipLine(said: string, symbol = TIP_SYMBOL): HTMLElement {
  return banner({ tone: "Tip", size: "Line", ...(symbol ? { icon: "lightbulb" } : {}), body: said });
}
