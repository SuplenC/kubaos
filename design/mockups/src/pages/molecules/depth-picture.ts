// The DepthPicture page: `docs/design/components/depth-picture.md`. A wallpaper with its depth: the
// picture, its plate and its city, which pushes in faster than the sky. The Lock draws it with its
// clock between the plate and the city; the wallpaper picker's wall draws it on every picture.

import { depthPicture, type DepthPictureOptions } from "../../lib/depth-picture.ts";
import { row } from "../../lib/layout.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { PURPLE_HOUR, SHIBUYA, TOWER } from "../templates/locks.ts";

/** A sample's box: 16:10, as the screens are, a wallpaper's thumbnail's size twice over. */
const SIZE = { width: "calc(2 * var(--size-thumbnail_width))", height: "calc(2 * var(--size-thumbnail_height))" };

type Picked = Omit<DepthPictureOptions, "width" | "height" | "picture">;

function drawn(wallpaper = PURPLE_HOUR, options: Picked = {}): HTMLElement {
  return depthPicture({ picture: wallpaper.picture, ...(wallpaper.depth ? { depth: wallpaper.depth } : {}), ...SIZE, ...options });
}

/** Live: a click pushes it in and back, D turns its depth off and on, each easing over `long`. */
function live(): HTMLElement {
  const box = drawn(PURPLE_HOUR, { push: 1 });
  box.tabIndex = 0;
  box.style.transition = "--push var(--motion-long) var(--easing-emphasized_decel), --depth var(--motion-long) var(--easing-emphasized_decel)";
  box.addEventListener("click", () => box.style.setProperty("--push", box.style.getPropertyValue("--push") === "1" ? "0" : "1"));
  box.addEventListener("keydown", (event) => {
    if (event.key !== "d" && event.key !== "D") return;
    const on = box.style.getPropertyValue("--depth") !== "0";
    box.removeAttribute("data-flat");
    box.style.setProperty("--depth", on ? "0" : "1");
    if (on) box.addEventListener("transitionend", () => box.toggleAttribute("data-flat", box.style.getPropertyValue("--depth") === "0"), { once: true });
  });
  return box;
}

const ON = "the picture";

const PUSHES: Move = {
  name: "Pushing in",
  what: "The picture and plate grow to `scale.push`, the city to `scale.near`, round the bottom middle.",
  tracks: [{ part: "the push", on: ON, target: ".k-depth-picture", property: "--push", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the push", on: ON, target: ".k-depth-picture", property: "--push", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => drawn(PURPLE_HOUR, { push: 0 }),
};

const FLATTENS: Move = {
  name: "Depth off",
  what: "The city settles onto the sky and the plate goes with it; reduced motion swaps at once.",
  tracks: [{ part: "the depth", on: ON, target: ".k-depth-picture", property: "--depth", from: 1, to: 0, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the depth", on: ON, target: ".k-depth-picture", property: "--depth", from: 1, to: 0, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: () => drawn(TOWER, { push: 1 }),
};

const WALKS: Move = {
  name: "Sliding past",
  what: "Browsing: the city starts `space.s24` ahead of its sky and catches up as the picture lands.",
  tracks: [{ part: "the city's lead", on: ON, target: ".k-depth-picture", property: "--city-lead", from: { length: "space-s24" }, to: 0, duration: "long", curve: "spring" }],
  reduced: [{ part: "the city's lead", on: ON, target: ".k-depth-picture", property: "--city-lead", from: 0, to: 0, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: () => drawn(PURPLE_HOUR, { push: 1 }),
};

export default page({
  level: "molecules",
  slug: "depth-picture",
  title: "DepthPicture",
  lede: "A wallpaper with its depth: the picture, its plate and its city, which pushes in faster than the sky.",
  code: { unbuilt: "QML DepthPicture: not built yet" },
  stage: { live, hint: "click to push in, D for depth", daylight: true },
  variants: [
    { name: "Pushed in", for: "the city off the sky: the lock, the wall", sample: () => drawn(PURPLE_HOUR, { push: 1 }) },
    { name: "At 1:1", for: "the desktop's picture, exactly", sample: () => drawn(PURPLE_HOUR, { push: 0 }) },
    { name: "A spire", for: "Tokyo Tower: one tall city", sample: () => drawn(TOWER, { push: 1 }) },
    { name: "Depth off", for: "pushed in, the city flat on the sky", sample: () => drawn(TOWER, { push: 1, flat: true }) },
    { name: "No cut-out", for: "a picture of your own: flat", sample: () => drawn(SHIBUYA, { push: 1 }) },
  ],
  sizes: doesntApply("the size of the box it fills, the picture as `cover` lays it."),
  states: {
    shown: ["rest", "disabled"],
    absent: {
      lit: "it takes no pointer or keyboard",
      pressed: "it takes no press",
      selected: "it holds no choice",
      loading: "its box's surface until the picture arrives",
      empty: "it always holds a picture",
      error: "a picture it can't read is flat",
    },
    rows: nonEmpty([
      { name: "Shinjuku", cell: (state: "rest" | "disabled") => drawn(PURPLE_HOUR, { push: 1, flat: state === "disabled" }) },
      { name: "Tokyo Tower", cell: (state: "rest" | "disabled") => drawn(TOWER, { push: 1, flat: state === "disabled" }) },
    ]),
    looks: { rest: "depth on: the city off the sky", disabled: "depth off: the picture alone" },
  },
  focus: doesntApply("not a control: it has no line."),
  content: {
    text: doesntApply("no words."),
    missingPicture: [{ name: "Not arrived", for: "the box's surface, then the picture", sample: () => drawn({ ...SHIBUYA, picture: "" }) }],
    counts: doesntApply("one picture a box."),
  },
  motion: { arrives: doesntApply("it is drawn as its wallpaper is."), leaves: doesntApply("it goes with its wallpaper."), between: [PUSHES, FLATTENS, WALKS] },
  keyboard: doesntApply("it takes no keys; what holds it does."),
  measures: {
    samples: nonEmpty([{ name: "Pushed in", for: "the push and the city's lead", sample: () => row("s16", "start", drawn(PURPLE_HOUR, { push: 1 }), drawn(PURPLE_HOUR, { push: 0 })) }]),
    measures: [
      { part: "Picture and plate, pushed", token: "scale-push" },
      { part: "City, pushed", token: "scale-near" },
      { part: "City's lead, sliding past", token: "space-s24" },
    ],
    reflow: doesntApply("it fills its box at any size."),
  },
});
