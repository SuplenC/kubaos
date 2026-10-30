// Token values read back from the tokens' CSS, for the few things worked out in script rather than
// in CSS: a length in logical pixels, measured on the root at scale 1, a plain number, the tokens
// of a family by name, and what the tokens say in words.

const lengths = new Map<string, number>();
const families = new Map<string, string[]>();
const numbers = new Map<string, number>();
const curves = new Map<string, number[]>();

/** The token `--name`'s length in logical pixels. */
export function length(name: string): number {
  const known = lengths.get(name);
  if (known !== undefined) return known;
  if (raw(name) === "") throw new Error(`no token --${name}`);
  const meter = document.createElement("div");
  meter.style.cssText = `position:absolute;visibility:hidden;width:var(--${name})`;
  document.documentElement.append(meter);
  const value = meter.getBoundingClientRect().width;
  meter.remove();
  lengths.set(name, value);
  return value;
}

/** Forgets every length and number read: the tokens changed. */
export function forgetLengths(): void {
  families.clear();
  lengths.clear();
  numbers.clear();
  curves.clear();
}

/** The token `--name`'s cubic Bézier, its four numbers, read once: a ball is eased along it each
 *  frame. */
export function cubic(name: string): number[] {
  const known = curves.get(name);
  if (known !== undefined) return known;
  const inside = raw(name).replace(/^cubic-bezier\((.*)\)$/, "$1");
  const value = inside.split(",").map((n) => Number.parseFloat(n));
  if (value.length !== 4 || value.some(Number.isNaN)) throw new Error(`--${name} is no cubic-bezier()`);
  curves.set(name, value);
  return value;
}

/** The token `--name`'s number, as written: an opacity, a step of the strike. */
export function number(name: string): number {
  const known = numbers.get(name);
  if (known !== undefined) return known;
  const value = Number.parseFloat(getComputedStyle(document.documentElement).getPropertyValue(`--${name}`));
  if (Number.isNaN(value)) throw new Error(`--${name} is no number`);
  numbers.set(name, value);
  return value;
}

/** The token `--name` as written: a duration's `100ms`, a curve's `cubic-bezier(…)`. */
export function raw(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(`--${name}`).trim();
}

/**
 * Every token whose custom property starts `--prefix`, without the dashes, in the order the
 * tokens' CSS declares them, each once: what a foundation lists, so a token added to the tokens
 * shows on its page without the page changing.
 */
export function names(prefix: string): string[] {
  const known = families.get(prefix);
  if (known) return known;
  const found = new Set<string>();
  for (const sheet of document.styleSheets) {
    for (const rule of sheet.cssRules) {
      if (!(rule instanceof CSSStyleRule)) continue;
      for (const property of rule.style) {
        if (property.startsWith(`--${prefix}`)) found.add(property.slice(2));
      }
    }
  }
  const list = [...found];
  families.set(prefix, list);
  return list;
}

/**
 * The token `--name` as the tokens' CSS declares it on the root, before any `var()` in it is
 * filled in: a derived colour's formula.
 */
export function declared(name: string): string {
  for (const sheet of document.styleSheets) {
    for (const rule of sheet.cssRules) {
      if (!(rule instanceof CSSStyleRule) || !rule.selectorText.startsWith(":root")) continue;
      const value = rule.style.getPropertyValue(`--${name}`).trim();
      if (value) return value;
    }
  }
  return "";
}

/** The token `--name`'s string, as the tokens' CSS writes what they say in words. */
export function words(name: string): string {
  const value = raw(name);
  if (!value.startsWith('"')) throw new Error(`--${name} is no string`);
  return JSON.parse(value) as string;
}
