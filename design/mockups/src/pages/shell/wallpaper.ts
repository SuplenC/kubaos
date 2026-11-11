// The wallpaper picker (`docs/design/components/wallpaper-picker.md`, `modules/wallpaper/`, not
// built yet): a gallery. Every picture of a folder hangs on a wall, a floor per folder, a neon lamp
// over each; each is drawn as the picture, your desktop on it, or your lock, in its own colours;
// the one in front of you steps out to the full screen, and the live desktop is that one stepped
// out, so opening steps back into the wall and applying or cancelling steps out again.

import { doesntApply, type Duration, type Easing, type Move, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { locking, lockingReduced, touching, touchingReduced, waking, wakingReduced } from "../templates/lock-moves.ts";
import { FOLDERS, gallery, type GalleryOptions, KABUKICHO, pictureToLock, sideToDesktop, sideToLock, toDesktop, toDesktopReduced, toPicture, toPictureReduced, type View } from "./wallpaper-parts.ts";

type Shown = "rest" | "lit" | "open" | "disabled" | "loading" | "empty";

/** The gallery drawn still. */
const still = (options: GalleryOptions = {}) => () => gallery(options);

/** The gallery drawn still in a box, so a move can reach its root. */
const held = (options: GalleryOptions = {}) => () => {
  const box = document.createElement("span");
  box.style.display = "block";
  box.append(gallery(options));
  return box;
};

/** The front picture's frame, and every other picture's, as the moves' targets name them. */
const FRONT = ".gl-art[data-on]";
const SIDES = ".gl-art:not([data-on])";
const MAIN = "the picture in front of you";
const OTHERS = "the pictures either side";
const ROOM = "the gallery";

/** Every track of `tracks` a phase later: after `phase`, then its own cue. */
function after(phase: Duration, tracks: readonly Track<Easing>[]): Track<Easing>[] {
  return tracks.map((track) => {
    const own: readonly Duration[] = track.cue === undefined ? [] : typeof track.cue === "string" ? [track.cue] : track.cue;
    return { ...track, cue: [phase, ...own] as unknown as NonEmpty<Duration> };
  });
}

const stepOut = (from: number, to: number, duration: Duration): Track<Easing> => ({ part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from, to, duration, curve: "emphasized_decel" });
const barFades = (from: number, to: number, duration: Duration, curve: Easing): Track<Easing> => ({ part: "the bar", on: ROOM, target: ".gl-bar, .gl-info", property: "opacity", from, to, duration, curve });

const OPENING: Move = {
  name: "Opening",
  what: "Super+Ctrl+Space: the desktop steps back into its frame as the wall builds round it.",
  tracks: [stepOut(1, 0, "slow"), ...toPicture(MAIN, FRONT), ...toPicture(OTHERS, SIDES, false), barFades(0, 1, "long", "emphasized_decel")],
  reduced: [
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    ...after("short", toPictureReduced(MAIN, FRONT)),
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    barFades(0, 1, "short", "standard"),
  ],
  back: "cuts",
  sample: held({ open: false }),
};

const APPLYING: Move = {
  name: "Applying",
  what: "Enter: the picture in front of you grows into the desktop as the wall goes.",
  tracks: [stepOut(0, 1, "slow"), ...toDesktop(MAIN, FRONT), ...toDesktop(OTHERS, SIDES, false), barFades(1, 0, "medium", "emphasized_accel")],
  reduced: [
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from: 0, to: 1, duration: "instant", cue: "short", curve: "standard" },
    ...after("short", toDesktopReduced(MAIN, FRONT)),
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    barFades(1, 0, "short", "standard"),
  ],
  back: "cuts",
  sample: held({ at: 1 }),
};

const CANCELLING: Move = {
  name: "Cancelling",
  what: "Esc: the wall walks back to the picture you had, then it grows into the desktop.",
  tracks: [
    { part: "the wall", on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--at", from: 1, to: 0, duration: "long", curve: "spring" },
    ...after("long", [stepOut(0, 1, "slow"), ...toDesktop(MAIN, ".gl-floor:first-child .gl-art:first-child"), barFades(1, 0, "medium", "emphasized_accel")]),
  ],
  reduced: [
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the wall", on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--at", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    { part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from: 0, to: 1, duration: "instant", cue: "short", curve: "standard" },
    ...after("short", toDesktopReduced(MAIN, ".gl-floor:first-child .gl-art:first-child")),
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    barFades(1, 0, "short", "standard"),
  ],
  back: "cuts",
  sample: held({ at: 1 }),
};

const TO_DESKTOP: Move = {
  name: "Picture to desktop",
  what: "2: every picture becomes its desktop; the one in front eases back to 1:1.",
  tracks: nonEmpty([...toDesktop(MAIN, FRONT), ...toDesktop(OTHERS, SIDES, false)]),
  reduced: nonEmpty([...toDesktopReduced(MAIN, FRONT), ...toDesktopReduced(OTHERS, SIDES)]),
  back: "cuts",
  sample: held({ bare: true, at: 2, view: "picture" }),
};

const LOCKS: Move = {
  name: "Desktop to lock",
  what: "3: the Lock's own locking in front, the bar's time growing into the clock behind the city.",
  tracks: [...locking(MAIN, FRONT), ...sideToLock(OTHERS, SIDES)],
  reduced: [...lockingReduced(MAIN, FRONT), ...sideToLock(OTHERS, SIDES, true)],
  back: "cuts",
  sample: held({ bare: true, at: 2, view: "desktop" }),
};

const UNLOCKS: Move = {
  name: "Lock to desktop",
  what: "2: the Lock's own unlocking in front, the clock flying into the bar's time.",
  tracks: [...touching(MAIN, FRONT), ...sideToDesktop(OTHERS, SIDES)],
  reduced: [...touchingReduced(MAIN, FRONT), ...sideToDesktop(OTHERS, SIDES, true)],
  back: "cuts",
  sample: held({ bare: true, at: 2, view: "lock" }),
};

const TO_LOCK: Move = {
  name: "Picture to lock",
  what: "3 from the picture: every clock fades in behind its city.",
  tracks: nonEmpty([...pictureToLock(MAIN, FRONT), ...pictureToLock(OTHERS, SIDES, false)]),
  reduced: nonEmpty([...pictureToLock(MAIN, FRONT, true, true), ...pictureToLock(OTHERS, SIDES, false, true)]),
  back: "cuts",
  sample: held({ bare: true, at: 2, view: "picture" }),
};

const WAKES: Move = {
  name: "Waking the lock",
  what: "W: the Lock's own waking, in the frame or on the full screen.",
  tracks: waking(MAIN, FRONT),
  reduced: wakingReduced(MAIN, FRONT),
  back: "cuts",
  sample: held({ bare: true, at: 2, view: "lock" }),
};

const FULL: Move = {
  name: "Full screen",
  what: "F: the room grows round the picture in front of you until it is the screen.",
  tracks: [stepOut(0, 1, "slow")],
  reduced: [
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from: 0, to: 1, duration: "instant", cue: "short", curve: "standard" },
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: held({ bare: true, at: 2, view: "desktop" }),
};

const WALL = "the wall";
const OLD = ".gl-floor:first-child .gl-art:nth-child(2)";
const NEW = ".gl-floor:first-child .gl-art:nth-child(3)";

const BROWSES: Move = {
  name: "Browsing",
  what: "→: the wall glides a step, each city ahead of its sky; the one reached lifts.",
  tracks: [
    { part: WALL, on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--at", from: 1, to: 2, duration: "long", curve: "spring" },
    { part: "its city's lead", on: "the picture reached", target: `${NEW} .k-lock`, property: "--aside", from: 1, to: 0, duration: "long", curve: "spring" },
    { part: "its city's lead", on: "the picture left", target: `${OLD} .k-lock`, property: "--aside", from: 0, to: -1, duration: "long", curve: "spring" },
    { part: "the push", on: "the picture reached", target: `${NEW} .k-lock`, property: "--push", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the push", on: "the picture left", target: `${OLD} .k-lock`, property: "--push", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "its frame", on: "the picture reached", target: `${NEW} .gl-frame`, property: "scale", from: { scale: "behind" }, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "its frame", on: "the picture left", target: `${OLD} .gl-frame`, property: "scale", from: 1, to: { scale: "behind" }, duration: "long", curve: "emphasized_decel" },
    { part: "its lamp", on: "the picture reached", target: `${NEW} .gl-lamp`, property: "--lamp", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "its lamp", on: "the picture left", target: `${OLD} .gl-lamp`, property: "--lamp", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  reduced: [{ part: WALL, on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--at", from: 1, to: 2, duration: "instant", curve: "standard" }],
  back: "cuts",
  sample: held({ bare: true, at: 1 }),
};

const FLATTENS: Move = {
  name: "Depth off",
  what: "D: the city settles onto the sky and its plate goes; on a lock the clock fades round it.",
  tracks: [{ part: "the depth", on: MAIN, target: `${FRONT} .k-lock`, property: "--depth", from: 1, to: 0, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: "the depth", on: MAIN, target: `${FRONT} .k-lock`, property: "--depth", from: 1, to: 0, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: held({ bare: true, at: 2 }),
};

/** A view's gallery in `state`. */
function cell(view: View) {
  return (state: Shown): HTMLElement => {
    switch (state) {
      case "rest":
        return gallery({ at: 2, view });
      case "lit":
        return gallery({ at: 2, view, tip: view === "picture" ? "Desktop" : view === "desktop" ? "Lock" : "Wake" });
      case "open":
        return gallery({ at: 2, view, shortcuts: true });
      case "disabled":
        return gallery({ at: 4, view });
      case "loading":
        return gallery({ at: 2, view, stills: false });
      case "empty":
        return gallery({ folder: 2, view: "picture" });
    }
  };
}

/** The live gallery, shut: the desktop, which a click or Space opens. */
function live(): HTMLElement {
  return gallery({ open: false, live: true });
}

/** The front picture with a name at the toolbar's length. */
const named = (words: string) => gallery({ at: 0, name: words });

export default page<Shown>({
  level: "shell",
  slug: "wallpaper",
  title: "Wallpaper picker",
  lede: "A gallery: every picture on a wall, each drawn as itself, your desktop or your lock, the one in front lit.",
  code: { unbuilt: "QML WallpaperPicker: not built yet" },
  from: ["lock", "depth-picture", "tabs", "button", "button-bar", "icon-button", "chip", "switch", "list", "empty-state", "shortcuts", "tooltip"],
  stage: { live, hint: "click it, then the keys: ? lists them", daylight: true },
  variants: [
    { name: "Picture", for: "the wall: each picture alone, the front one lifted", sample: still({ at: 1 }) },
    { name: "Desktop", for: "each its desktop, in its own colours", sample: still({ at: 1, view: "desktop" }) },
    { name: "Lock", for: "each its lock, the clock behind the city", sample: still({ at: 2, view: "lock" }) },
    { name: "Full screen, desktop", for: "the front one fills the screen", sample: still({ at: 2, view: "desktop", full: true }) },
    { name: "Full screen, lock awake", for: "the password's sheet up, the bar lifted", sample: still({ at: 2, view: "lock", full: true, awake: true }) },
    { name: "A video", for: "its loop breathes over its poster", sample: still({ at: 3 }) },
    { name: "Depth off", for: "the tower flat on its sky", sample: still({ at: 2, view: "lock", flat: [FOLDERS[0]!.pictures[2]!.name] }) },
    { name: "Shut", for: "the live desktop", sample: still({ open: false }) },
  ],
  sizes: doesntApply("as large as its screen; the wall is shares of it (`[scale]`)."),
  states: {
    shown: ["rest", "lit", "open", "disabled", "loading", "empty"],
    absent: {
      pressed: "its buttons press as their own pages say",
      selected: "the applied picture is marked Current at rest",
      error: "a picture it can't read isn't hung",
    },
    rows: nonEmpty((["picture", "desktop", "lock"] as const).map((view) => ({ name: view === "picture" ? "Picture" : view === "desktop" ? "Desktop" : "Lock", cell: cell(view) }))),
    looks: {
      rest: "the front one lit, its lamp burning",
      lit: "a control's name and key over it",
      open: "the shortcuts over their button",
      disabled: "no cut-out: depth's switch off, with why",
      loading: "the windows' stills not landed yet",
      empty: "an `EmptyState` in the frame, where to drop them",
    },
  },
  focus: {
    tubes: [
      { name: "The lamp over the front picture", tone: "accent", filled: true, content: false },
      { name: "Its controls", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the lamp's core burns, its glow on the wall", signsOff: "the lamp as glass: no glow" },
    signsOff: () => gallery({ at: 2, signsOff: true }),
  },
  content: {
    text: [{ name: "A picture's name", words: { short: "Kabukicho rain", long: "Kabukicho rain after the last train home" }, sample: named }],
    missingPicture: [
      {
        name: "Not arrived",
        for: "`surface_container_lowest`, then the picture",
        sample: still({ folders: [{ name: "Mine", icon: "person", pictures: [{ ...KABUKICHO, name: "Not arrived", wallpaper: { ...KABUKICHO.wallpaper, picture: "" } }, KABUKICHO] }], folder: 0, at: 0 }),
      },
    ],
    counts: [
      { name: "None", for: "Downloads: an `EmptyState` in the frame", sample: still({ folder: 2 }) },
      { name: "One", for: "Mine: no neighbours", sample: still({ folder: 1, at: 0 }) },
      { name: "Many", for: "KubaOS: five, two either side in view", sample: still({ at: 2 }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [APPLYING, CANCELLING],
    between: [BROWSES, TO_DESKTOP, LOCKS, UNLOCKS, TO_LOCK, WAKES, FULL, FLATTENS],
  },
  keyboard: {
    keys: [
      { keys: ["Super+Ctrl+Space"], does: "opens or shuts it" },
      { keys: ["←", "→"], does: "the picture before or after, in every view" },
      { keys: ["↑", "↓"], does: "the floor above or below" },
      { keys: ["1", "2", "3"], does: "Picture, Desktop, Lock" },
      { keys: ["F", "Space"], does: "full screen, and back to the wall" },
      { keys: ["Click"], does: "on the front picture: full screen; there, back to the wall" },
      { keys: ["D"], does: "depth on or off, on a picture with it" },
      { keys: ["W"], does: "in Lock: wakes it, or lets it rest" },
      { keys: ["Enter"], does: "applies the one in front" },
      { keys: ["Esc"], does: "the shortcuts first; else cancels, the old one back" },
      { keys: ["?"], does: "the shortcuts" },
    ],
    order: "Tab: the shortcuts' button, the views, Full screen, Cancel, Wake, Apply.",
    lands: "Opening, on the gallery itself; shutting, back where it was.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The wall", for: "pictures, gaps, lamps, placard, floors", sample: still({ at: 2 }) },
      { name: "The bar, full screen", for: "the bar's parts and the name", sample: still({ at: 2, view: "desktop", full: true }) },
    ]),
    measures: [
      { part: "Picture's height", token: "scale-gallery_picture" },
      { part: "Pictures' top edge", token: "scale-gallery_top" },
      { part: "Between pictures", token: "scale-gallery_gap" },
      { part: "Lamp's width", token: "scale-gallery_lamp" },
      { part: "Lamp's height", token: "space-s4" },
      { part: "Lamp over its picture", token: "space-s32" },
      { part: "Wall's foot", token: "scale-gallery_floor" },
      { part: "Picture → placard", token: "space-s24" },
      { part: "Frame's corner", token: "radius-surface" },
      { part: "A picture stepped back", token: "scale-behind" },
      { part: "City ahead, browsing", token: "space-s24" },
      { part: "Floors' inset", token: "space-s32" },
      { part: "Bar's padding", token: "space-s8" },
      { part: "Bar's gaps", token: "space-s16" },
      { part: "Bar, shortcuts → foot", token: "space-s24" },
      { part: "Tooltip over its control", token: "space-s8" },
    ],
    reflow: { note: "The wall is shares of the screen: the same shape on every screen.", sample: () => gallery({ at: 2, size: windowSize() }) },
  },
});
