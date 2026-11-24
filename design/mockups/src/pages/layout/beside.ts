// The Beside page: `docs/design/components/beside.md`. A column with a picture beside it that stays
// in view while the column scrolls, or above it in a room too narrow for both: Settings › Camera's
// controls and its live preview.

import { beside } from "../../lib/beside.ts";
import { cameraPreview } from "../../lib/camera-preview.ts";
import { list } from "../../lib/list.ts";
import { within } from "../../lib/layout.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { DESKTOP, FULL_HD, MINIMUM, settingsWindow } from "../apps/settings/settings-window.ts";
import { CAMERA } from "../apps/settings/settings-data.ts";

/** Rooms the pair is shown in: under, about and over the switch point; the page content's widths. */
const ROOMS = {
  above: "calc(408 * var(--px))",
  beside: "calc(808 * var(--px))",
  wide: "calc(1592 * var(--px))",
} as const;

/** A well of rows standing for the controls. */
const rows = () =>
  list([
    { title: "Light behind you", subtitle: "Lifts your face when a window or lamp is behind you.", toggle: true },
    { title: "Automatic colour", subtitle: "Matches the colour of the room's light.", toggle: true },
    { title: "Brighter in dim light", subtitle: "Fewer frames a second when the room is dark.", toggle: false },
  ]);

const pair = (room: string) => within(room, beside(rows(), cameraPreview({ look: "lifted" })));

/** The Camera page in a window, its picture beside or above its controls. */
const camera = (size?: typeof MINIMUM) => () => settingsWindow({ section: CAMERA, values: { seed: { camera: { values: { integrated: { light: 1 } } } } } }, size);

type Shown = "rest";

export default page<Shown>({
  level: "layout",
  slug: "beside",
  title: "Beside",
  lede: "A column with a picture beside it, kept in view as the column scrolls, or above it when narrow.",
  code: { unbuilt: "Rust and QML: not built yet" },
  stage: { live: () => pair("100%"), hint: "pick a window size: the picture moves above", daylight: true },
  variants: [
    { name: "Beside", for: "the picture 40% of the room", sample: () => pair(ROOMS.beside) },
    { name: "Beside, wide", for: "the column at its most, 720", sample: () => pair(ROOMS.wide) },
    { name: "Above", for: "the column would be under 300", sample: () => pair(ROOMS.above) },
  ],
  sizes: [
    { name: "In a window, 720 × 480", for: "Settings › Camera: the picture above", sample: camera(MINIMUM) },
    { name: "In a window, 1400 × 900", for: "Settings › Camera: the picture beside", sample: camera(DESKTOP) },
    { name: "In a window, 1920 × 1080", for: "Settings › Camera: the controls at 720", sample: camera(FULL_HD) },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "it takes no pointer and no focus",
      pressed: "it takes no press",
      disabled: "what it holds is disabled on its own",
      selected: "nothing to pick",
      loading: "the picture's `Media` holds its box",
      empty: "a page with no picture draws no Beside",
      error: "it has no value to be wrong",
    },
    rows: [{ name: "Beside", cell: () => pair(ROOMS.beside) }],
    looks: { rest: "it draws nothing but what it holds" },
  },
  focus: doesntApply("it is no control: no line and no glow."),
  content: {
    text: doesntApply("it holds no words of its own."),
    missingPicture: doesntApply("the picture's `Media` keeps its box, so the column holds."),
    counts: doesntApply("one column and one picture, always."),
  },
  motion: {
    arrives: doesntApply("it comes with its page."),
    leaves: doesntApply("it goes with its page."),
    between: doesntApply("layout never animates: crossing the switch point moves the picture at once."),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "the column's stops; the picture takes none" }],
    order: "The column, in its own order; the picture has no stop.",
    lands: "Nowhere of its own: its page lands on its first row.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Beside", for: "the column, the gap and the picture", sample: () => pair(ROOMS.beside) },
      { name: "Above", for: "the picture over the column", sample: () => pair(ROOMS.above) },
    ]),
    measures: [
      { part: "The picture at its least", token: "size-beside_picture_least" },
      { part: "The picture at its most", token: "size-beside_picture_most" },
      { part: "The column at its most", token: "size-beside_controls_most" },
      { part: "The column at its least", token: "size-beside_controls_least" },
      { part: "Column to picture", token: "space-s24" },
      { part: "Picture over column", token: "space-s32" },
      { part: "Where the picture stays", token: "space-s32" },
    ],
    reflow: { note: "Beside from a 916 px window up; above it under that.", sample: () => pair("100%") },
  },
});
