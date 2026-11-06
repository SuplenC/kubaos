// The PictureChoice page: `docs/design/components/picture-choice.md`. A choice made by its
// picture, a wallpaper above all: the picture edge to edge in its tile, its name under it or over
// its foot, picked with the accent's line and its mark, lit with the cold tube.

import hero from "../../../../../branding/png/kubaos-hero.jpg?url";
import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import { choiceGroup, type Picks } from "../../lib/card.ts";
import type { Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { stack, within } from "../../lib/layout.ts";
import { type PictureChoiceOptions, pictureChoice, setPickedPicture, type Tag } from "../../lib/picture-choice.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const NAME: Words = { short: "Shinjuku", long: "Shinjuku, night: the alley behind the station after the rain" };

/** A thumbnail's room: `size.thumbnail_width`. */
const THUMB = "var(--size-thumbnail_width)";

const CURRENT: Tag = { icon: "check", label: "Current" };
const PICKED: Tag = { icon: "check", label: "Picked" };
const VIDEO: Tag = { icon: "play_arrow", label: "Video" };

const WALLPAPERS: readonly PictureChoiceOptions[] = [
  { source: photo, name: "Shinjuku, night" },
  { source: wallpaper, name: "Golden Gai" },
  { source: hero, name: "Omoide Yokocho", badge: VIDEO },
  { source: blurred, name: "Kabukicho, rain" },
  { source: photo, name: "Ebisu", arrival: "loading" },
  { source: wallpaper, name: "Shimokita", disabled: true },
];

/** `options` alone, in a thumbnail's room, in `state`. */
function boxed(options: PictureChoiceOptions, state: Shown = "rest"): HTMLElement {
  return within(THUMB, pictureChoice({ ...options, ...stateOptions(state) }));
}

/** A set of pictures, `columns` across: picked as one (the shell's picker) or many (Kōbō's). */
function set(picks: Picks, items: readonly PictureChoiceOptions[], columns = 3): HTMLElement {
  const many = picks === "many";
  const cells = items.map((options, index) =>
    pictureChoice({ ...options, many, mark: many ? PICKED : CURRENT, place: many ? "over" : "under", picked: index === 0 || (many && index === 2) }),
  );
  const width = `calc(${columns} * var(--size-thumbnail_width) + ${columns - 1} * var(--space-s16))`;
  return within(width, choiceGroup(picks, columns, cells, { target: (cell) => cell.querySelector<HTMLElement>(".k-picture-tile")!, set: setPickedPicture, arrowsPick: false }));
}

function live(): HTMLElement {
  return stack("s32", "start", set("one", WALLPAPERS.slice(0, 2), 2), set("many", WALLPAPERS.slice(0, 2), 2));
}

const ROWS: readonly { name: string; options: PictureChoiceOptions }[] = [
  { name: "Not picked, name under", options: { source: wallpaper, name: "Golden Gai" } },
  { name: "Picked, name under", options: { source: photo, name: "Shinjuku, night", picked: true, mark: CURRENT } },
  { name: "Not picked, name over", options: { source: hero, name: "Omoide Yokocho", place: "over", badge: VIDEO } },
  { name: "Picked, name over", options: { source: photo, name: "Shinjuku, night", place: "over", picked: true, mark: PICKED } },
];

type MatrixState = "rest" | "lit" | "pressed" | "disabled" | "loading";

function cell(options: PictureChoiceOptions, state: MatrixState): HTMLElement {
  if (state === "loading") return boxed({ ...options, arrival: "loading" });
  return boxed(options, state);
}

/** A picked picture and one not, in `state`: what the lighting moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () =>
    within(
      `calc(2 * ${THUMB} + var(--space-s16))`,
      choiceGroup("one", 2, [pictureChoice({ ...ROWS[1]!.options, ...stateOptions(state) }), pictureChoice({ ...ROWS[0]!.options, mark: CURRENT, ...stateOptions(state) })], {
        target: (each) => each.querySelector<HTMLElement>(".k-picture-tile")!,
        set: setPickedPicture,
      }),
    );
}

const ON = "a picked picture and one not";

export default page({
  level: "molecules",
  slug: "picture-choice",
  title: "PictureChoice",
  lede: "A choice made by its picture: as large as its tile, picked with the accent's line and its mark.",
  code: { rust: "picture::picture_choice(picture, aspect, name)", qml: "PictureChoice", unbuilt: "cold lit tube: not built yet" },
  stage: { live, hint: "press a picture, or Tab, the arrows and Enter", daylight: true },
  variants: [
    { name: "Name under", for: "the shell's picker: one applied, Current on it", sample: () => boxed(ROWS[1]!.options) },
    { name: "Name over", for: "Kōbō's grid: several picked, the name on the foot", sample: () => boxed(ROWS[3]!.options) },
    { name: "With a badge", for: "a fact whatever its state: Video", sample: () => boxed(ROWS[2]!.options) },
    { name: "One of a set", for: "one picked, as radios", sample: () => set("one", WALLPAPERS.slice(0, 3)) },
    { name: "Any of a set", for: "each picked or not, as checkboxes", sample: () => set("many", WALLPAPERS.slice(0, 3)) },
  ],
  sizes: doesntApply("as wide as its cell, a thumbnail's in the pickers, as tall as its aspect."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled", "loading"],
    absent: {
      selected: "picked and not are rows here",
      empty: "a set with none is an `EmptyState`",
      error: "keeps its placeholder, held still; its name says which",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    looks: {
      rest: "picked: the accent's line, glow at night, its mark",
      lit: "the cold tube in the picked line's place, glow at night",
      pressed: "tube `cold_core`, glow × `focus.pressed_glow`; by day a cold tint",
      disabled: "picture at `opacity.disabled_content`, name dimmed; skipped by the keys",
      loading: "its placeholder breathing in the box",
    },
  },
  focus: {
    tubes: [
      { name: "Picked, at rest", tone: "accent", filled: false, content: false },
      { name: "Lit, picked or not", tone: "cold", filled: false, content: false },
    ],
    looks: {
      lit: "accent tube when picked, cold tube when lit, glow outside",
      signsOff: "`primary_glass` picked, `cold_glass` lit, no glow",
    },
    signsOff: (index: number) => boxed(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "The name under", words: NAME, sample: (words: string) => boxed({ source: photo, name: words, picked: true, mark: CURRENT }) },
      { name: "The name over", words: NAME, sample: (words: string) => boxed({ source: photo, name: words, place: "over" }) },
    ],
    missingPicture: [
      { name: "Not yet loaded", for: "the placeholder breathes; the grid doesn't move", sample: () => boxed({ name: "Ebisu", arrival: "loading" }) },
      { name: "Can't be read", for: "the placeholder held still; the name says which", sample: () => boxed({ name: "Ebisu", arrival: "broken", place: "over" }) },
    ],
    counts: [
      { name: "One", for: "no choice: the screen shows a `Media` instead", sample: () => boxed({ source: photo, name: "Shinjuku, night", picked: true, mark: CURRENT }) },
      { name: "A row", for: "three across", sample: () => set("one", WALLPAPERS.slice(0, 3)) },
      { name: "Many", for: "rows of three, one height each", sample: () => set("one", WALLPAPERS) },
    ],
  },
  motion: {
    arrives: doesntApply("a set comes with its surface; a picture replaces its placeholder as `Media` says."),
    leaves: doesntApply("it goes with its surface."),
    between: [
      {
        name: "Picked",
        what: "The accent's line, mark and name fade in over `short` on `standard`, as the old pick fades back.",
        tracks: [
          { part: "picked", on: "the picture picked", target: ".k-picture-choice:last-child", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" },
          { part: "picked", on: "the picture it unpicks", target: ".k-picture-choice:first-child", property: "--checked", from: 1, to: 0, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: pair(),
      },
      { name: "Focus arrives", what: "It lights up over `short` on `standard`: the cold tube and glow in, a picked one's line out.", tracks: lights(".k-picture-tile", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-picture-tile", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot and the glow brightens; by day the cold tone tints it.", tracks: press(".k-picture-tile", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "into the set or out: one stop, on its pick" },
      { keys: ["←", "→", "h", "l"], does: "to the picture before or after; moving never picks" },
      { keys: ["↑", "↓", "k", "j"], does: "to the picture above or below, a row at a time" },
      { keys: ["Home", "End"], does: "to the first or last picture" },
      { keys: ["Enter", "Space"], does: "picks it, or turns it in a many-pick set" },
    ],
    order: "One stop in reading order; the arrows pass a disabled picture and stop at the set's ends.",
    lands: "A picker opens on its pick; a pick keeps the keyboard where it is.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Name under, picked", for: "box, mark `space.s8` in, name `space.s8` under", sample: () => boxed(ROWS[1]!.options) },
      { name: "Name over, with a badge", for: "badge `space.s8` in, name `space.s12` from the foot", sample: () => boxed(ROWS[2]!.options) },
    ]),
    measures: [
      { part: "Width in the pickers", token: "size-thumbnail_width" },
      { part: "Aspect", token: "aspect-wallpaper" },
      { part: "Corners", token: "radius-control" },
      { part: "Chip inset, name under", token: "space-s8" },
      { part: "Name over, inset", token: "space-s12" },
      { part: "Name line", token: "type-small-line" },
      { part: "Chips", token: "size-chip" },
      { part: "Between pictures", token: "space-s16" },
    ],
    reflow: {
      note: "A thumbnail wide each; a picker shows as many columns as fit and scrolls a whole row at a time.",
      sample: () => within("100%", set("one", WALLPAPERS, 3)),
    },
  },
});
