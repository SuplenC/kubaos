// Where a sample uses each token, read from the cascade itself rather than guessed from pixels. The
// sample is cloned into a box that isn't drawn, and each token in turn is set to a value it never
// has on every element of the clone; an element whose computed geometry then changes takes that
// token, in the properties that changed. The cascade, the `.mock` boxes that declare the tokens
// again for their scale, custom properties standing between (`--gap: var(--space-s12)` read as
// `gap: var(--gap)`) and media queries are all the browser's own. A length worked out in script
// from a token's value is a number by the time CSS sees it, so the lib names those tokens on the
// box itself (`workedOut` in `lib/probe.ts`).

import { raw } from "../../lib/tokens.ts";

/** What a token sets on an element, which says how its place is drawn. */
export type Kind = "box" | "padding" | "gap" | "margin";

export type Side = "top" | "right" | "bottom" | "left";

/** One token on one element: the properties it reaches there, and how its place is drawn. */
export interface Use {
  readonly token: string;
  readonly kind: Kind;
  /** The sides a padding or margin takes it on. */
  readonly sides: readonly Side[];
  readonly properties: readonly string[];
}

/** Every element of a sample with the tokens it takes. */
export type Uses = Map<Element, Use[]>;

const SIDES: readonly Side[] = ["top", "right", "bottom", "left"];

/**
 * The properties read, each a longhand, none inherited but the type's: the box, its spacing, its
 * corners and strokes, where it is put, its shadow and fill. A token reaching only a colour or a
 * duration isn't a measure.
 */
const PROPERTIES = [
  "width",
  "height",
  "min-width",
  "min-height",
  "max-width",
  "max-height",
  "flex-basis",
  "aspect-ratio",
  "grid-template-columns",
  "grid-template-rows",
  "grid-auto-rows",
  "grid-auto-columns",
  ...SIDES.map((side) => `padding-${side}`),
  ...SIDES.map((side) => `margin-${side}`),
  "row-gap",
  "column-gap",
  ...SIDES,
  "translate",
  "transform",
  ...SIDES.map((side) => `border-${side}-width`),
  "border-top-left-radius",
  "border-top-right-radius",
  "border-bottom-right-radius",
  "border-bottom-left-radius",
  "outline-width",
  "outline-offset",
  "box-shadow",
  "opacity",
  "background-color",
  "background-size",
  "mask-size",
] as const;

/** The type's properties, which children inherit: an element takes the token only where its parent's differs. */
const INHERITED = ["font-size", "line-height", "letter-spacing"] as const;

const READ: readonly string[] = [...PROPERTIES, ...INHERITED];

/** The tokens script worked a box out from, which its styles don't show (`lib/probe.ts`). */
const WORKED_OUT = "tokens";

/**
 * A value `token` never has, of its own type: an aspect ratio stays a ratio, a number a number, a
 * length a length; or null for a token of no such type (a colour, a duration, a word).
 */
function sentinel(token: string): string | null {
  const value = raw(token);
  if (value === "") return null;
  if (value.includes("/")) return "1 / 7.31";
  // Of the plain numbers, only an opacity sets a measure: a weight or a speed doesn't.
  if (/^-?[\d.]+$/.test(value)) return token.startsWith("opacity-") ? String(Number(value) + 0.37) : null;
  if (/px|calc|%/.test(value)) return `calc(${value} + 1013.37px)`;
  return null;
}

/**
 * What the properties a token changed on an element make of it, a use for each kind: its padding
 * on the sides it took it, its gaps, its margin, and its box for the rest (a size, a corner, a
 * shadow). A spacer's size is the gap it is.
 */
function usesOf(token: string, changed: readonly string[], spacer: boolean): Use[] {
  if (spacer) return [{ token, kind: "gap", sides: [], properties: changed }];
  const sidesOf = (prefix: string) => SIDES.filter((side) => changed.includes(`${prefix}-${side}`));
  const gaps: string[] = changed.filter((property) => property === "row-gap" || property === "column-gap");
  const rest = changed.filter((property) => !/^(padding|margin)-/.test(property) && !gaps.includes(property));
  const out: Use[] = [];
  const padding = sidesOf("padding");
  if (padding.length > 0) out.push({ token, kind: "padding", sides: padding, properties: padding.map((side) => `padding-${side}`) });
  if (gaps.length > 0) out.push({ token, kind: "gap", sides: [], properties: gaps });
  const margin = sidesOf("margin");
  if (margin.length > 0) out.push({ token, kind: "margin", sides: margin, properties: margin.map((side) => `margin-${side}`) });
  if (rest.length > 0) out.push({ token, kind: "box", sides: [], properties: rest });
  return out;
}

function isSpacer(element: Element): boolean {
  const probe = (element as HTMLElement).dataset.probe;
  return probe !== undefined && probe.includes('"role":"spacer"');
}

let probeSheet: CSSStyleSheet | null = null;

/** The sheet that sets one token on the clone, added to the document once. */
function sheet(): CSSStyleSheet {
  if (probeSheet) return probeSheet;
  probeSheet = new CSSStyleSheet();
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, probeSheet];
  return probeSheet;
}

/** Where one sample uses its tokens, read a token or an element at a time, as it is asked. */
export interface Reader {
  /** Every element taking `token`, with how. */
  token(token: string): Uses;
  /** The tokens `element` takes of those `tokens` gives, asked once, with how. */
  element(element: Element, tokens: () => readonly string[]): Use[];
  /** Forgets what was read: the sample changed. */
  forget(): void;
}

/**
 * Reads where `sample` uses its tokens, by the cascade, and each box the lib says script worked
 * out from one of them. `sample` must be in the document, where its selectors match; its clone
 * goes in beside it for each question and out again before anything is drawn. A token's places
 * are read once per token and an element's tokens once per element, so pointing at one never
 * waits on the rest (every token of a template at once took a second).
 */
export function reader(sample: Element): Reader {
  let state: Clone | null = null;
  const byToken = new Map<string, Uses>();
  const byElement = new Map<Element, Use[]>();

  const token = (name: string): Uses => {
    const known = byToken.get(name);
    if (known) return known;
    const uses: Uses = new Map();
    const add = (element: Element, use: Use) => uses.set(element, [...(uses.get(element) ?? []), use]);
    const clone = (state ??= cloneOf(sample));
    for (const element of clone.originals) {
      if (workedOut(element).includes(name)) add(element, { token: name, kind: "box", sides: [], properties: [] });
    }
    clone.probe(name, null, (i, changed) => {
      for (const use of usesOf(name, changed, isSpacer(clone.originals[i]!))) add(clone.originals[i]!, use);
    });
    byToken.set(name, uses);
    return uses;
  };

  const element = (part: Element, candidates: () => readonly string[]): Use[] => {
    const known = byElement.get(part);
    if (known) return known;
    const tokens = candidates();
    const clone = (state ??= cloneOf(sample));
    const i = clone.originals.indexOf(part);
    const uses: Use[] = workedOut(part)
      .filter((name) => tokens.includes(name))
      .map((name) => ({ token: name, kind: "box", sides: [], properties: [] }));
    if (i >= 0) {
      for (const name of tokens) {
        clone.probe(name, [i], (_, changed) => uses.push(...usesOf(name, changed, isSpacer(part))));
      }
    }
    byElement.set(part, uses);
    return uses;
  };

  return {
    token,
    element,
    forget: () => {
      state = null;
      byToken.clear();
      byElement.clear();
    },
  };
}

function workedOut(element: Element): string[] {
  return (element as HTMLElement).dataset?.[WORKED_OUT]?.split(" ") ?? [];
}

/** A sample's clone, laid out to read: its elements beside the originals, and their styles. */
interface Clone {
  readonly originals: readonly Element[];
  /**
   * Sets `token` to a value it never has on the clone and tells `found` each element, of `only` or
   * of all, whose properties then changed, with those properties.
   */
  probe(token: string, only: readonly number[] | null, found: (index: number, changed: string[]) => void): void;
}

function cloneOf(sample: Element): Clone {
  const originals = [sample, ...sample.querySelectorAll("*")];
  const copy = sample.cloneNode(true) as Element;
  const box = document.createElement("span");
  box.className = "page-uses";
  box.append(copy);
  const copies = [copy, ...copy.querySelectorAll("*")];
  const index = new Map(copies.map((each, i) => [each, i]));
  const parents = copies.map((each) => (each.parentElement ? (index.get(each.parentElement) ?? -1) : -1));
  const styles = copies.map((each) => getComputedStyle(each));
  const before = new Map<number, string[]>();
  const read = (i: number) => READ.map((property) => styles[i]!.getPropertyValue(property));
  const probe = (token: string, only: readonly number[] | null, found: (index: number, changed: string[]) => void) => {
    const value = sentinel(token);
    if (!value) return;
    const asked = only ?? copies.map((_, i) => i);
    // Each element's parent too: an inherited value is the element's own use only where its parent's differs.
    const wanted = [...new Set(asked.flatMap((i) => (parents[i]! >= 0 ? [i, parents[i]!] : [i])))];
    sample.after(box);
    try {
      for (const i of wanted) if (!before.has(i)) before.set(i, read(i));
      sheet().replaceSync(`.page-uses, .page-uses * { --${token}: ${value} !important; }`);
      const after = new Map(wanted.map((i) => [i, read(i)]));
      for (const i of asked) {
        const values = after.get(i)!;
        const was = before.get(i)!;
        const parent = parents[i]!;
        const changed = READ.filter((_, p) => {
          if (values[p] === was[p]) return false;
          if (p >= PROPERTIES.length && parent >= 0) return values[p] !== after.get(parent)![p];
          return true;
        });
        if (changed.length > 0) found(i, changed);
      }
    } finally {
      sheet().replaceSync("");
      box.remove();
    }
  };
  return { originals, probe };
}

/**
 * Every token that could reach `element`'s properties: named in its own inline style or a rule
 * matching it, or in its ancestors' up to `sample`, whose custom properties it inherits
 * (`--gap: var(--space-s12)` on a row, read by its `gap`). `--px` and `--scale`, which every
 * length is drawn in, aren't measures.
 */
export function tokensFor(element: Element, sample: Element): string[] {
  const found = new Set<string>();
  const scan = (text: string) => {
    for (const match of text.matchAll(/var\(--([a-z0-9_-]+)/g)) found.add(match[1]!);
  };
  const chain: Element[] = [];
  for (let at: Element | null = element; at; at = at === sample ? null : at.parentElement) chain.push(at);
  for (const each of chain) scan(each.getAttribute("style") ?? "");
  for (const rule of styleRules()) {
    // A pseudo-element's rule is its element's: `::before` is drawn from the box it is on.
    const selector = rule.selectorText.replace(/::?(before|after|placeholder|marker|selection)\b/g, "");
    try {
      if (chain.some((each) => each.matches(selector))) scan(rule.style.cssText);
    } catch {
      // A selector the engine can't match on an element (a vendor one) names none of its tokens.
    }
  }
  found.delete("px");
  found.delete("scale");
  return [...found].filter((token) => sentinel(token) !== null);
}

/** Every style rule of the page's sheets that names a custom property, inside media and layers too. */
function styleRules(): CSSStyleRule[] {
  const list: CSSStyleRule[] = [];
  const walk = (group: CSSRuleList) => {
    for (const rule of group) {
      if (rule instanceof CSSStyleRule && rule.style.cssText.includes("var(--")) list.push(rule);
      if ("cssRules" in rule) walk((rule as CSSGroupingRule).cssRules);
    }
  };
  for (const each of document.styleSheets) walk(each.cssRules);
  return list;
}
