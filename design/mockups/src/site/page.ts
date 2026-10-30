// A mockup page: what a component, template or screen shows, part by part, in the order the
// mockup skill lists them (`.claude/skills/mockup/SKILL.md`); or a foundation, one family of
// tokens shown as it is used. Every part is required: a part that doesn't apply says so with
// `doesntApply(why)`, so no page can leave one out unnoticed.
//
// A page only says what it shows; `site/render.ts` draws every page of a kind the same way.

import type { Words } from "../lib/content.ts";
import type { Tone } from "../lib/focus.ts";
import type { ColourPath } from "./recolour.ts";

/** A part that doesn't apply to this page, and why. */
export interface DoesntApply {
  readonly doesntApply: string;
}

export function doesntApply(why: string): DoesntApply {
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

/** The levels, in the navigation's order (`docs/DESIGN.md`, "Levels and where they live"). */
export const LEVELS = [
  { slug: "foundations", title: "Foundations" },
  { slug: "atoms", title: "Atoms" },
  { slug: "molecules", title: "Molecules" },
  { slug: "templates", title: "Templates" },
  { slug: "shell", title: "Shell surfaces" },
  { slug: "kobo", title: "Kōbō" },
  { slug: "apps", title: "Apps" },
] as const;
export type Level = (typeof LEVELS)[number]["slug"];

/** One variant or size, what it is for, and how it looks. */
export interface Variant {
  readonly name: string;
  readonly for: string;
  readonly sample: Sample;
}

/** Every state the mockup skill names. */
export const STATES = [
  "rest",
  "hovered",
  "pressed",
  "focused",
  "disabled",
  "selected",
  "loading",
  "empty",
  "error",
] as const;
export type State = (typeof STATES)[number];

export const STATE_NAMES: Record<State, string> = {
  rest: "Resting",
  hovered: "Hovered",
  pressed: "Pressed",
  focused: "Focused",
  disabled: "Disabled",
  selected: "Selected",
  loading: "Loading",
  empty: "Empty",
  error: "Error",
};

/**
 * The state matrix: a row per variant, a column per state it has, each cell live. Every state
 * the page doesn't show is in `absent` with why, which the types hold to the list.
 */
export interface States<S extends State> {
  readonly shown: readonly S[];
  readonly absent: Record<Exclude<State, S>, string>;
  readonly rows: NonEmpty<{ readonly name: string; readonly cell: (state: S) => Element }>;
  /** What each state looks like and does, in words. */
  readonly described: Record<S, string>;
}

/**
 * A focus tube the part lights, and its tone: the part's own, or the accent for a part with none
 * (`docs/DESIGN.md`, "Focus"). A page names each, so no tube's colour is left to a default.
 */
export interface Tube {
  /** The variant or state it lights on. */
  readonly name: string;
  readonly tone: Tone;
  /** Filled with its tone's colour: the fill is the tube, and the tone's core rides outside it. */
  readonly filled: boolean;
  /** Its content is drawn in its tone, so lit, the content is a tube too: core-hot, with the glow. */
  readonly content: boolean;
}

/**
 * The looks of the focused state. Nothing the user does picks one; the scene does: `lit` is the
 * neon burning, in dark mode at rest; `unlit` is the same line as unlit glass with no glow, in
 * light mode always and in dark mode while a sequence has the lights out (`docs/DESIGN.md`, "Focus").
 */
export const LOOKS = ["lit", "unlit"] as const;
export type Look = (typeof LOOKS)[number];

export const LOOK_NAMES: Record<Look, string> = {
  lit: "Lit",
  unlit: "Unlit",
};

/**
 * The focus line a part lights: every tube in its tone, what each look makes of the line, and the
 * state matrix's "focused · unlit" column, pinned as "focused" is.
 */
export interface Focus {
  readonly tubes: NonEmpty<Tube>;
  /** What the tube, the core or the knob is in each look, in words. */
  readonly looks: Record<Look, string>;
  /** The matrix's row at `index`, focused in its unlit look. */
  readonly unlit: (index: number) => Element;
}

/** The content extremes the mockup skill names. */
export interface Content {
  /** The shortest and the longest text, and the translation 40% longer (the toolbar's text). */
  readonly text: readonly { readonly name: string; readonly words: Words; readonly sample: (words: string) => Element }[] | DoesntApply;
  readonly missingPicture: readonly Variant[] | DoesntApply;
  /** None, one and many items. */
  readonly counts: readonly Variant[] | DoesntApply;
}

/** A duration token's name. */
export type Duration = "instant" | "micro" | "short" | "medium" | "long" | "slow" | "scene" | "recolour" | "settle" | "strike";
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
 * The neon sign's stutter: `neon.strike`'s steps, a tube switching on, or the same steps played
 * backwards, a tube switching off.
 */
export type Strike = "neon.strike" | "neon.strike.reversed";

/** How a move runs over its duration: an easing curve, or the neon sign's stutter. */
export type Curve = Easing | Strike;

/**
 * A value an animated property takes: a plain number, or a token's, named so a library reads the
 * same token: an opacity, a colour role, a length.
 */
export type Value = number | { readonly opacity: string } | { readonly color: string } | { readonly length: string };

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
  readonly name: string;
  readonly what: string;
  /** The sample the move plays on, at rest: the player moves its parts. */
  readonly sample: () => Element;
  /** A still beat at the move's end, after its last part, that belongs to the move. */
  readonly tail?: Duration;
  /**
   * What plays in light mode, where the neon is off, when that differs. Nothing in it strikes, so
   * the stutter doesn't change it; reduced motion's own tracks still win.
   */
  readonly daylight?: NonEmpty<Track<Easing>>;
}

/** A move on easing curves alone. */
interface Eased {
  readonly tracks: NonEmpty<Track<Easing>>;
  /** What plays with reduced motion: the same tracks, or the ones it turns them into. */
  readonly reduced: "same" | NonEmpty<Track<Easing>>;
  readonly unstuttered?: never;
  /**
   * How the loop goes back to the first state: the move `plays` backwards where that is the same
   * move reversed (a hover leaving), or it `cuts` back where going back is a move of its own
   * (focus leaving after focus arriving), which its own player shows.
   */
  readonly back: "plays" | "cuts";
}

/**
 * A move that strikes as a neon tube does. Reduced motion and the stutter off each turn a strike
 * into a fade, so it names both; a strike is never played backwards, so its loop cuts back.
 */
interface Struck {
  readonly tracks: NonEmpty<Track>;
  readonly reduced: NonEmpty<Track<Easing>>;
  readonly unstuttered: NonEmpty<Track<Easing>>;
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
  readonly keys: NonEmpty<{ readonly keys: NonEmpty<string>; readonly does: string }>;
  readonly order: string;
  readonly lands: string;
}

/** A measure of the sample, by the token it comes from. */
export interface Measure {
  readonly what: string;
  readonly token: string;
}

/** Every box, padding and gap by its token, and the reflow across window sizes. */
export interface Measures {
  /** The samples the measures overlay outlines, box by box. */
  readonly samples: NonEmpty<Variant>;
  readonly measures: NonEmpty<Measure>;
  /** The part in a window of the toolbar's size, from the minimum to a full screen. */
  readonly reflow: { readonly described: string; readonly sample: Sample } | DoesntApply;
}

export interface Page<S extends State = State> {
  readonly level: Exclude<Level, "foundations">;
  /** The page's address and its shots' name: `button`. */
  readonly slug: string;
  readonly title: string;
  /** One sentence: what it is. */
  readonly summary: string;
  /** The thing itself, live, which the description sits beside. */
  readonly live: Sample;
  /** How it looks and behaves, a paragraph each. */
  readonly description: NonEmpty<string>;
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

/** A part of a foundation: its name, what it shows in words, and the thing itself, live. */
export interface Shown {
  readonly name: string;
  readonly described: string;
  readonly sample: Sample;
}

/** What holds a family of tokens to its rules, and how. */
export interface Check {
  readonly check: string;
  readonly holds: string;
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
  readonly summary: string;
  /** The family in use, live, which the description sits beside. */
  readonly live: Sample;
  /** What the family is for and how it is picked, a paragraph each. */
  readonly description: NonEmpty<string>;
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
