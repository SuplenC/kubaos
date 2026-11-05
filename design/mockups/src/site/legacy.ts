// The page schema before pages were cut to short words: a summary, paragraphs of description and
// a sentence or more in every cell. A page still written that way is drawn through `page` and
// `foundation` here, which clamp each field to its first sentence and to its kind's most words
// (`MOST`), so it draws in the same shell as a page written short. A page is rewritten short by
// moving it to `site/page.ts`'s `page` or `foundation`.

import {
  type AnyPage,
  applies,
  type Check,
  type Content,
  type DoesntApply,
  doesntApply,
  type Focus,
  type Foundation,
  type Keyboard,
  type Motion,
  MOST,
  type Move,
  type NonEmpty,
  nonEmpty,
  type Page,
  page as shortPage,
  type Sample,
  type Shown,
  type State,
  type States,
  type Variant,
  type WordKind,
  wordCount,
} from "./page.ts";

/** A page's words as they were: paragraphs and sentences. */
interface Wordy<S extends State> {
  readonly summary: string;
  readonly live: Sample;
  readonly description: NonEmpty<string>;
  readonly states: Omit<States<S>, "looks"> & { readonly described: Record<S, string> };
  readonly measures: {
    readonly samples: NonEmpty<Variant>;
    readonly measures: NonEmpty<{ readonly what: string; readonly token: string }>;
    readonly reflow: { readonly described: string; readonly sample: Sample } | DoesntApply;
  };
}

export type LegacyPage<S extends State = State> = Omit<Page<S>, "lede" | "code" | "stage" | "states" | "measures"> & Wordy<S>;

interface LegacyShown {
  readonly name: string;
  readonly described: string;
  readonly sample: Sample;
}

type Shelf = NonEmpty<LegacyShown> | DoesntApply;

export interface LegacyFoundation extends Omit<Foundation, "lede" | "stage" | "tokens" | "rules" | "modes" | "themes" | "scales" | "content"> {
  readonly summary: string;
  readonly live: Sample;
  readonly description: NonEmpty<string>;
  readonly tokens: NonEmpty<LegacyShown>;
  readonly rules: NonEmpty<LegacyShown>;
  readonly modes: Shelf;
  readonly themes: Shelf;
  readonly scales: Shelf;
  readonly content: Shelf;
}

/** `words` cut to its first sentence, then to `kind`'s most words, an ellipsis where it was cut. */
export function clamp(words: string, kind: WordKind): string {
  const sentence = words.split(/(?<=[.!?])\s+(?=[A-Z`(])/)[0] ?? words;
  if (wordCount(sentence) <= MOST[kind]) return sentence;
  const kept: string[] = [];
  for (const word of sentence.split(/\s+/)) {
    if (wordCount([...kept, word].join(" ")) > MOST[kind]) break;
    kept.push(word);
  }
  let cut = kept.join(" ");
  // A token cut in half would set the rest of the line as code.
  if ((cut.match(/`/g) ?? []).length % 2 === 1) cut = cut.slice(0, cut.lastIndexOf("`")).trimEnd();
  return `${cut.replace(/[,;:.]$/, "")}…`;
}

const cell = (words: string) => clamp(words, "cell");

/**
 * `list` with each name clamped to a label, numbered where two clamp alike: a name is also where a
 * stage's chip finds its sample.
 */
function named<T extends { readonly name: string }>(list: readonly T[]): T[] {
  const seen = new Map<string, number>();
  return list.map((item) => {
    const name = clamp(item.name, "label");
    const count = (seen.get(name) ?? 0) + 1;
    seen.set(name, count);
    return { ...item, name: count === 1 ? name : `${name} ${count}` };
  });
}
const note = (words: string) => clamp(words, "note");

function why<T>(part: T | DoesntApply, keep: (value: T) => T): T | DoesntApply {
  return applies(part) ? keep(part) : doesntApply(note(part.doesntApply));
}

function variants(list: readonly Variant[] | DoesntApply): readonly Variant[] | DoesntApply {
  return why(list, (rows) => named(rows).map((row) => ({ ...row, for: cell(row.for) })));
}

function moves(list: readonly Move[] | DoesntApply): readonly Move[] | DoesntApply {
  return why(list, (rows) => named(rows).map((move) => ({ ...move, what: note(move.what) })));
}

function motion(spec: Motion): Motion {
  return { arrives: moves(spec.arrives), leaves: moves(spec.leaves), between: moves(spec.between) };
}

function record<K extends string>(words: Record<K, string>): Record<K, string> {
  return Object.fromEntries(Object.entries<string>(words).map(([key, value]) => [key, cell(value)])) as Record<K, string>;
}

/** The names in code the first paragraph gives: "Rust `select::select`, QML `Select`". */
function code(first: string): Page["code"] {
  const rust = /Rust `([^`]+)`/.exec(first)?.[1];
  const qml = /QML `([^`]+)`/.exec(first)?.[1];
  return { ...(rust ? { rust } : {}), ...(qml ? { qml } : {}) };
}

const HINT = "try it";

/** A page written long, clamped to short words. */
export function page<S extends State>(spec: LegacyPage<S>): Page {
  const { summary, live, description, states, measures, focus, content, keyboard, ...rest } = spec;
  const { described, ...matrix } = states;
  const short: Page<S> = {
    ...rest,
    lede: clamp(summary, "lede"),
    code: code(description[0]),
    stage: { live, hint: HINT, daylight: true },
    variants: variants(spec.variants),
    sizes: variants(spec.sizes),
    states: { ...matrix, rows: nonEmpty(named(matrix.rows)), absent: record(matrix.absent), looks: record(described) },
    focus: why<Focus>(focus, (lit) => ({ ...lit, looks: record(lit.looks) })),
    content: {
      text: applies(content.text) ? named(content.text) : doesntApply(note(content.text.doesntApply)),
      missingPicture: variants(content.missingPicture),
      counts: variants(content.counts),
    } satisfies Content,
    motion: motion(spec.motion),
    keyboard: why<Keyboard>(keyboard, (keys) => ({
      keys: nonEmpty(keys.keys.map((row) => ({ ...row, does: cell(row.does) }))),
      order: note(keys.order),
      lands: note(keys.lands),
    })),
    measures: {
      samples: nonEmpty(named(measures.samples).map((sample) => ({ ...sample, for: cell(sample.for) }))),
      measures: nonEmpty(measures.measures.map((measure) => ({ part: clamp(measure.what, "label"), token: measure.token }))),
      reflow: applies(measures.reflow) ? { note: note(measures.reflow.described), sample: measures.reflow.sample } : doesntApply(note(measures.reflow.doesntApply)),
    },
  };
  return shortPage(short);
}

function shown(list: NonEmpty<LegacyShown>): NonEmpty<Shown> {
  return nonEmpty(named(list).map((row) => ({ name: row.name, note: note(row.described), sample: row.sample })));
}

function shelf(list: Shelf): NonEmpty<Shown> | DoesntApply {
  return applies(list) ? shown(list) : doesntApply(note(list.doesntApply));
}

/** A foundation written long, clamped to short words. */
export function foundation(spec: LegacyFoundation): Foundation {
  const { summary, live, description: _, ...rest } = spec;
  return {
    ...rest,
    lede: clamp(summary, "lede"),
    stage: { live, hint: HINT, daylight: true },
    tokens: shown(spec.tokens),
    rules: shown(spec.rules),
    modes: shelf(spec.modes),
    themes: shelf(spec.themes),
    scales: shelf(spec.scales),
    content: shelf(spec.content),
    motion: why(spec.motion, motion),
    checks: nonEmpty(spec.checks.map((check: Check) => ({ ...check, holds: cell(check.holds) }))),
  } satisfies AnyPage;
}
