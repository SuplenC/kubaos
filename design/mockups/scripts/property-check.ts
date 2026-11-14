// Holds the site's registered custom properties to one meaning each. An `@property` is global: the
// page takes one registration per name, the last, whatever file and component it is written for,
// and every other use of the name is held to it. A value its syntax refuses is invalid at
// computed-value time and silently becomes the initial value.
//
//   node scripts/property-check.ts
//
// It fails when a name is registered twice with a different `syntax`, `inherits` or
// `initial-value`, and when a number property (`<number>`, `<integer>`) is declared with a
// colour, naming each place. It reads every CSS file under `src/`, and `CSS.registerProperty`
// calls in its TypeScript.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/** One registration of a name: where it is and what it says. */
interface Registration {
  readonly where: string;
  readonly syntax: string;
  readonly inherits: string;
  readonly initial: string;
}

/** A value that is a colour whatever it reads, by the function or literal it starts with. */
const COLOUR = /^(?:#|rgba?\(|hsla?\(|hwb\(|lab\(|lch\(|oklab\(|oklch\(|color\(|color-mix\(|light-dark\(|var\(--color-)/;

const NUMBERS = new Set(['"<number>"', '"<integer>"']);

const src = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "src");

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    return entry.isDirectory() ? files(full) : [full];
  });
}

function lineOf(text: string, index: number): number {
  return text.slice(0, index).split("\n").length;
}

/** The value of `field` in a registration's body, quotes normalised to double. */
function field(body: string, name: string): string {
  const value = new RegExp(`${name}\\s*:\\s*([^;,}]+)`).exec(body)?.[1]?.trim() ?? "";
  return value.replace(/^'(.*)'$/, '"$1"');
}

function registrations(file: string, text: string): [string, Registration][] {
  const relative = path.relative(src, file);
  const pattern = file.endsWith(".css")
    ? /@property\s+(--[\w-]+)\s*\{([^}]*)\}/g
    : /CSS\.registerProperty\(\s*\{[^}]*?name\s*:\s*["'](--[\w-]+)["']([^}]*)\}/g;
  return [...text.matchAll(pattern)].map((match) => {
    const body = match[2] ?? "";
    return [
      match[1] ?? "",
      {
        where: `${relative}:${lineOf(text, match.index)}`,
        syntax: field(body, "syntax"),
        inherits: field(body, "inherits"),
        initial: field(body, file.endsWith(".css") ? "initial-value" : "initialValue"),
      },
    ];
  });
}

function main(): void {
  const sources = files(src).filter((file) => /\.(css|ts)$/.test(file));
  const texts = new Map(sources.map((file) => [file, readFileSync(file, "utf8")]));
  const registered = new Map<string, Registration[]>();
  for (const [file, text] of texts) {
    for (const [name, registration] of registrations(file, text)) {
      registered.set(name, [...(registered.get(name) ?? []), registration]);
    }
  }

  const failures: string[] = [];
  for (const [name, all] of registered) {
    const [first, ...rest] = all;
    if (!first) continue;
    const differ = rest.some((r) => r.syntax !== first.syntax || r.inherits !== first.inherits || r.initial !== first.initial);
    if (differ) {
      const each = all.map((r) => `  ${r.where}: syntax ${r.syntax}, inherits ${r.inherits}, initial ${r.initial}`);
      failures.push(`${name} is registered differently:\n${each.join("\n")}`);
    }
  }

  for (const [file, text] of texts) {
    if (!file.endsWith(".css")) continue;
    for (const match of text.matchAll(/(?<![\w-])(--[\w-]+)\s*:\s*([^;{}]+);/g)) {
      const name = match[1] ?? "";
      const syntax = registered.get(name)?.[0]?.syntax;
      if (syntax && NUMBERS.has(syntax) && COLOUR.test((match[2] ?? "").trim())) {
        failures.push(`${path.relative(src, file)}:${lineOf(text, match.index)}: ${name} is registered as ${syntax} but given a colour`);
      }
    }
  }

  if (failures.length > 0) {
    process.stderr.write(`property-check: ${failures.join("\n")}\n`);
    process.exit(1);
  }
  process.stdout.write(`property-check: ${registered.size} registered properties, each registered one way\n`);
}

main();
