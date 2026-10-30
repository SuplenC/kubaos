// A mockup page: what a component, template or screen shows, part by part, in the order the
// mockup skill lists them (`.claude/skills/mockup/SKILL.md`). Every part is required: a part that
// doesn't apply says so with `doesntApply(why)`, so no page can leave one out unnoticed.
//
// A page only says what it shows; `site/render.ts` draws every page the same way.

import type { Words } from "../lib/content.ts";
import type { Tone } from "../lib/focus.ts";

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
export type Duration = "instant" | "micro" | "short" | "medium" | "long" | "slow" | "scene";
/** An easing token's name. */
export type Easing =
  | "standard"
  | "standard_decel"
  | "standard_accel"
  | "emphasized_decel"
  | "emphasized_accel"
  | "linear"
  | "pop"
  | "spring";

/**
 * How a move runs over its duration: an easing curve, or the neon sign's stutter, `neon.strike`'s
 * steps.
 */
export type Curve = Easing | "neon.strike";

/**
 * A value an animated property takes: a plain number, or a token's, named so a library reads the
 * same token: an opacity, a colour role, a length.
 */
export type Value = number | { readonly opacity: string } | { readonly color: string } | { readonly length: string };

/** One part's change in a move: what it is, the property, its values and its timing. */
export interface Track {
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
  /** How long after the move starts this part does; a part with none starts with it. */
  readonly cue?: Duration;
  readonly curve: Curve;
}

/**
 * One move, as data: every part that moves, each property's values and timing. Its player animates
 * from this same object and shows it as the move's recipe, so what plays and what a library reads
 * never disagree. The player loops it: the first state held, the move, the last state held, back.
 */
export interface Move {
  readonly name: string;
  readonly what: string;
  readonly tracks: NonEmpty<Track>;
  /** What plays with reduced motion: the same tracks, or the ones it turns them into. */
  readonly reduced: "same" | NonEmpty<Track>;
  /** What plays with the focus stutter off, which a move that strikes must say. */
  readonly unstuttered?: NonEmpty<Track>;
  /**
   * How the loop goes back to the first state: the move `plays` backwards where that is the same
   * move reversed (a hover leaving), or it `cuts` back where going back is a move of its own
   * (focus leaving after focus arriving), which its own player shows.
   */
  readonly back: "plays" | "cuts";
  /** The sample the move plays on, at rest: the player moves its parts. */
  readonly sample: () => Element;
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
  readonly level: Level;
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
  /** Every focus tube it lights, each in its tone; a part that takes no focus doesn't apply. */
  readonly focus: NonEmpty<Tube> | DoesntApply;
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
