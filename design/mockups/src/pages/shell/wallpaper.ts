// The wallpaper picker (`docs/design/components/wallpaper-picker.md`, `modules/wallpaper/`, not
// built yet): a gallery. Every picture of a folder hangs on a wall, a floor per folder, a neon lamp
// over each; each is drawn as the picture, your desktop on it, or your lock, in its own colours;
// the one in front of you steps out to the full screen, and the live desktop is that one stepped
// out, so opening steps back into the wall and applying or cancelling steps out again.

import { doesntApply, type Duration, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Track } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";
import { locking, lockingReduced, touching, touchingReduced, waking, wakingReduced } from "../templates/lock-moves.ts";
import { type ClockLook, DIMMED, type EditorPose } from "./wallpaper-clock.ts";
import { BLUE, FOLDERS, gallery, type GalleryOptions, KABUKICHO, pictureToLock, sideToDesktop, sideToLock, toDesktop, toDesktopReduced, toPicture, toPictureReduced, type View } from "./wallpaper-parts.ts";

type Shown = "rest" | "lit" | "open" | "disabled" | "loading" | "empty";

/** The clock's editor on the tower, drawn in `pose`. */
const editing = (pose: EditorPose = {}, options: GalleryOptions = {}) => screen(() => gallery({ at: 2, view: "lock", edit: pose, ...options }));
/** The clock's editor ready to move from `from` to `to`, for a move's player. */
const editMove = (from: EditorPose | "wall", to: EditorPose | "wall") => held({ at: 2, view: "lock", edit: { from, to } });
/** The tower's clock set by `look`, on the wall. */
const setOnTower = (look: Partial<ClockLook>) => ({ [BLUE.name]: look });
const CLOCK_OFF: Partial<ClockLook> = { shown: { wide: false } };
const DATE_OFF: Partial<ClockLook> = { dated: { wide: false } };
const MOVED: Partial<ClockLook> = { place: { wide: { x: 0.3, y: 0.82, s: 0.2 } }, date: { wide: { size: 0.08, gap: 0.12, align: "left" } }, colour: { mode: "custom", h: 40, l: 0.8 }, glow: { mode: "dark", strength: 0.6 } };

/** The gallery drawn still. */
const still = (options: GalleryOptions = {}) => screen(() => gallery(options));

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

/** The room's light, `--room-lit`: the lamp over the front picture, its light and the room's colours. */
const LIGHT = "the light";
const lightOff = (cue?: Duration): Track<Easing> => ({ part: LIGHT, on: ROOM, target: ".gl-screen", property: "--room-lit", from: 1, to: 0, duration: "short", ...(cue ? { cue } : {}), curve: "emphasized_accel" });
const lightOffCalm = (cue?: Duration): Track<Easing> => ({ part: LIGHT, on: ROOM, target: ".gl-screen", property: "--room-lit", from: 1, to: 0, duration: "short", ...(cue ? { cue } : {}), curve: "standard" });
/** The lamp strikes on: the sign's flicker over `strike`; its steps as fades with Steady. */
const lightOn = (cue: Duration | NonEmpty<Duration>, steady = false): Track => ({ part: LIGHT, on: ROOM, target: ".gl-screen", property: "--room-lit", from: 0, to: 1, duration: "strike", cue, curve: steady ? "linear" : "neon.strike" });
const lightOnCalm = (cue: Duration | NonEmpty<Duration>): Track<Easing> => ({ part: LIGHT, on: ROOM, target: ".gl-screen", property: "--room-lit", from: 0, to: 1, duration: "short", cue, curve: "standard" });

const stepOut = (from: number, to: number, duration: Duration): Track<Easing> => ({ part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from, to, duration, curve: "emphasized_decel" });
const barFades = (from: number, to: number, duration: Duration, curve: Easing): Track<Easing> => ({ part: "the bar", on: ROOM, target: ".gl-bar, .gl-info", property: "opacity", from, to, duration, curve });

const OPENING: Move = {
  name: "Opening",
  what: "Super+Ctrl+Space: the desktop steps back into its dark room; its lamp strikes as it lands.",
  tracks: [stepOut(1, 0, "slow"), ...toPicture(MAIN, FRONT), ...toPicture(OTHERS, SIDES, false), barFades(0, 1, "long", "emphasized_decel"), lightOn("slow")],
  unflickered: [stepOut(1, 0, "slow"), ...toPicture(MAIN, FRONT), ...toPicture(OTHERS, SIDES, false), barFades(0, 1, "long", "emphasized_decel"), lightOn("slow", true) as Track<Easing>],
  reduced: [
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "stepping out", on: ROOM, target: ".gl-screen", property: "--out", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
    ...after("short", toPictureReduced(MAIN, FRONT)),
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
    barFades(0, 1, "short", "standard"),
    lightOnCalm("short"),
  ],
  back: "cuts",
  sample: held({ open: false }),
};

const APPLYING: Move = {
  name: "Applying",
  what: "Enter: the lamp goes off as the picture in front of you grows into the desktop.",
  tracks: [lightOff(), stepOut(0, 1, "slow"), ...toDesktop(MAIN, FRONT), ...toDesktop(OTHERS, SIDES, false), barFades(1, 0, "medium", "emphasized_accel")],
  reduced: [
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    lightOffCalm(),
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
  what: "Esc: the lamp goes off, the wall walks back in the dark, the old picture grows into the desktop.",
  tracks: [
    lightOff(),
    { part: "the wall", on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--gl-at", from: 1, to: 0, duration: "long", curve: "spring" },
    ...after("long", [stepOut(0, 1, "slow"), ...toDesktop(MAIN, ".gl-floor:first-child .gl-art:first-child"), barFades(1, 0, "medium", "emphasized_accel")]),
  ],
  reduced: [
    lightOffCalm(),
    { part: "the room", on: ROOM, target: ".gl-room", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the wall", on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--gl-at", from: 1, to: 0, duration: "instant", cue: "short", curve: "standard" },
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

// The clock's editor: one number moves the picture's frame, its shape and its clock, another the
// panel, another takes the wall away; with reduce motion each fades out and in round the change.
const STAGE = "the picture and its clock";
const PANEL = "the clock's panel";
const EDITOR = "the clock's editor";
const morphs = (duration: Duration = "long"): Track<Easing> => ({ part: STAGE, on: EDITOR, target: ".ce-stage", property: "--ce-morph", from: 0, to: 1, duration, curve: "emphasized_decel" });
const slides: Track<Easing> = { part: PANEL, on: EDITOR, target: ".ce-panel", property: "--ce-slide", from: 0, to: 1, duration: "long", curve: "emphasized_decel" };
const dims = (from: number, to: number): Track<Easing> => ({ part: "the wall round it", on: ROOM, target: DIMMED, property: "--ce-dim", from, to, duration: "long", curve: "standard" });
/** What reduce motion makes of a move of `part` on `target`: out over `short`, the change, back in. */
const faded = (part: string, target: string, property: string): Track<Easing>[] => [
  { part, on: EDITOR, target, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" },
  { part, on: EDITOR, target, property, from: 0, to: 1, duration: "instant", cue: "short", curve: "standard" },
  { part, on: EDITOR, target, property: "opacity", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
];
const BOX: Track<Easing> = { part: "the clock's box", on: EDITOR, target: ".ce-box", property: "opacity", from: 0, to: 1, duration: "medium", curve: "standard" };

const EDIT_OPENS: Move = {
  name: "Clock editor opening",
  what: "E: the frame moves over beside the panel, unpushed; the panel slides in; the wall goes.",
  tracks: [morphs(), slides, dims(0, 1), BOX],
  reduced: nonEmpty([...faded(STAGE, ".ce-stage", "--ce-morph"), ...faded(PANEL, ".ce-panel", "--ce-slide"), { ...dims(0, 1), duration: "short" }]),
  back: "cuts",
  sample: editMove("wall", {}),
};

const EDIT_CLOSES: Move = {
  name: "Done or Cancel",
  what: "↵ or Esc: the same move back, the panel sliding out, the push and the wall returning.",
  tracks: [morphs(), slides, dims(1, 0), { ...BOX, from: 1, to: 0 }],
  reduced: nonEmpty([...faded(STAGE, ".ce-stage", "--ce-morph"), ...faded(PANEL, ".ce-panel", "--ce-slide"), { ...dims(1, 0), duration: "short" }]),
  back: "cuts",
  sample: editMove({}, "wall"),
};

const SHAPE: Move = {
  name: "Wide to portrait",
  what: "One morph of the frame, the picture's crop and the clock; each shape keeps its own values.",
  tracks: [morphs()],
  reduced: nonEmpty(faded(STAGE, ".ce-stage", "--ce-morph")),
  back: "plays",
  sample: editMove({}, { tab: "portrait" }),
};

const EDIT_FULL: Move = {
  name: "Editor full screen",
  what: "F: the frame fills the screen; the panel goes to its edge as a closed drawer.",
  tracks: [morphs(), slides],
  reduced: nonEmpty([...faded(STAGE, ".ce-stage", "--ce-morph"), ...faded(PANEL, ".ce-panel", "--ce-slide")]),
  back: "plays",
  sample: editMove({}, { full: true, drawer: "closed" }),
};

const DRAWER: Move = {
  name: "Drawer opening",
  what: "The edge under the pointer, or the panel's keys: it slides out, cut flush by the edge.",
  tracks: [slides],
  reduced: nonEmpty(faded(PANEL, ".ce-panel", "--ce-slide")),
  back: "plays",
  sample: editMove({ full: true, drawer: "closed" }, { full: true, drawer: "open" }),
};

const DOCKS: Move = {
  name: "Rip off and dock",
  what: "Past the pull it rips free, catching the pointer over medium; let go, it settles.",
  tracks: [{ ...slides, duration: "medium", curve: "standard" }],
  reduced: [{ ...slides, duration: "instant", curve: "standard" }],
  back: "plays",
  sample: editMove({ full: true, drawer: "open" }, { full: true, drawer: "float" }),
};

const RESETS: Move = {
  name: "Standard place",
  what: "R: the clock goes back to the lock's own place, size, look and date in one move.",
  tracks: [morphs()],
  reduced: nonEmpty(faded(STAGE, ".ce-stage", "--ce-morph")),
  back: "plays",
  sample: held({ at: 2, view: "lock", looks: setOnTower(MOVED), edit: { from: { look: MOVED }, to: { look: { place: {}, date: {}, colour: { mode: "auto" }, glow: { mode: "off", strength: 0.5 } } } } }),
};

const WALL = "the wall";
const OLD = ".gl-floor:first-child .gl-art:nth-child(2)";
const NEW = ".gl-floor:first-child .gl-art:nth-child(3)";

const OLD_LAMP: Track<Easing> = { part: "its lamp", on: "the picture left", target: `${OLD} .gl-lamp`, property: "--lamp", from: 1, to: 0, duration: "short", curve: "emphasized_accel" };
const newLamp = (steady: boolean): Track => ({ part: "its lamp", on: "the picture reached", target: `${NEW} .gl-lamp`, property: "--lamp", from: 0, to: 1, duration: "strike", cue: "long", curve: steady ? "linear" : "neon.strike" });

const BROWSES: Move = {
  name: "Browsing",
  what: "→: lights off, the wall glides a step in the dark, the new lamp strikes in its colour.",
  tracks: [
    lightOff(),
    OLD_LAMP,
    { part: WALL, on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--gl-at", from: 1, to: 2, duration: "long", curve: "spring" },
    { part: "its city's lead", on: "the picture reached", target: `${NEW} .k-lock`, property: "--aside", from: 1, to: 0, duration: "long", curve: "spring" },
    { part: "its city's lead", on: "the picture left", target: `${OLD} .k-lock`, property: "--aside", from: 0, to: -1, duration: "long", curve: "spring" },
    { part: "the push", on: "the picture reached", target: `${NEW} .k-lock`, property: "--push", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the push", on: "the picture left", target: `${OLD} .k-lock`, property: "--push", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "its frame", on: "the picture reached", target: `${NEW} .gl-frame`, property: "scale", from: { scale: "behind" }, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "its frame", on: "the picture left", target: `${OLD} .gl-frame`, property: "scale", from: 1, to: { scale: "behind" }, duration: "long", curve: "emphasized_decel" },
    lightOn("long"),
    newLamp(false),
  ],
  unflickered: [
    lightOff(),
    OLD_LAMP,
    { part: WALL, on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--gl-at", from: 1, to: 2, duration: "long", curve: "spring" },
    { part: "its city's lead", on: "the picture reached", target: `${NEW} .k-lock`, property: "--aside", from: 1, to: 0, duration: "long", curve: "spring" },
    { part: "its city's lead", on: "the picture left", target: `${OLD} .k-lock`, property: "--aside", from: 0, to: -1, duration: "long", curve: "spring" },
    { part: "the push", on: "the picture reached", target: `${NEW} .k-lock`, property: "--push", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "the push", on: "the picture left", target: `${OLD} .k-lock`, property: "--push", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "its frame", on: "the picture reached", target: `${NEW} .gl-frame`, property: "scale", from: { scale: "behind" }, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: "its frame", on: "the picture left", target: `${OLD} .gl-frame`, property: "scale", from: 1, to: { scale: "behind" }, duration: "long", curve: "emphasized_decel" },
    lightOn("long", true) as Track<Easing>,
    newLamp(true) as Track<Easing>,
  ],
  reduced: [
    lightOffCalm(),
    { ...OLD_LAMP, curve: "standard" },
    { part: WALL, on: ROOM, target: ".gl-floor:first-child .gl-track", property: "--gl-at", from: 1, to: 2, duration: "instant", cue: "short", curve: "standard" },
    lightOnCalm("short"),
    { part: "its lamp", on: "the picture reached", target: `${NEW} .gl-lamp`, property: "--lamp", from: 0, to: 1, duration: "short", cue: "short", curve: "standard" },
  ],
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
  return screen((state: Shown): HTMLElement => {
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
  });
}

/** The clock's editor in `state`. */
const editorCell = screen((state: Shown): HTMLElement => {
  switch (state) {
    case "rest":
      return gallery({ at: 2, view: "lock", edit: {} });
    case "lit":
      return gallery({ at: 2, view: "lock", edit: { lit: true }, tip: "Done" });
    case "open":
      return gallery({ at: 2, view: "lock", edit: {}, shortcuts: true });
    case "disabled":
      return gallery({ at: 4, view: "lock", edit: { look: DATE_OFF } });
    case "loading":
      return gallery({ at: 2, view: "lock", edit: {}, stills: false });
    case "empty":
      return gallery({ at: 2, view: "lock", edit: { look: CLOCK_OFF } });
  }
});

/** The live gallery, shut: the desktop, which a click or Space opens. */
function live(): HTMLElement {
  return gallery({ open: false, live: true });
}

/** The front picture with a name at the toolbar's length. */
const named = screen((words: string) => gallery({ at: 0, name: words }));

export default page<Shown>({
  level: "shell",
  slug: "wallpaper",
  title: "Wallpaper picker",
  lede: "A gallery: every picture on a wall, each drawn as itself, your desktop or your lock, the one in front lit.",
  code: { unbuilt: "QML WallpaperPicker: not built yet" },
  from: ["lock", "depth-picture", "tabs", "button", "button-bar", "icon-button", "chip", "switch", "slider", "colour-picker", "panel-page", "list", "empty-state", "shortcuts", "tooltip"],
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
    { name: "Lock, clock off", for: "the picture alone: no clock, no date", sample: still({ at: 2, view: "lock", looks: setOnTower(CLOCK_OFF) }) },
    { name: "Lock, date off", for: "the digits alone", sample: still({ at: 2, view: "lock", looks: setOnTower(DATE_OFF) }) },
    { name: "Clock editor", for: "E: the panel beside the lock at rest, unpushed", sample: editing() },
    { name: "Editor, dragging", for: "the guides and the room it may move in", sample: editing({ dragging: true }) },
    { name: "Editor, portrait screen", for: "the portrait shape's own values", sample: editing({ tab: "portrait" }) },
    { name: "Editor, a look set", for: "a custom colour, a dark glow, the date left", sample: editing({ look: MOVED }) },
    { name: "Editor, clock off", for: "its rows disabled, no box on the picture", sample: editing({ look: CLOCK_OFF }) },
    { name: "Editor, date off", for: "its sub-rows disabled, the box round the digits", sample: editing({ look: DATE_OFF }) },
    { name: "No depth layers", for: "Depth disabled, with why", sample: screen(() => gallery({ at: 4, view: "lock", edit: {} })) },
    { name: "Full screen, drawer closed", for: "a strip of the panel at the edge", sample: editing({ full: true, drawer: "closed" }) },
    { name: "Full screen, drawer open", for: "cut flush by the edge, over the bar", sample: editing({ full: true, drawer: "open" }) },
    { name: "Full screen, floating", for: "ripped off the edge, anywhere", sample: editing({ full: true, drawer: "float" }) },
  ],
  sizes: doesntApply("as large as its screen; the wall is shares of it (`[scale]`)."),
  states: {
    shown: ["rest", "lit", "open", "disabled", "loading", "empty"],
    absent: {
      pressed: "its buttons press as their own pages say",
      selected: "the applied picture is marked Current at rest",
      error: "a picture it can't read isn't hung",
    },
    rows: nonEmpty([
      ...(["picture", "desktop", "lock"] as const).map((view) => ({ name: view === "picture" ? "Picture" : view === "desktop" ? "Desktop" : "Lock", cell: cell(view) })),
      { name: "Clock editor", cell: editorCell },
    ]),
    looks: {
      rest: "the front one lit, its lamp burning",
      lit: "a control's name and key over it",
      open: "the shortcuts over their button",
      disabled: "no cut-out: depth off, with why; editing, date off",
      loading: "the windows' stills not landed yet",
      empty: "an `EmptyState` in the frame; editing, clock off",
    },
  },
  focus: {
    tubes: [
      { name: "The lamp over the front picture", tone: "accent", filled: true, content: false },
      { name: "Its controls", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the lamp's core burns, its glow and its light on the wall", signsOff: "the lamp as glass: no glow, no light" },
    signsOff: screen(() => gallery({ at: 2, signsOff: true })),
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
    between: [BROWSES, TO_DESKTOP, LOCKS, UNLOCKS, TO_LOCK, WAKES, FULL, FLATTENS, EDIT_OPENS, EDIT_CLOSES, SHAPE, EDIT_FULL, DRAWER, DOCKS, RESETS],
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
      { keys: ["E"], does: "in Lock: the clock's editor" },
      { keys: ["j", "k"], does: "editing: the setting below or above, switches included" },
      { keys: ["h", "l"], does: "editing: less or more; Shift a fine step" },
      { keys: ["Space"], does: "editing, on a switch: Clock or Date on or off" },
      { keys: ["R"], does: "editing: the standard place, both switches on" },
      { keys: ["D"], does: "editing: in front, or back to its last step" },
      { keys: ["Alt"], does: "dragging the clock: no sticking to the guides" },
      { keys: ["Wheel"], does: "on the clock's box: its size; Shift a fine step" },
    ],
    order: "Tab: shortcuts, views, Full screen, Cancel, Wake, Clock, Apply; editing, the panel's rows.",
    lands: "Opening, on the gallery; the editor, on Across; leaving it, the gallery.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The wall", for: "pictures, gaps, lamps, placard, floors", sample: still({ at: 2 }) },
      { name: "The bar, full screen", for: "the bar's parts and the name", sample: still({ at: 2, view: "desktop", full: true }) },
      { name: "The clock's editor", for: "the frame, the panel, the box", sample: editing() },
      { name: "The drawer", for: "docked open at the edge", sample: editing({ full: true, drawer: "open" }) },
    ]),
    measures: [
      { part: "Picture's height", token: "scale-gallery_picture" },
      { part: "Pictures' top edge", token: "scale-gallery_top" },
      { part: "Between pictures", token: "scale-gallery_gap" },
      { part: "Lamp's width", token: "scale-gallery_lamp" },
      { part: "Lamp's height", token: "space-s4" },
      { part: "Lamp over its picture", token: "space-s32" },
      { part: "Lamp's light, across", token: "scale-gallery_light_width" },
      { part: "Lamp's light, down", token: "scale-gallery_light_height" },
      { part: "Lamp's light at a quarter", token: "scale-gallery_light_falloff" },
      { part: "Lamp's light, lit", token: "opacity-lamp_light" },
      { part: "Front lamp's light, off", token: "opacity-lamp_dark" },
      { part: "Picture → placard", token: "space-s24" },
      { part: "Frame's corner", token: "radius-surface" },
      { part: "A picture stepped back", token: "scale-behind" },
      { part: "City ahead, browsing", token: "space-s24" },
      { part: "Floors' inset", token: "space-s32" },
      { part: "Bar's padding", token: "space-s8" },
      { part: "Bar's gaps", token: "space-s16" },
      { part: "Bar, shortcuts → foot", token: "space-s24" },
      { part: "Tooltip over its control", token: "space-s8" },
      { part: "Editor's frame inset", token: "space-s32" },
      { part: "Panel's width", token: "size-panel" },
      { part: "Panel from the screen's edge", token: "space-s24" },
      { part: "Clock's box round it", token: "space-s16" },
      { part: "Box's corner points", token: "size-check" },
      { part: "Clock's allowed area inset", token: "space-s32" },
      { part: "Awake clock over the sheet", token: "space-s16" },
      { part: "Clock's least size", token: "scale-clock_least" },
      { part: "Clock's largest size", token: "scale-clock_most" },
      { part: "Guides' pull", token: "scale-clock_snap" },
      { part: "Date's largest size", token: "scale-clock_date_most" },
      { part: "Date's least gap", token: "scale-clock_gap_least" },
      { part: "Date's largest gap", token: "scale-clock_gap_most" },
      { part: "Glow's tight ring", token: "scale-clock_glow_edge" },
      { part: "Glow's wide ring", token: "scale-clock_glow" },
      { part: "Closed drawer's strip", token: "space-s12" },
      { part: "Drawer's overhang", token: "space-s24" },
      { part: "Sticky step from the edge", token: "scale-dock_sticky" },
      { part: "Pull to rip or dock", token: "scale-dock_pull" },
      { part: "Drawer's give", token: "scale-dock_rubber" },
    ],
    reflow: { note: "The wall is shares of the screen: the same shape on every screen.", sample: () => gallery({ at: 2, size: windowSize() }) },
  },
});
