// The on-air pills: the sharing pill while the screen is shared and the camera pill while a camera
// is in use, each a pill of its own on its meaning's container, first on the bar's right, a count
// in it past one, and the panel each opens, a card for each share or camera. Their parts are
// `bar-parts.ts`' and `sharing-parts.ts`'.

import type { Shown as ControlShown } from "../../lib/control.ts";
import { hint } from "../../lib/hint.ts";
import { row, stack } from "../../lib/layout.ts";
import { pillChanging } from "../../lib/pill.ts";
import { lights, pillSwap, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Strike, type Track } from "../../site/page.ts";
import { mutePill } from "./bar-audio-parts.ts";
import { barStrip, bellPill, type CameraUse, cameraPill, EVERYDAY, ON_AIR_WORDS, onAirOptions, type Share, sharingContent, sharingPill, statusPill } from "./bar-parts.ts";
import { SCREEN_WIDTH } from "./screen-parts.ts";
import { CAMERAS, cameraPanel, cameraShot, NO_ICON, onCorner, SHARES, sharingPanel, sharingShot } from "./sharing-parts.ts";

const CAMERA: CameraUse = CAMERAS.first.use;
const TWO_SHARES: readonly Share[] = [SHARES.screen, SHARES.second];
const TWO_CAMERAS: readonly CameraUse[] = [CAMERAS.first.use, CAMERAS.second.use];

/** The bar's right end from the on-air pills on: the pills, the mute pill, the bell, status. */
function end(sharing: Share | readonly Share[] | null = SHARES.screen, camera: CameraUse | readonly CameraUse[] | null = CAMERA, state: ControlShown = "rest"): HTMLElement {
  return row("s8", "centre", ...(sharing ? [sharingPill(sharing, state)] : []), ...(camera ? [cameraPill(camera, state)] : []), mutePill({ input: { recording: true } }), bellPill(), statusPill());
}

/** A pill lit, over a room as tall as its Hint and the gap to it, so the bubble stays in the sample. */
function hinted(pill: HTMLElement, words: string): HTMLElement {
  const room = hint(words);
  room.style.visibility = "hidden";
  room.setAttribute("aria-hidden", "true");
  for (const each of [room, ...room.querySelectorAll<HTMLElement | SVGElement>("[data-probe]")]) delete each.dataset.probe;
  return stack("s8", "start", pill, room);
}

/** The bar along a whole screen, both pills first on its right. */
const onTheBar = (sharing: Share | readonly Share[] = SHARES.screen, camera: CameraUse | readonly CameraUse[] = CAMERA) =>
  barStrip({ width: SCREEN_WIDTH, content: { ...EVERYDAY, sharing, camera } });

/**
 * The stage: the screen's corner with both pills, two shares running; a click on a pill opens its
 * panel and on it again shuts it. Change what's shared goes to picking's page; Stop sharing ends
 * that share, the pill's count with it, and the pill goes with the last.
 */
function live(): HTMLElement {
  const holder = document.createElement("span");
  // A box of its own, as wide as what it holds: the stage measures a sample by its box, and one
  // with none (`contents`) reads as reaching from the page's edge.
  holder.style.display = "inline-block";
  let shares: readonly Share[] = TWO_SHARES;
  let open: "sharing" | "camera" | null = "sharing";
  const draw = () => {
    const bar: Element[] = [];
    if (shares.length > 0) {
      const sharing = sharingPill(shares);
      sharing.addEventListener("click", () => toggle("sharing"));
      bar.push(sharing);
    }
    const camera = cameraPill(CAMERA);
    camera.addEventListener("click", () => toggle("camera"));
    bar.push(camera, mutePill({ input: { recording: true } }), bellPill(), statusPill());
    const panel =
      open === "sharing" && shares.length > 0
        ? sharingPanel({
            shares,
            onChange: () => (location.hash = "#/shell/share-picker"),
            onStop: (index) => {
              shares = shares.filter((_, at) => at !== index);
              draw();
            },
          })
        : open === "camera"
          ? cameraPanel()
          : null;
    holder.replaceChildren(onCorner(panel ?? document.createElement("span"), bar, panel ? undefined : 0));
  };
  const toggle = (which: "sharing" | "camera") => {
    open = open === which ? null : which;
    draw();
  };
  draw();
  return holder;
}

const PILL = ".shell-on-air";

/** A pill beside the bar's right end, as it stands while it arrives or leaves. */
function besideEnd(make: () => HTMLElement): () => HTMLElement {
  return () => row("s8", "centre", make(), mutePill({ input: { recording: true } }), bellPill(), statusPill());
}

/**
 * The pill striking on where it stands, then lit for a beat so the eye finds it, and back to rest.
 * The bar's right side is laid from its end, so the pill takes its room at once and moves no other.
 */
function arrival(power: Strike | Easing, reduced: boolean): NonEmpty<Track> {
  if (reduced) {
    return [
      { part: "the pill", on: "the pill", target: PILL, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
      { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
      { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 1, to: 0, duration: "short", cue: ["short", "scene"], curve: "standard" },
    ];
  }
  return [
    { part: "the power", on: "the pill", target: PILL, property: "opacity", from: 0, to: 1, duration: "medium", curve: power },
    { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
    { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 1, to: 0, duration: "long", cue: ["medium", "short", "scene"], curve: "standard" },
  ];
}

function departure(power: Strike | Easing, reduced: boolean): NonEmpty<Track> {
  return [{ part: "the power", on: "the pill", target: PILL, property: "opacity", from: 1, to: 0, duration: reduced ? "short" : "medium", curve: power }];
}

/** Only easings: what reduced motion and the flicker off play. */
const eased = (tracks: NonEmpty<Track>) => tracks as NonEmpty<Track<Easing>>;

function arrives(name: string, what: string, make: () => HTMLElement): Move {
  return { name, what, tracks: arrival("neon.strike", false), reduced: eased(arrival("standard", true)), unflickered: eased(arrival("standard", false)), back: "cuts", sample: besideEnd(make) };
}

function leaves(name: string, make: () => HTMLElement): Move {
  return {
    name,
    what: "Stopped or closed: the pill flickers out; the pills beyond it stay put.",
    tracks: departure("neon.strike.reversed", false),
    reduced: eased(departure("standard", true)),
    unflickered: eased(departure("standard", false)),
    back: "cuts",
    sample: besideEnd(make),
  };
}

/** A second share: the symbol stays solid and the count fades in, in its own room, as any pill's content changes. */
const COUNTED: Move = {
  name: "A second share",
  what: "The symbol stays; the count fades in as its room opens, over `medium`.",
  ...pillSwap("the sharing pill", "in"),
  back: "plays",
  sample: () => pillChanging({ ...onAirOptions("sharing"), interactive: true }, sharingContent(SHARES.screen), sharingContent(TWO_SHARES)),
};

type State = "rest" | "lit" | "pressed" | "open";

const LONG_WINDOW: Share = { kind: "window", what: "Visual Studio Code - Insiders's window", who: "Chromium" };

export default page<State>({
  level: "shell",
  slug: "on-air-pills",
  title: "Sharing and camera pills",
  lede: "A pill each while the screen is shared or a camera is on, and the panel each opens.",
  code: { qml: "SharingPill, CameraPill", unbuilt: "Not built yet" },
  from: ["pill", "panel-page", "card", "tabs", "media", "list", "button"],
  stage: { live, hint: "click a pill: its panel; Stop sharing ends one share", daylight: true },
  variants: [
    { name: "Sharing a screen", for: "the screen's symbol on `sharing_container`", sample: () => sharingPill(SHARES.screen) },
    { name: "Sharing a window", for: "the window's symbol", sample: () => sharingPill(SHARES.window) },
    { name: "Sharing a region", for: "the region's symbol", sample: () => sharingPill(SHARES.region) },
    { name: "A camera in use", for: "the camera on `error_container`, as a recording", sample: () => cameraPill(CAMERA) },
    { name: "Two shares", for: "the symbol and the count", sample: () => sharingPill(TWO_SHARES) },
    { name: "Two cameras", for: "the camera and the count", sample: () => cameraPill(TWO_CAMERAS) },
    { name: "Both, in order", for: "sharing first, then the camera", sample: () => end() },
    { name: "The sharing pill's hint", for: "what and with whom", sample: () => hinted(sharingPill(SHARES.screen, "lit"), ON_AIR_WORDS.sharing(SHARES.screen.what, SHARES.screen.who)) },
    { name: "The camera pill's hint", for: "the app that has it", sample: () => hinted(cameraPill(CAMERA, "lit"), ON_AIR_WORDS.camera(CAMERA.who)) },
    { name: "On the bar", for: "first on the right, nearest the clock", sample: screen(() => onTheBar()) },
    { name: "On the bar, counted", for: "two shares and two cameras", sample: screen(() => onTheBar(TWO_SHARES, TWO_CAMERAS)) },
  ],
  sizes: [
    { name: "A pill, one", for: "a circle a pill tall", sample: () => sharingPill(SHARES.screen) },
    { name: "A pill, counted", for: "symbol and count, `space.s8` at the ends still", sample: () => sharingPill(TWO_SHARES) },
    { name: "A panel", for: "`size.panel` wide, as every panel", sample: screen(() => sharingShot()) },
  ],
  groups: [
    {
      title: "The sharing panel",
      rows: [
        { name: "A screen shared", for: "a card: what, with whom, Change, Stop", sample: screen(() => sharingShot()) },
        { name: "Two shares", for: "a card each, each with its own Stop", sample: screen(() => sharingShot({ shares: TWO_SHARES })) },
        { name: "Alone, no camera", for: "only the sharing pill in the bar", sample: screen(() => sharingShot({}, false)) },
      ],
    },
    {
      title: "The camera panel",
      rows: [
        { name: "A camera in use", for: "preview, the app, quality, Close", sample: screen(() => cameraShot()) },
        { name: "Two cameras", for: "a card each, each with its own Close", sample: screen(() => cameraShot({ cameras: [CAMERAS.first, CAMERAS.second] })) },
        { name: "Quality", for: "the camera's modes; whether the app follows is the build's to find", sample: screen(() => cameraShot({ cameras: [{ ...CAMERAS.first, resolution: 1, rate: 1 }] })) },
        { name: "Alone, no share", for: "only the camera pill in the bar", sample: screen(() => cameraShot({}, false)) },
      ],
    },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "open"],
    absent: {
      disabled: "shown only while on air",
      selected: "nothing to pick in the pill",
      loading: "the preview holds Media's placeholder as the camera starts",
      empty: "nothing on air: no pill",
      error: "a share that fails ends; its notification says so",
    },
    rows: [
      { name: "Sharing pill", cell: (state: State) => (state === "open" ? sharingShot() : sharingPill(SHARES.screen, state)) },
      { name: "Camera pill", cell: (state: State) => (state === "open" ? cameraShot() : cameraPill(CAMERA, state)) },
    ],
    looks: {
      rest: "its symbol in its ink on its container",
      lit: "pointer: its tube in its colour, glow at night, hint",
      pressed: "the tube white-hot",
      open: "its panel under it; the pill rests, its hint off the title",
    },
  },
  focus: {
    tubes: [
      { name: "Sharing pill", tone: "sharing", filled: false, content: false },
      { name: "Camera pill", tone: "recording", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the edge, the glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => (index === 0 ? sharingPill(SHARES.screen, "signs-off") : cameraPill(CAMERA, "signs-off")),
  },
  content: {
    text: [
      {
        name: "What is shared, in the hint",
        words: { short: SHARES.screen.what, long: LONG_WINDOW.what },
        sample: (words: string) => hinted(sharingPill({ ...SHARES.screen, what: words }, "lit"), ON_AIR_WORDS.sharing(words, SHARES.screen.who)),
      },
    ],
    missingPicture: [
      { name: "An app with no icon", for: "its initial on the camera card's row", sample: screen(() => cameraShot({ cameras: [{ ...CAMERAS.first, asker: NO_ICON }] })) },
      { name: "The camera starting", for: "Media's placeholder until a frame comes", sample: screen(() => cameraShot({ cameras: [{ ...CAMERAS.first, picture: null }] })) },
    ],
    counts: [
      { name: "Nothing on air", for: "neither pill", sample: () => end(null, null) },
      { name: "A share", for: "the sharing pill alone", sample: () => end(SHARES.screen, null) },
      { name: "A share and a camera", for: "both, sharing first", sample: () => end() },
      { name: "Two of each", for: "each pill with its count", sample: () => end(TWO_SHARES, TWO_CAMERAS) },
    ],
  },
  motion: {
    arrives: [
      arrives("The sharing pill arrives", "Its room opens, it strikes on, then burns lit for a beat so the eye finds it.", () => sharingPill(SHARES.screen)),
      arrives("The camera pill arrives", "The same: room, strike, a beat lit, then rest on its red.", () => cameraPill(CAMERA)),
    ],
    leaves: [leaves("The sharing pill leaves", () => sharingPill(SHARES.screen)), leaves("The camera pill leaves", () => cameraPill(CAMERA))],
    between: [
      COUNTED,
      { name: "Lit", what: "The pointer onto it: it lights over `short` on `standard`.", tracks: lights(PILL, "both pills", 0, 1), reduced: "same", back: "cuts", sample: () => end() },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot; its panel opens.", tracks: press(PILL, "both pills"), reduced: "same", back: "plays", sample: () => end(SHARES.screen, CAMERA, "lit") },
      ...powerMoves(() => end(SHARES.screen, CAMERA, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "walks the open panel's buttons, card by card" },
      { keys: ["Enter", "Space"], does: "on a button: Change, Stop, a mode, Close" },
      { keys: ["←", "→"], does: "on a quality's Tabs: the mode before or after" },
      { keys: ["Esc"], does: "closes the panel" },
    ],
    order: "The pills take no Tab; an open panel holds the keyboard until it closes.",
    lands: "The sharing panel, on the first card's Change; the camera panel, on Resolution.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Both pills, counted", for: "circle, count, gaps", sample: () => end(TWO_SHARES, CAMERA) },
      { name: "The sharing panel, two shares", for: "cards and their actions", sample: screen(() => sharingShot({ shares: TWO_SHARES })) },
      { name: "The camera panel", for: "preview, row, quality, Close", sample: screen(() => cameraShot()) },
    ]),
    measures: [
      { part: "Pill's height", token: "size-control_sm" },
      { part: "Pill's ends, alone or counted", token: "space-s8" },
      { part: "Symbol to count", token: "space-s4" },
      { part: "Panel's width", token: "size-panel" },
      { part: "Between cards", token: "space-s16" },
      { part: "Between a card's buttons", token: "space-s8" },
      { part: "Quality's label to Tabs", token: "space-s8" },
    ],
    reflow: doesntApply("a pill keeps its height and never gives way; the panels scroll on a short screen."),
  },
});
