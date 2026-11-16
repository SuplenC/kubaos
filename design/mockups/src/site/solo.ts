// Solo mode, `?solo=STATE#/LEVEL/PAGE`: one state of one page and nothing else, at its own size on
// the page's background, for a capture (`scripts/shot.ts`) or a quick look. Only that state is
// built, so a page of many screens loads in the time of one. The top bar's settings still come
// from the address (`mode`, `scale`, `theme`, `reduced`, `speed`, `flicker`, `text`, `window`);
// `at=WxH` lays a whole screen out at W by H logical pixels.
//
// A page's states are every sample it names, undrawn: the stage's chips (Live, then each state),
// then its variants, sizes, content, measured samples, reflow and moves, or a foundation's shown
// parts. Each is found by its name as the page shows it, or by its slug: the name's, or, where two
// share a name, the group's and the name's (`variant-go`, `content-go`). A move plays its loop from
// the start, as its player does. A missing or unknown state lists the page's states as links, and
// an unknown page every page.

import { type AnyPage, applies, type DoesntApply, isFoundation, type Move, type Motion, type Shown, type Variant } from "./page.ts";
import { el, label } from "./dom.ts";
import { allScreens, played, sizeScreen } from "./full-screen.ts";
import { textSamples } from "./parts/content.ts";
import { windowed } from "./parts/measures.ts";
import { stagePicks } from "./parts/stage.ts";
import { type Mode, settings, windowSize } from "./settings.ts";
import { PAGES } from "../pages/index.ts";
import { findState, slugOf, type StateName } from "./state-names.ts";

const QUERY = new URLSearchParams(location.search);

/** The state asked for: null outside solo mode, empty for the list of states. */
export const SOLO = QUERY.get("solo");

/**
 * A state a page can be drawn in alone, as scripts read it from `window.kubaos.states()`: its name
 * as the page shows it (`Go: no apps`), its slug, unique on the page (`go-no-apps`), and where the
 * page shows it (`stage`, `variant`, `size`, `content`, `measured`, `reflow`, `motion`).
 */
export type SoloState = StateName;

interface Entry extends Omit<SoloState, "slug"> {
  readonly draw: (mode: Mode) => Element[];
  readonly move?: Move;
}

function samples(group: string, rows: readonly Variant[] | DoesntApply): Entry[] {
  return applies(rows) ? rows.map((row) => ({ name: row.name, group, draw: () => [row.sample()] })) : [];
}

function shown(group: string, rows: readonly Shown[] | DoesntApply): Entry[] {
  return applies(rows) ? rows.map((row) => ({ name: row.name, group, draw: () => [row.sample()] })) : [];
}

function moves(spec: Motion | DoesntApply): Entry[] {
  if (!applies(spec)) return [];
  const all = [spec.arrives, spec.leaves, spec.between].flatMap((list) => (applies(list) ? list : []));
  return all.map((move) => ({ name: move.name, group: "motion", draw: () => [move.sample()], move }));
}

/** Every state of `page`, in the order the page shows them, none of them drawn. */
function entries(page: AnyPage): Entry[] {
  const stage = stagePicks(page.stage, isFoundation(page) ? undefined : { states: page.states, focus: page.focus }).map(
    (pick): Entry => ({ name: pick.name, group: "stage", draw: pick.draw }),
  );
  if (isFoundation(page)) {
    return [
      ...stage,
      ...shown("tokens", page.tokens),
      ...shown("rules", page.rules),
      ...shown("modes", page.modes),
      ...shown("themes", page.themes),
      ...shown("scales", page.scales),
      ...shown("content", page.content),
      ...moves(page.motion),
    ];
  }
  const text = applies(page.content.text) ? textSamples(page.content.text).map((each): Entry => ({ name: each.name, group: "content", draw: () => [each.make()] })) : [];
  const reflow = page.measures.reflow;
  return [
    ...stage,
    ...samples("variant", page.variants),
    ...samples("size", page.sizes),
    ...text,
    ...samples("content", page.content.missingPicture),
    ...samples("content", page.content.counts),
    ...samples("measured", page.measures.samples),
    ...(applies(reflow) ? [{ name: `In a window, ${windowSize().name}`, group: "reflow", draw: () => [windowed(reflow.sample())] }] : []),
    ...moves(page.motion),
  ];
}

/**
 * Each entry with its slug: its name's, or where names collide, its group's and its name's, so a
 * slug doesn't change with the order of the page's samples; a numeral after any left alike.
 */
function slugged(list: readonly Entry[]): (Entry & { readonly slug: string })[] {
  const count = (slugs: readonly string[]) => slugs.reduce((seen, slug) => seen.set(slug, (seen.get(slug) ?? 0) + 1), new Map<string, number>());
  const plain = list.map((entry) => slugOf(entry.name));
  const plainCount = count(plain);
  const grouped = list.map((entry, i) => (plainCount.get(plain[i]!)! > 1 ? `${entry.group}-${plain[i]!}` : plain[i]!));
  const groupedCount = count(grouped);
  const taken = new Map<string, number>();
  return list.map((entry, i) => {
    const slug = grouped[i]!;
    if (groupedCount.get(slug)! === 1) return { ...entry, slug };
    const nth = (taken.get(slug) ?? 0) + 1;
    taken.set(slug, nth);
    return { ...entry, slug: nth === 1 ? slug : `${slug}-${nth}` };
  });
}

/** `page`'s states, as scripts read them. */
export function soloStates(page: AnyPage | undefined): SoloState[] {
  return page ? slugged(entries(page)).map(({ name, slug, group }) => ({ name, slug, group })) : [];
}

/** The page the address names, exactly: solo mode draws no other page in its place. */
export function soloPage(): AnyPage | undefined {
  const [, level, slug] = location.hash.split("/");
  return PAGES.find((page) => page.level === level && page.slug === slug);
}

/** The address of `slug` on `page`, keeping every other setting. */
function address(page: AnyPage, slug: string): string {
  const query = new URLSearchParams(location.search);
  query.set("solo", slug);
  return `?${query.toString()}#/${page.level}/${page.slug}`;
}

function list(heading: string, links: readonly { readonly words: string; readonly slug: string; readonly href: string }[]): HTMLElement {
  const rows = links.map((link) => {
    const anchor = el("a", "page-from-link", label(link.words, "body", "primary"));
    anchor.setAttribute("href", link.href);
    return el("li", "solo-row", anchor, label(link.slug, "mono", "on_surface_variant"));
  });
  return el("div", "solo-list", label(heading, "title", "on_surface"), el("ul", "solo-rows", ...rows));
}

/** What solo mode drew in `box`: the state shown, or null for a list. */
export interface SoloDrawn {
  readonly shown: SoloState | null;
}

/** Draws the state `SOLO` names of the page the address names into `box`, or the list to pick from. */
export function drawSolo(box: HTMLElement): SoloDrawn {
  box.dataset.solo = "";
  box.removeAttribute("data-screen");
  const page = soloPage();
  if (!page) {
    box.replaceChildren(list("Pick a page", PAGES.map((each) => ({ words: each.title, slug: `${each.level}/${each.slug}`, href: address(each, "") }))));
    return { shown: null };
  }
  const states = slugged(entries(page));
  const wanted = SOLO ?? "";
  const found = findState(states, wanted);
  if (!wanted || !found) {
    const heading = wanted ? `${page.title} has no state "${wanted}"` : `${page.title}: pick a state`;
    box.replaceChildren(list(heading, states.map((state) => ({ words: `${state.group} · ${state.name}`, slug: state.slug, href: address(page, state.slug) }))));
    return { shown: null };
  }
  const drawn = found.draw(settings().mode);
  box.replaceChildren(el("div", "page-stage-flow solo-flow", ...drawn));
  const at = /^(\d+)x(\d+)$/.exec(QUERY.get("at") ?? "");
  if (at) {
    for (const each of drawn) sizeScreen(each, `calc(${at[1]} * var(--px))`, `calc(${at[2]} * var(--px))`);
  }
  box.toggleAttribute("data-screen", allScreens(drawn));
  box.dataset.solo = found.slug;
  if (found.move && drawn[0]) played(found.move, drawn[0], 0, 1);
  return { shown: { name: found.name, slug: found.slug, group: found.group } };
}
