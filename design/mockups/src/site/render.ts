// Draws a page: its header (the crumb, the title, the lede and the names in code), the stage with
// the thing itself live, then every part as tables and samples, in the mockup skill's order, or a
// foundation's parts in theirs. Each section is listed in the right rail; a part that doesn't apply
// is one muted line in its place, not a section. Every page is checked for its words first
// (`wordy`): a page with a field past its kind's most isn't drawn.

import { type AnyPage, applies, type DoesntApply, type Foundation, isFoundation, LEVELS, type Page, STATE_NAMES, type State, wordy } from "./page.ts";
import { el, label, prose } from "./dom.ts";
import { typeset } from "../lib/text.ts";
import { content } from "./parts/content.ts";
import { checks, shownAll } from "./parts/shown.ts";
import { keyboard } from "./parts/keyboard.ts";
import { matrix } from "./parts/matrix.ts";
import { measures } from "./parts/measures.ts";
import { motion } from "./parts/motion.ts";
import { notApplying, section, subheading } from "./parts/section.ts";
import { shelves } from "./parts/shelf.ts";
import { stage } from "./parts/stage.ts";
import { pair, specTable, titled } from "./parts/table.ts";
import { tubes } from "./parts/tubes.ts";
import { variants } from "./parts/variants.ts";
import type { Section } from "./rail.ts";
import { PAGES } from "../pages/index.ts";

/** A page drawn: its container, and its sections for the rail. */
export interface Drawn {
  readonly container: HTMLElement;
  readonly sections: readonly Section[];
}

/** A part of the page: a section the rail lists, or a muted line for a part that doesn't apply. */
type Part = Section | Element;

export function render(page: AnyPage): Drawn {
  const found = wordy(page);
  if (found.length > 0) {
    const list = found.map((each) => `${each.where} (${each.kind}): "${each.words}"`).join("; ");
    throw new Error(`${page.slug} has words past their most: ${list}`);
  }
  const overview: Section = { title: "Overview", element: el("div", "page-overview", header(page), hero(page)) };
  const parts = [overview, ...(isFoundation(page) ? renderFoundation(page) : renderComponent(page))];
  const sections = parts.filter((part): part is Section => !(part instanceof Element));
  const container = el("div", "page-container", ...parts.map((part) => (part instanceof Element ? part : part.element)));
  return { container, sections };
}

function header(page: AnyPage): HTMLElement {
  const level = LEVELS.find((each) => each.slug === page.level)?.title ?? page.level;
  const tags = isFoundation(page) ? [] : codeTags(page);
  return el(
    "header",
    "page-header",
    label(`${level} › ${page.title}`, "small", "on_surface_variant"),
    el("h1", "page-title", label(page.title, "display", "on_surface")),
    prose(page.lede, "body_large", "on_surface_variant"),
    ...(!isFoundation(page) && page.from ? [builtFrom(page.from)] : []),
    ...(tags.length > 0 ? [el("div", "page-tags", ...tags)] : []),
  );
}

/** "Built from Pill.": each part the page is built from, a link to its page. */
function builtFrom(slugs: readonly string[]): HTMLElement {
  const links = slugs.map((slug) => {
    const found = PAGES.find((each) => each.slug === slug);
    if (!found) throw new Error(`built from ${slug}, which has no page`);
    const link = el("a", "page-from-link", found.title);
    link.setAttribute("href", `#/${found.level}/${found.slug}`);
    return link;
  });
  const words: (Node | string)[] = ["Built from "];
  links.forEach((link, index) => words.push(...(index === 0 ? [] : [index === links.length - 1 ? " and " : ", "]), link));
  words.push(".");
  const line = el("p", "page-prose page-from", ...words);
  typeset(line, "body");
  line.style.color = "var(--color-on_surface_variant)";
  return line;
}

/** The names in code its component page gives, and what isn't built yet. */
function codeTags(page: Page): HTMLElement[] {
  const tag = (words: string, warn = false) => {
    const box = el("span", "page-code-tag", label(words, "mono"));
    box.toggleAttribute("data-warn", warn);
    return box;
  };
  const { rust, qml, unbuilt } = page.code;
  return [...(rust ? [tag(`Rust ${rust}`)] : []), ...(qml ? [tag(`QML ${qml}`)] : []), ...(unbuilt ? [tag(unbuilt, true)] : [])];
}

function hero(page: AnyPage): HTMLElement {
  return isFoundation(page) ? stage(page.stage) : stage(page.stage, { states: page.states, focus: page.focus });
}

/** A section under `title`. */
function part(title: string, ...body: Element[]): Section {
  return { title, element: section(title, ...body) };
}

/** A section under `title` drawn by `draw`, or a muted line in its place. */
function partOr<T>(title: string, value: T | DoesntApply, draw: (value: T) => Element[]): Part {
  return applies(value) ? part(title, ...draw(value)) : notApplying(value.doesntApply, title);
}

function renderFoundation(page: Foundation): Part[] {
  const shown = (parts: Parameters<typeof shownAll>[0]) => shelves(shownAll(parts));
  return [
    part("Tokens", ...shown(page.tokens)),
    part("Rules", ...shown(page.rules)),
    partOr("Light and dark", page.modes, shown),
    partOr("Themes", page.themes, shown),
    partOr("At every scale", page.scales, shown),
    partOr("Content", page.content, shown),
    partOr("Motion", page.motion, (spec) => [motion(spec)]),
    part("Checks", checks(page.checks)),
  ];
}

/** The states table: each state shown and the few words of its look, then those it hasn't, muted. */
function statesTable(page: Page): HTMLElement {
  const { states } = page;
  const shown = states.shown.map((state) => ({ cells: [STATE_NAMES[state], states.looks[state]] }));
  const absent = (Object.entries(states.absent) as [State, string][]).map(([state, why]) => ({ cells: [STATE_NAMES[state], `doesn't apply: ${why}`], muted: true }));
  return specTable({ heads: ["State", "Look"] }, [...shown, ...absent]).element;
}

function renderComponent(page: Page): Part[] {
  const measured = measures(page.measures);
  const lit = applies(page.focus) ? [tubes(page.focus)] : [notApplying(page.focus.doesntApply, "The lit line")];
  return [
    part(
      "Anatomy",
      pair(titled("Parts", measured.table), titled("States", statesTable(page))),
      ...lit,
      subheading("Inspect"),
      measured.status,
      ...shelves(measured.samples),
    ),
    part("Variants", ...shelves(variants("", page.variants, "Variants"), variants("Sizes", page.sizes), ...(page.groups ?? []).map((group) => variants(group.title, group.rows)))),
    part("Motion", motion(page.motion)),
    partOr("Keyboard", page.keyboard, (keys) => [keyboard(keys)]),
    part("Content", ...shelves(...content(page.content))),
    part("Every state", ...shelves(matrix(page.states, page.focus))),
    partOr("Reflow", page.measures.reflow, () => shelves(measured.reflow)),
  ];
}
