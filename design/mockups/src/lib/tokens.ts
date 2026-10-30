// Token values read back from the tokens' CSS, for the few things worked out in script rather than
// in CSS: a length in logical pixels, measured on the root at scale 1, and a plain number.

const lengths = new Map<string, number>();
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
