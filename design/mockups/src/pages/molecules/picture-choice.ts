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
import { text } from "../../lib/text.ts";
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
  return stack(
    "s32",
    "start",
    stack("s12", "start", text("The shell's wallpaper picker: one applied, its name under it.", "small", "on_surface_variant"), set("one", WALLPAPERS.slice(0, 4), 2)),
    stack("s12", "start", text("Kōbō's wallpapers: several picked, the name over the foot.", "small", "on_surface_variant"), set("many", WALLPAPERS.slice(0, 4), 2)),
  );
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
  summary: "A choice made by its picture, a wallpaper above all: the picture as large as its tile, picked with the accent's line and its mark.",
  live,
  description: [
    "Rust `picture::picture_choice(picture, aspect, name)`, QML `PictureChoice`. The shell's wallpaper picker (one applied, its name under it, Current) and Kōbō's wallpapers (several picked, the name over the foot, Picked) are this one component. Press a picture, or Tab into the set, move with the arrows and press Enter.",
    "The picture covers the whole tile, edge to edge in its own rounded shape at the control radius (12): a `Media` box at its `[aspect]`, held before the picture loads, with no frame round it, so the picture is as large as the tile. A badge (a `Neutral` chip: Video) sits `space.s8` in from the top left whatever its state.",
    "Its name is `small` on one line ending in an ellipsis: under the picture, `space.s8` below it, in `on_surface_variant`, `on_surface` while it is picked (`name-under`); or over its foot, `space.s12` in, light on a scrim of `scrim` darkening from nothing 40% down the tile to `opacity.backdrop` at its foot, whatever the mode, since it is on a photo (`name-over`).",
    "Picked, it says so with the accent: the line inside its edge burns as a tube (`primary` at `focus.tube`) with the glow outside at night, and its mark shows in a `Selected` chip `space.s8` in from the top right (a check and Current, or Picked). Picking fades the line, the mark and the name over `short`.",
    "Its states follow the neon. Lit, under the pointer or with the keyboard on it, the cold tube (`cold` at `focus.tube`) burns inside its edge with the glow outside at night, in the picked line's place, since the accent already means picked. Pressed, the tube runs white-hot, the glow brightens, and by day the cold tone tints the picture (`opacity.press_tint`). Disabled (a wallpaper this screen can't show), the picture is at `opacity.disabled_content` (its placeholder the disabled container), its name and chips dim, and it has no line, no press and no stop for the keyboard.",
    "A set lays its pictures in a grid `space.s16` apart, each a thumbnail's width (`size.thumbnail_width`), every row one height. A grid that scrolls moves a whole row at a time, so no picture is ever cut (`docs/DESIGN.md`, \"Rules every component keeps\").",
  ],
  variants: [
    { name: "Name under", for: "The shell's picker: one applied, Current on it.", sample: () => boxed(ROWS[1]!.options) },
    { name: "Name over", for: "Kōbō's grid: several picked, the name on the photo's foot.", sample: () => boxed(ROWS[3]!.options) },
    { name: "With a badge", for: "A fact about the picture whatever its state: Video.", sample: () => boxed(ROWS[2]!.options) },
    { name: "One of a set", for: "One picked, as radios: picking another unpicks it.", sample: () => set("one", WALLPAPERS.slice(0, 3)) },
    { name: "Any of a set", for: "Each picked or not, as checkboxes.", sample: () => set("many", WALLPAPERS.slice(0, 3)) },
  ],
  sizes: doesntApply("a picture choice is as wide as its cell, a thumbnail's width in the pickers, and as tall as its aspect makes that."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled", "loading"],
    absent: {
      selected: "picked and not are its value, and each is a row here, so every state shows in both.",
      empty: "a set with no pictures is an `EmptyState` in the grid's place; a picture with none is its placeholder, under Loading.",
      error: "a picture that can't be read keeps its placeholder, held still, as `Media` shows; its name says which it is.",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: MatrixState) => cell(options, state) }))),
    described: {
      rest: "Not picked, the picture alone; picked, the accent's line inside its edge with the glow outside at night, and its mark. By day the line with no glow.",
      lit: "Under the pointer or with the keyboard on it, here with the signs on: the cold tube inside its edge with the glow outside, in the picked line's place. By day the tube with no glow.",
      pressed: "Lit and pressed: the tube runs white-hot (`cold_core`), the glow `focus.pressed_glow` times the lit one; by day the cold tone tints the picture at `opacity.press_tint`. Letting go picks it.",
      disabled: "The picture at `opacity.disabled_content`, its name and chips dimmed; no line, no glow, no press, skipped by the keyboard.",
      loading: "Its placeholder, breathing in the picture's box, as `Media`'s; its name, chips and line as they are.",
    },
  },
  focus: {
    tubes: [
      { name: "Picked, at rest", tone: "accent", filled: false, content: false },
      { name: "Lit, picked or not", tone: "cold", filled: false, content: false },
    ],
    looks: {
      lit: "At night the signs burn: picked, the accent's tube inside the tile's edge at the control radius with the glow outside; lit, the cold tube in its place.",
      signsOff: "With the neon unpowered, by day always and at night while a power change has the lights out: the picked line is `primary_glass` and the lit one `cold_glass`, with no glow.",
    },
    signsOff: (index: number) => boxed(ROWS[index]!.options, "signs-off"),
  },
  content: {
    text: [
      { name: "The name under", words: NAME, sample: (words: string) => boxed({ source: photo, name: words, picked: true, mark: CURRENT }) },
      { name: "The name over", words: NAME, sample: (words: string) => boxed({ source: photo, name: words, place: "over" }) },
    ],
    missingPicture: [
      { name: "Not yet loaded", for: "The placeholder breathes in the picture's box; the grid doesn't move when it arrives.", sample: () => boxed({ name: "Ebisu", arrival: "loading" }) },
      { name: "Can't be read", for: "The placeholder held still; the name says which it is.", sample: () => boxed({ name: "Ebisu", arrival: "broken", place: "over" }) },
    ],
    counts: [
      { name: "One", for: "A set of one is no choice: the screen shows the picture as a `Media`, with no line.", sample: () => boxed({ source: photo, name: "Shinjuku, night", picked: true, mark: CURRENT }) },
      { name: "A row", for: "Three across.", sample: () => set("one", WALLPAPERS.slice(0, 3)) },
      { name: "Many", for: "Rows of three, one height each, a picture loading and one disabled among them.", sample: () => set("one", WALLPAPERS) },
    ],
  },
  motion: {
    arrives: doesntApply("a set comes with the surface it is on; a picture takes its placeholder's place as `Media` says."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      {
        name: "Picked",
        what: "A press, Enter or Space: the accent's line fades in over `short` on `standard`, its mark and its name with it, as the one that was picked fades back. Never the flicker.",
        tracks: [
          { part: "picked", on: "the picture picked", target: ".k-picture-choice:last-child", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" },
          { part: "picked", on: "the picture it unpicks", target: ".k-picture-choice:first-child", property: "--checked", from: 1, to: 0, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: pair(),
      },
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on it, it lights up over `short` on `standard`: the cold tube and glow fade in as a picked one's accent line fades out.", tracks: lights(".k-picture-tile", ON, 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-picture-tile", ON, 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "A lit picture pressed: over `micro` the tube runs white-hot, the glow brightens, and by day the cold tone tints it.", tracks: press(".k-picture-tile", ON), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Into the set or on out of it: the set is one stop, on its picked picture, else its first." },
      { keys: ["←", "→", "h", "l"], does: "To the picture before or after, in reading order. Moving never picks: a pick applies a wallpaper." },
      { keys: ["↑", "↓", "k", "j"], does: "To the picture above or below, a row of the grid at a time; a grid that scrolls brings the whole row into view." },
      { keys: ["Home", "End"], does: "To the first or the last picture." },
      { keys: ["Enter", "Space"], does: "Picks the picture the keyboard is on (one of a set), or turns it (any of it), once per press, never on key repeat." },
    ],
    order: "One stop in the surface's reading order. The arrows pass a disabled picture by and stop at the set's ends.",
    lands: "A picker opens on its picked picture. A pick keeps the keyboard where it is.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Name under, picked", for: "The picture's box, the mark `space.s8` in, and the name `space.s8` under it.", sample: () => boxed(ROWS[1]!.options) },
      { name: "Name over, with a badge", for: "The badge `space.s8` in, and the name `space.s12` in from the foot.", sample: () => boxed(ROWS[2]!.options) },
    ]),
    measures: [
      { what: "Its width in the pickers.", token: "size-thumbnail_width" },
      { what: "Its width to its height.", token: "aspect-wallpaper" },
      { what: "Its corners.", token: "radius-control" },
      { what: "The chips in from the corners, and the name under the picture.", token: "space-s8" },
      { what: "The name over the foot, in from the edges.", token: "space-s12" },
      { what: "The name's line.", token: "type-small-line" },
      { what: "The chips.", token: "size-chip" },
      { what: "Between two pictures across and down.", token: "space-s16" },
    ],
    reflow: {
      described: "Each picture is a thumbnail's width; a picker shows as many columns as its room holds, its rows one height, and scrolls a whole row at a time. Names end in an ellipsis where they don't fit.",
      sample: () => within("100%", set("one", WALLPAPERS, 3)),
    },
  },
});
