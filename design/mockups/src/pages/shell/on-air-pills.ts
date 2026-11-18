// The on-air pills: the sharing pill while the screen is shared and the camera pill while a camera
// is in use, each a pill of its own in its privacy colour, first on the bar's right, and the panel
// each opens. Their parts are `bar-parts.ts`' and `sharing-parts.ts`'.

import type { Shown as ControlShown } from "../../lib/control.ts";
import { hint } from "../../lib/hint.ts";
import { row, stack } from "../../lib/layout.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, screen, type Strike, type Track } from "../../site/page.ts";
import { mutePill } from "./bar-audio-parts.ts";
import { barStrip, bellPill, cameraPill, EVERYDAY, ON_AIR_WORDS, type Share, sharingPill, statusPill } from "./bar-parts.ts";
import { SCREEN_WIDTH } from "./screen-parts.ts";
import { cameraPanel, cameraShot, DISCORD, MEET, NO_ICON, onCorner, type Pick, sharingPanel, sharingShot, shareOf, FIRST } from "./sharing-parts.ts";

const SCREEN_SHARE: Share = shareOf(FIRST, MEET);
const WINDOW_SHARE: Share = shareOf({ kind: "window", screen: 0, window: 1 }, MEET);
const REGION_SHARE: Share = shareOf({ kind: "region", screen: 0, window: 0 }, MEET);
const CAMERA = { who: DISCORD.app };

/** The bar's right end from the on-air pills on: the pills, the mute pill, the bell, status. */
function end(sharing: Share | null = SCREEN_SHARE, camera: boolean = true, state: ControlShown = "rest"): HTMLElement {
  return row("s8", "centre", ...(sharing ? [sharingPill(sharing, state)] : []), ...(camera ? [cameraPill(CAMERA, state)] : []), mutePill({ input: { recording: true } }), bellPill(), statusPill());
}

/** A pill lit, over a room as tall as its Hint and the gap to it, so the bubble stays in the sample. */
function hinted(pill: HTMLElement, words: string): HTMLElement {
  const room = hint(words);
  room.style.visibility = "hidden";
  room.setAttribute("aria-hidden", "true");
  for (const each of [room, ...room.querySelectorAll<HTMLElement | SVGElement>("[data-probe]")]) delete each.dataset.probe;
  return stack("s8", "start", pill, room);
}

/**
 * The stage: the screen's corner with both pills; a click on one opens its panel and on it again
 * shuts it. In the sharing panel a pick switches the share at once, the pill's symbol with it.
 */
function live(): HTMLElement {
  const holder = document.createElement("span");
  holder.style.display = "contents";
  let pick: Pick = FIRST;
  let open: "sharing" | "camera" | null = null;
  const draw = () => {
    const share = shareOf(pick, MEET);
    const sharing = sharingPill(share, open === "sharing" ? "lit" : "rest");
    const camera = cameraPill(CAMERA, open === "camera" ? "lit" : "rest");
    sharing.addEventListener("click", () => toggle("sharing"));
    camera.addEventListener("click", () => toggle("camera"));
    const bar = [sharing, camera, mutePill({ input: { recording: true } }), bellPill(), statusPill()];
    const panel =
      open === "sharing"
        ? sharingPanel({
            pick,
            onPick: (next) => {
              pick = next;
              const fresh = sharingPill(shareOf(pick, MEET), "lit");
              fresh.addEventListener("click", () => toggle("sharing"));
              holder.querySelector(".shell-on-air[data-on-air=sharing]")?.replaceWith(fresh);
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

/** A pill drawn with its room shut, so its arrival opens it: its width eased from nothing to a circle's. */
function arrivingPill(make: () => HTMLElement): () => HTMLElement {
  return () => {
    const made = make();
    made.style.setProperty("--from", "0px");
    made.style.setProperty("--to", "var(--size-control_sm)");
    return row("s8", "centre", made, mutePill({ input: { recording: true } }), bellPill(), statusPill());
  };
}

/** The room opening, the pill striking on, then lit for a beat so the eye finds it, and back to rest. */
function arrival(power: Strike | Easing, reduced: boolean): NonEmpty<Track> {
  if (reduced) {
    return [
      { part: "the pill", on: "the pill", target: PILL, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
      { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
      { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 1, to: 0, duration: "short", cue: ["short", "scene"], curve: "standard" },
    ];
  }
  return [
    { part: "the room", on: "the pill", target: PILL, property: "--grown", from: 0, to: 1, duration: "medium", curve: "standard_decel" },
    { part: "the power", on: "the pill", target: PILL, property: "opacity", from: 0, to: 1, duration: "medium", cue: "medium", curve: power },
    { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 0, to: 1, duration: "short", cue: ["medium", "medium"], curve: "standard" },
    { part: "the beat lit", on: "the pill", target: PILL, property: "--lit", from: 1, to: 0, duration: "long", cue: ["medium", "medium", "short", "scene"], curve: "standard" },
  ];
}

function departure(power: Strike | Easing, reduced: boolean): NonEmpty<Track> {
  if (reduced) return [{ part: "the pill", on: "the pill", target: PILL, property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }];
  return [
    { part: "the power", on: "the pill", target: PILL, property: "opacity", from: 1, to: 0, duration: "medium", curve: power },
    { part: "the room", on: "the pill", target: PILL, property: "--grown", from: 1, to: 0, duration: "medium", cue: "medium", curve: "standard_accel" },
  ];
}

/** Only easings: what reduced motion and the flicker off play. */
const eased = (tracks: NonEmpty<Track>) => tracks as NonEmpty<Track<Easing>>;

function arrives(name: string, what: string, make: () => HTMLElement): Move {
  return {
    name,
    what,
    tracks: arrival("neon.strike", false),
    reduced: eased(arrival("standard", true)),
    unflickered: eased(arrival("standard", false)),
    back: "cuts",
    sample: arrivingPill(make),
  };
}

function leaves(name: string, make: () => HTMLElement): Move {
  return {
    name,
    what: "Stopped or closed: the pill flickers out, then its room closes; the pills beyond it stay put.",
    tracks: departure("neon.strike.reversed", false),
    reduced: eased(departure("standard", true)),
    unflickered: eased(departure("standard", false)),
    back: "cuts",
    sample: () => {
      const made = make();
      made.style.setProperty("--from", "0px");
      made.style.setProperty("--to", "var(--size-control_sm)");
      return row("s8", "centre", made, mutePill({ input: { recording: true } }), bellPill(), statusPill());
    },
  };
}

type State = "rest" | "lit" | "pressed" | "open";

const LONG_WINDOW: Share = { kind: "window", what: "Visual Studio Code - Insiders's window", who: "Chromium" };

export default page<State>({
  level: "shell",
  slug: "on-air-pills",
  title: "Sharing and camera pills",
  lede: "A pill each while the screen is shared or a camera is on, and the panel each opens.",
  code: { qml: "SharingPill, CameraPill", unbuilt: "Not built yet" },
  from: ["pill", "panel-page", "card", "tabs", "picture-choice", "media", "list", "button"],
  stage: { live, hint: "click a pill: its panel; pick in it to switch", daylight: true },
  variants: [
    { name: "Sharing a screen", for: "the screen's symbol on the violet tint", sample: () => sharingPill(SCREEN_SHARE) },
    { name: "Sharing a window", for: "the window's symbol", sample: () => sharingPill(WINDOW_SHARE) },
    { name: "Sharing a region", for: "the region's symbol", sample: () => sharingPill(REGION_SHARE) },
    { name: "A camera in use", for: "the camera on the camera's red tint", sample: () => cameraPill(CAMERA) },
    { name: "Both, in order", for: "sharing first, then the camera", sample: () => end() },
    { name: "The sharing pill's hint", for: "what and with whom", sample: () => hinted(sharingPill(SCREEN_SHARE, "lit"), ON_AIR_WORDS.sharing(SCREEN_SHARE.what, SCREEN_SHARE.who)) },
    { name: "The camera pill's hint", for: "the app that has it", sample: () => hinted(cameraPill(CAMERA, "lit"), ON_AIR_WORDS.camera(CAMERA.who)) },
    { name: "On the bar", for: "first on the right, nearest the clock", sample: screen(() => barStrip({ width: SCREEN_WIDTH, content: { ...EVERYDAY, sharing: SCREEN_SHARE, camera: CAMERA } })) },
  ],
  sizes: [
    { name: "A pill", for: "one size: a circle a pill tall", sample: () => sharingPill(SCREEN_SHARE) },
    { name: "A panel", for: "`size.panel` wide, as every panel", sample: screen(() => sharingShot()) },
  ],
  groups: [
    {
      title: "The sharing panel",
      rows: [
        { name: "A screen shared", for: "what, held notifications, the choice, Stop", sample: screen(() => sharingShot()) },
        { name: "A window shared", for: "the choice on Window, its row ticked", sample: screen(() => sharingShot({ pick: { kind: "window", window: 1 } })) },
        { name: "The keyboard in it", for: "the pick lit; arrows switch the share", sample: screen(() => sharingShot({ keyboard: true })) },
        { name: "Alone, no camera", for: "only the sharing pill in the bar", sample: screen(() => sharingShot({}, false)) },
      ],
    },
    {
      title: "The camera panel",
      rows: [
        { name: "A camera in use", for: "preview, the app, quality, Close", sample: screen(() => cameraShot()) },
        { name: "720p at 60 fps", for: "the modes the camera offers", sample: screen(() => cameraShot({ resolution: 1, rate: 1 })) },
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
      { name: "Sharing pill", cell: (state: State) => (state === "open" ? sharingShot() : sharingPill(SCREEN_SHARE, state)) },
      { name: "Camera pill", cell: (state: State) => (state === "open" ? cameraShot() : cameraPill(CAMERA, state)) },
    ],
    looks: {
      rest: "its symbol in its colour on its tint",
      lit: "pointer: its tube in its colour, glow at night, hint",
      pressed: "the tube white-hot",
      open: "its panel under it, the pill held lit",
    },
  },
  focus: {
    tubes: [
      { name: "Sharing pill", tone: "sharing", filled: false, content: false },
      { name: "Camera pill", tone: "recording", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the edge, the glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => (index === 0 ? sharingPill(SCREEN_SHARE, "signs-off") : cameraPill(CAMERA, "signs-off")),
  },
  content: {
    text: [
      {
        name: "What is shared, in the hint",
        words: { short: SCREEN_SHARE.what, long: LONG_WINDOW.what },
        sample: (words: string) => hinted(sharingPill({ ...SCREEN_SHARE, what: words }, "lit"), ON_AIR_WORDS.sharing(words, SCREEN_SHARE.who)),
      },
    ],
    missingPicture: [
      { name: "An app with no icon", for: "its initial on the camera panel's row", sample: screen(() => cameraShot({ who: NO_ICON })) },
      { name: "The camera starting", for: "Media's placeholder until a frame comes", sample: screen(() => cameraShot({ picture: null })) },
    ],
    counts: [
      { name: "Nothing on air", for: "neither pill", sample: () => end(null, false) },
      { name: "A share", for: "the sharing pill alone", sample: () => end(SCREEN_SHARE, false) },
      { name: "A share and a camera", for: "both, sharing first", sample: () => end() },
    ],
  },
  motion: {
    arrives: [
      arrives("The sharing pill arrives", "Its room opens, it strikes on, then burns lit for a beat so the eye finds it.", () => sharingPill(SCREEN_SHARE)),
      arrives("The camera pill arrives", "The same: room, strike, a beat lit, then rest on its red tint.", () => cameraPill(CAMERA)),
    ],
    leaves: [leaves("The sharing pill leaves", () => sharingPill(SCREEN_SHARE)), leaves("The camera pill leaves", () => cameraPill(CAMERA))],
    between: [
      { name: "Lit", what: "The pointer onto it: it lights over `short` on `standard`.", tracks: lights(PILL, "both pills", 0, 1), reduced: "same", back: "cuts", sample: () => end() },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot; its panel opens.", tracks: press(PILL, "both pills"), reduced: "same", back: "plays", sample: () => end(SCREEN_SHARE, true, "lit") },
      ...powerMoves(() => end(SCREEN_SHARE, true, "lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["←", "→"], does: "in the sharing panel's pane: switches the share at once" },
      { keys: ["Tab", "Shift+Tab"], does: "walks the open panel's controls" },
      { keys: ["Enter", "Space"], does: "on Stop sharing or Close: does it" },
      { keys: ["Esc"], does: "closes the panel" },
    ],
    order: "The pills take no Tab; an open panel holds the keyboard until it closes.",
    lands: "The sharing panel, in its pane on the share; the camera panel, on Resolution.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Both pills", for: "circles, their gap", sample: () => end() },
      { name: "The sharing panel", for: "card, choice, footer", sample: screen(() => sharingShot()) },
      { name: "The camera panel", for: "preview, row, quality, footer", sample: screen(() => cameraShot()) },
    ]),
    measures: [
      { part: "Pill's height", token: "size-control_sm" },
      { part: "Pill's ends, a lone symbol", token: "space-s8" },
      { part: "Symbol", token: "icon-s16" },
      { part: "Between pills", token: "space-s8" },
      { part: "Panel's width", token: "size-panel" },
      { part: "Between the panel's parts", token: "space-s16" },
      { part: "Label to the choice", token: "space-s8" },
      { part: "Quality's label to Tabs", token: "space-s8" },
    ],
    reflow: doesntApply("a pill keeps its circle and never gives way; the panels scroll on a short screen."),
  },
});
