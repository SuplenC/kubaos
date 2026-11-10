// A mockup page: what a component, template or screen shows, part by part, in the order the
// mockup skill lists them (`.claude/skills/mockup/SKILL.md`); or a foundation, one family of
// tokens shown as it is used. Every part is required: a part that doesn't apply says so with
// `doesntApply(why)`, so no page can leave one out unnoticed.
//
// A page's words are a one-line lede, labels, table cells and one-line notes, never paragraphs:
// the samples show what the words would restate, and the full reference is the component's page
// under `docs/design/components/`. Each kind of words has a most it may hold (`MOST`), and
// `wordy` finds every one past it, which the site refuses to draw.
//
// A page only says what it shows; `site/render.ts` draws every page of a kind the same way.

import type { Words } from "../lib/content.ts";
import type { Tone } from "../lib/focus.ts";
import type { ColourPath } from "./recolour.ts";

/** A name: a variant's, a part's, a row's. A few words. */
export type Label = string;
/** A table's cell: what a variant is for, what a state looks like, what a key does. */
export type Cell = string;
/** A note under a sample or a move, or why a part doesn't apply: one line, two at most. */
export type Note = string;
/** The page's lede: what it is, one sentence. */
export type Lede = string;

/** The most words each kind holds, a `backticked` token counting as one. */
export const MOST = { label: 6, cell: 12, note: 24, lede: 24 } as const;
export type WordKind = keyof typeof MOST;

/** A part that doesn't apply to this page, and why, in a note: drawn as one muted line. */
export interface DoesntApply {
  readonly doesntApply: Note;
}

export function doesntApply(why: Note): DoesntApply {
  return { doesntApply: why };
}

export function applies<T>(part: T | DoesntApply): part is T {
  return !(typeof part === "object" && part !== null && "doesntApply" in part);
}

/** A list that holds at least one thing: a required part can't be left empty. */
export type NonEmpty<T> = readonly [T, ...T[]];

/** `list` as a non-empty one, for a list made by `map`: a page with nothing in it fails to draw. */
export function nonEmpty<T>(list: readonly T[]): NonEmpty<T> {
  if (list.length === 0) throw new Error("a required part of a page is empty");
  return list as unknown as NonEmpty<T>;
}

/** Something live on the page: drawn again whenever the words or the page change. */
export type Sample = () => Element;

/**
 * The levels (`docs/DESIGN.md`, "Levels and where they live"), and the layout primitives beside
 * them, in the navigation's order.
 */
export const LEVELS = [
  { slug: "foundations", title: "Foundations" },
  { slug: "layout", title: "Layout" },
  { slug: "atoms", title: "Atoms" },
  { slug: "molecules", title: "Molecules" },
  { slug: "templates", title: "Templates" },
  { slug: "shell", title: "Shell surfaces" },
  { slug: "kobo", title: "Kōbō" },
  { slug: "apps", title: "Apps" },
] as const;
export type Level = (typeof LEVELS)[number]["slug"];

/** One variant or size, what it is for in a few words, and how it looks. */
export interface Variant {
  readonly name: Label;
  readonly for: Cell;
  readonly sample: Sample;
}

/** Every state the mockup skill names, and open, which only a part that opens has. */
export const STATES = [
  "rest",
  "lit",
  "pressed",
  "open",
  "disabled",
  "selected",
  "loading",
  "empty",
  "error",
] as const;
export type State = (typeof STATES)[number];
/** The states only some parts have: a page without one leaves it out with no word on why. */
type Optional = "open";

export const STATE_NAMES: Record<State, string> = {
  rest: "Rest",
  lit: "Lit",
  pressed: "Pressed",
  open: "Open",
  disabled: "Disabled",
  selected: "Selected",
  loading: "Loading",
  empty: "Empty",
  error: "Error",
};

/**
 * The states: a row per variant and a column per state it has, which the stage's chips pin and the
 * compact matrix draws once in each mode. Every state the page doesn't show is in `absent` with
 * why, which the types hold to the list.
 */
export interface States<S extends State> {
  readonly shown: readonly S[];
  readonly absent: Record<Exclude<State, S | Optional>, Cell>;
  readonly rows: NonEmpty<{ readonly name: Label; readonly cell: (state: S) => Element }>;
  /** What each state looks like, a few words: the states table's second column. */
  readonly looks: Record<S, Cell>;
}

/**
 * The line and glow a part lights, and its tone: the part's own, or the accent for a part with
 * none (`docs/DESIGN.md`, "States"). A page names each, so no line's colour is left to a default.
 */
export interface Tube {
  /** The variant or state it lights on. */
  readonly name: Label;
  readonly tone: Tone;
  /** It has a fill: the line inside the fill's edge is the tone's core by night. */
  readonly filled: boolean;
  /** Its content is drawn in its tone, so lit at night, the content burns as a tube too. */
  readonly content: boolean;
}

/**
 * The looks of the lit state. Nothing the user does picks one; the scene does: `lit` is the signs
 * on, the neon burning, at night at rest; `signsOff` is the same control still lit with the neon
 * unpowered, its line unlit glass with no glow, by day always and at night while a power change has
 * the lights out (`docs/DESIGN.md`, "States").
 */
export const LOOKS = ["lit", "signsOff"] as const;
export type Look = (typeof LOOKS)[number];

export const LOOK_NAMES: Record<Look, string> = {
  lit: "Lit · signs on",
  signsOff: "Lit · signs off",
};

/**
 * The line a part lights when it is lit: every line in its tone, what each look makes of it, and
 * the state matrix's "Lit · signs off" column at night, pinned as "Lit" is.
 */
export interface Focus {
  readonly tubes: NonEmpty<Tube>;
  /** What the line, the glow and the content are in each look, a few words. */
  readonly looks: Record<Look, Cell>;
  /** The matrix's row at `index`, lit with the signs off. */
  readonly signsOff: (index: number) => Element;
}

/** The content extremes the mockup skill names. */
export interface Content {
  /** The shortest and the longest text, and the translation 40% longer (the toolbar's text). */
  readonly text: readonly { readonly name: Label; readonly words: Words; readonly sample: (words: string) => Element }[] | DoesntApply;
  readonly missingPicture: readonly Variant[] | DoesntApply;
  /** None, one and many items. */
  readonly counts: readonly Variant[] | DoesntApply;
}

/** A duration token's name. */
export type Duration = "instant" | "micro" | "short" | "medium" | "long" | "slow" | "scene" | "recolour" | "settle" | "strike" | "caret_glow" | "caret_hold" | "caret_fade" | "hum" | "swap_lag" | "swap_fade";
/** An easing token's name. */
export type Easing =
  | "standard"
  | "standard_decel"
  | "standard_accel"
  | "emphasized_decel"
  | "emphasized_accel"
  | "linear"
  | "pop"
  | "spring"
  | "gather";

/**
 * The neon sign's flicker: `neon.strike`'s steps, a tube switching on, or the same steps played
 * backwards, a tube switching off.
 */
export type Strike = "neon.strike" | "neon.strike.reversed";

/** How a move runs over its duration: an easing curve, or the neon sign's flicker. */
export type Curve = Easing | Strike;

/**
 * A value an animated property takes: a plain number, or a token's, named so a library reads the
 * same token: an opacity, a scale factor, a colour role, a length.
 */
export type Value = number | { readonly opacity: string } | { readonly scale: string } | { readonly color: string } | { readonly length: string };

/**
 * One part's change in a move: what it is, the property, its values and its timing. Tracks of one
 * move on the same target and property play as one timeline in cue order: until a later one's cue
 * the value is what the earlier left. They are listed in cue order, take numbers and never overlap
 * (`recipe.ts`, `leads`); the recipe's JSON keeps each as it is, for a library to play in that order.
 */
export interface Track<C extends Curve = Curve> {
  /** The part, as a library names it: the state layer, the tube, the glow, the core. */
  readonly part: string;
  /** Which of the sample's variants it is on: every kind, Tonal and Text. */
  readonly on: string;
  /** The sample's elements it moves, as a selector inside the sample. */
  readonly target: string;
  /** The CSS property animated, as the libraries animate the same one: `opacity`. */
  readonly property: string;
  readonly from: Value;
  readonly to: Value;
  readonly duration: Duration;
  /**
   * How long after the move starts this part does, a duration or the sum of several, one per
   * phase before it; a part with none starts with it.
   */
  readonly cue?: Duration | NonEmpty<Duration>;
  readonly curve: C;
  /**
   * For a track that moves every colour role from one palette to another (`--recolour`, 0 to 1):
   * the path each role takes between its two colours, and the palettes the sample goes between.
   */
  readonly colours?: Recolour;
}

/** Every colour role moving from one palette to another along a path. */
export interface Recolour {
  readonly path: ColourPath;
  /** The palette it starts in and the one it ends in, by name: Neon sign, dark. */
  readonly from: string;
  readonly to: string;
}

/**
 * One move, as data: every part that moves, each property's values and timing. Its player animates
 * from this same object and shows it as the move's recipe, so what plays and what a library reads
 * never disagree. The player loops it: the first state held, the move, the last state held, back.
 */
export type Move = MoveWords & (Eased | Struck);

interface MoveWords {
  readonly name: Label;
  /** What happens, one line: the player and its recipe show the rest. */
  readonly what: Note;
  /** The sample the move plays on, at rest: the player moves its parts. */
  readonly sample: () => Element;
  /** A still beat at the move's end, after its last part, that belongs to the move. */
  readonly tail?: Duration;
  /**
   * What plays in light mode, where the neon is off, when that differs. Nothing in it strikes, so
   * the flicker doesn't change it; reduced motion's own tracks still win.
   */
  readonly daylight?: NonEmpty<Track<Easing>>;
}

/** A move on easing curves alone. */
interface Eased {
  readonly tracks: NonEmpty<Track<Easing>>;
  /** What plays with reduced motion: the same tracks, or the ones it turns them into. */
  readonly reduced: "same" | NonEmpty<Track<Easing>>;
  readonly unflickered?: never;
  /**
   * How the loop goes back to the first state: the move `plays` backwards where that is the same
   * move reversed (a hover leaving), or it `cuts` back where going back is a move of its own
   * (focus leaving after focus arriving), which its own player shows.
   */
  readonly back: "plays" | "cuts";
}

/**
 * A move that strikes as a neon tube does. Reduced motion and the flicker off each turn a strike
 * into a fade, so it names both; a strike is never played backwards, so its loop cuts back.
 */
interface Struck {
  readonly tracks: NonEmpty<Track>;
  readonly reduced: NonEmpty<Track<Easing>>;
  readonly unflickered: NonEmpty<Track<Easing>>;
  readonly back: "cuts";
}

/** How the part arrives, leaves and moves between its states. */
export interface Motion {
  readonly arrives: readonly Move[] | DoesntApply;
  readonly leaves: readonly Move[] | DoesntApply;
  readonly between: readonly Move[] | DoesntApply;
}

/** The keyboard: every key, where focus goes, and where it lands. */
export interface Keyboard {
  readonly keys: NonEmpty<{ readonly keys: NonEmpty<string>; readonly does: Cell }>;
  /** Where Tab and Shift+Tab take focus. */
  readonly order: Note;
  /** Where focus lands when the part appears or goes. */
  readonly lands: Note;
}

/** A measure of the sample, by the token it comes from: the anatomy table's row. */
export interface Measure {
  /** The part it measures, a few words: Height, Inset, Option row. */
  readonly part: Label;
  readonly token: string;
}

/** Every box, padding and gap by its token, and the reflow across window sizes. */
export interface Measures {
  /** The samples the measures inspect, part by part and token by token. */
  readonly samples: NonEmpty<Variant>;
  readonly measures: NonEmpty<Measure>;
  /** The part in a window of the toolbar's size, from the minimum to a full screen. */
  readonly reflow: { readonly note: Note; readonly sample: Sample } | DoesntApply;
}

/**
 * The page's names in code, for its header's tags: only those the component's page under
 * `docs/design/components/` gives, and where it says a toolkit has yet to build it, that.
 */
export interface Code {
  readonly rust?: string;
  readonly qml?: string;
  /** What isn't built yet, as that page says: QML not built yet. */
  readonly unbuilt?: Label;
}

/**
 * The stage, which leads the page: the thing itself, live and large, by night and by day side by
 * side, under a bar of the states' chips with a hint on how to try it.
 */
export interface Stage {
  readonly live: Sample;
  /** How to try the live sample, a few words: click it, or Tab and ↓. */
  readonly hint: Cell;
  /** The day pane, or why it doesn't apply: a lock drawn in its wallpaper's night scheme. */
  readonly daylight: true | DoesntApply;
}

export interface Page<S extends State = State> {
  readonly level: Exclude<Level, "foundations">;
  /** The page's address and its shots' name: `button`. */
  readonly slug: string;
  readonly title: string;
  /** One sentence: what it is. */
  readonly lede: Lede;
  readonly code: Code;
  /**
   * The library's parts it is built from, by their pages' slugs, for a page of a surface built
   * from them: its header says "Built from" and links each.
   */
  readonly from?: NonEmpty<string>;
  readonly stage: Stage;
  readonly variants: readonly Variant[] | DoesntApply;
  readonly sizes: readonly Variant[] | DoesntApply;
  readonly states: States<S>;
  /**
   * Its focus line: every tube it lights, each in its tone, and both looks of it; a part with no
   * focus line doesn't apply.
   */
  readonly focus: Focus | DoesntApply;
  readonly content: Content;
  readonly motion: Motion;
  readonly keyboard: Keyboard | DoesntApply;
  readonly measures: Measures;
}

/**
 * A page, with its states' type worked out from what it shows. The list of pages holds pages of
 * every kind of state matrix, so each is widened to `Page`: the matrix calls a row's `cell` only
 * with the states in `shown`, which are `S`, so the wider type is never used to call it otherwise.
 */
export function page<S extends State>(spec: Page<S>): Page {
  return spec as unknown as Page;
}

/** A part of a foundation: its name, a note on what to read in it, and the thing itself, live. */
export interface Shown {
  readonly name: Label;
  readonly note: Note;
  readonly sample: Sample;
}

/** What holds a family of tokens to its rules, and how. */
export interface Check {
  readonly check: Label;
  readonly holds: Cell;
}

/**
 * A foundation: one family of tokens (colour, type, space, radii, elevation), every token shown
 * as it is used. It isn't a component, so it has no states, keyboard or focus line of its own;
 * every part it does have is required, as a component page's are, and one that doesn't apply says
 * why.
 */
export interface Foundation {
  readonly level: "foundations";
  readonly slug: string;
  readonly title: string;
  /** One sentence: what the family is. */
  readonly lede: Lede;
  /** The family in use, live, by night and by day. */
  readonly stage: Stage;
  /** Every token of the family, live, by its token, with its value and what it is for. */
  readonly tokens: NonEmpty<Shown>;
  /** The rules the values keep, each with a worked example, live. */
  readonly rules: NonEmpty<Shown>;
  /** The family in light and in dark, side by side. */
  readonly modes: NonEmpty<Shown> | DoesntApply;
  /** The family in every theme the toolbar paints in, so it is never judged in one accent alone. */
  readonly themes: NonEmpty<Shown> | DoesntApply;
  /** The family at 1, 1.25, 1.5 and 2, in device pixels. */
  readonly scales: NonEmpty<Shown> | DoesntApply;
  /** The content extremes: the longest word, a translation 40% longer. */
  readonly content: NonEmpty<Shown> | DoesntApply;
  /** How the family moves, as moves of data. */
  readonly motion: Motion | DoesntApply;
  /** What holds the family to its rules. */
  readonly checks: NonEmpty<Check>;
}

/** A foundation's page. */
export function foundation(spec: Foundation): Foundation {
  return spec;
}

/** Any page of the site. */
export type AnyPage = Page | Foundation;

export function isFoundation(page: AnyPage): page is Foundation {
  return page.level === "foundations";
}

/** `words`' count, a `backticked` token one word. */
export function wordCount(words: string): number {
  return words.replaceAll(/`[^`]*`/g, "x").split(/\s+/).filter(Boolean).length;
}

/** A field of a page past its kind's most: where it is, its kind and its words. */
export interface Wordy {
  readonly where: string;
  readonly kind: WordKind;
  readonly words: string;
}

/** Every field of `page` past its kind's most: none on a page the site draws. */
export function wordy(page: AnyPage): Wordy[] {
  const found: Wordy[] = [];
  const hold = (where: string, kind: WordKind, words: string) => {
    if (wordCount(words) > MOST[kind] || (kind === "lede" && /[.!?]\s+\S/.test(words))) found.push({ where, kind, words });
  };
  const each = <T>(part: T | DoesntApply, where: string, check: (value: T) => void) => {
    if (applies(part)) check(part);
    else hold(where, "note", part.doesntApply);
  };
  const variants = (list: readonly Variant[] | DoesntApply, where: string) =>
    each(list, where, (rows) => rows.forEach((row) => (hold(`${where} ${row.name}`, "label", row.name), hold(`${where} ${row.name}`, "cell", row.for))));
  const motion = (spec: Motion | DoesntApply) =>
    each(spec, "motion", (moves) =>
      (["arrives", "leaves", "between"] as const).forEach((when) =>
        each(moves[when], `motion ${when}`, (list) => list.forEach((move) => (hold(`move ${move.name}`, "label", move.name), hold(`move ${move.name}`, "note", move.what)))),
      ),
    );
  hold("lede", "lede", page.lede);
  hold("stage hint", "cell", page.stage.hint);
  each(page.stage.daylight, "stage daylight", () => {});
  motion(page.motion);
  if (isFoundation(page)) {
    const shown = (list: NonEmpty<Shown> | DoesntApply, where: string) =>
      each(list, where, (rows) => rows.forEach((row) => (hold(`${where} ${row.name}`, "label", row.name), hold(`${where} ${row.name}`, "note", row.note))));
    shown(page.tokens, "tokens");
    shown(page.rules, "rules");
    shown(page.modes, "modes");
    shown(page.themes, "themes");
    shown(page.scales, "scales");
    shown(page.content, "content");
    page.checks.forEach((check) => (hold(`check ${check.check}`, "label", check.check), hold(`check ${check.check}`, "cell", check.holds)));
    return found;
  }
  if (page.code.unbuilt) hold("code", "label", page.code.unbuilt);
  variants(page.variants, "variant");
  variants(page.sizes, "size");
  for (const [state, words] of Object.entries(page.states.looks)) hold(`state ${state}`, "cell", words as string);
  for (const [state, words] of Object.entries(page.states.absent)) hold(`absent ${state}`, "cell", words as string);
  page.states.rows.forEach((row) => hold(`row ${row.name}`, "label", row.name));
  each(page.focus, "focus", (focus) => LOOKS.forEach((look) => hold(`look ${look}`, "cell", focus.looks[look])));
  each(page.content.text, "content text", (rows) => rows.forEach((row) => hold(`content ${row.name}`, "label", row.name)));
  variants(page.content.missingPicture, "missing picture");
  variants(page.content.counts, "counts");
  each(page.keyboard, "keyboard", (keyboard) => {
    keyboard.keys.forEach((row) => hold(`key ${row.keys.join(" ")}`, "cell", row.does));
    hold("keyboard order", "note", keyboard.order);
    hold("keyboard lands", "note", keyboard.lands);
  });
  variants(page.measures.samples, "measured");
  page.measures.measures.forEach((measure) => hold(`measure ${measure.token}`, "label", measure.part));
  each(page.measures.reflow, "reflow", (reflow) => hold("reflow", "note", reflow.note));
  return found;
}
