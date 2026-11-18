// A Settings section's own page, as every section after Appearance and Power has one: its live window,
// seeded with a few changed settings; its sub-sections scrolled to; each sub-page opened; what isn't
// built yet, landed on; the narrow window; and the tables every section shares. A section adds what
// is its own: its special parts' states, its error and empty states, its moves.

import { doesntApply, type Move, nonEmpty, page, screen, type Variant } from "../../../site/page.ts";
import { deeperMoves } from "../../templates/sidebar-turns.ts";
import { defaults, sectionData, type Values } from "./settings-data.ts";
import { settingsOf } from "./settings-parts.ts";
import { landedOn, liveWindow, MINIMUM, settingsWindow } from "./settings-window.ts";

type Shown = "rest" | "selected";

export interface SectionPageSpec {
  /** The section's place in the sidebar (`settings-data.ts`). */
  readonly index: number;
  readonly slug: string;
  /** The page's title: "Settings: Displays". */
  readonly title: string;
  /** One sentence: what the section holds. */
  readonly lede: string;
  /** How to try the live window, a few words. */
  readonly hint: string;
  /** The values the live window and the changed samples start with: a few settings off their defaults. */
  readonly seed: Partial<Values>;
  /** The section's own samples: its special parts' states, its error and empty states. */
  readonly variants: readonly Variant[];
  /** The section's own moves, beside the sideways turn into its sub-pages. */
  readonly moves?: readonly Move[];
  /** The section's own keys, beside every section's. */
  readonly keys?: readonly { readonly keys: readonly [string, ...string[]]; readonly does: string }[];
  /** The library's parts the section's own parts are built from, beside every section's. */
  readonly from?: readonly string[];
  /** The section's own measures, beside every section's. */
  readonly measures?: readonly { readonly part: string; readonly token: string }[];
}

/** The window on `index`'s section with `values`, still, at the default size or `size`. */
const at = (index: number, values: Partial<Values> = {}, more: Parameters<typeof settingsWindow>[0] = {}) => () => settingsWindow({ section: index, values, ...more });

export function sectionPage(spec: SectionPageSpec): ReturnType<typeof page> {
  const data = sectionData(defaults(), spec.index);
  const name = data.title;
  const subs: Variant[] = data.subs.map((sub, index) => ({ name: sub.title, for: sub.body ?? `${name}'s ${sub.title.toLowerCase()}`, sample: screen(at(spec.index, {}, { sub: index })) }));
  const pages: Variant[] = (data.pages ?? []).map((sub) => ({ name: `${sub.title}, opened`, for: `a sub-page; Back to ${sub.back}`, sample: screen(at(spec.index, { opened: sub.id })) }));
  // What isn't built: a row on a sub-page shows with its page open, a row its neighbour shows with the
  // section at its top, the rest landed on.
  const unbuilt: Variant[] = settingsOf(data)
    .filter((each) => each.unbuilt)
    .map((each) => {
      const page = data.pages?.find((sub) => sub.wells.flat().includes(each));
      const shown = each.shows ? each.shows() : true;
      const draw = page ? at(spec.index, { opened: page.id }) : shown ? () => settingsWindow(landedOn(spec.index, each.id)) : at(spec.index);
      return { name: `Not built: ${each.title}`, for: shown || page ? "no backend writes it yet" : "no backend yet; shown by the row above", sample: screen(draw) };
    });
  const rest = at(spec.index);
  const changed = at(spec.index, spec.seed);
  return page<Shown>({
    level: "apps",
    slug: spec.slug,
    title: spec.title,
    lede: spec.lede,
    code: { unbuilt: "kubaos-settings: not built yet" },
    from: nonEmpty(["sidebar-page", "list", "tabs", "slider", "select", "field", "switch", "button", "icon-button", "banner", "keycap", ...(spec.from ?? [])]),
    stage: { live: () => liveWindow({ section: spec.index, values: spec.seed }), hint: spec.hint, daylight: true },
    variants: nonEmpty([...subs, ...pages, ...spec.variants, ...unbuilt]),
    sizes: [
      { name: "Minimum", for: "720 × 480: a name too long stacks its control", sample: screen(() => settingsWindow({ section: spec.index, values: spec.seed }, MINIMUM)) },
      { name: "Default", for: "1120 × 720", sample: screen(rest) },
    ],
    states: {
      shown: ["rest", "selected"],
      absent: {
        lit: "a row lights as `ListRow` does",
        pressed: "a row presses as `ListRow` does",
        disabled: "a fact row says why it can't change",
        loading: "every value is read as it opens",
        empty: "every sub-section has its settings",
        error: "a field says its own, under its name",
      },
      rows: [{ name: data.title, cell: screen((state: Shown) => (state === "rest" ? rest() : changed())) }],
      looks: {
        rest: "at its defaults: no Reset anywhere",
        selected: "a few changed: their resets and Reset",
      },
    },
    focus: doesntApply("no line of its own: its rows and controls light their own."),
    content: {
      text: doesntApply("its words are fixed; names and lines clamp, or stack when narrow."),
      missingPicture: doesntApply("rows carry no picture."),
      counts: [
        { name: "At its defaults", for: "no reset anywhere", sample: screen(rest) },
        { name: "A few changed", for: "each changed row's reset, its heading's and the title's Reset", sample: screen(changed) },
      ],
    },
    motion: {
      arrives: doesntApply("it comes with its section's page turn, as every section does."),
      leaves: doesntApply("it goes with its section."),
      between: data.pages?.length || spec.moves?.length ? nonEmpty([...(spec.moves ?? []), ...(data.pages?.length ? deeperMoves(at(spec.index, { opened: data.pages[0]!.id })) : [])]) : doesntApply("its rows and controls move on their own pages."),
    },
    keyboard: {
      keys: nonEmpty([
        { keys: ["↑", "↓"], does: "the setting above or below" },
        { keys: ["←", "→"], does: "a choice's or a slider's value" },
        { keys: ["Enter", "Space"], does: "a switch flips; a select opens; a row opens its page" },
        { keys: ["Tab"], does: "on a changed row: its reset, before the control" },
        { keys: ["Enter", "Space"], does: "held on a reset or Reset: puts it back once full" },
        { keys: ["Esc"], does: "a sub-page: back to where it opened" },
        ...(spec.keys ?? []),
      ]),
      order: "The title's Reset, then each heading's and its wells; a changed row's reset.",
      lands: "On its first row from the sidebar; on the setting from search.",
    },
    measures: {
      samples: nonEmpty([{ name: data.subs[0]?.title ?? name, for: "its rows and their controls", sample: rest }]),
      measures: nonEmpty([
        { part: "A setting's row", token: "size-row_two_line" },
        { part: "The control column", token: "size-menu" },
        { part: "A row's reset; a heading's Reset", token: "size-control_sm" },
        { part: "A stacked row's lines", token: "space-s8" },
        ...(spec.measures ?? []),
      ]),
      reflow: doesntApply("the window's own; Settings shows it."),
    },
  });
}
