// The CameraPreview page: `docs/design/components/camera-preview.md`. A camera's live picture
// beside the controls that set it, a drawn stand-in in the mockups; or, while another app has the
// camera, the same box saying who.

import { cameraPreview } from "../../lib/camera-preview.ts";
import { within } from "../../lib/layout.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The picture at its least, `size.beside_picture_least`, and at its most. */
const LEAST = "var(--size-beside_picture_least)";
const MOST = "var(--size-beside_picture_most)";

type Shown = "rest" | "disabled";

const lifted = (width = LEAST) => within(width, cameraPreview({ look: "lifted" }));
const busy = (by: string | null, width = LEAST) => within(width, cameraPreview({ look: "lifted", inUseBy: by }));

export default page<Shown>({
  level: "molecules",
  slug: "camera-preview",
  title: "CameraPreview",
  lede: "A camera's live picture, at `aspect.camera`, or a box saying which app has the camera.",
  code: { unbuilt: "Rust and QML: not built yet" },
  from: nonEmpty(["media", "icon", "text"]),
  stage: { live: () => lifted("var(--size-panel)"), hint: "nothing to press: it shows the camera", daylight: true },
  variants: [
    { name: "Face lifted", for: "Light behind you on", sample: () => lifted() },
    { name: "Backlit", for: "the camera's own picture", sample: () => within(LEAST, cameraPreview({ look: "backlit" })) },
    { name: "In use by Firefox", for: "another app has the camera", sample: () => busy("Firefox") },
    { name: "In use by another app", for: "the system can't tell which", sample: () => busy(null) },
  ],
  sizes: [
    { name: "Least", for: "`size.beside_picture_least`, 280", sample: () => lifted(LEAST) },
    { name: "Most", for: "`size.beside_picture_most`, 640, or more on a wide page", sample: () => lifted(MOST) },
  ],
  states: {
    shown: ["rest", "disabled"],
    absent: {
      lit: "it takes no pointer or keyboard",
      pressed: "it takes no press",
      selected: "nothing to pick",
      loading: "the `Media` skeleton holds its box",
      empty: "no camera: the page says so instead",
      error: "a camera that fails is not connected",
    },
    rows: [{ name: "The preview", cell: (state: Shown) => (state === "rest" ? lifted() : busy("Firefox")) }],
    looks: {
      rest: "the picture, edge to edge, content radius",
      disabled: "in use: the well's fill, symbol and one line",
    },
  },
  focus: doesntApply("it takes no keyboard: Tab passes it."),
  content: {
    text: [{ name: "Who has it", words: { short: "In use by Zoom", long: "In use by Microsoft Teams (work or school)" }, sample: (words: string) => within(LEAST, cameraPreview({ look: "lifted", inUseBy: words.replace(/^In use by /, "") })) }],
    missingPicture: doesntApply("the `Media` skeleton holds the box until the picture comes."),
    counts: doesntApply("one camera's picture at a time."),
  },
  motion: {
    arrives: doesntApply("it comes with its page."),
    leaves: doesntApply("it goes with its page."),
    between: doesntApply("the picture changes as the camera's does, live."),
  },
  keyboard: doesntApply("it takes no keyboard: Tab passes it."),
  measures: {
    samples: nonEmpty([
      { name: "The picture", for: "its box at the camera's shape", sample: () => lifted() },
      { name: "In use", for: "the box, symbol and line", sample: () => busy("Firefox") },
    ]),
    measures: [
      { part: "The picture's shape", token: "aspect-camera" },
      { part: "Corner", token: "radius-content" },
      { part: "Symbol", token: "icon-s32" },
      { part: "Symbol to line", token: "space-s12" },
      { part: "Line's room at the sides", token: "space-s16" },
    ],
    reflow: { note: "As wide as its room, the camera's shape tall, down to the grid.", sample: () => within("100%", cameraPreview({ look: "lifted" })) },
  },
});
